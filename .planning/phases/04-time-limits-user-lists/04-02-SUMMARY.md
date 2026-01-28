---
phase: 04-time-limits-user-lists
plan: 02
subsystem: connection
tags: [tokio, timer, cancellation-token, session-management, ansi-art, timeout]

# Dependency graph
requires:
  - phase: 02-auth-connection
    provides: Session management, WebSocket tx channel, goodbye screen pattern
  - phase: 04-01
    provides: TimeLimitsConfig with per-level time limits

provides:
  - SessionTimer struct with per-minute/per-second countdown task
  - CancellationToken-based graceful timer cancellation
  - Timer JSON message protocol for client status bar
  - Timeout-specific goodbye screen with time-expired messaging
  - Timer expiry flags (expired, low_time) for session polling

affects: [04-03, 04-04, 04-05, 04-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "tokio::spawn background timer task with CancellationToken cleanup"
    - "Arc<AtomicBool> for cross-task state flags (expired, low_time)"
    - "tokio::select! racing timer ticks against cancellation"
    - "Distinct ANSI goodbye screens for normal quit vs timeout"

key-files:
  created:
    - backend/src/connection/timer.rs
  modified:
    - backend/src/connection/mod.rs
    - backend/src/services/goodbye.rs

key-decisions:
  - "Timer ticks per-minute normally, switches to per-second in final minute"
  - "Timer sends JSON messages with remaining time, unit (min/sec/unlimited), warning level"
  - "expired and low_time flags exposed via Arc<AtomicBool> for session polling"
  - "CancellationToken enables clean timer cancellation on quit"
  - "Timeout goodbye uses LightRed border vs LightCyan for normal quit"

patterns-established:
  - "SessionTimer::spawn() returns struct with cancel/wait/is_expired/is_low_time methods"
  - "TimerResult enum distinguishes Expired vs Cancelled outcomes"
  - "Unlimited time (sysop) sends special 'unlimited' unit and never expires"
  - "render_timeout_goodbye() follows same CP437 box-drawing pattern as render_goodbye()"

# Metrics
duration: 6min
completed: 2026-01-27
---

# Phase 4 Plan 02: Session Timer & Status Bar Summary

**Backend session timer task with per-minute/per-second countdown, CancellationToken cleanup, and timeout-specific ANSI goodbye screen**

## Performance

- **Duration:** 6 min
- **Started:** 2026-01-27T23:36:40Z
- **Completed:** 2026-01-27T23:42:33Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- SessionTimer spawns async tokio task that ticks per-minute normally, per-second in final minute
- Timer sends JSON messages to client with remaining time, unit, warning level, handle, and users_online
- CancellationToken enables graceful cancellation on clean quit (select! races timer against cancel)
- expired and low_time flags exposed via Arc<AtomicBool> for session polling (timeout trigger, bank withdrawal)
- render_timeout_goodbye() creates distinct ANSI art screen for time-expired disconnects with LightRed border

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Session Timer Task with CancellationToken** - `ce66ae1` (feat)
2. **Task 2: Timeout Goodbye Screen** - `7ca1e98` (feat)

## Files Created/Modified
- `backend/src/connection/timer.rs` - SessionTimer struct with spawn/cancel/wait/is_expired/is_low_time methods
- `backend/src/connection/mod.rs` - Added pub mod timer declaration
- `backend/src/services/goodbye.rs` - Added render_timeout_goodbye function for time-expired disconnects

## Decisions Made
- **Timer tick rate switching:** Per-minute ticks normally to minimize WebSocket traffic. Switches to per-second ticks in final minute for accurate countdown display.
- **Unlimited time handling:** Sysop level (0 minutes) sends single "unlimited" unit message then waits for cancellation, never expires.
- **Warning level calculation:** "normal" for >5 minutes, "yellow" for <=5 minutes, "red" for <=1 minute.
- **Timeout goodbye color:** LightRed border (warning feel) distinguishes from LightCyan normal quit.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation followed plan specification directly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Timer infrastructure complete for remaining Phase 4 plans:
- Plan 04-03 can integrate SessionTimer into WebSocket session lifecycle
- Plan 04-04 can use timer_warning JSON messages for status bar color changes
- Plan 04-05 can use low_time flag to trigger time bank withdrawal prompt
- Plan 04-06 can use expired flag to trigger graceful timeout sequence with render_timeout_goodbye

All timer task mechanisms, JSON message protocols, and timeout screens ready for integration.

---
*Phase: 04-time-limits-user-lists*
*Completed: 2026-01-27*
