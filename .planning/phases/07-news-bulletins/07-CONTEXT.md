# Phase 7: News & Bulletins - Context

**Gathered:** 2026-01-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Display news from RSS feeds to users via menu access. No sysop bulletins — RSS-only. Multiple feeds merged and grouped by source. Users access news from main menu via N hotkey.

</domain>

<decisions>
## Implementation Decisions

### Display Timing
- Menu access only — no news on login
- No unread indicator in status bar
- N hotkey on main menu for direct access (like M for mail, C for chat)

### News Sources
- Multiple RSS feeds supported, merged into one news view
- Focus on retro/BBS-themed news if available, with tech and world news as fallback
- Feeds configured in config.toml by sysop
- BBS-themed ANSI header (e.g., "THE WIRE" or similar atmospheric name)

### News Presentation
- Headlines + snippet format in list view
- Arrow keys / N-P paging for navigation (up/down to highlight, Enter to select)
- Full article shown in terminal with [More] paging, Q to go back
- Source attribution shown before each title (e.g., "[Ars Technica]")

### RSS Behavior
- Fetch fresh on every access (no caching)
- Show error message if feed unavailable (no fallback to cached content)
- 10 most recent articles per feed
- Articles grouped by feed (not merged chronologically)

### Bulletins
- No bulletins — phase is RSS news only
- Sysop bulletin feature explicitly excluded

### Claude's Discretion
- Exact ANSI header design and theming
- Default RSS feed URLs to ship with
- Error message formatting
- Navigation key bindings details

</decisions>

<specifics>
## Specific Ideas

- User wants retro/BBS-themed news feeds if real RSS sources exist for that content
- Themed header like "THE WIRE" or "NEWS FROM THE MATRIX" for atmosphere
- Navigation should feel like browsing a classic BBS news section

</specifics>

<deferred>
## Deferred Ideas

- Sysop bulletins — user explicitly declined for this phase
- Unread news indicator — user explicitly declined

</deferred>

---

*Phase: 07-news-bulletins*
*Context gathered: 2026-01-28*
