---
phase: 05-email-system
plan: 03
subsystem: services
tags: [session, mail, sentinel-handlers, menu-routing]

# Dependency graph
requires:
  - phase: 05-email-system
    plan: 01
    provides: Messages DB CRUD operations (get_inbox_page, get_message_by_id, create_message, etc.)
  - phase: 05-email-system
    plan: 02
    provides: Mail rendering functions and ComposeFlow state machine
  - phase: 03-navigation
    provides: MenuAction::ExecuteCommand pattern for command routing
  - phase: 04-time-limits
    provides: Sentinel service pattern (__whos_online__, __last_callers__, __user_lookup__)
provides:
  - Mail command routing from main menu (M hotkey -> "mail" command -> show_inbox)
  - __mail_inbox__ sentinel handler for paginated inbox navigation (C/N/P/Q/digit)
  - __mail_read__ sentinel handler for message actions (R/D/N/Q)
  - __mail_compose__ sentinel handler driving ComposeFlow state machine
  - Login notification for unread messages after authentication
affects: [05-04-timer-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Mail command routing follows MenuAction::ExecuteCommand pattern
    - Sentinel handlers follow established __service_name__ pattern
    - Character-by-character input processing for message number entry
    - ComposeFlow state machine driven via handle_char returning ComposeAction
    - Self-mail validation via user_id comparison (not handle comparison)
    - Mailbox-full check before sending message

key-files:
  created: []
  modified:
    - backend/src/websocket/session.rs

key-decisions:
  - "Mail command routing added to BOTH MenuAction::ExecuteCommand match blocks"
  - "Page size hardcoded to 10 messages per page (no config field)"
  - "Mail input buffer accumulates digits for message number selection"
  - "Self-mail check compares user_id values after async DB lookup"
  - "Mailbox full check uses config.mail.mailbox_size_limit"
  - "Login notification shows in all three auth paths (login, resume, registration)"
  - "Unread notification appears AFTER welcome message, BEFORE main menu"

patterns-established:
  - "Sentinel handler pattern for mail views: __mail_inbox__, __mail_read__, __mail_compose__"
  - "show_inbox() helper method pattern for menu command dispatch"
  - "Character-by-character input with match arms for single-char commands"
  - "ComposeAction routing pattern for async DB operations within synchronous state machine"

# Metrics
duration: 8min
completed: 2026-01-28
---

# Phase 5 Plan 03: Mail Session Integration Summary

**Sentinel service handlers for inbox, read, and compose with menu command routing and login notification**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-28T05:27:40Z
- **Completed:** 2026-01-28T05:36:08Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Mail command routing from main menu M hotkey to show_inbox()
- Three sentinel handlers for mail navigation (inbox, read, compose)
- Inbox pagination with C/N/P/Q/digit navigation
- Message reading with R/D/N/Q actions (reply, delete, next, quit)
- Compose flow with character-by-character input through ComposeFlow state machine
- Self-mail and mailbox-full validation enforced
- Login notification for unread messages in all three authentication paths

## Task Commits

Each task was committed atomically:

1. **Task 1: Sentinel service handlers and mail command routing** - `65c4d74` (feat)
2. **Task 2: Login notification for new mail** - `81c4a21` (feat)

## Files Created/Modified
- `backend/src/websocket/session.rs` - Added 4 mail-related fields to Session struct, imported messages DB and mail services modules, added "mail" command to both MenuAction::ExecuteCommand match blocks, implemented 3 sentinel handlers (__mail_inbox__, __mail_read__, __mail_compose__) with helper methods, added unread mail notification to 3 auth paths

## Decisions Made

1. **Mail command routing in BOTH match blocks** - Added "mail" command to both MenuAction::ExecuteCommand match blocks (lines ~1195 and ~1270) following exact same pattern as "profile", "whos_online", etc. This ensures both single-keypress and command dispatch paths work.

2. **Hardcoded page size** - Used 10 messages per page (hardcoded) since MailConfig only has mailbox_size_limit field, not page_size. This is a reasonable default matching common BBS conventions.

3. **Mail input buffer for message numbers** - Added mail_input_buffer: Option<String> to Session struct to accumulate digit input for message number selection, cleared after Enter or Q.

4. **Self-mail check via user_id** - Self-mail validation compares user_id values (not handles) after async find_user_by_handle lookup. This prevents case-sensitivity issues and handles changes.

5. **Mailbox limit from config** - Used self.state.config.mail.mailbox_size_limit for capacity check (not page_size or mailbox_limit which don't exist).

6. **Login notification in all auth paths** - Added unread mail check to session resume (line ~507), login success (line ~663), and registration success (line ~870). Notification appears AFTER welcome message, BEFORE main menu display.

7. **ComposeAction routing pattern** - Compose handler routes all ComposeAction variants including Echo, ShowPrompt, NeedRecipientLookup, SendMessage, Aborted, ShowHelp, ShowLines. Async DB operations (recipient lookup, mailbox check, message creation) handled in session.rs, not in synchronous ComposeFlow.

8. **Message number to inbox entry mapping** - Message numbers are 1-based for user display. Calculation: page_start = page * page_size; page_index = msg_num - page_start - 1. Validates range before fetching full message.

## Deviations from Plan

**Auto-fix (Rule 2 - Missing Critical):**

1. **Messages_sent counter increment** - Plan specified incrementing messages_sent via update_user_field, but didn't provide the current value. Added fetch of current messages_sent from user record before incrementing to avoid overwriting with literal value.

## Issues Encountered

**MailConfig field names** - Initial implementation used `config.mail.page_size` and `config.mail.mailbox_limit` but actual MailConfig only has `mailbox_size_limit` field (from Plan 05-01). Fixed by hardcoding page size to 10 and using correct field name.

**ComposeFlow state access for reply** - When showing reply compose header, needed to display subject but ComposeFlow doesn't expose subject via state() method. Worked around by re-displaying subject from original message after calling render_compose_header.

## Next Phase Readiness

**Ready for Plan 04 (Timer Integration):**
- Mail sentinel handlers in place and functional
- show_inbox() helper method ready to be called from timer integration
- All mail navigation working (inbox -> read -> compose/reply/delete)

**Foundation complete for:**
- Plan 04: Menu configuration and timer integration (add M hotkey to config.toml)
- Future mail enhancements: sent messages, message threading, attachments

**No blockers.** All session integration complete. Mail system fully functional pending config.toml menu entry.

---
*Phase: 05-email-system*
*Completed: 2026-01-28*
