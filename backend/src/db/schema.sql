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
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login TEXT,
    total_logins INTEGER NOT NULL DEFAULT 0,
    total_time_minutes INTEGER NOT NULL DEFAULT 0,
    messages_sent INTEGER NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    node_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_activity TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS verification_codes (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL,
    code TEXT NOT NULL,
    code_type TEXT NOT NULL DEFAULT 'registration',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    used INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS login_attempts (
    id INTEGER PRIMARY KEY,
    handle TEXT NOT NULL,
    ip_address TEXT,
    attempted_at TEXT NOT NULL DEFAULT (datetime('now')),
    success INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_verification_expires ON verification_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_login_attempts_handle ON login_attempts(handle);
CREATE INDEX IF NOT EXISTS idx_users_handle_lower ON users(handle_lower);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)
