# Phase 4 Context: Time Limits & User Lists

**Phase Goal:** BBS enforces daily time limits and displays active/recent users
**Captured:** 2026-01-27

## Requirements

- TIME-01: Daily time limit per user level (enforced, visible countdown)
- TIME-02: Warnings at 5-minute and 1-minute remaining
- TIME-03: Graceful forced logout at zero (auto-save game state first)
- TIME-04: Time bank — save unused daily minutes for future sessions
- TIME-05: Session timer visible in status line
- USER-04: Last callers list showing recent login history
- USER-05: View other users' profiles

## Decisions

### 1. Timer Display & Status Bar

**Status bar location:** Row 24 (bottom of content area, leaving 23 usable content rows)
**Bar style:** Full-width background bar (Wildcat style)
**Visibility scope:** Always visible everywhere — menus, services, games, chat
**Content:** User name | Number of people online | Time remaining

**Warning colors (background):**
- Normal: Black background (standard text color)
- 5-minute warning: Yellow background (adjust text color dark for contrast)
- 1-minute warning: Red background (adjust text color light for contrast)

**Warning method:** Status bar color flash only — no overlay text interrupting the user

**Timer tick:** Client-side countdown. Server sends remaining time; client counts down per-minute normally, per-second during the last minute. This is a browser BBS, not a real BBS — no server push needed for timer display.

### 2. Time Banking

**Banking model:** Auto-save unused daily time with a configurable cap (minutes)
**Withdrawal trigger:** Prompt appears at the 1-minute warning
- If user responds: banked time is applied (show banked amount available)
- If user does not respond: no top-off, time expires normally
- Display banked time amount in the withdrawal prompt

**Time limits:** Configurable per user level in config.toml (e.g., Guest=30min, User=60min, Sysop=unlimited)
**Daily reset:** Midnight server time

### 3. Who's Online & Last Callers

**Who's Online display:** Handle + Node Number + Current Activity + Idle Time (full detail)
**Last Callers list:** Configurable count (default 15 entries)
**Caller info:** Handle + Date/Time of login + Session Duration

**Profile viewing:** Both methods supported:
1. From Who's Online list — select a user to view their profile
2. Standalone lookup — enter a handle to view any user's profile

### 4. Graceful Timeout Behavior

**Timeout action:** Immediate goodbye screen + disconnect
- Show timeout-specific goodbye screen (different from normal quit — "Time expired" messaging)
- 3-second delay for user to read, then disconnect

**Game state on timeout:** Games receive a timeout lifecycle event
- Game-specific handling — some games may persist state, others may not
- The BBS sends the event; each game decides what to do with it

**Reconnection after timeout:** User can reconnect only if they have banked time available
- Zero daily time + zero banked time = cannot log in until midnight reset
- Zero daily time + banked time available = allowed to log in (banked time used)

## Existing Architecture Context

Phase 4 builds on:
- **Session management** (Phase 2): Session struct in `backend/src/websocket/session.rs` with user info, tx channel
- **Node manager** (Phase 2): `backend/src/services/node_manager.rs` tracks active nodes with user info
- **Menu system** (Phase 3): MenuSession state machine in `backend/src/menu/` with navigation
- **Config system** (Phase 1): `backend/src/config.rs` with TOML deserialization, `#[serde(default)]` pattern
- **AnsiWriter** (Phase 1): `backend/src/terminal/ansi.rs` for formatted output
- **User model** (Phase 2): `backend/src/models/user.rs` with stats tracking
- **Auth system** (Phase 2): Session tokens, login/logout lifecycle
- **Goodbye sequence** (Phase 2): `handle_quit()` in session.rs with NO CARRIER disconnect

## Success Criteria

1. User sees session timer countdown in status bar (row 24)
2. User receives warning via status bar color change at 5 minutes (yellow) and 1 minute (red)
3. User is gracefully logged out at zero time with timeout-specific goodbye screen
4. Unused daily time auto-saved to bank; withdrawal prompted at 1-minute warning
5. User can view Who's Online list (handle, node, activity, idle)
6. User can view Last Callers list (configurable count, default 15)
7. User can view other users' profiles (from Who's Online or standalone lookup)

---
*Context captured: 2026-01-27*
