# Phase 5: Email System - Context

**Gathered:** 2026-01-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Inter-user private messaging within the BBS. Users can compose and send messages to other users by handle, read their inbox with unread indicators, reply to messages with quoted text, delete messages permanently, and receive new-mail notifications at login and during active sessions. Public message boards, file attachments, and group messaging are out of scope.

</domain>

<decisions>
## Implementation Decisions

### Message Composition
- Line-by-line entry: type a line, press Enter for next line, /s to send, /a to abort. Authentic BBS mail feel.
- Full header prompts: To (handle) -> Subject -> Body. Traditional BBS mail format.
- Recipient addressed by typing handle directly. Error if handle not found.
- No message length limit. Message sends immediately after /s (no preview/confirm step).
- Self-mail blocked: error if user addresses message to their own handle.

### Inbox & Reading
- Numbered list with unread marker: # | From | Subject | Date | Status (N for new, R for read). User enters number to read.
- Paginated at 15 messages per page with [N]ext page / [P]rev page navigation.
- Reading a message shows full content with action bar at bottom: [R]eply [D]elete [N]ext [Q]uit to inbox.
- Deletion is immediate and permanent. No trash folder, no recovery.

### Reply & Threading
- Replies include quoted original message with > prefix on each line. User types new content below.
- Subject auto-populated with "Re: [original subject]" (not editable by user).
- Flat chronological inbox listing. Replies show "Re: Subject" but are not grouped into threads.
- Self-mail blocked on replies as well.

### New Mail Notification
- Login notification: "You have X new messages." displayed after welcome-back. Inform only, no prompt to read.
- Real-time notification: Status bar indicator (e.g., "MAIL" flag on row 24) when unread mail exists during active session.
- No inline ANSI alert for real-time -- status bar only.
- Configurable mailbox size limit: sysop sets max messages per user in config.toml. "Mailbox full" error when exceeded.

### Claude's Discretion
- Exact line editor commands beyond /s and /a (e.g., /h for help, /l to list lines)
- ANSI styling of inbox list and message display (colors, box drawing)
- How "Mailbox full" is communicated to senders
- Status bar MAIL indicator placement and color
- Database schema for messages table

</decisions>

<specifics>
## Specific Ideas

- Line-by-line editor matches authentic BBS mail conventions (not a full-screen editor)
- To/Subject/Body prompt flow mirrors classic Wildcat/RBBS mail
- Flat inbox with "Re:" subjects is exactly how 90s BBS mail worked
- Status bar already exists on row 24 -- MAIL indicator extends it naturally
- Deletion matches BBS convention of permanent removal (no modern "undo" affordances)

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 05-email-system*
*Context gathered: 2026-01-28*
