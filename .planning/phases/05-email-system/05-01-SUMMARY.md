---
phase: 05-email-system
plan: 01
subsystem: database
tags: [sqlite, sqlx, messages, mail, crud]

# Dependency graph
requires:
  - phase: 02-auth-and-connection
    provides: Database layer with SQLite pool, User table, string-based sqlx queries
  - phase: 04-time-limits
    provides: EST timezone convention with datetime('now', '-5 hours')
provides:
  - Messages table with sender/recipient foreign keys and CASCADE delete
  - InboxEntry struct for lightweight inbox list queries
  - Full CRUD operations for messages (create, read, count, mark read, delete)
  - MailConfig with mailbox_size_limit in config.toml
  - Mailbox capacity checking function
affects: [06-chat-system, 05-email-system-plans-02-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - InboxEntry projection pattern for list views (omits body, recipient_id)
    - Self-mail validation in create_message (returns Protocol error)
    - Newline normalization in create_message (all \r\n and \r to \n)
    - Ownership checks built into SQL queries (recipient_id in WHERE clause)
    - Deduplication in get_sender_handles before querying

key-files:
  created:
    - backend/src/db/messages.rs
  modified:
    - backend/src/db/schema.sql
    - backend/src/db/mod.rs
    - backend/src/config.rs
    - config.toml
    - backend/src/services/login.rs (test config)
    - backend/src/services/registration.rs (test config)

key-decisions:
  - "InboxEntry struct separates list view from full message read"
  - "Self-mail validation prevents sender_id == recipient_id"
  - "Newline normalization ensures consistent \n-only storage"
  - "Ownership checks in SQL prevent unauthorized message access"
  - "Mail config named 'mail' (not 'email') to avoid SMTP EmailConfig confusion"

patterns-established:
  - "Projection structs for list views: InboxEntry has id, sender_id, subject, sent_at, is_read (omits body, recipient_id)"
  - "Validation in CRUD layer: create_message validates before INSERT"
  - "Built-in ownership: queries use 'AND recipient_id = ?' for security"

# Metrics
duration: 5min
completed: 2026-01-28
---

# Phase 5 Plan 01: Email System Foundation Summary

**Messages table with CASCADE foreign keys, InboxEntry projection pattern, and MailConfig with mailbox size limits**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-28T20:00:43Z
- **Completed:** 2026-01-28T20:05:13Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Messages table DDL with sender/recipient foreign keys, CASCADE delete, and indexed queries
- Complete CRUD operations: create, read (full and paginated), count, mark read, delete
- InboxEntry struct provides lightweight list view without body content
- MailConfig section in config.toml with mailbox_size_limit defaulting to 100
- Self-mail validation and newline normalization in message creation

## Task Commits

Each task was committed atomically:

1. **Task 1: Messages table schema and CRUD operations** - `2ddd607` (feat)
2. **Task 2: Mail configuration section** - `9a9d1ce` (feat)

## Files Created/Modified
- `backend/src/db/messages.rs` - Message and InboxEntry structs with 9 CRUD functions
- `backend/src/db/schema.sql` - Messages table DDL with foreign keys and indexes
- `backend/src/db/mod.rs` - Registered messages module
- `backend/src/config.rs` - MailConfig struct with mailbox_size_limit
- `config.toml` - [mail] section with mailbox_size_limit = 100
- `backend/src/services/login.rs` - Updated test_config to include mail field
- `backend/src/services/registration.rs` - Updated test_config to include mail field

## Decisions Made

1. **InboxEntry projection pattern** - Separate struct for list views omits body and recipient_id, reducing memory for paginated inbox display. Full Message struct only loaded when reading individual message.

2. **Self-mail validation** - create_message validates sender_id != recipient_id and returns sqlx::Error::Protocol to prevent users sending messages to themselves.

3. **Newline normalization** - create_message normalizes all \r\n and \r to \n before storing, ensuring consistent line endings across platforms.

4. **Ownership checks in SQL** - All query functions include `AND recipient_id = ?` in WHERE clauses, preventing unauthorized message access at database level.

5. **Mail vs Email naming** - Config section named `mail` (not `email`) to avoid confusion with existing SMTP `EmailConfig` used for verification emails.

6. **Sender handle batch loading** - get_sender_handles accepts slice of IDs and deduplicates before querying, enabling efficient inbox display with sender names.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Test configuration updates** - Adding mail field to Config struct broke test_config() functions in login.rs and registration.rs tests. Fixed by adding `mail: crate::config::MailConfig::default()` to both test configs. This is expected maintenance when adding new Config fields.

## Next Phase Readiness

**Ready for Plans 02-04:**
- Messages table and indexes in place
- CRUD operations tested and functional
- MailConfig available for mailbox capacity checks
- InboxEntry struct ready for paginated inbox display

**Foundation complete for:**
- Plan 02: Read Mail service (uses get_inbox_page, get_message_by_id, mark_message_read)
- Plan 03: Send Mail service (uses create_message, check_mailbox_full, get_sender_handles)
- Plan 04: Sent Messages service (needs additional query functions for sender_id filtering)

**No blockers.** All database and config foundation in place.

---
*Phase: 05-email-system*
*Completed: 2026-01-28*
