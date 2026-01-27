# Architecture Patterns: Web-Based BBS

**Domain:** Retro BBS with modern web terminal
**Researched:** 2026-01-26
**Confidence:** MEDIUM (based on knowledge of Rust patterns, WebSocket terminals, and classic BBS architecture)

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser Client                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  xterm.js    │  │ Audio Player │  │  WebSocket Client    │  │
│  │  Terminal    │  │ (modem SFX)  │  │                      │  │
│  └──────┬───────┘  └──────────────┘  └──────────┬───────────┘  │
│         │                                        │               │
└─────────┼────────────────────────────────────────┼───────────────┘
          │ ANSI/Terminal Data                     │
          │                                        │ WebSocket
          └────────────────────────────────────────┘
                                                   │
┌─────────────────────────────────────────────────┼───────────────┐
│                      Rust Server                │               │
│                                                 │               │
│  ┌──────────────────────────────────────────────▼────────────┐  │
│  │           WebSocket Handler (tokio-tungstenite)           │  │
│  │  - Connection lifecycle                                   │  │
│  │  - Message routing                                        │  │
│  │  - Keep-alive / heartbeat                                 │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │              Session Manager                              │  │
│  │  - User authentication                                    │  │
│  │  - Session state (logged in, current service, etc)        │  │
│  │  - Time limits (session duration, daily limits)           │  │
│  │  - Concurrent user caps                                   │  │
│  │  - Idle timeout                                           │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │         Service Router / Dispatcher                       │  │
│  │  - Routes user input to active service                    │  │
│  │  - Manages service transitions                            │  │
│  │  - Handles service lifecycle                              │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │           Service Registry (Plugin System)                │  │
│  │  - Static registration via trait objects                  │  │
│  │  - Service metadata (name, description, enabled)          │  │
│  │  - Service factory functions                              │  │
│  └───────────────┬──────────────────────────┬────────────────┘  │
│                  │                          │                   │
│     ┌────────────▼────┐         ┌──────────▼─────────┐         │
│     │  Core Services  │         │   Game Services    │         │
│     ├─────────────────┤         ├────────────────────┤         │
│     │ - Login         │         │ - LORD (turn-based)│         │
│     │ - Main Menu     │         │ - Drug Wars        │         │
│     │ - Message Board │         │ - Adventure Games  │         │
│     │ - File Browser  │         │ - Real-time Multi  │         │
│     │ - User List     │         │                    │         │
│     │ - Chat          │         │                    │         │
│     └────────┬────────┘         └──────────┬─────────┘         │
│              │                             │                   │
│              │ All implement Service trait │                   │
│              └──────────────┬──────────────┘                   │
│                             │                                  │
│  ┌──────────────────────────▼───────────────────────────────┐  │
│  │              Service Trait Interface                      │  │
│  │                                                           │  │
│  │  trait Service {                                          │  │
│  │    fn on_enter(&mut self, ctx: &ServiceContext);         │  │
│  │    fn on_input(&mut self, input: &[u8], ctx: &mut        │  │
│  │                ServiceContext) -> ServiceTransition;      │  │
│  │    fn on_exit(&mut self, ctx: &ServiceContext);          │  │
│  │    fn metadata(&self) -> &ServiceMetadata;                │  │
│  │  }                                                        │  │
│  │                                                           │  │
│  │  ServiceContext provides:                                 │  │
│  │    - ANSI writer                                          │  │
│  │    - Database access                                      │  │
│  │    - User session info                                    │  │
│  │    - System config                                        │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │           ANSI Terminal Writer                            │  │
│  │  - ANSI escape code generation                            │  │
│  │  - Screen buffering                                       │  │
│  │  - Color/formatting helpers                               │  │
│  │  - Common UI patterns (menus, prompts)                    │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │         Database Layer (SQLite via sqlx)                  │  │
│  │  - User accounts                                          │  │
│  │  - Messages                                               │  │
│  │  - Game state (per-game tables)                           │  │
│  │  - Session history                                        │  │
│  │  - System configuration                                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| WebSocket Handler | Transport layer, connection lifecycle | Session Manager (owns sessions) |
| Session Manager | Authentication, time limits, user caps | WebSocket Handler (receives), Service Router (delegates) |
| Service Router | Dispatch input to active service, manage transitions | Session Manager (uses context), Service Registry (queries), Services (invokes) |
| Service Registry | Plugin registration, metadata, enable/disable | Service Router (queried), Config (reads enabled list) |
| Service (trait) | Business logic for a specific feature | Database Layer (via context), ANSI Writer (via context) |
| ANSI Writer | Terminal rendering, escape codes | Services (used by), WebSocket (outputs to) |
| Database Layer | Persistent storage | Services (queried by) |

### Data Flow

**Incoming (User Input):**
```
Browser (keypress)
  → WebSocket message
    → WebSocket Handler (deserializes)
      → Session Manager (validates session, checks timeouts)
        → Service Router (identifies active service)
          → Active Service.on_input()
            → ANSI Writer (generates output)
              → WebSocket Handler (sends to client)
```

**Outgoing (Server Output):**
```
Service logic
  → ANSI Writer (ctx.write("text"), ctx.move_cursor(), etc)
    → Terminal buffer
      → WebSocket Handler (flush on await point)
        → Browser
          → xterm.js (renders ANSI)
```

**Service Transition:**
```
Service.on_input() returns ServiceTransition::Switch(new_service_id)
  → Service Router
    → Current service.on_exit()
    → Service Registry (create new service instance)
    → New service.on_enter()
```

**Session Lifecycle:**
```
WebSocket connect
  → Session Manager (create Session)
    → Service Router (start with LoginService)
      → LoginService.on_enter() (display welcome screen)

User authenticates
  → LoginService validates credentials
    → ServiceTransition::Switch("main_menu")
      → MainMenuService.on_enter()

Idle timeout or time limit reached
  → Session Manager detects
    → Service Router
      → Current service.on_exit()
        → WebSocket Handler (close connection)
```

## Patterns to Follow

### Pattern 1: Trait-Based Plugin System (Static Registration)

**What:** Each service is a type implementing the `Service` trait. Registration happens at compile time via a registry macro or explicit function calls in `main()`.

**When:** You want type safety, no dynamic loading complexity, and services are known at build time.

**Why this over dynamic loading:**
- Rust's dynamic loading (libloading) requires careful ABI management
- Cross-platform DLL/SO loading is complex
- Most BBS features are known upfront
- Can still enable/disable via config
- Easier to debug and maintain

**Example:**
```rust
// In lib.rs or services/mod.rs
pub trait Service: Send {
    fn metadata(&self) -> &ServiceMetadata;
    fn on_enter(&mut self, ctx: &mut ServiceContext);
    fn on_input(&mut self, input: &[u8], ctx: &mut ServiceContext) -> ServiceTransition;
    fn on_exit(&mut self, ctx: &ServiceContext);
}

pub enum ServiceTransition {
    Continue,           // Stay in current service
    Switch(String),     // Switch to service by ID
    Disconnect,         // End session
}

pub struct ServiceMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,  // Set from config
}

// Registry pattern
pub struct ServiceRegistry {
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn Service>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };

        // Static registration
        registry.register("login", || Box::new(LoginService::new()));
        registry.register("main_menu", || Box::new(MainMenuService::new()));
        registry.register("messages", || Box::new(MessageBoardService::new()));
        registry.register("lord", || Box::new(LordGameService::new()));

        registry
    }

    pub fn create(&self, service_id: &str) -> Option<Box<dyn Service>> {
        self.factories.get(service_id).map(|factory| factory())
    }
}
```

### Pattern 2: Service Context for Dependency Injection

**What:** Pass a mutable `ServiceContext` to every service method. Context contains everything a service needs.

**When:** Always. This avoids tight coupling and makes services testable.

**Example:**
```rust
pub struct ServiceContext<'a> {
    pub user: &'a User,
    pub session: &'a Session,
    pub db: &'a DatabasePool,
    pub writer: &'a mut AnsiWriter,
    pub config: &'a SystemConfig,
}

// Services use it like:
impl Service for MainMenuService {
    fn on_input(&mut self, input: &[u8], ctx: &mut ServiceContext) -> ServiceTransition {
        match input {
            b"M" => {
                // Read messages from database
                let messages = ctx.db.get_messages(ctx.user.id).await?;

                // Write to terminal
                ctx.writer.clear_screen();
                ctx.writer.write_line("Message Board");
                ctx.writer.write_line("=============");
                for msg in messages {
                    ctx.writer.write_line(&format!("{}: {}", msg.from, msg.subject));
                }

                ServiceTransition::Continue
            }
            b"G" => ServiceTransition::Switch("game_menu".to_string()),
            b"Q" => ServiceTransition::Disconnect,
            _ => ServiceTransition::Continue,
        }
    }
}
```

### Pattern 3: ANSI Writer with Buffering

**What:** A struct that builds ANSI escape sequences and buffers output until flushed to WebSocket.

**When:** Always for terminal output. Don't let services emit raw strings.

**Example:**
```rust
pub struct AnsiWriter {
    buffer: Vec<u8>,
}

impl AnsiWriter {
    pub fn clear_screen(&mut self) {
        self.buffer.extend_from_slice(b"\x1b[2J\x1b[H");
    }

    pub fn move_cursor(&mut self, row: u16, col: u16) {
        self.buffer.extend_from_slice(format!("\x1b[{};{}H", row, col).as_bytes());
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.buffer.extend_from_slice(format!("\x1b[{};{}m", fg.ansi_code(), bg.ansi_code()).as_bytes());
    }

    pub fn write(&mut self, text: &str) {
        self.buffer.extend_from_slice(text.as_bytes());
    }

    pub fn write_line(&mut self, text: &str) {
        self.write(text);
        self.buffer.extend_from_slice(b"\r\n");
    }

    pub fn draw_box(&mut self, x: u16, y: u16, width: u16, height: u16) {
        // IBM PC extended ASCII box drawing
        // Or use Unicode box drawing for modern terminals
    }

    pub fn flush(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.buffer)
    }
}
```

### Pattern 4: Configuration-Driven Service Enable/Disable

**What:** Load service enable/disable flags from config file or database. Registry filters disabled services.

**When:** Always. Sysop shouldn't need to recompile to enable/disable features.

**Example:**
```toml
# config.toml
[system]
max_concurrent_users = 10
session_timeout_minutes = 30

[services]
login = true
main_menu = true
messages = true
file_browser = false  # Disabled
chat = true

[services.lord]
enabled = true
daily_turns = 20

[services.drug_wars]
enabled = false  # Disabled
```

```rust
pub struct ServiceRegistry {
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn Service>>>,
    config: ServiceConfig,
}

impl ServiceRegistry {
    pub fn new(config: ServiceConfig) -> Self {
        // Register all services, but respect config
        // ...
    }

    pub fn is_enabled(&self, service_id: &str) -> bool {
        self.config.services.get(service_id)
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    pub fn create(&self, service_id: &str) -> Option<Box<dyn Service>> {
        if !self.is_enabled(service_id) {
            return None;
        }
        self.factories.get(service_id).map(|factory| factory())
    }
}
```

### Pattern 5: Session Management with Tokio Tasks

**What:** Each WebSocket connection spawns a tokio task. Task owns the session state and service router.

**When:** Always for concurrent connection handling.

**Example:**
```rust
// In WebSocket handler
async fn handle_connection(
    ws: WebSocket,
    session_manager: Arc<SessionManager>,
    registry: Arc<ServiceRegistry>,
    db: DatabasePool,
) {
    let (mut sender, mut receiver) = ws.split();

    // Create session
    let session = session_manager.create_session().await;
    let user = None; // Set after login

    // Create service router
    let mut router = ServiceRouter::new(registry.clone(), session.id);
    router.switch_to("login").await;

    // Main loop
    loop {
        tokio::select! {
            // Handle incoming messages
            Some(Ok(msg)) = receiver.next() => {
                if let Message::Binary(data) = msg {
                    // Check session timeout
                    if session_manager.is_expired(&session).await {
                        let _ = sender.send(Message::Text("Session expired".into())).await;
                        break;
                    }

                    // Route to active service
                    let mut ctx = ServiceContext {
                        user: user.as_ref().unwrap(),
                        session: &session,
                        db: &db,
                        writer: &mut AnsiWriter::new(),
                        config: &config,
                    };

                    let transition = router.handle_input(&data, &mut ctx).await;

                    // Send output
                    let output = ctx.writer.flush();
                    if !output.is_empty() {
                        let _ = sender.send(Message::Binary(output)).await;
                    }

                    // Handle transition
                    match transition {
                        ServiceTransition::Switch(service_id) => {
                            router.switch_to(&service_id).await;
                        }
                        ServiceTransition::Disconnect => break,
                        ServiceTransition::Continue => {}
                    }
                }
            }

            // Handle idle timeout
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                if session_manager.is_idle(&session).await {
                    let _ = sender.send(Message::Text("Idle timeout".into())).await;
                    break;
                }
            }
        }
    }

    // Cleanup
    session_manager.remove_session(&session).await;
}
```

### Pattern 6: xterm.js WebSocket Protocol

**What:** xterm.js expects binary WebSocket messages containing UTF-8 terminal data (including ANSI escape codes).

**When:** Always for browser terminal integration.

**Protocol:**
- Client → Server: Binary messages with user keypresses (UTF-8 encoded)
- Server → Client: Binary messages with terminal output (ANSI escape codes + UTF-8 text)
- xterm.js handles all rendering, scrollback, cursor positioning

**Example client integration:**
```javascript
// In browser
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';

const term = new Terminal({
    cursorBlink: true,
    fontFamily: 'IBM VGA 8x16',  // Retro font
    fontSize: 16,
    theme: {
        background: '#000000',
        foreground: '#c0c0c0',
    }
});

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

term.open(document.getElementById('terminal'));
fitAddon.fit();

// WebSocket connection
const ws = new WebSocket('ws://localhost:8080/ws');
ws.binaryType = 'arraybuffer';

ws.onmessage = (event) => {
    const data = new Uint8Array(event.data);
    term.write(data);
};

term.onData((data) => {
    // Send user input to server
    ws.send(new TextEncoder().encode(data));
});

// Audio player for modem sounds
const audioContext = new AudioContext();
const modemConnect = new Audio('/sounds/modem_connect.mp3');
modemConnect.play();  // On WebSocket connect
```

### Pattern 7: Game Engine Abstraction

**What:** Each game type has a common interface, but different internal state machines.

**When:** Building multiple game types (turn-based, real-time, adventure).

**Game Categories:**

**Turn-Based RPG (LORD, Usurper):**
```rust
pub struct TurnBasedGameState {
    player_stats: PlayerStats,
    daily_turns_remaining: u32,
    last_action_time: SystemTime,
    location: LocationId,
    inventory: Vec<Item>,
}

impl Service for TurnBasedRPG {
    fn on_input(&mut self, input: &[u8], ctx: &mut ServiceContext) -> ServiceTransition {
        // Load state from DB
        let mut state = ctx.db.load_game_state::<TurnBasedGameState>(
            ctx.user.id, "lord"
        ).await?;

        // Check turn limit
        if state.daily_turns_remaining == 0 {
            ctx.writer.write_line("No turns remaining today. Come back tomorrow!");
            return ServiceTransition::Switch("main_menu".to_string());
        }

        // Process action
        match input {
            b"F" => self.handle_fight(&mut state, ctx),
            b"I" => self.handle_inn(&mut state, ctx),
            b"H" => self.handle_healer(&mut state, ctx),
            _ => {}
        }

        // Save state
        ctx.db.save_game_state(ctx.user.id, "lord", &state).await?;

        ServiceTransition::Continue
    }
}
```

**Trading Game (Drug Wars):**
```rust
pub struct TradingGameState {
    cash: u64,
    debt: u64,
    inventory: HashMap<ItemId, u32>,
    location: LocationId,
    day: u32,
    max_days: u32,
}

// Similar pattern, but focused on economy simulation
```

**Real-Time Multiplayer (Acrophobia):**
```rust
pub struct RealTimeGameState {
    room_id: RoomId,
    phase: GamePhase,  // Waiting, Writing, Voting, Results
    phase_deadline: SystemTime,
    players: Vec<PlayerId>,
}

// Requires broadcast to all players in room
// Needs tokio::sync::broadcast channel for room events
impl Service for RealTimeMultiplayer {
    fn on_enter(&mut self, ctx: &mut ServiceContext) {
        // Subscribe to room broadcast channel
        self.subscribe_to_room(ctx.user.room_id);
    }

    // Handle both user input AND room events
    async fn tick(&mut self, ctx: &mut ServiceContext) {
        // Check for phase transitions
        if SystemTime::now() > self.state.phase_deadline {
            self.advance_phase(ctx).await;
            self.broadcast_to_room(ctx).await;
        }
    }
}
```

**MUD (Kyrandia):**
```rust
pub struct AdventureGameState {
    location: LocationId,
    inventory: Vec<ItemId>,
    flags: HashMap<String, bool>,  // Story flags
    visited_locations: HashSet<LocationId>,
}

// Parser-based input
impl Service for AdventureGame {
    fn on_input(&mut self, input: &[u8], ctx: &mut ServiceContext) -> ServiceTransition {
        let command = String::from_utf8_lossy(input).to_lowercase();
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts.get(0) {
            Some(&"go") => self.handle_movement(parts.get(1), ctx),
            Some(&"take") => self.handle_take(parts.get(1), ctx),
            Some(&"use") => self.handle_use(parts.get(1), parts.get(3), ctx),
            Some(&"look") => self.describe_location(ctx),
            _ => ctx.writer.write_line("I don't understand that."),
        }

        ServiceTransition::Continue
    }
}
```

### Pattern 8: Classic BBS Door Pattern (Modernized)

**What:** Classic BBS doors used drop files (DOOR.SYS, DORINFO1.DEF) and FOSSIL drivers. Modernize with JSON state files and service context.

**Classic pattern:**
- BBS writes drop file with user info
- External door program reads drop file
- Door program reads/writes via FOSSIL driver (serial port emulation)
- Door program writes updated state
- BBS reads updated state

**Modern equivalent:**
```rust
// Drop file becomes ServiceContext
pub struct ServiceContext<'a> {
    pub user: &'a User,           // DOOR.SYS user info
    pub session: &'a Session,     // DOOR.SYS time info
    pub db: &'a DatabasePool,     // Persistent state
    pub writer: &'a mut AnsiWriter, // FOSSIL write
    pub config: &'a SystemConfig,
}

// FOSSIL driver becomes AnsiWriter + input handling
// No need for separate process or IPC

// If you WANT to support external door programs (optional):
pub struct ExternalDoorService {
    door_path: PathBuf,
}

impl Service for ExternalDoorService {
    fn on_enter(&mut self, ctx: &mut ServiceContext) {
        // Write drop file
        let drop_file = DoorSysFile {
            user_name: ctx.user.name.clone(),
            time_remaining: ctx.session.time_remaining(),
            // ... other fields
        };

        std::fs::write("/tmp/DOOR.SYS", drop_file.to_string())?;

        // Spawn door program with PTY
        // Proxy PTY I/O to/from WebSocket
        // This is advanced - only if you need legacy door support
    }
}
```

### Pattern 9: SQLite Schema for Multi-Game State

**What:** Each game gets its own table(s) for state. Common pattern for user-game linkage.

**Example:**
```sql
-- Core tables
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    connected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Generic game state (JSON blob approach)
CREATE TABLE game_states (
    user_id INTEGER NOT NULL,
    game_id TEXT NOT NULL,
    state_json TEXT NOT NULL,  -- Serialized game state
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, game_id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- LORD-specific tables (structured approach)
CREATE TABLE lord_players (
    user_id INTEGER PRIMARY KEY,
    level INTEGER DEFAULT 1,
    experience INTEGER DEFAULT 0,
    gold INTEGER DEFAULT 0,
    hit_points INTEGER DEFAULT 20,
    max_hit_points INTEGER DEFAULT 20,
    strength INTEGER DEFAULT 10,
    defense INTEGER DEFAULT 10,
    charm INTEGER DEFAULT 10,
    daily_fights_remaining INTEGER DEFAULT 10,
    last_turn_reset DATE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE lord_monsters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    level INTEGER NOT NULL,
    hit_points INTEGER NOT NULL,
    strength INTEGER NOT NULL,
    defense INTEGER NOT NULL,
    experience_reward INTEGER NOT NULL,
    gold_reward INTEGER NOT NULL
);

-- Drug Wars tables
CREATE TABLE drug_wars_games (
    user_id INTEGER PRIMARY KEY,
    cash INTEGER DEFAULT 2000,
    debt INTEGER DEFAULT 5000,
    day INTEGER DEFAULT 1,
    location TEXT DEFAULT 'bronx',
    game_over BOOLEAN DEFAULT FALSE,
    score INTEGER DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE drug_wars_inventory (
    user_id INTEGER NOT NULL,
    item_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    PRIMARY KEY (user_id, item_id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Message board tables
CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    from_user_id INTEGER NOT NULL,
    to_user_id INTEGER,  -- NULL for public messages
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP,
    FOREIGN KEY (from_user_id) REFERENCES users(id),
    FOREIGN KEY (to_user_id) REFERENCES users(id)
);

-- System configuration
CREATE TABLE system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT
);

-- Audit log
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    action TEXT NOT NULL,
    details TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**JSON vs Structured:**
- JSON blob (`game_states` table): Fast to build, flexible schema, harder to query
- Structured tables (`lord_players`, etc): Easier to query, more type-safe, more schema work
- Recommendation: Start with JSON for rapid prototyping, migrate to structured for complex games

## Anti-Patterns to Avoid

### Anti-Pattern 1: Services Directly Owning WebSocket Senders

**What goes wrong:** Passing `WebSocketSender` to services creates lifetime and ownership complexity.

**Why bad:**
- Services can't be easily tested (need mock WebSocket)
- Ownership issues with mutable borrows
- Mixing transport layer with business logic

**Instead:** Use `AnsiWriter` that buffers output. Service router flushes buffer to WebSocket.

### Anti-Pattern 2: Blocking I/O in Service Handlers

**What goes wrong:** Calling blocking database or file operations directly in service methods.

**Why bad:**
- Blocks the tokio runtime
- Other connections stall
- Poor scalability

**Instead:** Use `sqlx` with async/await. All service methods should be async or use `tokio::task::spawn_blocking` for CPU-intensive work.

### Anti-Pattern 3: Global Mutable State

**What goes wrong:** Using `static mut` or `lazy_static!` with `Mutex` for game state.

**Why bad:**
- Concurrency bugs
- Hard to test
- Doesn't scale
- Lost data on crash

**Instead:** Store all state in SQLite. Use in-memory caching with `Arc<RwLock<HashMap>>` only for hot paths, with write-through to DB.

### Anti-Pattern 4: String-Based Service IDs Without Registry Validation

**What goes wrong:** Services return `ServiceTransition::Switch("doesnt_exist")` and router silently fails or panics.

**Why bad:**
- Runtime errors instead of compile-time checks
- Hard to debug

**Instead:**
```rust
// Define service IDs as constants
pub mod service_ids {
    pub const LOGIN: &str = "login";
    pub const MAIN_MENU: &str = "main_menu";
    pub const MESSAGES: &str = "messages";
    pub const LORD: &str = "lord";
}

// Registry validates IDs on registration
impl ServiceRegistry {
    pub fn register(&mut self, id: &'static str, factory: ...) {
        assert!(!self.factories.contains_key(id), "Duplicate service ID: {}", id);
        self.factories.insert(id.to_string(), factory);
    }
}

// Router validates on switch
impl ServiceRouter {
    pub async fn switch_to(&mut self, service_id: &str) -> Result<(), ServiceError> {
        if !self.registry.is_enabled(service_id) {
            return Err(ServiceError::Disabled(service_id.to_string()));
        }

        let service = self.registry.create(service_id)
            .ok_or_else(|| ServiceError::NotFound(service_id.to_string()))?;

        // ... switch logic
        Ok(())
    }
}
```

### Anti-Pattern 5: No Input Sanitization

**What goes wrong:** Directly using user input in SQL queries or file paths.

**Why bad:**
- SQL injection
- Path traversal
- Terminal escape sequence injection (ANSI bombs)

**Instead:**
```rust
// Use parameterized queries
sqlx::query("SELECT * FROM users WHERE username = ?")
    .bind(username)
    .fetch_one(&db).await?;

// Validate input length
if input.len() > 1024 {
    ctx.writer.write_line("Input too long");
    return ServiceTransition::Continue;
}

// Strip dangerous escape sequences
fn sanitize_input(input: &[u8]) -> String {
    input.iter()
        .filter(|&&b| b >= 32 || b == b'\r' || b == b'\n')
        .map(|&b| b as char)
        .collect()
}
```

### Anti-Pattern 6: Mixing UI and Business Logic

**What goes wrong:** Game logic interleaved with ANSI drawing code.

**Why bad:**
- Hard to test
- Hard to refactor
- Can't reuse logic

**Instead:**
```rust
// Separate model and view
pub struct LordGame {
    state: LordGameState,
}

impl LordGame {
    // Pure business logic
    pub fn attack_monster(&mut self, monster: &Monster) -> BattleResult {
        // ... calculate damage, update state, return result
    }
}

pub struct LordGameService {
    game: LordGame,
}

impl Service for LordGameService {
    fn on_input(&mut self, input: &[u8], ctx: &mut ServiceContext) -> ServiceTransition {
        match input {
            b"F" => {
                let monster = self.get_random_monster();
                let result = self.game.attack_monster(&monster);

                // View logic
                self.render_battle_result(&result, ctx);
            }
            _ => {}
        }
        ServiceTransition::Continue
    }
}

impl LordGameService {
    // View rendering
    fn render_battle_result(&self, result: &BattleResult, ctx: &mut ServiceContext) {
        ctx.writer.clear_screen();
        ctx.writer.set_color(Color::Red, Color::Black);
        ctx.writer.write_line(&format!("You attack {} for {} damage!", result.monster_name, result.damage));
        // ... more rendering
    }
}
```

## Scalability Considerations

| Concern | At 10 users | At 100 users | At 1000 users |
|---------|-------------|--------------|---------------|
| WebSocket connections | Direct tokio tasks | Same (tokio handles this) | Same (OS may need tuning) |
| Database | SQLite in-process | Same (read-heavy, few writes) | Consider PostgreSQL for write-heavy |
| Session state | In-memory HashMap | Same with RwLock | Redis for multi-process |
| Game state | Load/save per action | Cache hot state in memory | Separate game server processes |
| Concurrent users cap | Config-based limit | Same | Load balancer + sticky sessions |

**SQLite Limits:**
- Handles ~100K reads/sec on SSD
- Writes serialized (one writer at a time)
- Good for read-heavy workloads (BBS is mostly reads)
- Sufficient for 100s of concurrent users
- Use WAL mode for better concurrency

**When to scale beyond single server:**
- 1000+ concurrent users
- Needs HA/failover
- Multi-region

**Scaling path:**
1. Single Rust process, SQLite, in-memory sessions (0-100 users)
2. Add Redis for session cache, keep SQLite (100-500 users)
3. Switch to PostgreSQL, Redis, multiple processes behind load balancer (500+ users)

## Build Order Implications

**Dependency Graph:**
```
1. ANSI Writer (no dependencies)
   ↓
2. Service Trait + Context (depends on ANSI Writer)
   ↓
3. Simple Service (e.g., Echo) (depends on trait)
   ↓
4. Service Registry (depends on trait)
   ↓
5. Service Router (depends on registry + trait)
   ↓
6. Session Manager (depends on router)
   ↓
7. WebSocket Handler (depends on session manager)
   ↓
8. Database Layer (parallel with above)
   ↓
9. Complex Services (depends on database + all above)
   ↓
10. Game Services (depends on complex services)
```

**Suggested Build Phases:**

**Phase 1: Terminal Foundation**
- ANSI Writer
- WebSocket handler (echo server)
- xterm.js client
- Verify round-trip

**Phase 2: Service Framework**
- Service trait
- Service context
- Simple echo service
- Service registry (hard-coded)
- Service router (basic)

**Phase 3: Session Management**
- Session manager
- Time limits
- Concurrent user caps
- Idle timeout

**Phase 4: Database Layer**
- SQLite integration
- User accounts
- Session persistence
- Migration system

**Phase 5: Core Services**
- Login service
- Main menu
- User list
- Simple message board

**Phase 6: Game Framework**
- Generic game state tables
- Turn limit system
- Daily reset logic

**Phase 7: First Game**
- LORD or Drug Wars (simpler than real-time)
- Validates architecture
- Exposes integration issues early

**Phase 8: Advanced Games**
- Real-time multiplayer (broadcast system)
- Adventure games (parser)

**Critical Path:**
- Can't build services without trait (Phase 2)
- Can't build router without registry (Phase 2)
- Can't build games without database (Phase 4)
- Can't test full flow until WebSocket + sessions (Phase 3)

**Parallelization Opportunities:**
- ANSI Writer + Database Layer (independent)
- Individual game services (after Phase 6)
- Frontend work (after Phase 1)

## Modern BBS Architecture References

**Source confidence note:** These recommendations are based on established Rust async patterns (tokio ecosystem), terminal emulation standards (xterm.js is the de facto standard), and classic BBS architecture adapted for modern web. The trait-based plugin system is idiomatic Rust. The WebSocket + terminal pattern is proven by projects like ttyd and Wetty.

**Key differences from 1990s BBS:**
- WebSocket replaces serial/telnet
- xterm.js replaces terminal emulator
- SQLite replaces FOSSIL files
- Trait objects replace .EXE door programs
- Tokio async replaces multi-process
- In-memory sessions replace COM port state

**What's preserved:**
- ANSI terminal UI
- Door game concept (now "services")
- Turn-based gameplay
- Time limits and user caps
- Text-based interaction model
- Sysop configuration

