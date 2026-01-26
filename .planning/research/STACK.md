# Technology Stack

**Project:** The Construct (Web-based BBS)
**Researched:** 2026-01-26
**Confidence:** MEDIUM (based on training data through Jan 2025, requires version verification)

## Recommended Stack

### Core Web Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **axum** | 0.7.x | HTTP + WebSocket server | Type-safe, minimal boilerplate, excellent WebSocket support via tower, integrates seamlessly with tokio ecosystem. Better ergonomics than actix-web for WebSocket upgrades. |
| **tokio** | 1.x | Async runtime | Industry standard async runtime, required by axum. Mature, well-documented, excellent performance. |
| **tower** | 0.4.x | Middleware layer | Provides WebSocket upgrade handlers, request/response middleware. Core to axum's architecture. |
| **tower-http** | 0.5.x | HTTP middleware | CORS, compression, tracing, static file serving. Essential for browser compatibility. |

**Alternatives Considered:**
- **actix-web**: More mature, slightly faster benchmarks, but more boilerplate and less intuitive WebSocket integration
- **warp**: Filter-based API is harder to reason about, declining ecosystem momentum
- **rocket**: Excellent DX but still catching up on async, less flexible for WebSocket patterns

**Recommendation: axum** because it offers the best balance of type safety, WebSocket ergonomics, and ecosystem integration for a real-time BBS application.

### Frontend Terminal

| Technology | Version | Purpose | Why |
|------------|---------|---------|---|
| **xterm.js** | 5.x | Terminal emulator | Industry standard browser terminal. Full ANSI/VT100 support, excellent performance, mobile-friendly with touch handlers. Active development, large community. |
| **xterm-addon-fit** | 0.8.x | Responsive sizing | Auto-fits terminal to container, essential for mobile responsiveness. |
| **xterm-addon-web-links** | 0.9.x | Clickable URLs | Enables hyperlinks in terminal output (useful for help text, external references). |
| **xterm-addon-webgl** | 0.16.x | WebGL rendering | Dramatically improves performance for ANSI art and rapid screen updates. Falls back to canvas if WebGL unavailable. |

**ANSI Art Handling:**
- xterm.js natively supports full ANSI escape sequences including 256-color and TrueColor
- No additional library needed for CP437 if using Unicode equivalents
- Consider `text-encoding` polyfill for legacy CP437 if authentic byte-for-byte BBS art needed

**Mobile Responsiveness:**
- xterm.js has built-in touch support for scrolling
- `fit` addon handles dynamic resizing
- Virtual keyboard handling requires custom implementation (capture focus, prevent native keyboard from covering terminal)

### Database Layer

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **sqlx** | 0.7.x | SQLite ORM | Compile-time query verification, async-native, excellent migration support. Safer than rusqlite, more flexible than diesel. |
| **sqlx-cli** | 0.7.x | Migration tool | Manages schema migrations, database creation. |

**Schema Design Note:** Use sqlx macros (`query!`, `query_as!`) for compile-time SQL verification. This catches schema errors at build time, critical for a modular/pluggable service architecture.

**Alternatives Considered:**
- **diesel**: Excellent type safety, but diesel's DSL is overkill for SQLite and async support still experimental
- **rusqlite**: Synchronous blocking I/O conflicts with async WebSocket patterns, would require separate thread pool
- **sea-orm**: Good async support but heavier weight, more complex for SQLite use case

**Recommendation: sqlx** because compile-time verification + async-first design + migration tooling is ideal for a modular BBS with evolving schema.

### WebSocket Communication

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **axum::extract::ws** | 0.7.x | WebSocket upgrade | Built into axum, leverages tower-http. Type-safe, ergonomic API. |
| **tokio-tungstenite** | 0.21.x | WebSocket protocol | Underlying WebSocket implementation used by axum. Async-native, battle-tested. |
| **serde** + **serde_json** | 1.x | Message serialization | For structured messages (chat, game state). Use newline-delimited JSON for terminal text streams. |

**Architecture Pattern:**
```rust
// WebSocket handler upgrades connection, spawns per-user session task
// Session task owns:
// - WebSocket sender/receiver
// - User state (current door, time remaining)
// - Subscription to broadcast channels (chat, notifications)
```

**For real-time chat:** Use `tokio::sync::broadcast` channel for fan-out messaging
**For multiplayer games:** Use `tokio::sync::mpsc` for per-game-instance coordination

### Audio Playback

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **Web Audio API** | Native browser | Modem sounds, BBS effects | No library needed. Use `AudioContext` + `AudioBuffer` for samples, `OscillatorNode` for synthesized tones. |
| **howler.js** | 2.2.x (optional) | Audio sprite management | Only if you need complex audio sequencing. Probably overkill for modem sounds. |

**Recommendation:** Use raw Web Audio API. Modem sounds are simple:
1. Pre-load WAV/OGG samples (handshake, carrier tone)
2. Trigger via `AudioBufferSourceNode` on WebSocket connect
3. ~10KB total asset size

### Build Tools

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **trunk** | 0.18.x | Rust WASM bundler | If using Rust for frontend (yew/leptos). Handles WASM compilation, asset bundling. |
| **vite** | 5.x | Frontend bundler | If using plain JS/TS frontend. Faster than webpack, excellent dev experience. |

**Recommendation:** Use **vite** for this project. Reasons:
- Terminal UI is mostly xterm.js (JavaScript), no need for Rust frontend
- Vite's dev server has excellent HMR for CSS/JS tweaks
- Smaller learning curve than trunk
- Better integration with npm ecosystem (xterm.js addons)

**Frontend Structure:**
```
frontend/
  src/
    main.ts          # WebSocket connection, xterm.js initialization
    terminal.ts      # Terminal rendering, ANSI handling
    audio.ts         # Modem sounds via Web Audio API
    gameloop.ts      # Client-side game rendering coordination
  public/
    sounds/          # Modem WAV files
    fonts/           # CP437 font if needed
```

### Game Loop Patterns

| Pattern | Technology | Use Case |
|---------|-----------|----------|
| **Turn-based** | State machine in Rust | LORD, Usurper - server authoritative, client displays results |
| **Real-time** | `tokio::time::interval` | Acrophobia timer, Drug Wars countdown - tick at fixed intervals |
| **Multiplayer real-time** | Broadcast channels + game loop task | Multi-user chat, simultaneous gameplay |

**Architecture:**
```rust
// Door game trait
#[async_trait]
trait DoorGame {
    async fn handle_input(&mut self, user_id: UserId, input: String) -> GameResponse;
    async fn tick(&mut self) -> Vec<(UserId, GameEvent)>;  // For real-time games
    fn is_complete(&self) -> bool;
}

// Game instance manager spawns task per game instance
// Task runs game loop, broadcasts events to subscribed WebSocket sessions
```

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **anyhow** | 1.x | Error handling | Application-level error handling (not library code) |
| **thiserror** | 1.x | Custom errors | For door game plugins, service modules |
| **tracing** | 0.1.x | Structured logging | Debug WebSocket connections, game state transitions |
| **tracing-subscriber** | 0.3.x | Log formatting | Console output, file logging |
| **serde** | 1.x | Serialization | Config files, JSON messages, SQLite JSON columns |
| **toml** | 0.8.x | Config parsing | Server configuration (ports, time limits, user caps) |
| **chrono** | 0.4.x | Date/time | User login tracking, time limit enforcement |
| **uuid** | 1.x | Session IDs | WebSocket session identification |
| **rand** | 0.8.x | RNG | Game mechanics (dice rolls, random encounters) |
| **lazy_static** or **once_cell** | 1.x | Static config | Global database pool, broadcast channels |
| **dashmap** | 5.x | Concurrent HashMap | Active WebSocket sessions, game instances |

### Development Tools

| Tool | Version | Purpose |
|------|---------|---------|
| **cargo-watch** | 8.x | Auto-rebuild on file change |
| **cargo-sqlx** | 0.7.x | Database CLI (migrations, query verification) |
| **rust-analyzer** | Latest | LSP for IDE support |

## Installation

### Backend (Rust)

```bash
# Create new project
cargo new bbs --bin
cd bbs

# Core dependencies
cargo add axum@0.7
cargo add tokio --features full
cargo add tower@0.4
cargo add tower-http --features "fs,trace,cors"

# Database
cargo add sqlx --features "runtime-tokio-native-tls,sqlite,migrate"
cargo install sqlx-cli --no-default-features --features sqlite

# WebSocket
cargo add tokio-tungstenite

# Serialization
cargo add serde --features derive
cargo add serde_json

# Utilities
cargo add anyhow
cargo add thiserror
cargo add tracing
cargo add tracing-subscriber
cargo add toml
cargo add chrono
cargo add uuid --features "v4,serde"
cargo add rand
cargo add once_cell
cargo add dashmap
```

### Frontend (JavaScript/TypeScript)

```bash
# Create frontend project
npm create vite@latest frontend -- --template vanilla-ts
cd frontend

# Terminal emulation
npm install xterm
npm install @xterm/addon-fit
npm install @xterm/addon-web-links
npm install @xterm/addon-webgl

# Optional: Audio (only if not using raw Web Audio API)
# npm install howler
```

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Web Framework | axum | actix-web | More boilerplate, less ergonomic WebSocket API |
| Web Framework | axum | warp | Filter-based API harder to reason about, less ecosystem momentum |
| Web Framework | axum | rocket | Still catching up on async, less flexible for WebSockets |
| Database | sqlx | diesel | Diesel DSL overkill for SQLite, async support experimental |
| Database | sqlx | rusqlite | Synchronous I/O conflicts with async WebSocket patterns |
| Terminal | xterm.js | terminus | Less mature, smaller community, fewer addons |
| Frontend Build | vite | webpack | Slower, more complex configuration |
| Frontend Build | vite | trunk | Unnecessary WASM complexity for JS-based terminal UI |

## Architecture Decisions

### Why WebSocket over SSE?
- **Bidirectional:** BBS needs client input AND server push
- **Binary support:** Potential for efficient binary protocols later
- **Game state sync:** Real-time multiplayer needs low-latency bidirectional

### Why SQLite over PostgreSQL?
- **Simplicity:** No separate database server for small-to-medium BBS
- **Portability:** Single file database, easy backup/restore
- **Performance:** Sufficient for hundreds of concurrent users with proper indexing
- **When to migrate:** If >1000 concurrent users or multi-server deployment needed

### Why Async/Await over Threads?
- **WebSocket scalability:** Thousands of idle connections with minimal memory
- **I/O bound:** BBS is mostly waiting on user input, database queries
- **Ecosystem:** Tokio + axum is the Rust async standard

### Modular Door Game Architecture

**Plugin Pattern:**
```rust
// Each door game is a separate crate
// Main BBS binary loads door games via trait objects
// SQLite stores game state in JSON columns (flexible schema per game)

bbs/
  src/
    main.rs              # Axum server, WebSocket handler
    session.rs           # User session management
    door_manager.rs      # Game instance lifecycle

  doors/
    lord/                # LORD clone (separate crate)
    usurper/             # Usurper clone (separate crate)
    drugwars/            # Drug Wars clone (separate crate)
```

**Why JSON columns for game state?**
- Each door game has different state schema
- SQLite JSON functions support queries
- No schema migrations when adding new games
- Drawback: Type safety at runtime not compile-time

## Mobile Responsiveness Strategy

### Terminal Sizing
```typescript
// Use xterm-addon-fit to handle resize
const fitAddon = new FitAddon();
terminal.loadAddon(fitAddon);

// Fit on window resize
window.addEventListener('resize', () => fitAddon.fit());

// Initial fit
fitAddon.fit();
```

### Keyboard Handling
```typescript
// Prevent mobile keyboard from covering terminal
// Use input method composition events
terminal.onData((data) => {
  // Send to WebSocket
  websocket.send(data);
});

// Virtual keyboard: Use contenteditable div overlay
// Capture input, forward to xterm, hide div
```

### Touch Scrolling
- xterm.js handles touch scrolling natively
- No additional library needed

### Font Sizing
- Default xterm.js font size works on mobile
- Consider larger font option (16px) for accessibility
- CP437 fonts must be web-safe or embedded

## Confidence Assessment

| Component | Confidence | Notes |
|-----------|------------|-------|
| axum version | MEDIUM | Version 0.7.x current as of Jan 2025, verify exact latest |
| xterm.js version | MEDIUM | Version 5.x current as of Jan 2025, verify latest stable |
| sqlx version | MEDIUM | Version 0.7.x current as of Jan 2025, verify latest |
| WebSocket approach | HIGH | axum + tokio-tungstenite is standard pattern |
| Terminal emulation | HIGH | xterm.js is industry standard, no viable alternatives |
| SQLite choice | HIGH | Appropriate for BBS use case, well-understood tradeoffs |
| Audio approach | HIGH | Web Audio API is stable, simple for BBS modem sounds |
| Mobile responsiveness | MEDIUM | xterm.js touch support confirmed, virtual keyboard needs testing |
| Game loop patterns | HIGH | tokio patterns well-established for game servers |

## Known Gaps

**Requires verification:**
- Exact current versions of axum, xterm.js, sqlx as of Jan 2026
- xterm.js addon package names (may have moved to `@xterm/` scope)
- Best practices for xterm.js virtual keyboard on iOS Safari
- CP437 font rendering in modern browsers (Unicode vs embedded font)

**Needs phase-specific research:**
- Door game state serialization patterns (research during game implementation phase)
- WebSocket message rate limiting and backpressure handling (research during scaling phase)
- SQLite WAL mode tuning for concurrent WebSocket writes (research during performance phase)

## Version Verification Required

**IMPORTANT:** All version numbers in this document are based on training data through January 2025. Before implementation, verify current versions:

```bash
# Check crates.io for latest versions
cargo search axum
cargo search sqlx
cargo search tokio

# Check npm for latest versions
npm view xterm version
npm view @xterm/addon-fit version
npm view vite version
```

**Update this document with verified versions before roadmap creation.**

## Sources

**Confidence Level: MEDIUM-LOW (Training data only, web verification unavailable)**

This research is based on:
- Rust ecosystem knowledge through January 2025
- xterm.js documentation and community patterns
- Tokio async patterns and best practices
- Prior BBS implementation experience in training data

**REQUIRES VERIFICATION:**
- All version numbers should be verified via crates.io, npm, official docs
- xterm.js mobile keyboard best practices should be verified with current documentation
- axum 0.7 API stability should be confirmed (may have evolved)

**Recommended verification steps:**
1. Check crates.io for current versions of axum, sqlx, tokio
2. Check npm for current xterm.js version and addon package names
3. Review xterm.js GitHub issues for mobile keyboard patterns
4. Verify Web Audio API browser compatibility for target browsers
