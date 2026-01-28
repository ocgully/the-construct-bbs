CREATE TABLE IF NOT EXISTS users (
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
    created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    last_login TEXT,
    total_logins INTEGER NOT NULL DEFAULT 0,
    total_time_minutes INTEGER NOT NULL DEFAULT 0,
    messages_sent INTEGER NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0,
    daily_time_used INTEGER NOT NULL DEFAULT 0,
    banked_time INTEGER NOT NULL DEFAULT 0,
    last_daily_reset TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    node_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    last_activity TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS verification_codes (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL,
    code TEXT NOT NULL,
    code_type TEXT NOT NULL DEFAULT 'registration',
    created_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    expires_at TEXT NOT NULL,
    used INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS login_attempts (
    id INTEGER PRIMARY KEY,
    handle TEXT NOT NULL,
    ip_address TEXT,
    attempted_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    success INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS session_history (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    handle TEXT NOT NULL,
    login_time TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    logout_time TEXT,
    duration_minutes INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY,
    sender_id INTEGER NOT NULL,
    recipient_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    sent_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    is_read INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipient_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_verification_expires ON verification_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_login_attempts_handle ON login_attempts(handle);
CREATE INDEX IF NOT EXISTS idx_users_handle_lower ON users(handle_lower);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_session_history_login ON session_history(login_time);
CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_id, sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id, sent_at DESC)
