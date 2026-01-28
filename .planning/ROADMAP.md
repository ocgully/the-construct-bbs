# Roadmap: The Construct

## Overview

The Construct delivers an authentic BBS experience through a browser terminal, built from foundation to games. Starting with robust ANSI rendering and WebSocket infrastructure, we layer authentication, navigation, and communication features before culminating in five faithful door game clones. Every phase delivers a complete, verifiable capability that builds toward the core value: the feeling of dialing into an exclusive, underground system with artificial scarcity, ANSI art, and social games.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Terminal Foundation** - ANSI rendering, WebSocket I/O, service architecture
- [x] **Phase 2: Authentication & Connection** - User accounts, login/logout, connection experience
- [x] **Phase 3: Navigation System** - Wildcat-style menus with ANSI art and hotkeys
- [x] **Phase 4: Time Limits & User Lists** - Daily time caps, session timers, who's online
- [ ] **Phase 5: Email System** - Inter-user private messaging
- [ ] **Phase 6: Chat & Real-Time Communication** - Live teleconference, user paging
- [ ] **Phase 7: News & Bulletins** - RSS feed integration, sysop bulletins
- [ ] **Phase 8: First Door Game (Drug Wars)** - Commodity trading game, validates game architecture
- [ ] **Phase 9: Second Door Game (LORD)** - Complex RPG with combat, PvP, dragon boss
- [ ] **Phase 10: Third Door Game (Usurper)** - Medieval RPG, darker theme
- [ ] **Phase 11: Multiplayer Door Game (Acrophobia)** - Real-time party game
- [ ] **Phase 12: MUD Door Game (Kyrandia)** - Multi-User Dungeon
- [ ] **Phase 13: Sysop Administration** - Admin panel, user management, system config
- [ ] **Phase 14: Easter Eggs & Polish** - Hidden lore, secret commands, final polish

## Phase Details

### Phase 1: Terminal Foundation
**Goal**: Browser terminal correctly renders authentic ANSI art and provides pluggable service architecture
**Depends on**: Nothing (first phase)
**Requirements**: UX-01, UX-02, UX-04, UX-05, ARCH-01, ARCH-02, ARCH-03, ARCH-04
**Success Criteria** (what must be TRUE):
  1. User can view CP437 ANSI art with correct box-drawing characters and colors in browser terminal
  2. Terminal adapts to mobile screen sizes with functional touch keyboard input
  3. Terminal displays paginated output with [More] prompts (no infinite scrolling)
  4. All user input is keyboard-driven without mouse dependency
  5. Service architecture supports pluggable modules that can be enabled/disabled via configuration
**Plans**: 5 plans

Plans:
- [x] 01-01-PLAN.md -- Rust backend scaffolding, config system, service trait + registry
- [x] 01-02-PLAN.md -- Terminal output engine (AnsiWriter, CP437 conversion, paging)
- [x] 01-03-PLAN.md -- Frontend terminal (xterm.js, CRT effects, mobile, WebSocket client)
- [x] 01-04-PLAN.md -- Backend WebSocket handler, session management, service routing
- [x] 01-05-PLAN.md -- Integration, ANSI art welcome screen, visual verification

### Phase 2: Authentication & Connection
**Goal**: Users can create accounts, log in securely, and experience authentic BBS connection sequence
**Depends on**: Phase 1
**Requirements**: AUTH-01, AUTH-02, AUTH-03, AUTH-04, CONN-01, CONN-02, CONN-03, CONN-04, USER-01, USER-02, USER-03
**Success Criteria** (what must be TRUE):
  1. User hears modem handshake sound when connecting to BBS
  2. User sees ANSI art splash screen during connection sequence
  3. User can register new account with username and password
  4. User can log in with existing credentials and session persists across page refresh
  5. User receives "line busy" rejection when max concurrent users reached
  6. User sees goodbye screen with session stats on logout
  7. User profile displays name, join date, location, signature with tracked stats
**Plans**: 7 plans

Plans:
- [x] 02-01-PLAN.md -- Database foundation (SQLx, schema, User CRUD) and config extensions
- [x] 02-02-PLAN.md -- Auth core (Argon2 password hashing, session tokens, validation, profanity filter) and NodeManager
- [x] 02-03-PLAN.md -- Connection ceremony (modem audio, typewriter text, ANSI splash, line-busy rejection)
- [x] 02-04-PLAN.md -- Registration service (interactive terminal prompts, email verification, handle validation)
- [x] 02-05-PLAN.md -- Login flow (handle/password prompts, session token persistence, session resumption)
- [x] 02-06-PLAN.md -- User profile ANSI card and goodbye sequence with session stats
- [x] 02-07-PLAN.md -- Flow integration, end-to-end wiring, and human verification

### Phase 3: Navigation System
**Goal**: Users can navigate BBS using Wildcat-style numbered/lettered menus with ANSI art
**Depends on**: Phase 2
**Requirements**: NAV-01, NAV-02, NAV-03, NAV-04, UX-03
**Success Criteria** (what must be TRUE):
  1. User sees main menu with numbered/lettered options in Wildcat style
  2. User can navigate hierarchical menu structure with breadcrumbs
  3. User can use hotkeys for rapid menu traversal
  4. All menu screens display authentic ANSI art headers and borders
**Plans**: 3 plans

Plans:
- [x] 03-01-PLAN.md -- Menu config schema, TOML menu definitions, and Stoic quotes module
- [x] 03-02-PLAN.md -- Menu state machine, type-ahead buffer, and ANSI rendering
- [x] 03-03-PLAN.md -- Session integration, legacy cleanup, and end-to-end verification

### Phase 4: Time Limits & User Lists
**Goal**: BBS enforces daily time limits and displays active/recent users
**Depends on**: Phase 3
**Requirements**: TIME-01, TIME-02, TIME-03, TIME-04, TIME-05, USER-04, USER-05
**Success Criteria** (what must be TRUE):
  1. User sees session timer countdown in status line
  2. User receives warnings at 5-minute and 1-minute remaining
  3. User is gracefully logged out at zero time with auto-saved state
  4. User can save unused daily minutes to time bank for future sessions
  5. User can view who's online from menu
  6. User can view last callers list showing recent login history
  7. User can view other users' profiles
**Plans**: 6 plans

Plans:
- [x] 04-01-PLAN.md -- Database schema extensions, time limits config, NodeManager idle tracking
- [x] 04-02-PLAN.md -- Backend session timer task with CancellationToken and timeout goodbye
- [x] 04-03-PLAN.md -- Frontend status bar renderer and client-side countdown timer
- [x] 04-04-PLAN.md -- Who's Online and Last Callers ANSI display services
- [x] 04-05-PLAN.md -- User profile lookup service and menu item registration
- [x] 04-06-PLAN.md -- Timer lifecycle integration, timeout enforcement, command routing

### Phase 5: Email System
**Goal**: Users can send and receive private messages to other BBS users
**Depends on**: Phase 4
**Requirements**: MAIL-01, MAIL-02, MAIL-03, MAIL-04, MAIL-05
**Success Criteria** (what must be TRUE):
  1. User can send private message to another user by username
  2. User can read inbox with clear unread indicators
  3. User can reply to received messages
  4. User can delete messages from inbox
  5. User sees "You have new mail" notification on login
**Plans**: 4 plans

Plans:
- [ ] 05-01-PLAN.md -- Messages database table, CRUD operations, and mail config
- [ ] 05-02-PLAN.md -- Mail ANSI rendering functions and compose state machine
- [ ] 05-03-PLAN.md -- Session integration with sentinel handlers and login notification
- [ ] 05-04-PLAN.md -- Status bar MAIL indicator and menu config wiring

### Phase 6: Chat & Real-Time Communication
**Goal**: Users can participate in live chat room and page other users
**Depends on**: Phase 5
**Requirements**: CHAT-01, CHAT-02, CHAT-03, CHAT-04, CHAT-05, CHAT-06
**Success Criteria** (what must be TRUE):
  1. User can enter single-room teleconference chat
  2. User sees messages from all users in real-time
  3. User sees join/leave announcements when users enter/exit chat
  4. User can use action commands like /me waves
  5. User can page another online user to request private chat
  6. User can view who's online list from chat and main menu
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 7: News & Bulletins
**Goal**: Users see current news and sysop bulletins on login or from menu
**Depends on**: Phase 6
**Requirements**: NEWS-01, NEWS-02, NEWS-03
**Success Criteria** (what must be TRUE):
  1. User sees news feed on login or can access from menu
  2. News is sourced from configurable RSS feed (world news)
  3. Sysop-posted bulletins appear alongside RSS news
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 8: First Door Game (Drug Wars)
**Goal**: Users can play Drug Wars clone with state persistence and leaderboards
**Depends on**: Phase 7
**Requirements**: GAME-05, GAME-06, GAME-07, GAME-08
**Success Criteria** (what must be TRUE):
  1. User can launch Drug Wars from main menu
  2. User can buy/sell commodities across multiple locations
  3. User encounters random events (police, mugging, deals)
  4. User game state persists between sessions (save/resume works)
  5. User can view leaderboard showing top players
  6. User game completes after 30-day in-game limit
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 9: Second Door Game (LORD)
**Goal**: Users can play Legend of the Red Dragon clone with daily turns and PvP
**Depends on**: Phase 8
**Requirements**: GAME-02
**Success Criteria** (what must be TRUE):
  1. User can launch LORD from main menu
  2. User can fight monsters in forest with turn-based combat
  3. User can visit inn for healing and social interactions
  4. User can engage in PvP combat with other players
  5. User progresses toward defeating dragon boss
  6. User daily turn limit resets at midnight
  7. User game state persists between sessions
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 10: Third Door Game (Usurper)
**Goal**: Users can play Usurper clone with medieval kingdom theme
**Depends on**: Phase 9
**Requirements**: GAME-03
**Success Criteria** (what must be TRUE):
  1. User can launch Usurper from main menu
  2. User can manage medieval kingdom with quests
  3. User can choose character class affecting gameplay
  4. User experiences darker tone than LORD (theme and narrative)
  5. User daily turn limit enforced
  6. User game state persists between sessions
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 11: Multiplayer Door Game (Acrophobia)
**Goal**: Users can play real-time multiplayer Acrophobia with timed rounds
**Depends on**: Phase 10
**Requirements**: GAME-01
**Success Criteria** (what must be TRUE):
  1. User can launch Acrophobia from main menu and join active round
  2. User sees random acronym and submits backronym within time limit
  3. User votes on best submissions from other players
  4. User sees real-time updates as other players submit and vote
  5. User sees round winners and scores
  6. User game requires minimum players to start round
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 12: MUD Door Game (Kyrandia)
**Goal**: Users can play Kyrandia MUD (Multi-User Dungeon)
**Depends on**: Phase 11
**Requirements**: GAME-04
**Success Criteria** (what must be TRUE):
  1. User can launch Kyrandia MUD from main menu
  2. User can explore persistent world with room-based navigation
  3. User can interact with other players in real-time
  4. User can engage in MUD-style combat and social interaction
  5. User game state persists between sessions
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 13: Sysop Administration
**Goal**: Sysop can manage users, toggle services, and monitor system from admin panel
**Depends on**: Phase 12
**Requirements**: SYSO-01, SYSO-02, SYSO-03, SYSO-04, SYSO-05, SYSO-06
**Success Criteria** (what must be TRUE):
  1. Sysop can access admin panel from main menu (Sysop level only)
  2. Sysop can edit user accounts, reset passwords, ban/unban users
  3. Sysop can toggle services on/off without code changes
  4. Sysop can view system stats (active users, total accounts, uptime)
  5. Sysop can kick active users from the system
  6. Sysop can configure time limits and concurrent user caps
**Plans**: TBD

Plans:
- [ ] TBD during planning

### Phase 14: Easter Eggs & Polish
**Goal**: BBS contains hidden lore references and final polish for authentic experience
**Depends on**: Phase 13
**Requirements**: EGGS-01, EGGS-02, EGGS-03
**Success Criteria** (what must be TRUE):
  1. User discovers in-world lore references to Crystal Ice Palace, Gulliver's Travels, and News Journal Center
  2. User finds hidden commands, ANSI art nods, or secret rooms referencing the three BBSes
  3. User encounters easter egg references in door game dialogue and descriptions
  4. BBS feels cohesive and polished with authentic retro atmosphere
**Plans**: TBD

Plans:
- [ ] TBD during planning

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11 -> 12 -> 13 -> 14

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Terminal Foundation | 5/5 | Complete | 2026-01-26 |
| 2. Authentication & Connection | 7/7 | Complete | 2026-01-27 |
| 3. Navigation System | 3/3 | Complete | 2026-01-27 |
| 4. Time Limits & User Lists | 6/6 | Complete | 2026-01-28 |
| 5. Email System | 0/4 | Not started | - |
| 6. Chat & Real-Time Communication | 0/TBD | Not started | - |
| 7. News & Bulletins | 0/TBD | Not started | - |
| 8. First Door Game (Drug Wars) | 0/TBD | Not started | - |
| 9. Second Door Game (LORD) | 0/TBD | Not started | - |
| 10. Third Door Game (Usurper) | 0/TBD | Not started | - |
| 11. Multiplayer Door Game (Acrophobia) | 0/TBD | Not started | - |
| 12. MUD Door Game (Kyrandia) | 0/TBD | Not started | - |
| 13. Sysop Administration | 0/TBD | Not started | - |
| 14. Easter Eggs & Polish | 0/TBD | Not started | - |

---
*Roadmap created: 2026-01-26*
*Last updated: 2026-01-28 (Phase 5 planned)*
