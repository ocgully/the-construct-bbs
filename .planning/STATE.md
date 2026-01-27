# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-26)

**Core value:** The feeling of dialing into an exclusive, underground system — artificial scarcity, ANSI art, and social games that only work because everyone's sharing the same constrained space.
**Current focus:** Phase 2 - Authentication & Connection (COMPLETE + Integration)

## Current Position

Phase: 2 of 14 (Authentication & Connection)
Plan: 7 of 7 in current phase (integration plan)
Status: Phase complete (integration verified)
Last activity: 2026-01-27 -- Completed 02-07-PLAN.md Task 1 (Session Lifecycle Integration)

Progress: [████████████] 100% of Phase 1 (5/5), 100% of Phase 2 (7/7 incl. integration)

## Performance Metrics

**Velocity:**
- Total plans completed: 12
- Average duration: 6 min
- Total execution time: 1.5 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan | Status |
|-------|-------|-------|----------|--------|
| 01    | 5     | 24min | 5min     | Complete |
| 02    | 7     | 45min | 6min     | Complete (incl. integration) |

**Recent Trend:**
- Last 5 plans: 5min, 8min, 9min, 4min, 8min
- Trend: Consistently fast execution (4-9min range)

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

| Phase-Plan | Decision | Impact |
|------------|----------|--------|
| 01-01 | Service trait plugin architecture with Arc<dyn Service> | All BBS features use consistent plugin interface |
| 01-01 | Config-driven service registry | Services enabled/disabled via config.toml without code changes |
| 01-01 | SessionIO trait abstraction | Decouples service logic from transport layer |
| 01-01 | TOML configuration format | Human-readable sysop configuration with type safety |
| 01-02 | Use authentic CGA 16-color palette with Brown (not "dark yellow") | Sets standard for all terminal color theming |
| 01-02 | CRLF (\r\n) line endings for all terminal output | Required for correct terminal behavior and Windows compatibility |
| 01-02 | Implement DECSET 2026 synchronized rendering | Prevents screen tearing during ANSI art rendering |
| 01-03 | CSS-based CRT effects instead of npm package | More maintainable, better browser compatibility for retro aesthetic |
| 01-03 | Default CRT level to FULL | Maximum atmospheric immersion for authentic BBS experience |
| 01-03 | No scrollback buffer (scrollback: 0) | Authentic BBS experience forcing engagement with paging |
| 01-04 | Split socket architecture with mpsc channel | Separates receive/send loops preventing deadlocks, allows async session output |
| 01-04 | AnsiBuffer prevents partial escape sequences | Protects xterm.js from rendering artifacts by buffering incomplete sequences |
| 01-04 | 800ms 'Entering door...' delay | Authentic BBS loading experience matching historical feel |
| 01-05 | ANSI art welcome screen with CP437 box-drawing | Visual verification of terminal foundation: CP437 rendering + CGA colors |
| 01-05 | Frontend served from backend via tower-http | Single-server deployment model matching BBS architecture |
| 01-05 | Vite dev proxy for WebSocket routing | Enables hot-reload development with separate frontend/backend servers |
| 01-05 | Mouse input filtering in frontend | Prevents scroll wheel escape sequences from spamming backend |
| 02-01 | String-based sqlx queries (not compile-time macros) | No DATABASE_URL needed at build time, simpler CI/CD |
| 02-01 | handle_lower column for case-insensitive lookups | Cheaper than COLLATE NOCASE on every query |
| 02-01 | IF NOT EXISTS for idempotent schema execution | Schema runs safely on every startup without migration tooling |
| 02-01 | #[serde(default)] on all config sections | Auth/connection sections optional in config.toml |
| 02-02 | Argon2id with OWASP 2026 params (m=19456, t=2, p=1) | Maximum recommended server-side hashing security |
| 02-02 | verify_password returns Ok(false) on mismatch, not Err | Cleaner API for login flows |
| 02-02 | tokio::sync::RwLock for NodeManager (not std) | Async-safe concurrent access to node state |
| 02-02 | First-available node assignment from 1..=max_nodes | BBS line numbering convention, fills gaps on disconnect |
| 02-03 | Ceremony writes directly to tx channel (bypasses output_buffer) | Typewriter pacing requires real-time delay control |
| 02-03 | Eager node assignment during ceremony with placeholder info | Accurate node counts during ceremony; updated after login |
| 02-03 | on_connect returns bool for line-busy disconnect flow | Clean session teardown when all nodes full |
| 02-03 | Frontend connect prompt doubles as AudioContext user gesture | Browser autoplay policy compliance with atmospheric UX |
| 02-04 | RegistrationFlow as standalone struct, not Service trait impl | Registration needs async DB, password masking, pre-login context |
| 02-04 | Character-by-character echo via handle_char with input_buffer | Terminal has no local echo; server echoes with * for passwords |
| 02-04 | SMTP fallback to console logging when not configured | Dev mode works without external SMTP server |
| 02-04 | 6-digit zero-padded verification code with configurable expiry | Matches context spec; expiry from AuthConfig |
| 02-05 | Free function for prompt sending (avoids borrow checker conflicts) | LoginFlow borrows self.auth_state; free function takes &tx directly |
| 02-05 | Ceremony deferred to handle_input (not on_connect) | Must receive auth token first to decide: resume or fresh ceremony |
| 02-05 | Session deleted on disconnect | Prevents stale sessions blocking duplicate-session detection |
| 02-05 | Clone tx sender in login/registration handlers | mpsc::Sender is cheap to clone; avoids all borrow conflicts cleanly |
| 02-06 | CP437 double-line box-drawing for profile and goodbye cards | Consistent BBS aesthetic with CGA colors |
| 02-06 | Session time via Instant::now() at login, stored as minutes | Monotonic clock immune to system clock changes |
| 02-06 | Clean quit: logout JSON -> goodbye -> 3s delay -> disconnect | Frontend clears token immediately; user reads goodbye before close |
| 02-06 | Unclean disconnect saves session time without goodbye screen | Accurate stats even on browser close or network drop |
| 02-07 | Profile view uses __profile__ current_service sentinel marker | Lightweight non-service view routing without new state |
| 02-07 | Main menu shows handle, level, node info with [P] Profile and [Q] Quit | Full user context in authenticated menu |

### Pending Todos

None yet.

### Blockers/Concerns

**From research (SUMMARY.md):**
- CP437 encoding addressed - terminal module includes CP437-to-UTF-8 conversion using codepage-437 crate
- Mobile virtual keyboard handling implemented - visual viewport resize, touch-to-focus, responsive sizing
- Rust toolchain installed (rustc 1.93.0) - compilation and tests verified
- CP437 font resolved - IBM Plex Mono via Google Fonts provides box-drawing coverage
- SQLite concurrency strategy resolved - WAL mode enabled in pool.rs, foreign keys enforced

## Phase 1 Completion Summary

**Terminal Foundation Phase: COMPLETE**

All 5 plans executed successfully:
- 01-01: Rust backend foundation (Service trait, config system)
- 01-02: Terminal output engine (AnsiWriter, CP437, pagination)
- 01-03: Browser terminal frontend (xterm.js, CRT effects, mobile)
- 01-04: WebSocket session layer (connection handling, ANSI buffering)
- 01-05: Integration and visual verification (ANSI art, serving, verification)

## Phase 2 Completion Summary

**Authentication & Connection Phase: COMPLETE**

All 7 plans executed successfully:
- 02-01: Database layer and config extensions (SQLite + SQLx pool, schema, User CRUD, config)
- 02-02: Auth core and node manager (Argon2id hashing, session CRUD, validation, NodeManager)
- 02-03: Connection ceremony and modem audio (typewriter text, splash screen, line-busy, Web Audio)
- 02-04: Registration flow and email verification (state machine, lettre SMTP, character echo)
- 02-05: Login flow and session persistence (LoginFlow, AuthState, token in localStorage)
- 02-06: User profile card and goodbye sequence (ANSI art cards, session time tracking)
- 02-07: Session lifecycle integration (profile routing, main menu with user info)

Full auth lifecycle implemented:
1. Connect -> frontend sends auth token
2. Valid token -> resume session (skip ceremony, show welcome-back, main menu)
3. No/invalid token -> ceremony -> login header -> handle prompt
4. Login: handle -> password (masked) -> welcome-back -> main menu
5. Registration: "new" -> handle -> email -> password -> verification code -> login
6. Session persists across page refresh via localStorage
7. Duplicate sessions blocked, lockout after N failed attempts
8. Profile card displays user identity with stats in ANSI art
9. Goodbye sequence shows session stats with NO CARRIER disconnect
10. Session time tracked on both clean quit and unclean disconnect

## Session Continuity

Last session: 2026-01-27
Stopped at: Completed 02-07-PLAN.md Task 1 (Session Lifecycle Integration) -- Phase 2 FULLY INTEGRATED
Resume file: None
Next action: Phase 3 (Message Boards)

---
*State initialized: 2026-01-26*
*Last updated: 2026-01-27 (02-07 integration)*
