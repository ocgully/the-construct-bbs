# The Construct

## What This Is

A web-based Bulletin Board System (BBS) built in Rust, delivering an authentic retro BBS experience through a modern browser terminal. Inspired heavily by Wildcat BBS, The Construct recreates the full dial-up era — modem handshake sounds, ANSI art splash screens, inter-user email, live chat, and a library of faithful door game clones — all accessible from desktop or phone. Every service is a pluggable, data-driven module that the sysop can toggle on/off or extend.

## Core Value

The feeling of dialing into an exclusive, underground system — artificial scarcity, ANSI art, and social games that only work because everyone's sharing the same constrained space.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Modem handshake sound effect plays on connection
- [ ] ANSI art splash/connection screen displayed on login
- [ ] User registration and authentication system
- [ ] Main menu navigation (Wildcat-style numbered/lettered menu)
- [ ] Inter-user email (internal BBS message system)
- [ ] Single-room live chat (classic teleconference)
- [ ] Acrophobia clone — real-time multiplayer rounds with timed submissions and voting
- [ ] Legend of the Red Dragon clone — daily turns, combat, inn, PvP
- [ ] Usurper clone — medieval RPG, darker tone than LORD
- [ ] Kyrandia clone — MUD (Multi-User Dungeon)
- [ ] Drug Wars clone — buy/sell commodities across locations, random events
- [ ] Max concurrent user cap (real enforced limit — "line busy" when full)
- [ ] Daily per-user time limits (real enforced)
- [ ] Sysop admin panel — manage users, toggle services, view stats
- [ ] Pluggable/modular service architecture — all services isolated, data-driven enable/disable
- [ ] Web browser terminal frontend (xterm.js)
- [ ] Mobile-responsive terminal (adapts to phone screens and touch keyboards)
- [ ] Easter eggs referencing Crystal Ice Palace, Gulliver's Travels, and News Journal Center — woven into lore (old nodes, archived transmissions) and hidden discoveries (secret commands, ANSI art nods, game dialogue references)

### Out of Scope

- Telnet/SSH direct connection — web-only for v1
- OAuth/social login — classic username/password fits the BBS aesthetic
- Multiple chat rooms — single teleconference room captures the era
- Real-time notifications/push — users check messages when they log in, like the old days
- Native mobile app — responsive web terminal is sufficient
- Original DOS door game emulation (DOSBox) — faithful clones in Rust instead

## Context

- Inspired by Wildcat BBS specifically — menu structure, flow, and feel should reference Wildcat's design patterns
- The builder frequented Crystal Ice Palace, Gulliver's Travels, and News Journal Center — these real BBSes should be honored through in-world lore and hidden easter eggs scattered across the system and within games
- Door games (LORD, Usurper, Trade Wars, Drug Wars) were the social glue of BBS culture — faithful mechanical clones are more important than pixel-perfect visual replicas
- Acrophobia is a party game (random acronym, players submit backronyms, vote on best) — needs real-time multiplayer with enough players to be fun
- The "line busy" experience and time limits aren't just nostalgia — they create genuine scarcity that makes the space feel alive and exclusive
- Kyrandia is a MUD (Multi-User Dungeon) — distinct from the single-player door games, requiring persistent world state, real-time multi-user interaction, and room-based exploration

## Constraints

- **Tech stack**: Rust backend, xterm.js frontend, SQLite persistence — chosen for systems-level control and single-binary deployment simplicity
- **Architecture**: Every user-facing service must be a pluggable module with a common interface — sysop enables/disables via configuration, not code changes
- **Visual style**: Authentic Wildcat-era ANSI art — proper 16-color, borders, menu screens, splash art — but readable and not gratuitously animated. The real deal, not a caricature.
- **Mobile**: Terminal must be usable on phone screens with touch keyboards — same experience, responsive layout

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust backend | Systems-level language for a systems-era artifact; performance and single-binary deployment | — Pending |
| xterm.js web terminal | Accessible from any browser, no client install needed | — Pending |
| SQLite over PostgreSQL | Single-file DB matches single-server BBS model; simpler ops | — Pending |
| Faithful clones over DOS emulation | Full control over game mechanics, native Rust performance, modular architecture | — Pending |
| Single chat room | Authentic to the era — one teleconference, everyone's in it | — Pending |
| Data-driven service registry | Sysop toggles services via config; new services added without core changes | — Pending |

---
*Last updated: 2026-01-26 after initialization*
