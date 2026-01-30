---
phase: 08-first-door-game-drug-wars
plan: 10
subsystem: games
tags: [architecture, refactoring, multi-game]

dependency-graph:
  requires: ["08-09"]
  provides: ["multi-game-architecture", "games-registry"]
  affects: ["future-door-games"]

tech-stack:
  patterns: ["module-registry", "self-contained-games"]

key-files:
  created:
    - backend/src/games/mod.rs
    - backend/src/games/grand_theft_meth/mod.rs
  modified:
    - backend/src/main.rs
    - backend/src/websocket/session.rs
    - backend/src/services/grand_theft_meth/service.rs
    - CLAUDE.md
  moved:
    - backend/src/game/*.rs -> backend/src/games/grand_theft_meth/*.rs

decisions:
  - id: "08-10-01"
    decision: "Multi-game architecture with games/ registry"
    rationale: "Scalable structure for adding LORD, Usurper, Acrophobia, Kyrandia"

metrics:
  duration: "8min"
  completed: "2026-01-30"
---

# Phase 08 Plan 10: Game Code Restructure Summary

**One-liner:** Restructured game code from /game/ to /games/grand_theft_meth/ for multi-game architecture

## What Was Built

Reorganized the Grand Theft Meth game module from a flat `backend/src/game/` structure to a scalable multi-game architecture under `backend/src/games/grand_theft_meth/`.

### Tasks Completed

| # | Task | Commit |
|---|------|--------|
| 1 | Create new games folder structure | 3093421 |
| 2 | Move game files to new location | 3093421 |
| 3 | Update internal module references | 3093421 |
| 4 | Update external references | 3093421 |
| 5 | Remove old game folder | 3093421 |
| 6 | Update CLAUDE.md with game pattern | 3093421 |
| 7 | Verify compilation and tests | 3093421 |

### Key Changes

**New folder structure:**
```
backend/src/games/
├── mod.rs                    # Game registry
└── grand_theft_meth/         # Self-contained game
    ├── mod.rs                # Public exports
    ├── data.rs               # Static game data
    ├── state.rs              # GameState
    ├── screen.rs             # GtmFlow state machine
    ├── render.rs             # ANSI rendering
    ├── economy.rs            # Bank, loans, casino
    ├── events.rs             # Random events, combat
    └── quest.rs              # Story/quests
```

**Import updates:**
- `crate::game::` -> `crate::games::grand_theft_meth::`
- Internal imports use `super::` for sibling modules

**Files updated:**
- `main.rs`: `mod game;` -> `mod games;`
- `session.rs`: 3 import references updated
- `service.rs`: 2 import references updated
- `CLAUDE.md`: Documented door game architecture pattern

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- [x] All files moved to new location
- [x] Old backend/src/game/ folder deleted
- [x] All imports updated (no `crate::game::` references remain)
- [x] `cargo check` passes
- [x] All 229 tests pass
- [x] CLAUDE.md documents the pattern

## Next Phase Readiness

Multi-game architecture ready for future door games. To add a new game:
1. Create `games/{game_name}/` folder with same structure
2. Register in `games/mod.rs`: `pub mod {game_name};`
3. Create service in `services/{game_name}/`
4. Add menu entry in config.toml
