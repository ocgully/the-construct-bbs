---
phase: 03-navigation-system
plan: 03
subsystem: navigation
tags: [menu, session, navigation, wildcat, single-keypress, command-stacking]

# Dependency graph
requires:
  - phase: 03-02
    provides: MenuSession state machine, MenuAction enum, render functions for menus
  - phase: 02-07
    provides: Session lifecycle, AuthState, profile routing
provides:
  - Fully integrated config-driven menu navigation in session lifecycle
  - Single-keypress hotkey navigation (no Enter required)
  - Command stacking via type-ahead buffer (e.g., G1)
  - MenuSession created on authentication with user level mapping
  - Menu state resets to MainMenu when exiting services
affects: [04-chat-rooms, 05-mail-system, 06-games, any phase adding menu items]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - MenuSession lifecycle: created on authentication, persists during session
    - User level mapping: Sysop=255, User=0 for menu filtering
    - Single-keypress routing: process each char individually through MenuSession
    - Type-ahead buffer draining on submenu transitions for command stacking
    - Menu state reset to MainMenu on service exit

key-files:
  created: []
  modified:
    - backend/src/websocket/session.rs
    - backend/src/services/welcome_art.rs
    - backend/src/menu/mod.rs
    - backend/src/services/login.rs
    - backend/src/services/registration.rs
    - backend/src/menu/render.rs

key-decisions:
  - "MenuSession created immediately after authentication in all paths (login, registration, session resume)"
  - "User level string mapped to u8 via helper function: Sysop=255, User=0"
  - "handle_authenticated_input() processes each character individually for single-keypress navigation"
  - "Type-ahead buffer drained after EnterSubmenu for command stacking (e.g., G1)"
  - "Menu state reset to MainMenu via reset_to_main() when exiting services"
  - "Quit and Profile commands extracted into helper methods (handle_quit, handle_profile_view)"
  - "Legacy menu rendering functions removed from welcome_art.rs"

patterns-established:
  - "MenuSession field in Session struct: Option<MenuSession>, created on auth, None before auth"
  - "show_menu() replaces show_main_menu(): renders current menu state (MainMenu or Submenu)"
  - "Single-keypress navigation: loop over input chars, skip escape sequences, process_key() for each char"
  - "Command stacking: buffered keys drained after submenu transitions"
  - "Service exit cleanup: reset menu state, delay, show menu"

# Metrics
duration: 9min
completed: 2026-01-27
---

# Phase 3 Plan 3: Navigation Logic Integration Summary

**Config-driven Wildcat-style menu navigation fully integrated into session lifecycle with single-keypress hotkeys, command stacking, and automatic menu state management**

## Performance

- **Duration:** 9 min
- **Started:** 2026-01-27T06:37:01Z
- **Completed:** 2026-01-27T06:46:14Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Authenticated users navigate via MenuSession with single-keypress hotkeys (no Enter required for menu selection)
- Command stacking works via type-ahead buffer (e.g., G1 goes to Games > item 1)
- Main menu shows config-driven items with MOTD quotes and user info (handle, level, node)
- Submenus accessible via hotkey with [Q] Back to Main Menu
- Profile and Quit commands functional from main menu
- Service launch/exit correctly transitions menu state
- Legacy Phase 2 menu code removed, all rendering uses config-driven menu::render

## Task Commits

Each task was committed atomically:

1. **Task 1: Session integration with menu navigation system** - `fe3d4fb` (feat)
   - Added menu_session field to Session struct
   - Created MenuSession on authentication with user level mapping
   - Replaced show_main_menu() with show_menu() for config-driven menus
   - Rewrote handle_authenticated_input() for single-keypress navigation
   - Added process_typeahead() for command stacking
   - Extracted handle_quit() and handle_profile_view() helper methods
   - Fixed test configs to include menu field

2. **Task 2: Clean up legacy menu code and verify end-to-end** - `fb1ac5d` (refactor)
   - Removed render_main_menu_with_user() and render_main_menu() from welcome_art.rs
   - Removed unused welcome_art import from session.rs
   - Removed unused exports from menu/mod.rs
   - All 162 tests passing with no compilation errors

## Files Created/Modified
- `backend/src/websocket/session.rs` - Integrated MenuSession into session lifecycle, rewrote handle_authenticated_input() for single-keypress navigation
- `backend/src/services/login.rs` - Added menu field to test Config struct
- `backend/src/services/registration.rs` - Added menu field to test Config struct
- `backend/src/menu/render.rs` - Fixed test assertion for ANSI-encoded menu output
- `backend/src/services/welcome_art.rs` - Removed legacy menu rendering functions
- `backend/src/menu/mod.rs` - Removed unused exports

## Decisions Made
- **MenuSession lifecycle:** Created immediately on authentication (login, registration, resume) with user level mapping (Sysop=255, User=0). This ensures menu state is always ready when user reaches authenticated state.
- **Single-keypress navigation:** Each character in input processed individually through MenuSession.process_key(). Escape sequences and control chars (except Enter) are skipped. This matches Wildcat BBS behavior.
- **Command stacking:** After EnterSubmenu action, drain type-ahead buffer to process buffered keys against new submenu state. This enables G1 to go directly to Games > item 1.
- **Service exit cleanup:** When service returns Exit action, call reset_to_main() on MenuSession before showing menu. This prevents user from being stuck in a submenu after exiting a service.
- **Helper method extraction:** Extracted quit and profile logic into separate methods (handle_quit, handle_profile_view) to keep code DRY since they're called from multiple places (menu actions, type-ahead processing).

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Test failure for menu rendering:**
- **Issue:** Test expected literal `[P] Profile` but output contained ANSI escape codes: `[92m[P][97m Profile`
- **Resolution:** Updated test assertions to check for components separately (`[P]` and `Profile`) rather than exact string match. This properly tests that both hotkey and name are rendered without being brittle to ANSI formatting.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Navigation system complete and ready for service integration:**
- Main menu displays config-driven items from config.toml with proper level filtering
- Submenus accessible and navigable with single-keypress hotkeys
- Command stacking functional for rapid navigation
- Help system accessible via ? key at all menu levels
- Profile and Quit commands working from main menu
- Service launch and exit properly managed with menu state transitions

**Ready for Phase 4 (Chat Rooms) and beyond:**
- All new services can be added to config.toml menu sections
- Level-gating works via min_level field
- Menu system will automatically render new items in appropriate sections
- No menu-related code changes needed for new features

---
*Phase: 03-navigation-system*
*Completed: 2026-01-27*
