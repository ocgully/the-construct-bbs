---
phase: 08-first-door-game-drug-wars
plan: 01
subsystem: door-games
tags: [sqlite, sqlx, door-game, leaderboard]

# Dependency graph
requires:
  - phase: 02-authentication-connection
    provides: SqlitePool pattern and database conventions
provides:
  - Self-contained game database pattern for door games
  - GtmDb struct with save/load and leaderboard operations
  - Game state persistence independent of BBS core
affects: [08-02, 08-03, 08-04, 08-05, 08-06, 08-07, 08-08, 08-09, future-door-games]

# Tech tracking
tech-stack:
  added: []
  patterns: [self-contained-game-database, separate-sqlite-pool-per-game]

key-files:
  created:
    - backend/src/services/grand_theft_meth/db.rs
    - backend/src/services/grand_theft_meth/mod.rs
  modified:
    - backend/src/services/mod.rs

key-decisions:
  - "Door games use self-contained databases (not BBS db)"
  - "GtmDb struct encapsulates game's database"

patterns-established:
  - "Self-contained game database: Each door game owns its SqlitePool to dedicated .db file"
  - "Game module pattern: services/game_name/db.rs with own schema and CRUD operations"
  - "Zero BBS coupling: Games can be added/removed without touching core database"

# Metrics
duration: 3min
completed: 2026-01-29
---

# Phase 08 Plan 01: Game Database Foundation Summary

**Self-contained GtmDb struct with dedicated SqlitePool for grand_theft_meth.db - saves, completions, and leaderboard with zero BBS core coupling**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-29T05:29:19Z
- **Completed:** 2026-01-29T05:32:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- GtmDb struct owns dedicated SqlitePool for grand_theft_meth.db file
- Schema with saves table (JSON state per user) and completions table (finished game records)
- CRUD operations: save_game, load_game, delete_save, has_save
- Leaderboard query with RANK() window function for top scores
- Established pattern for all future door games (pluggable, self-contained)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create game module directory and database module** - `e7aa351` (feat)
2. **Task 2: Export game module from services** - `a2e8f14` (feat)

## Files Created/Modified
- `backend/src/services/grand_theft_meth/db.rs` - GtmDb struct with SqlitePool, schema initialization, save/load/leaderboard operations
- `backend/src/services/grand_theft_meth/mod.rs` - Module exports for GtmDb
- `backend/src/services/mod.rs` - Added grand_theft_meth module export

## Decisions Made

**1. Door games use self-contained databases (not BBS db)**
- Rationale: Each door game is a pluggable module - no coupling to BBS core database. Games can be added/removed without schema migrations or touching bbs.db. Establishes pattern for all future games.

**2. GtmDb struct encapsulates game's database**
- Rationale: Game module owns its SqlitePool, schema, CRUD operations. Initialized on first launch (not at BBS startup). Data stored in data/grand_theft_meth.db alongside bbs.db.

**3. EST timezone offset (-5 hours) in datetime() calls**
- Rationale: Matches existing BBS convention from Phase 4 session history - all datetime fields maintain EST timezone consistency.

**4. Leaderboard uses RANK() window function**
- Rationale: Proper ranking with ties (vs ROW_NUMBER which would arbitrarily order ties). SQLite supports window functions since 3.25.0 (released 2018).

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation straightforward following existing SqlitePool pattern from backend/src/db/pool.rs.

## User Setup Required

None - no external service configuration required. Database file created automatically on first game launch.

## Next Phase Readiness

Database foundation complete. Ready for:
- Phase 08-02: Game state and core logic implementation
- Phase 08-03: Menu integration and session routing
- All game data will persist to grand_theft_meth.db via GtmDb operations

---
*Phase: 08-first-door-game-drug-wars*
*Completed: 2026-01-29*
