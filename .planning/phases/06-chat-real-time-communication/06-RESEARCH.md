# Phase 6: Chat & Real-Time Communication - Research

**Researched:** 2026-01-28
**Domain:** Real-time WebSocket chat with broadcast messaging
**Confidence:** HIGH

## Summary

Phase 6 implements a single-room teleconference chat system where users can see messages from all participants in real-time, with join/leave announcements, action commands (/me), and user paging capabilities. The research focused on understanding Tokio's broadcast channel patterns, WebSocket state management, and integration with the existing BBS architecture.

The standard approach for multi-user chat in Rust/Tokio is to use `tokio::sync::broadcast` channels for message distribution, with each WebSocket connection subscribing to receive broadcasts. The existing codebase already has the foundational infrastructure: Tokio async runtime, WebSocket handling via tokio-tungstenite/axum, NodeManager for presence tracking, and xterm.js terminal on the frontend.

**Primary recommendation:** Implement a ChatRoom manager using tokio::sync::broadcast for message distribution, integrate with existing NodeManager for presence, and extend the session state machine with a Chat state that handles command parsing and broadcasts.

## Standard Stack

### Core (Already in Project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.x | Async runtime with broadcast channels | Industry standard for async Rust, built-in broadcast primitive |
| axum | 0.7 | WebSocket extraction and routing | Already used for WebSocket connections, integrates with tokio |
| tokio-tungstenite | (via axum) | WebSocket protocol implementation | De facto standard for async WebSocket in Rust |
| serde_json | 1.x | JSON serialization for control messages | Already used for modem/timer JSON signals |

### Supporting (Already Available)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4 | Timestamp formatting for messages | Already used, for [HH:MM] prefixes |
| AnsiWriter | (existing) | Terminal color and formatting | Already used for all terminal output |
| xterm.js | (frontend) | Terminal emulation with bell support | Already integrated, supports BEL character |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tokio::sync::broadcast | tokio::sync::mpsc with manual fan-out | Broadcast is purpose-built for 1-to-many, automatic cleanup |
| Single global channel | Per-room channels with HashMap | Single room = simpler, future multi-room can add HashMap |
| State machine in Session | Separate Service trait implementation | State machine is more direct, Service is more modular |

**Installation:**
Already have all required dependencies in Cargo.toml. No new crates needed.

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── connection/
│   ├── node_manager.rs       # Already exists - tracks online users
│   └── chat_manager.rs       # NEW - manages broadcast channel & chat state
├── services/
│   └── chat.rs               # NEW - chat command parsing & rendering
└── websocket/
    └── session.rs            # EXTEND - add Chat state to AuthState enum
```

### Pattern 1: Broadcast Channel for Message Distribution
**What:** Use `tokio::sync::broadcast::channel()` to create a multi-producer, multi-consumer channel where all subscribers receive every message.

**When to use:** When multiple recipients need to see identical messages without manual iteration.

**Example:**
```rust
// Source: https://tokio.rs/tokio/tutorial/channels (Tokio official docs)
use tokio::sync::broadcast;

// Create broadcast channel with capacity 100
let (tx, _rx) = broadcast::channel(100);

// Each new subscriber gets their own receiver
let mut rx1 = tx.subscribe();
let mut rx2 = tx.subscribe();

// Send to all subscribers
tx.send("Hello everyone".to_string()).unwrap();

// Both receivers see the message
assert_eq!(rx1.recv().await.unwrap(), "Hello everyone");
assert_eq!(rx2.recv().await.unwrap(), "Hello everyone");
```

**Integration with existing code:**
- Store `broadcast::Sender` in a `ChatManager` struct
- On user join: call `tx.subscribe()` to get their receiver
- Use `tokio::select!` to handle both WebSocket input and broadcast messages

### Pattern 2: ChatManager as Shared State
**What:** Create a `ChatManager` struct with `Arc<ChatManager>` shared across all sessions, managing the broadcast channel and tracking who's in chat.

**When to use:** When multiple sessions need coordinated access to chat state.

**Example:**
```rust
// Adapted from https://github.com/tokio-rs/tokio/blob/master/examples/chat.rs
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use std::collections::HashMap;

pub struct ChatManager {
    // Broadcast channel for all chat messages
    tx: broadcast::Sender<ChatMessage>,
    // Track who's currently in chat (separate from NodeManager online users)
    participants: Arc<RwLock<HashMap<i64, String>>>, // user_id -> handle
    // Optional: configurable capacity limit
    capacity: usize,
}

impl ChatManager {
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            tx,
            participants: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.tx.subscribe()
    }

    pub async fn broadcast(&self, msg: ChatMessage) {
        // send() returns Err if no receivers, which is fine
        let _ = self.tx.send(msg);
    }

    pub async fn join(&self, user_id: i64, handle: String) -> Result<(), String> {
        let mut participants = self.participants.write().await;
        if participants.len() >= self.capacity {
            return Err("Chat room is full".to_string());
        }
        participants.insert(user_id, handle);
        Ok(())
    }

    pub async fn leave(&self, user_id: i64) {
        let mut participants = self.participants.write().await;
        participants.remove(&user_id);
    }

    pub async fn get_participants(&self) -> Vec<String> {
        let participants = self.participants.read().await;
        participants.values().cloned().collect()
    }
}
```

**Integration with AppState:**
```rust
// In main.rs
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) registry: ServiceRegistry,
    pub(crate) db_pool: SqlitePool,
    pub(crate) node_manager: NodeManager,
    pub(crate) chat_manager: ChatManager,  // NEW
}
```

### Pattern 3: Chat State in Session State Machine
**What:** Add a `Chat` variant to the existing `AuthState` enum with its own receive loop using `tokio::select!`.

**When to use:** When chat is a distinct mode with its own input handling and message flow.

**Example:**
```rust
// In session.rs
enum AuthState {
    AwaitingAuth,
    ConnectionCeremony,
    Login(LoginFlow),
    // ... existing states
    Chat {
        rx: broadcast::Receiver<ChatMessage>,
        last_dm_sender: Option<String>,
    },
    MainMenu(MenuSession),
}

// In handle_input for Chat state:
async fn handle_chat_input(&mut self, input: &str) {
    if input.starts_with('/') {
        // Parse command
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0] {
            "/quit" | "/q" => { /* exit chat */ }
            "/me" => { /* broadcast action */ }
            "/who" => { /* show participants */ }
            "/msg" => { /* send DM */ }
            _ => { /* unknown command error */ }
        }
    } else {
        // Regular message - broadcast to all
        let msg = ChatMessage::Public {
            sender: self.user_handle.clone(),
            text: input.to_string(),
        };
        self.app_state.chat_manager.broadcast(msg).await;
    }
}

// Main chat loop with tokio::select!
loop {
    tokio::select! {
        // Receive broadcasts from other users
        result = rx.recv() => {
            match result {
                Ok(msg) => self.render_chat_message(msg),
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // Too slow, missed n messages
                    self.write(&format!("*** Missed {} messages ***", n));
                }
                Err(_) => break, // Channel closed
            }
        }
        // Handle WebSocket input (already being read elsewhere)
        // This is tricky - need to coordinate with main recv loop
    }
}
```

**Note on coordination:** The session already has a main `handle_input` loop in `ws_handler`. The chat state needs to interleave broadcast receives with user input. Two approaches:

1. **Spawn a separate task** to listen to broadcasts and forward to the session's mpsc::Sender
2. **Move the WebSocket recv loop** into the Chat state handler so it can use tokio::select!

Recommend approach #1 for minimal disruption to existing architecture.

### Pattern 4: Command Parsing with Pattern Matching
**What:** Use simple string matching on input prefix to dispatch commands.

**When to use:** For IRC-style slash commands with limited complexity.

**Example:**
```rust
fn parse_chat_command(input: &str) -> ChatCommand {
    if !input.starts_with('/') {
        return ChatCommand::Message(input.to_string());
    }

    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).copied().unwrap_or("");

    match cmd {
        "/quit" | "/q" => ChatCommand::Quit,
        "/help" | "/?" => ChatCommand::Help,
        "/who" => ChatCommand::Who,
        "/me" => ChatCommand::Action(args.to_string()),
        "/page" => {
            let target = args.split_whitespace().next().unwrap_or("");
            ChatCommand::Page(target.to_string())
        }
        "/msg" => {
            let mut parts = args.splitn(2, ' ');
            let target = parts.next().unwrap_or("");
            let text = parts.next().unwrap_or("");
            ChatCommand::DirectMessage {
                target: target.to_string(),
                text: text.to_string(),
            }
        }
        "/r" => ChatCommand::Reply(args.to_string()),
        _ => ChatCommand::Unknown(cmd.to_string()),
    }
}

enum ChatCommand {
    Message(String),
    Quit,
    Help,
    Who,
    Action(String),
    Page(String),
    DirectMessage { target: String, text: String },
    Reply(String),
    Unknown(String),
}
```

### Anti-Patterns to Avoid
- **Broadcast capacity too small:** Slow receivers can lag and miss messages. Use 100+ capacity for chat.
- **Forgetting to unsubscribe on exit:** Always clean up broadcast receiver when leaving chat.
- **Blocking operations in message handlers:** Keep rendering fast to avoid lagging behind broadcast.
- **Manual iteration over connections:** Use broadcast channel, don't iterate sessions manually.
- **Shared mutable state without locks:** Use RwLock for participant tracking, read locks for queries.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Message distribution to multiple recipients | Manual HashMap of tx channels with iteration | `tokio::sync::broadcast` | Broadcast handles lagged receivers, automatic cleanup, optimized memory |
| Command parsing for simple slash commands | Regex-based parser or full command framework | Simple string `starts_with` + `splitn` | Over-engineering for 6 commands; match on prefix is fast and clear |
| Bell sound for terminal | Custom Web Audio implementation | xterm.js `onBell` event + existing audio.ts | Already have audio loading infrastructure for modem sounds |
| Timestamp formatting | Manual time parsing and formatting | `chrono` (already in deps) for UTC, format with `%H:%M` | Already using chrono, handles timezones and formatting |
| Presence tracking | New tracking system | Extend existing `NodeManager` | Already tracks online users with handles, just filter for chat participants |

**Key insight:** The project already has 90% of the infrastructure needed. The new code is primarily: (1) ChatManager with broadcast channel, (2) command parsing, (3) Chat state in session, (4) rendering functions. Don't rebuild WebSocket handling, presence tracking, or terminal infrastructure.

## Common Pitfalls

### Pitfall 1: Broadcast Receiver Lagging
**What goes wrong:** If a receiver can't keep up with broadcast rate, it will lag behind and eventually miss messages with `RecvError::Lagged(n)`.

**Why it happens:** Broadcast channel has bounded capacity. If a slow subscriber doesn't recv() fast enough, new messages overwrite old ones.

**How to avoid:**
- Use adequate channel capacity (100+ for chat)
- Keep message rendering fast (no slow DB queries in render path)
- Handle `RecvError::Lagged` gracefully with "*** Missed N messages ***" notification
- Consider increasing capacity if users report missed messages

**Warning signs:**
- Users complaining about missing messages
- Lagged errors in logs
- Chat feeling sluggish or delayed

### Pitfall 2: Deadlock from Holding Locks Across .await
**What goes wrong:** If you acquire a `RwLock` or `Mutex` and then `.await` while holding the guard, you can deadlock the async runtime.

**Why it happens:** Tokio may suspend your task at the `.await` and schedule another task on the same thread. If that task tries to acquire the same lock, deadlock.

**How to avoid:**
- NEVER hold a lock guard across an `.await` point
- Acquire lock, clone data, drop guard, THEN await
- Use `tokio::sync::RwLock` (async-aware) if you must hold across await, but prefer avoiding it
- Pattern: `{ let data = state.read().await.clone(); } /* guard dropped */ do_async_thing(data).await;`

**Warning signs:**
- Tasks hanging indefinitely
- No error messages, just freezing
- Works with few users, deadlocks under load

### Pitfall 3: Not Cleaning Up on Disconnect
**What goes wrong:** User disconnects but remains in chat participant list, causing ghost users and eventual memory leak.

**Why it happens:** Forgot to call `chat_manager.leave()` in disconnect handler.

**How to avoid:**
- Call `chat_manager.leave(user_id)` in `Session::on_disconnect()`
- Use RAII pattern: store user_id in session, cleanup in Drop
- Broadcast leave announcement before cleanup
- Verify participant count doesn't grow unbounded

**Warning signs:**
- Participant list includes disconnected users
- `/who` shows ghosts
- Memory usage grows with churn

### Pitfall 4: Race Condition Between Join and First Message
**What goes wrong:** User joins chat, broadcast receiver is created, but first messages arrive before user sees join announcement or initial prompt.

**Why it happens:** Asynchronous ordering - other users' messages can broadcast while you're still rendering the join sequence.

**How to avoid:**
- Subscribe to broadcast channel BEFORE sending join announcement
- Render chat UI and prompt BEFORE broadcasting join
- Order: (1) subscribe, (2) render UI, (3) broadcast join
- Drain any queued messages after UI is ready

**Warning signs:**
- Users see messages before seeing "Welcome to chat"
- Join announcements appear out of order
- Confused UX on entry

### Pitfall 5: Direct Messages Visible to Everyone
**What goes wrong:** DMs accidentally broadcast to public channel instead of sent peer-to-peer.

**Why it happens:** All messages use same broadcast channel; need separate direct send path.

**How to avoid:**
- DMs must NOT use broadcast channel
- For 1:1 messaging, send directly to recipient's session mpsc::Sender
- Need a way to look up recipient's session tx by handle/user_id
- Option 1: Add `get_session_tx(user_id)` to AppState
- Option 2: DMs still go through broadcast but with `ChatMessage::Direct { to, from, text }` and receivers filter
- **Recommend Option 2** for simplicity: broadcast with metadata, receivers only display if they're sender or recipient

**Warning signs:**
- DMs appearing in everyone's chat
- Privacy complaints
- Test with two users, verify third doesn't see DM

## Code Examples

Verified patterns from official sources and existing codebase:

### Creating and Using Broadcast Channel
```rust
// Source: https://tokio.rs/tokio/tutorial/channels
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum ChatMessage {
    Public { sender: String, text: String },
    Action { sender: String, action: String },
    System { text: String },
    Direct { from: String, to: String, text: String },
    Join { handle: String },
    Leave { handle: String },
}

// Create channel with capacity 100
let (tx, _rx) = broadcast::channel::<ChatMessage>(100);

// Store tx in AppState/ChatManager
let chat_manager = ChatManager { tx, /* ... */ };

// Each user subscribes on joining chat
let mut rx = chat_manager.tx.subscribe();

// Send messages
let msg = ChatMessage::Public {
    sender: "Alice".to_string(),
    text: "Hello!".to_string(),
};
chat_manager.tx.send(msg).unwrap();

// Receive messages with error handling
match rx.recv().await {
    Ok(msg) => { /* render message */ }
    Err(broadcast::error::RecvError::Lagged(n)) => {
        eprintln!("Lagged by {} messages", n);
    }
    Err(broadcast::error::RecvError::Closed) => {
        eprintln!("Channel closed");
    }
}
```

### Integrating with Existing Session Architecture
```rust
// In session.rs - extend existing pattern
use tokio::sync::broadcast;

// Add to Session struct
pub struct Session {
    // ... existing fields
    chat_rx: Option<broadcast::Receiver<ChatMessage>>,
}

// In handle_input for AuthState::MainMenu
MenuAction::EnterChat => {
    // Subscribe to broadcasts
    let rx = self.app_state.chat_manager.subscribe();

    // Join chat room
    if let Err(e) = self.app_state.chat_manager.join(
        self.user_id,
        self.user_handle.clone()
    ).await {
        self.write(&format!("Cannot join chat: {}", e));
        return;
    }

    // Broadcast join announcement
    let msg = ChatMessage::Join {
        handle: self.user_handle.clone()
    };
    self.app_state.chat_manager.broadcast(msg).await;

    // Render chat UI
    self.render_chat_screen();

    // Spawn task to forward broadcasts to session output
    let tx = self.tx.clone();
    let handle = self.user_handle.clone();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let formatted = format_chat_message(&msg, &handle);
                    let _ = tx.send(formatted).await;
                }
                Err(_) => break,
            }
        }
    });

    // Transition to Chat state
    self.auth_state = AuthState::Chat { /* ... */ };
}
```

### Rendering Chat Messages with Color and Format
```rust
// Adapted from existing services/mail.rs and terminal/ansi.rs
use crate::terminal::{AnsiWriter, Color};
use chrono::Utc;

fn format_chat_message(msg: &ChatMessage, my_handle: &str) -> String {
    let mut w = AnsiWriter::new();

    match msg {
        ChatMessage::Public { sender, text } => {
            // [HH:MM] Handle: Message text
            let now = Utc::now();
            let timestamp = now.format("%H:%M");

            w.write_str(&format!("[{}] ", timestamp));
            w.set_fg(Color::Green); // Dark green
            w.write_str(sender);
            w.reset_color();
            w.write_str(": ");
            w.set_fg(Color::LightGreen);
            w.write_str(text);
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Action { sender, action } => {
            // * Handle waves
            w.set_fg(Color::LightGreen);
            w.write_str("* ");
            w.write_str(sender);
            w.write_str(" ");
            w.write_str(action);
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::System { text } => {
            // *** System message ***
            w.set_fg(Color::Yellow);
            w.write_str("*** ");
            w.write_str(text);
            w.write_str(" ***");
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Direct { from, to, text } => {
            // Only render if I'm sender or recipient
            if from != my_handle && to != my_handle {
                return String::new(); // Don't render
            }

            let now = Utc::now();
            let timestamp = now.format("%H:%M");
            let prefix = if from == my_handle { "-> " } else { "<- " };

            w.write_str(&format!("[{}] ", timestamp));
            w.set_fg(Color::LightCyan);
            w.write_str(prefix);
            w.write_str(if from == my_handle { to } else { from });
            w.reset_color();
            w.write_str(": ");
            w.set_fg(Color::White);
            w.write_str(text);
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Join { handle } => {
            w.set_fg(Color::Yellow);
            w.write_str(&format!("*** {} has joined the chat ***", handle));
            w.reset_color();
            w.writeln("");
        }
        ChatMessage::Leave { handle } => {
            w.set_fg(Color::Yellow);
            w.write_str(&format!("*** {} has left the chat ***", handle));
            w.reset_color();
            w.writeln("");
        }
    }

    w.flush()
}
```

### Parsing Chat Commands
```rust
// Simple pattern matching - no external crate needed
fn parse_chat_command(input: &str) -> ChatCommand {
    let trimmed = input.trim();

    // Empty input
    if trimmed.is_empty() {
        return ChatCommand::Empty;
    }

    // Not a command - regular message
    if !trimmed.starts_with('/') {
        return ChatCommand::Message(trimmed.to_string());
    }

    // Split command and arguments
    let mut parts = trimmed.splitn(2, ' ');
    let cmd = parts.next().unwrap(); // Safe: we know it starts with /
    let args = parts.next().unwrap_or("").trim();

    match cmd {
        "/quit" | "/q" => ChatCommand::Quit,
        "/help" | "/?" => ChatCommand::Help,
        "/who" => ChatCommand::Who,
        "/me" => {
            if args.is_empty() {
                ChatCommand::Error("Usage: /me <action>".to_string())
            } else {
                ChatCommand::Action(args.to_string())
            }
        }
        "/page" => {
            if args.is_empty() {
                ChatCommand::Error("Usage: /page <handle>".to_string())
            } else {
                ChatCommand::Page(args.to_string())
            }
        }
        "/msg" => {
            let mut msg_parts = args.splitn(2, ' ');
            let target = msg_parts.next().unwrap_or("");
            let text = msg_parts.next().unwrap_or("");

            if target.is_empty() || text.is_empty() {
                ChatCommand::Error("Usage: /msg <handle> <message>".to_string())
            } else {
                ChatCommand::DirectMessage {
                    target: target.to_string(),
                    text: text.to_string(),
                }
            }
        }
        "/r" => {
            if args.is_empty() {
                ChatCommand::Error("Usage: /r <message>".to_string())
            } else {
                ChatCommand::Reply(args.to_string())
            }
        }
        _ => ChatCommand::Unknown(cmd.to_string()),
    }
}

enum ChatCommand {
    Empty,
    Message(String),
    Quit,
    Help,
    Who,
    Action(String),
    Page(String),
    DirectMessage { target: String, text: String },
    Reply(String),
    Unknown(String),
    Error(String),
}
```

### Bell Sound for Paging (Frontend)
```typescript
// In frontend/src/websocket.ts - extend existing JSON message handling
// Add to onmessage handler after existing type checks

if (parsed.type === 'bell') {
  // Play bell sound for page notification
  playBellSound();
  return; // Don't write JSON to terminal
}

// In frontend/src/audio.ts - extend existing audio loading
let bellBuffer: AudioBuffer | null = null;

export async function loadBellSound(): Promise<void> {
  try {
    const resp = await fetch('/audio/bell.mp3');
    if (resp.ok) {
      const buf = await resp.arrayBuffer();
      bellBuffer = await audioContext.decodeAudioData(buf);
    }
  } catch (e) {
    console.warn('Failed to load bell sound:', e);
  }
}

export async function playBellSound(): Promise<void> {
  return playBuffer(bellBuffer);
}

// Backend sends: { "type": "bell" } when /page happens
// Or send BEL character (\x07) and use xterm.js onBell event:

// In frontend/src/main.ts
terminal.onBell(() => {
  playBellSound();
});

// Backend sends: "\x07" in message text
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual HashMap<addr, tx> iteration | `tokio::sync::broadcast` channel | Tokio 0.2+ (2020) | Automatic fan-out, lagged receiver handling, simpler code |
| axum 0.6 WebSocket API | axum 0.7 extract::ws | axum 0.7 (2023) | Cleaner API, better type safety |
| xterm.js built-in bell sound | xterm.js onBell event (5.0+) | xterm.js 5.0 (2022) | Embedder controls sound, smaller bundle size |
| std::sync::Mutex across await | `tokio::sync::RwLock` or avoid | Ongoing best practice | Prevents deadlocks in async code |

**Deprecated/outdated:**
- xterm.js `bellSound` and `bellStyle` options: Removed in v5.0, use `onBell` event instead
- Holding `std::sync` locks across `.await`: Can deadlock, use tokio::sync or drop before await

## Open Questions

1. **Chat capacity default value**
   - What we know: User wants configurable limit independent of max_nodes
   - What's unclear: Good default value for single-room chat
   - Recommendation: Start with `max_nodes * 2` as default, allow config override. Single room with 32 users (if max_nodes=16) is reasonable for BBS scale.

2. **DM visibility approach**
   - What we know: Need 1:1 messages visible only to sender and recipient
   - What's unclear: Broadcast with filtering vs. direct session send
   - Recommendation: Broadcast with metadata (easier to implement, consistent with architecture). Add `ChatMessage::Direct { from, to, text }` and filter in render function. Only visible if `my_handle == from || my_handle == to`.

3. **Auto-scroll with xterm.js scrollback=0**
   - What we know: Current terminal has `scrollback: 0` (no scroll buffer)
   - What's unclear: Does this force auto-scroll or prevent it?
   - Recommendation: Test behavior. If scrollback=0 prevents scroll, chat works as-is. If users can scroll up, they won't see new messages (no auto-scroll in xterm.js). May need to keep scrollback=0 for chat mode specifically.

4. **Bell sound file choice**
   - What we know: Need bell sound for pages, already have audio loading infrastructure
   - What's unclear: Use system bell, custom sound, or reuse existing modem sounds
   - Recommendation: Add `bell.mp3` to `frontend/public/audio/` similar to modem sounds. Short (0.5s), classic terminal bell or BBS door chime sound. Load in `audio.ts` alongside modem sounds.

## Sources

### Primary (HIGH confidence)
- [Tokio Channels Tutorial](https://tokio.rs/tokio/tutorial/channels) - Official documentation on mpsc, broadcast, oneshot, watch channels
- [Tokio Chat Example (GitHub)](https://github.com/tokio-rs/tokio/blob/master/examples/chat.rs) - Official chat server example using broadcast
- [Axum Documentation](https://docs.rs/axum/latest/axum/) - Version 0.7 WebSocket extraction patterns
- [Tokio Shared State Tutorial](https://tokio.rs/tokio/tutorial/shared-state) - Official guide on Arc, Mutex, RwLock patterns
- Existing codebase: backend/src/websocket/session.rs, connection/node_manager.rs, terminal/ansi.rs - Established patterns for WebSocket handling, presence tracking, and terminal rendering

### Secondary (MEDIUM confidence)
- [Google Comprehensive Rust: Broadcast Chat](https://google.github.io/comprehensive-rust/concurrency/async-exercises/chat-app.html) - Tutorial on broadcast chat with tokio::select!
- [Codez Up: Multi-User Chat with Rust & Tokio (2025)](https://codezup.com/rust-tokio-multi-user-chat-app/) - Recent practical implementation guide
- [Medium: Building WebSocket Chat Server in Rust (2024)](https://medium.com/@FAANG/building-a-real-time-websocket-chat-server-in-rust-8a751cdade43) - Recent production patterns
- [Medium: How to Avoid Memory Leaks in Tokio (2025)](https://medium.com/@adamszpilewicz/%EF%B8%8F-how-to-avoid-memory-leaks-in-tokio-0aeb9ae2387d) - Current best practices
- [xterm.js Issue #3014: Bell event support](https://github.com/xtermjs/xterm.js/issues/3014) - Discussion of onBell API

### Tertiary (LOW confidence)
- WebSearch results on WebSocket chat patterns - General implementation discussion, not authoritative
- Community forum discussions on deadlock prevention - Anecdotal but consistent with official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in project, broadcast channel is documented official pattern
- Architecture: HIGH - Patterns verified in official Tokio examples and existing codebase structure
- Pitfalls: MEDIUM - Common issues documented in multiple sources, but specific manifestation depends on implementation details

**Research date:** 2026-01-28
**Valid until:** ~90 days (stable ecosystem, Tokio 1.x is mature)

---

**Ready for planning:** Yes. All core patterns identified, existing infrastructure mapped, integration points clear. Planner has sufficient detail to create concrete task breakdown.
