---
phase: 07-news-bulletins
plan: 03
subsystem: terminal-ui
tags: [ansi, menu-integration, keyboard-navigation, rss, session-lifecycle]

# Dependency graph
requires:
  - phase: 07-02
    provides: NewsState and ANSI rendering functions for news display
  - phase: 06-05
    provides: Menu integration patterns for commands via ExecuteCommand
  - phase: 05-03
    provides: Sentinel service pattern for input routing
provides:
  - News accessible via N hotkey from main menu
  - Arrow key navigation for news list browsing
  - Full article viewing with N/P navigation
  - Loading screen and error handling for feed fetches
affects: [phase-08-games, phase-13-sysop]

# Tech tracking
tech-stack:
  added: []
  patterns: [sentinel-service-pattern, arrow-key-input-handling, async-feed-loading]

key-files:
  created: []
  modified:
    - backend/src/websocket/session.rs
    - config.toml

key-decisions:
  - "N hotkey triggers news directly (not submenu) matching M for mail and C for chat"
  - "Arrow key escape sequences handled in session input for list navigation"
  - "Separate sentinels for news list view (__news__) and error screen (__news_error__)"

patterns-established:
  - "Arrow key navigation pattern: check for \\x1b[A/\\x1bOA (up) and \\x1b[B/\\x1bOB (down)"
  - "Loading screen shown before async operations (feed fetching)"
  - "Error-only state (__news_error__) for graceful degradation when all feeds fail"

# Metrics
duration: 4min
completed: 2026-01-28
---

# Phase 7 Plan 3: News Integration Summary

**News accessible via N hotkey with arrow key navigation, article viewing, and graceful error handling for RSS feeds**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-29T01:19:54Z
- **Completed:** 2026-01-29T01:23:56Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- News integrated into main menu as direct command (N hotkey)
- Full input routing with arrow keys for list navigation
- Article viewing mode with N/P navigation between articles
- Loading screen shows while feeds are being fetched
- Graceful error handling when feeds fail to load

## Task Commits

Each task was committed atomically:

1. **Task 1: Update config.toml menu to use news as command** - `84d2a91` (feat)
2. **Task 2: Add news state and input handling to session.rs** - `14780df` (feat)
3. **Task 3: Test news feature end-to-end** - (verification task, no commit)

## Files Created/Modified
- `config.toml` - Changed News menu entry from submenu to direct command
- `backend/src/websocket/session.rs` - Added news imports, state field, sentinel handlers, menu command handler, and handle_news_input method

## Decisions Made

**Menu integration pattern:**
- News follows direct command pattern (like Mail and Chat) instead of submenu
- N hotkey triggers news command immediately without submenu navigation

**Input handling architecture:**
- Arrow key escape sequences detected in handle_news_input
- Separate viewing states: list view (navigation, selection) vs article view (reading, paging)
- Q returns from article to list, or from list to main menu

**Error handling strategy:**
- Loading screen shown during async feed fetch
- If all feeds fail, show error screen with __news_error__ sentinel
- Any key press from error screen returns to main menu

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation proceeded smoothly following established patterns from mail and chat integration.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for Phase 8 (First Door Game - Drug Wars):**
- All core BBS services complete (mail, chat, news)
- Session lifecycle patterns well-established for game integration
- Menu system ready to support Games submenu

**Phase 7 complete:**
- Users can access news via N hotkey
- RSS feeds fetched and displayed with source attribution
- List navigation works with arrow keys and N/P paging
- Article viewing works with full navigation
- Feed errors handled gracefully

**Testing notes:**
Manual end-to-end testing should verify:
1. N hotkey at main menu opens news with loading screen
2. News list displays articles grouped by feed source
3. Arrow keys move selection highlight
4. Enter opens article for reading
5. Q exits from article back to list
6. Q exits from list back to main menu
7. Feed errors show user-friendly message

---
*Phase: 07-news-bulletins*
*Completed: 2026-01-28*
