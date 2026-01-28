use sqlx::sqlite::SqlitePool;

use crate::auth::email::{generate_verification_code, send_verification_email};
use crate::auth::password::hash_password;
use crate::auth::validation::{validate_email, validate_handle, validate_password};
use crate::config::Config;
use crate::db::user::{create_user, email_exists, handle_exists};
use crate::db::verification::store_verification_code;
use crate::terminal::{AnsiWriter, Color};

/// The stages of the interactive registration flow.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrationState {
    EnterHandle,
    EnterEmail,
    EnterPassword,
    ConfirmPassword,
    EnterVerificationCode,
    Complete,
}

/// Result of processing one step of registration input.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrationResult {
    /// Show next prompt (state advanced successfully).
    Continue,
    /// Show error message, then re-show current prompt.
    Error(String),
    /// Show a message, then continue to next prompt.
    Message(String),
    /// Registration complete -- contains the new user's ID.
    Complete(i64),
    /// Registration failed permanently (e.g., too many verification attempts).
    Failed(String),
}

/// Interactive registration flow as a state machine.
///
/// This is NOT a Service trait implementation. It is a special-purpose flow
/// that the Session drives directly, because:
/// - Registration needs async DB access (Service trait methods are sync)
/// - Registration needs password masking (requires special terminal handling)
/// - Registration is a pre-login flow, not a post-login service
pub struct RegistrationFlow {
    state: RegistrationState,
    handle: Option<String>,
    email: Option<String>,
    password: Option<String>,
    verification_code: Option<String>,
    attempts: u32,
    input_buffer: String,
}

impl RegistrationFlow {
    /// Create a new registration flow starting at handle entry.
    pub fn new() -> Self {
        Self {
            state: RegistrationState::EnterHandle,
            handle: None,
            email: None,
            password: None,
            verification_code: None,
            attempts: 0,
            input_buffer: String::new(),
        }
    }

    /// Return the prompt text for the current state.
    pub fn current_prompt(&self) -> &str {
        match self.state {
            RegistrationState::EnterHandle => "Choose your handle: ",
            RegistrationState::EnterEmail => "Enter your email address: ",
            RegistrationState::EnterPassword => "Choose a password: ",
            RegistrationState::ConfirmPassword => "Confirm password: ",
            RegistrationState::EnterVerificationCode => "Enter 6-digit verification code: ",
            RegistrationState::Complete => "",
        }
    }

    /// Whether the current state requires password masking (asterisk echo).
    pub fn needs_password_mask(&self) -> bool {
        matches!(
            self.state,
            RegistrationState::EnterPassword | RegistrationState::ConfirmPassword
        )
    }

    /// Return the current state.
    pub fn state(&self) -> &RegistrationState {
        &self.state
    }

    /// Maximum input length for the current state.
    fn max_input_length(&self) -> usize {
        match self.state {
            RegistrationState::EnterHandle => 20,
            RegistrationState::EnterEmail => 254,
            RegistrationState::EnterPassword | RegistrationState::ConfirmPassword => 128,
            RegistrationState::EnterVerificationCode => 6,
            RegistrationState::Complete => 0,
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
                // Erase the last displayed character: move back, overwrite with space, move back
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

    /// Process a complete line of input for the current registration state.
    ///
    /// This is the core state machine. Each state validates input and advances
    /// to the next state on success.
    pub async fn handle_input(
        &mut self,
        input: &str,
        pool: &SqlitePool,
        config: &Config,
    ) -> RegistrationResult {
        match self.state {
            RegistrationState::EnterHandle => self.handle_enter_handle(input, pool).await,
            RegistrationState::EnterEmail => self.handle_enter_email(input, pool).await,
            RegistrationState::EnterPassword => self.handle_enter_password(input),
            RegistrationState::ConfirmPassword => {
                self.handle_confirm_password(input, pool, config).await
            }
            RegistrationState::EnterVerificationCode => {
                self.handle_enter_verification_code(input, pool).await
            }
            RegistrationState::Complete => RegistrationResult::Complete(0),
        }
    }

    // --- State handlers ---

    async fn handle_enter_handle(
        &mut self,
        input: &str,
        pool: &SqlitePool,
    ) -> RegistrationResult {
        let trimmed = input.trim();

        if let Err(msg) = validate_handle(trimmed) {
            return RegistrationResult::Error(msg);
        }

        match handle_exists(pool, trimmed).await {
            Ok(true) => {
                return RegistrationResult::Error("That handle is already taken".to_string());
            }
            Ok(false) => {}
            Err(e) => {
                return RegistrationResult::Error(format!("Database error: {}", e));
            }
        }

        self.handle = Some(trimmed.to_string());
        self.state = RegistrationState::EnterEmail;
        RegistrationResult::Continue
    }

    async fn handle_enter_email(&mut self, input: &str, pool: &SqlitePool) -> RegistrationResult {
        let trimmed = input.trim().to_lowercase();

        if let Err(msg) = validate_email(&trimmed) {
            return RegistrationResult::Error(msg);
        }

        match email_exists(pool, &trimmed).await {
            Ok(true) => {
                return RegistrationResult::Error(
                    "That email is already registered".to_string(),
                );
            }
            Ok(false) => {}
            Err(e) => {
                return RegistrationResult::Error(format!("Database error: {}", e));
            }
        }

        self.email = Some(trimmed);
        self.state = RegistrationState::EnterPassword;
        RegistrationResult::Continue
    }

    fn handle_enter_password(&mut self, input: &str) -> RegistrationResult {
        if let Err(msg) = validate_password(input) {
            return RegistrationResult::Error(msg);
        }

        self.password = Some(input.to_string());
        self.state = RegistrationState::ConfirmPassword;
        RegistrationResult::Continue
    }

    async fn handle_confirm_password(
        &mut self,
        input: &str,
        pool: &SqlitePool,
        config: &Config,
    ) -> RegistrationResult {
        let stored_password = match &self.password {
            Some(p) => p.clone(),
            None => {
                self.state = RegistrationState::EnterPassword;
                return RegistrationResult::Error("Internal error, please re-enter password".to_string());
            }
        };

        if input != stored_password {
            // Mismatch -- go back to password entry
            self.password = None;
            self.state = RegistrationState::EnterPassword;
            return RegistrationResult::Error("Passwords do not match".to_string());
        }

        // Hash password using spawn_blocking (CPU-intensive Argon2)
        let password_clone = stored_password.clone();
        let hash_result =
            tokio::task::spawn_blocking(move || hash_password(&password_clone)).await;

        let password_hash = match hash_result {
            Ok(Ok(hash)) => hash,
            Ok(Err(e)) => {
                return RegistrationResult::Error(format!("Failed to hash password: {}", e));
            }
            Err(e) => {
                return RegistrationResult::Error(format!("Internal error: {}", e));
            }
        };

        // Create user in DB (email_verified = 0)
        let handle = self.handle.as_ref().unwrap();
        let email = self.email.as_ref().unwrap();

        let user = match create_user(pool, handle, email, &password_hash).await {
            Ok(u) => u,
            Err(e) => {
                return RegistrationResult::Error(format!("Failed to create account: {}", e));
            }
        };

        // Generate and store verification code
        let code = generate_verification_code();
        let expiry_hours = config.auth.verification_code_expiry_hours;
        if let Err(e) =
            store_verification_code(pool, email, &code, "registration", expiry_hours).await
        {
            return RegistrationResult::Error(format!("Failed to store verification code: {}", e));
        }

        // Send verification email (async spawn, doesn't block)
        let email_config = config.email.clone();
        let email_addr = email.clone();
        let code_clone = code.clone();
        let _ = send_verification_email(&email_config, &email_addr, &code_clone, "The Construct BBS").await;

        self.verification_code = Some(code);
        self.state = RegistrationState::EnterVerificationCode;

        let msg = format!("Verification code sent to {}", email);
        // Store user_id for later (Complete result)
        // We'll re-fetch from self.email during verification
        let _ = user.id; // user_id is available but we validate via DB
        RegistrationResult::Message(msg)
    }

    async fn handle_enter_verification_code(
        &mut self,
        input: &str,
        pool: &SqlitePool,
    ) -> RegistrationResult {
        let trimmed = input.trim();
        let email = match &self.email {
            Some(e) => e.clone(),
            None => {
                return RegistrationResult::Failed("Internal error: no email stored".to_string());
            }
        };

        match crate::db::verification::validate_verification_code(
            pool,
            &email,
            trimmed,
            "registration",
        )
        .await
        {
            Ok(true) => {
                self.state = RegistrationState::Complete;
                // Fetch user to get ID
                match crate::db::user::find_user_by_email(pool, &email).await {
                    Ok(Some(user)) => RegistrationResult::Complete(user.id),
                    Ok(None) => RegistrationResult::Failed("User not found after verification".to_string()),
                    Err(e) => RegistrationResult::Failed(format!("Database error: {}", e)),
                }
            }
            Ok(false) => {
                self.attempts += 1;
                if self.attempts >= 3 {
                    RegistrationResult::Failed(
                        "Too many failed attempts. Please try registering again.".to_string(),
                    )
                } else {
                    RegistrationResult::Error(format!(
                        "Invalid code, try again ({} of 3 attempts)",
                        self.attempts
                    ))
                }
            }
            Err(e) => RegistrationResult::Error(format!("Database error: {}", e)),
        }
    }
}

/// Render the ANSI art header for the registration screen.
pub fn render_registration_header() -> String {
    let mut w = AnsiWriter::new();

    w.set_fg(Color::LightCyan);
    w.bold();
    // Top border
    w.writeln("\u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}");
    // Middle line with text
    w.write_str("\u{2502}");
    w.set_fg(Color::White);
    w.bold();
    w.write_str("                    NEW USER REGISTRATION                     ");
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("\u{2502}");
    // Bottom border
    w.writeln("\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}");
    w.reset_color();
    w.writeln("");

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RegistrationFlow unit tests ---

    #[test]
    fn new_flow_starts_at_enter_handle() {
        let flow = RegistrationFlow::new();
        assert_eq!(*flow.state(), RegistrationState::EnterHandle);
    }

    #[test]
    fn current_prompt_matches_state() {
        let mut flow = RegistrationFlow::new();
        assert_eq!(flow.current_prompt(), "Choose your handle: ");

        flow.state = RegistrationState::EnterEmail;
        assert_eq!(flow.current_prompt(), "Enter your email address: ");

        flow.state = RegistrationState::EnterPassword;
        assert_eq!(flow.current_prompt(), "Choose a password: ");

        flow.state = RegistrationState::ConfirmPassword;
        assert_eq!(flow.current_prompt(), "Confirm password: ");

        flow.state = RegistrationState::EnterVerificationCode;
        assert_eq!(flow.current_prompt(), "Enter 6-digit verification code: ");

        flow.state = RegistrationState::Complete;
        assert_eq!(flow.current_prompt(), "");
    }

    #[test]
    fn needs_password_mask_only_for_password_states() {
        let mut flow = RegistrationFlow::new();
        assert!(!flow.needs_password_mask()); // EnterHandle

        flow.state = RegistrationState::EnterEmail;
        assert!(!flow.needs_password_mask());

        flow.state = RegistrationState::EnterPassword;
        assert!(flow.needs_password_mask());

        flow.state = RegistrationState::ConfirmPassword;
        assert!(flow.needs_password_mask());

        flow.state = RegistrationState::EnterVerificationCode;
        assert!(!flow.needs_password_mask());
    }

    #[test]
    fn handle_char_echoes_printable_character() {
        let mut flow = RegistrationFlow::new();
        let echo = flow.handle_char('a');
        assert_eq!(echo, Some("a".to_string()));
        assert_eq!(flow.input_buffer, "a");
    }

    #[test]
    fn handle_char_echoes_asterisk_for_password() {
        let mut flow = RegistrationFlow::new();
        flow.state = RegistrationState::EnterPassword;

        let echo = flow.handle_char('s');
        assert_eq!(echo, Some("*".to_string()));
        assert_eq!(flow.input_buffer, "s");
    }

    #[test]
    fn handle_char_backspace_erases_last_char() {
        let mut flow = RegistrationFlow::new();
        flow.handle_char('a');
        flow.handle_char('b');
        assert_eq!(flow.input_buffer, "ab");

        let echo = flow.handle_char('\x7f'); // DEL (backspace)
        assert_eq!(echo, Some("\x08 \x08".to_string()));
        assert_eq!(flow.input_buffer, "a");
    }

    #[test]
    fn handle_char_backspace_on_empty_returns_none() {
        let mut flow = RegistrationFlow::new();
        let echo = flow.handle_char('\x7f');
        assert_eq!(echo, None);
    }

    #[test]
    fn handle_char_bs_char_works_too() {
        let mut flow = RegistrationFlow::new();
        flow.handle_char('x');
        let echo = flow.handle_char('\x08'); // BS
        assert_eq!(echo, Some("\x08 \x08".to_string()));
        assert_eq!(flow.input_buffer, "");
    }

    #[test]
    fn handle_char_enter_returns_none() {
        let mut flow = RegistrationFlow::new();
        flow.handle_char('a');
        assert_eq!(flow.handle_char('\r'), None);
        assert_eq!(flow.handle_char('\n'), None);
        // Buffer should still contain 'a' (Enter doesn't modify buffer)
        assert_eq!(flow.input_buffer, "a");
    }

    #[test]
    fn handle_char_control_chars_ignored() {
        let mut flow = RegistrationFlow::new();
        assert_eq!(flow.handle_char('\x01'), None); // SOH
        assert_eq!(flow.handle_char('\x1b'), None); // ESC
        assert_eq!(flow.input_buffer, "");
    }

    #[test]
    fn take_input_clears_buffer() {
        let mut flow = RegistrationFlow::new();
        flow.handle_char('h');
        flow.handle_char('i');
        let input = flow.take_input();
        assert_eq!(input, "hi");
        assert_eq!(flow.input_buffer, "");
    }

    #[test]
    fn handle_char_password_confirm_also_masks() {
        let mut flow = RegistrationFlow::new();
        flow.state = RegistrationState::ConfirmPassword;

        let echo = flow.handle_char('p');
        assert_eq!(echo, Some("*".to_string()));
    }

    #[test]
    fn render_registration_header_contains_title() {
        let header = render_registration_header();
        assert!(
            header.contains("NEW USER REGISTRATION"),
            "header should contain title text"
        );
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
                "CREATE TABLE IF NOT EXISTS verification_codes (
                    id INTEGER PRIMARY KEY,
                    email TEXT NOT NULL,
                    code TEXT NOT NULL,
                    code_type TEXT NOT NULL DEFAULT 'registration',
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    expires_at TEXT NOT NULL,
                    used INTEGER NOT NULL DEFAULT 0
                )",
            )
            .execute(&pool)
            .await
            .expect("create verification_codes table");

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
                mail: crate::config::MailConfig::default(),
                chat: crate::config::ChatConfig::default(),
            }
        }

        #[tokio::test]
        async fn handle_validation_rejects_short_handle() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            let result = flow.handle_input("ab", &pool, &config).await;
            assert!(matches!(result, RegistrationResult::Error(_)));
            assert_eq!(*flow.state(), RegistrationState::EnterHandle);
        }

        #[tokio::test]
        async fn handle_validation_accepts_valid_handle() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            let result = flow.handle_input("DarkAngel", &pool, &config).await;
            assert_eq!(result, RegistrationResult::Continue);
            assert_eq!(*flow.state(), RegistrationState::EnterEmail);
            assert_eq!(flow.handle, Some("DarkAngel".to_string()));
        }

        #[tokio::test]
        async fn duplicate_handle_rejected() {
            let pool = setup_test_db().await;
            let config = test_config();

            // Create existing user
            crate::db::user::create_user(&pool, "TakenUser", "other@test.com", "hash")
                .await
                .expect("create existing user");

            let mut flow = RegistrationFlow::new();
            let result = flow.handle_input("TakenUser", &pool, &config).await;
            assert!(matches!(result, RegistrationResult::Error(ref msg) if msg.contains("already taken")));
            assert_eq!(*flow.state(), RegistrationState::EnterHandle);
        }

        #[tokio::test]
        async fn email_validation_rejects_invalid() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            // Advance to email state
            flow.handle_input("TestUser", &pool, &config).await;
            assert_eq!(*flow.state(), RegistrationState::EnterEmail);

            let result = flow.handle_input("not-an-email", &pool, &config).await;
            assert!(matches!(result, RegistrationResult::Error(_)));
            assert_eq!(*flow.state(), RegistrationState::EnterEmail);
        }

        #[tokio::test]
        async fn email_validation_accepts_valid() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            flow.handle_input("TestUser", &pool, &config).await;
            let result = flow.handle_input("test@example.com", &pool, &config).await;
            assert_eq!(result, RegistrationResult::Continue);
            assert_eq!(*flow.state(), RegistrationState::EnterPassword);
            assert_eq!(flow.email, Some("test@example.com".to_string()));
        }

        #[tokio::test]
        async fn duplicate_email_rejected() {
            let pool = setup_test_db().await;
            let config = test_config();

            crate::db::user::create_user(&pool, "ExistingUser", "taken@test.com", "hash")
                .await
                .expect("create existing user");

            let mut flow = RegistrationFlow::new();
            flow.handle_input("NewUser", &pool, &config).await;
            let result = flow.handle_input("taken@test.com", &pool, &config).await;
            assert!(matches!(result, RegistrationResult::Error(ref msg) if msg.contains("already registered")));
        }

        #[tokio::test]
        async fn password_validation_rejects_short() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            flow.handle_input("TestUser", &pool, &config).await;
            flow.handle_input("test@example.com", &pool, &config).await;

            let result = flow.handle_input("short", &pool, &config).await;
            assert!(matches!(result, RegistrationResult::Error(_)));
            assert_eq!(*flow.state(), RegistrationState::EnterPassword);
        }

        #[tokio::test]
        async fn password_mismatch_goes_back() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            flow.handle_input("TestUser", &pool, &config).await;
            flow.handle_input("test@example.com", &pool, &config).await;
            flow.handle_input("goodpassword", &pool, &config).await;
            assert_eq!(*flow.state(), RegistrationState::ConfirmPassword);

            let result = flow.handle_input("wrongpassword", &pool, &config).await;
            assert!(matches!(result, RegistrationResult::Error(ref msg) if msg.contains("do not match")));
            assert_eq!(*flow.state(), RegistrationState::EnterPassword);
            assert!(flow.password.is_none());
        }

        #[tokio::test]
        async fn full_registration_flow() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            // Step 1: Handle
            let r = flow.handle_input("CoolUser", &pool, &config).await;
            assert_eq!(r, RegistrationResult::Continue);

            // Step 2: Email
            let r = flow.handle_input("cool@example.com", &pool, &config).await;
            assert_eq!(r, RegistrationResult::Continue);

            // Step 3: Password
            let r = flow.handle_input("securepass", &pool, &config).await;
            assert_eq!(r, RegistrationResult::Continue);

            // Step 4: Confirm password -- should create user and send code
            let r = flow.handle_input("securepass", &pool, &config).await;
            assert!(
                matches!(r, RegistrationResult::Message(ref msg) if msg.contains("cool@example.com")),
                "expected Message with email, got {:?}",
                r
            );
            assert_eq!(*flow.state(), RegistrationState::EnterVerificationCode);
            assert!(flow.verification_code.is_some());

            // Step 5: Enter verification code
            let code = flow.verification_code.clone().unwrap();
            let r = flow.handle_input(&code, &pool, &config).await;
            assert!(
                matches!(r, RegistrationResult::Complete(id) if id > 0),
                "expected Complete with user_id > 0, got {:?}",
                r
            );
            assert_eq!(*flow.state(), RegistrationState::Complete);

            // Verify user exists in DB with email_verified = 1
            let user = crate::db::user::find_user_by_handle(&pool, "CoolUser")
                .await
                .expect("find user")
                .expect("user should exist");
            assert_eq!(user.email_verified, 1);
        }

        #[tokio::test]
        async fn verification_code_too_many_attempts() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            // Advance through registration
            flow.handle_input("TestUser2", &pool, &config).await;
            flow.handle_input("test2@example.com", &pool, &config).await;
            flow.handle_input("securepass", &pool, &config).await;
            flow.handle_input("securepass", &pool, &config).await;
            assert_eq!(*flow.state(), RegistrationState::EnterVerificationCode);

            // 3 wrong attempts
            let r = flow.handle_input("000000", &pool, &config).await;
            assert!(matches!(r, RegistrationResult::Error(_)));
            let r = flow.handle_input("000001", &pool, &config).await;
            assert!(matches!(r, RegistrationResult::Error(_)));
            let r = flow.handle_input("000002", &pool, &config).await;
            assert!(matches!(r, RegistrationResult::Failed(ref msg) if msg.contains("Too many")));
        }

        #[tokio::test]
        async fn email_is_lowercased() {
            let pool = setup_test_db().await;
            let config = test_config();
            let mut flow = RegistrationFlow::new();

            flow.handle_input("TestUser3", &pool, &config).await;
            flow.handle_input("Test@EXAMPLE.COM", &pool, &config).await;

            assert_eq!(flow.email, Some("test@example.com".to_string()));
        }
    }
}
