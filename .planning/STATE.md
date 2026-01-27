# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-26)

**Core value:** The feeling of dialing into an exclusive, underground system — artificial scarcity, ANSI art, and social games that only work because everyone's sharing the same constrained space.
**Current focus:** Phase 2 - Authentication & Connection

## Current Position

Phase: 2 of 14 (Authentication & Connection)
Plan: 1 of 5 in current phase
Status: In progress
Last activity: 2026-01-27 -- Completed 02-01-PLAN.md (Database Layer and Config Extensions)

Progress: [██████░░░░] 100% of Phase 1 (5/5), 20% of Phase 2 (1/5)

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: 5 min
- Total execution time: 0.9 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan | Status |
|-------|-------|-------|----------|--------|
| 01    | 5     | 24min | 5min     | Complete |
| 02    | 1     | 7min  | 7min     | In progress |

**Recent Trend:**
- Last 5 plans: 3min, 7min, 3min, 4min, 7min
- Trend: Consistently fast execution (3-7min range)

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

## Phase 2 Progress

**Authentication & Connection Phase: IN PROGRESS**

Plans completed:
- 02-01: Database layer and config extensions (SQLite + SQLx pool, schema, User CRUD, config)

Plans remaining:
- 02-02: Password authentication
- 02-03: Session management
- 02-04: Registration flow
- 02-05: Connection ceremony

## Session Continuity

Last session: 2026-01-27
Stopped at: Completed 02-01-PLAN.md (Database Layer and Config Extensions)
Resume file: None
Next action: Phase 2, Plan 02 - Password Authentication

---
*State initialized: 2026-01-26*
*Last updated: 2026-01-27*
