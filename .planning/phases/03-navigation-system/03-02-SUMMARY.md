---
phase: 03-navigation-system
plan: 02
subsystem: navigation
tags: [menu, state-machine, ansi, rendering, cp437, typeahead, wildcat]

# Dependency graph
requires:
  - phase: 03-01
    provides: MenuItem/MenuConfig schema, Stoic quotes collection, menu module foundation
  - phase: 01-02
    provides: AnsiWriter with CP437 box-drawing and CGA color support
  - phase: 02-07
    provides: User level context for menu filtering
provides:
  - MenuState enum tracking MainMenu vs Submenu navigation
  - MenuAction enum for all state transitions
  - TypeAheadBuffer for command stacking (G1 = Games > item 1)
  - MenuSession managing state + buffer + user level
  - ANSI rendering functions for main menu, submenus, and help screens
affects: [03-03-input-handling, door-games, all-service-modules]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "State machine pattern for menu navigation with explicit state transitions"
    - "Type-ahead buffering for command stacking (multi-key sequences)"
    - "Adaptive column layout (single vs two-column based on item count)"
    - "BorderStyle enum for consistent CP437 border rendering"

key-files:
  created:
    - backend/src/menu/state.rs
    - backend/src/menu/render.rs
  modified:
    - backend/src/menu/mod.rs

key-decisions:
  - "MenuState enum has two variants: MainMenu and Submenu with key"
  - "MenuAction returns from process_key to indicate transition result"
  - "TypeAheadBuffer uses VecDeque with 16-char max capacity"
  - "drain_buffer stops at LaunchService/ExecuteCommand to preserve remaining buffer"
  - "Main menu uses double-line CP437 borders, submenus use single-line"
  - "Adaptive layout: single column for <=7 items, two columns for >7"
  - "MOTD area shows random Stoic quote from embedded collection"
  - "Q key has dual behavior: BackToMain in submenu, ExecuteCommand(quit) at main"

patterns-established:
  - "State machine with explicit process_key returning MenuAction"
  - "Type-ahead buffer enabling command stacking for power users"
  - "BorderStyle enum for render function consistency"
  - "Context-sensitive help with state-based content"

# Metrics
duration: 3min
completed: 2026-01-27
---

# Phase 03 Plan 02: Menu State Machine & Rendering Summary

**MenuSession state machine with type-ahead buffering and authentic Wildcat-style ANSI rendering using CP437 double/single-line borders**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-27T01:55:25Z
- **Completed:** 2026-01-27T01:58:47Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- State machine handles MainMenu ↔ Submenu transitions with explicit MenuAction returns
- Type-ahead buffer enables command stacking (G1 navigates to Games > launches first game)
- Main menu rendering with double-line CP437 borders, MOTD quotes, adaptive column layout, and user info
- Submenu rendering with single-line borders and [Q] Back option
- Context-sensitive help screens for both menu states

## Task Commits

Each task was committed atomically:

1. **Task 1: Menu state machine and type-ahead buffer** - `3dc11af` (feat)
   - MenuState enum (MainMenu, Submenu)
   - MenuAction enum (Redraw, EnterSubmenu, BackToMain, LaunchService, ExecuteCommand, ShowHelp, Buffered, None)
   - TypeAheadBuffer with FIFO VecDeque (16-char capacity)
   - MenuSession with process_key, buffer_key, drain_buffer, reset_to_main
   - Fixed missing `pub mod quotes;` in mod.rs
   - Comprehensive tests for state transitions and buffer behavior

2. **Task 2: ANSI menu rendering** - `7c87060` (feat)
   - render_main_menu with double-line CP437 borders (C9/CD/BB/C8/BC/BA)
   - render_submenu with single-line CP437 borders (DA/C4/BF/C0/D9/B3)
   - render_help with state-based content (MainMenu vs Submenu)
   - BorderStyle enum for consistent border rendering
   - Adaptive layout: single column (≤7 items) vs two columns (>7 items)
   - Title centering, MOTD quote display, user info with node count
   - Tests verify output contains expected elements

## Files Created/Modified
- `backend/src/menu/state.rs` - MenuState enum, MenuAction enum, TypeAheadBuffer, MenuSession with process_key logic
- `backend/src/menu/render.rs` - render_main_menu, render_submenu, render_help with CP437 box-drawing and CGA colors
- `backend/src/menu/mod.rs` - Added quotes, state, render module exports

## Decisions Made

**Q key dual behavior:** At MainMenu, 'Q' matches command item returning ExecuteCommand("quit"). At Submenu, 'Q' always returns BackToMain before item matching. This gives Q precedence in submenus for consistent Back behavior.

**drain_buffer stops at terminal actions:** When processing buffered keys, drain_buffer stops immediately after LaunchService or ExecuteCommand to preserve remaining buffer for next menu. This prevents command stacking from over-consuming input.

**Adaptive column layout threshold:** Main menu switches to two-column layout when >7 items. This balances visual density with readability on 80-column screens.

**MOTD quote integration:** Main menu calls random_stoic_quote() directly in render function rather than passing as parameter. This keeps the quote random on each menu redraw.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing `pub mod quotes;` to mod.rs**
- **Found during:** Task 1 (Menu state machine)
- **Issue:** mod.rs was missing `pub mod quotes;` declaration, preventing render module from importing random_stoic_quote
- **Fix:** Added `pub mod quotes;` and `pub use quotes::random_stoic_quote;` to mod.rs
- **Files modified:** backend/src/menu/mod.rs
- **Verification:** Build succeeded, render module imports work
- **Committed in:** 3dc11af (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Essential fix to unblock Task 2. No scope creep.

## Issues Encountered
None - both tasks implemented as specified.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness

**Ready for 03-03 (Navigation Logic):**
- MenuSession provides complete state machine for input handling
- render_* functions available for screen output
- TypeAheadBuffer tested and ready for command stacking
- All menu items filtered by user_level before rendering

**Integration points:**
- Main menu shows user info (handle, level, node) - all data available from session
- State transitions return MenuAction for external handling
- reset_to_main() available for service exit flow

**No blockers.** Phase 3 Plan 3 can wire MenuSession into WebSocket session handling.

---
*Phase: 03-navigation-system*
*Completed: 2026-01-27*
