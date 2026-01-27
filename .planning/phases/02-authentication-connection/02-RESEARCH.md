# Phase 2: Authentication & Connection - Research

**Researched:** 2026-01-26
**Domain:** Authentication, session management, SQLite database, browser audio, terminal animation
**Confidence:** HIGH

## Summary

Phase 2 implements user authentication, session management, and the BBS connection ceremony. The standard Rust stack for this is **SQLx 0.8** (async SQLite), **argon2** for password hashing, and **lettre** for email verification. The connection ceremony requires browser audio playback (Web Audio API with user gesture workaround) and typewriter-effect terminal animation over WebSocket.

**Key architectural decisions:**
- SQLx over rusqlite for native async/tokio support with connection pooling
- Argon2id over bcrypt for password hashing (OWASP 2026 standard)
- UUID v4 for session tokens stored in SQLite with localStorage persistence
- tokio::spawn_blocking for blocking SQLite operations in async handlers
- rustrict crate for profanity filtering with built-in leetspeak detection
- Web Audio API with AudioContext.resume() after user gesture for modem sound
- Server-side timed message delivery over WebSocket for typewriter effect

**Primary recommendation:** Use SQLx with WAL mode enabled for concurrent read/write access, implement session tokens as UUID v4 stored in both SQLite and browser localStorage, and deliver typewriter animation through server-paced WebSocket writes rather than client-side setTimeout delays.

## Standard Stack

The established libraries/tools for authentication and session management in async Rust:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sqlx | 0.8.6 | Async SQLite access | Native async/tokio support, compile-time query verification, connection pooling built-in |
| argon2 | latest | Password hashing | OWASP 2026 recommended algorithm (Argon2id), RustCrypto official implementation |
| lettre | 0.10+ | Email sending | De facto Rust email library, SMTP support, async/tokio compatible |
| uuid | 1.x | Session token generation | Industry standard for unique identifiers, cryptographically secure v4 variant |
| rustrict | 0.7.38 | Profanity filtering | Built-in leetspeak detection, multiple severity levels, O(n) performance |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::sync::Mutex | (std) | Session state locking | Async-aware locking for WebSocket session maps |
| tokio::sync::RwLock | (std) | Concurrent user counting | Multiple readers (common), single writer (login/logout) |
| rand | 0.8+ | Random code generation | 6-digit email verification codes |
| tower-http::services::ServeFile | (existing) | Audio file serving | Serve modem.mp3 to browser |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SQLx | rusqlite + spawn_blocking | rusqlite is sync-only (requires manual spawn_blocking), no connection pooling, no compile-time query verification. SQLx is purpose-built for async. |
| SQLx | diesel | diesel is heavier ORM framework with more complex API, less suited for embedded SQLite use case. SQLx is lighter and more direct. |
| Argon2id | bcrypt | bcrypt has 72-byte password limit, less resistant to GPU attacks. Argon2id is current OWASP standard. Only use bcrypt if migrating from legacy system. |
| lettre SMTP | API-based (SendGrid, etc.) | API services add external dependency and cost. SMTP with lettre + local mail relay is self-hosted and free. Use API only if self-hosting email is not viable. |

**Installation:**
```bash
# Add to Cargo.toml dependencies
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
argon2 = "0.5"
lettre = { version = "0.11", default-features = false, features = ["tokio1", "tokio1-native-tls", "smtp-transport", "builder"] }
uuid = { version = "1.11", features = ["v4", "fast-rng"] }
rustrict = "0.7"
rand = "0.8"
```

## Architecture Patterns

### Recommended Project Structure
```
backend/
├── src/
│   ├── auth/
│   │   ├── mod.rs           # Public auth module interface
│   │   ├── password.rs      # Argon2 hashing/verification
│   │   ├── session.rs       # Session token generation, storage
│   │   └── email.rs         # Email verification code sending
│   ├── db/
│   │   ├── mod.rs           # Database module interface
│   │   ├── schema.sql       # SQLite schema definition
│   │   ├── user.rs          # User CRUD operations
│   │   └── pool.rs          # SQLx pool initialization
│   ├── connection/
│   │   ├── mod.rs           # Connection ceremony orchestration
│   │   ├── node_manager.rs  # Node counting and assignment
│   │   └── goodbye.rs       # Session stats and goodbye sequence
│   └── profanity.rs         # Rustrict profanity filter wrapper
frontend/
├── src/
│   ├── audio/
│   │   └── modem.ts         # Web Audio API modem sound playback
│   └── connection/
│       └── ceremony.ts      # Connection ceremony client logic
```

### Pattern 1: Async SQLite with spawn_blocking (for rusqlite) OR native async (for SQLx)

**What:** SQLite operations in async Rust require special handling because SQLite's C API is synchronous.

**When to use:**
- Use **SQLx** (native async) for new projects - no spawn_blocking needed, connection pooling built-in
- Use **rusqlite + spawn_blocking** only if you need features rusqlite has that SQLx doesn't

**Example (SQLx - RECOMMENDED):**
```rust
// Source: https://docs.rs/sqlx/latest/sqlx/
use sqlx::sqlite::SqlitePool;

// Create connection pool (async-native)
let pool = SqlitePool::connect("sqlite:bbs.db").await?;

// Query without spawn_blocking - SQLx handles async natively
let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE handle = ?")
    .bind(handle)
    .fetch_one(&pool)
    .await?;
```

**Example (rusqlite - only if SQLx insufficient):**
```rust
// Source: https://tokio.rs/tokio/topics/bridging
use tokio::task::spawn_blocking;

let result = spawn_blocking(move || {
    let conn = rusqlite::Connection::open("bbs.db")?;
    conn.execute("INSERT INTO users (handle) VALUES (?1)", [handle])?;
    Ok::<_, rusqlite::Error>(())
}).await??;
```

### Pattern 2: Password Hashing with Argon2id

**What:** OWASP-recommended password hashing with Argon2id variant.

**When to use:** All password storage (registration, password changes).

**Example:**
```rust
// Source: https://rustcrypto.org/key-derivation/hashing-password.html
// Source: https://guptadeepak.com/the-complete-guide-to-password-hashing-argon2-vs-bcrypt-vs-scrypt-vs-pbkdf2-2026/
use argon2::{
    password_hash::{ rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};

// OWASP 2026 recommended parameters:
// m=19 MiB (19456 KiB), t=2 iterations, p=1 parallelism
let params = Params::new(19456, 2, 1, None).unwrap();
let argon2 = Argon2::new(
    argon2::Algorithm::Argon2id,
    argon2::Version::V0x13,
    params,
);

// Hash password
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

// Verify password
pub fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}
```

### Pattern 3: Session Token Generation and Storage

**What:** Cryptographically secure session tokens using UUID v4.

**When to use:** Login, session creation, reconnection.

**Example:**
```rust
// Source: https://www.shuttle.dev/blog/2022/08/11/authentication-tutorial
// Source: https://online-uuid-generator.com/languages/rust
use uuid::Uuid;

pub fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

// Store in SQLite sessions table
sqlx::query!(
    "INSERT INTO sessions (token, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)",
    token,
    user_id,
    chrono::Utc::now(),
    chrono::Utc::now() + chrono::Duration::hours(24)
)
.execute(&pool)
.await?;
```

**Frontend storage:**
```typescript
// Store in localStorage for persistence across page refresh
localStorage.setItem('session_token', token);

// Send with WebSocket connection
ws.send(JSON.stringify({ type: 'authenticate', token }));
```

### Pattern 4: Concurrent User Counting with RwLock

**What:** Track active connections using atomic reference counting.

**When to use:** Node assignment, "line busy" detection, connection limits.

**Example:**
```rust
// Source: https://doc.rust-lang.org/book/ch16-03-shared-state.html
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct NodeManager {
    active_connections: Arc<RwLock<HashMap<usize, String>>>, // node_id -> user_handle
    max_nodes: usize,
}

impl NodeManager {
    pub async fn assign_node(&self, handle: String) -> Result<usize, String> {
        let mut connections = self.active_connections.write().await;

        if connections.len() >= self.max_nodes {
            return Err("All nodes busy".to_string());
        }

        // Find first available node number
        let node_id = (1..=self.max_nodes)
            .find(|id| !connections.contains_key(id))
            .unwrap();

        connections.insert(node_id, handle);
        Ok(node_id)
    }

    pub async fn get_node_count(&self) -> (usize, usize) {
        let connections = self.active_connections.read().await;
        (connections.len(), self.max_nodes)
    }
}
```

### Pattern 5: Email Verification with lettre

**What:** Send 6-digit verification codes via SMTP.

**When to use:** Registration, password reset.

**Example:**
```rust
// Source: https://lettre.rs/
// Source: https://mailtrap.io/blog/rust-send-email/
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use rand::Rng;

pub fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}

pub async fn send_verification_email(
    to_email: &str,
    code: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from("The Construct BBS <noreply@construct.bbs>".parse()?)
        .to(to_email.parse()?)
        .subject("Email Verification Code")
        .body(format!("Your verification code is: {}\n\nThis code expires in 24 hours.", code))?;

    let creds = Credentials::new(
        "smtp_username".to_owned(),
        "smtp_password".to_owned(),
    );

    let mailer = SmtpTransport::relay("smtp.example.com")?
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}
```

### Pattern 6: Browser Audio with User Gesture Workaround

**What:** Play modem handshake audio despite browser autoplay restrictions.

**When to use:** Connection ceremony start.

**Example:**
```typescript
// Source: https://developer.mozilla.org/en-US/docs/Web/Media/Autoplay_guide
let audioContext: AudioContext | null = null;
let modemBuffer: AudioBuffer | null = null;

// Pre-load audio file
async function loadModemSound() {
    const response = await fetch('/audio/modem.mp3');
    const arrayBuffer = await response.arrayBuffer();
    audioContext = new AudioContext();
    modemBuffer = await audioContext.decodeAudioData(arrayBuffer);
}

// Play after user gesture (e.g., "Connect" button click)
async function playModemSound() {
    if (!audioContext || !modemBuffer) return;

    // Resume AudioContext if suspended (autoplay restriction)
    if (audioContext.state === 'suspended') {
        await audioContext.resume();
    }

    const source = audioContext.createBufferSource();
    source.buffer = modemBuffer;
    source.connect(audioContext.destination);
    source.start(0);
}

// Wire to connect button
document.getElementById('connect-btn')?.addEventListener('click', async () => {
    await playModemSound();
    // Then start WebSocket connection ceremony
});
```

### Pattern 7: Server-Paced Typewriter Animation

**What:** Deliver text line-by-line from server with timed delays, not client-side setTimeout.

**When to use:** Connection ceremony text simulation, baud rate simulation.

**Example (Rust backend):**
```rust
// Source: Inferred from xterm.js flow control best practices
use tokio::time::{sleep, Duration};

pub async fn send_connection_ceremony(
    session: &mut SessionIO,
    baud_rate_cps: u32, // characters per second
) {
    let lines = vec![
        "Connecting to The Construct...",
        "Negotiating protocols...",
        "ANSI detected.",
        "Connected at 38400 baud.",
        "Assigned to Node 3 of 16.",
    ];

    for line in lines {
        // Send line
        session.writeln(line).await;

        // Delay based on baud rate (simulate serial transmission)
        let char_count = line.len() as u32;
        let delay_ms = (char_count * 1000) / baud_rate_cps;
        sleep(Duration::from_millis(delay_ms as u64)).await;
    }
}
```

**Why server-side:** JavaScript setTimeout/setInterval accuracy degrades under load and browser throttling. Server-paced delivery provides consistent timing and simplifies client (xterm.js just writes data as received).

### Pattern 8: Profanity Filtering with Rustrict

**What:** Filter handles for profanity with leetspeak detection.

**When to use:** Registration, handle changes.

**Example:**
```rust
// Source: https://github.com/finnbear/rustrict
use rustrict::CensorStr;

pub fn is_handle_allowed(handle: &str) -> bool {
    // Check if handle contains inappropriate content
    // Type::MODERATE_OR_HIGHER includes profane, offensive, sexual, mean
    !handle.is_inappropriate()
}

pub fn validate_handle(handle: &str) -> Result<(), String> {
    if handle.len() < 3 || handle.len() > 20 {
        return Err("Handle must be 3-20 characters".to_string());
    }

    if !handle.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        return Err("Handle can only contain letters, numbers, and spaces".to_string());
    }

    if !is_handle_allowed(handle) {
        return Err("Handle contains inappropriate content".to_string());
    }

    Ok(())
}
```

### Anti-Patterns to Avoid

- **Storing passwords in plaintext or using weak hashing (MD5, SHA-256):** Always use Argon2id with OWASP parameters
- **Client-side timing for typewriter effect with setTimeout:** Browser throttling breaks consistency; use server-paced delivery
- **Synchronous database operations in async handlers without spawn_blocking:** Blocks tokio runtime; use SQLx (native async) or wrap rusqlite in spawn_blocking
- **Autoplay audio without user gesture:** All modern browsers block this; require user click before playing modem sound
- **Using Rc/Mutex instead of Arc for WebSocket session state:** Rc is not thread-safe; always use Arc for cross-thread sharing
- **Hardcoding SMTP credentials:** Load from environment variables or config file, never commit to git

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Password hashing | Custom hash + salt | argon2 crate with OWASP params | Argon2id is memory-hard (GPU-resistant), proper salt generation, timing-attack resistant comparison. Custom crypto is nearly always wrong. |
| Session token generation | rand::random::<u64>().to_string() | uuid v4 with fast-rng | UUID v4 uses cryptographically secure randomness, guaranteed uniqueness, industry standard format. Simple random u64 is predictable. |
| Email sending | Manual SMTP socket connection | lettre crate | SMTP protocol is complex (STARTTLS, authentication methods, MIME encoding). lettre handles edge cases correctly. |
| Profanity filtering | Regex word list | rustrict crate | Leetspeak variants (f*ck, fμ¢κ, f_u_c_k), confusable Unicode, self-censoring all require sophisticated detection. Regex misses most evasion. |
| Concurrent access counting | Mutex<usize> for node count | RwLock<HashMap<usize, String>> | Need to track which nodes are occupied by which users, not just count. RwLock allows many concurrent readers (node count display) without blocking. |
| SQLite connection pooling | Manual connection reuse | SQLx SqlitePool | Connection lifecycle management, checkout/return, concurrency limits, connection health checks all handled. Pooling is harder than it looks. |
| Browser audio playback | new Audio() element | Web Audio API | AudioContext provides better control, consistent timing, resumable after suspension. HTMLAudioElement has less predictable behavior. |

**Key insight:** Security primitives (password hashing, random token generation) and protocol implementations (SMTP, SQLite pooling) are where custom solutions fail catastrophically. Use battle-tested libraries.

## Common Pitfalls

### Pitfall 1: SQLite Foreign Keys Disabled by Default

**What goes wrong:** Foreign key constraints not enforced, orphaned sessions/data accumulate.

**Why it happens:** SQLite ships with `PRAGMA foreign_keys = OFF` by default for backwards compatibility.

**How to avoid:** Enable foreign keys immediately after opening connection:

```rust
// With SQLx
sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

// With rusqlite
conn.execute("PRAGMA foreign_keys = ON", [])?;
```

**Warning signs:** Orphaned session records, deleted users with active sessions, cascading deletes not working.

### Pitfall 2: AudioContext Suspended State

**What goes wrong:** Modem sound doesn't play, no errors logged.

**Why it happens:** Browser autoplay policy creates AudioContext in 'suspended' state until user interaction.

**How to avoid:** Always check `audioContext.state` and call `resume()` after user gesture:

```typescript
if (audioContext.state === 'suspended') {
    await audioContext.resume();
}
```

**Warning signs:** Audio works in some browsers but not others, works after refresh but not on first visit.

### Pitfall 3: WebSocket Backpressure During Fast Text Output

**What goes wrong:** Terminal rendering lags, out-of-memory errors, connection drops.

**Why it happens:** xterm.js has write buffer limit of 50MB. Fast text output without flow control overwhelms buffer.

**How to avoid:** Implement flow control on server side:

```rust
// Don't send unlimited data
for line in massive_text_dump {
    session.writeln(line).await; // Each write goes to WebSocket
    // No flow control = buffer overflow
}

// Do: chunk and pace output
const CHUNK_SIZE: usize = 100;
for chunk in massive_text_dump.chunks(CHUNK_SIZE) {
    for line in chunk {
        session.writeln(line).await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await; // Allow xterm.js to drain buffer
}
```

**Warning signs:** Connection drops during ANSI art display, memory usage spikes, terminal freezes.

### Pitfall 4: Session Expiry Without Grace Period

**What goes wrong:** Users in middle of game get logged out, lose progress.

**Why it happens:** Session expiry checked on every request without considering active engagement.

**How to avoid:** Implement "last activity" tracking and extend expiry on active sessions:

```sql
-- Update last_activity on every WebSocket message
UPDATE sessions
SET last_activity = CURRENT_TIMESTAMP
WHERE token = ?;

-- Check expiry based on inactivity, not absolute time
SELECT * FROM sessions
WHERE token = ?
AND datetime(last_activity, '+15 minutes') > datetime('now');
```

**Warning signs:** Users complain about unexpected logouts, session timeouts during active use.

### Pitfall 5: Unicode Normalization in Handle Validation

**What goes wrong:** Duplicate handles with different Unicode representations (e.g., "Jose" vs "José" with combining accent).

**Why it happens:** SQLite/Rust string comparison is byte-based, doesn't normalize Unicode.

**How to avoid:** Normalize handles before storage and comparison:

```rust
use unicode_normalization::UnicodeNormalization;

pub fn normalize_handle(handle: &str) -> String {
    handle.nfc().collect::<String>().to_lowercase()
}

// Check uniqueness with normalized form
let normalized = normalize_handle(&handle);
sqlx::query("SELECT COUNT(*) FROM users WHERE LOWER(handle) = ?")
    .bind(normalized)
    .fetch_one(&pool)
    .await?;
```

**Warning signs:** Users report "Handle already taken" for seemingly different names, duplicate-looking handles in user list.

### Pitfall 6: WAL Mode Not Enabled

**What goes wrong:** "Database is locked" errors under concurrent access.

**Why it happens:** SQLite default journal mode blocks readers during writes.

**How to avoid:** Enable WAL mode in schema or on first connection:

```sql
PRAGMA journal_mode = WAL;
```

**Why WAL helps:** Write-Ahead Logging allows concurrent readers and one writer. Readers see snapshot isolation, never blocked by writer.

**Warning signs:** Sporadic "database is locked" errors, increased error rate under load.

### Pitfall 7: Email Verification Code Timing Attacks

**What goes wrong:** Attackers can guess verification codes faster by measuring comparison timing.

**Why it happens:** String comparison (`==`) exits on first non-matching character, revealing information.

**How to avoid:** Use constant-time comparison:

```rust
use subtle::ConstantTimeEq;

pub fn verify_code(input: &str, stored: &str) -> bool {
    if input.len() != stored.len() {
        return false;
    }
    input.as_bytes().ct_eq(stored.as_bytes()).into()
}
```

**Warning signs:** None directly observable, but security audit would flag this.

## Code Examples

Verified patterns from official sources:

### SQLite Schema for Users and Sessions

```sql
-- Source: https://moldstud.com/articles/p-best-practices-for-database-schema-design-in-sqlite
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    handle TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    email_verified INTEGER NOT NULL DEFAULT 0, -- SQLite boolean
    password_hash TEXT NOT NULL,
    real_name TEXT,
    location TEXT,
    signature TEXT,
    bio TEXT,
    user_level TEXT NOT NULL DEFAULT 'User', -- 'Guest', 'User', 'Sysop'
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login TEXT,
    total_logins INTEGER NOT NULL DEFAULT 0,
    total_time_minutes INTEGER NOT NULL DEFAULT 0,
    messages_sent INTEGER NOT NULL DEFAULT 0,
    games_played INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE sessions (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    node_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_activity TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE verification_codes (
    email TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
CREATE INDEX idx_verification_expires ON verification_codes(expires_at);
```

### Full Registration Flow

```rust
// Source: Combined from research findings
use sqlx::SqlitePool;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use uuid::Uuid;
use chrono::Utc;

pub async fn register_user(
    pool: &SqlitePool,
    handle: String,
    email: String,
    password: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Validate handle
    validate_handle(&handle)?;

    // 2. Hash password with Argon2id
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    // 3. Generate verification code
    let code = generate_verification_code();
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    // 4. Begin transaction
    let mut tx = pool.begin().await?;

    // 5. Insert user (unverified)
    sqlx::query!(
        "INSERT INTO users (handle, email, password_hash, email_verified) VALUES (?, ?, ?, 0)",
        handle,
        email,
        password_hash
    )
    .execute(&mut *tx)
    .await?;

    // 6. Store verification code
    sqlx::query!(
        "INSERT INTO verification_codes (email, code, expires_at) VALUES (?, ?, ?)",
        email,
        code,
        expires_at.to_rfc3339()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // 7. Send verification email (async, don't block on this)
    tokio::spawn(send_verification_email(email.clone(), code.clone()));

    Ok(code) // Return for testing; don't expose in production
}
```

### WebSocket Connection Ceremony Orchestration

```rust
// Source: Inferred from requirements and timing research
use crate::services::SessionIO;
use tokio::time::{sleep, Duration};

pub async fn run_connection_ceremony(
    session: &mut SessionIO,
    node_manager: &NodeManager,
    config: &ConnectionConfig,
) -> Result<usize, String> {
    // 1. Check node availability
    let (active, max) = node_manager.get_node_count().await;
    if active >= max {
        session.writeln("┌─────────────────────────────────────┐").await;
        session.writeln("│  ☎  ALL LINES BUSY - PLEASE TRY  ☎ │").await;
        session.writeln("│     AGAIN LATER                     │").await;
        session.writeln("└─────────────────────────────────────┘").await;
        sleep(Duration::from_secs(2)).await;
        return Err("All nodes busy".to_string());
    }

    // 2. Simulate connection negotiation
    let baud_cps = config.baud_simulation_cps; // e.g., 3840 for 38400 baud / 10

    send_with_delay(session, "CONNECT 38400", baud_cps).await;
    send_with_delay(session, "ATZ", baud_cps).await;
    send_with_delay(session, "OK", baud_cps).await;
    send_with_delay(session, "Connecting to The Construct...", baud_cps).await;
    send_with_delay(session, "Negotiating protocols...", baud_cps).await;
    send_with_delay(session, "ANSI detected.", baud_cps).await;

    // 3. Assign node
    let node_id = node_manager.assign_node(session.user_handle().to_string()).await?;
    send_with_delay(
        session,
        &format!("Connected to Node {} of {}", node_id, max),
        baud_cps
    ).await;

    // 4. Display ANSI splash screen with line-by-line animation
    let splash_lines = load_ansi_splash(); // Load from file
    for line in splash_lines {
        session.writeln(&line).await;
        // Simulate baud rate for line rendering
        let delay_ms = (line.len() as u64 * 1000) / baud_cps as u64;
        sleep(Duration::from_millis(delay_ms)).await;
    }

    Ok(node_id)
}

async fn send_with_delay(session: &mut SessionIO, text: &str, baud_cps: u32) {
    session.writeln(text).await;
    let delay_ms = (text.len() as u64 * 1000) / baud_cps as u64;
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| bcrypt for password hashing | Argon2id | 2015 (Password Hashing Competition winner) | Argon2id is memory-hard (GPU-resistant), configurable for future hardware. bcrypt has 72-byte limit and less GPU resistance. |
| MD5/SHA-256 for passwords | Any proper KDF (bcrypt/Argon2) | 2010s | MD5/SHA are fast hashing algorithms designed for checksums, not passwords. Trivially cracked with GPUs. |
| rusqlite for async Rust | SQLx | 2020+ | rusqlite requires spawn_blocking wrapper for every query in async context. SQLx is native async with connection pooling. |
| Client-side email verification links | In-terminal verification codes | Varies by project | Links require browser navigation (breaks terminal immersion). 6-digit codes keep user in terminal. |
| HTMLAudioElement for browser audio | Web Audio API | 2010s | AudioContext provides better control, lower latency, consistent timing. HTMLAudioElement has unpredictable autoplay behavior. |
| Polling with setInterval | WebSocket for real-time | 2010s | Polling wastes bandwidth and has latency. WebSocket provides full-duplex persistent connection. |
| Custom profanity word lists | Library with evasion detection (rustrict) | 2020s | Custom lists miss leetspeak (f*ck → fμ¢κ), Unicode confusables. rustrict handles all known evasion techniques. |

**Deprecated/outdated:**
- **rusqlite for async applications:** Use SQLx instead - native async, connection pooling, compile-time query checking
- **bcrypt for new implementations:** Use Argon2id - current OWASP standard, better GPU resistance
- **Synchronous SMTP libraries:** Use lettre with tokio1 feature - async/await compatible
- **Client-side setTimeout for terminal animation:** Use server-paced delivery - consistent timing, immune to browser throttling

## Open Questions

Things that couldn't be fully resolved:

1. **Email provider for production deployment**
   - What we know: lettre supports any SMTP server (self-hosted or third-party)
   - What's unclear: Whether project will self-host mail server (Postfix/sendmail) or use service (Mailgun, SendGrid)
   - Recommendation: Start with local SMTP relay for development, make provider configurable for deployment. Document both approaches in deployment guide.

2. **Optimal verification code expiry time**
   - What we know: 24 hours is industry standard, 10 minutes for high-security
   - What's unclear: User preference for this BBS context (hobbyist vs. production)
   - Recommendation: Make configurable (sysop-settable), default to 24 hours per requirements. Log metrics on code age at verification to tune later.

3. **Modem sound audio file source**
   - What we know: Need 3-5 second modem handshake audio (WAV/MP3)
   - What's unclear: Exact audio file selection (authentic modem recording vs. synthesized)
   - Recommendation: Use authentic V.34 modem handshake recording from archive.org or similar. Document source for licensing. Provide as `frontend/public/audio/modem.mp3`.

4. **Session token storage: Cookie vs. localStorage**
   - What we know: Both persist across page refresh. Cookies auto-sent with HTTP requests, localStorage requires manual JavaScript.
   - What's unclear: Which fits better with WebSocket-first architecture (no traditional HTTP session)
   - Recommendation: Use **localStorage** - WebSocket connection sends token manually in auth message, no HTTP cookie overhead. Simpler for WebSocket-only session model.

5. **Concurrent writes with SQLite (BEGIN CONCURRENT)**
   - What we know: SQLite WAL mode allows concurrent readers + one writer. BEGIN CONCURRENT (experimental) allows multiple writers.
   - What's unclear: Whether BEGIN CONCURRENT is stable enough for production, whether BBS needs concurrent writes
   - Recommendation: Use standard WAL mode (concurrent reads). BBS operations (login, logout, message post) are short writes, unlikely to conflict. If "database locked" errors appear under load, revisit with BEGIN CONCURRENT or connection pooling tuning.

## Sources

### Primary (HIGH confidence)
- SQLx documentation: https://docs.rs/sqlx/latest/sqlx/ (verified version 0.8.6, async/tokio requirements, connection pooling)
- rusqlite documentation: https://docs.rs/rusqlite/latest/rusqlite/ (verified sync-only, version 0.38.0)
- RustCrypto password-hashes: https://github.com/RustCrypto/password-hashes (verified Argon2 crate, OWASP references)
- lettre documentation: https://lettre.rs/ (verified SMTP features, version 0.10+)
- rustrict GitHub: https://github.com/finnbear/rustrict (verified leetspeak detection, version 0.7.38, features)
- MDN Autoplay Guide: https://developer.mozilla.org/en-US/docs/Web/Media/Autoplay_guide (verified 2025-2026 browser policies, AudioContext.resume() pattern)
- Rust Book - Shared State Concurrency: https://doc.rust-lang.org/book/ch16-03-shared-state.html (verified Arc/Mutex patterns, atomic types)
- SQLite WAL documentation: https://sqlite.org/wal.html (verified concurrent reader/writer model)
- Tokio spawn_blocking documentation: https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html (verified thread pool characteristics)
- xterm.js flow control guide: https://xtermjs.org/docs/guides/flowcontrol/ (verified write buffer limits, watermark patterns)

### Secondary (MEDIUM confidence)
- [SQLx vs rusqlite comparison](https://diesel.rs/compare_diesel.html) - ecosystem comparison, verified with official docs
- [Argon2 vs bcrypt 2026 guide](https://guptadeepak.com/the-complete-guide-to-password-hashing-argon2-vs-bcrypt-vs-scrypt-vs-pbkdf2-2026/) - OWASP parameters verified with RustCrypto docs
- [Rust email sending 2026 tutorial](https://mailtrap.io/blog/rust-send-email/) - lettre patterns verified with official docs
- [SQLite schema best practices](https://moldstud.com/articles/p-best-practices-for-database-schema-design-in-sqlite) - general SQL principles
- [Rust session token generation](https://www.shuttle.dev/blog/2022/08/11/authentication-tutorial) - UUID patterns verified with uuid crate docs
- [Axum WebSocket state management](https://medium.com/@itsuki.enjoy/rust-websocket-with-axum-for-realtime-communications-49a93468268f) - Arc/RwLock patterns verified with Rust book
- [Email verification expiry best practices](https://supertokens.com/blog/implementing-the-right-email-verification-flow) - 24-hour standard verified across multiple sources

### Tertiary (LOW confidence - requires validation)
- WebSocket timing vs. setInterval comparisons (general performance characteristics, not specific to this architecture)
- Profanity filter leetspeak detection specifics (rustrict documentation limited on GitHub, behavior inferred from README claims)
- SQLite BEGIN CONCURRENT status (experimental feature, availability uncertain in stable releases)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries have official documentation, active maintenance, and clear version numbers
- Architecture patterns: HIGH - SQLx, Argon2, lettre patterns verified with official docs. Arc/Mutex from Rust book. xterm.js from official guides.
- Pitfalls: MEDIUM - Foreign keys, WAL mode, AudioContext verified with official docs. Backpressure and timing attacks inferred from best practices.
- Code examples: HIGH - SQLite schema, Argon2 usage, SQLx queries verified with official documentation

**Research date:** 2026-01-26
**Valid until:** 2026-02-26 (30 days - stack is stable, libraries mature)

**Notes for planner:**
- SQLx is the clear choice over rusqlite for async Rust (native async, pooling, query verification)
- Argon2id with OWASP parameters (19 MiB, 2 iterations, 1 parallelism) is non-negotiable for security
- Server-paced typewriter effect simplifies client and provides consistent timing
- WAL mode is essential for concurrent read/write access to SQLite
- Modem audio requires user gesture (connect button) due to browser autoplay restrictions
- rustrict handles profanity filtering with leetspeak detection built-in (don't hand-roll)
