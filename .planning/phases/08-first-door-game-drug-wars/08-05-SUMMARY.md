---
phase: 08-first-door-game-drug-wars
plan: 05
subsystem: game
tags: [events, combat, random-events, weighted-selection, rust]

# Dependency graph
requires:
  - phase: 08-02
    provides: GameState with health, notoriety, gang relations, weapons
  - phase: 08-03
    provides: GtmFlow screen handlers and travel logic
provides:
  - Random event system with weighted probability selection
  - Combat resolution with multiple action types (Fight, Run, Talk, Bribe)
  - Price event modifications to market prices
  - Find events (cash, drugs)
  - Trenchcoat upgrade encounter
affects: [08-06, 08-07, 08-08, 08-09]

# Tech tracking
tech-stack:
  added: [rand::distributions::WeightedIndex]
  patterns: [Event-based game mechanics, State-based weighted selection]

key-files:
  created: [backend/src/game/events.rs]
  modified: [backend/src/game/screen.rs, backend/src/game/mod.rs]

key-decisions:
  - "Event triggering uses 15% base chance on travel with dynamic weights based on game state"
  - "Police encounters scale with notoriety (weight = 15 + notoriety/5)"
  - "Loan shark enforcer only appears when debt > $10k, weight scales with debt amount"
  - "Gang encounters only in hostile territory, weight scales with negative relation"
  - "Combat resolution uses player weapon damage + 0-10 random vs enemy stats"
  - "Run action has 60% escape chance, takes half damage if caught"
  - "Talk action only works on police with 30% success rate"
  - "Bribe action only works on police, success rate based on amount ($100+=50%, $500+=80%, $1000+=95%)"
  - "Price spikes multiply prices by 200-400%, drops reduce to 50-80%"
  - "Trenchcoat upgrade requires dumping all inventory for +1 tier capacity"

patterns-established:
  - "Event system separated from screen logic for testability and reuse"
  - "Combat resolution returns CombatResult struct, apply_combat_result mutates state"
  - "Event effects applied via dedicated functions (apply_price_event, apply_find_event)"

# Metrics
duration: 5min
completed: 2026-01-29
---

# Phase 08 Plan 05: Event System Summary

**Random event system with weighted probability selection based on game state (notoriety, debt, gang relations)**

## Performance

- **Duration:** 5 min 24 sec
- **Started:** 2026-01-29T05:47:08Z
- **Completed:** 2026-01-29T05:52:32Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Comprehensive event system with 9 event types (FindCash, FindDrugs, PriceDrop, PriceSpike, Police, Mugger, LoanSharkEnforcer, Gang, TrenchcoatGuy)
- Combat resolution with 4 action types supporting different outcomes and state modifications
- Dynamic event weighting based on player state (notoriety increases police encounters, debt triggers enforcers, gang relations affect territory encounters)
- Price events that significantly modify market economics (spikes 200-400%, drops 50-80%)
- Integrated event triggering into travel flow for both borough and city changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Create event system with weighted selection** - `48b6c46` (feat)
2. **Task 2: Integrate events into GtmFlow screen handlers** - `13bd0ec` (feat)

## Files Created/Modified
- `backend/src/game/events.rs` - Event triggering, combat resolution, event application logic
- `backend/src/game/screen.rs` - Updated travel/combat/event handlers to use events module
- `backend/src/game/mod.rs` - Added events module export

## Decisions Made

**Event Weight Configuration:**
- FindCash: 20 (common positive event)
- FindDrugs: 10 (less common, limited by inventory space)
- PriceDrop: 15, PriceSpike: 10 (market events)
- Police: 15 + notoriety/5 (scales with heat)
- Mugger: 20 (common danger)
- LoanSharkEnforcer: 5 + debt/500000 when debt > $10k (debt-triggered)
- Gang: 10 + (-relation/5) in hostile territory (relation-triggered)
- TrenchcoatGuy: 5 (rare, only if coat_tier < 3)

**Combat Mechanics:**
- Player damage = weapon damage + 0-10 random
- Enemy damage varies by type (Police: 15, Mugger: 10, Gang: 20, Enforcer: 25)
- Fight: Direct confrontation, winner determined by damage rolls
- Run: 60% escape chance, takes enemy_damage/2 if caught
- Talk: Police only, 30% success, reduces notoriety by 5 on success
- Bribe: Police only, success rate by amount ($1+=20%, $100+=50%, $500+=80%, $1000+=95%)

**Notoriety Impact:**
- Killing police: +15 notoriety (very bad)
- Killing gang members: +5 notoriety
- Losing combat: -2 notoriety (lost respect)
- Running away: -1 to -3 notoriety
- Talking successfully: -5 notoriety
- Bribing successfully: -3 notoriety

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation proceeded smoothly with all tests passing.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Event system fully integrated into gameplay:
- Random events trigger after all travel (borough and city)
- Combat encounters support all 4 action types
- Price events modify market dynamics
- Find events add resources to player inventory
- Trenchcoat upgrades provide capacity progression

Ready for:
- Phase 08-06: Rendering event and combat screens with ANSI art
- Phase 08-07: Quest system integration with event-driven story progression
- Phase 08-08: Gun shop and weapon acquisition affecting combat outcomes

Note: Event messages and combat result display will be enhanced in rendering phase. Current implementation returns to main menu after events for state machine flow, rendering will add detailed event/combat screens.

---
*Phase: 08-first-door-game-drug-wars*
*Completed: 2026-01-29*
