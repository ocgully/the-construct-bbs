---
phase: 02-authentication-connection
plan: 05
subsystem: auth-session
tags: [login, session, token, localStorage, state-machine, argon2, lockout]
depends_on:
  requires: ["02-03", "02-04"]
  provides: ["login-flow", "session-persistence", "auth-state-machine", "token-storage"]
  affects: ["03-*"]
tech_stack:
  added: ["serde_json"]
  patterns: ["AuthState enum", "free-function borrow pattern", "JSON control messages"]
key_files:
  created:
    - backend/src/services/login.rs
    - backend/src/db/login_attempts.rs
  modified:
    - backend/src/websocket/session.rs
    - backend/src/websocket/mod.rs
    - backend/src/services/mod.rs
    - backend/src/db/mod.rs
    - backend/Cargo.toml
    - frontend/src/websocket.ts
decisions:
  - id: "02-05-01"
    description: "Free function pattern for send_colored_prompt to avoid Rust borrow checker conflicts"
    rationale: "LoginFlow borrows self.auth_state; self.send_prompt() needs &mut self -- incompatible. Free function takes &tx directly."
  - id: "02-05-02"
    description: "Ceremony deferred to handle_input (not on_connect)"
    rationale: "Frontend sends auth JSON immediately on connect. Server must receive that first to decide: resume session (skip ceremony) or fresh connect (run ceremony)."
  - id: "02-05-03"
    description: "on_disconnect deletes session from DB"
    rationale: "Prevents stale sessions blocking duplicate-session detection. Token cleared on explicit logout too."
  - id: "02-05-04"
    description: "Clone tx sender for login/registration handlers"
    rationale: "mpsc::Sender is Clone; cloning avoids borrow conflicts when flow borrows self.auth_state and we need to send via self.tx"
metrics:
  duration: "9 min"
  completed: "2026-01-27"
  tests_added: 13
  tests_total: 122
---

# Phase 02 Plan 05: Login Flow and Session Persistence Summary

JWT-less session auth with Argon2id password verification, localStorage token persistence, and full AuthState lifecycle management in the WebSocket session layer.

## What Was Built

### Task 1: Login Flow State Machine
- **LoginFlow** struct with `EnterHandle` and `EnterPassword { handle }` states
- **LoginResult** enum: Continue, Error, Success, SwitchToRegistration, Locked
- Character-by-character input handling with asterisk masking for passwords
- Handle validation: empty rejected, "new" (case-insensitive) switches to registration
- Lockout check before password attempt (configurable max_attempts + lockout_minutes)
- Email verification check (unverified accounts rejected)
- Password verification via `spawn_blocking` (Argon2 is CPU-intensive)
- Duplicate session detection (blocks concurrent logins for same user)
- Sysop level override from config.auth.sysop_handles
- Login header: box-drawn ANSI with BBS name, tagline, node count
- Welcome-back message: handle, last login date, total calls
- **login_attempts DB module**: record_login_attempt, get_recent_failures, is_locked_out

### Task 2: Session Auth State Management and Token Persistence
- **AuthState enum**: AwaitingAuth, ConnectionCeremony, Login(LoginFlow), Registration(RegistrationFlow), Authenticated
- Full auth lifecycle in session.rs: auth token check -> ceremony -> login/register -> authenticated
- Session token sent to frontend as JSON `{ type: "session", token }` via tx channel
- Frontend intercepts JSON messages (session, logout types) -- not written to terminal
- Frontend stores token in localStorage (`bbs_session_token`)
- Frontend sends token on reconnect (already wired in Plan 03)
- Session resumption: valid token skips ceremony, assigns node, shows welcome-back + main menu
- Registration flow wired: "new" at login prompt transitions to registration; completion returns to login
- Ceremony deferred: runs after auth message received (not during on_connect)
- Logout sends `{ type: "logout" }` JSON to clear frontend token
- Disconnecting state checked in WebSocket recv loop for clean shutdown
- Session cleanup on disconnect: deletes DB session, releases node

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added serde_json dependency**
- **Found during:** Task 1 (preparing for Task 2)
- **Issue:** session.rs needs JSON parsing/serialization for auth messages and token sending
- **Fix:** Added `serde_json = "1"` to Cargo.toml
- **Commit:** a711ac2

**2. [Rule 1 - Bug] Rust borrow checker conflicts in session handlers**
- **Found during:** Task 2
- **Issue:** `flow` borrows `self.auth_state` mutably; `self.send_prompt()` needs `&mut self` -- double mutable borrow
- **Fix:** Extracted `send_colored_prompt` as a free function taking `&tx`; restructured handlers to scope flow borrows within blocks; clone `tx` sender for independent sends
- **Commit:** 4b47256

**3. [Rule 2 - Missing Critical] Ceremony must be deferred**
- **Found during:** Task 2
- **Issue:** Plan says ceremony runs on_connect, but frontend sends auth JSON immediately. If ceremony runs first, the auth message arrives during ceremony (ignored). Session can never resume.
- **Fix:** on_connect() now just returns true. Ceremony runs in handle_awaiting_auth() after receiving the auth message and determining token is invalid/missing.
- **Commit:** 4b47256

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| Free function for prompt sending | Avoids Rust borrow checker conflict between auth_state flow and tx sender |
| Ceremony deferred to handle_input | Must receive auth token first to decide: resume or fresh ceremony |
| Session deleted on disconnect | Prevents stale sessions blocking duplicate-session detection |
| Clone tx sender in handlers | mpsc::Sender is cheap to clone; avoids all borrow conflicts cleanly |

## Commits

| Hash | Message |
|------|---------|
| a711ac2 | feat(02-05): add login flow state machine and login attempts DB |
| 4b47256 | feat(02-05): add session auth state management and token persistence |

## Verification Results

- `cargo check`: passes (only pre-existing dead code warnings)
- `cargo test`: 122/122 pass (13 new login tests + 3 login_attempts tests)
- `npm run build`: passes (frontend compiles with JSON interception)

## Next Phase Readiness

Phase 2 (Authentication & Connection) is now **COMPLETE**. All 5 plans executed:
- 02-01: Database layer and config extensions
- 02-02: Auth core and node manager
- 02-03: Connection ceremony and modem audio
- 02-04: Registration flow and email verification
- 02-05: Login flow and session persistence

The full "dial in -> authenticate -> enter BBS" flow is implemented:
1. Frontend connects, sends auth token (or null)
2. Server validates token: valid = resume session; invalid = run ceremony + login
3. Login: handle -> password (masked) -> session token -> welcome-back -> main menu
4. Registration: "new" -> handle -> email -> password -> verify code -> login
5. Session persists across page refresh via localStorage token
6. Duplicate sessions blocked, failed logins locked out after N attempts

Ready for Phase 3 (Message Boards) which can build on the authenticated session.
