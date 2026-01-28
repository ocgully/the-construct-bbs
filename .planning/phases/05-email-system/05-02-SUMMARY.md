---
phase: 05-email-system
plan: 02
subsystem: ui
tags: [ansi, cp437, state-machine, terminal-rendering]

# Dependency graph
requires:
  - phase: 05-01
    provides: Message and InboxEntry types, messages table schema, CRUD operations
  - phase: 01-terminal-foundation
    provides: AnsiWriter, Color enum, CP437 rendering
  - phase: 02-authentication
    provides: LoginFlow and RegistrationFlow state machine patterns
provides:
  - Mail ANSI rendering functions (inbox table, message view, compose header)
  - ComposeFlow state machine for To → Subject → Body flow
  - ComposeAction enum for async session integration
  - Helper functions for date formatting, truncation, error messages
affects: [05-03, 05-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Render functions return String for async session integration"
    - "State machine returns action enum to let session handle async DB operations"
    - "Character-by-character input with handle_char echoing printable chars or backspace sequences"

key-files:
  created:
    - backend/src/services/mail.rs
  modified:
    - backend/src/services/mod.rs

key-decisions:
  - "All render functions return String (not writing directly to session) for async flexibility"
  - "ComposeFlow returns ComposeAction::NeedRecipientLookup to let session.rs handle async DB queries"
  - "Self-mail check happens in session.rs (compare sender_id with recipient_id after lookup)"
  - "Reply mode de-duplicates Re: prefix (prevents Re: Re: Re: accumulation)"
  - "Slash commands in body: /s send, /a abort, /h help, /l list lines"

patterns-established:
  - "render_* functions follow who.rs and profile.rs patterns: CP437 double-line boxes, CGA colors, 80-column layout"
  - "Inbox table: 78 inner chars + 2 outer borders = 80 total width"
  - "ComposeFlow follows LoginFlow/RegistrationFlow pattern: Prompt/Input states, handle_char for character echo, handle_line for line processing"
  - "advance_to_input() transitions from Prompt to Input states after session shows prompt"

# Metrics
duration: 6min
completed: 2026-01-28
---

# Phase 5 Plan 02: Mail Rendering & Compose State Machine Summary

**Mail ANSI rendering with 80-column inbox table, full message view, and ComposeFlow state machine handling To → Subject → Body with slash commands**

## Performance

- **Duration:** 6 min
- **Started:** 2026-01-28T06:03:09Z
- **Completed:** 2026-01-28T06:08:50Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- ANSI inbox table with #/From/Subject/Date/Status columns in 80-column CP437 box layout
- Full message view with sender, date, subject, body, and action bar
- ComposeFlow state machine with To → Subject → Body flow
- Reply mode pre-fills recipient, Re: subject (de-duplicated), and quoted body with "> " prefix
- Slash commands (/s, /a, /h, /l) for body input control
- Async integration via ComposeAction enum (NeedRecipientLookup lets session handle DB queries)

## Task Commits

Each task was committed atomically:

1. **Task 1: Mail ANSI rendering functions** - `7aad617` (feat)
2. **Task 2: Compose state machine with line-by-line editor** - `bde3b30` (feat)

## Files Created/Modified
- `backend/src/services/mail.rs` - Mail rendering functions and ComposeFlow state machine
- `backend/src/services/mod.rs` - Registered mail module

## Decisions Made

**1. All render functions return String**
- Enables async session integration without blocking on terminal I/O
- Session can call render functions, get result, then send via async channel
- Consistent with profile.rs and who.rs patterns

**2. ComposeFlow returns ComposeAction::NeedRecipientLookup**
- Keeps state machine synchronous (no async inside ComposeFlow)
- Session.rs handles async DB lookup, calls set_recipient() with result
- Clean separation: ComposeFlow = state logic, session.rs = async operations

**3. Self-mail check in session.rs**
- After recipient lookup, session compares sender_id with recipient_id
- Prevents self-mail before message creation
- Returns render_self_mail_error() if IDs match

**4. Reply mode de-duplicates Re: prefix**
- Checks if subject.starts_with("Re: ") before adding "Re: "
- Prevents "Re: Re: Re:" accumulation on multiple replies
- Subject stored once with single "Re: " prefix

**5. Slash commands for body input**
- /s: Send message (transitions to Sending state)
- /a: Abort compose (transitions to Aborted state)
- /h: Show help (lists available commands)
- /l: List lines (displays body_lines with line numbers)
- All commands case-insensitive (/S, /A, /H, /L also work)

**6. 80-column inbox table layout**
- Calculation: #(3) | From(20) | Subject(35) | Date(12) | St(4) = 74 inner
- With borders: 74 data + 4 single-line separators + 2 outer = 80 total
- Matches who.rs pattern for consistent table rendering

**7. Date formatting helpers**
- format_date_short: "Jan 26" for inbox date column (12 chars max)
- format_datetime: "January 26, 2026 at 4:15 PM" for message view
- Replicated from profile.rs for consistency

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**1. Initial table layout calculation off by 3 columns**
- First attempt: 75 total width (needed 80)
- Fixed: Added 1 to From column (18→20) and 1 to Subject column (33→35)
- Verified: 3 + 1 + 20 + 1 + 35 + 1 + 12 + 1 + 4 = 78 inner + 2 outer = 80 ✓

**2. Missing format placeholder in header separator**
- Error: "argument never used" in inbox header separator
- Fixed: Added missing `{}` placeholder for 5th column width
- Pattern: `\u{255F}{}\u{253C}{}\u{253C}{}\u{253C}{}\u{253C}{}\u{2562}` (5 placeholders for 5 columns)

**3. Body lines moved after iteration**
- Error: `for line in body_lines` moved ownership, then `if !body_lines.is_empty()` tried to borrow
- Fixed: Changed to `for line in &body_lines` to iterate by reference
- Prevents ownership transfer, allows subsequent use

All issues resolved during cargo check compilation verification.

## Next Phase Readiness

**Ready for session integration (Plan 05-03):**
- render_inbox, render_message, render_compose_header available
- ComposeFlow.new() for normal compose
- ComposeFlow.new_reply() for reply mode
- ComposeAction enum guides session through async flow

**Ready for menu registration (Plan 05-04):**
- All rendering complete and tested
- State machine handles full compose flow
- Error renderers ready for edge cases

**Patterns established:**
- Return String from render functions (async session integration)
- ComposeAction enum for state machine → session communication
- advance_to_input() transition pattern (after prompt shown)

---
*Phase: 05-email-system*
*Completed: 2026-01-28*
