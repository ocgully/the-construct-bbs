---
phase: 08-first-door-game-drug-wars
plan: 04
subsystem: game-ui
tags: [rust, rendering, ansi-art, ui, door-game]

dependency_graph:
  requires:
    - 08-02  # Game data structures for state and data lookups
  provides:
    - ANSI rendering layer for all game screens
    - Status bar component with game state display
    - Atmospheric ANSI art styling
  affects:
    - 08-05  # Screen integration will wire these renders to GtmFlow
    - 08-06+ # All future game logic will use these render functions

tech_stack:
  added:
    - AnsiWriter pattern for game screen rendering
  patterns:
    - "Render functions return String (async-compatible)"
    - "format_money helper for currency display with thousand separators"
    - "Status bar as reusable component across all gameplay screens"

key_files:
  created:
    - backend/src/game/render.rs
  modified:
    - backend/src/game/mod.rs

decisions:
  - decision: "Reuse existing enums from screen.rs instead of duplicating"
    rationale: "TradeMode, EnemyType, GameEvent already defined in screen module"
    phase_plan: "08-04"
  - decision: "format_money with thousand separators and cent precision"
    rationale: "Atmospheric presentation: $12,345.67 more readable than 1234567 cents"
    phase_plan: "08-04"
  - decision: "Status bar shows 8 metrics in 2 lines with 80-column layout"
    rationale: "Day, cash, debt, health, location, coat capacity, net worth, actions - everything player needs at a glance"
    phase_plan: "08-04"
  - decision: "Health uses color coding: green >70, yellow >30, red <30"
    rationale: "Visual danger indication without reading the number"
    phase_plan: "08-04"
  - decision: "Travel screen shows borough features in DarkGray annotations"
    rationale: "Hospital, Guns, Mob Doc, Gang territory visible before committing travel action"
    phase_plan: "08-04"
  - decision: "Trade screen filters sell list to owned commodities only"
    rationale: "Prevents clutter - only show what player can actually sell"
    phase_plan: "08-04"
  - decision: "Combat screen shows weapon damage and bare fists fallback"
    rationale: "Player knows their attack power before choosing fight/run"
    phase_plan: "08-04"
  - decision: "Mob doctor costs more ($150 vs $100) but no notoriety risk"
    rationale: "Risk/reward tradeoff for high-heat players"
    phase_plan: "08-04"
  - decision: "Gun shop shows melee and firearms in separate sections"
    rationale: "Clear weapon categories matching game mechanics (2 slots)"
    phase_plan: "08-04"
  - decision: "Leaderboard uses rank-based colors: gold/white/brown/gray"
    rationale: "Olympic medal styling for top 3 players"
    phase_plan: "08-04"

metrics:
  duration: 5min
  completed: 2026-01-29
---

# Phase 08 Plan 04: ANSI Rendering Layer Summary

**One-liner:** Complete ANSI rendering system for all Grand Theft Meth game screens with atmospheric styling and status bar component

## What Was Built

Created `backend/src/game/render.rs` with 14 public render functions covering every game screen:

**Core Screens:**
- `render_intro()` - Story intro with "press any key" prompt
- `render_main_menu()` - Dynamic hub menu with location-based options
- `render_status_bar()` - Reusable 2-line status component (day, cash, debt, HP, location, coat, net worth, actions)
- `render_travel()` - Borough/city selection with feature annotations
- `render_trade()` - Buy/sell with price list and inventory display
- `render_confirm_quit()` - Save confirmation dialog

**Encounter Screens:**
- `render_combat()` - Enemy encounters with fight/run/bribe options
- `render_event()` - Random events (price spikes, finds, trenchcoat guy)
- `render_game_over()` - Win/lose ASCII art with final stats

**Location Screens:**
- `render_loan_shark()` - Debt management with atmospheric dialogue
- `render_bank()` - Deposit/withdraw with interest rate display
- `render_hospital()` - Healing menu (regular hospital or mob doctor variant)
- `render_gun_shop()` - Weapon purchase list (melee + firearms sections)

**Meta Screens:**
- `render_leaderboard_screen()` - Top scores hall of fame with rank colors

**Helper Functions:**
- `format_money()` - Converts cents to "$1,234.56" format with thousand separators
- `render_header()` - "GRAND THEFT METH" ASCII art title (red/yellow)

## Technical Decisions

### Render Pattern Consistency
All render functions follow the established BBS pattern:
```rust
pub fn render_*(...) -> String {
    let mut w = AnsiWriter::new();
    // ... ANSI writing ...
    w.flush()
}
```

This enables async session integration without blocking on terminal I/O.

### Status Bar as Component
The status bar is called by other render functions (not standalone). This ensures every gameplay screen shows consistent game state information.

### Color Semantics
- **LightRed**: Danger (combat, debt, loan shark)
- **LightGreen**: Positive (cash, health >70, bank)
- **Yellow**: Warnings (health 30-70, events, questions)
- **LightCyan**: Interactive prompts and hotkeys
- **White/LightGray**: Neutral content
- **DarkGray**: Borders, separators, annotations

### Dynamic Menu Generation
Main menu options appear/disappear based on:
- Bank unlocked: Show [K] Bank option
- Borough features: Show [H] Hospital/Mob Doc, [G] Gun Shop if available
- City features: Show [C] Casino if available

Travel screen annotations show features before committing action:
```
[2] Brooklyn [Mafia, Mob Doc]
[3] Manhattan [Hospital, Guns]
```

## Deviations from Plan

None - plan executed exactly as written. All 14 screen render functions implemented as specified.

## Verification Results

✅ `cargo check` passes without errors
✅ All render functions return String (async-compatible)
✅ Status bar displays all 8 required metrics
✅ Trade screen shows prices and inventory
✅ Combat screen shows enemy type and weapon info
✅ ANSI art title uses red/yellow CGA styling
✅ Thousand separator formatting: $12,345.67
✅ Health color-coding: green/yellow/red based on HP

## Integration Points

**Upstream (requires):**
- `GameState` from state.rs for game state access
- `TradeMode`, `EnemyType`, `GameEvent` enums from screen.rs
- `CITIES`, `COMMODITIES`, `WEAPONS` static data from data.rs
- Lookup functions: `get_city()`, `get_borough()`, `get_commodity()`, `get_weapon()`
- `AnsiWriter`, `Color` from terminal module

**Downstream (provides to):**
- Plan 08-05 (Screen Integration): `GtmFlow` will call these render functions based on `GameScreen` enum
- Plan 08-06+: All game logic that needs screen output

## Files Changed

```
backend/src/game/render.rs       | 1014 lines (new)
backend/src/game/mod.rs          |    2 lines (export render module)
```

## Next Phase Readiness

**Ready for 08-05 (Screen Integration):**
- ✅ All render functions exported from `game::render::*`
- ✅ Function signatures match expected `GtmFlow` integration pattern
- ✅ Status bar works as component (called by other renders)
- ✅ No blocking I/O (all functions return String)

**Blockers:** None

**Concerns:** None

## Implementation Notes

### Trade Screen Filtering
Sell mode only shows commodities the player owns (qty > 0). This prevents cluttered screens when player has empty inventory.

### Combat Options
Police encounters show additional options:
- [T] Try to talk your way out
- [B] Bribe the officer

Other enemy types only show [F] Fight and [R] Run.

### Hospital Cost Logic
```rust
let heal_cost = if is_mob_doctor { 15000 } else { 10000 };
```
Mob doctors charge 50% more ($150 vs $100) but don't report you to authorities.

### Leaderboard Rank Colors
```rust
let rank_color = match rank {
    1 => Color::Yellow,     // Gold
    2 => Color::White,      // Silver
    3 => Color::Brown,      // Bronze
    _ => Color::LightGray,  // Regular
};
```

### Currency Display Examples
- `200000` cents → `$2,000.00`
- `1234567` cents → `$12,345.67`
- `-550000` cents → `-$5,500.00`

## Key Learnings

1. **Component-based rendering**: Status bar as reusable component prevents duplication
2. **Dynamic menu generation**: Options appear/disappear based on game state and location
3. **Color coding**: Consistent semantics make UI self-documenting
4. **Annotation over verbosity**: Feature lists in DarkGray more effective than separate help screens
5. **Thousand separators matter**: $12,345 significantly more readable than $12345 in fast-paced gameplay

---

**Phase 8 Progress:** 3/9 plans complete (33%)
**Next Plan:** 08-05 - Screen Integration (wire renders to GtmFlow state machine)
