---
phase: 04-time-limits-user-lists
plan: 03
subsystem: ui
tags: [typescript, xterm.js, websocket, ansi, status-bar, timer]

# Dependency graph
requires:
  - phase: 01-terminal-foundation
    provides: xterm.js terminal, WebSocket connection, ANSI rendering foundation
  - phase: 02-auth-connection
    provides: Session management, user authentication
  - phase: 04-01
    provides: TimeLimitsConfig, time banking database schema

provides:
  - StatusBar class rendering ANSI status bar at row 24 with color-coded warnings
  - SessionTimer class with client-side countdown synchronized to server updates
  - WebSocket JSON message interception for timer/timeout/timer_warning types
  - Timer lifecycle management (start on auth, stop on logout/disconnect/timeout)

affects: [04-02, 04-04, 04-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ANSI cursor save/restore pattern for persistent UI elements"
    - "Client-side countdown with periodic server sync to minimize WebSocket traffic"
    - "Optional parameter pattern for WebSocket handler extensions"

key-files:
  created:
    - frontend/src/status-bar.ts
    - frontend/src/timer.ts
  modified:
    - frontend/src/websocket.ts
    - frontend/src/main.ts

key-decisions:
  - "Status bar uses blue background (normal), yellow (5min warning), red (1min warning)"
  - "Client-side countdown ticks per-minute normally, switches to per-second in last minute"
  - "Timer JSON messages intercepted by WebSocket handler, never written to terminal"
  - "DEC cursor save/restore (ESC 7, ESC 8) for ANSI status bar positioning"
  - "Timer instance passed via options parameter to maintain backward compatibility"

patterns-established:
  - "StatusBar.update() pattern for incremental field updates without full re-render"
  - "SessionTimer.restartCountdown() clears old interval before creating new to prevent duplicates"
  - "WebSocket opts parameter pattern for optional feature injection"

# Metrics
duration: 3min
completed: 2026-01-28
---

# Phase 4 Plan 03: Status Bar & Client Timer Summary

**Client-side status bar with ANSI positioning at row 24, displaying countdown timer with color-coded warnings (blue/yellow/red)**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-28T00:43:34Z
- **Completed:** 2026-01-28T00:46:39Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- StatusBar class renders persistent ANSI bar at row 24 with save/restore cursor positioning
- SessionTimer class maintains client-side countdown with per-minute ticking (switches to per-second in last minute)
- WebSocket handler intercepts timer/timer_warning/timeout JSON messages
- Timer lifecycle integrated with authentication (show on login, hide on logout/disconnect/timeout)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Status Bar Renderer and Client-Side Timer** - `0ab30ad` (feat)
2. **Task 2: Wire Timer into WebSocket Message Handler** - `7fb989a` (feat)

## Files Created/Modified
- `frontend/src/status-bar.ts` - StatusBar class with ANSI positioning, color-coded warning levels
- `frontend/src/timer.ts` - SessionTimer class with client-side countdown and server sync
- `frontend/src/websocket.ts` - Added timer message interception, opts parameter for timer instance
- `frontend/src/main.ts` - Created StatusBar and SessionTimer instances, passed to connectWebSocket

## Decisions Made
- **Color scheme:** Blue background for normal state (more visible than black), yellow at 5min, red at 1min
- **DEC cursor sequences:** Using ESC 7/ESC 8 for save/restore (more widely supported than SCO ESC[s/ESC[u)
- **Interval management:** Clear old interval before creating new one to prevent duplicate ticking
- **Unlimited time handling:** Special case for unlimited time (displays "Unlimited", no countdown)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Removed unused lastUnit field**
- **Found during:** Task 1 (Status bar and timer creation)
- **Issue:** timer.ts had unused private field lastUnit that was in plan skeleton but not used
- **Fix:** Removed the field declaration
- **Files modified:** frontend/src/timer.ts
- **Verification:** TypeScript compilation succeeds, build passes
- **Committed in:** 0ab30ad (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 cleanup)
**Impact on plan:** Cleanup only - removed dead code. No scope creep.

## Issues Encountered
None - plan executed smoothly.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness

Frontend timer infrastructure complete:
- Plan 04-02 can implement backend session timer task that sends timer JSON messages
- Plan 04-04 can use status bar for Who's Online count updates
- Plan 04-06 can trigger timeout sequence via timeout JSON message

Status bar visible and ready to display time remaining once backend timer messages are implemented.

---
*Phase: 04-time-limits-user-lists*
*Completed: 2026-01-28*
