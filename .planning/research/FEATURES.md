# Feature Landscape: BBS Systems

**Domain:** Bulletin Board Systems (Classic + Modern Web-based)
**Researched:** 2026-01-26
**Confidence:** MEDIUM (based on training knowledge of classic BBS architecture, unable to verify with current sources)

## Table Stakes

Features users expect from a BBS experience. Missing these = product feels incomplete or inauthentic.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **User Authentication** | Core security, personalization | Low | Username/password, handle/alias support |
| **Message Bases (Forums)** | Primary communication method | Medium | Threaded discussions, multiple bases/conferences |
| **File Areas** | File sharing was core BBS function | Medium | Upload/download, file tagging, descriptions |
| **Email (NetMail)** | Private user-to-user messaging | Medium | Inbox, sent items, basic threading |
| **User Profiles** | Identity and stats tracking | Low | Join date, post count, location, signature |
| **ANSI Art Display** | Visual identity of BBS culture | Medium | Splash screens, menus, color codes (CP437) |
| **Menu System** | Navigation paradigm | Low-Medium | Hierarchical menus, hotkeys, breadcrumbs |
| **Who's Online** | See other active users | Low | User list, activity status |
| **Time Limits** | Classic BBS constraint mechanism | Medium | Session timers, daily limits, warnings |
| **User Levels/Access** | Permission tiers (Sysop, Co-Sysop, User, Guest) | Medium | Feature gating, file access, message base access |
| **Bulletins/News** | System announcements | Low | Text files displayed on login or menu |
| **Last Callers List** | Recent user activity log | Low | Who called when, builds community feel |
| **User Statistics** | Calls, uploads/downloads, posts | Low | Leaderboards, activity tracking |
| **Logout/Goodbye** | Proper session termination | Low | Goodbye screens, time-used display |

## Differentiators

Features that set a BBS apart. Not expected, but highly valued. These create competitive advantage.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Door Games** | Major draw for users, high engagement | High | LORD, TradeWars, Usurper, Drug Wars, etc. |
| **Real-time Chat/Teleconference** | Synchronous multi-user chat rooms | Medium-High | Chat rooms, split-screen chat, action commands |
| **Door Game: LORD** | Iconic daily-turn RPG | High | Forest encounters, inn, skills, PvP, daily turns |
| **Door Game: Usurper** | Medieval fantasy RPG, LORD competitor | High | Kingdom building, quests, character progression |
| **Door Game: Drug Wars** | Trading/economics simulation | Medium | Commodity markets, risk/reward, loan sharks |
| **Door Game: Legend of Kyrandia** | Adventure/puzzle game | High | Inventory, puzzles, exploration, story |
| **Door Game: Acrophobia** | Party word game (multiplayer) | Medium-High | Real-time acronym challenges, scoring |
| **QWK Mail Packets** | Offline mail reader support | Medium | Packet generation, batch downloads |
| **ANSI Editor Integration** | In-system art creation | High | TheDraw-style editor, CP437 support |
| **External Protocols** | ZMODEM, XMODEM for file transfers | Medium | Protocol negotiation, resume support |
| **File Tagging** | Batch download queue | Medium | Tag files across areas, download all at once |
| **User-to-User Paging** | Real-time user interrupts | Medium | Chat requests, status (available/busy) |
| **Message Taglines** | Customizable message signatures | Low | Witty one-liners, personality |
| **Voting Booths** | Polls and surveys | Low-Medium | Question/answer tracking, results display |
| **Event Calendar** | Schedule system events, door resets | Medium | Maintenance windows, game resets, contests |
| **Sysop Chat** | User can page sysop for help | Medium | Real-time admin communication |
| **ANSI Music** | Sound through ANSI codes | Low-Medium | Nostalgic but rarely used in web context |
| **Matrix Login** | Hackery-themed login sequence | Low | Aesthetic, sets tone |
| **User-created Content** | User-uploaded ANSI art, files | Medium | Moderation needed, community building |

## Anti-Features

Features to explicitly NOT build. Common mistakes or scope creep.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Web-style Forum Threading** | Breaks BBS authenticity | Use classic linear message threading with quote replies |
| **Rich Text / Markdown** | Anachronistic, not BBS-like | Stick to ANSI codes and plain text |
| **Infinite Scrolling** | Not BBS paradigm | Paginated lists with [More] prompts |
| **Mouse-driven UI** | BBSes were keyboard-only | Hotkeys and command-line style navigation |
| **Social Media Integration** | Dilutes retro feel | Keep it self-contained |
| **Mobile Responsiveness** | Terminal emulation doesn't translate | Desktop/tablet only, fixed-width terminal |
| **Real-time Notifications** | Modern web pattern | Use login bulletins and mail waiting flags |
| **OAuth/Social Login** | Breaks immersion | Traditional username/password |
| **User Profile Pictures** | Visual clutter, not BBS culture | ANSI avatars or text-based handles only |
| **Video/Audio Embedding** | Beyond BBS capabilities | Text and ANSI art only |
| **Infinite Session Times** | Removes classic constraint | Enforce time limits (even if generous) |
| **Complex Permission Systems** | Over-engineering | Simple user levels: Guest, User, Co-Sysop, Sysop |

## Feature Deep-Dive: Door Games

Door games are complex sub-systems requiring detailed mechanics.

### LORD (Legend of the Red Dragon)

**Core Loop:**
- Daily turn limit (forest fights, typically 12-20 per day)
- Forest encounters: monsters, fairies, old hags, gems
- Character progression: experience, gold, levels, skills
- Equipment: weapons, armor upgrades at weapon shop
- The Inn: flirt, ask for stories, potentially romance NPC
- PvP combat: attack other players (death = gold/exp loss)
- Daily events: Dark Cloak, Barak's servant, random encounters
- Dragon fights at level milestones
- Resurrection and death mechanics

**Complexity:** High (state machine, turn tracking, combat system, NPC interactions)

### Usurper

**Core Loop:**
- Similar to LORD but medieval fantasy theme
- Kingdom management: peasants, soldiers, fortifications
- Quest system: dungeon crawls, boss fights
- Marriage and children mechanics
- Class system: fighter, mage, thief
- Daily turn economy

**Complexity:** High (more complex than LORD, kingdom simulation)

### Drug Wars

**Core Loop:**
- Travel between cities (Subway, prices change per location)
- Buy low, sell high commodity trading (Cocaine, Heroin, Acid, etc.)
- Random events: cops, mugging, find stash
- Loan shark debt mechanics with interest
- Limited inventory space
- Time limit (30 days)

**Complexity:** Medium (simpler than RPGs, but market simulation + events)

### Legend of Kyrandia

**Core Loop:**
- Exploration: rooms, directions, mapping
- Inventory management: collect items, use items
- Puzzle solving: use X on Y to progress
- Story progression: narrative beats
- No combat (adventure game style)

**Complexity:** High (content-heavy, story scripting, puzzle logic)

### Acrophobia

**Core Loop:**
- Multiplayer synchronous gameplay
- Round structure: reveal acronym (e.g., "BBS")
- Players submit phrase matching acronym
- Voting phase: players vote on funniest/best
- Scoring and leaderboards
- Timer-based rounds

**Complexity:** Medium-High (real-time coordination, voting system)

## Feature Deep-Dive: Email/Messaging

### BBS Email (NetMail) Characteristics

**Interface:**
- Command-driven: Read, Post, Reply, Delete, Forward
- Message list: numbered, shows From, Subject, Date
- Unread message indicators
- Message threading (RE: prefixes)

**Features:**
- Carbon copy (CC)
- Message priority flags
- Read receipts (optional)
- Message storage limits per user
- Batch delete

**Complexity:** Medium (database, message parsing, threading logic)

**Not included in BBS email:**
- Attachments (files shared via file areas instead)
- HTML formatting
- Inline images

## Feature Deep-Dive: Teleconference/Chat

### Classic BBS Chat Implementations

**Split-Screen Chat:**
- Two-user: screen divided horizontally
- Sysop chat: user requests, sysop accepts
- Line-by-line updates

**Multi-User Chat Rooms:**
- Named rooms/channels
- Join/part messages
- User list in room
- Action commands (/me waves)
- Whisper/private messages within chat
- Chat logging

**Chat Mechanics:**
- Who's in chat indicator from main menu
- Join/leave commands
- Idle timeouts
- Chat handle vs. main username

**Complexity:** Medium-High (WebSocket for real-time, room state management)

## Feature Deep-Dive: Sysop Tools

### BBS Admin Panel Capabilities

**User Management:**
- Edit user accounts: level, time bank, flags
- Delete/ban users
- View user activity logs
- Grant/revoke access to areas
- Reset passwords

**System Configuration:**
- Message base setup: create/delete, set permissions
- File area setup: paths, permissions, ratios
- Time limit policies
- User level definitions
- System name, taglines, ANSI screens

**Monitoring:**
- Who's online (with kick capability)
- System logs: login attempts, errors, uploads
- Storage usage: file areas, message counts
- User statistics: top downloaders, posters

**Maintenance:**
- Pack message bases (delete old/deleted messages)
- Purge old files
- Backup/restore
- Event scheduler (game resets, maintenance)

**Complexity:** High (comprehensive admin interface, audit logging)

## Feature Deep-Dive: Time Limits

### Time Limit Mechanics

**Implementation:**
- Daily time allowance per user level (e.g., 60 min/day for users, unlimited for sysop)
- Session timer visible in UI (e.g., status line)
- Warnings at thresholds (5 min, 1 min remaining)
- Forced logout at zero
- Time bank: save unused time for later

**Enforcement:**
- Time paused during uploads (incentive to contribute)
- Time paused during sysop chat
- Time deducted for downloads
- Door games may have separate time tracking

**User Experience:**
- Creates urgency and value
- Encourages focused sessions
- Rewards contributors (uploaders get bonus time)

**Complexity:** Medium (timer logic, state persistence, UI integration)

## Feature Deep-Dive: ANSI Art Culture

### ANSI Art Characteristics

**Technical:**
- CP437 character set (extended ASCII, box-drawing characters)
- ANSI escape codes for color (16 foreground, 8 background)
- 80-column width standard (some BBSes supported 132)
- .ANS file format

**Cultural:**
- Splash screens: BBS logo, welcome screen
- Menu headers: decorated command lists
- Scrolling text
- ANSI art galleries (file areas dedicated to art)
- Art groups: ACiD, iCE, etc.

**TheDraw:**
- DOS-based ANSI editor
- Features: brushes, fill, color selection, character palettes
- Saved as .ANS files
- Industry standard for BBS art

**Modern Web Implementation:**
- Render with xterm.js using CP437 font
- Store as .ANS files or convert to ANSI escape sequences
- Upload tool for user-contributed art
- Gallery viewer (file area browsing)

**Complexity:** Medium (ANSI parser, CP437 font, file display)

## Feature Dependencies

```
User Authentication
  ├─> User Profiles
  ├─> User Levels/Access
  └─> Message Bases (to post)
      └─> Email (NetMail)

Message Bases
  └─> User Profiles (post counts, signatures)

File Areas
  ├─> User Levels (access control)
  └─> External Protocols (optional, for transfers)

Time Limits
  ├─> User Levels (different allowances)
  └─> Time Bank (optional enhancement)

Door Games
  ├─> User Profiles (game characters tied to BBS user)
  ├─> Time Limits (may pause or have separate limits)
  └─> User Statistics (plays counted)

Real-time Chat
  ├─> Who's Online (to see who to chat with)
  └─> User-to-User Paging (to initiate)

ANSI Art Display
  ├─> Menu System (decorated menus)
  ├─> File Areas (art galleries)
  └─> Bulletins (visual announcements)

Sysop Tools
  └─> User Management
      └─> All features (admin controls everything)
```

## MVP Recommendation

For MVP, prioritize these **table stakes** features:

1. **User Authentication** - Foundation
2. **Menu System** - Navigation
3. **Message Bases** - Core communication (2-3 bases minimum)
4. **Email (NetMail)** - Private messaging
5. **File Areas** - At least 1 area with basic upload/download
6. **ANSI Art Display** - Splash screen, menu headers
7. **User Profiles** - Basic stats and info
8. **Time Limits** - Enforces classic constraint
9. **Who's Online** - Community awareness
10. **Last Callers** - Activity feed

For MVP, include **ONE differentiator** to hook users:

- **Door Game: Drug Wars** (simplest to implement, still highly engaging)
  - OR Real-time Chat (if focusing on community over games)

Defer to **post-MVP** (Phase 2+):

- **LORD** - High complexity, phase 2 priority
- **Usurper** - Similar to LORD, phase 3
- **Legend of Kyrandia** - Content-heavy, phase 3-4
- **Acrophobia** - Requires critical mass of users, phase 3
- **QWK Mail Packets** - Niche feature, phase 4
- **ANSI Editor** - Nice-to-have, phase 4
- **File Tagging** - Enhancement, phase 2
- **Voting Booths** - Enhancement, phase 3
- **Event Calendar** - Administrative feature, phase 2

## Feature Complexity Matrix

| Feature | Complexity | LOE (estimate) | Dependencies |
|---------|------------|----------------|--------------|
| User Authentication | Low | 1-2 days | None |
| Message Bases | Medium | 3-5 days | Auth, ANSI |
| Email (NetMail) | Medium | 3-4 days | Auth, Message infrastructure |
| File Areas | Medium | 4-6 days | Auth, storage strategy |
| ANSI Art Display | Medium | 2-3 days | xterm.js integration |
| Menu System | Low-Medium | 2-3 days | ANSI, navigation state |
| Time Limits | Medium | 2-3 days | Auth, session management |
| Who's Online | Low | 1 day | Auth, session tracking |
| Real-time Chat | Medium-High | 5-7 days | WebSocket, room state |
| Drug Wars (door) | Medium | 7-10 days | Auth, state machine |
| LORD (door) | High | 15-20 days | Auth, complex state machine, combat |
| Usurper (door) | High | 20-25 days | Similar to LORD + kingdom sim |
| Kyrandia (door) | High | 25-30 days | Story scripting, puzzle engine |
| Acrophobia (door) | Medium-High | 10-12 days | Real-time multiplayer, voting |
| Sysop Tools | High | 10-15 days | All features (admin panel) |

## Phasing Recommendation

**Phase 1 (MVP): Core BBS**
- Auth, menus, message bases, email, basic file area
- ANSI display, time limits, user profiles
- One simple door game (Drug Wars)

**Phase 2: Community Features**
- Real-time chat
- File tagging, enhanced file areas
- Sysop tools (basic admin panel)
- LORD (major feature)

**Phase 3: Game Expansion**
- Usurper
- Acrophobia (needs user base from phases 1-2)
- Voting booths, bulletins enhancements

**Phase 4: Advanced Features**
- Legend of Kyrandia (content-heavy)
- ANSI editor
- QWK mail packets
- Event calendar

## Sources

**Confidence Level: MEDIUM**

This research is based on training knowledge of classic BBS systems (1985-1995 era) including:
- Wildcat! BBS architecture and feature set
- LORD, Usurper, Trade Wars, Drug Wars door game mechanics
- BBS文化 (ANSI art, TheDraw, art groups)
- Modern BBS software (Mystic BBS, Synchronet, ENiGMA½) architectures

**Unable to verify with current sources:**
- Context7 and WebSearch tools were unavailable during research
- Recommendations based on historical BBS design patterns
- Modern BBS revival feature sets not confirmed with 2025-2026 sources

**Recommended validation:**
- Review Mystic BBS, Synchronet, ENiGMA½ documentation for modern implementations
- Cross-reference door game mechanics with existing source code or documentation
- Verify ANSI rendering techniques with xterm.js documentation

**Known gaps:**
- Current state of BBS revival community (2025-2026)
- Modern web-based BBS implementations and their feature choices
- Specific technical implementation details for door game state machines
