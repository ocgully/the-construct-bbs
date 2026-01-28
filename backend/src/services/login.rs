use sqlx::sqlite::SqlitePool;

use crate::auth::password::verify_password;
use crate::auth::session::{create_session, get_active_session_for_user};
use crate::config::Config;
use crate::db::login_attempts::{is_locked_out, record_login_attempt};
use crate::db::user::{find_user_by_handle, update_last_login};
use crate::terminal::{AnsiWriter, Color};

/// The stages of the interactive login flow.
#[derive(Debug, Clone, PartialEq)]
enum LoginState {
    EnterHandle,
    EnterPassword { handle: String },
}

/// Result of processing one step of login input.
#[derive(Debug, Clone, PartialEq)]
pub enum LoginResult {
    /// Show next prompt (state advanced successfully).
    Continue,
    /// Show error message, then re-show current prompt.
    Error(String),
    /// Login succeeded -- contains session info.
    Success {
        user_id: i64,
        handle: String,
        token: String,
        user_level: String,
        last_login: Option<String>,
    },
    /// User typed "new" -- switch to registration flow.
    SwitchToRegistration,
    /// Account locked out -- too many failed attempts.
    Locked,
}

/// Interactive login flow as a state machine.
///
/// Like RegistrationFlow, this is NOT a Service trait implementation. It is a
/// special-purpose flow that the Session drives directly, because:
/// - Login needs async DB access (Service trait methods are sync)
/// - Login needs password masking (requires special terminal handling)
/// - Login is a pre-authentication flow, not a post-login service
pub struct LoginFlow {
    state: LoginState,
    input_buffer: String,
    failed_attempts: u32,
}

impl LoginFlow {
    /// Create a new login flow starting at handle entry.
    pub fn new() -> Self {
        Self {
            state: LoginState::EnterHandle,
            input_buffer: String::new(),
            failed_attempts: 0,
        }
    }

    /// Return the prompt text for the current state.
    pub fn current_prompt(&self) -> &str {
        match &self.state {
            LoginState::EnterHandle => "Enter your handle: ",
            LoginState::EnterPassword { .. } => "Password: ",
        }
    }

    /// Whether the current state requires password masking (asterisk echo).
    pub fn needs_password_mask(&self) -> bool {
        matches!(self.state, LoginState::EnterPassword { .. })
    }

    /// Maximum input length for the current state.
    fn max_input_length(&self) -> usize {
        match &self.state {
            LoginState::EnterHandle => 20,
            LoginState::EnterPassword { .. } => 128,
        }
    }

    /// Handle a single character of input and return the echo string.
    ///
    /// - For printable characters: returns the character itself (or '*' for password fields).
    /// - For backspace (\x7f or \x08): removes last char from buffer and returns
    ///   the backspace-erase sequence (\x08 \x20 \x08).
    /// - For other control characters: returns None (no echo).
    pub fn handle_char(&mut self, ch: char) -> Option<String> {
        // Backspace: \x7f (DEL) or \x08 (BS)
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return Some("\x08 \x08".to_string());
            }
            return None; // Nothing to erase
        }

        // Enter: \r or \n -- don't accumulate, caller will call take_input
        if ch == '\r' || ch == '\n' {
            return None;
        }

        // Ignore other control characters
        if ch.is_control() {
            return None;
        }

        // Enforce max length for current field
        if self.input_buffer.len() >= self.max_input_length() {
            return None;
        }

        // Printable character -- accumulate in buffer
        self.input_buffer.push(ch);

        if self.needs_password_mask() {
            Some("*".to_string())
        } else {
            Some(ch.to_string())
        }
    }

    /// Take and clear the accumulated input buffer.
    ///
    /// Called when Enter is received to get the full line of input.
    pub fn take_input(&mut self) -> String {
        std::mem::take(&mut self.input_buffer)
    }

    /// Process a complete line of input for the current login state.
    ///
    /// This is the core state machine. Each state validates input and advances
    /// to the next state on success.
    pub async fn handle_input(
        &mut self,
        input: &str,
        pool: &SqlitePool,
        config: &Config,
    ) -> LoginResult {
        match &self.state {
            LoginState::EnterHandle => self.handle_enter_handle(input, pool, config).await,
            LoginState::EnterPassword { .. } => {
                self.handle_enter_password(input, pool, config).await
            }
        }
    }

    // --- State handlers ---

    async fn handle_enter_handle(
        &mut self,
        input: &str,
        pool: &SqlitePool,
        config: &Config,
    ) -> LoginResult {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return LoginResult::Error("Handle cannot be empty".to_string());
        }

        // "new" (case-insensitive) switches to registration
        if trimmed.eq_ignore_ascii_case("new") {
            return LoginResult::SwitchToRegistration;
        }

        // Check lockout
        match is_locked_out(
            pool,
            trimmed,
            config.auth.max_login_attempts,
            config.auth.lockout_minutes,
        )
        .await
        {
            Ok(true) => {
                return LoginResult::Error(
                    "Account temporarily locked. Try again later.".to_string(),
                );
            }
            Ok(false) => {}
            Err(e) => {
                return LoginResult::Error(format!("Database error: {}", e));
            }
        }

        // Look up user
        match find_user_by_handle(pool, trimmed).await {
            Ok(None) => {
                return LoginResult::Error(
                    "Handle not found. Type 'new' to register.".to_string(),
                );
            }
            Ok(Some(user)) => {
                // Check email verification
                if user.email_verified == 0 {
                    return LoginResult::Error(
                        "Account not verified. Please complete email verification.".to_string(),
                    );
                }

                // Advance to password entry
                self.state = LoginState::EnterPassword {
                    handle: trimmed.to_string(),
                };
                LoginResult::Continue
            }
            Err(e) => LoginResult::Error(format!("Database error: {}", e)),
        }
    }

    async fn handle_enter_password(
        &mut self,
        input: &str,
        pool: &SqlitePool,
        config: &Config,
    ) -> LoginResult {
        // Extract handle from current state
        let handle = match &self.state {
            LoginState::EnterPassword { handle } => handle.clone(),
            _ => return LoginResult::Error("Internal error".to_string()),
        };

        // Look up user again to get password hash
        let user = match find_user_by_handle(pool, &handle).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                self.state = LoginState::EnterHandle;
                return LoginResult::Error("User not found".to_string());
            }
            Err(e) => {
                self.state = LoginState::EnterHandle;
                return LoginResult::Error(format!("Database error: {}", e));
            }
        };

        // Verify password using spawn_blocking (Argon2 is CPU-intensive)
        let password = input.to_string();
        let hash = user.password_hash.clone();
        let verify_result =
            tokio::task::spawn_blocking(move || verify_password(&password, &hash)).await;

        let password_valid = match verify_result {
            Ok(Ok(valid)) => valid,
            Ok(Err(e)) => {
                let _ = record_login_attempt(pool, &handle, false).await;
                self.state = LoginState::EnterHandle;
                return LoginResult::Error(format!("Verification error: {}", e));
            }
            Err(e) => {
                let _ = record_login_attempt(pool, &handle, false).await;
                self.state = LoginState::EnterHandle;
                return LoginResult::Error(format!("Internal error: {}", e));
            }
        };

        if !password_valid {
            // Record failed attempt
            let _ = record_login_attempt(pool, &handle, false).await;
            self.failed_attempts += 1;

            if self.failed_attempts >= config.auth.max_login_attempts {
                return LoginResult::Locked;
            }

            let mut msg = "Invalid password".to_string();
            if self.failed_attempts >= 1 {
                msg.push_str(". Forgot password? Type 'reset' at handle prompt");
            }

            // Go back to handle entry
            self.state = LoginState::EnterHandle;
            return LoginResult::Error(msg);
        }

        // Password correct -- record success
        let _ = record_login_attempt(pool, &handle, true).await;

        // Check for duplicate session
        match get_active_session_for_user(pool, user.id).await {
            Ok(Some(_)) => {
                self.state = LoginState::EnterHandle;
                return LoginResult::Error(
                    "You are already connected from another session".to_string(),
                );
            }
            Ok(None) => {}
            Err(e) => {
                self.state = LoginState::EnterHandle;
                return LoginResult::Error(format!("Database error: {}", e));
            }
        }

        // Update last_login and total_logins
        let _ = update_last_login(pool, user.id).await;

        // Create session token
        let node_id_opt: Option<i32> = None; // Will be set by session layer
        let token = match create_session(
            pool,
            user.id,
            node_id_opt,
            config.auth.session_duration_hours,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                self.state = LoginState::EnterHandle;
                return LoginResult::Error(format!("Failed to create session: {}", e));
            }
        };

        // Determine user level (sysop override from config)
        let user_level = if config
            .auth
            .sysop_handles
            .iter()
            .any(|h| h.eq_ignore_ascii_case(&user.handle))
        {
            "Sysop".to_string()
        } else {
            user.user_level.clone()
        };

        LoginResult::Success {
            user_id: user.id,
            handle: user.handle.clone(),
            token,
            user_level,
            last_login: user.last_login.clone(),
        }
    }
}

/// Render the ANSI art login header with BBS name, tagline, and node info.
/// Full 80-column width (78 inner + 2 border chars).
pub fn render_login_header(tagline: &str, active_nodes: usize, max_nodes: usize) -> String {
    let mut w = AnsiWriter::new();
    let inner = 78;

    let available = max_nodes.saturating_sub(active_nodes);
    let h_line = "\u{2500}".repeat(inner);

    w.set_fg(Color::LightCyan);
    w.bold();

    // Top border
    w.writeln(&format!("\u{250C}{}\u{2510}", h_line));

    // BBS name line (centered)
    w.write_str("\u{2502}");
    w.set_fg(Color::Yellow);
    w.bold();
    w.write_str(&format!("{:^78}", "THE CONSTRUCT BBS"));
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("\u{2502}");

    // Tagline line (centered)
    w.write_str("\u{2502}");
    w.set_fg(Color::LightGreen);
    let quoted_tagline = format!("\"{}\"", tagline);
    w.write_str(&format!("{:^78}", quoted_tagline));
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("\u{2502}");

    // Empty line
    w.write_str("\u{2502}");
    w.write_str(&" ".repeat(inner));
    w.writeln("\u{2502}");

    // Node info line (centered)
    w.write_str("\u{2502}");
    w.set_fg(Color::White);
    let node_info = format!(
        "Node {} of {} -- {} lines available",
        active_nodes, max_nodes, available
    );
    w.write_str(&format!("{:^78}", node_info));
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("\u{2502}");

    // Empty line
    w.write_str("\u{2502}");
    w.write_str(&" ".repeat(inner));
    w.writeln("\u{2502}");

    // Instruction line (centered)
    w.write_str("\u{2502}");
    w.set_fg(Color::White);
    let instruction = "Enter your handle or type 'new' to register";
    w.write_str(&format!("{:^78}", instruction));
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("\u{2502}");

    // Bottom border
    w.writeln(&format!("\u{2514}{}\u{2518}", h_line));
    w.reset_color();
    w.writeln("");

    w.flush()
}

/// Render the welcome-back message shown after successful login.
pub fn render_welcome_back(handle: &str, last_login: Option<&str>, total_logins: i32) -> String {
    let mut w = AnsiWriter::new();

    w.set_fg(Color::DarkGray);
    w.writeln("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");

    w.write_str("  Welcome back, ");
    w.set_fg(Color::Yellow);
    w.bold();
    w.write_str(handle);
    w.reset_color();
    w.writeln("!");

    w.set_fg(Color::LightCyan);
    if let Some(last) = last_login {
        w.writeln(&format!("  Last login: {}", last));
    } else {
        w.writeln("  First login!");
    }
    w.writeln(&format!("  Total calls: {}", total_logins));

    w.set_fg(Color::DarkGray);
    w.writeln("\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- LoginFlow unit tests ---

    #[test]
    fn new_flow_starts_at_enter_handle() {
        let flow = LoginFlow::new();
        assert_eq!(flow.current_prompt(), "Enter your handle: ");
    }

    #[test]
    fn password_prompt_shown_for_password_state() {
        let mut flow = LoginFlow::new();
        flow.state = LoginState::EnterPassword {
            handle: "test".to_string(),
        };
        assert_eq!(flow.current_prompt(), "Password: ");
    }

    #[test]
    fn needs_password_mask_only_for_password_state() {
        let mut flow = LoginFlow::new();
        assert!(!flow.needs_password_mask()); // EnterHandle

        flow.state = LoginState::EnterPassword {
            handle: "test".to_string(),
        };
        assert!(flow.needs_password_mask());
    }

    #[test]
    fn handle_char_echoes_printable_character() {
        let mut flow = LoginFlow::new();
        let echo = flow.handle_char('a');
        assert_eq!(echo, Some("a".to_string()));
        assert_eq!(flow.input_buffer, "a");
    }

    #[test]
    fn handle_char_echoes_asterisk_for_password() {
        let mut flow = LoginFlow::new();
        flow.state = LoginState::EnterPassword {
            handle: "test".to_string(),
        };

        let echo = flow.handle_char('s');
        assert_eq!(echo, Some("*".to_string()));
        assert_eq!(flow.input_buffer, "s");
    }

    #[test]
    fn handle_char_backspace_erases_last_char() {
        let mut flow = LoginFlow::new();
        flow.handle_char('a');
        flow.handle_char('b');
        assert_eq!(flow.input_buffer, "ab");

        let echo = flow.handle_char('\x7f');
        assert_eq!(echo, Some("\x08 \x08".to_string()));
        assert_eq!(flow.input_buffer, "a");
    }

    #[test]
    fn handle_char_backspace_on_empty_returns_none() {
        let mut flow = LoginFlow::new();
        let echo = flow.handle_char('\x7f');
        assert_eq!(echo, None);
    }

    #[test]
    fn handle_char_enter_returns_none() {
        let mut flow = LoginFlow::new();
        flow.handle_char('a');
        assert_eq!(flow.handle_char('\r'), None);
        assert_eq!(flow.handle_char('\n'), None);
        assert_eq!(flow.input_buffer, "a");
    }

    #[test]
    fn handle_char_control_chars_ignored() {
        let mut flow = LoginFlow::new();
        assert_eq!(flow.handle_char('\x01'), None);
        assert_eq!(flow.handle_char('\x1b'), None);
        assert_eq!(flow.input_buffer, "");
    }

    #[test]
    fn take_input_clears_buffer() {
        let mut flow = LoginFlow::new();
        flow.handle_char('h');
        flow.handle_char('i');
        let input = flow.take_input();
        assert_eq!(input, "hi");
        assert_eq!(flow.input_buffer, "");
    }

    #[test]
    fn render_login_header_contains_bbs_name() {
        let header = render_login_header("Test tagline", 3, 16);
        assert!(header.contains("THE CONSTRUCT BBS"));
        assert!(header.contains("Test tagline"));
        assert!(header.contains("16"));
    }

    #[test]
    fn render_welcome_back_contains_handle() {
        let msg = render_welcome_back("DarkAngel", Some("2026-01-25"), 47);
        assert!(msg.contains("DarkAngel"));
        assert!(msg.contains("2026-01-25"));
        assert!(msg.contains("47"));
    }

    #[test]
    fn render_welcome_back_first_login() {
        let msg = render_welcome_back("NewUser", None, 1);
        assert!(msg.contains("NewUser"));
        assert!(msg.contains("First login"));
    }

    // --- Async state machine tests (require DB) ---

    #[cfg(test)]
    mod async_tests {
        use super::*;
        use sqlx::sqlite::SqlitePoolOptions;

        async fn setup_test_db() -> SqlitePool {
            let pool = SqlitePoolOptions::new()
                .connect("sqlite::memory:")
                .await
                .expect("connect to in-memory db");

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY,
                    handle TEXT UNIQUE NOT NULL,
                    handle_lower TEXT UNIQUE NOT NULL,
                    email TEXT UNIQUE NOT NULL,
                    email_verified INTEGER NOT NULL DEFAULT 0,
                    password_hash TEXT NOT NULL,
                    real_name TEXT,
                    location TEXT,
                    signature TEXT,
                    bio TEXT,
                    user_level TEXT NOT NULL DEFAULT 'User',
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    last_login TEXT,
                    total_logins INTEGER NOT NULL DEFAULT 0,
                    total_time_minutes INTEGER NOT NULL DEFAULT 0,
                    messages_sent INTEGER NOT NULL DEFAULT 0,
                    games_played INTEGER NOT NULL DEFAULT 0,
                    daily_time_used INTEGER NOT NULL DEFAULT 0,
                    banked_time INTEGER NOT NULL DEFAULT 0,
                    last_daily_reset TEXT
                )",
            )
            .execute(&pool)
            .await
            .expect("create users table");

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS sessions (
                    token TEXT PRIMARY KEY,
                    user_id INTEGER NOT NULL,
                    node_id INTEGER,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    last_activity TEXT NOT NULL DEFAULT (datetime('now')),
                    expires_at TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
                )",
            )
            .execute(&pool)
            .await
            .expect("create sessions table");

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS login_attempts (
                    id INTEGER PRIMARY KEY,
                    handle TEXT NOT NULL,
                    ip_address TEXT,
                    attempted_at TEXT NOT NULL DEFAULT (datetime('now')),
                    success INTEGER NOT NULL DEFAULT 0
                )",
            )
            .execute(&pool)
            .await
            .expect("create login_attempts table");

            pool
        }

        fn test_config() -> Config {
            Config {
                server: crate::config::ServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: 3000,
                },
                terminal: crate::config::TerminalConfig { cols: 80, rows: 24 },
                services: vec![],
                auth: crate::config::AuthConfig::default(),
                connection: crate::config::ConnectionConfig::default(),
                email: None,
                menu: crate::menu::MenuConfig::default(),
                time_limits: crate::config::TimeLimitsConfig::default(),
            }
        }

        async fn create_verified_user(pool: &SqlitePool, handle: &str, password: &str) {
            let hash = crate::auth::password::hash_password(password).unwrap();
            crate::db::user::create_user(pool, handle, &format!("{}@test.com", handle.to_lowercase()), &hash)
                .await
                .expect("create user");
            // Mark email as verified
            sqlx::query("UPDATE users SET email_verified = 1 WHERE handle = ?")
                .bind(handle)
                .execute(pool)
                .await
                .expect("verify email");
        }

        #[tokio::test]
        async fn empty_handle_rejected() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = LoginFlow::new();

            let result = flow.handle_input("", &pool, &config).await;
            assert!(matches!(result, LoginResult::Error(ref msg) if msg.contains("empty")));
        }

        #[tokio::test]
        async fn new_triggers_switch_to_registration() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = LoginFlow::new();

            let result = flow.handle_input("new", &pool, &config).await;
            assert_eq!(result, LoginResult::SwitchToRegistration);
        }

        #[tokio::test]
        async fn new_case_insensitive() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = LoginFlow::new();

            let result = flow.handle_input("NEW", &pool, &config).await;
            assert_eq!(result, LoginResult::SwitchToRegistration);
        }

        #[tokio::test]
        async fn unknown_handle_shows_error() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = LoginFlow::new();

            let result = flow.handle_input("NonExistent", &pool, &config).await;
            assert!(matches!(result, LoginResult::Error(ref msg) if msg.contains("not found")));
        }

        #[tokio::test]
        async fn unverified_account_rejected() {
            let pool = setup_test_db().await;
            let config = test_config();

            // Create unverified user
            let hash = crate::auth::password::hash_password("password").unwrap();
            crate::db::user::create_user(&pool, "Unverified", "unv@test.com", &hash)
                .await
                .expect("create user");

            let mut flow = LoginFlow::new();
            let result = flow.handle_input("Unverified", &pool, &config).await;
            assert!(matches!(result, LoginResult::Error(ref msg) if msg.contains("not verified")));
        }

        #[tokio::test]
        async fn valid_handle_advances_to_password() {
            let pool = setup_test_db().await;
            let config = test_config();
            create_verified_user(&pool, "TestUser", "goodpassword").await;

            let mut flow = LoginFlow::new();
            let result = flow.handle_input("TestUser", &pool, &config).await;
            assert_eq!(result, LoginResult::Continue);
            assert!(flow.needs_password_mask());
        }

        #[tokio::test]
        async fn wrong_password_returns_error() {
            let pool = setup_test_db().await;
            let config = test_config();
            create_verified_user(&pool, "TestUser", "goodpassword").await;

            let mut flow = LoginFlow::new();
            flow.handle_input("TestUser", &pool, &config).await;
            let result = flow.handle_input("wrongpassword", &pool, &config).await;
            assert!(matches!(result, LoginResult::Error(ref msg) if msg.contains("Invalid password")));
        }

        #[tokio::test]
        async fn correct_password_returns_success() {
            let pool = setup_test_db().await;
            let config = test_config();
            create_verified_user(&pool, "TestUser", "goodpassword").await;

            let mut flow = LoginFlow::new();
            flow.handle_input("TestUser", &pool, &config).await;
            let result = flow.handle_input("goodpassword", &pool, &config).await;
            assert!(
                matches!(result, LoginResult::Success { ref handle, .. } if handle == "TestUser"),
                "expected Success, got {:?}",
                result
            );
        }

        #[tokio::test]
        async fn lockout_after_max_attempts() {
            let pool = setup_test_db().await;
            let mut config = test_config();
            config.auth.max_login_attempts = 3;
            create_verified_user(&pool, "LockUser", "goodpassword").await;

            let mut flow = LoginFlow::new();

            // Attempt wrong passwords
            for _ in 0..3 {
                flow.handle_input("LockUser", &pool, &config).await;
                let result = flow.handle_input("wrong", &pool, &config).await;
                if matches!(result, LoginResult::Locked) {
                    return; // Test passed
                }
            }

            panic!("Expected Locked result after max attempts");
        }
    }
}
