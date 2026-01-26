# Requirements: The Construct

**Defined:** 2026-01-26
**Core Value:** The feeling of dialing into an exclusive, underground system — artificial scarcity, ANSI art, and social games that only work because everyone's sharing the same constrained space.

## v1 Requirements

### Connection Experience

- [ ] **CONN-01**: Modem handshake sound effect plays when user connects
- [ ] **CONN-02**: ANSI art splash screen displayed during connection sequence
- [ ] **CONN-03**: "Line busy" rejection when max concurrent users reached
- [ ] **CONN-04**: Goodbye screen with session stats on logout

### Authentication

- [ ] **AUTH-01**: User can register with username and password
- [ ] **AUTH-02**: User can log in with existing credentials
- [ ] **AUTH-03**: Passwords stored with secure hashing (argon2/bcrypt)
- [ ] **AUTH-04**: Session persists across page refresh (reconnect to active session)

### Navigation

- [ ] **NAV-01**: Main menu with Wildcat-style numbered/lettered options
- [ ] **NAV-02**: Hierarchical menu navigation with breadcrumbs
- [ ] **NAV-03**: Hotkey support for rapid menu traversal
- [ ] **NAV-04**: ANSI art menu headers and borders (authentic Wildcat-era style)

### Email

- [ ] **MAIL-01**: User can send private messages to other users
- [ ] **MAIL-02**: User can read inbox with unread indicators
- [ ] **MAIL-03**: User can reply to received messages
- [ ] **MAIL-04**: User can delete messages
- [ ] **MAIL-05**: "You have new mail" notification on login

### Chat

- [ ] **CHAT-01**: Single-room live teleconference chat
- [ ] **CHAT-02**: Real-time message display for all users in room
- [ ] **CHAT-03**: Join/leave announcements when users enter/exit chat
- [ ] **CHAT-04**: Action commands (/me waves, /me laughs)
- [ ] **CHAT-05**: User paging — request private chat with an online user
- [ ] **CHAT-06**: Who's Online list visible from chat and main menu

### Door Games

- [ ] **GAME-01**: Acrophobia clone — real-time multiplayer rounds with timed acronym submissions and voting
- [ ] **GAME-02**: Legend of the Red Dragon clone — daily turns, forest combat, inn, PvP, dragon boss progression
- [ ] **GAME-03**: Usurper clone — medieval RPG with kingdom management, quests, class system, daily turns
- [ ] **GAME-04**: Legend of Kyrandia clone — adventure/puzzle game with inventory, exploration, story progression
- [ ] **GAME-05**: Drug Wars clone — buy/sell commodities across locations, random events, loan shark, 30-day limit
- [ ] **GAME-06**: All door games accessible from main menu as pluggable services
- [ ] **GAME-07**: Game state persists between sessions (save/resume)
- [ ] **GAME-08**: Per-game leaderboards and player stats

### User Features

- [ ] **USER-01**: User profile with display name, join date, location, signature
- [ ] **USER-02**: User stats tracking (logins, time spent, games played, messages sent)
- [ ] **USER-03**: User levels — Guest, User, Co-Sysop, Sysop with permission gating
- [ ] **USER-04**: Last callers list showing recent login history
- [ ] **USER-05**: View other users' profiles

### Time Management

- [ ] **TIME-01**: Daily time limit per user level (enforced, visible countdown)
- [ ] **TIME-02**: Warnings at 5-minute and 1-minute remaining
- [ ] **TIME-03**: Graceful forced logout at zero (auto-save game state first)
- [ ] **TIME-04**: Time bank — save unused daily minutes for future sessions
- [ ] **TIME-05**: Session timer visible in status line

### Sysop Administration

- [ ] **SYSO-01**: Sysop panel accessible from main menu (Sysop level only)
- [ ] **SYSO-02**: User management — edit accounts, reset passwords, ban/unban
- [ ] **SYSO-03**: Toggle services on/off without code changes (data-driven config)
- [ ] **SYSO-04**: View system stats — active users, total accounts, uptime
- [ ] **SYSO-05**: Kick active users from the system
- [ ] **SYSO-06**: Configure time limits and user caps

### News & Bulletins

- [ ] **NEWS-01**: News feed displayed on login or accessible from menu
- [ ] **NEWS-02**: News sourced from configurable RSS feed (world news)
- [ ] **NEWS-03**: Sysop can post custom bulletins alongside RSS news

### Architecture

- [x] **ARCH-01**: All user-facing services implemented as pluggable modules with common interface
- [x] **ARCH-02**: Service registry — data-driven enable/disable via configuration
- [x] **ARCH-03**: Services isolated and encapsulated — no cross-service state leakage
- [x] **ARCH-04**: New services addable without modifying core BBS code

### User Experience

- [x] **UX-01**: xterm.js browser terminal with CP437 font for authentic ANSI rendering
- [x] **UX-02**: Mobile-responsive terminal — adapts to phone screens and touch keyboards
- [ ] **UX-03**: Authentic ANSI art throughout — menus, splash screens, game UI (Wildcat-era quality)
- [x] **UX-04**: Paginated output with [More] prompts (no infinite scrolling)
- [x] **UX-05**: Keyboard-driven navigation (no mouse dependency)

### Easter Eggs

- [ ] **EGGS-01**: In-world lore references to Crystal Ice Palace, Gulliver's Travels, and News Journal Center (old defunct BBS nodes, archived transmissions, legends)
- [ ] **EGGS-02**: Hidden discoveries — secret commands, ANSI art nods, hidden rooms referencing the three BBSes
- [ ] **EGGS-03**: Easter egg references scattered in door game dialogue and descriptions

## v2 Requirements

### Communication

- **COMM-01**: Multiple chat rooms / channels
- **COMM-02**: Sysop chat — users can page sysop for real-time help

### Content

- **CONT-01**: Message bases / forums (if moderation tooling sufficient)
- **CONT-02**: File areas with upload/download (if abuse prevention solved)
- **CONT-03**: ANSI art gallery — user-contributed art display

### Games

- **GAME-09**: Trade Wars 2002 clone — space trading/strategy
- **GAME-10**: Additional door games as community requests

### Features

- **FEAT-01**: QWK mail packets for offline reading
- **FEAT-02**: Voting booths / polls
- **FEAT-03**: Event calendar for game resets and maintenance

## Out of Scope

| Feature | Reason |
|---------|--------|
| Message bases / forums | Moderation nightmare in 2026 — abuse risk too high for v1 |
| File areas / upload-download | Abuse vector for illegal content — not worth the risk |
| Telnet/SSH direct connection | Web-only for v1 — browser terminal is sufficient |
| OAuth / social login | Breaks BBS immersion — traditional username/password |
| Mouse-driven UI | BBSes were keyboard-only — hotkeys and command navigation |
| Rich text / Markdown | Anachronistic — ANSI codes and plain text only |
| Infinite scrolling | Not BBS paradigm — paginated with [More] prompts |
| Real-time push notifications | Users check messages on login, like the old days |
| Native mobile app | Responsive web terminal is sufficient |
| Profile pictures | Not BBS culture — ANSI art handles only |
| DOS door game emulation | Faithful Rust clones give full control |
| Upload credit / bonus time | No file areas to incentivize |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONN-01 | Phase 2 | Pending |
| CONN-02 | Phase 2 | Pending |
| CONN-03 | Phase 2 | Pending |
| CONN-04 | Phase 2 | Pending |
| AUTH-01 | Phase 2 | Pending |
| AUTH-02 | Phase 2 | Pending |
| AUTH-03 | Phase 2 | Pending |
| AUTH-04 | Phase 2 | Pending |
| NAV-01 | Phase 3 | Pending |
| NAV-02 | Phase 3 | Pending |
| NAV-03 | Phase 3 | Pending |
| NAV-04 | Phase 3 | Pending |
| MAIL-01 | Phase 5 | Pending |
| MAIL-02 | Phase 5 | Pending |
| MAIL-03 | Phase 5 | Pending |
| MAIL-04 | Phase 5 | Pending |
| MAIL-05 | Phase 5 | Pending |
| CHAT-01 | Phase 6 | Pending |
| CHAT-02 | Phase 6 | Pending |
| CHAT-03 | Phase 6 | Pending |
| CHAT-04 | Phase 6 | Pending |
| CHAT-05 | Phase 6 | Pending |
| CHAT-06 | Phase 6 | Pending |
| GAME-01 | Phase 11 | Pending |
| GAME-02 | Phase 9 | Pending |
| GAME-03 | Phase 10 | Pending |
| GAME-04 | Phase 12 | Pending |
| GAME-05 | Phase 8 | Pending |
| GAME-06 | Phase 8 | Pending |
| GAME-07 | Phase 8 | Pending |
| GAME-08 | Phase 8 | Pending |
| USER-01 | Phase 2 | Pending |
| USER-02 | Phase 2 | Pending |
| USER-03 | Phase 2 | Pending |
| USER-04 | Phase 4 | Pending |
| USER-05 | Phase 4 | Pending |
| TIME-01 | Phase 4 | Pending |
| TIME-02 | Phase 4 | Pending |
| TIME-03 | Phase 4 | Pending |
| TIME-04 | Phase 4 | Pending |
| TIME-05 | Phase 4 | Pending |
| SYSO-01 | Phase 13 | Pending |
| SYSO-02 | Phase 13 | Pending |
| SYSO-03 | Phase 13 | Pending |
| SYSO-04 | Phase 13 | Pending |
| SYSO-05 | Phase 13 | Pending |
| SYSO-06 | Phase 13 | Pending |
| NEWS-01 | Phase 7 | Pending |
| NEWS-02 | Phase 7 | Pending |
| NEWS-03 | Phase 7 | Pending |
| ARCH-01 | Phase 1 | Complete |
| ARCH-02 | Phase 1 | Complete |
| ARCH-03 | Phase 1 | Complete |
| ARCH-04 | Phase 1 | Complete |
| UX-01 | Phase 1 | Complete |
| UX-02 | Phase 1 | Complete |
| UX-03 | Phase 3 | Pending |
| UX-04 | Phase 1 | Complete |
| UX-05 | Phase 1 | Complete |
| EGGS-01 | Phase 14 | Pending |
| EGGS-02 | Phase 14 | Pending |
| EGGS-03 | Phase 14 | Pending |

**Coverage:**
- v1 requirements: 56 total
- Mapped to phases: 56
- Unmapped: 0 ✓

---
*Requirements defined: 2026-01-26*
*Last updated: 2026-01-26 after roadmap creation*
