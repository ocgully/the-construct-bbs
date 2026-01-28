# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-26)

**Core value:** The feeling of dialing into an exclusive, underground system — artificial scarcity, ANSI art, and social games that only work because everyone's sharing the same constrained space.
**Current focus:** Phase 4 - Time Limits & User Lists (IN PROGRESS)

## Current Position

Phase: 4 of 14 (Time Limits & User Lists)
Plan: 6 of 6 in current phase
Status: Phase complete
Last activity: 2026-01-28 -- Completed 04-04-PLAN.md (Who's Online and Last Callers)

Progress: [█████████████] 100% of Phase 1 (5/5), 100% of Phase 2 (7/7), 100% of Phase 3 (3/3), 100% of Phase 4 (6/6)

## Performance Metrics

**Velocity:**
- Total plans completed: 21
- Average duration: 6 min
- Total execution time: 2.1 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan | Status |
|-------|-------|-------|----------|--------|
| 01    | 5     | 24min | 5min     | Complete |
| 02    | 7     | 45min | 6min     | Complete (incl. integration) |
| 03    | 3     | 16min | 5min     | Complete |
| 04    | 6     | 42min | 7min     | Complete |

**Recent Trend:**
- Last 5 plans: 3min, 9min, 8min, 4min, 7min
- Trend: Consistently fast execution (3-9min range)

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
| 03-01 | MenuItem enum uses internally-tagged serde with type/hotkey/name/order/min_level | Clean TOML syntax for menu configuration |
| 03-01 | All menu fields use #[serde(default)] for graceful config loading | Menu section entirely optional in config.toml |
| 03-01 | 26 Stoic quotes embedded for MOTD rotation (not configurable text files) | Thematic consistency with "The Construct" atmosphere |
| 03-01 | Future service items commented out in config.toml with phase annotations | Sysop visibility into planned features |
| 03-02 | Q key dual behavior: BackToMain in submenu, ExecuteCommand(quit) at main | Submenus give Q precedence for consistent Back behavior |
| 03-02 | drain_buffer stops at LaunchService/ExecuteCommand | Preserves remaining buffer for next menu, prevents over-consumption |
| 03-02 | Adaptive column layout threshold at 7 items | Main menu switches to two columns when >7 items for balance |
| 03-02 | MOTD quote called in render function (not passed as parameter) | Keeps quote random on each menu redraw |
| 03-03 | MenuSession created on authentication in all paths (login, registration, resume) | Menu state ready immediately when authenticated |
| 03-03 | User level string mapped to u8: Sysop=255, User=0 | Consistent numeric values for menu filtering |
| 03-03 | Single-keypress navigation: process each char individually through MenuSession | Matches Wildcat BBS immediate response behavior |
| 03-03 | Type-ahead buffer drained after EnterSubmenu for command stacking | Enables G1 to go directly to Games > item 1 |
| 03-03 | Menu state reset to MainMenu via reset_to_main() on service exit | Prevents user stuck in submenu after exiting service |
| 04-01 | Time limits configurable per user level (Guest/User/Sysop) in config.toml | Phase 4 timer and banking features use per-level daily_minutes and time_bank_cap |
| 04-01 | Session history uses datetime('now', '-5 hours') for EST timezone consistency | All datetime fields maintain EST timezone offset |
| 04-01 | Time banking with daily reset detection using SQLite date() comparison | Daily reset triggered by comparing date(last_daily_reset) < date('now', '-5 hours') |
| 04-01 | NodeManager tracks current_activity string and last_input timestamp | Enables Who's Online display and idle detection |
| 04-02 | Timer ticks per-minute normally, switches to per-second in final minute | Minimizes WebSocket traffic while providing accurate countdown in final minute |
| 04-02 | expired and low_time flags exposed via Arc<AtomicBool> for session polling | Session can check timer state without blocking on async task |
| 04-02 | CancellationToken enables clean timer cancellation on quit | tokio::select! races timer ticks against cancellation signal |
| 04-02 | Timeout goodbye uses LightRed border vs LightCyan for normal quit | Visual distinction between time-expired and voluntary disconnect |
| 04-04 | Who's Online and Last Callers use render functions (not Service trait) for async data access | Session fetches data async, calls render_* functions for display |
| 04-04 | 80-column table width with careful border/data column calculation | Both tables: 75 data columns + 5 borders = 80 total |
| 04-05 | Phase 4 features as main menu commands (not submenu) | W/L/U hotkeys provide direct access to Who's Online, Last Callers, User Lookup |
| 04-05 | User lookup reuses render_profile_card with is_own_profile=false | Consistent profile display without edit options when viewing others |

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

## Phase 3 Completion Summary

**Navigation System Phase: COMPLETE (3/3 plans complete)**

All plans completed:
- 03-01: Menu configuration schema (MenuItem enum, MenuConfig, TOML definitions, Stoic quotes)
- 03-02: Menu state machine and rendering (MenuSession, TypeAheadBuffer, ANSI rendering)
- 03-03: Navigation logic integration (MenuSession lifecycle, single-keypress navigation, command stacking)

Full navigation system implemented:
1. Config-driven menu structure defined in config.toml with level-gating
2. MenuSession state machine with type-ahead buffer for command stacking
3. Wildcat-style ANSI rendering (double-line main menu, single-line submenus)
4. Integrated into session lifecycle with single-keypress hotkey navigation
5. Main menu: MOTD quotes, user info, adaptive 2-column layout for >7 items
6. Submenus: [Q] Back to Main Menu, hotkey items, help via ?
7. Profile and Quit commands functional from main menu
8. Service launch/exit properly managed with menu state transitions

## Phase 4 Completion Summary

**Time Limits & User Lists Phase: COMPLETE (6/6 plans complete)**

All plans completed:
- 04-01: Time Limits Foundation (TimeLimitsConfig, session_history table, time banking queries, NodeManager extensions)
- 04-02: Session Timer & Status Bar (Timer task spawning, status bar renderer, client-side countdown)
- 04-03: Status Bar Integration (WebSocket timer messages, status bar positioning, warning colors)
- 04-04: User Lists Display (Who's Online and Last Callers ANSI table rendering)
- 04-05: User Lists Menu Integration (User profile lookup renders, main menu registration W/L/U)
- 04-06: Session lifecycle integration (timeout handling, time banking withdrawal, graceful disconnect)

Full time limits and user lists system implemented:
1. Per-level time limits (Guest/User/Sysop) configured in config.toml
2. Session timer with per-minute countdown (per-second in final minute)
3. Status bar at row 24 showing user, online count, time remaining
4. Warning colors at 5min (yellow) and 1min (red)
5. Time banking with daily reset and withdrawal prompt
6. Who's Online display with Node/Handle/Activity/Idle columns
7. Last Callers list with Handle/Date/Time/Duration
8. User profile lookup by handle
9. Graceful timeout with timeout-specific goodbye screen
10. Session history tracking with login/logout timestamps

## Session Continuity

Last session: 2026-01-28
Stopped at: Completed 04-04-PLAN.md (Who's Online and Last Callers)
Resume file: None
Next action: Phase 4 complete - ready for Phase 5

---
*State initialized: 2026-01-26*
*Last updated: 2026-01-28 (04-04 user lists display)*
