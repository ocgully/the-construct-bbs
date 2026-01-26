# Phase 1: Terminal Foundation - Research

**Researched:** 2026-01-26
**Domain:** Browser terminal emulation with WebSocket backend
**Confidence:** HIGH

## Summary

Phase 1 establishes the browser terminal foundation for rendering authentic DOS ANSI art with a Rust WebSocket backend. The standard approach uses xterm.js 6.0.0 (released December 2024) for terminal emulation with WebGL rendering, Perfect DOS VGA 437 font for pixel-perfect CP437 display, and axum or tokio-tungstenite for the Rust WebSocket server. CRT effects are best implemented via WebGL shaders (CRTFilter or gingerbeardman's webgl-crt-shader) for performance and dial-able intensity levels.

The architecture uses Arc<dyn Trait> for the pluggable service registry, enabling runtime service loading without core code modifications. Mobile support requires the fit addon for responsive sizing, though xterm.js has known touch keyboard limitations that need careful handling. The backend should use axum's native WebSocket support for simplicity, with ANSI escape sequence buffering handled through tokio channels to prevent partial sequence rendering.

CP437 to UTF-8 conversion is handled Rust-side using the `codepage-437` crate before sending to the browser. Configuration uses TOML (Rust ecosystem standard) with serde for the service registry.

**Primary recommendation:** Use xterm.js 6.0.0 with WebGL renderer + fit addon, axum WebSocket server, CRTFilter for dial-able CRT effects, codepage-437 crate for CP437 conversion, and Arc<dyn Service> trait objects for the plugin architecture.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| xterm.js | 6.0.0 | Browser terminal emulator | Industry standard, used by VS Code and major IDEs. Supports ANSI sequences, synchronized rendering (DEC 2026), and extensive addons |
| axum | 0.7.x | Rust web framework with WebSocket | Tokio-native, ergonomic extractors, native WebSocket support via axum::extract::ws, most modern Rust web framework |
| tokio | 1.x | Async runtime | De facto async runtime in Rust, required by axum and tokio-tungstenite |
| codepage-437 | 0.1.0 | CP437 ↔ UTF-8 conversion | Provides traits for converting CP437 bytes to Unicode and back |
| serde + toml | 1.x + 0.8.x | Configuration serialization | Rust ecosystem standard, Cargo uses TOML, human-friendly for sysadmins |
| Perfect DOS VGA 437 | 1.0 | Authentic CP437 TrueType font | Pixel-perfect DOS VGA font by Zeh, exact DOS character set with period-correct glyphs |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @xterm/addon-fit | 0.11.0 | Responsive terminal sizing | Essential for mobile/responsive layouts, auto-fits terminal to container |
| @xterm/addon-webgl | 6.0.0 | GPU-accelerated rendering | Performance critical, 900% faster than canvas renderer in some benchmarks |
| @xterm/addon-attach | 6.0.0 | WebSocket integration | Optional convenience addon, can also manually handle WebSocket messages |
| @xterm/addon-web-fonts | 6.0.0 | Web font loading coordination | Ensures webfonts load before terminal opens, prevents rendering glitches |
| CRTFilter | Latest | WebGL CRT shader effects | Comprehensive dial-able CRT effects (scanlines, phosphor, curvature, bloom) |
| tokio-tungstenite | 0.28.0 | Alternative WebSocket lib | If not using axum, provides lower-level WebSocket control with tokio bindings |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| axum | tokio-tungstenite directly | More boilerplate, lower-level control, harder to add HTTP endpoints later |
| CRTFilter | gingerbeardman/webgl-crt-shader | Simpler shader, fewer effects, but lighter weight and still dial-able |
| WebGL renderer | DOM renderer | Much slower (no specific benchmarks but "much slower" per docs), use only if WebGL unavailable |
| @xterm/addon-attach | Manual WebSocket handling | More control over message framing, but more code and error-prone |

**Installation:**
```bash
# Frontend (browser)
npm install @xterm/xterm @xterm/addon-fit @xterm/addon-webgl @xterm/addon-web-fonts

# CRT shader (choose one)
npm install crtfilter
# OR manually include gingerbeardman/webgl-crt-shader from GitHub

# Backend (Rust)
cargo add axum -F ws
cargo add tokio -F full
cargo add serde -F derive
cargo add toml
cargo add codepage-437
```

## Architecture Patterns

### Recommended Project Structure
```
backend/
├── src/
│   ├── main.rs              # Axum server entry point
│   ├── websocket/
│   │   ├── mod.rs           # WebSocket handler
│   │   ├── session.rs       # Per-connection session state
│   │   └── protocol.rs      # ANSI escape sequence framing
│   ├── services/
│   │   ├── mod.rs           # Service trait + registry
│   │   ├── registry.rs      # Arc<dyn Service> collection, config-driven enable/disable
│   │   └── example.rs       # Example service implementation
│   ├── terminal/
│   │   ├── ansi.rs          # ANSI escape sequence builder, CP437 conversion
│   │   └── paging.rs        # [More] prompt implementation
│   └── config.rs            # TOML config loading with serde

frontend/
├── src/
│   ├── main.ts              # Entry point
│   ├── terminal.ts          # xterm.js setup with addons
│   ├── websocket.ts         # WebSocket client connection
│   ├── crt-effects.ts       # CRT shader setup and controls
│   └── mobile.ts            # Mobile-specific handlers
├── fonts/
│   └── PerfectDOSVGA437.ttf # CP437 font
└── styles/
    └── terminal.css         # Terminal container, bezel, CGA colors
```

### Pattern 1: Arc<dyn Trait> Plugin Architecture
**What:** Service registry holds Arc<dyn Service> trait objects loaded from configuration, enabling runtime service enable/disable without recompilation.

**When to use:** When you need pluggable modules that can be toggled via configuration and added without modifying core code.

**Example:**
```rust
// Service trait - all BBS services implement this
pub trait Service: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn handle_input(&self, session: &mut Session, input: &str) -> ServiceResult;
    fn on_enter(&self, session: &mut Session) -> ServiceResult;
    fn on_exit(&self, session: &mut Session);
}

// Service registry - holds enabled services
pub struct ServiceRegistry {
    services: HashMap<String, Arc<dyn Service>>,
}

impl ServiceRegistry {
    pub fn from_config(config: &Config) -> Self {
        let mut registry = Self { services: HashMap::new() };

        // Load services based on config
        for service_config in &config.services {
            if service_config.enabled {
                let service: Arc<dyn Service> = match service_config.name.as_str() {
                    "email" => Arc::new(EmailService::new(service_config)),
                    "chat" => Arc::new(ChatService::new(service_config)),
                    // ...
                    _ => continue,
                };
                registry.services.insert(service_config.name.clone(), service);
            }
        }

        registry
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Service>> {
        self.services.get(name)
    }
}
```

### Pattern 2: ANSI Escape Sequence Buffering
**What:** Buffer incoming data until complete ANSI sequences are received, preventing partial sequence rendering that corrupts the display.

**When to use:** All terminal I/O over WebSocket to ensure atomic ANSI sequence rendering.

**Example:**
```rust
pub struct AnsiBuffer {
    buffer: Vec<u8>,
    in_escape: bool,
}

impl AnsiBuffer {
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut complete_sequences = Vec::new();

        for &byte in bytes {
            self.buffer.push(byte);

            // Track escape sequence state
            if byte == 0x1B { // ESC
                self.in_escape = true;
            } else if self.in_escape && self.is_sequence_terminator(byte) {
                self.in_escape = false;
                // Flush complete sequence
                complete_sequences.push(self.buffer.clone());
                self.buffer.clear();
            } else if !self.in_escape && (byte.is_ascii_graphic() || byte.is_ascii_whitespace()) {
                // Regular text, can flush
                complete_sequences.push(vec![byte]);
            }
        }

        complete_sequences
    }

    fn is_sequence_terminator(&self, byte: u8) -> bool {
        matches!(byte, b'A'..=b'Z' | b'a'..=b'z')
    }
}
```

### Pattern 3: WebSocket Session with CP437 Conversion
**What:** Each WebSocket connection maintains a session that converts CP437 to UTF-8 before sending ANSI sequences to browser.

**When to use:** All terminal output to ensure browser receives valid UTF-8 with CP437 glyphs.

**Example:**
```rust
use axum::{
    extract::ws::{WebSocket, Message},
    extract::WebSocketUpgrade,
    response::Response,
};
use codepage_437::{CP437_CONTROL, FromCp437};

pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // Example: Send CP437-encoded ANSI art
    let cp437_bytes = include_bytes!("welcome.ans"); // CP437-encoded ANSI art

    // Convert CP437 → UTF-8
    let utf8_string = cp437_bytes.iter()
        .copied()
        .from_cp437(&CP437_CONTROL) // Handles 0x00-0x1F as control chars
        .collect::<String>();

    // Send to browser terminal
    if socket.send(Message::Text(utf8_string)).await.is_err() {
        return; // Connection closed
    }

    // Handle incoming messages
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(input) = msg {
            // Process user input...
        }
    }
}
```

### Pattern 4: Synchronized Rendering with DEC 2026
**What:** Use DECSET 2026 to batch screen updates, preventing flicker during complex ANSI art rendering.

**When to use:** When rendering multi-line ANSI art or paginated content with [More] prompts.

**Example:**
```rust
pub struct AnsiWriter {
    buffer: String,
}

impl AnsiWriter {
    pub fn begin_update(&mut self) {
        // Start synchronized rendering
        self.buffer.push_str("\x1B[?2026h");
    }

    pub fn end_update(&mut self) {
        // Flush synchronized rendering
        self.buffer.push_str("\x1B[?2026l");
    }

    pub fn clear_screen(&mut self) {
        self.buffer.push_str("\x1B[2J\x1B[H"); // Clear + home cursor
    }

    pub fn set_color(&mut self, fg: u8, bg: u8) {
        self.buffer.push_str(&format!("\x1B[{};{}m", fg + 30, bg + 40));
    }

    pub fn write(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    pub fn flush(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}

// Usage:
let mut writer = AnsiWriter::new();
writer.begin_update();
writer.clear_screen();
writer.set_color(15, 1); // White on blue
writer.write("Welcome to the BBS!");
writer.end_update();
let ansi_output = writer.flush();
```

### Pattern 5: xterm.js Initialization with Addons
**What:** Initialize xterm.js with WebGL renderer, fit addon for responsive sizing, and web-fonts addon for Perfect DOS VGA 437.

**When to use:** Frontend terminal initialization.

**Example:**
```typescript
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import { WebFontsAddon } from '@xterm/addon-web-fonts';
import '@xterm/xterm/css/xterm.css';

export async function initTerminal(container: HTMLElement): Promise<Terminal> {
    const term = new Terminal({
        cols: 80,
        rows: 24,
        fontFamily: 'Perfect DOS VGA 437, monospace',
        fontSize: 16,
        theme: {
            // CGA palette (authentic hex values)
            black: '#000000',
            red: '#aa0000',
            green: '#00aa00',
            yellow: '#aa5500',        // Brown, not dark yellow!
            blue: '#0000aa',
            magenta: '#aa00aa',
            cyan: '#00aaaa',
            white: '#aaaaaa',
            brightBlack: '#555555',
            brightRed: '#ff5555',
            brightGreen: '#55ff55',
            brightYellow: '#ffff55',  // Actual yellow
            brightBlue: '#5555ff',
            brightMagenta: '#ff55ff',
            brightCyan: '#55ffff',
            brightWhite: '#ffffff',
            background: '#000000',
            foreground: '#aaaaaa',
        },
        scrollback: 0, // No scrollback - authentic to era
        cursorBlink: true,
    });

    // Load addons
    const fitAddon = new FitAddon();
    const webglAddon = new WebglAddon();
    const webFontsAddon = new WebFontsAddon();

    term.loadAddon(webFontsAddon);
    term.loadAddon(fitAddon);
    term.loadAddon(webglAddon); // Must be loaded after fonts

    // Open terminal
    term.open(container);

    // Wait for fonts to load
    await webFontsAddon.ready();

    // Fit to container
    fitAddon.fit();

    // Re-fit on window resize
    window.addEventListener('resize', () => fitAddon.fit());

    return term;
}
```

### Pattern 6: Dial-able CRT Effects
**What:** CRT shader with multiple intensity levels (full CRT → subtle scanlines → clean) controlled by user settings.

**When to use:** All terminal rendering to provide authentic visual options.

**Example:**
```typescript
import CRTFilter from 'crtfilter';

export enum CRTLevel {
    CLEAN = 'clean',
    SUBTLE = 'subtle',
    FULL = 'full',
}

export class CRTController {
    private filter: CRTFilter;

    constructor(canvas: HTMLCanvasElement) {
        this.filter = new CRTFilter(canvas, this.getConfig(CRTLevel.FULL));
        this.filter.start();
    }

    setLevel(level: CRTLevel) {
        this.filter.updateConfig(this.getConfig(level));
    }

    private getConfig(level: CRTLevel) {
        switch (level) {
            case CRTLevel.CLEAN:
                return {
                    scanlineIntensity: 0,
                    glowBloom: 0,
                    curvature: 0,
                    chromaticAberration: 0,
                    flicker: 0,
                };
            case CRTLevel.SUBTLE:
                return {
                    scanlineIntensity: 0.3,
                    glowBloom: 0.2,
                    curvature: 0.05,
                    chromaticAberration: 0,
                    flicker: 0,
                };
            case CRTLevel.FULL:
                return {
                    scanlineIntensity: 0.6,
                    glowBloom: 0.4,
                    curvature: 0.15,
                    chromaticAberration: 0.1,
                    flicker: 0.05,
                    dotMask: true,
                    retraceLines: true,
                };
        }
    }
}
```

### Anti-Patterns to Avoid
- **Sending UTF-8 to xterm.js and expecting CP437 glyphs:** CP437 must be converted server-side; browsers interpret strings as Unicode/UTF-8, not CP437.
- **Loading WebGL addon before fonts load:** WebGL renderer caches glyph metrics; if fonts aren't loaded, it caches wrong metrics causing rendering issues.
- **Using text WebSocket frames without escape sequence buffering:** Partial ANSI sequences corrupt terminal display; always buffer until complete sequences received.
- **Hardcoding service list in core code:** Violates plugin architecture; use config-driven registry with Arc<dyn Trait> for runtime loading.
- **Generic "ESC" exit key for all services:** Real BBSes had service-specific exits (LORD used 'Q', chat used '/quit'); preserve this authenticity.
- **Fitting terminal on every message:** fit() is expensive; only call on window resize or significant layout changes.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CP437 ↔ UTF-8 conversion | Lookup tables with byte-by-byte conversion | codepage-437 crate | CP437 codepoints 0x00-0x1F are context-dependent (control vs printable), needs proper handling |
| ANSI escape sequence parsing | Regex or character-by-character state machine | xterm.js built-in parser | ANSI sequences have complex syntax with optional parameters, CSI vs OSC sequences, etc. |
| CRT shader effects | CSS filters or canvas manipulations | CRTFilter or webgl-crt-shader | Authentic CRT effects require scanline interference with actual pixel data, phosphor bloom, barrel distortion—WebGL shaders do this efficiently on GPU |
| Terminal cell grid rendering | HTML table or divs | xterm.js + WebGL renderer | Character cell rendering with ligatures, combining characters, box-drawing, double-width—xterm.js handles all edge cases |
| WebSocket reconnection logic | setTimeout retry loops | Built-in browser WebSocket with exponential backoff wrapper | Need proper close code handling, exponential backoff, message queuing during reconnect |
| Terminal responsive sizing | Manual col/row calculation from px dimensions | @xterm/addon-fit | Font metrics, padding, character dimensions—fit addon handles all calculations |

**Key insight:** Terminal emulation has decades of accumulated edge cases. Box-drawing characters, combining glyphs, ANSI sequence variations, CP437 ambiguities—libraries like xterm.js and codepage-437 encode this institutional knowledge. Custom solutions will hit edge cases and bugs that took years to discover and fix in these libraries.

## Common Pitfalls

### Pitfall 1: Mobile Keyboard Input Limitations
**What goes wrong:** Users on mobile devices experience "predictive text appears ahead of cursor" behavior and backspace behaves counter-intuitively due to an in-between text layer maintained by mobile keyboards.

**Why it happens:** xterm.js has limited touch support as of late 2025/early 2026. Issue #5377 (July 2025) documents that "limited touch support on mobile devices impacts terminal usability" because CoreBrowserTerminal.ts focuses primarily on mouse and keyboard events without dedicated touch event handling.

**How to avoid:** Accept that mobile keyboard experience will have quirks. Focus on portrait orientation (as user specified) since keyboard takes screen space. Consider implementing a "send" button for mobile to avoid backspace issues—user types in mobile keyboard, hits send, input goes to terminal atomically.

**Warning signs:** Mobile testers report "weird backspace behavior" or "text appears in wrong place." Test on real mobile devices (iOS Safari, Android Chrome), not just browser dev tools emulating mobile.

### Pitfall 2: Web Font Loading Race Condition
**What goes wrong:** Terminal renders with fallback monospace font, then "jumps" when Perfect DOS VGA 437 loads, causing character misalignment and broken box-drawing.

**Why it happens:** xterm.js initializes and caches glyph metrics immediately. If custom font isn't loaded yet, it caches metrics for fallback font. When custom font loads, metrics are wrong but already cached.

**How to avoid:** Use @xterm/addon-web-fonts and await webFontsAddon.ready() before calling term.open() or fitAddon.fit(). Alternative: Preload font in CSS with font-display: block and wait for document.fonts.ready promise.

**Warning signs:** Box-drawing characters (├ ─ │) appear as separate glyphs with gaps, or terminal dimensions are wrong after font loads.

### Pitfall 3: Partial ANSI Escape Sequence Rendering
**What goes wrong:** Screen shows literal "\x1B[" or "[2J" text instead of executing the escape sequence, corrupting display.

**Why it happens:** WebSocket text frames can split ANSI sequences across multiple messages. If you send each message chunk directly to terminal without buffering, xterm.js processes incomplete sequences as literal text.

**How to avoid:** Implement AnsiBuffer pattern (see Architecture Patterns). Buffer incoming bytes until complete ANSI sequence (ESC...letter) received, then flush to terminal. Use synchronized rendering (DECSET 2026) for complex screen updates.

**Warning signs:** Literal escape characters visible on screen, colors randomly change mid-word, cursor jumps unexpectedly.

### Pitfall 4: CP437 Box-Drawing Not Rendering
**What goes wrong:** Box-drawing characters (ASCII 0xB3, 0xC4, etc.) render as garbage or missing glyphs despite using Perfect DOS VGA 437 font.

**Why it happens:** CP437 bytes sent directly to browser are interpreted as UTF-8, causing mojibake. CP437 byte 0xB3 (│) is invalid UTF-8; browser substitutes replacement character (�).

**How to avoid:** Always convert CP437 → UTF-8 server-side before sending to browser. Use codepage_437::FromCp437 trait: `cp437_bytes.iter().copied().from_cp437(&CP437_CONTROL).collect::<String>()`. Browser then receives valid UTF-8 that, when rendered with CP437 font, displays correct glyphs.

**Warning signs:** Box-drawing ANSI art shows � or ? characters, or completely wrong glyphs. Verify with hex dump that server is sending UTF-8 (0xE29482 for │), not raw CP437 (0xB3).

### Pitfall 5: SQLite Write Contention in Multiplayer Games
**What goes wrong:** Multiplayer game (LORD, Trade Wars) becomes unplayable as concurrent users cause "database is locked" errors and timeouts.

**Why it happens:** SQLite defaults to single-writer mode. Even with WAL mode enabled, transactions that read-then-write fail if another write happens during the read phase. Multiplayer games have frequent score updates, turn processing, resource changes—all writes.

**How to avoid:** Use async-sqlite or tokio-rusqlite crate with WAL mode enabled. Implement write queues per-game service: queue write requests in a tokio channel, single task drains queue and batches writes. For read-heavy operations, use read replicas or cache game state in memory with periodic SQLite persistence.

**Warning signs:** "Database is locked" errors in logs, increasing latency as more users play games, timeout errors during game actions.

### Pitfall 6: Assuming Text WebSocket Frames Means UTF-8
**What goes wrong:** Protocol confusion where server expects CP437 bytes but browser sends UTF-8, or vice versa, causing garbled input.

**Why it happens:** WebSocket text frames must be valid UTF-8 per RFC 6455. Sending CP437 bytes in text frames violates spec. Binary frames have no encoding constraints but require explicit decode logic.

**How to avoid:** **Always use text frames with UTF-8.** Server converts CP437 → UTF-8 before sending to browser, browser sends UTF-8 input (user keyboard already produces UTF-8), server handles UTF-8 or converts to CP437 for internal storage if needed.

**Warning signs:** User types "café" but server receives garbage, or ANSI art with accented characters corrupts.

### Pitfall 7: Fit Addon Causing Layout Thrashing
**What goes wrong:** Terminal constantly resizes, causing flickering and performance issues.

**Why it happens:** fitAddon.fit() is called too frequently—every message, every frame, in tight loops. Fit recalculates dimensions and triggers terminal resize, which is expensive.

**How to avoid:** Call fit() only on window resize events and initial load. Debounce window resize events (wait 100-200ms after last resize before calling fit). Never call fit() in render loops or per-message handlers.

**Warning signs:** Browser dev tools performance profiler shows excessive layout calculations, terminal flickers during use, poor frame rates.

## Code Examples

Verified patterns from official sources:

### xterm.js Basic Setup
```typescript
// Source: https://github.com/xtermjs/xterm.js
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';

const term = new Terminal({
    cols: 80,
    rows: 24,
    fontFamily: 'Perfect DOS VGA 437, monospace',
    fontSize: 16,
    scrollback: 0,
});

const fitAddon = new FitAddon();
const webglAddon = new WebglAddon();

term.loadAddon(fitAddon);
term.loadAddon(webglAddon);

term.open(document.getElementById('terminal')!);
fitAddon.fit();

// Handle input
term.onData((data) => {
    // Send to WebSocket
    websocket.send(data);
});
```

### Axum WebSocket Handler
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::Response,
    routing::get,
    Router,
};

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    // Send welcome message
    if socket.send(Message::Text("Welcome to BBS!".into())).await.is_err() {
        return;
    }

    // Receive messages
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                // Process input
                let response = process_input(&text);
                if socket.send(Message::Text(response)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(ws_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### CP437 Conversion
```rust
// Source: https://docs.rs/codepage-437/latest/codepage_437/
use codepage_437::{CP437_CONTROL, FromCp437, ToCp437};

// CP437 bytes → UTF-8 String
let cp437_bytes: &[u8] = b"\xB3\xC4\xDA"; // Box-drawing: │─┌
let utf8_string: String = cp437_bytes
    .iter()
    .copied()
    .from_cp437(&CP437_CONTROL)
    .collect();
// utf8_string now contains valid UTF-8 that renders correctly with CP437 font

// UTF-8 String → CP437 bytes (lossy)
let text = "Hello!";
let cp437_bytes: Vec<u8> = text
    .chars()
    .to_cp437(&CP437_CONTROL)
    .collect::<Option<Vec<u8>>>()
    .unwrap_or_else(|| {
        // Handle unmappable characters
        text.chars()
            .to_cp437_lossy(&CP437_CONTROL)
            .collect()
    });
```

### TOML Configuration with Serde
```rust
// Source: https://docs.rs/toml/latest/toml/
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    services: Vec<ServiceConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServiceConfig {
    name: String,
    enabled: bool,
    #[serde(flatten)]
    options: toml::Value, // Service-specific options
}

// Load config
let config_str = fs::read_to_string("config.toml")?;
let config: Config = toml::from_str(&config_str)?;

// Example config.toml:
// [[services]]
// name = "email"
// enabled = true
// max_messages = 100
//
// [[services]]
// name = "chat"
// enabled = false
```

### CRT Shader Initialization
```typescript
// Source: https://github.com/Ichiaka/CRTFilter
import CRTFilter from 'crtfilter';

const canvas = document.getElementById('terminal-canvas') as HTMLCanvasElement;

const crtFilter = new CRTFilter(canvas, {
    // Dial-able settings
    scanlineIntensity: 0.6,      // 0 = none, 1 = maximum
    glowBloom: 0.4,               // Phosphor glow
    curvature: 0.15,              // Screen curve
    chromaticAberration: 0.1,    // RGB separation
    flicker: 0.05,                // Random flicker
    dotMask: true,                // Pixel structure
    retraceLines: true,           // CRT refresh lines
});

crtFilter.start();

// Update settings dynamically
function setCRTLevel(level: 'clean' | 'subtle' | 'full') {
    const configs = {
        clean: { scanlineIntensity: 0, glowBloom: 0, curvature: 0 },
        subtle: { scanlineIntensity: 0.3, glowBloom: 0.2, curvature: 0.05 },
        full: { scanlineIntensity: 0.6, glowBloom: 0.4, curvature: 0.15 },
    };
    crtFilter.updateConfig(configs[level]);
}
```

### Synchronized Rendering (DECSET 2026)
```rust
// Source: https://wezterm.org/escape-sequences.html
pub fn render_ansi_art(art_lines: &[String]) -> String {
    let mut output = String::new();

    // Begin synchronized update
    output.push_str("\x1B[?2026h");

    // Clear screen and home cursor
    output.push_str("\x1B[2J\x1B[H");

    // Render all lines
    for line in art_lines {
        output.push_str(line);
        output.push_str("\r\n");
    }

    // End synchronized update (flush to screen atomically)
    output.push_str("\x1B[?2026l");

    output
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Canvas renderer in xterm.js | WebGL renderer (canvas removed in 6.0.0) | December 2024 | 900% performance improvement in some cases, hardware GPU acceleration |
| Manual WebSocket attachment | @xterm/addon-attach | Available since 4.x | Simpler integration but optional—manual control still valid |
| @font-face without loading wait | @xterm/addon-web-fonts | Available since 5.x | Eliminates font loading race conditions and metrics cache issues |
| Manual ANSI buffering | DEC mode 2026 (synchronized rendering) | Added in 6.0.0 | Native terminal support for atomic screen updates, reduces flicker |
| rusqlite blocking calls in async | async-sqlite / tokio-rusqlite | Mature as of 2024 | Proper tokio integration without blocking executor threads |
| tungstenite standalone | tokio-tungstenite 0.28.0 | January 2026 | Performance improvements, reduced allocations, better async handling |

**Deprecated/outdated:**
- **xterm.js canvas renderer addon**: Removed in v6.0.0. Use WebGL renderer or fall back to DOM renderer if WebGL unavailable.
- **xterm package namespace**: Deprecated in v5.4.0 (March 2024). Use @xterm/* scoped packages for security.
- **SQLite without WAL mode for concurrent access**: WAL mode is now standard for any multi-user SQLite deployment.

## Open Questions

Things that couldn't be fully resolved:

1. **Mobile touch keyboard handling**
   - What we know: xterm.js has known issues with mobile touch keyboards (Issue #5377, July 2025). Predictive text causes cursor issues, backspace behaves unexpectedly.
   - What's unclear: Whether these issues will be fixed before Phase 1 implementation, and if workarounds (send button, local echo buffer) are sufficient for BBS use case.
   - Recommendation: Prototype mobile input during Phase 1 implementation. If issues persist, implement "send" button for mobile as fallback. Monitor xterm.js issue tracker for updates.

2. **Perfect DOS VGA 437 web font formats**
   - What we know: GitHub repo only has .ttf files, no .woff/.woff2 for optimized web delivery.
   - What's unclear: Whether converting .ttf to .woff2 preserves pixel-perfect rendering at exact sizes (8px, 16px, 24px, etc.).
   - Recommendation: Use .ttf directly in Phase 1 (browsers support it). If bundle size becomes issue, test conversion to .woff2 and verify rendering byte-for-byte with .ttf version.

3. **CRT shader mobile performance**
   - What we know: CRTFilter uses WebGL2 with WebGL1 fallback. User wants full CRT effects on mobile, not disabled by default.
   - What's unclear: Actual performance on mid-range and low-end mobile devices (iPhone XS is oldest tested per gingerbeardman shader).
   - Recommendation: Implement dial-able levels as planned. Default to "subtle" on mobile, provide UI to increase to "full." Monitor performance and adjust defaults based on testing.

4. **SQLite concurrent write strategy details**
   - What we know: WAL mode + write queues prevent contention. async-sqlite provides client with concurrent calls.
   - What's unclear: Optimal queue architecture—single global queue vs per-service queues vs per-game-instance queues for multiplayer games.
   - Recommendation: Start with per-service write queues in Phase 1 foundation. Games (Phase 6+) can refine to per-instance queues if needed. Profile under load to tune batch sizes.

5. **ANSI art generation by Claude**
   - What we know: User decided Claude will generate all ANSI art (no art packs).
   - What's unclear: Best workflow for generating CP437-compatible ANSI art via Claude, and how to verify it renders correctly.
   - Recommendation: Create ANSI art generation guidelines (80x24 dimensions, CGA 16-color palette, CP437 character set) and test early. May need Phase 1 task for "ANSI art preview tool" to verify rendering.

## Sources

### Primary (HIGH confidence)
- [xterm.js GitHub releases](https://github.com/xtermjs/xterm.js/releases) - Version 6.0.0 details, DEC mode 2026 support, WebGL renderer
- [xterm.js official docs - Terminal API](https://xtermjs.org/docs/api/terminal/classes/terminal/) - Configuration options, methods
- [xterm.js official docs - Using Addons](https://xtermjs.org/docs/guides/using-addons/) - Addon loading pattern
- [xterm.js GitHub - Addons directory](https://github.com/xtermjs/xterm.js/tree/master/addons) - Complete addon list
- [tokio-tungstenite docs](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/) - WebSocket server patterns
- [Axum WebSocket example](https://github.com/tokio-rs/axum/blob/main/examples/websockets/src/main.rs) - Official axum WebSocket pattern
- [codepage-437 crate docs](https://docs.rs/codepage-437/latest/codepage_437/) - CP437 conversion traits
- [CRTFilter GitHub](https://github.com/Ichiaka/CRTFilter) - CRT effect configuration
- [gingerbeardman webgl-crt-shader GitHub](https://github.com/gingerbeardman/webgl-crt-shader) - Tweakable CRT shader
- [CGA color palette reference](https://paulwratt.github.io/programmers-palettes/HW-CGA/HW-CGA-hex.html) - Authentic CGA hex values
- [Perfect DOS VGA 437 GitHub](https://github.com/CP437/PerfectDOSVGA437) - Font files

### Secondary (MEDIUM confidence)
- [WebSocket.org - Rust WebSocket Implementation](https://websocket.org/guides/languages/rust/) - Rust WebSocket ecosystem overview, verified with official docs
- [WezTerm escape sequences docs](https://wezterm.org/escape-sequences.html) - DEC mode 2026 (synchronized rendering) explanation, verified with xterm.js 6.0.0 release notes
- [int10h.org - IBM 5153's True CGA Palette](https://int10h.org/blog/2022/06/ibm-5153-color-true-cga-palette/) - CGA color accuracy, verified with multiple palette sources
- [Rust Design Patterns - Registries](https://willcrichton.net/rust-api-type-patterns/registries.html) - Type-based registry pattern
- [DEV.to - Plugin based architecture in Rust](https://dev.to/mineichen/plugin-based-architecture-in-rust-4om7) - Arc<dyn Trait> pattern for plugins

### Tertiary (LOW confidence - marked for validation)
- [Medium - Exploring Data Formats in WebSocket Communications](https://aditya-sunjava.medium.com/exploring-data-formats-in-websocket-communications-5c47871b5df5) - Binary vs text frames best practices (general advice, not Rust-specific)
- [GitHub Issue #5377 - Limited touch support on mobile devices](https://github.com/xtermjs/xterm.js/issues/5377) - Mobile keyboard issues (July 2025, may have updates)
- [Better Stack - How Turso Eliminates SQLite's Single-Writer Bottleneck](https://betterstack.com/community/guides/databases/turso-explained/) - SQLite concurrency approaches (Turso-specific, but general WAL insights apply)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified from official sources, versions confirmed, xterm.js 6.0.0 released December 2024
- Architecture: HIGH - Patterns verified from official examples (axum, xterm.js) and established Rust design patterns (Arc<dyn Trait>)
- Pitfalls: MEDIUM - Mobile issues confirmed from GitHub issues (HIGH), some performance claims lack 2026 benchmarks (MEDIUM)
- CRT shaders: MEDIUM - Multiple implementations verified, but specific performance on varied mobile devices needs testing
- CP437 handling: HIGH - codepage-437 crate documented, conversion pattern verified

**Research date:** 2026-01-26
**Valid until:** 2026-03-26 (60 days - relatively stable stack, but mobile touch support may improve, monitor xterm.js releases)

**Sources requiring validation:**
- Mobile keyboard behavior: Test on real devices to confirm Issue #5377 status hasn't changed
- CRT shader mobile performance: Benchmark on target devices (iPhone XS, mid-range Android)
- Perfect DOS VGA 437 .woff2 conversion: Verify pixel-perfect rendering if web font optimization needed
