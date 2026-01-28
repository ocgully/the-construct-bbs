---
phase: 06
plan: 01
subsystem: chat
tags: [tokio, broadcast, real-time, websocket]
requires: [05]
provides: [ChatManager, ChatMessage, ChatConfig]
affects: [06-02, 06-03, 06-04, 06-05]
tech-stack:
  added: []
  patterns: [broadcast channel for 1-to-many messaging, RwLock participant tracking]
key-files:
  created:
    - backend/src/connection/chat_manager.rs
  modified:
    - backend/src/connection/mod.rs
    - backend/src/config.rs
    - backend/src/main.rs
    - backend/src/services/login.rs
    - backend/src/services/registration.rs
decisions:
  - id: 06-01-01
    summary: "ChatMessage enum with 7 variants (Public, Action, System, Direct, Join, Leave, Page)"
    impact: "Covers all chat message types including emotes, private messages, and notifications"
  - id: 06-01-02
    summary: "Broadcast channel buffer size of 100 messages"
    impact: "Buffer for in-flight messages, separate from participant capacity"
  - id: 06-01-03
    summary: "Case-insensitive handle lookup for /msg commands"
    impact: "User-friendly private messaging regardless of handle case"
  - id: 06-01-04
    summary: "Default chat capacity of 32 users (2x max_nodes)"
    impact: "Reasonable default that can be configured per deployment"
metrics:
  duration: 7m
  completed: 2026-01-28
---

# Phase 6 Plan 1: ChatManager with Broadcast Channel Summary

ChatManager with tokio::sync::broadcast for 1-to-many real-time message distribution, RwLock participant tracking, and ChatConfig with capacity setting.

## What Was Built

### ChatMessage Enum
Seven message variants covering all chat communication types:
- **Public** - Regular chat messages with sender and text
- **Action** - /me emote actions
- **System** - System announcements (errors, capacity messages)
- **Direct** - Private messages between two users
- **Join** - User joined chat notification
- **Leave** - User left chat notification
- **Page** - Page notification to trigger bell on recipient

### ChatManager Struct
Core chat room management with:
- `broadcast::Sender<ChatMessage>` for 1-to-many distribution
- `Arc<RwLock<HashMap<i64, String>>>` for thread-safe participant tracking
- Configurable capacity enforcement

### Methods Implemented
| Method | Purpose |
|--------|---------|
| `new(capacity)` | Create manager with broadcast channel (buffer: 100) |
| `subscribe()` | Get Receiver for chat messages |
| `broadcast(msg)` | Send message to all subscribers (ignores no-receivers) |
| `join(user_id, handle)` | Add participant with capacity check |
| `leave(user_id)` | Remove participant |
| `get_participants()` | List all handles in chat |
| `is_in_chat(user_id)` | Check if user is in chat |
| `get_participant_count()` | Current participant count |
| `get_handle_user_id(handle)` | Case-insensitive reverse lookup for /msg |

### ChatConfig
Configuration struct with:
- `capacity: usize` - Maximum users in chat (default: 32)
- Serde defaults for optional config section

### AppState Integration
- `chat_manager: ChatManager` field in AppState
- Initialized with `config.chat.capacity`
- Startup message: "Chat capacity: N users"

## Test Coverage
8 unit tests added for ChatManager:
- Manager creation
- Join adds participant / respects capacity / is idempotent
- Leave removes participant
- Get participants returns handles
- Case-insensitive handle lookup
- Broadcast works with/without subscribers

## Commits

| Hash | Description |
|------|-------------|
| 126705e | feat(06-01): add ChatManager with broadcast channel |
| c1e6f89 | feat(06-01): add ChatConfig and wire ChatManager into AppState |

## Verification Results

1. `cargo check` - Passes with expected dead code warnings (foundation not yet used)
2. `cargo test` - All 200 tests pass (8 new ChatManager tests)
3. Server prints "Chat capacity: 32 users" on startup

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated test configs with chat field**
- **Found during:** Task 2 verification
- **Issue:** Test config structs in login.rs and registration.rs missing new chat field
- **Fix:** Added `chat: crate::config::ChatConfig::default()` to both test_config functions
- **Files modified:** backend/src/services/login.rs, backend/src/services/registration.rs
- **Commit:** c1e6f89

## Next Phase Readiness

Ready for 06-02 (ChatState and session integration):
- ChatManager available in AppState for session access
- subscribe() returns Receiver for session to monitor
- join/leave methods for session lifecycle
- broadcast() for sending messages
- All message types defined for rendering

## Technical Notes

- Broadcast channel pattern from tokio: `let _ = tx.send(msg)` ignores RecvError when no subscribers
- RwLock from tokio (not std) for async-safe concurrent access
- Clone derive on ChatManager enables AppState cloning for handler extraction
- Idempotent join (no error if already in chat) simplifies session lifecycle
