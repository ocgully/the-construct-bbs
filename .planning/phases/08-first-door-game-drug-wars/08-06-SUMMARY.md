---
phase: 08-first-door-game-drug-wars
plan: 06
subsystem: game-economy
tags: [rust, game-economy, loan-shark, bank, casino, blackjack, roulette, gambling]

# Dependency graph
requires:
  - phase: 08-02
    provides: GameState with currency fields (cash, debt, bank_balance)
  - phase: 08-04
    provides: format_money helper for currency formatting
provides:
  - Loan shark functions (borrow_money, pay_debt, pay_all_debt)
  - Bank functions (deposit, withdraw, deposit_all, withdraw_all)
  - Casino games (blackjack with 3:2 payout, roulette with 35:1 numbers, horse betting)
  - Check bank unlock at $50,000 threshold
affects: [08-07, 08-08, 08-09]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Result<T, String> for economy operations with error messages
    - Enum variants with data (BlackjackResult, RouletteBet) for game outcomes

key-files:
  created:
    - backend/src/game/economy.rs
  modified:
    - backend/src/game/mod.rs
    - backend/src/game/render.rs
    - backend/src/game/screen.rs

key-decisions:
  - "Loan shark max borrow is 2x current debt"
  - "Bank unlocks at $50,000 cash (check_bank_unlock function)"
  - "Casino games bet 10% of cash with $1.00 minimum"
  - "Blackjack: simplified 2-card comparison with 3:2 natural blackjack payout"
  - "Roulette: 35:1 for number bets, 1:1 for color/odd/even bets"
  - "Horse betting: 6 horses with odds from 2x to 8x, win chances from 40% to 10%"
  - "format_money exported as public for economy module use"

patterns-established:
  - "Economy functions modify GameState directly and return Result<T, String>"
  - "Screen handlers use economy functions and return GtmAction::SaveGame on success"
  - "Casino uses single-key menu for game selection, auto-bet 10% of cash"

# Metrics
duration: 5min
completed: 2026-01-29
---

# Phase 08 Plan 06: Economy Systems Summary

**Complete loan shark, bank, and casino implementation with borrowing limits, bank unlock threshold, and three casino games (blackjack, roulette, horses)**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-29T05:46:59Z
- **Completed:** 2026-01-29T05:51:45Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Loan shark borrowing up to 2x current debt with full payment option
- Bank deposit/withdrawal with $50,000 unlock threshold
- Casino with blackjack (3:2 natural payout), roulette (35:1 numbers), and horse betting (6 horses)
- All economy operations use basis point arithmetic for precise calculations
- Screen handlers integrated with economy functions for player actions

## Task Commits

Each task was committed atomically:

1. **Task 1: Create economy module with loan shark and bank** - `c98b232` (feat)
2. **Task 2: Wire economy into GtmFlow screen handlers** - `0d7d0af` (feat)

## Files Created/Modified
- `backend/src/game/economy.rs` - Loan shark, bank, and casino game functions
- `backend/src/game/mod.rs` - Export economy module
- `backend/src/game/render.rs` - Made format_money public for economy use
- `backend/src/game/screen.rs` - Integrated economy into GtmFlow handlers

## Decisions Made

**Loan shark mechanics:**
- Max borrow is 2x current debt (prevents runaway borrowing)
- Quick pay: P key pays all debt, B key borrows 50% of current debt
- 10% daily interest already applied in advance_day() from 08-03

**Bank mechanics:**
- Unlocks at $50,000 cash threshold (check_bank_unlock function)
- Quick deposit/withdraw: D deposits all cash, W withdraws all from bank
- 5% daily interest already applied in advance_day() from 08-03

**Casino mechanics:**
- Menu navigation: 1/B=blackjack, 2/R=roulette, 3/H=horses
- Auto-bet 10% of cash with $1.00 minimum for simplicity
- Blackjack: 3:2 payout for natural 21, simplified 2-card comparison
- Roulette: 35:1 for number bets, 1:1 for color/odd/even, 0 is green
- Horses: 6 horses with odds 2x-8x, win chances 40%-10% (risk/reward)

**Implementation patterns:**
- All economy functions return Result<T, String> with error messages
- Screen handlers save game state on successful transactions
- format_money made public for reuse across game modules

## Deviations from Plan

**1. [Rule 3 - Blocking] Made format_money public in render.rs**
- **Found during:** Task 1 (economy module compilation)
- **Issue:** format_money was private, economy.rs needed it for error messages
- **Fix:** Changed `fn format_money` to `pub fn format_money` in render.rs
- **Files modified:** backend/src/game/render.rs
- **Verification:** Cargo check passes, economy module compiles
- **Committed in:** c98b232 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (blocking)
**Impact on plan:** Essential visibility fix for code reuse. No scope creep.

## Issues Encountered

None - plan executed smoothly. All economy functions compiled and integrated correctly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Economy systems complete and ready for integration:
- Players can borrow from loan shark (creates gameplay tension)
- Bank provides safe storage with interest (rewards success)
- Casino offers risk/reward gambling (money sink/source)
- All functions properly validate amounts and state
- Screen handlers trigger game saves on transactions

Ready for Phase 08-07 (screen rendering for economy displays) and 08-08 (final integration).

---
*Phase: 08-first-door-game-drug-wars*
*Completed: 2026-01-29*
