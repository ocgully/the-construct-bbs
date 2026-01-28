---
phase: 05-email-system
plan: 04
subsystem: ui
tags: [status-bar, mail-indicator, timer, websocket, ansi]

# Dependency graph
requires:
  - phase: 05-01
    provides: get_unread_count function in messages.rs
  - phase: 04-02
    provides: SessionTimer and status bar infrastructure
provides:
  - Backend timer sends has_mail flag on each tick
  - Frontend status bar renders MAIL indicator when unread messages exist
  - Mail accessible from main menu via M hotkey
affects: [05-05-integration, future-notification-system]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Timer queries database on each tick for real-time state updates"
    - "ANSI color codes embedded in status bar content with visible length accounting"

key-files:
  created: []
  modified:
    - backend/src/connection/timer.rs
    - backend/src/websocket/session.rs
    - frontend/src/status-bar.ts
    - frontend/src/timer.ts
    - config.toml

key-decisions:
  - "Timer checks unread mail on every tick (per-minute and per-second) - indexed query, negligible overhead"
  - "MAIL indicator placed between handle and online count in status bar layout"
  - "Yellow bold ANSI styling for MAIL indicator using \\x1b[33m\\x1b[1m"
  - "Changed Mail from submenu to command in config.toml (M hotkey triggers mail command)"

patterns-established:
  - "Timer failures (DB query errors) fail silently with false flag - don't break timer for auxiliary features"
  - "Status bar ANSI codes embedded in content with separate visible length calculation"

# Metrics
duration: 6min
completed: 2026-01-28
---

# Phase 05 Plan 04: MAIL Indicator Summary

**Status bar displays real-time MAIL indicator when user has unread messages, updated automatically on timer ticks**

## Performance

- **Duration:** 6 min
- **Started:** 2026-01-28T05:27:19Z
- **Completed:** 2026-01-28T05:33:19Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Backend timer checks unread mail count on every tick and includes has_mail flag in JSON
- Frontend status bar renders yellow bold "MAIL" indicator between handle and online count
- MAIL indicator appears/disappears automatically as messages are read without user action
- Mail command accessible from main menu via M hotkey (config.toml updated from submenu to command)

## Task Commits

Each task was committed atomically:

1. **Task 1: Backend timer sends has_mail flag** - `b7ed2ea` (feat)
2. **Task 2: Frontend MAIL indicator and menu config** - `285640c` (feat)

## Files Created/Modified
- `backend/src/connection/timer.rs` - Added user_id and pool parameters, checks get_unread_count on each tick, includes has_mail in all timer JSON messages
- `backend/src/websocket/session.rs` - Updated all SessionTimer::spawn call sites to pass user_id and pool
- `frontend/src/status-bar.ts` - Added hasMail property, renders yellow bold "MAIL" text with ANSI color codes, accounts for ANSI codes in visible length calculation
- `frontend/src/timer.ts` - Passes has_mail from server data to status bar update
- `config.toml` - Changed Mail from submenu to command type for M hotkey

## Decisions Made

**Timer overhead acceptable for real-time updates:**
- Timer checks unread count on every tick (per-minute normally, per-second in final minute)
- Query is indexed (recipient_id, is_read) and fast
- Fail-silent pattern (errors return false) prevents timer breaking on DB issues

**MAIL indicator placement and styling:**
- Positioned between handle (left) and online count (center) in status bar
- Yellow bold (\\x1b[33m\\x1b[1m) stands out without being alarm-level
- Visible length calculation accounts for embedded ANSI codes to maintain 80-column layout

**Menu configuration change:**
- Mail changed from submenu type to command type in config.toml
- Enables direct inbox access via M hotkey from main menu
- Command handler implemented by Plan 05-03 (parallel execution)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Backend compilation blocked during Task 1:**
- Issue: Plan 05-03 (Mail command handlers) was still executing in parallel, causing missing method errors
- Resolution: Verified Task 1 changes in isolation, continued to Task 2, re-checked after Plan 05-03 completed
- Result: Backend compiles successfully with all changes integrated

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- MAIL indicator functional and integrated with timer system
- Ready for Phase 5 integration testing (05-05)
- Future notification system can extend has_mail pattern to other real-time indicators
- Status bar layout can accommodate additional indicators (current: handle, MAIL, online, time)

---
*Phase: 05-email-system*
*Completed: 2026-01-28*
