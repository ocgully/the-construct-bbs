---
phase: 04-time-limits-user-lists
plan: 04
subsystem: services
tags: [ansi-art, cp437, bbs-ui, social-features, user-lists]

# Dependency graph
requires:
  - phase: 04-01
    provides: NodeManager extensions (current_activity, last_input), session_history table
  - phase: 01-terminal-foundation
    provides: AnsiWriter with CP437 box-drawing, CGA color palette
  - phase: 02-auth-connection
    provides: NodeManager with node tracking

provides:
  - render_whos_online function for active user display
  - render_last_callers function for recent session history display
  - CP437 ANSI table rendering for social features
  - Idle time calculation from last_input timestamps

affects: [04-05, 04-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Render functions for async data display (not Service trait impls)"
    - "80-column ANSI table layout with CP437 double-line borders"
    - "Duration formatting for user-friendly time display"

key-files:
  created:
    - backend/src/services/who.rs
    - backend/src/services/last_callers.rs
  modified:
    - backend/src/services/mod.rs

key-decisions:
  - "Who's Online and Last Callers use render functions (not Service trait) for async data access"
  - "80-column table width with careful border/data column calculation"
  - "Idle time formatted as seconds/minutes/hours for readability"
  - "Duration formatted as minutes or hours+minutes"

patterns-established:
  - "Render function pattern: render_* functions that take data and return ANSI string"
  - "Table layout: double-line CP437 borders, consistent color scheme (LightCyan borders, Yellow headers)"
  - "Empty state handling: centered message within table borders"

# Metrics
duration: 7min
completed: 2026-01-28
---

# Phase 4 Plan 04: Who's Online and Last Callers Summary

**CP437 ANSI table displays for Who's Online (node/handle/activity/idle) and Last Callers (handle/time/duration) with 80-column layout**

## Performance

- **Duration:** 7 min
- **Started:** 2026-01-28T00:43:53Z
- **Completed:** 2026-01-28T00:51:36Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Who's Online display with Node, Handle, Activity, Idle columns using full NodeInfo data
- Last Callers display with #, Handle, Date/Time, Duration columns from session_history
- Both tables use consistent CP437 double-line box-drawing borders
- Idle time calculation from last_input timestamps with smart formatting (seconds/minutes/hours)
- Session duration formatting (minutes or hours+minutes)
- Empty state handling for both displays

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Who's Online Service** - `9fe323d` (feat)
2. **Task 2: Create Last Callers Service** - `b91b417` (feat)

## Files Created/Modified
- `backend/src/services/who.rs` - render_whos_online function with NodeInfo display
- `backend/src/services/last_callers.rs` - render_last_callers function with SessionHistoryEntry display
- `backend/src/services/mod.rs` - Added who and last_callers module exports

## Decisions Made

**Render function pattern instead of Service trait:**
The plan correctly identified that these displays need async data access (NodeManager methods and database queries) which the synchronous Service trait methods cannot provide. Following the existing profile view pattern, these are implemented as standalone render functions that the session calls directly after fetching the data.

**Table width calculation:**
Both tables carefully calculated to exactly 80 columns:
- Who's Online: 6 + 20 + 30 + 19 = 75 data columns + 5 borders = 80
- Last Callers: 4 + 20 + 34 + 17 = 75 data columns + 5 borders = 80

**Consistent visual style:**
- Border color: LightCyan
- Header color: Yellow (bold)
- Label color: LightGray
- Value color: White (bold)
- CP437 double-line borders (U+2554, U+2550, U+2557, etc.)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation proceeded smoothly with clear plan guidance.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Display functions ready for integration:
- Plan 04-05 can wire these into menu commands for user access
- Plan 04-06 can use Who's Online display for multi-user features
- Both functions ready to receive data from NodeManager and session_history queries

The render functions are complete and tested (cargo check passes). They await integration into the menu system and session routing.

---
*Phase: 04-time-limits-user-lists*
*Completed: 2026-01-28*
