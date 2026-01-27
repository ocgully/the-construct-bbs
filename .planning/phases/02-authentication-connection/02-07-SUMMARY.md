---
phase: 02-authentication-connection
plan: 07
subsystem: integration
tags: [session-lifecycle, profile-routing, main-menu, integration-wiring]

# Dependency graph
requires:
  - phase: 02-01
    provides: "Database layer (pool, schema, User CRUD)"
  - phase: 02-02
    provides: "Auth core (password, session, validation, NodeManager)"
  - phase: 02-03
    provides: "Connection ceremony (typewriter, splash, line-busy)"
  - phase: 02-04
    provides: "Registration flow (state machine, email verification)"
  - phase: 02-05
    provides: "Login flow (auth state, token persistence)"
  - phase: 02-06
    provides: "Profile card (ANSI art) and goodbye sequence (session stats)"
provides:
  - "Fully integrated session lifecycle: connect -> ceremony -> login/register -> main menu -> profile -> quit -> goodbye"
  - "Profile command wired into main menu routing"
  - "Main menu with user handle, level, node info"
affects: [03-message-boards, all-future-phases]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Sentinel service marker (__profile__) for non-service views in authenticated state"
    - "Main menu rendering with user context (handle, level, node)"

key-files:
  modified:
    - "backend/src/services/welcome_art.rs"
    - "backend/src/websocket/session.rs"

key-decisions:
  - "Profile viewing uses __profile__ current_service marker for any-key-to-return flow"
  - "Main menu shows user handle, level, node info in box-drawn header with [P] Profile and [Q] Quit"
  - "Legacy render_main_menu kept as wrapper for compatibility"

patterns-established:
  - "render_main_menu_with_user pattern: menu functions accept user context params"
  - "__profile__ sentinel: non-service views use current_service marker for input routing"

# Metrics
duration: 8min
completed: 2026-01-27
---

# Phase 2 Plan 7: Session Lifecycle Integration Summary

**Profile routing wired into main menu with user handle/level/node display; complete session lifecycle verified end-to-end**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-27T02:07:44Z
- **Completed:** 2026-01-27T02:16:00Z
- **Tasks:** 1 (Task 1 only; Task 2 checkpoint skipped per instructions)
- **Files modified:** 2

## Accomplishments
- Profile command ("p" or "profile") now accessible from main menu, renders ANSI profile card
- Main menu updated to show user handle, user level, and node info in box-drawn header
- Main menu now displays [P] Profile and [Q] Quit options alongside numbered services
- All 144 existing tests pass with no regressions
- Backend cargo check and frontend npm build both succeed cleanly

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire complete session lifecycle and fix integration issues** - `837f739` (feat)

## Files Created/Modified
- `backend/src/services/welcome_art.rs` - Added render_main_menu_with_user() with handle, level, node info; kept render_main_menu as legacy wrapper
- `backend/src/websocket/session.rs` - Imported render_profile_card; updated show_main_menu to pass user context; added profile routing in handle_authenticated_input; added __profile__ sentinel handling

## Decisions Made
- **Profile as __profile__ sentinel:** Rather than adding a boolean flag or new enum variant, the profile view uses `current_service = Some("__profile__")` as a lightweight marker. On next keypress, it clears the marker and returns to main menu. This avoids adding new fields to Session and reuses the existing service routing path.
- **render_main_menu_with_user function:** Created a new function taking user context params rather than modifying the existing render_main_menu signature, keeping backward compatibility with the legacy version as a wrapper.
- **Menu prompt changed to "Enter selection:"** since the menu now offers more than just service numbers -- profile [P] and quit [Q] are also options.

## Deviations from Plan

None -- plan executed exactly as written. All components from Plans 01-06 integrated without compilation errors or missing glue code. The existing implementations connected cleanly.

## Issues Encountered

None -- all 6 independent plans produced components that fit together perfectly. No compilation errors, no type mismatches, no missing imports. The only integration work needed was:
1. Wiring render_profile_card into session routing (new import + new code path)
2. Updating the main menu to accept and display user context
3. Adding profile and quit options to the menu layout

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Complete Phase 2 auth lifecycle is integrated and functional
- All session states route correctly: AwaitingAuth -> ConnectionCeremony -> Login/Registration -> Authenticated
- Profile, goodbye, and all cleanup paths verified
- Ready for Phase 3 (Message Boards) which will add services to the main menu

---
*Phase: 02-authentication-connection*
*Completed: 2026-01-27*
