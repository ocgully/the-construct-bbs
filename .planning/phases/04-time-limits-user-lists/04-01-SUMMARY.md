---
phase: 04-time-limits-user-lists
plan: 01
subsystem: database
tags: [sqlx, sqlite, config, time-banking, session-tracking, toml]

# Dependency graph
requires:
  - phase: 02-auth-connection
    provides: User model, database pool, session management, NodeManager
  - phase: 01-terminal-foundation
    provides: Config system with serde deserialization

provides:
  - TimeLimitsConfig with per-level time limits (guest/user/sysop) and bank caps
  - session_history table for tracking login/logout history
  - Time banking columns on users table (daily_time_used, banked_time, last_daily_reset)
  - Time banking query functions (check_daily_reset, reset_daily_time, update_daily_time_used, withdraw_banked_time)
  - NodeManager extensions for activity tracking (current_activity, last_input fields)
  - Session history CRUD operations (insert, update_logout, get_last_callers)

affects: [04-02, 04-03, 04-04, 04-05, 04-06]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-level configuration using nested TOML tables with #[serde(default)]"
    - "Time banking with daily reset detection via SQLite datetime functions"
    - "Activity tracking in NodeManager for idle detection"

key-files:
  created:
    - backend/src/db/session_history.rs
  modified:
    - backend/src/config.rs
    - backend/src/db/schema.sql
    - backend/src/db/user.rs
    - backend/src/connection/node_manager.rs
    - config.toml

key-decisions:
  - "Time limits configurable per user level (Guest/User/Sysop) in config.toml"
  - "Session history uses datetime('now', '-5 hours') for EST timezone consistency"
  - "Time banking with daily reset detection using SQLite date() comparison"
  - "NodeManager tracks current_activity string and last_input timestamp for idle detection"

patterns-established:
  - "TimeLimitsConfig::get_time_config(user_level) helper method for level-based lookups"
  - "Session history CRUD pattern: insert on login, update_logout on disconnect"
  - "Time banking query functions return Result for error handling consistency"

# Metrics
duration: 8min
completed: 2026-01-27
---

# Phase 4 Plan 01: Time Limits Foundation Summary

**Database schema and config extensions for per-user time limits, time banking, session history, and activity tracking**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-27T22:00:26Z
- **Completed:** 2026-01-27T22:08:52Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- TimeLimitsConfig with per-level daily_minutes and time_bank_cap (Guest=30min/0bank, User=60min/120bank, Sysop=unlimited/0bank)
- session_history table with login/logout tracking and duration_minutes field
- Time banking columns on users table with daily reset detection via SQLite datetime queries
- NodeManager extended with current_activity and last_input fields for idle/activity tracking
- All time banking query functions implemented (check_daily_reset, reset_daily_time, update_daily_time_used, get_user_time_info, withdraw_banked_time)

## Task Commits

Each task was committed atomically:

1. **Task 1: Time Limits Config and Database Schema Extensions** - `a9171b4` (feat)
2. **Task 2: Session History DB Module, Time Banking Queries, and NodeManager Extensions** - `2797fd7` (feat)

## Files Created/Modified
- `backend/src/config.rs` - Added TimeLimitsConfig with TimeLevelConfig nested struct and get_time_config helper
- `backend/src/db/schema.sql` - Added session_history table and time banking columns to users table
- `config.toml` - Added [time_limits] section with per-level defaults
- `backend/src/db/session_history.rs` - Session history CRUD (insert, update_logout, get_last_callers)
- `backend/src/db/user.rs` - Added time banking query functions and updated User struct
- `backend/src/db/mod.rs` - Added session_history module export
- `backend/src/connection/node_manager.rs` - Added current_activity and last_input to NodeInfo with update methods
- `backend/src/services/login.rs` - Updated test schema and Config struct for new fields
- `backend/src/services/profile.rs` - Updated test User struct for new fields
- `backend/src/services/registration.rs` - Updated test schema and Config struct for new fields

## Decisions Made
- **Database deletion during dev:** SQLite CREATE TABLE IF NOT EXISTS skips alterations. During development, deleted existing database files (bbs.db, bbs.db-shm, bbs.db-wal) to recreate schema with new columns. This is acceptable for dev stage.
- **EST timezone in schema:** All datetime fields use datetime('now', '-5 hours') for EST consistency with existing schema pattern.
- **Zero daily_minutes = unlimited:** Sysop level uses 0 to indicate unlimited time (no enforcement).
- **Time bank cap enforcement:** Cap enforced during reset_daily_time to prevent over-banking.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Test failures from missing schema columns:**
- **Problem:** Unit tests create in-memory databases with hardcoded schemas that didn't include new columns (daily_time_used, banked_time, last_daily_reset).
- **Resolution:** Updated test setup functions in login.rs and registration.rs to include new columns in CREATE TABLE statements.
- **Rule applied:** Rule 1 (Bug fix) - Tests failing due to outdated schema is a bug.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Foundation complete for Phase 4 time limit features:
- Plan 04-02 can implement session timer with countdown display
- Plan 04-03 can use session_history for Last Callers display
- Plan 04-04 can use NodeInfo.current_activity for Who's Online display
- Plan 04-05 can use time banking queries for time withdrawal and daily reset
- Plan 04-06 can implement graceful timeout with session_history tracking

All required data structures, database tables, and query functions are in place.

---
*Phase: 04-time-limits-user-lists*
*Completed: 2026-01-27*
