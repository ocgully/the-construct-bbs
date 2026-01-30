use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    AppState,
    auth::session::create_session,
    connection::{ceremony, ChatMessage},
    db::{
        messages::{
            check_mailbox_full, create_message, delete_message, get_inbox_count,
            get_inbox_page, get_message_by_id, get_sender_handles, get_unread_count,
            mark_message_read, InboxEntry,
        },
        user::{find_user_by_id, find_user_by_handle, update_last_login, update_user_field, update_user_time},
    },
    menu::{self, MenuAction, MenuSession, MenuState},
    services::{
        ServiceAction, SessionIO,
        chat::{
            parse_chat_command, ChatCommand, render_chat_message, render_chat_help,
            render_chat_who, render_chat_welcome, render_chat_error,
        },
        goodbye::render_goodbye,
        login::{LoginFlow, LoginResult, render_login_header, render_welcome_back},
        mail::{
            render_compose_header, render_compose_help, render_inbox, render_mailbox_full_error,
            render_message, render_new_mail_notification, render_self_mail_error,
            render_user_not_found_error, ComposeAction, ComposeFlow, format_body_lines,
        },
        news::{
            fetch_feeds, render_news_list, render_news_article,
            render_news_loading, render_news_errors, NewsState,
        },
        profile::{render_profile_card, render_profile_edit_menu_string},
        registration::{RegistrationFlow, RegistrationResult, render_registration_header},
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
    /// Menu navigation state (created when user authenticates)
    menu_session: Option<MenuSession>,
    /// Session timer for time limit enforcement
    session_timer: Option<crate::connection::timer::SessionTimer>,
    /// Session history ID for updating logout time
    session_history_id: Option<i64>,
    /// Input buffer for user lookup handle entry
    lookup_input: Option<String>,
    /// Whether bank withdrawal has been offered this session
    withdrawal_offered: bool,
    /// Current inbox page number
    mail_page: usize,
    /// Active compose flow state machine
    mail_compose: Option<ComposeFlow>,
    /// ID of message currently being read (for reply/delete context)
    mail_reading_id: Option<i64>,
    /// Input buffer for message number entry in inbox
    mail_input_buffer: Option<String>,
    /// Handle of last DM sender for /r reply command
    last_dm_sender: Option<String>,
    /// Cancellation token for chat broadcast receiver task
    chat_cancel: Option<CancellationToken>,
    /// Current news viewing state
    news_state: Option<NewsState>,
    /// Active Grand Theft Meth game state
    gtm_flow: Option<crate::game::GtmFlow>,
}

/// Map user level string to numeric level for menu filtering
fn user_level_to_num(level: &str) -> u8 {
    match level {
        "Sysop" => 255,
        "User" => 0,
        _ => 0,
    }
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
            menu_session: None,
            session_timer: None,
            session_history_id: None,
            lookup_input: None,
            withdrawal_offered: false,
            mail_page: 0,
            mail_compose: None,
            mail_reading_id: None,
            mail_input_buffer: None,
            last_dm_sender: None,
            chat_cancel: None,
            news_state: None,
            gtm_flow: None,
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

    /// Show the current menu (main menu or submenu).
    async fn show_menu(&mut self) {
        let menu_session = match &self.menu_session {
            Some(ms) => ms,
            None => return,
        };

        let (handle, user_level_name) = match &self.auth_state {
            AuthState::Authenticated { handle, user_level, .. } => {
                (handle.clone(), user_level.clone())
            }
            _ => return,
        };

        let max_nodes = self.state.config.connection.max_nodes as usize;

        let output = match menu_session.state() {
            MenuState::MainMenu => {
                menu::render::render_main_menu(
                    &self.state.config.menu,
                    menu_session.user_level(),
                    &handle,
                    &user_level_name,
                    self.node_id,
                    max_nodes,
                )
            }
            MenuState::Submenu { submenu_key } => {
                let items = self.state.config.menu.submenu_items(
                    submenu_key,
                    menu_session.user_level(),
                );
                let submenu_name = self.state.config.menu.submenu_name(submenu_key);
                menu::render::render_submenu(
                    submenu_key,
                    submenu_name,
                    &items,
                    menu_session.user_level(),
                )
            }
        };

        let _ = self.tx.send(output).await;
    }

    /// Start the session timer and check for daily time reset.
    ///
    /// Called after authentication in all three paths (login, resume, registration).
    /// Checks whether the daily time limit has reset (midnight boundary),
    /// banks unused time, and spawns the countdown timer task.
    async fn start_session_timer(&mut self, user_id: i64, user_level: &str) {
        let pool = &self.state.db_pool;
        let config = &self.state.config;

        // Check daily reset (midnight boundary crossed since last login)
        if let Ok(needs_reset) = crate::db::user::check_daily_reset(pool, user_id).await {
            if needs_reset {
                let time_cfg = config.time_limits.get_time_config(user_level);
                let _ = crate::db::user::reset_daily_time(
                    pool, user_id, time_cfg.daily_minutes, time_cfg.time_bank_cap,
                ).await;
            }
        }

        // Get time configuration for this user level
        let time_cfg = config.time_limits.get_time_config(user_level);

        if time_cfg.daily_minutes == 0 {
            // Unlimited time (sysop) -- still start timer for status bar display
            let (active, _) = self.state.node_manager.get_status().await;
            let handle = match &self.auth_state {
                AuthState::Authenticated { handle, .. } => handle.clone(),
                _ => "Unknown".to_string(),
            };
            let timer = crate::connection::timer::SessionTimer::spawn(
                self.tx.clone(), 0, handle, active, user_id, self.state.db_pool.clone(),
            );
            self.session_timer = Some(timer);
            return;
        }

        let (daily_used, _banked, _) = match crate::db::user::get_user_time_info(pool, user_id).await {
            Ok(info) => info,
            Err(_) => (0, 0, 0),
        };

        let remaining = (time_cfg.daily_minutes - daily_used).max(0);

        if remaining <= 0 {
            // No daily time left -- check banked time
            let banked = match crate::db::user::get_user_time_info(pool, user_id).await {
                Ok((_, b, _)) => b,
                Err(_) => 0,
            };
            if banked <= 0 {
                // No time available at all -- disconnect
                let mut w = AnsiWriter::new();
                w.set_fg(Color::LightRed);
                w.bold();
                w.writeln("");
                w.writeln("Your daily time has expired and you have no banked time.");
                w.writeln("Please try again after midnight.");
                w.reset_color();
                let _ = self.tx.send(w.flush()).await;
                tokio::time::sleep(Duration::from_secs(3)).await;
                self.disconnecting = true;
                return;
            }
            // Has banked time -- withdraw it
            let withdrawn = crate::db::user::withdraw_banked_time(pool, user_id, banked).await.unwrap_or(0);
            let mut w = AnsiWriter::new();
            w.set_fg(Color::Yellow);
            w.writeln(&format!("Using {} minutes of banked time.", withdrawn));
            w.reset_color();
            let _ = self.tx.send(w.flush()).await;
            // Start timer with withdrawn amount
            let (active, _) = self.state.node_manager.get_status().await;
            let handle = match &self.auth_state {
                AuthState::Authenticated { handle, .. } => handle.clone(),
                _ => "Unknown".to_string(),
            };
            let timer = crate::connection::timer::SessionTimer::spawn(
                self.tx.clone(), withdrawn, handle, active, user_id, self.state.db_pool.clone(),
            );
            self.session_timer = Some(timer);
            return;
        }

        // Normal case: start timer with remaining daily time
        let (active, _) = self.state.node_manager.get_status().await;
        let handle = match &self.auth_state {
            AuthState::Authenticated { handle, .. } => handle.clone(),
            _ => "Unknown".to_string(),
        };
        let timer = crate::connection::timer::SessionTimer::spawn(
            self.tx.clone(), remaining, handle, active, user_id, self.state.db_pool.clone(),
        );
        self.session_timer = Some(timer);
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

                                    // Check for new mail
                                    if let Ok(unread) = get_unread_count(&self.state.db_pool, user.id).await {
                                        if unread > 0 {
                                            let notification = render_new_mail_notification(unread);
                                            let _ = self.tx.send(notification).await;
                                        }
                                    }

                                    // Set authenticated state
                                    self.auth_state = AuthState::Authenticated {
                                        user_id: user.id,
                                        handle: user.handle.clone(),
                                        token: token_str.to_string(),
                                        user_level: user_level.clone(),
                                        login_time: std::time::Instant::now(),
                                    };

                                    // Create menu session
                                    let user_level_num = user_level_to_num(&user_level);
                                    self.menu_session = Some(MenuSession::new(user_level_num));

                                    // Start session timer
                                    self.start_session_timer(user.id, &user_level).await;
                                    if self.disconnecting { return; }

                                    // Record session history
                                    if let Ok(history_id) = crate::db::session_history::insert_session_history(
                                        &self.state.db_pool, user.id, &user.handle,
                                    ).await {
                                        self.session_history_id = Some(history_id);
                                    }

                                    // Update NodeManager activity
                                    if let Some(node_id) = self.node_id {
                                        self.state.node_manager.update_activity(node_id, "Main Menu").await;
                                    }

                                    // Brief pause then show main menu
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    self.show_menu().await;
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

                        // Check for new mail
                        if let Ok(unread) = get_unread_count(&self.state.db_pool, user_id).await {
                            if unread > 0 {
                                let notification = render_new_mail_notification(unread);
                                let _ = tx.send(notification).await;
                            }
                        }

                        // Transition to Authenticated
                        self.auth_state = AuthState::Authenticated {
                            user_id,
                            handle: handle.clone(),
                            token,
                            user_level: user_level.clone(),
                            login_time: std::time::Instant::now(),
                        };

                        // Create menu session
                        let user_level_num = user_level_to_num(&user_level);
                        self.menu_session = Some(MenuSession::new(user_level_num));

                        // Start session timer
                        self.start_session_timer(user_id, &user_level).await;
                        if self.disconnecting { return; }

                        // Record session history
                        if let Ok(history_id) = crate::db::session_history::insert_session_history(
                            &self.state.db_pool, user_id, &handle,
                        ).await {
                            self.session_history_id = Some(history_id);
                        }

                        // Update NodeManager activity
                        if let Some(node_id) = self.node_id {
                            self.state.node_manager.update_activity(node_id, "Main Menu").await;
                        }

                        println!("User {} logged in", handle);

                        // Brief pause then show main menu
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        self.show_menu().await;
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

                                // Check for new mail
                                if let Ok(unread) = get_unread_count(&self.state.db_pool, user_id).await {
                                    if unread > 0 {
                                        let notification = render_new_mail_notification(unread);
                                        let _ = tx.send(notification).await;
                                    }
                                }

                                // Transition to Authenticated
                                self.auth_state = AuthState::Authenticated {
                                    user_id,
                                    handle: user.handle.clone(),
                                    token,
                                    user_level: user_level.clone(),
                                    login_time: std::time::Instant::now(),
                                };

                                // Create menu session
                                let user_level_num = user_level_to_num(&user_level);
                                self.menu_session = Some(MenuSession::new(user_level_num));

                                // Start session timer
                                self.start_session_timer(user_id, &user_level).await;
                                if self.disconnecting { return; }

                                // Record session history
                                if let Ok(history_id) = crate::db::session_history::insert_session_history(
                                    &self.state.db_pool, user_id, &user.handle,
                                ).await {
                                    self.session_history_id = Some(history_id);
                                }

                                // Update NodeManager activity
                                if let Some(node_id) = self.node_id {
                                    self.state.node_manager.update_activity(node_id, "Main Menu").await;
                                }

                                println!("User {} auto-logged in after registration", user.handle);

                                // Brief pause then show main menu
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                self.show_menu().await;
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
        // Update last input time for idle tracking
        if let Some(node_id) = self.node_id {
            self.state.node_manager.update_last_input(node_id).await;
        }

        // Check for timeout (timer expired flag set by timer.rs)
        if let Some(timer) = &self.session_timer {
            if timer.is_expired() {
                self.handle_timeout().await;
                return;
            }
        }

        // Check for low time -- offer bank withdrawal once per session
        if !self.withdrawal_offered {
            if let Some(timer) = &self.session_timer {
                if timer.is_low_time() {
                    self.withdrawal_offered = true;
                    // Check if banked time available
                    if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
                        let user_id = *user_id;
                        let banked = match crate::db::user::get_user_time_info(&self.state.db_pool, user_id).await {
                            Ok((_, b, _)) => b,
                            Err(_) => 0,
                        };
                        if banked > 0 {
                            // Offer withdrawal -- non-blocking message
                            let mut w = AnsiWriter::new();
                            w.writeln("");
                            w.set_fg(Color::Yellow);
                            w.bold();
                            w.writeln("  WARNING: Less than 1 minute remaining!");
                            w.writeln(&format!("  You have {} minutes in your time bank.", banked));
                            w.writeln("  Press [B] to use banked time, or any other key to continue.");
                            w.reset_color();
                            let _ = self.tx.send(w.flush()).await;
                            self.current_service = Some("__time_bank_prompt__".to_string());
                            return; // Wait for response
                        }
                    }
                }
            }
        }

        // Chat mode: process chat input
        if let Some(service_name) = &self.current_service {
            if service_name == "__chat__" {
                self.handle_chat_input(input).await;
                return;
            }
        }

        // News mode: process news input
        if let Some(service_name) = &self.current_service {
            if service_name == "__news__" {
                self.handle_news_input(input).await;
                return;
            }
        }

        // Grand Theft Meth game
        if let Some(service_name) = &self.current_service {
            if service_name == crate::services::grand_theft_meth::SENTINEL {
                self.handle_gtm_input(input).await;
                return;
            }
        }

        // News error screen: any input returns to menu
        if let Some(service_name) = &self.current_service {
            if service_name == "__news_error__" {
                if !input.is_empty() && !input.starts_with('\x1b') {
                    self.news_state = None;
                    self.current_service = None;
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                    self.show_menu().await;
                }
                return;
            }
        }

        // Profile edit mode needs raw input (spaces, Enter) -- check BEFORE trimming
        if let Some(service_name) = &self.current_service {
            if service_name.starts_with("__profile_edit_") {
                self.handle_profile_edit_input(input).await;
                return;
            }
        }

        // If we're in paging mode, any keypress advances to next page
        if self.pending_pages.is_some() {
            let trimmed = input.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('\x1b') {
                self.send_next_page().await;
            }
            return;
        }

        // Phase 4 sentinel services
        if let Some(service_name) = &self.current_service {
            let service_name = service_name.clone();

            // Time bank withdrawal prompt
            if service_name == "__time_bank_prompt__" {
                let trimmed = input.trim();
                if trimmed.eq_ignore_ascii_case("b") {
                    // Withdraw banked time
                    if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
                        let user_id = *user_id;
                        let withdrawn = crate::db::user::withdraw_banked_time(
                            &self.state.db_pool, user_id, 30, // Withdraw 30 minutes
                        ).await.unwrap_or(0);
                        if withdrawn > 0 {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightGreen);
                            w.writeln(&format!("  {} minutes of banked time applied!", withdrawn));
                            w.reset_color();
                            let _ = self.tx.send(w.flush()).await;
                            // Restart timer with new remaining time
                            if let Some(old_timer) = self.session_timer.take() {
                                old_timer.cancel();
                            }
                            let (active, _) = self.state.node_manager.get_status().await;
                            let handle = match &self.auth_state {
                                AuthState::Authenticated { handle, .. } => handle.clone(),
                                _ => "Unknown".to_string(),
                            };
                            let new_timer = crate::connection::timer::SessionTimer::spawn(
                                self.tx.clone(), withdrawn, handle, active, user_id, self.state.db_pool.clone(),
                            );
                            self.session_timer = Some(new_timer);
                            self.withdrawal_offered = false; // Allow re-prompt if they run low again
                        }
                    }
                }
                // Return to menu regardless
                self.current_service = None;
                if let Some(ms) = &mut self.menu_session {
                    ms.reset_to_main();
                }
                self.show_menu().await;
                return;
            }

            // Who's Online and Last Callers: any keypress returns to menu
            if service_name == "__whos_online__" || service_name == "__last_callers__" {
                if !input.is_empty() && !input.starts_with('\x1b') {
                    self.current_service = None;
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                    self.show_menu().await;
                }
                return;
            }

            // User Lookup: character-by-character input for handle
            if service_name == "__user_lookup__" {
                self.handle_user_lookup_input(input).await;
                return;
            }

            // User Lookup view/retry: any keypress returns to lookup prompt
            if service_name == "__user_lookup_view__" || service_name == "__user_lookup_retry__" {
                if !input.is_empty() && !input.starts_with('\x1b') {
                    self.handle_user_lookup_start().await;
                }
                return;
            }

            // Mail Inbox: handle C/N/P/Q/digit input
            if service_name == "__mail_inbox__" {
                self.handle_mail_inbox_input(input).await;
                return;
            }

            // Mail Read: handle R/D/N/Q
            if service_name == "__mail_read__" {
                self.handle_mail_read_input(input).await;
                return;
            }

            // Mail Compose: character-by-character through ComposeFlow
            if service_name == "__mail_compose__" {
                self.handle_mail_compose_input(input).await;
                return;
            }
        }

        // Profile menu: handle 1/2/3/4/q
        if let Some(service_name) = &self.current_service {
            if service_name == "__profile__" {
                self.handle_profile_menu_input(input).await;
                return;
            }

            // Currently in a service - route input to it
            let trimmed = input.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('\x1b') {
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
                            if let Some(ms) = &mut self.menu_session {
                                ms.reset_to_main();
                            }
                            // Update NodeManager activity
                            if let Some(node_id) = self.node_id {
                                self.state.node_manager.update_activity(node_id, "Main Menu").await;
                            }
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            self.show_menu().await;
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
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                    self.show_menu().await;
                }
            }
            return;
        }

        // AT MENU -- single keypress navigation
        for ch in input.chars() {
            // Skip escape sequences and control chars (except Enter)
            if ch == '\x1b' || (ch.is_control() && ch != '\r' && ch != '\n') {
                continue;
            }

            let action = {
                let menu_session = match &mut self.menu_session {
                    Some(ms) => ms,
                    None => return,
                };
                menu_session.process_key(ch, &self.state.config.menu)
            };

            match action {
                MenuAction::Redraw => {
                    self.show_menu().await;
                }
                MenuAction::EnterSubmenu(_key) => {
                    // State already transitioned in process_key
                    // Brief pause for transition feel
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    self.show_menu().await;
                    // Process any buffered keys (command stacking)
                    self.process_typeahead().await;
                }
                MenuAction::BackToMain => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    self.show_menu().await;
                }
                MenuAction::LaunchService(service_name) => {
                    // Echo the key
                    let _ = self.tx.send(format!("{}", ch)).await;
                    self.enter_service(&service_name).await;
                    return; // Stop processing further chars
                }
                MenuAction::ExecuteCommand(cmd) => {
                    match cmd.as_str() {
                        "quit" => {
                            // Echo 'Q'
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.handle_quit().await;
                            return;
                        }
                        "profile" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.handle_profile_view().await;
                            return;
                        }
                        "whos_online" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.handle_whos_online().await;
                            return;
                        }
                        "last_callers" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.handle_last_callers().await;
                            return;
                        }
                        "user_lookup" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.handle_user_lookup_start().await;
                            return;
                        }
                        "mail" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.show_inbox().await;
                            return;
                        }
                        "chat" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            self.enter_chat().await;
                            return;
                        }
                        "news" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            // Show loading screen
                            let loading = render_news_loading();
                            let _ = self.tx.send(loading).await;

                            // Fetch feeds
                            let feeds = &self.state.config.news.feeds;
                            let result = fetch_feeds(feeds).await;

                            // Create state and show list
                            let state = NewsState::new(result);
                            if state.articles.is_empty() && !state.errors.is_empty() {
                                // All feeds failed
                                let error_screen = render_news_errors(&state.errors);
                                let _ = self.tx.send(error_screen).await;
                                self.news_state = Some(state);
                                self.current_service = Some("__news_error__".to_string());
                            } else {
                                let list_screen = render_news_list(&state);
                                let _ = self.tx.send(list_screen).await;
                                self.news_state = Some(state);
                                self.current_service = Some("__news__".to_string());
                            }
                            return;
                        }
                        "grand_theft_meth" => {
                            let _ = self.tx.send(format!("{}", ch)).await;
                            use crate::services::grand_theft_meth::{SENTINEL, start_game};

                            if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                                match start_game(&self.state.gtm_db, *user_id, handle).await {
                                    Ok((flow, screen)) => {
                                        self.gtm_flow = Some(flow);
                                        self.current_service = Some(SENTINEL.to_string());
                                        let _ = self.tx.send(screen).await;
                                    }
                                    Err(e) => {
                                        self.output_buffer.set_fg(Color::LightRed);
                                        self.output_buffer.writeln(&format!("  Error: {}", e));
                                        self.output_buffer.reset_color();
                                        self.flush_output().await;
                                    }
                                }
                            }
                            return;
                        }
                        _ => {
                            // Unknown command, redraw
                            self.show_menu().await;
                        }
                    }
                }
                MenuAction::ShowHelp => {
                    let help = {
                        let ms = self.menu_session.as_ref().unwrap();
                        menu::render::render_help(
                            ms.state(),
                            &self.state.config.menu,
                            ms.user_level(),
                        )
                    };
                    let _ = self.tx.send(help).await;
                    // After help, wait for any key then redraw menu
                    // (Next input will trigger Redraw since help isn't a state)
                }
                MenuAction::Buffered | MenuAction::None => {
                    // Do nothing
                }
            }
        }
    }

    /// Process buffered type-ahead keys (for command stacking like G1)
    async fn process_typeahead(&mut self) {
        let actions = {
            let ms = match &mut self.menu_session {
                Some(ms) => ms,
                None => return,
            };
            ms.drain_buffer(&self.state.config.menu)
        };
        for action in actions {
            match action {
                MenuAction::EnterSubmenu(_) => {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    self.show_menu().await;
                }
                MenuAction::LaunchService(svc) => {
                    self.enter_service(&svc).await;
                    return;
                }
                MenuAction::ExecuteCommand(cmd) => {
                    match cmd.as_str() {
                        "quit" => { self.handle_quit().await; return; }
                        "profile" => { self.handle_profile_view().await; return; }
                        "whos_online" => { self.handle_whos_online().await; return; }
                        "last_callers" => { self.handle_last_callers().await; return; }
                        "user_lookup" => { self.handle_user_lookup_start().await; return; }
                        "mail" => { self.show_inbox().await; return; }
                        "chat" => { self.enter_chat().await; return; }
                        "news" => {
                            // Show loading screen
                            let loading = render_news_loading();
                            let _ = self.tx.send(loading).await;

                            // Fetch feeds
                            let feeds = &self.state.config.news.feeds;
                            let result = fetch_feeds(feeds).await;

                            // Create state and show list
                            let state = NewsState::new(result);
                            if state.articles.is_empty() && !state.errors.is_empty() {
                                // All feeds failed
                                let error_screen = render_news_errors(&state.errors);
                                let _ = self.tx.send(error_screen).await;
                                // Wait for any key to return
                                self.news_state = Some(state);
                                self.current_service = Some("__news_error__".to_string());
                            } else {
                                let list = render_news_list(&state);
                                let _ = self.tx.send(list).await;
                                self.news_state = Some(state);
                                self.current_service = Some("__news__".to_string());
                            }
                            return;
                        }
                        "grand_theft_meth" => {
                            use crate::services::grand_theft_meth::{SENTINEL, start_game};

                            if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                                match start_game(&self.state.gtm_db, *user_id, handle).await {
                                    Ok((flow, screen)) => {
                                        self.gtm_flow = Some(flow);
                                        self.current_service = Some(SENTINEL.to_string());
                                        let _ = self.tx.send(screen).await;
                                    }
                                    Err(e) => {
                                        let mut w = AnsiWriter::new();
                                        w.set_fg(Color::LightRed);
                                        w.writeln(&format!("  Error: {}", e));
                                        w.reset_color();
                                        let _ = self.tx.send(w.flush()).await;
                                    }
                                }
                            }
                            return;
                        }
                        _ => {}
                    }
                }
                MenuAction::Redraw => {
                    self.show_menu().await;
                }
                _ => {}
            }
        }
    }

    /// Handle quit command - full goodbye sequence
    async fn handle_quit(&mut self) {
        // Cancel session timer
        if let Some(timer) = self.session_timer.take() {
            timer.cancel();
        }

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

            // Update daily time used
            let _ = crate::db::user::update_daily_time_used(
                &self.state.db_pool, *user_id, session_minutes,
            ).await;

            // Update session history with logout time
            if let Some(history_id) = self.session_history_id {
                let _ = crate::db::session_history::update_session_history_logout(
                    &self.state.db_pool, history_id, session_minutes as i32,
                ).await;
            }

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
    }

    /// Handle profile view command
    async fn handle_profile_view(&mut self) {
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
                if let Some(ms) = &mut self.menu_session {
                    ms.reset_to_main();
                }
                self.show_menu().await;
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

    /// Handle Who's Online command
    async fn handle_whos_online(&mut self) {
        let nodes = self.state.node_manager.get_active_nodes_full().await;
        let output = crate::services::who::render_whos_online(&nodes);
        let _ = self.tx.send(output).await;
        // Set sentinel service to wait for keypress then return to menu
        self.current_service = Some("__whos_online__".to_string());

        if let Some(node_id) = self.node_id {
            self.state.node_manager.update_activity(node_id, "Who's Online").await;
        }
    }

    /// Handle Last Callers command
    async fn handle_last_callers(&mut self) {
        let limit = self.state.config.time_limits.last_callers_count;
        let entries = crate::db::session_history::get_last_callers(
            &self.state.db_pool, limit,
        ).await.unwrap_or_default();
        let output = crate::services::last_callers::render_last_callers(&entries);
        let _ = self.tx.send(output).await;
        // Set sentinel service to wait for keypress then return to menu
        self.current_service = Some("__last_callers__".to_string());

        if let Some(node_id) = self.node_id {
            self.state.node_manager.update_activity(node_id, "Last Callers").await;
        }
    }

    /// Handle User Lookup start -- show prompt
    async fn handle_user_lookup_start(&mut self) {
        let prompt = crate::services::user_profile::render_lookup_prompt();
        let _ = self.tx.send(prompt).await;
        self.current_service = Some("__user_lookup__".to_string());
        self.lookup_input = Some(String::new());

        if let Some(node_id) = self.node_id {
            self.state.node_manager.update_activity(node_id, "User Lookup").await;
        }
    }

    /// Handle character-by-character input during user lookup handle entry
    async fn handle_user_lookup_input(&mut self, input: &str) {
        let tx = self.tx.clone();

        for ch in input.chars() {
            if ch == '\r' || ch == '\n' {
                let handle_input = self.lookup_input.take().unwrap_or_default();
                let _ = tx.send("\r\n".to_string()).await;

                if handle_input.eq_ignore_ascii_case("q") || handle_input.is_empty() {
                    // Cancel lookup
                    self.current_service = None;
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                    self.show_menu().await;
                    return;
                }

                // Look up user
                match find_user_by_handle(&self.state.db_pool, &handle_input).await {
                    Ok(Some(user)) => {
                        // Show profile card (read-only)
                        let card = render_profile_card(&user, false);
                        let _ = tx.send(card).await;
                        let footer = crate::services::user_profile::render_profile_footer();
                        let _ = tx.send(footer).await;
                        // Switch to "viewing profile" state -- next keypress returns to lookup prompt
                        self.current_service = Some("__user_lookup_view__".to_string());
                    }
                    _ => {
                        let err = crate::services::user_profile::render_user_not_found(&handle_input);
                        let _ = tx.send(err).await;
                        // Stay in lookup mode, reinitialize input
                        self.lookup_input = Some(String::new());
                        // Next keypress shows prompt again
                        self.current_service = Some("__user_lookup_retry__".to_string());
                    }
                }
                return;
            } else if ch == '\x7f' || ch == '\x08' {
                // Backspace
                if let Some(buf) = &mut self.lookup_input {
                    if buf.pop().is_some() {
                        let _ = tx.send("\x08 \x08".to_string()).await;
                    }
                }
            } else if !ch.is_control() {
                if let Some(buf) = &mut self.lookup_input {
                    buf.push(ch);
                }
                let _ = tx.send(ch.to_string()).await;
            }
        }
    }

    /// Handle timeout -- timer expired, force logout with timeout goodbye screen
    async fn handle_timeout(&mut self) {
        // Take timer to prevent double handling
        let timer = self.session_timer.take();
        if let Some(t) = timer {
            t.cancel(); // Ensure task is cleaned up
        }

        // Send logout JSON to frontend
        let logout_msg = serde_json::json!({ "type": "logout" });
        let _ = self.tx.send(serde_json::to_string(&logout_msg).unwrap()).await;

        if let AuthState::Authenticated { user_id, handle, token, login_time, .. } = &self.auth_state {
            let session_secs = login_time.elapsed().as_secs();
            let session_minutes = (session_secs / 60) as i64;

            // Update daily time used
            let _ = crate::db::user::update_daily_time_used(
                &self.state.db_pool, *user_id, session_minutes,
            ).await;

            // Update total time
            let _ = update_user_time(
                &self.state.db_pool, *user_id, session_minutes,
            ).await;

            // Update session history
            if let Some(history_id) = self.session_history_id {
                let _ = crate::db::session_history::update_session_history_logout(
                    &self.state.db_pool, history_id, session_minutes as i32,
                ).await;
            }

            // Fetch stats for goodbye
            let (total_calls, total_time) = match find_user_by_id(&self.state.db_pool, *user_id).await {
                Ok(Some(u)) => (u.total_logins as i64, u.total_time_minutes as i64),
                _ => (0, session_minutes),
            };

            // Render timeout goodbye
            let goodbye = crate::services::goodbye::render_timeout_goodbye(
                handle, session_minutes, total_calls, total_time,
            );
            let _ = self.tx.send(goodbye).await;

            // Delete session token
            let _ = crate::auth::session::delete_session(&self.state.db_pool, token).await;

            println!("User {} timed out ({}m session)", handle, session_minutes);
        }

        // Let user read the goodbye screen
        tokio::time::sleep(Duration::from_secs(3)).await;
        self.disconnecting = true;
    }

    /// Check if session timer has expired and handle timeout if so.
    /// Called periodically from the main websocket loop to handle idle timeouts.
    /// Returns true if session timed out (caller should disconnect).
    pub async fn check_and_handle_timeout(&mut self) -> bool {
        if let Some(timer) = &self.session_timer {
            if timer.is_expired() {
                self.handle_timeout().await;
                return true;
            }
        }
        false
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

                if let Some(ms) = &mut self.menu_session {
                    ms.reset_to_main();
                }
                tokio::time::sleep(Duration::from_millis(1500)).await;
                self.show_menu().await;
                return;
            }

            // Mark as current service
            self.current_service = Some(service_name.to_string());

            // Update NodeManager activity
            if let Some(node_id) = self.node_id {
                self.state.node_manager.update_activity(node_id, &format!("In {}", service.name())).await;
            }

            self.flush_output().await;
        } else {
            self.output_buffer.set_fg(Color::LightRed);
            self.output_buffer.writeln(&format!("Service '{}' not found", service_name));
            self.output_buffer.reset_color();
            self.flush_output().await;
        }
    }

    /// Enter chat mode: join ChatManager, subscribe to broadcasts, spawn receiver task
    async fn enter_chat(&mut self) {
        // Get user info
        let (user_id, handle) = match &self.auth_state {
            AuthState::Authenticated { user_id, handle, .. } => (*user_id, handle.clone()),
            _ => return,
        };

        // Try to join chat
        if let Err(e) = self.state.chat_manager.join(user_id, handle.clone()).await {
            let _ = self.tx.send(render_chat_error(&e)).await;
            return;
        }

        // Subscribe to broadcasts
        let mut rx = self.state.chat_manager.subscribe();

        // Broadcast join announcement
        self.state.chat_manager.broadcast(ChatMessage::Join { handle: handle.clone() });

        // Send welcome message
        let _ = self.tx.send(render_chat_welcome()).await;

        // Set sentinel service
        self.current_service = Some("__chat__".to_string());

        // Update activity
        if let Some(node_id) = self.node_id {
            self.state.node_manager.update_activity(node_id, "In Chat").await;
        }

        // Spawn task to forward broadcasts to session tx
        let tx = self.tx.clone();
        let my_handle = handle.clone();
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        break;
                    }
                    result = rx.recv() => {
                        match result {
                            Ok(msg) => {
                                let formatted = render_chat_message(&msg, &my_handle);
                                if !formatted.is_empty() {
                                    // Check if this is a page for me (trigger bell)
                                    if let ChatMessage::Page { to, .. } = &msg {
                                        if to == &my_handle {
                                            // Send bell JSON signal
                                            let _ = tx.send(r#"{"type":"bell"}"#.to_string()).await;
                                        }
                                    }
                                    // Check if this is a DM to me (trigger bell)
                                    if let ChatMessage::Direct { from, to, .. } = &msg {
                                        if to == &my_handle && from != &my_handle {
                                            let _ = tx.send(r#"{"type":"bell"}"#.to_string()).await;
                                        }
                                    }
                                    let _ = tx.send(formatted).await;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                let _ = tx.send(render_chat_error(&format!("Missed {} messages", n))).await;
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                break;
                            }
                        }
                    }
                }
            }
        });

        self.chat_cancel = Some(cancel);
    }

    /// Exit chat mode: cancel receiver, broadcast leave, leave ChatManager
    async fn exit_chat(&mut self) {
        // Cancel broadcast receiver task
        if let Some(cancel) = self.chat_cancel.take() {
            cancel.cancel();
        }

        // Get user info and broadcast leave
        if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
            // Broadcast leave announcement
            self.state.chat_manager.broadcast(ChatMessage::Leave { handle: handle.clone() });

            // Leave ChatManager
            self.state.chat_manager.leave(*user_id).await;
        }

        // Clear sentinel
        self.current_service = None;

        // Update activity
        if let Some(node_id) = self.node_id {
            self.state.node_manager.update_activity(node_id, "Main Menu").await;
        }
    }

    /// Handle input while in chat mode
    async fn handle_chat_input(&mut self, input: &str) {
        let handle = match &self.auth_state {
            AuthState::Authenticated { handle, .. } => handle.clone(),
            _ => return,
        };

        let cmd = parse_chat_command(input);

        match cmd {
            ChatCommand::Empty => {
                // Do nothing on empty input
            }
            ChatCommand::Message(text) => {
                self.state.chat_manager.broadcast(ChatMessage::Public {
                    sender: handle,
                    text,
                });
            }
            ChatCommand::Quit => {
                self.exit_chat().await;
                // Return to main menu
                if let Some(ms) = &mut self.menu_session {
                    ms.reset_to_main();
                }
                self.show_menu().await;
            }
            ChatCommand::Help => {
                let _ = self.tx.send(render_chat_help()).await;
            }
            ChatCommand::Who => {
                let participants = self.state.chat_manager.get_participants().await;
                let _ = self.tx.send(render_chat_who(&participants)).await;
            }
            ChatCommand::Action(action) => {
                self.state.chat_manager.broadcast(ChatMessage::Action {
                    sender: handle,
                    action,
                });
            }
            ChatCommand::Page(target) => {
                // Check if target is in chat
                let participants = self.state.chat_manager.get_participants().await;
                let target_lower = target.to_lowercase();
                if let Some(found) = participants.iter().find(|h| h.to_lowercase() == target_lower) {
                    self.state.chat_manager.broadcast(ChatMessage::Page {
                        from: handle,
                        to: found.clone(),
                    });
                    let _ = self.tx.send(render_chat_error(&format!("Paged {}", found))).await;
                } else {
                    let _ = self.tx.send(render_chat_error(&format!("{} is not in chat", target))).await;
                }
            }
            ChatCommand::DirectMessage { target, text } => {
                // Find target in chat (case-insensitive)
                let participants = self.state.chat_manager.get_participants().await;
                let target_lower = target.to_lowercase();
                if let Some(found) = participants.iter().find(|h| h.to_lowercase() == target_lower) {
                    // Can't DM yourself
                    if found.to_lowercase() == handle.to_lowercase() {
                        let _ = self.tx.send(render_chat_error("Cannot message yourself")).await;
                    } else {
                        self.state.chat_manager.broadcast(ChatMessage::Direct {
                            from: handle,
                            to: found.clone(),
                            text,
                        });
                    }
                } else {
                    let _ = self.tx.send(render_chat_error(&format!("{} is not in chat", target))).await;
                }
            }
            ChatCommand::Reply(_text) => {
                // Simplified: suggest using /msg instead
                let _ = self.tx.send(render_chat_error("Use /msg <handle> <message> to send direct messages")).await;
            }
            ChatCommand::Unknown(cmd) => {
                let _ = self.tx.send(render_chat_error(&format!("Unknown command: {}. Type /help for available commands.", cmd))).await;
            }
            ChatCommand::Error(msg) => {
                let _ = self.tx.send(render_chat_error(&msg)).await;
            }
        }
    }

    /// Handle input for Grand Theft Meth game
    async fn handle_gtm_input(&mut self, input: &str) {
        use crate::services::grand_theft_meth::{
            render_screen, save_game_state, record_game_completion, get_game_leaderboard,
        };
        use crate::game::{GtmAction, GameScreen};

        // Process each character
        for ch in input.chars() {
            let action = {
                let flow = match &mut self.gtm_flow {
                    Some(f) => f,
                    None => return,
                };
                flow.handle_char(ch)
            };

            match action {
                GtmAction::Continue => {}
                GtmAction::Echo(s) => {
                    let _ = self.tx.send(s).await;
                }
                GtmAction::Render(s) => {
                    let _ = self.tx.send(s).await;
                }
                GtmAction::SaveGame => {
                    // Save to game's own database
                    if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                        if let Some(flow) = &self.gtm_flow {
                            let _ = save_game_state(&self.state.gtm_db, *user_id, handle, flow).await;
                        }
                    }
                    // Render new screen
                    if let Some(flow) = &self.gtm_flow {
                        let screen = render_screen(flow);
                        let _ = self.tx.send(screen).await;
                    }
                }
                GtmAction::GameOver { final_score: _, story_completed: _ } => {
                    // Record completion
                    if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                        if let Some(flow) = &self.gtm_flow {
                            let _ = record_game_completion(
                                &self.state.gtm_db,
                                *user_id,
                                handle,
                                flow,
                            ).await;
                        }
                    }

                    // Show game over screen
                    if let Some(flow) = &self.gtm_flow {
                        let screen = render_screen(flow);
                        let _ = self.tx.send(screen).await;
                    }

                    // Exit game on next input
                    self.gtm_flow = None;
                    self.current_service = None;
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                }
                GtmAction::Quit => {
                    // Save and exit
                    if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                        if let Some(flow) = &self.gtm_flow {
                            let _ = save_game_state(&self.state.gtm_db, *user_id, handle, flow).await;
                        }
                    }

                    self.gtm_flow = None;
                    self.current_service = None;
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                    self.show_menu().await;
                    return;
                }
            }
        }

        // Handle leaderboard screen specially (needs async data)
        if let Some(flow) = &self.gtm_flow {
            if matches!(flow.current_screen(), GameScreen::Leaderboard) {
                let entries = get_game_leaderboard(&self.state.gtm_db).await;
                let leaderboard_entries: Vec<_> = entries.iter()
                    .map(|e| (e.handle.clone(), e.final_score, e.days_played, e.story_completed))
                    .collect();
                let screen = crate::game::render::render_leaderboard_screen(&leaderboard_entries);
                let _ = self.tx.send(screen).await;
            }
        }
    }

    /// Show inbox (mail command handler from menu)
    async fn show_inbox(&mut self) {
        if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
            let user_id = *user_id;
            let page_size = 10i64; // Messages per page

            // Fetch inbox data
            match get_inbox_page(&self.state.db_pool, user_id, self.mail_page as i64, page_size).await {
                Ok(entries) => {
                    // Get sender handles
                    let sender_ids: Vec<i64> = entries.iter().map(|e| e.sender_id).collect();
                    let handles_result = get_sender_handles(&self.state.db_pool, &sender_ids).await;
                    let sender_handles: std::collections::HashMap<i64, String> = handles_result
                        .unwrap_or_default()
                        .into_iter()
                        .collect();

                    // Get total count
                    let total_count = get_inbox_count(&self.state.db_pool, user_id)
                        .await
                        .unwrap_or(0);

                    // Render inbox
                    let output = render_inbox(
                        &entries,
                        self.mail_page as i64,
                        total_count,
                        page_size,
                        &sender_handles,
                    );
                    let _ = self.tx.send(output).await;

                    // Set sentinel service
                    self.current_service = Some("__mail_inbox__".to_string());

                    // Update activity
                    if let Some(node_id) = self.node_id {
                        self.state.node_manager.update_activity(node_id, "Mail").await;
                    }
                }
                Err(e) => {
                    let mut w = AnsiWriter::new();
                    w.set_fg(Color::LightRed);
                    w.writeln(&format!("Error loading inbox: {}", e));
                    w.reset_color();
                    let _ = self.tx.send(w.flush()).await;
                }
            }
        }
    }

    /// Handle inbox input (C/N/P/Q/digit)
    async fn handle_mail_inbox_input(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.starts_with('\x1b') {
            return;
        }

        for ch in trimmed.chars() {
            match ch {
                'c' | 'C' => {
                    // Start compose flow
                    if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                        let user_id = *user_id;
                        let handle = handle.clone();
                        let mut compose_flow = ComposeFlow::new(user_id, handle);
                        let prompt = compose_flow.current_prompt().to_string();
                        compose_flow.advance_to_input();
                        self.mail_compose = Some(compose_flow);
                        self.current_service = Some("__mail_compose__".to_string());
                        send_colored_prompt(&self.tx, &prompt, false).await;
                    }
                    return;
                }
                'n' | 'N' => {
                    // Next page
                    if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
                        let user_id = *user_id;
                        let page_size = 10i64; // Messages per page
                        let total_count = get_inbox_count(&self.state.db_pool, user_id)
                            .await
                            .unwrap_or(0);
                        let max_page = ((total_count + page_size - 1) / page_size).saturating_sub(1).max(0);

                        if (self.mail_page as i64) < max_page {
                            self.mail_page += 1;
                            self.show_inbox().await;
                        }
                    }
                    return;
                }
                'p' | 'P' => {
                    // Previous page
                    if self.mail_page > 0 {
                        self.mail_page -= 1;
                        self.show_inbox().await;
                    }
                    return;
                }
                'q' | 'Q' => {
                    // Return to menu
                    self.current_service = None;
                    self.mail_page = 0;
                    self.mail_input_buffer = None;
                    if let Some(ms) = &mut self.menu_session {
                        ms.reset_to_main();
                    }
                    self.show_menu().await;
                    return;
                }
                '0'..='9' => {
                    // Accumulate digit
                    if self.mail_input_buffer.is_none() {
                        self.mail_input_buffer = Some(String::new());
                    }
                    if let Some(buf) = &mut self.mail_input_buffer {
                        buf.push(ch);
                        let _ = self.tx.send(ch.to_string()).await;
                    }
                }
                '\r' | '\n' => {
                    // Enter pressed: read message
                    if let Some(num_str) = self.mail_input_buffer.take() {
                        let _ = self.tx.send("\r\n".to_string()).await;
                        if let Ok(msg_num) = num_str.parse::<i64>() {
                            self.handle_read_message(msg_num).await;
                        }
                    }
                    return;
                }
                _ => {
                    // Ignore other chars
                }
            }
        }
    }

    /// Handle reading a message by number
    async fn handle_read_message(&mut self, msg_num: i64) {
        if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
            let user_id = *user_id;
            let page_size = 10i64; // Messages per page

            // Fetch inbox page to get the message at that index
            match get_inbox_page(&self.state.db_pool, user_id, self.mail_page as i64, page_size).await {
                Ok(entries) => {
                    // Convert msg_num (1-based display) to 0-based index on current page
                    let page_start = self.mail_page as i64 * page_size;
                    let page_index = msg_num - page_start - 1;

                    if page_index >= 0 && (page_index as usize) < entries.len() {
                        let entry = &entries[page_index as usize];
                        let message_id = entry.id;

                        // Fetch full message
                        match get_message_by_id(&self.state.db_pool, message_id, user_id).await {
                            Ok(Some(msg)) => {
                                // Mark as read
                                let _ = mark_message_read(&self.state.db_pool, message_id, user_id).await;

                                // Get sender handle
                                let sender_handle = match get_sender_handles(&self.state.db_pool, &[msg.sender_id]).await {
                                    Ok(handles) => handles.into_iter().next().map(|(_, h)| h).unwrap_or_else(|| "(unknown)".to_string()),
                                    Err(_) => "(unknown)".to_string(),
                                };

                                // Render message
                                let output = render_message(&msg, &sender_handle);
                                let _ = self.tx.send(output).await;

                                // Set state to reading
                                self.mail_reading_id = Some(message_id);
                                self.current_service = Some("__mail_read__".to_string());
                            }
                            _ => {
                                let mut w = AnsiWriter::new();
                                w.set_fg(Color::LightRed);
                                w.writeln("Error loading message.");
                                w.reset_color();
                                let _ = self.tx.send(w.flush()).await;
                            }
                        }
                    } else {
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::LightRed);
                        w.writeln("Invalid message number.");
                        w.reset_color();
                        let _ = self.tx.send(w.flush()).await;
                    }
                }
                Err(_) => {
                    let mut w = AnsiWriter::new();
                    w.set_fg(Color::LightRed);
                    w.writeln("Error loading inbox.");
                    w.reset_color();
                    let _ = self.tx.send(w.flush()).await;
                }
            }
        }
    }

    /// Handle mail read input (R/D/N/Q)
    async fn handle_mail_read_input(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() || trimmed.starts_with('\x1b') {
            return;
        }

        let ch = trimmed.chars().next().unwrap_or(' ');

        match ch {
            'r' | 'R' => {
                // Reply to message
                if let (Some(message_id), AuthState::Authenticated { user_id, handle, .. }) =
                    (self.mail_reading_id, &self.auth_state)
                {
                    let user_id = *user_id;
                    let sender_handle = handle.clone();

                    match get_message_by_id(&self.state.db_pool, message_id, user_id).await {
                        Ok(Some(msg)) => {
                            // Get recipient handle
                            let recipient_handle = match get_sender_handles(&self.state.db_pool, &[msg.sender_id]).await {
                                Ok(handles) => handles.into_iter().next().map(|(_, h)| h).unwrap_or_else(|| "(unknown)".to_string()),
                                Err(_) => "(unknown)".to_string(),
                            };

                            // Create reply compose flow
                            let mut compose_flow = ComposeFlow::new_reply(
                                user_id,
                                sender_handle,
                                msg.sender_id,
                                recipient_handle.clone(),
                                msg.subject.clone(),
                                msg.body.clone(),
                            );

                            // Show compose header
                            let header = render_compose_header(&recipient_handle);
                            let _ = self.tx.send(header).await;

                            // Show subject
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightGray);
                            w.write_str("  Subject: ");
                            w.set_fg(Color::White);
                            w.writeln(&msg.subject);
                            w.reset_color();
                            w.writeln("");
                            let _ = self.tx.send(w.flush()).await;

                            // Show body prompt
                            let prompt = compose_flow.current_prompt().to_string();
                            compose_flow.advance_to_input();
                            let _ = self.tx.send(prompt).await;

                            self.mail_compose = Some(compose_flow);
                            self.current_service = Some("__mail_compose__".to_string());
                        }
                        _ => {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightRed);
                            w.writeln("Error loading message for reply.");
                            w.reset_color();
                            let _ = self.tx.send(w.flush()).await;
                        }
                    }
                }
            }
            'd' | 'D' => {
                // Delete message
                if let (Some(message_id), AuthState::Authenticated { user_id, .. }) =
                    (self.mail_reading_id, &self.auth_state)
                {
                    let user_id = *user_id;
                    match delete_message(&self.state.db_pool, message_id, user_id).await {
                        Ok(true) => {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightGreen);
                            w.writeln("");
                            w.writeln("  Message deleted.");
                            w.reset_color();
                            let _ = self.tx.send(w.flush()).await;
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                        _ => {
                            let mut w = AnsiWriter::new();
                            w.set_fg(Color::LightRed);
                            w.writeln("");
                            w.writeln("  Error deleting message.");
                            w.reset_color();
                            let _ = self.tx.send(w.flush()).await;
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                    // Return to inbox
                    self.mail_reading_id = None;
                    self.show_inbox().await;
                }
            }
            'n' | 'N' => {
                // Next message (just return to inbox for now)
                self.mail_reading_id = None;
                self.show_inbox().await;
            }
            'q' | 'Q' => {
                // Return to inbox
                self.mail_reading_id = None;
                self.show_inbox().await;
            }
            _ => {
                // Ignore other input
            }
        }
    }

    /// Handle mail compose input (character-by-character through ComposeFlow)
    async fn handle_mail_compose_input(&mut self, input: &str) {
        let tx = self.tx.clone();

        for ch in input.chars() {
            if let Some(compose_flow) = &mut self.mail_compose {
                let action = compose_flow.handle_char(ch);

                match action {
                    ComposeAction::Continue => {
                        // Do nothing
                    }
                    ComposeAction::Echo(s) => {
                        let _ = tx.send(s).await;
                    }
                    ComposeAction::ShowPrompt(s) => {
                        let _ = tx.send("\r\n".to_string()).await;
                        send_colored_prompt(&tx, &s, false).await;
                    }
                    ComposeAction::NeedRecipientLookup(handle) => {
                        let _ = tx.send("\r\n".to_string()).await;

                        // Look up recipient
                        match find_user_by_handle(&self.state.db_pool, &handle).await {
                            Ok(Some(user)) => {
                                // Check self-mail
                                if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
                                    if user.id == *user_id {
                                        // Self-mail error
                                        let error = render_self_mail_error();
                                        let _ = tx.send(error).await;
                                        if let Some(flow) = &mut self.mail_compose {
                                            flow.set_recipient_error();
                                            let prompt = flow.current_prompt().to_string();
                                            send_colored_prompt(&tx, &prompt, false).await;
                                            flow.advance_to_input();
                                        }
                                        continue;
                                    }
                                }

                                // Valid recipient
                                if let Some(flow) = &mut self.mail_compose {
                                    flow.set_recipient(user.id, user.handle.clone());
                                    let prompt = flow.current_prompt().to_string();
                                    send_colored_prompt(&tx, &prompt, false).await;
                                    flow.advance_to_input();
                                }
                            }
                            _ => {
                                // User not found
                                let error = render_user_not_found_error(&handle);
                                let _ = tx.send(error).await;
                                if let Some(flow) = &mut self.mail_compose {
                                    flow.set_recipient_error();
                                    let prompt = flow.current_prompt().to_string();
                                    send_colored_prompt(&tx, &prompt, false).await;
                                    flow.advance_to_input();
                                }
                            }
                        }
                    }
                    ComposeAction::SendMessage { recipient_id, recipient_handle, subject, body } => {
                        let _ = tx.send("\r\n".to_string()).await;

                        // Check mailbox full
                        let limit = self.state.config.mail.mailbox_size_limit;
                        match check_mailbox_full(&self.state.db_pool, recipient_id, limit).await {
                            Ok(true) => {
                                // Mailbox full
                                let error = render_mailbox_full_error();
                                let _ = tx.send(error).await;
                                tokio::time::sleep(Duration::from_secs(2)).await;
                            }
                            _ => {
                                // Send message
                                if let AuthState::Authenticated { user_id, .. } = &self.auth_state {
                                    let user_id = *user_id;
                                    match create_message(&self.state.db_pool, user_id, recipient_id, &subject, &body).await {
                                        Ok(_) => {
                                            // Increment messages_sent counter
                                            let current = match find_user_by_id(&self.state.db_pool, user_id).await {
                                                Ok(Some(u)) => u.messages_sent,
                                                _ => 0,
                                            };
                                            let _ = update_user_field(&self.state.db_pool, user_id, "messages_sent", &(current + 1).to_string()).await;

                                            let mut w = AnsiWriter::new();
                                            w.set_fg(Color::LightGreen);
                                            w.writeln(&format!("  Message sent to {}.", recipient_handle));
                                            w.reset_color();
                                            let _ = tx.send(w.flush()).await;
                                            tokio::time::sleep(Duration::from_millis(500)).await;
                                        }
                                        Err(e) => {
                                            let mut w = AnsiWriter::new();
                                            w.set_fg(Color::LightRed);
                                            w.writeln(&format!("  Error sending message: {}", e));
                                            w.reset_color();
                                            let _ = tx.send(w.flush()).await;
                                            tokio::time::sleep(Duration::from_millis(500)).await;
                                        }
                                    }
                                }
                            }
                        }

                        // Return to inbox
                        self.mail_compose = None;
                        self.show_inbox().await;
                        return;
                    }
                    ComposeAction::Aborted => {
                        let _ = tx.send("\r\n".to_string()).await;
                        let mut w = AnsiWriter::new();
                        w.set_fg(Color::Yellow);
                        w.writeln("  Message aborted.");
                        w.reset_color();
                        let _ = tx.send(w.flush()).await;
                        tokio::time::sleep(Duration::from_millis(500)).await;

                        // Return to inbox
                        self.mail_compose = None;
                        self.show_inbox().await;
                        return;
                    }
                    ComposeAction::ShowHelp => {
                        let help = render_compose_help();
                        let _ = tx.send(help).await;
                    }
                    ComposeAction::ShowLines(s) => {
                        let _ = tx.send(s).await;
                    }
                }
            }
        }
    }

    /// Called when client disconnects - save session time, release node, and clean up.
    ///
    /// For unclean disconnects (browser close, network drop), this saves the
    /// elapsed session time to the DB without showing the goodbye screen.
    /// For clean disconnects (quit command), the goodbye sequence already handled
    /// time saving and token deletion.
    pub async fn on_disconnect(&mut self) {
        // Cancel session timer
        if let Some(timer) = self.session_timer.take() {
            timer.cancel();
        }

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

            // Update daily time used
            let _ = crate::db::user::update_daily_time_used(
                &self.state.db_pool, *user_id, session_minutes,
            ).await;

            // Update session history with logout time
            if let Some(history_id) = self.session_history_id {
                let _ = crate::db::session_history::update_session_history_logout(
                    &self.state.db_pool, history_id, session_minutes as i32,
                ).await;
            }

            // Delete session token
            let _ = crate::auth::session::delete_session(&self.state.db_pool, token).await;
            println!(
                "Session for {} ended ({}m, unclean disconnect)",
                handle, session_minutes
            );
        }

        // Exit chat if in chat mode
        if let Some(service_name) = &self.current_service {
            if service_name == "__chat__" {
                // Cancel broadcast receiver
                if let Some(cancel) = self.chat_cancel.take() {
                    cancel.cancel();
                }
                // Broadcast leave and cleanup
                if let AuthState::Authenticated { user_id, handle, .. } = &self.auth_state {
                    self.state.chat_manager.broadcast(ChatMessage::Leave { handle: handle.clone() });
                    self.state.chat_manager.leave(*user_id).await;
                }
            }
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

    /// Handle input while in news view
    async fn handle_news_input(&mut self, input: &str) {
        let Some(ref mut state) = self.news_state else {
            // No state, return to menu
            self.current_service = None;
            if let Some(ms) = &mut self.menu_session {
                ms.reset_to_main();
            }
            self.show_menu().await;
            return;
        };

        // Check for escape sequences (arrow keys)
        // Up arrow: \x1b[A or \x1bOA
        // Down arrow: \x1b[B or \x1bOB
        let is_up = input == "\x1b[A" || input == "\x1bOA";
        let is_down = input == "\x1b[B" || input == "\x1bOB";

        if state.viewing_article {
            // Article view mode
            match input.to_uppercase().as_str() {
                "Q" => {
                    // Return to list
                    state.exit_article();
                    let list = render_news_list(state);
                    let _ = self.tx.send(list).await;
                }
                "N" => {
                    // Next article
                    state.select_next();
                    if let Some(article) = state.current_article() {
                        let view = render_news_article(article);
                        let _ = self.tx.send(view).await;
                    }
                }
                "P" => {
                    // Previous article
                    state.select_prev();
                    if let Some(article) = state.current_article() {
                        let view = render_news_article(article);
                        let _ = self.tx.send(view).await;
                    }
                }
                _ => {} // Ignore other input
            }
        } else {
            // List view mode
            if is_up {
                state.select_prev();
                let list = render_news_list(state);
                let _ = self.tx.send(list).await;
            } else if is_down {
                state.select_next();
                let list = render_news_list(state);
                let _ = self.tx.send(list).await;
            } else {
                match input.to_uppercase().as_str() {
                    "Q" => {
                        // Quit to main menu
                        self.news_state = None;
                        self.current_service = None;
                        if let Some(ms) = &mut self.menu_session {
                            ms.reset_to_main();
                        }
                        self.show_menu().await;
                    }
                    "\r" | "\n" => {
                        // Enter - view selected article
                        if state.has_articles() {
                            state.enter_article();
                            if let Some(article) = state.current_article() {
                                let view = render_news_article(article);
                                let _ = self.tx.send(view).await;
                            }
                        }
                    }
                    "N" => {
                        // Next page
                        let page_size = 15;
                        if state.page_offset + page_size < state.articles.len() {
                            state.page_offset += page_size;
                            state.selected_idx = state.page_offset;
                            let list = render_news_list(state);
                            let _ = self.tx.send(list).await;
                        }
                    }
                    "P" => {
                        // Previous page
                        let page_size = 15;
                        if state.page_offset >= page_size {
                            state.page_offset -= page_size;
                            state.selected_idx = state.page_offset;
                            let list = render_news_list(state);
                            let _ = self.tx.send(list).await;
                        }
                    }
                    _ => {} // Ignore other input
                }
            }
        }
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
