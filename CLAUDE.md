# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

The Construct BBS - A web-based Bulletin Board System built in Rust with a browser-based xterm.js terminal frontend. Inspired by Wildcat BBS, delivering authentic retro BBS experience with modem sounds, ANSI art, door games, and artificial scarcity (node limits, time limits).

## Commands

### Development
```powershell
# Full dev environment (from project root)
.\dev.ps1

# Manual startup
cd frontend && npm install && npm run build
cd backend && cargo run
```

### Testing
```powershell
cd backend && cargo test                    # All tests
cd backend && cargo test <test_name>        # Single test
cd backend && cargo test <module>::         # Module tests (e.g., cargo test game::)
```

### Build
```powershell
cd frontend && npm run build    # Frontend (outputs to frontend/dist/)
cd backend && cargo build       # Backend
```

## Architecture

### Backend (Rust/Axum)

**Session Flow**: WebSocket connection → `websocket/session.rs` manages the state machine that handles:
- Connection ceremony (modem sounds, ANSI splash)
- Authentication (login/registration)
- Menu navigation
- Service dispatch

**Service Trait Pattern**: All BBS features implement `Service` trait (`services/mod.rs`):
```rust
pub trait Service: Send + Sync {
    fn name(&self) -> &str;
    fn on_enter(&self, session: &mut dyn SessionIO) -> Result<(), ServiceError>;
    fn handle_input(&self, session: &mut dyn SessionIO, input: &str) -> Result<ServiceAction, ServiceError>;
    fn on_exit(&self, session: &mut dyn SessionIO);
}
```

**Key Modules**:
- `services/registry.rs` - Data-driven service registration from config.toml
- `menu/state.rs` - Menu state machine, hotkey handling, submenu navigation
- `connection/node_manager.rs` - Enforces max concurrent users (artificial scarcity)
- `connection/ceremony.rs` - Modem handshake, baud simulation, ANSI splash
- `terminal/ansi.rs` - ANSI escape codes, CP437→UTF-8, synchronized rendering
- `games/` - Door game registry (multi-game architecture)

**Database**: SQLite with sqlx. Main DB is `bbs.db`, games have separate DBs (e.g., `data/grand_theft_meth.db`).

### Frontend (TypeScript/xterm.js)

Minimal frontend - terminal rendering and WebSocket transport only. All logic lives in backend.

- `terminal.ts` - xterm.js setup with retro fonts
- `websocket.ts` - Binary WebSocket protocol to backend
- `crt-effects.ts` - CRT shader effects (F12 to cycle)
- `status-bar.ts` - Session timer display

### Configuration

`config.toml` at project root controls:
- Server settings, terminal dimensions
- Menu structure (hotkeys, submenus, services)
- Auth settings (lockout, session duration)
- Connection limits (max nodes, baud simulation, time limits)
- News feeds

Menus are fully data-driven - add/remove menu items via config, not code.

## Door Games

Door games use a multi-game architecture under `games/`:

```
backend/src/games/
├── mod.rs                    # Game registry
└── grand_theft_meth/         # Each game is self-contained
    ├── mod.rs                # Public exports
    ├── data.rs               # Static game data (cities, items, etc.)
    ├── state.rs              # GameState - player state, inventory
    ├── screen.rs             # GameScreen enum, state machine (GtmFlow)
    ├── render.rs             # ANSI UI rendering
    ├── economy.rs            # Bank, loans, casino
    ├── events.rs             # Random events, combat
    └── quest.rs              # Story/quest progression
```

**Key patterns**:
- `GtmFlow` - State machine managing screen transitions and input handling
- `GameScreen` - Enum of all possible game screens (MainMenu, Travel, Trade, etc.)
- `GtmAction` - Actions returned by input handlers (Render, Save, Exit, etc.)
- Each game has own database via `services/{game}/db.rs` (e.g., `grand_theft_meth.db`)

**Adding a new game**:
1. Create `games/{game_name}/` folder with same structure
2. Register in `games/mod.rs`: `pub mod {game_name};`
3. Create service in `services/{game_name}/` for session routing
4. Add menu entry in config.toml

## ANSI Art

- Use `AnsiWriter` from `terminal/ansi.rs` for all terminal output
- CP437 box-drawing characters converted to UTF-8 automatically
- 16-color CGA palette via `Color` enum
- Synchronized rendering with `begin_sync()`/`end_sync()` to prevent flicker

## Automated Agent Safety

When using `--dangerously-skip-permissions` for automated agents:
- MUST run inside Firecracker VM or equivalent isolated environment
- Agent only has access to the worktree directory
- Network access restricted to necessary endpoints only
- No access to credentials, SSH keys, or sensitive host files
- This protects against prompt injection and malicious code execution
