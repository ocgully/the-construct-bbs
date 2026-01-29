---
phase: 08-first-door-game-drug-wars
plan: 02
subsystem: game-engine
tags: [rust, serde, game-state, static-data, drug-wars]

# Dependency graph
requires:
  - phase: 08-01
    provides: GtmDb with separate database pool pattern
provides:
  - GameState struct with full game state representation
  - Static game data (cities, boroughs, commodities, weapons, gangs)
  - Helper methods for game calculations (net worth, interest, capacity)
  - Lookup functions for accessing static game data
affects: [08-03, 08-04, 08-05, 08-06, 08-07, 08-08, 08-09]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "GameState with serde serialization for JSON persistence"
    - "Static data with &'static lifetimes for zero-cost game world definition"
    - "Integer cents for currency to avoid floating point precision issues"
    - "Basis points arithmetic for interest calculations (11000/10000 = 1.10)"

key-files:
  created:
    - backend/src/game/mod.rs
    - backend/src/game/state.rs
    - backend/src/game/data.rs
  modified:
    - backend/src/main.rs

key-decisions:
  - "Currency stored as i64 cents to avoid float precision issues"
  - "Basis points arithmetic for interest (debt * 11000 / 10000 = 1.10x)"
  - "Static data with &'static lifetimes for zero-allocation game world"
  - "HashMap for inventory, gang_relations, addiction (dynamic keys)"

patterns-established:
  - "GameState::new() creates initial game state (Bronx, NYC, $2000 cash, $5500 debt)"
  - "Helper methods for derived state (coat_capacity, inventory_count, net_worth)"
  - "Lookup functions (get_city, get_commodity, etc.) for static data access"
  - "TravelMode enum for travel cost calculations (Bus/Plane)"

# Metrics
duration: 5min
completed: 2026-01-29
---

# Phase 08 Plan 02: Game Data Structures Summary

**GameState with serde serialization, 5 cities with 20 boroughs, 11 commodities, 12 weapons, 3 gangs, and helper methods for game calculations**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-29T05:29:11Z
- **Completed:** 2026-01-29T05:34:23Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Complete GameState struct with all game fields (day, actions, location, cash, debt, health, notoriety, inventory, weapons, gang_relations, quest_state, stats, addiction)
- Static game world data: 5 cities (NYC, Miami, London, Tokyo, Bogota) with 20 boroughs
- 11 commodities with price ranges and properties (addictive, action_boost)
- 12 weapons (4 melee, 8 guns) with damage and price data
- 3 gangs (Triads, Cartel, Mafia) with tribute costs
- Helper methods for game calculations (coat_capacity, inventory_count, net_worth, interest)
- Lookup functions for accessing static data by key

## Task Commits

Each task was committed atomically:

1. **Tasks 1-2: Create game module with GameState and static data** - `2ad62fb` (feat)

**Bug fix:** `0f5b3a8` (fix: test fixtures missing news field)

## Files Created/Modified
- `backend/src/game/mod.rs` - Module exports for state and data
- `backend/src/game/state.rs` - GameState struct with serde derives, helper methods
- `backend/src/game/data.rs` - Static game data (cities, commodities, weapons, gangs), lookup functions
- `backend/src/main.rs` - Added game module declaration

## Decisions Made

**Currency as cents:** All currency values stored as i64 in cents to avoid floating point precision issues. $2,000 = 200000 cents.

**Basis points arithmetic:** Interest calculations use integer arithmetic to avoid floats. 10% debt interest = (debt * 11000) / 10000. 5% bank interest = (balance * 10500) / 10000.

**Static data with 'static lifetimes:** Cities, commodities, weapons, gangs use &'static str and &'static slices for zero-allocation game world definition. Data lives in binary, no runtime allocation.

**HashMap for dynamic collections:** inventory (commodity -> quantity), gang_relations (gang -> relation), addiction (commodity -> level) use HashMap since keys vary per game state.

**Initial state:** GameState::new() starts in Bronx, NYC with $2,000 cash, $5,500 debt, 100 HP, 5 actions/day, coat tier 0 (100 capacity).

**Coat tiers:** 0=100 units, 1=125, 2=150, 3=250. Upgrade costs escalate ($500, $1000, $2500).

**Travel modes:** Intra-city taxi = $20 instant. Inter-city bus = $100 + 1 day. Plane = $500 instant.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test fixtures missing news field**
- **Found during:** Verification (cargo test)
- **Issue:** Test fixtures in login.rs and registration.rs were missing NewsConfig field added in Phase 7, causing test compilation failures
- **Fix:** Added `news: crate::config::NewsConfig::default()` to test_config() in both files
- **Files modified:** backend/src/services/login.rs, backend/src/services/registration.rs
- **Verification:** All 200 tests pass
- **Committed in:** 0f5b3a8 (separate bug fix commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Test fixture bug was pre-existing from Phase 7. Fix required for verification. No scope creep.

## Issues Encountered
None - implementation proceeded as planned.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness

**Ready for Phase 08-03 (GTM Database Layer):**
- GameState struct ready for JSON serialization to database
- Static data accessible via lookup functions for price generation
- Helper methods available for game logic calculations

**Ready for Phase 08-04+ (Game Logic):**
- Complete data structures for all game mechanics
- Interest calculations ready for daily processing
- Travel cost functions ready for location changes
- Gang relations HashMap ready for reputation system

**No blockers.** All game data structures and static content defined and verified.

---
*Phase: 08-first-door-game-drug-wars*
*Completed: 2026-01-29*
