---
phase: 08
plan: 03
subsystem: game-mechanics
tags: [rust, state-machine, game-logic, flow-control]

requires:
  - 08-02  # Game data structures (GameState, static data)
  - 05-02  # ComposeFlow pattern reference

provides:
  - Game screen state machine (GtmFlow)
  - Screen transition logic
  - Action/day management system
  - Trade buy/sell mechanics
  - Random event system

affects:
  - 08-04  # Rendering will use GameScreen enum
  - 08-05  # Session integration will use GtmAction
  - 08-06+ # Future plans use GtmFlow as core game engine

tech-stack:
  added: []
  patterns:
    - Synchronous state machine returning actions (ComposeFlow pattern)
    - Weighted random event selection with WeightedIndex
    - Single-key vs buffered input handling

key-files:
  created:
    - backend/src/game/screen.rs  # GameScreen, GtmFlow, event system
  modified:
    - backend/src/game/mod.rs  # Export screen module

decisions:
  - slug: gtm-flow-synchronous-state-machine
    summary: GtmFlow returns GtmAction enum for session to handle
    context: Following ComposeFlow pattern from mail.rs
    rationale: Keeps state machine synchronous, session.rs handles async DB/rendering
    impact: Clean separation - GtmFlow owns game logic, session owns I/O

  - slug: single-key-vs-buffered-input
    summary: Most screens use single-key input, quantity entry uses buffered
    context: is_single_key_screen determines processing mode
    rationale: Menu-driven navigation feels immediate, quantity needs Enter
    impact: MainMenu/Travel/Trade menus process keys instantly, amounts buffer

  - slug: fifteen-percent-event-chance
    summary: 15% chance for random event after travel
    context: Called in handle_travel after moving to new borough
    rationale: Frequent enough for variety, rare enough to not annoy
    impact: Player encounters ~1 event every 7 borough moves on average

  - slug: weighted-event-selection
    summary: Events have different probabilities, adjusted by game state
    context: WeightedIndex samples from event list with weights
    rationale: Dynamic difficulty - high debt increases enforcer encounters
    impact: Game responds to player state (debt, gang relations, coat tier)

  - slug: use-action-triggers-advance-day
    summary: use_action decrements counter, calls advance_day when 0
    context: Travel and successful trades consume actions
    rationale: Centralized day advancement logic in one place
    impact: All game progression flows through use_action/advance_day

  - slug: day-advancement-applies-interest
    summary: advance_day applies debt (10%), bank (5%), notoriety decay
    context: Called when actions_remaining hits 0
    rationale: Daily interest compounds pressure, matches original game
    impact: Debt grows fast (doubles in ~7 days), bank grows slowly

metrics:
  duration: 4min
  completed: 2026-01-29

wave: 2
wave-progress: 1/4
---

# Phase 08 Plan 03: Game State Machine Summary

**One-liner:** Screen flow state machine with action management, trade mechanics, and weighted random events

## What Was Built

Created the GtmFlow state machine that manages game screen transitions and processes player actions.

### Core Components

**1. GameScreen Enum**
- 15 screen variants covering all game states
- Intro, MainMenu, Travel, Trade (with TradeMode), Combat, Event, LoanShark, Bank, Hospital, GunShop, Quest, Casino, GameOver, Leaderboard, ConfirmQuit
- Nested enums: TradeMode (Menu/Buying/Selling/BuyAmount/SellAmount), EnemyType, GameEvent, CasinoGame

**2. GtmAction Enum**
- Actions returned to session.rs for handling
- Continue, Render(String), Echo(String), SaveGame, GameOver, Quit
- Follows ComposeFlow pattern - state machine returns actions

**3. GtmFlow State Machine**
- Holds GameState, current GameScreen, prices HashMap, input_buffer
- handle_char processes single characters, returns GtmAction
- is_single_key_screen determines immediate vs buffered processing
- process_input routes to screen-specific handlers

**4. Screen Handlers**
- handle_intro: Any key advances to MainMenu
- handle_main_menu: T=Travel, B=Trade, L=LoanShark, K=Bank (if unlocked), H=Hospital, G=GunShop, C=Casino, Q=Quest, S=Leaderboard, X=Quit
- handle_travel: Borough selection or C for city selection
- handle_trade: Buy/sell flow with quantity input
- Placeholder handlers for Combat, Event, LoanShark, Bank, Hospital, GunShop, Quest, Casino

**5. Trade Mechanics**
- TradeMode state: Menu → Buying/Selling → BuyAmount/SellAmount
- Buy: Check capacity (coat_tier), check cash, deduct payment, add to inventory
- Sell: Check owned quantity, add cash, remove from inventory, calculate profit
- Updates stats.total_bought, stats.total_sold, stats.total_profit

**6. Travel System**
- Borough selection within city (taxi, instant)
- City selection (TODO: bus/plane costs and time)
- use_action called after successful move
- maybe_trigger_event called after borough travel

**7. Action/Day Management**
- use_action decrements actions_remaining, calls advance_day when 0
- advance_day: increment day, reset to 5 actions, apply interest, decay notoriety
- Debt interest: 10% daily ((debt * 11000) / 10000)
- Bank interest: 5% daily if unlocked ((balance * 10500) / 10000)
- Notoriety decay: 10% ((notoriety * 90) / 100)
- Game over check: day > 90
- Bank unlock: cash >= $50,000
- Max net worth tracking

**8. Random Event System**
- maybe_trigger_event: 15% base chance after travel
- WeightedIndex samples from event list
- Base events: FindCash (20 weight), Police (25), Mugger (20)
- Price events: PriceDrop (15), PriceSpike (10)
- TrenchcoatGuy (5) if coat_tier < 3
- LoanSharkEnforcer (10 + debt/500k) if debt > $10,000
- Gang encounters (15 + -relation/5) in hostile territory
- Dynamic weights adjust to game state

**9. Price Generation**
- generate_prices: Random base price within commodity min/max
- Regional modifiers: Tokyo meth 1.5x, London cocaine 0.7x, Bogota cocaine 0.5x
- Volatility: ±20% random variation
- Regenerated on travel (new location = new prices)

## Decisions Made

See frontmatter decisions section.

Key architectural choice: GtmFlow is synchronous, returns GtmAction for session.rs to handle async operations. This matches the ComposeFlow pattern and keeps game logic pure.

## Deviations from Plan

**Task consolidation:** Plan had two separate tasks, but Task 2 (random event triggering) was naturally implemented as part of Task 1 in the `maybe_trigger_event` method. The method was called from `handle_travel`, making it a cohesive single implementation.

No other deviations - plan executed exactly as written.

## Integration Points

**Depends on:**
- backend/src/game/state.rs: GameState struct and interest methods
- backend/src/game/data.rs: Static game data (CITIES, COMMODITIES, etc.)

**Provides to:**
- Future 08-04 (Rendering): GameScreen enum to determine what to display
- Future 08-05 (Session integration): GtmAction enum to handle state machine results
- Future 08-06+ (Feature plans): GtmFlow as core game engine

## Testing Notes

**Verified via cargo check:**
- All types compile cleanly
- No errors, only unused warnings (expected - not integrated yet)
- rand::distributions::WeightedIndex compiles correctly
- All screen handlers return GtmAction correctly

**Manual verification needed (08-05):**
- Input flow: character → handle_char → process_input → handler → action
- Trade math: capacity checks, cash deductions, inventory updates
- Day advancement: interest calculations, game over at day 90
- Random events: 15% trigger rate, weighted selection works

## Files Changed

**Created:**
- backend/src/game/screen.rs (695 lines)
  - GameScreen enum (15 variants)
  - TradeMode, EnemyType, GameEvent, CasinoGame enums
  - GtmAction enum (6 variants)
  - GtmFlow struct with state machine logic
  - All screen handlers
  - use_action / advance_day methods
  - maybe_trigger_event with weighted selection
  - generate_prices with regional modifiers

**Modified:**
- backend/src/game/mod.rs (3 lines)
  - Added `pub mod screen;`
  - Added `pub use screen::*;`

## Next Phase Readiness

**Ready for 08-04 (Rendering):**
- GameScreen enum provides all screen states
- GtmFlow.current_screen() exposes screen for rendering decisions

**Ready for 08-05 (Session integration):**
- GtmAction enum defines all possible results
- handle_char processes input incrementally
- SaveGame action signals when DB save needed

**Blockers:** None

**Concerns:** None - state machine is complete and follows established patterns

## Performance Notes

- HashMap lookups for prices: O(1)
- WeightedIndex sampling: O(n) setup, O(log n) sampling
- Event weight calculation: O(1) per event type
- All operations suitable for per-keystroke execution

**Duration:** 4 minutes (cargo check + commit)

## Technical Debt

None - implementation is complete and follows project patterns.

**TODO markers in code:**
- Combat resolution (handle_combat) - placeholder returns to menu
- Event resolution (handle_event) - placeholder returns to menu
- Loan shark payment/borrow flow - placeholder returns to menu
- Bank deposit/withdraw - placeholder returns to menu
- Gun shop purchases - placeholder returns to menu
- Quest system - placeholder returns to menu
- Casino games - placeholder returns to menu

These are intentionally incomplete - they're future phase work, not debt.

---

**Status:** Complete - Wave 2 plan 1/4 done
**Next:** 08-04-PLAN.md (Rendering functions for all screens)
