# Project Research Summary

**Project:** The Construct (Web-based BBS)
**Domain:** Bulletin Board System with modern web terminal interface
**Researched:** 2026-01-26
**Confidence:** MEDIUM

## Executive Summary

The Construct is a web-based retro Bulletin Board System (BBS) that combines classic BBS culture (ANSI art, door games, text-based navigation) with modern web technologies. Based on research, the recommended approach is a Rust backend (axum + tokio + sqlx/SQLite) serving a browser-based terminal emulator (xterm.js) over WebSocket. This architecture provides authentic BBS experience while enabling mobile access and modern deployment.

The technical foundation requires careful handling of ANSI/CP437 encoding (critical for authentic art rendering), WebSocket escape sequence framing (to prevent garbled terminal output), and async runtime management (to avoid blocking under multiplayer load). The architecture centers on a trait-based service system where each BBS feature (menus, games, chat) implements a common interface, enabling modular development and clean separation of concerns.

Key risk mitigation: (1) address CP437 encoding in Phase 1 or all ANSI art breaks, (2) design SQLite concurrency strategy upfront using WAL mode and write queues to prevent multiplayer game contention, (3) implement mobile virtual keyboard handling by Phase 2 to avoid excluding 50%+ of users, and (4) resist nostalgia-driven scope creep by strictly defining MVP around 3 door games and core messaging.

## Key Findings

### Recommended Stack

The stack emphasizes type safety, async-first design, and proven WebSocket/terminal patterns. Rust provides memory safety and concurrency without garbage collection overhead, critical for handling hundreds of idle WebSocket connections efficiently.

**Core technologies:**
- **axum 0.7.x + tokio 1.x**: HTTP/WebSocket server with industry-standard async runtime — best ergonomics for WebSocket upgrades and minimal boilerplate
- **xterm.js 5.x + addons**: Browser terminal emulator — only viable option for full ANSI/VT100 support with mobile touch handlers and WebGL performance
- **sqlx 0.7.x + SQLite**: Async database with compile-time query verification — safer than rusqlite (blocking I/O), more flexible than diesel (overkill for SQLite)
- **vite 5.x**: Frontend bundler — simpler than trunk (no WASM needed), faster dev experience than webpack
- **Web Audio API**: Modem sounds and retro audio — native browser support, no library needed for simple samples

**Architecture approach:** Trait-based service plugin system with static registration (compile-time), not dynamic loading. Each BBS feature (door game, menu, chat) implements the `Service` trait with `on_enter`, `on_input`, `on_exit` methods. Service context provides dependency injection (database, ANSI writer, session info) without tight coupling.

### Expected Features

**Must have (table stakes):**
- User authentication (username/password, handle/alias)
- Message bases (forums with threaded discussions)
- Email/NetMail (private user-to-user messaging)
- ANSI art display (splash screens, menus, CP437 support)
- Menu system (hierarchical navigation, hotkeys)
- Time limits (session timers, daily caps, warnings)
- User profiles (join date, stats, post counts)
- Who's online (active user list)
- Last callers list (activity log)
- File areas (upload/download with descriptions)

**Should have (competitive advantages):**
- Door games (LORD, Drug Wars, Usurper — major engagement drivers)
- Real-time chat (multi-user synchronous chat rooms)
- User-to-user paging (chat requests, status indicators)
- QWK mail packets (offline reader support)
- Sysop chat (real-time admin communication)
- File tagging (batch download queue)

**Defer (v2+):**
- Legend of Kyrandia (content-heavy adventure game — Phase 4)
- Acrophobia (multiplayer party game, needs user critical mass — Phase 3)
- ANSI editor integration (TheDraw-style editor — Phase 4)
- External protocols (ZMODEM/XMODEM — Phase 4, niche)
- Event calendar (administrative tool — Phase 3)
- Voting booths (polls/surveys — Phase 3)

**Anti-features (explicitly avoid):**
- Web-style forum threading (breaks BBS authenticity)
- Rich text/Markdown (use ANSI codes only)
- Infinite scrolling (use paginated [More] prompts)
- Mouse-driven UI (keyboard/hotkey navigation)
- OAuth/social login (traditional credentials only)
- Profile pictures (ANSI avatars or text handles only)

### Architecture Approach

The architecture follows a layered service model where WebSocket connections spawn tokio tasks that own session state and route input to active services. Services are stateless trait objects instantiated per-session, with all persistent state in SQLite and transient state in session context.

**Major components:**
1. **WebSocket Handler** — connection lifecycle, message framing, heartbeat detection
2. **Session Manager** — authentication, time limits, concurrent user caps, idle timeout enforcement
3. **Service Router** — dispatches input to active service, manages service transitions (login → menu → game)
4. **Service Registry** — static trait object factory with config-driven enable/disable
5. **ANSI Writer** — buffered terminal output with escape code generation, prevents malformed sequences
6. **Database Layer** — SQLite via sqlx with WAL mode, compile-time query verification, per-game state tables

**Data flow:** Browser keypress → WebSocket binary message → Session Manager (validates) → Service Router (identifies active service) → Service.on_input() → ANSI Writer (buffers output) → WebSocket flush → xterm.js renders.

**Service pattern:** Each service (LoginService, MainMenuService, LordGameService) implements `Service` trait with context injection. ServiceContext provides mutable ANSI writer, immutable database pool, user/session info. Services return `ServiceTransition` enum (Continue, Switch, Disconnect) to control flow.

**Game state storage:** JSON blob approach for rapid prototyping (`game_states` table with JSON column), migrate to structured tables for complex games (e.g., `lord_players` with typed columns). Trade-off: JSON is flexible but harder to query; structured is type-safe but requires migrations.

### Critical Pitfalls

1. **ANSI Art Character Set Mismatch (CP437 vs Unicode)** — Classic BBS art uses CP437 encoding; modern browsers use UTF-8. Without proper CP437-to-Unicode mapping and font, box-drawing characters render broken. **Mitigation:** Use CP437 mapping table (0xB0-0xDF range), load "Perfect DOS VGA 437" font, convert byte values to Unicode codepoints before rendering. Test with actual BBS ANSI files from 16colo.rs. **Phase 1 critical.**

2. **WebSocket Escape Sequence Splitting** — ANSI escape codes (e.g., `\x1b[31m` for red color) split across WebSocket frame boundaries cause garbled output, wrong colors, cursor misplacement. **Mitigation:** Buffer escape sequences server-side in complete units, use framing protocol (length-prefixed messages), flush only on complete lines or 16ms timeout. Test with throttled network. **Phase 1 critical.**

3. **SQLite Write Contention in Multiplayer Games** — SQLite allows one writer at a time; concurrent door game writes cause `SQLITE_BUSY` errors, lost progress, lag spikes. **Mitigation:** Enable WAL mode (`PRAGMA journal_mode=WAL`), batch writes (flush every 5-10 seconds), use actor pattern (single task owns writes, others send messages via channel), read from in-memory cache. **Phase 1 architecture decision.**

4. **Rust Async Runtime Blocking in Game Loops** — Blocking I/O or CPU-heavy computations in async functions starve Tokio runtime, freezing all connections. **Mitigation:** Use `tokio::task::spawn_blocking` for all blocking I/O (SQLite via rusqlite), yield with `tokio::task::yield_now()` every 1ms in tight loops, run door games on dedicated runtime. Profile with tokio-console. **Phase 2 enforcement.**

5. **Mobile Virtual Keyboard Blocking Terminal** — On mobile, virtual keyboard covers 50%+ of screen, users can't see typed text or terminal output. xterm.js doesn't auto-handle mobile keyboard resize. **Mitigation:** Listen for `visualViewport` resize events, use floating input field above keyboard, consider hybrid chat-style UI for mobile. Test on real iOS Safari and Android Chrome, not emulators. **Phase 2 requirement.**

6. **Over-Engineered Plugin System** — Complex trait hierarchies with async trait objects and dynamic loading create confusing APIs, compile-time overhead, and maintenance burden. **Mitigation:** Start with hardcoded games, extract common trait after 2-3 concrete implementations. Use static dispatch and small API surface (3-5 methods max). Defer plugin system until Phase 3+ with proven games. **Resist premature abstraction.**

7. **Time Limit Enforcement Race Conditions** — Sessions expire mid-game or mid-message without cleanup, causing lost progress, half-written data, user frustration. **Mitigation:** Warn at 5min/1min remaining, let current operation finish before timeout, use `tokio::select!` with cleanup on both paths, auto-save state every 30 seconds. **Phase 2 time system.**

## Implications for Roadmap

Based on research, suggested phase structure balances architectural foundation, user value delivery, and pitfall avoidance:

### Phase 1: Terminal Foundation & Core Infrastructure
**Rationale:** Must establish ANSI rendering, WebSocket I/O, and data layer correctly before building features. CP437 encoding and escape sequence framing are foundational — errors here break everything else.

**Delivers:**
- xterm.js terminal with CP437 font and Unicode mapping
- WebSocket handler with escape sequence buffering
- SQLite database with WAL mode and migration system
- Service trait interface and basic echo service
- Session manager with time limit skeleton

**Addresses (from FEATURES.md):**
- ANSI art display foundation
- User authentication schema (tables only)

**Avoids (from PITFALLS.md):**
- Pitfall #1 (CP437 encoding) — addressed with mapping table and font
- Pitfall #2 (escape sequence splitting) — buffered ANSI writer
- Pitfall #3 (SQLite concurrency) — WAL mode + write queue architecture
- Pitfall #13 (escape injection) — input sanitization layer
- Pitfall #17 (missing migrations) — sqlx migration setup

**Research flag:** LOW — xterm.js and axum are well-documented with standard patterns.

### Phase 2: Authentication, Navigation & Session Management
**Rationale:** Builds on terminal foundation to create usable BBS shell. Session management and time limits are table stakes that affect all subsequent features.

**Delivers:**
- Login service with password hashing
- Main menu service with hierarchical navigation
- User profiles and stats tracking
- Time limit enforcement with grace warnings
- Who's online / last callers lists
- Logout/goodbye screens

**Uses (from STACK.md):**
- sqlx for user account queries (compile-time verified)
- chrono for login tracking and time calculations
- ANSI Writer for menu decoration

**Implements (from ARCHITECTURE.md):**
- ServiceRegistry with config-driven enable/disable
- ServiceRouter with transition handling
- ServiceContext dependency injection

**Avoids (from PITFALLS.md):**
- Pitfall #7 (time limit race conditions) — graceful shutdown with auto-save
- Pitfall #14 (session state leak) — WebSocket close handlers and heartbeat

**Research flag:** LOW — standard web app patterns (auth, session management).

### Phase 3: Messaging & Communication
**Rationale:** Core BBS value proposition. Message bases and email are table stakes; real-time chat is differentiator. Can be built in parallel with Phase 4.

**Delivers:**
- Message bases (forums) with threading
- NetMail (private messaging) with inbox/sent
- Real-time chat rooms (WebSocket broadcast)
- User-to-user paging
- Bulletins/news system

**Addresses (from FEATURES.md):**
- Message bases (table stakes)
- Email/NetMail (table stakes)
- Real-time chat (differentiator)
- User-to-user paging (differentiator)

**Uses (from STACK.md):**
- `tokio::sync::broadcast` channel for chat fan-out
- SQLite JSON columns for flexible message storage

**Avoids (from PITFALLS.md):**
- Pitfall #11 (stale state in multiplayer) — WebSocket push model
- Pitfall #13 (escape injection) — sanitize chat messages

**Research flag:** LOW — standard messaging patterns, broadcast channels well-documented.

### Phase 4: File Areas
**Rationale:** Table stakes feature but lower priority than messaging. Simpler than games, can be developed in parallel.

**Delivers:**
- File upload/download with descriptions
- File tagging for batch downloads
- File area permissions by user level
- Upload credit system (bonus time)

**Addresses (from FEATURES.md):**
- File areas (table stakes)
- File tagging (differentiator)

**Avoids (from PITFALLS.md):**
- Pitfall #5 (async blocking) — use `spawn_blocking` for file I/O

**Research flag:** LOW — standard file handling patterns.

### Phase 5: First Door Game (Drug Wars)
**Rationale:** Simplest door game validates service architecture and game state patterns. Tests async game loops and database integration. Provides early user engagement.

**Delivers:**
- Drug Wars game (commodity trading, random events)
- Turn-based game state framework
- Daily turn reset logic
- Game state persistence (JSON blob)

**Uses (from STACK.md):**
- `tokio::time::interval` for game ticks
- rand crate for random events
- SQLite `game_states` table

**Implements (from ARCHITECTURE.md):**
- TradingGameState pattern
- Game service with state machine

**Avoids (from PITFALLS.md):**
- Pitfall #4 (async blocking) — spawn_blocking for DB writes
- Pitfall #19 (game balance vs time limits) — 10-20 minute sessions
- Pitfall #6 (over-engineering) — hardcode first, extract later

**Research flag:** MEDIUM — Game mechanics need balancing, turn economy needs design.

### Phase 6: Advanced Door Game (LORD)
**Rationale:** High-value differentiator, complex state machine. Builds on Drug Wars patterns but requires combat system, NPC interactions, PvP.

**Delivers:**
- LORD clone (forest fights, inn, PvP, dragon boss)
- Turn-based RPG framework
- Combat system with stats/equipment
- Daily turn economy

**Uses (from STACK.md):**
- Structured `lord_players` and `lord_monsters` tables
- Complex state machine

**Implements (from ARCHITECTURE.md):**
- TurnBasedRPG pattern
- Separate model (game logic) and view (ANSI rendering)

**Avoids (from PITFALLS.md):**
- Pitfall #4 (async blocking) — tight game loop yields frequently
- Pitfall #6 (mixing UI and logic) — separate BattleResult from rendering

**Research flag:** HIGH — Complex game mechanics, balance testing, requires `/gsd:research-phase` for combat system and progression.

### Phase 7: Real-Time Multiplayer Door Game (Acrophobia)
**Rationale:** Requires user critical mass from earlier phases. Tests broadcast system and real-time coordination.

**Delivers:**
- Acrophobia (multiplayer acronym game)
- Real-time game loop with phase transitions
- Voting system and leaderboards
- Room-based multiplayer coordination

**Uses (from STACK.md):**
- `tokio::sync::broadcast` for room events
- Timer-based round structure

**Implements (from ARCHITECTURE.md):**
- RealTimeMultiplayer pattern
- Broadcast channel subscription

**Avoids (from PITFALLS.md):**
- Pitfall #11 (state staleness) — event-driven updates
- Pitfall #4 (async blocking) — timer ticks yield properly

**Research flag:** MEDIUM — Real-time game coordination patterns, voting mechanics.

### Phase 8: Sysop Tools & Administration
**Rationale:** Deferred until core features proven. Enables operational management and moderation.

**Delivers:**
- Admin panel (user management, system config)
- Audit logging
- User ban/kick capabilities
- Message base setup and moderation
- System monitoring (logs, storage)

**Addresses (from FEATURES.md):**
- Sysop tools (table stakes for operation)

**Research flag:** LOW — Standard admin panel patterns.

### Phase Ordering Rationale

**Why this order:**
- Phases 1-2 are foundation — nothing works without terminal I/O, sessions, and auth
- Phase 3 (messaging) and Phase 4 (files) can be parallel — both table stakes, no dependencies
- Phase 5 (Drug Wars) validates game architecture before complex LORD (Phase 6)
- Phase 6 (LORD) is highest-value door game, justifies earlier phases
- Phase 7 (Acrophobia) needs user base from Phases 1-6 to be viable
- Phase 8 (admin) deferred until operational need proven

**Dependency-driven grouping:**
- Terminal foundation (Phase 1) blocks everything
- Session/auth (Phase 2) blocks user-facing features
- First game (Phase 5) blocks game framework extraction
- Complex game (Phase 6) requires simple game validation (Phase 5)

**Pitfall avoidance:**
- Phases 1-2 address all critical pitfalls (#1-3, #7) before feature work
- Phase 5 catches async blocking issues (#4) before complex LORD
- Mobile strategy (#5) forced by Phase 2 before public launch
- Plugin abstraction (#6) deferred until Phase 6+ with concrete patterns

### Research Flags

**Phases needing deeper research during planning:**
- **Phase 6 (LORD):** Complex game mechanics, combat balancing, progression curves — run `/gsd:research-phase` for RPG design patterns
- **Phase 7 (Acrophobia):** Real-time multiplayer coordination, voting system, timer synchronization — research game loop timing patterns

**Phases with standard patterns (skip research-phase):**
- **Phase 1:** xterm.js integration well-documented, axum WebSocket examples abundant
- **Phase 2:** Standard auth/session patterns, tokio time handling established
- **Phase 3:** Messaging is CRUD + broadcast channel (tokio docs cover this)
- **Phase 4:** File handling is standard Rust async I/O
- **Phase 5:** Simple state machine, straightforward economy simulation

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | MEDIUM | axum/tokio/sqlx versions from training data (Jan 2025), need verification for Jan 2026. xterm.js is stable and established. Architecture patterns proven. |
| Features | MEDIUM | Based on classic BBS feature sets (well-understood) and modern web BBS projects (Mystic, Synchronet, ENiGMA½). Table stakes vs differentiators derived from BBS culture knowledge. |
| Architecture | HIGH | Trait-based service system is idiomatic Rust, WebSocket+terminal pattern proven by ttyd/Wetty, tokio async patterns well-established. Component boundaries clear. |
| Pitfalls | MEDIUM | CP437/ANSI issues are documented in terminal emulation literature. SQLite concurrency well-understood. Mobile keyboard issues known from xterm.js GitHub. Async blocking is common Rust pitfall. |

**Overall confidence:** MEDIUM

### Gaps to Address

**Version verification required:**
- Exact current versions of axum (0.7.x?), sqlx (0.7.x?), tokio (1.x?) as of Jan 2026
- xterm.js version and addon package names (may have moved to `@xterm/` scope)
- Verify with `cargo search` and `npm view` before Phase 1

**Mobile keyboard best practices:**
- xterm.js mobile handling has evolved — check official docs and GitHub issues for 2025-2026 solutions
- Test `visualViewport` API on latest iOS Safari and Android Chrome
- Validate during Phase 2 planning, before committing to approach

**CP437 rendering:**
- Verify best CP437 font choice for web (Perfect DOS VGA 437 vs alternatives)
- Test Unicode vs embedded font approach in modern browsers
- Validate during Phase 1 spike

**SQLite at scale:**
- Current WAL mode tuning recommendations for concurrent WebSocket writes
- Connection pool sizing for 100+ concurrent users
- Research during Phase 1 if targeting >100 users initially

**Door game mechanics:**
- LORD combat formulas, balance curves (if aiming for authentic clone)
- Drug Wars market simulation parameters
- Research during Phase 5-6 planning, not earlier

**Nostalgia scope creep:**
- Define strict MVP: Phases 1-6 only for v1.0
- Defer Usurper, Kyrandia, QWK packets, ANSI editor to v2+
- Validate feature priority with potential users during Phase 3-4

## Sources

### Primary (MEDIUM confidence)
- **Rust Ecosystem (Tokio, axum, sqlx):** Training knowledge through Jan 2025 — patterns and APIs are stable but versions need verification
- **xterm.js Documentation:** Training knowledge of terminal emulation standards and xterm.js capabilities — mobile improvements may exist as of 2026
- **Classic BBS Architecture:** Training knowledge of Wildcat!, Mystic, Synchronet — well-established patterns from 1985-2025
- **Door Game Mechanics:** Training knowledge of LORD, Trade Wars, Usurper, Drug Wars — mechanics are frozen (retro games)

### Secondary (MEDIUM confidence)
- **SQLite Concurrency Model:** Training knowledge of WAL mode, locking behavior — well-documented and stable since SQLite 3.7 (2010)
- **WebSocket Protocol:** Training knowledge of browser WebSocket API and tokio-tungstenite — stable standard since 2011
- **Web Audio API:** Training knowledge of browser audio capabilities — stable standard, autoplay policies from 2018-2020

### Tertiary (LOW confidence, needs validation)
- **xterm.js Mobile Keyboard (2025-2026):** May have improved since training cutoff — check GitHub issues and official docs
- **axum 0.7 API Stability:** Version number from training data — verify current stable version on crates.io
- **Modern BBS Revival (2025-2026):** Community activity and feature preferences unknown post-training — validate with ENiGMA½, Synchronet communities if targeting existing BBS users

**Limitation:** WebSearch unavailable during research — all findings based on training data through Jan 2025. Version numbers, mobile improvements, and 2025-2026 BBS community trends require verification.

**Verification steps before Phase 1:**
1. Check crates.io: `cargo search axum`, `cargo search sqlx`, `cargo search tokio`
2. Check npm: `npm view xterm version`, `npm view @xterm/addon-fit version`
3. Review xterm.js GitHub issues tagged `mobile` or `keyboard` from 2025-2026
4. Test CP437 font rendering on Chrome/Safari/Firefox with sample ANSI files
5. Join /r/bbs or ENiGMA½ Discord to validate feature priorities with existing BBS community

---
*Research completed: 2026-01-26*
*Ready for roadmap: yes*
