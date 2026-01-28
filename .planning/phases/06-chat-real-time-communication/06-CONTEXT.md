# Phase 6: Chat & Real-Time Communication - Context

**Gathered:** 2026-01-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Single-room teleconference chat where users can see messages from all participants in real-time. Includes join/leave announcements, /me emote actions, and user-to-user paging for direct messages within the main chat. This is a single shared room — multiple chat rooms or channels are out of scope.

</domain>

<decisions>
## Implementation Decisions

### Chat room experience
- Message format: `[HH:MM] Handle: Message text` (timestamp prefix)
- Auto-scroll to bottom — new messages always visible, no scroll control
- Dedicated input line at bottom — fixed prompt area, messages display above
- Handle colors: Dark green for username, light green for message text
- Sysop can send "official" messages as "SysOp" in yellow, but regular sysop chat is same format as users (white text)

### User actions & commands
- Slash prefix for commands: /me, /who, /quit, /help
- /me actions display as: `* Handle waves` (asterisk prefix, no timestamp, no colon)
- /who shows users currently in chat
- /quit or /q exits chat, returns to menu
- /help or /? shows available commands
- Unknown commands show error: "Unknown command. Type /help for available commands."

### Paging & private chat
- Page via `/page <handle>` command in chat
- Page notification: Bell sound + message to recipient
- Direct messages only — 1:1 messages within main chat, visible only to sender and recipient
- DM syntax: `/msg <handle> <message>` for explicit, `/r <message>` for quick reply to last person

### Join/leave & presence
- Announcement format: `*** Handle has joined the chat ***` / `*** Handle has left the chat ***`
- System messages (join/leave, errors) in Yellow
- Separate configurable chat capacity limit (independent of max_nodes)
- No idle kick — users stay until /quit or disconnect

### Claude's Discretion
- Exact screen layout and row positioning
- DM display format (how to distinguish from public messages)
- Bell sound implementation details
- Chat capacity default value

</decisions>

<specifics>
## Specific Ideas

- Classic BBS teleconference feel with timestamps and asterisk-decorated announcements
- Sysop "official" announcements should feel distinct (yellow "SysOp" handle) from their regular chat participation
- Bell sound for pages creates urgency/attention like classic BBS paging

</specifics>

<deferred>
## Deferred Ideas

- Multiple chat rooms / channels — future phase
- Chat logging / history retrieval — future phase
- ~~Bug fix needed: Mail compose self-mail error causes input lock~~ — FIXED (missing advance_to_input() call after error prompts)

</deferred>

---

*Phase: 06-chat-real-time-communication*
*Context gathered: 2026-01-28*
