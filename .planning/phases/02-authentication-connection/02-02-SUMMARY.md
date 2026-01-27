---
phase: 02-authentication-connection
plan: 02
subsystem: auth
tags: [argon2id, uuid, rustrict, password-hashing, session-management, validation, node-manager, rwlock]

# Dependency graph
requires:
  - phase: 02-01
    provides: SQLite database pool, schema (sessions table), User CRUD, config (AuthConfig, ConnectionConfig)
provides:
  - Argon2id password hashing with OWASP 2026 parameters
  - Session token generation (UUID v4) and CRUD operations
  - Handle validation with profanity filter (rustrict)
  - Email and password validation
  - NodeManager with thread-safe node tracking and assignment
  - AppState with auth and connection modules wired in
affects: [02-03-session-management, 02-04-registration, 02-05-connection-ceremony]

# Tech tracking
tech-stack:
  added: []
  patterns: [Argon2id OWASP 2026 params, Arc<RwLock<HashMap>> for concurrent state, UUID v4 session tokens, rustrict profanity filter]

key-files:
  created:
    - backend/src/auth/mod.rs
    - backend/src/auth/password.rs
    - backend/src/auth/session.rs
    - backend/src/auth/validation.rs
    - backend/src/connection/mod.rs
    - backend/src/connection/node_manager.rs
  modified:
    - backend/src/main.rs

key-decisions:
  - "Argon2id with OWASP 2026 params: m=19456 KiB, t=2, p=1 (CPU-intensive, callers use spawn_blocking)"
  - "verify_password returns Ok(false) on mismatch, not Err (clean API for callers)"
  - "tokio::sync::RwLock for NodeManager (async-aware, not std RwLock)"
  - "First-available node assignment from 1..=max_nodes"

patterns-established:
  - "Pattern: hash_password/verify_password as sync blocking functions, callers wrap with spawn_blocking"
  - "Pattern: NodeManager with Arc<RwLock<HashMap>> cloneable via derive(Clone)"
  - "Pattern: Session CRUD with string-based sqlx queries and datetime arithmetic"

# Metrics
duration: 4min
completed: 2026-01-27
---

# Phase 02 Plan 02: Auth Core and Node Manager Summary

**Argon2id password hashing (OWASP 2026), UUID v4 session CRUD, handle/email/password validation with rustrict profanity filter, and RwLock-based NodeManager for connection scarcity**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-27T01:28:47Z
- **Completed:** 2026-01-27T01:33:11Z
- **Tasks:** 2
- **Files created:** 6
- **Files modified:** 1
- **Tests added:** 31 (23 auth + 8 node_manager)

## Accomplishments
- Password hashing with Argon2id using OWASP 2026 recommended parameters (m=19456 KiB, t=2, p=1)
- Session token generation (UUID v4) with create, validate, delete, cleanup, and duplicate-detection operations
- Handle validation: 3-20 chars, alphanumeric+spaces, no leading/trailing/consecutive spaces, reserved names blocked, profanity filtered via rustrict (catches leetspeak)
- Email validation: structural checks (single @, domain dot, max 254 chars)
- Password validation: 6-128 character bounds
- NodeManager tracks active connections with Arc<RwLock<HashMap<usize, NodeInfo>>>
- Node assignment gives first available node number (1-based), enforces max_nodes limit
- NodeManager integrated into AppState; prints "Node capacity: N nodes" on startup
- All 56 tests pass (31 new + 25 existing)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create auth module (password, session, validation)** - `29d0246` (feat)
2. **Task 2: Create NodeManager and wire into AppState** - `9a4d1d0` (feat)

## Files Created/Modified
- `backend/src/auth/mod.rs` - Auth module re-exports (password, session, validation)
- `backend/src/auth/password.rs` - Argon2id hash_password/verify_password with OWASP 2026 params
- `backend/src/auth/session.rs` - Session generate_token, create/validate/delete/cleanup CRUD
- `backend/src/auth/validation.rs` - validate_handle (with rustrict), validate_email, validate_password
- `backend/src/connection/mod.rs` - Connection module re-exports (NodeManager)
- `backend/src/connection/node_manager.rs` - NodeManager with assign/release/status/lookup methods
- `backend/src/main.rs` - Added mod auth, mod connection; NodeManager in AppState

## Decisions Made
- Argon2id with OWASP 2026 parameters (m=19456 KiB, t=2, p=1) -- most memory-intensive setting recommended for server-side hashing
- verify_password returns Ok(false) on mismatch rather than Err, providing cleaner API for login flows
- Used tokio::sync::RwLock (not std) for NodeManager to work correctly in async context
- Node assignment scans 1..=max_nodes for first available slot, matching BBS line numbering convention

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed test data triggering false positive in rustrict profanity filter**
- **Found during:** Task 1 test verification
- **Issue:** Test handle "User1234567890123456" (20 chars) was flagged as inappropriate by rustrict
- **Fix:** Changed test data to "Abcdefghij0123456789" which passes the profanity filter
- **Files modified:** backend/src/auth/validation.rs
- **Commit:** 29d0246

## Issues Encountered

None beyond the test data fix above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Auth module provides clean API for registration flow (Plan 04): validate_handle, validate_email, validate_password, hash_password
- Session CRUD ready for session management (Plan 03): create_session, validate_session, delete_session
- NodeManager ready for connection ceremony (Plan 05): assign_node, release_node, get_status
- All building blocks are pure logic modules with clear interfaces, ready for composition

---
*Phase: 02-authentication-connection*
*Completed: 2026-01-27*
