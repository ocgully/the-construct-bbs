use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    AppState,
    auth::session::create_session,
    connection::ceremony,
    db::user::{find_user_by_id, update_last_login, update_user_field, update_user_time},
    services::{
        ServiceAction, SessionIO,
        goodbye::render_goodbye,
        login::{LoginFlow, LoginResult, render_login_header, render_welcome_back},
        profile::{render_profile_card, render_profile_edit_menu_string},
        registration::{RegistrationFlow, RegistrationResult, render_registration_header},
        welcome_art,
    },
    terminal::{AnsiWriter, Color, Pager, Page, more_prompt, clear_more_prompt},
};

/// Authentication state machine for the session lifecycle.
///
/// Tracks where the user is in the connect -> authenticate -> use BBS flow.
enum AuthState {
    /// Waiting for frontend to send { type: "auth", token } JSON message.
    AwaitingAuth,
    /// Running the connection ceremony (modem sim, splash screen).
    ConnectionCeremony,
    /// In the interactive login flow (handle -> password -> welcome).
    Login(LoginFlow),
    /// In the interactive registration flow (handle -> email -> password -> verify).
    Registration(RegistrationFlow),
    /// Successfully authenticated and using the BBS.
    Authenticated {
        user_id: i64,
        handle: String,
        token: String,
        user_level: String,
        login_time: std::time::Instant,
    },
}

/// Per-connection session state
///
/// Each WebSocket connection gets its own Session that:
/// - Waits for auth token from frontend (AwaitingAuth)
/// - Runs the connection ceremony if no valid token (modem sim, splash screen)
/// - Manages login/registration flows with character-by-character echo
/// - Routes authenticated users to services (main menu vs active service)
/// - Implements SessionIO for service output
/// - Composes ANSI-formatted output using AnsiWriter
/// - Sends output to WebSocket via mpsc channel
/// - Tracks assigned node_id for release on disconnect
pub struct Session {
    tx: mpsc::Sender<String>,
    state: Arc<AppState>,
    auth_state: AuthState,
    current_service: Option<String>,
    output_buffer: AnsiWriter,
    pager: Pager,
    pending_pages: Option<Vec<Page>>,
    page_index: usize,
    pagination_buffer: Option<String>,
    /// Assigned BBS node number (1-based), set during ceremony
    node_id: Option<usize>,
    /// Set to true when session should be closed (e.g. line busy)
    disconnecting: bool,
}

impl Session {
    /// Create a new session
    pub fn new(tx: mpsc::Sender<String>, state: Arc<AppState>) -> Self {
        Self {
            tx,
            state,
            auth_state: AuthState::AwaitingAuth,
            current_service: None,
            output_buffer: AnsiWriter::new(),
            pager: Pager::new(25), // Standard terminal height
            pending_pages: None,
            page_index: 0,
            pagination_buffer: None,
            node_id: None,
            disconnecting: false,
        }
    }

    /// Returns true if the session is in a disconnecting state (line busy, etc.)
    pub fn is_disconnecting(&self) -> bool {
        self.disconnecting
    }

    /// Send the buffered output to the WebSocket
    async fn flush_output(&mut self) {
        // Check if there's paginated content to send
        if let Some(content) = self.pagination_buffer.take() {
            self.send_paginated(&content).await;
            return;
        }

        // Send normal buffered output
        if !self.output_buffer.is_empty() {
            let content = self.output_buffer.flush();
            // Ignore send errors (client disconnected)
            let _ = self.tx.send(content).await;
        }
    }

    /// Send paginated output with [More] prompts
    ///
    /// Splits text into pages and sends first page with [More] prompt.
    /// Subsequent pages are sent when user presses any key.
    pub async fn send_paginated(&mut self, text: &str) {
        let pages = self.pager.paginate(text);

        if pages.is_empty() {
            return;
        }

        // If only one page, send without pagination
        if pages.len() == 1 {
            let _ = self.tx.send(pages[0].to_ansi()).await;
            return;
        }

        // Send first page
        let first_page = &pages[0];
        let mut output = first_page.to_ansi();
        output.push_str("\r\n");
        output.push_str(&more_prompt());
        let _ = self.tx.send(output).await;

        // Store remaining pages for subsequent keypresses
        self.pending_pages = Some(pages);
        self.page_index = 1; // Start at second page
    }

    /// Send the next page in a paginated sequence
    /// Called when user presses any key during pagination
    async fn send_next_page(&mut self) {
        if let Some(pages) = &self.pending_pages {
            if self.page_index < pages.len() {
                let page = &pages[self.page_index];

                // Clear the [More] prompt
                let mut output = clear_more_prompt(80); // Standard terminal width

                // Add the page content
                output.push_str(&page.to_ansi());

                // If not the last page, add another [More] prompt
                if !page.is_last {
                    output.push_str("\r\n");
                    output.push_str(&more_prompt());
                }

                let _ = self.tx.send(output).await;
                self.page_index += 1;

                // If this was the last page, clear pagination state
                if page.is_last {
                    self.pending_pages = None;
                    self.page_index = 0;
                }
            }
        }
    }

    /// Called when client connects -- just sets up state.
    ///
    /// The actual ceremony runs after receiving the auth message from the frontend.
    /// Returns true always (line-busy check happens during ceremony).
    pub async fn on_connect(&mut self) -> bool {
        // Session starts in AwaitingAuth state.
        // The frontend will send { type: "auth", token } immediately on connect.
        // We process that in handle_input, which decides: ceremony+login or resume.
        true
    }

    /// Run the connection ceremony (modem sim, splash, node assignment).
    /// After ceremony completes, transitions to Login state.
    async fn run_ceremony_and_login(&mut self) {
        // Run the connection ceremony
        let ceremony_result = ceremony::run_connection_ceremony(
            &self.tx,
            &self.state.node_manager,
            &self.state.config.connection,
        )
        .await;

        match ceremony_result {
            Ok(node_id) => {
                self.node_id = Some(node_id);
                println!("Session connected on node {}", node_id);

                // Send ANSI splash screen with baud-rate simulation
                ceremony::send_splash_screen(
                    &self.tx,
                    self.state.config.connection.baud_simulation_cps,
                )
                .await;

                // Show login header
                let (active, max) = self.state.node_manager.get_status().await;
                let header = render_login_header(
                    &self.state.config.connection.tagline,
                    active,
                    max,
                );
                let _ = self.tx.send(header).await;

                // Transition to Login state
                let flow = LoginFlow::new();
                self.send_prompt(flow.current_prompt(), false).await;
                self.auth_state = AuthState::Login(flow);
            }
            Err(_) => {
                // All lines busy - session should disconnect
                self.disconnecting = true;
                println!("Session rejected: all lines busy");
            }
        }
    }

    /// Send a prompt with appropriate color formatting.
    /// Uses send_colored_prompt free function to avoid borrow conflicts.
    async fn send_prompt(&mut self, prompt: &str, is_password: bool) {
        send_colored_prompt(&self.tx, prompt, is_password).await;
    }

    /// Show the main menu with user info (handle, level, node).
    async fn show_main_menu(&mut self) {
        let services: Vec<(String, String)> = self.state.registry.list()
            .iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect();

        // Extract user info from auth state for the menu header
        let (handle, user_level) = match &self.auth_state {
            AuthState::Authenticated { handle, user_level, .. } => {
                (handle.clone(), user_level.clone())
            }
            _ => ("Guest".to_string(), "User".to_string()),
        };

        let max_nodes = self.state.config.connection.max_nodes as usize;
        let menu = welcome_art::render_main_menu_with_user(
            &services,
            &handle,
            &user_level,
            self.node_id,
            max_nodes,
        );
        let _ = self.tx.send(menu).await;
    }

    /// Handle user input -- main routing based on auth state
    pub async fn handle_input(&mut self, input: &str) {
        match &self.auth_state {
            AuthState::AwaitingAuth => {
                self.handle_awaiting_auth(input).await;
            }
            AuthState::ConnectionCeremony => {
                // Ignore input during ceremony (it's automated)
            }
            AuthState::Login(_) => {
                self.handle_login_input(input).await;
            }
            AuthState::Registration(_) => {
                self.handle_registration_input(input).await;
            }
            AuthState::Authenticated { .. } => {
                self.handle_authenticated_input(input).await;
            }
        }
    }

    /// Handle input while awaiting auth token from frontend.
    ///
    /// Parses the JSON { type: "auth", token } message.
    /// If token is valid, resumes session. Otherwise, runs ceremony.
    async fn handle_awaiting_auth(&mut self, input: &str) {
        // Try to parse as JSON auth message
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(input) {
            if parsed.get("type").and_then(|t| t.as_str()) == Some("auth") {
                let token = parsed.get("token").and_then(|t| t.as_str());

                if let Some(token_str) = token {
                    // Try to validate session token
                    let pool = &self.state.db_pool;
                    match crate::auth::session::validate_session(pool, token_str).await {
                        Ok(Some(user_id)) => {
                            // Valid token -- resume session
                            match find_user_by_id(pool, user_id).await {
                                Ok(Some(user)) => {
                                    // Check for duplicate session (Issue 10)
                                    if self.state.node_manager.is_user_connected(user.id).await {
                                        let mut w = AnsiWriter::new();
                                        w.set_fg(Color::LightRed);
                                        w.bold();
                                        w.writeln("");
                                        w.writeln("Already connected from another session. Disconnecting...");
                                        w.reset_color();
                                        let _ = self.tx.send(w.flush()).await;
                                        tokio::time::sleep(Duration::from_secs(2)).await;
                                        self.disconnecting = true;
                                        return;
                                    }

                                    // Assign a node for this reconnecting user
                                    match self.state.node_manager.assign_node(
                                        user.id,
                                        user.handle.clone(),
                                    ).await {
                                        Ok(node_id) => {
                                            self.node_id = Some(node_id);
                                            println!(
                                                "Session resumed for {} on node {}",
                                                user.handle, node_id
                                            );
                                        }
                                        Err(_) => {
                                            // All lines busy
                                            self.disconnecting = true;
                                            return;
                                        }
                                    }

                                    // Determine user level
                                    let user_level = if self.state.config.auth.sysop_handles
                                        .iter()
                                        .any(|h| h.eq_ignore_ascii_case(&user.handle))
                                    {
                                        "Sysop".to_string()
                                    } else {
                                        user.user_level.clone()
                                    };

                                    // Send welcome-back message
                                    let welcome = render_welcome_back(
                                        &user.handle,
                                        user.last_login.as_deref(),
                                        user.total_logins,
                                    );
                                    let _ = self.tx.send(welcome).await;

                                    // Set authenticated state
                                    self.auth_state = AuthState::Authenticated {
                                        user_id: user.id,
                                        handle: user.handle.clone(),
                                        token: token_str.to_string(),
                                        user_level,
                                        login_time: std::time::Instant::now(),
                                    };

                                    // Brief pause then show main menu
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    self.show_main_menu().await;
                                    return;
                                }
                                _ => {
                                    // User not found -- token is stale, fall through to ceremony
                                }
                            }
                        }
                        _ => {
                            // Token invalid or expired -- fall through to ceremony
                        }
                    }
                }

                // No valid token -- run ceremony + login
                self.auth_state = AuthState::ConnectionCeremony;
                self.run_ceremony_and_login().await;
                return;
            }
        }

        // Not a valid auth message -- run ceremony anyway
        self.auth_state = AuthState::ConnectionCeremony;
        self.run_ceremony_and_login().await;
    }

    /// Handle character-by-character input during login flow.
    async fn handle_login_input(&mut self, input: &str) {
        // Clone tx for sending without borrowing self
        let tx = self.tx.clone();

        for ch in input.chars() {
            if ch == '\r' || ch == '\n' {
                // Enter pressed: take accumulated input and process
                let (_line, result) = {
                    let flow = match &mut self.auth_state {
                        AuthState::Login(f) => f,
                        _ => return,
                    };
                    let line = flow.take_input();
                    let pool = &self.state.db_pool;
                    let config = &self.state.config.clone();
                    let result = flow.handle_input(&line, pool, config).await;
                    (line, result)
                };
                // flow borrow is now dropped -- we can freely use self

                // Send newline echo
                let _ = tx.send("\r\n".to_string()).await;

                match result {
                    LoginResult::Continue => {
                        self.send_login_prompt().await;
                    }
                    LoginResult::Error(msg) => {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::LightRed);
                        w.writeln(&msg);
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;
                        self.send_login_prompt().await;
                    }
                    LoginResult::Success {
                        user_id,
                        handle,
                        token,
                        user_level,
                        last_login,
                    } => {
                        // Check for duplicate session (Issue 10)
                        if self.state.node_manager.is_user_connected(user_id).await {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightRed);
                            w.bold();
                            w.writeln("");
                            w.writeln("Already connected from another session. Disconnecting...");
                            w.reset_color();
                            let _ = tx.send(w.flush()).await;
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            self.disconnecting = true;
                            return;
                        }

                        // Update node_manager with real user info
                        if let Some(node_id) = self.node_id {
                            self.state.node_manager.release_node(node_id).await;
                            match self.state.node_manager
                                .assign_node(user_id, handle.clone())
                                .await
                            {
                                Ok(new_node_id) => {
                                    self.node_id = Some(new_node_id);
                                }
                                Err(_) => {
                                    self.node_id = None;
                                }
                            }
                        }

                        // Send session token to frontend as JSON via tx channel
                        let token_msg = serde_json::json!({
                            "type": "session",
                            "token": &token
                        });
                        let _ = tx.send(
                            serde_json::to_string(&token_msg).unwrap()
                        ).await;

                        // Look up total_logins for welcome message
                        let total_logins = match find_user_by_id(
                            &self.state.db_pool,
                            user_id,
                        ).await {
                            Ok(Some(u)) => u.total_logins,
                            _ => 0,
                        };

                        // Show welcome-back message
                        let welcome = render_welcome_back(
                            &handle,
                            last_login.as_deref(),
                            total_logins,
                        );
                        let _ = tx.send(welcome).await;

                        // Transition to Authenticated
                        self.auth_state = AuthState::Authenticated {
                            user_id,
                            handle: handle.clone(),
                            token,
                            user_level,
                            login_time: std::time::Instant::now(),
                        };

                        println!("User {} logged in", handle);

                        // Brief pause then show main menu
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        self.show_main_menu().await;
                        return;
                    }
                    LoginResult::SwitchToRegistration => {
                        let header = render_registration_header();
                        let _ = tx.send(header).await;

                        let reg_flow = RegistrationFlow::new();
                        let prompt = reg_flow.current_prompt().to_string();
                        self.auth_state = AuthState::Registration(reg_flow);
                        self.send_prompt(&prompt, false).await;
                        return;
                    }
                    LoginResult::Locked => {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::LightRed);
                        w.bold();
                        w.writeln("");
                        w.writeln("Account locked due to too many failed attempts.");
                        w.writeln("Please try again later.");
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;

                        tokio::time::sleep(Duration::from_secs(2)).await;
                        self.disconnecting = true;
                        return;
                    }
                }
            } else {
                // Regular character: echo it
                let flow = match &mut self.auth_state {
                    AuthState::Login(f) => f,
                    _ => return,
                };
                if let Some(echo) = flow.handle_char(ch) {
                    let _ = tx.send(echo).await;
                }
            }
        }
    }

    /// Helper: send the current login flow prompt (avoids borrow conflicts).
    async fn send_login_prompt(&mut self) {
        let (prompt, is_pw) = match &self.auth_state {
            AuthState::Login(f) => (f.current_prompt().to_string(), f.needs_password_mask()),
            _ => return,
        };
        send_colored_prompt(&self.tx, &prompt, is_pw).await;
    }

    /// Handle character-by-character input during registration flow.
    async fn handle_registration_input(&mut self, input: &str) {
        let tx = self.tx.clone();

        for ch in input.chars() {
            if ch == '\r' || ch == '\n' {
                let result = {
                    let flow = match &mut self.auth_state {
                        AuthState::Registration(f) => f,
                        _ => return,
                    };
                    let line = flow.take_input();
                    let pool = &self.state.db_pool;
                    let config = &self.state.config.clone();
                    flow.handle_input(&line, pool, config).await
                };
                // flow borrow is now dropped

                let _ = tx.send("\r\n".to_string()).await;

                match result {
                    RegistrationResult::Continue => {
                        self.send_registration_prompt().await;
                    }
                    RegistrationResult::Error(msg) => {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::LightRed);
                        w.writeln(&msg);
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;
                        self.send_registration_prompt().await;
                    }
                    RegistrationResult::Message(msg) => {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::Yellow);
                        w.writeln(&msg);
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;
                        self.send_registration_prompt().await;
                    }
                    RegistrationResult::Complete(user_id) => {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::LightGreen);
                        w.bold();
                        w.writeln("");
                        w.writeln("Registration complete! Logging you in...");
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;

                        tokio::time::sleep(Duration::from_secs(1)).await;

                        // Auto-login: look up user, create session, authenticate
                        let pool = &self.state.db_pool;
                        match find_user_by_id(pool, user_id).await {
                            Ok(Some(user)) => {
                                // Update last_login
                                let _ = update_last_login(pool, user_id).await;

                                // Create session token
                                let token = match create_session(
                                    pool,
                                    user_id,
                                    self.node_id.map(|n| n as i32),
                                    self.state.config.auth.session_duration_hours,
                                ).await {
                                    Ok(t) => t,
                                    Err(e) => {
                                        let mut w = AnsiWriter::new();
                                        w.set_fg(Color::LightRed);
                                        w.writeln(&format!("Session error: {}", e));
                                        w.reset_color();
                                        let _ = tx.send(w.flush()).await;
                                        // Fall back to login flow
                                        let login_flow = LoginFlow::new();
                                        let prompt = login_flow.current_prompt().to_string();
                                        self.auth_state = AuthState::Login(login_flow);
                                        self.send_prompt(&prompt, false).await;
                                        return;
                                    }
                                };

                                // Send session token to frontend as JSON
                                let token_msg = serde_json::json!({
                                    "type": "session",
                                    "token": &token
                                });
                                let _ = tx.send(
                                    serde_json::to_string(&token_msg).unwrap()
                                ).await;

                                // Update node_manager with real user info
                                if let Some(node_id) = self.node_id {
                                    self.state.node_manager.release_node(node_id).await;
                                    match self.state.node_manager
                                        .assign_node(user_id, user.handle.clone())
                                        .await
                                    {
                                        Ok(new_node_id) => {
                                            self.node_id = Some(new_node_id);
                                        }
                                        Err(_) => {
                                            self.node_id = None;
                                        }
                                    }
                                }

                                // Determine user level
                                let user_level = if self.state.config.auth.sysop_handles
                                    .iter()
                                    .any(|h| h.eq_ignore_ascii_case(&user.handle))
                                {
                                    "Sysop".to_string()
                                } else {
                                    user.user_level.clone()
                                };

                                // Show welcome message
                                let welcome = render_welcome_back(
                                    &user.handle,
                                    user.last_login.as_deref(),
                                    1, // First login
                                );
                                let _ = tx.send(welcome).await;

                                // Transition to Authenticated
                                self.auth_state = AuthState::Authenticated {
                                    user_id,
                                    handle: user.handle.clone(),
                                    token,
                                    user_level,
                                    login_time: std::time::Instant::now(),
                                };

                                println!("User {} auto-logged in after registration", user.handle);

                                // Brief pause then show main menu
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                self.show_main_menu().await;
                            }
                            _ => {
                                // Fallback: send to login flow
                                let mut w = AnsiWriter::new();
                                w.set_fg(Color::LightRed);
                                w.writeln("Error loading user. Please log in manually.");
                                w.reset_color();
                                let _ = tx.send(w.flush()).await;

                                let login_flow = LoginFlow::new();
                                let prompt = login_flow.current_prompt().to_string();
                                self.auth_state = AuthState::Login(login_flow);
                                self.send_prompt(&prompt, false).await;
                            }
                        }
                        return;
                    }
                    RegistrationResult::Failed(msg) => {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::LightRed);
                        w.writeln(&msg);
                        w.writeln("");
                        w.writeln("Returning to login...");
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;

                        tokio::time::sleep(Duration::from_secs(1)).await;

                        let login_flow = LoginFlow::new();
                        let prompt = login_flow.current_prompt().to_string();
                        self.auth_state = AuthState::Login(login_flow);
                        self.send_prompt(&prompt, false).await;
                        return;
                    }
                }
            } else {
                let flow = match &mut self.auth_state {
                    AuthState::Registration(f) => f,
                    _ => return,
                };
                if let Some(echo) = flow.handle_char(ch) {
                    let _ = tx.send(echo).await;
                }
            }
        }
    }

    /// Helper: send the current registration flow prompt (avoids borrow conflicts).
    async fn send_registration_prompt(&mut self) {
        let (prompt, is_pw) = match &self.auth_state {
            AuthState::Registration(f) => (f.current_prompt().to_string(), f.needs_password_mask()),
            _ => return,
        };
        send_colored_prompt(&self.tx, &prompt, is_pw).await;
    }

    /// Handle input when user is authenticated (services, main menu).
    async fn handle_authenticated_input(&mut self, input: &str) {
        let trimmed = input.trim();

        // Ignore empty input and escape sequences (e.g. from mouse scroll)
        if trimmed.is_empty() || trimmed.starts_with('\x1b') {
            return;
        }

        // If we're in paging mode, any keypress advances to next page
        if self.pending_pages.is_some() {
            self.send_next_page().await;
            return;
        }

        if let Some(service_name) = &self.current_service {
            // Profile edit mode: accumulate input, save on Enter
            if service_name.starts_with("__profile_edit_") {
                self.handle_profile_edit_input(input).await;
                return;
            }

            // Profile menu: handle 1/2/3/4/q
            if service_name == "__profile__" {
                self.handle_profile_menu_input(input).await;
                return;
            }

            // Currently in a service - route input to it
            let service = self.state.registry.get(service_name).cloned();
            if let Some(service) = service {
                match service.handle_input(self, trimmed) {
                    Ok(ServiceAction::Continue) => {
                        // Service handled input, continue
                    }
                    Ok(ServiceAction::Exit) => {
                        // Service wants to exit
                        service.on_exit(self);
                        self.current_service = None;
                        self.flush_output().await;

                        // Return to main menu
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        self.show_main_menu().await;
                    }
                    Err(e) => {
                        // Service error
                        self.output_buffer.set_fg(Color::LightRed);
                        self.output_buffer.writeln(&format!("Error: {}", e));
                        self.output_buffer.reset_color();
                        self.flush_output().await;
                    }
                }
            } else {
                // Service disappeared from registry
                self.output_buffer.set_fg(Color::LightRed);
                self.output_buffer.writeln("Error: Service no longer available");
                self.output_buffer.reset_color();
                self.current_service = None;
                self.flush_output().await;
                self.show_main_menu().await;
            }
        } else {
            // At main menu - handle service selection or quit
            if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("q") {
                // User wants to disconnect -- full goodbye sequence
                // Send logout JSON to frontend first (clears localStorage token)
                let logout_msg = serde_json::json!({ "type": "logout" });
                let _ = self.tx.send(
                    serde_json::to_string(&logout_msg).unwrap()
                ).await;

                // Calculate session time and update DB
                if let AuthState::Authenticated {
                    user_id,
                    handle,
                    token,
                    login_time,
                    ..
                } = &self.auth_state
                {
                    let session_secs = login_time.elapsed().as_secs();
                    let session_minutes = (session_secs / 60) as i64;

                    // Update total_time_minutes in DB
                    let _ = update_user_time(
                        &self.state.db_pool,
                        *user_id,
                        session_minutes,
                    )
                    .await;

                    // Fetch updated user stats for goodbye screen
                    let (total_calls, total_time) =
                        match find_user_by_id(&self.state.db_pool, *user_id).await {
                            Ok(Some(u)) => {
                                (u.total_logins as i64, u.total_time_minutes as i64)
                            }
                            _ => (0, session_minutes),
                        };

                    // Render and send goodbye screen
                    let goodbye = render_goodbye(
                        handle,
                        session_minutes,
                        total_calls,
                        total_time,
                    );
                    let _ = self.tx.send(goodbye).await;

                    // Delete session token from DB
                    let _ = crate::auth::session::delete_session(
                        &self.state.db_pool,
                        token,
                    )
                    .await;

                    println!(
                        "User {} logged out ({}m session)",
                        handle, session_minutes
                    );
                }

                // Let user read the goodbye screen
                tokio::time::sleep(Duration::from_secs(3)).await;

                // Signal disconnection
                self.disconnecting = true;
                return;
            }

            // Profile command: show user's profile card with edit menu
            if trimmed.eq_ignore_ascii_case("profile") || trimmed.eq_ignore_ascii_case("p") {
                if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
                    let user_id = *user_id;
                    match find_user_by_id(&self.state.db_pool, user_id).await {
                        Ok(Some(user)) => {
                            let card = render_profile_card(&user, true);
                            let _ = self.tx.send(card).await;

                            // Show profile edit menu
                            let menu = render_profile_edit_menu_string();
                            let _ = self.tx.send(menu).await;

                            self.current_service = Some("__profile__".to_string());
                        }
                        _ => {
                            self.output_buffer.set_fg(Color::LightRed);
                            self.output_buffer.writeln("Error loading profile");
                            self.output_buffer.reset_color();
                            self.flush_output().await;
                        }
                    }
                }
                return;
            }

            // Try to parse as service number or name
            let services: Vec<(String, String)> = self.state.registry.list()
                .iter()
                .map(|(n, d)| (n.to_string(), d.to_string()))
                .collect();

            // Try number first
            if let Ok(num) = trimmed.parse::<usize>() {
                if num > 0 && num <= services.len() {
                    let service_name = services[num - 1].0.clone();
                    self.enter_service(&service_name).await;
                    return;
                }
            }

            // Try name match
            for (service_name, _) in &services {
                if service_name.eq_ignore_ascii_case(trimmed) {
                    self.enter_service(service_name).await;
                    return;
                }
            }

            // Invalid selection
            self.output_buffer.set_fg(Color::LightRed);
            self.output_buffer.writeln(&format!("Unknown command: {}", trimmed));
            self.output_buffer.set_fg(Color::LightCyan);
            self.output_buffer.write_str("Enter service number or (q)uit to disconnect: ");
            self.output_buffer.reset_color();
            self.flush_output().await;
        }
    }

    /// Handle input on the profile edit menu (1-4 to edit fields, Q to return).
    async fn handle_profile_menu_input(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.starts_with('\x1b') {
            return;
        }

        match trimmed {
            "q" | "Q" => {
                self.current_service = None;
                self.show_main_menu().await;
            }
            "1" => {
                self.current_service = Some("__profile_edit_real_name__".to_string());
                self.pagination_buffer = Some(String::new());
                send_colored_prompt(&self.tx, "Enter new real name: ", false).await;
            }
            "2" => {
                self.current_service = Some("__profile_edit_location__".to_string());
                self.pagination_buffer = Some(String::new());
                send_colored_prompt(&self.tx, "Enter new location: ", false).await;
            }
            "3" => {
                self.current_service = Some("__profile_edit_signature__".to_string());
                self.pagination_buffer = Some(String::new());
                send_colored_prompt(&self.tx, "Enter new signature: ", false).await;
            }
            "4" => {
                self.current_service = Some("__profile_edit_bio__".to_string());
                self.pagination_buffer = Some(String::new());
                send_colored_prompt(&self.tx, "Enter new bio: ", false).await;
            }
            _ => {
                // Invalid selection
                let mut w = AnsiWriter::new();
                w.set_fg(Color::LightRed);
                w.writeln("Invalid selection.");
                w.reset_color();
                let _ = self.tx.send(w.flush()).await;
            }
        }
    }

    /// Handle character-by-character input during profile field editing.
    async fn handle_profile_edit_input(&mut self, input: &str) {
        let tx = self.tx.clone();

        for ch in input.chars() {
            if ch == '\r' || ch == '\n' {
                // Enter pressed: save the field
                let field_name = {
                    let svc = self.current_service.as_deref().unwrap_or("");
                    match svc {
                        "__profile_edit_real_name__" => Some("real_name"),
                        "__profile_edit_location__" => Some("location"),
                        "__profile_edit_signature__" => Some("signature"),
                        "__profile_edit_bio__" => Some("bio"),
                        _ => None,
                    }
                };

                let edit_input = self.pagination_buffer.take().unwrap_or_default();

                let _ = tx.send("\r\n".to_string()).await;

                if let (Some(field), AuthState::Authenticated { user_id, .. }) =
                    (field_name, &self.auth_state)
                {
                    let user_id = *user_id;
                    match update_user_field(
                        &self.state.db_pool,
                        user_id,
                        field,
                        &edit_input,
                    ).await {
                        Ok(()) => {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightGreen);
                            w.writeln("Profile updated!");
                            w.reset_color();
                            let _ = tx.send(w.flush()).await;
                        }
                        Err(e) => {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightRed);
                            w.writeln(&format!("Error updating profile: {}", e));
                            w.reset_color();
                            let _ = tx.send(w.flush()).await;
                        }
                    }

                    // Re-show updated profile card + edit menu
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    match find_user_by_id(&self.state.db_pool, user_id).await {
                        Ok(Some(user)) => {
                            let card = render_profile_card(&user, true);
                            let _ = tx.send(card).await;
                            let menu = render_profile_edit_menu_string();
                            let _ = tx.send(menu).await;
                        }
                        _ => {}
                    }
                    self.current_service = Some("__profile__".to_string());
                }
                return;
            } else if ch == '\x7f' || ch == '\x08' {
                // Backspace
                if let Some(buf) = &mut self.pagination_buffer {
                    if buf.pop().is_some() {
                        let _ = tx.send("\x08 \x08".to_string()).await;
                    }
                }
            } else if !ch.is_control() {
                // Printable character: accumulate and echo
                if self.pagination_buffer.is_none() {
                    self.pagination_buffer = Some(String::new());
                }
                if let Some(buf) = &mut self.pagination_buffer {
                    buf.push(ch);
                }
                let _ = tx.send(ch.to_string()).await;
            }
        }
    }

    /// Enter a service (with authentic BBS "loading door" delay)
    async fn enter_service(&mut self, service_name: &str) {
        let service = self.state.registry.get(service_name).cloned();
        if let Some(service) = service {
            // Show "Entering door..." message
            self.output_buffer.writeln("");
            self.output_buffer.set_fg(Color::Yellow);
            self.output_buffer.bold();
            self.output_buffer.writeln(&format!("Entering {}...", service.name()));
            self.output_buffer.reset_color();
            self.flush_output().await;

            // Authentic BBS loading delay
            tokio::time::sleep(Duration::from_millis(800)).await;

            // Clear screen and enter service
            self.output_buffer.clear_screen();

            // Call service on_enter
            if let Err(e) = service.on_enter(self) {
                self.output_buffer.set_fg(Color::LightRed);
                self.output_buffer.writeln(&format!("Error entering service: {}", e));
                self.output_buffer.reset_color();
                self.flush_output().await;

                tokio::time::sleep(Duration::from_millis(1500)).await;
                self.show_main_menu().await;
                return;
            }

            // Mark as current service
            self.current_service = Some(service_name.to_string());
            self.flush_output().await;
        } else {
            self.output_buffer.set_fg(Color::LightRed);
            self.output_buffer.writeln(&format!("Service '{}' not found", service_name));
            self.output_buffer.reset_color();
            self.flush_output().await;
        }
    }

    /// Called when client disconnects - save session time, release node, and clean up.
    ///
    /// For unclean disconnects (browser close, network drop), this saves the
    /// elapsed session time to the DB without showing the goodbye screen.
    /// For clean disconnects (quit command), the goodbye sequence already handled
    /// time saving and token deletion.
    pub async fn on_disconnect(&mut self) {
        // Save session time and delete token if authenticated
        if let AuthState::Authenticated {
            user_id,
            handle,
            token,
            login_time,
            ..
        } = &self.auth_state
        {
            // Calculate and save session time (unclean disconnect -- no goodbye shown)
            let session_secs = login_time.elapsed().as_secs();
            let session_minutes = (session_secs / 60) as i64;
            let _ = update_user_time(&self.state.db_pool, *user_id, session_minutes).await;

            // Delete session token
            let _ = crate::auth::session::delete_session(&self.state.db_pool, token).await;
            println!(
                "Session for {} ended ({}m, unclean disconnect)",
                handle, session_minutes
            );
        }

        // Release the assigned node
        if let Some(node_id) = self.node_id {
            self.state.node_manager.release_node(node_id).await;
            println!("Node {} released", node_id);
        }

        // If in a service, call on_exit
        if let Some(service_name) = &self.current_service {
            let service = self.state.registry.get(service_name).cloned();
            if let Some(service) = service {
                service.on_exit(self);
                self.flush_output().await;
            }
        }

        println!("Session disconnected");
    }
}

/// Send a colored prompt via the tx channel (free function to avoid borrow conflicts).
async fn send_colored_prompt(tx: &mpsc::Sender<String>, prompt: &str, is_password: bool) {
    let mut w = AnsiWriter::new();
    if is_password {
        w.set_fg(Color::DarkGray);
    } else {
        w.set_fg(Color::LightCyan);
    }
    w.write_str(prompt);
    w.reset_color();
    let _ = tx.send(w.flush()).await;
}

/// Implement SessionIO trait for Service interaction
impl SessionIO for Session {
    fn write(&mut self, data: &str) {
        self.output_buffer.write_str(data);
    }

    fn writeln(&mut self, data: &str) {
        self.output_buffer.writeln(data);
    }

    fn queue_paginated(&mut self, data: &str) {
        self.pagination_buffer = Some(data.to_string());
    }
}
