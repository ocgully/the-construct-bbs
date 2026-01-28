---
phase: 04-time-limits-user-lists
plan: 06
type: execute
subsystem: session-lifecycle
tags: [timer, session-history, time-banking, who-online, last-callers, user-lookup, timeout]
status: complete
started: 2026-01-28T00:58:27Z
completed: 2026-01-28T01:05:18Z
duration: 7min
commits:
  - 8a6e86e
dependency_graph:
  requires: ["04-01", "04-02", "04-03", "04-04", "04-05"]
  provides: ["Full Phase 4 integration: timer lifecycle, command routing, session history, timeout handling"]
  affects: ["05-*"]
tech_stack:
  patterns:
    - "Sentinel service names for state-based input routing (__whos_online__, __user_lookup__, etc.)"
    - "Timer expired/low_time polling via Arc<AtomicBool> flags on each input"
    - "Session history insert on login, update on logout/disconnect"
    - "Daily reset check with auto-banking on authentication"
key_files:
  modified:
    - backend/src/websocket/session.rs
decisions:
  - "Timer start called in all three authentication paths (login, resume, registration) for consistency"
  - "Withdrawal offered once per session via withdrawal_offered flag; resets if bank withdrawal succeeds"
  - "User lookup uses sentinel service states (__user_lookup__, __user_lookup_view__, __user_lookup_retry__)"
  - "NodeManager activity updated on service enter, service exit (Main Menu), and Phase 4 command routing"
  - "Timeout check happens at top of handle_authenticated_input before any input processing"
  - "Daily time used updated in three places: quit, disconnect, and timeout for complete coverage"
metrics:
  tasks_completed: 2
  tests_passed: 164
  compilation_warnings: 27 (all pre-existing)
---

# Phase 4 Plan 6: Session Lifecycle Integration Summary

Full Phase 4 feature integration: timer lifecycle, command routing, session history, timeout handling, time banking withdrawal.

## What Was Done

### Task 1: Timer Lifecycle and Session History Integration
- Added 4 new fields to Session struct: `session_timer`, `session_history_id`, `lookup_input`, `withdrawal_offered`
- Created `start_session_timer()` method that checks daily reset, gets remaining time, handles zero-time and banked-time edge cases
- Called `start_session_timer()` in all 3 authentication paths (login success, session resume, registration auto-login)
- Added session history recording (`insert_session_history`) in all 3 authentication paths
- Added timer cancellation + session history update + daily time tracking in `handle_quit()`
- Added timer cancellation + session history update + daily time tracking in `on_disconnect()`
- Added NodeManager `update_activity("Main Menu")` after authentication in all paths
- Added NodeManager `update_last_input()` at top of `handle_authenticated_input()`
- Added NodeManager `update_activity()` on service enter and exit

### Task 2: Command Routing, Timeout Handling, and Time Bank Withdrawal
- Added timeout check (expired flag polling) at top of `handle_authenticated_input()`
- Implemented `handle_timeout()` method with logout JSON, stats save, timeout goodbye screen, session cleanup
- Added low-time check with time bank withdrawal prompt (offers [B] key to withdraw 30 minutes)
- Added `__time_bank_prompt__` sentinel handling with timer restart on withdrawal
- Added `handle_whos_online()` -- fetches nodes from NodeManager, renders table
- Added `handle_last_callers()` -- queries session_history, renders table
- Added `handle_user_lookup_start()` -- shows lookup prompt, enters character input mode
- Added `handle_user_lookup_input()` -- character-by-character handle entry with backspace, Enter submits
- Added `__user_lookup_view__` and `__user_lookup_retry__` sentinel states for post-lookup navigation
- Routed "whos_online", "last_callers", "user_lookup" commands in both main ExecuteCommand and typeahead handlers

## Key Decisions

1. **Timer start in all 3 auth paths**: Ensures timer starts regardless of how user authenticates
2. **Timeout check before input processing**: Prevents any user action after time expires
3. **Withdrawal offered once, reset on success**: Prevents spam but allows re-prompt if bank time consumed again
4. **Sentinel service pattern**: Reuses existing current_service mechanism for lightweight state routing
5. **30-minute bank withdrawal amount**: Reasonable chunk that extends session meaningfully
6. **Daily time tracking in 3 exit paths**: quit, disconnect, timeout -- ensures no time goes unrecorded

## Files Modified

- `backend/src/websocket/session.rs` -- +490 lines: timer lifecycle, command routing, all handler methods

## Deviations from Plan

None -- plan executed exactly as written.

## Verification

- `cargo check` passes (no new errors, all pre-existing warnings only)
- `cargo test` passes: 164/164 tests pass
- Full integration chain verified: authentication triggers timer start, session history recording, and NodeManager activity updates
- Command routing for W/L/U verified through compilation of all handler methods
- Timeout handling verified through compilation of handle_timeout() with goodbye screen
- Timer cancellation in quit/disconnect prevents double cleanup
