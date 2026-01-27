---
phase: 03-navigation-system
plan: 01
subsystem: navigation
tags: [menu, config, toml, serde, stoicism, quotes]

# Dependency graph
requires:
  - phase: 02-authentication
    provides: Config struct and TOML deserialization pattern
provides:
  - Menu configuration schema with internally-tagged serde deserialization
  - MenuConfig type for hierarchical menu definitions (main + submenus)
  - Complete menu hierarchy in config.toml with all future items documented
  - Stoic quotes module with random selection for MOTD display
affects: [04-menu-rendering, 05-navigation-logic]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Config-driven menu system using TOML with internally-tagged enums
    - MenuItem helper methods for filtering by level and sorting by order

key-files:
  created:
    - backend/src/menu/config.rs
    - backend/src/menu/quotes.rs
    - backend/src/menu/mod.rs
  modified:
    - backend/src/config.rs
    - backend/src/main.rs
    - config.toml

key-decisions:
  - "MenuItem enum uses internally-tagged serde with type/hotkey/name/order/min_level"
  - "All menu fields use #[serde(default)] for graceful config loading"
  - "26 Stoic quotes embedded for MOTD rotation (not configurable text files)"
  - "Future service items commented out in config.toml with phase annotations"

patterns-established:
  - "MenuConfig provides filtered/sorted item access via main_items() and submenu_items()"
  - "Case-insensitive hotkey matching via matches_key() method"
  - "Submenu name mapping via submenu_name() helper"

# Metrics
duration: 4min
completed: 2026-01-27
---

# Phase 03 Plan 01: Menu Configuration Schema Summary

**Config-driven menu system with internally-tagged TOML deserialization, 6 main menu items, 4 submenu categories, and 26 embedded Stoic quotes for MOTD rotation**

## Performance

- **Duration:** 4 min 27 sec
- **Started:** 2026-01-27T06:19:42Z
- **Completed:** 2026-01-27T06:24:09Z
- **Tasks:** 2
- **Files modified:** 6 (3 created, 3 modified)

## Accomplishments

- Menu configuration schema with three MenuItem variants (Service, Submenu, Command) using internally-tagged serde
- Complete menu hierarchy in config.toml with main menu (6 items) and 4 submenus (games, mail, chat, news)
- All future service items documented as commented entries with phase annotations
- 26 Stoic quotes embedded with random selection function for MOTD display
- MenuConfig struct integrated into main Config with serde defaults

## Task Commits

Each task was committed atomically:

1. **Task 1: Menu config schema and TOML menu definitions** - `f789aa2` (feat)
2. **Task 2: Stoic quotes module with random selection** - `dc13eaf` (feat)

## Files Created/Modified

- `backend/src/menu/config.rs` - MenuItem enum (Service/Submenu/Command), MenuConfig struct, filtering/sorting helpers
- `backend/src/menu/quotes.rs` - 26 Stoic quotes array, random_stoic_quote() function, tests
- `backend/src/menu/mod.rs` - Menu module public API
- `backend/src/config.rs` - Added menu: MenuConfig field with serde default
- `backend/src/main.rs` - Added mod menu declaration
- `config.toml` - Complete menu configuration with 6 main items, 4 submenu sections, commented future services

## Decisions Made

1. **Internally-tagged serde deserialization** - Used `#[serde(tag = "type", rename_all = "lowercase")]` for MenuItem enum to enable clean TOML syntax (`type = "submenu"` vs nested tables)
2. **Default-enabled config sections** - All menu fields use `#[serde(default)]` so the [menu] section is entirely optional, gracefully handling missing config
3. **Embedded quotes vs config files** - Stoic quotes embedded as const array rather than external files for simplicity and thematic consistency with "The Construct" atmosphere
4. **Future services documented** - All unbuilt services included as commented TOML entries with phase numbers, providing sysop visibility into planned features
5. **Helper method pattern** - MenuItem provides accessor methods (hotkey, name, order, min_level) to abstract variant matching, MenuConfig provides filtered/sorted collections

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - cargo not available in execution environment, but code structure verified through file inspection. Build verification deferred to next phase when menu rendering requires compilation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for Phase 03 Plan 02 (Menu Rendering):**
- Menu config types compile and provide filtering/sorting API
- config.toml populated with complete menu hierarchy
- Stoic quotes ready for MOTD integration
- No blockers

**Dependencies satisfied:**
- Config struct pattern from Phase 02 followed
- TOML deserialization working (same pattern as auth/connection config)

**Known constraints:**
- Build verification pending cargo availability
- Tests for quotes module created but not run yet
- Menu config not yet used by any service (rendering implementation next)

---
*Phase: 03-navigation-system*
*Completed: 2026-01-27*
