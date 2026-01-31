# Game Analysis: Summit, Tanks: Blitzkrieg, Depths of Diablo, Acromania

**Research Date**: 2026-01-30
**Analyst**: Claude Code Agent
**Reference Implementation**: `backend/src/games/grand_theft_meth/`

---

## Executive Summary

This document analyzes four multiplayer games slated for implementation on The Construct BBS. All four require real-time synchronization capabilities not present in the reference implementation (Grand Theft Meth is single-player turn-based). This represents a significant architectural expansion.

| Game | Type | Players | Complexity | Est. Effort |
|------|------|---------|------------|-------------|
| Summit | Co-op Survival | 1-4 | Moderate | 2-3 weeks |
| Tanks: Blitzkrieg | Real-time Combat | 2-6 | Complex | 4-6 weeks |
| Depths of Diablo | Co-op Roguelite | 1-4 | Complex | 4-6 weeks |
| Acromania | Party Word Game | 3-16 | Moderate | 1-2 weeks |

---

## 1. Summit

### Summary
A cooperative climbing survival game where 1-4 scouts must reach the summit of a procedurally generated mountain that changes daily. Features stamina management, questionable food items with risk/reward mechanics, co-op climbing aids (ropes, pitons), and 4 distinct biomes with escalating hazards.

### Key Implementation Questions

#### Real-Time Sync
1. **WebSocket message rate**: Spec shows WASD movement - what's the expected input rate? 10 inputs/second? 30?
2. **Position interpolation**: Do we smooth player positions or show raw updates?
3. **Latency tolerance**: What's acceptable lag? 100ms? 500ms? This affects gameplay feel significantly.

#### Matchmaking & Session
4. **Who can join?**: Can strangers join a run, or friends-only via invite code?
5. **Mid-game join**: Can players join after a run starts (beach biome only? any campfire?)
6. **Solo mode**: Is single-player officially supported or always co-op with empty slots?

#### Daily Mountain
7. **Seed distribution**: Same mountain for everyone worldwide, or per-timezone?
8. **When does day reset?**: Midnight UTC? Local time? BBS server time?
9. **Replay prevention**: Can you re-run today's mountain after reaching summit?

#### Mechanics Clarification
10. **Helping up ledges**: Is this automatic when adjacent, or requires button press from both?
11. **Item sharing distance**: Drop items anywhere, or must be adjacent to share?
12. **Downed state**: How long until permadeath? Can you crawl?
13. **Revival at campfire**: Does this consume a resource or free?

#### Disconnection Handling
14. **Player disconnects mid-climb**: AI takeover? Removed from game? Frozen in place?
15. **Rejoin**: Can disconnected player rejoin same run?
16. **Host migration**: If party leader disconnects, what happens?

#### Spectator Mode
17. **Allow spectating**: Can eliminated players watch? Can non-players spectate?

### Assumptions (if not clarified)
- WebSocket updates at 10Hz (100ms intervals) for positions
- Friends-only via invite code, no matchmaking with strangers
- Daily mountain resets at midnight UTC
- Cannot rejoin after disconnect (run continues without you)
- Single-player is supported (lonely scout mode)
- Spectating allowed for eliminated players only

### Conflicts with Architecture Doc
- **GAME_ARCHITECTURE.md** defines single-player patterns extensively
- Real-time sync section is sparse: only defines `GameMessage` enum stub
- No guidance on:
  - WebSocket room management
  - State reconciliation between clients
  - Tick rate / update frequency
  - Authority model (server-authoritative vs client prediction)

### Complexity Estimate: **MODERATE** (2-3 weeks)
- Procedural generation is deterministic from seed (simpler)
- 4 biomes with distinct hazards = significant content
- Real-time co-op sync is the major challenge
- Stamina physics less complex than combat systems

---

## 2. Tanks: Blitzkrieg

### Summary
Based on Flash Attack V2.2 (1989), a real-time multiplayer combat game where 2-6 players control military bases and 4 tanks each. Features split-screen viewports, procedural islands (65,536 variants), multiple weapon systems (phasers, mines, lasers, neutrons, seekers), ghost mode for eliminated players, and in-game radio communications.

### Key Implementation Questions

#### Display & Terminal
1. **Split-screen feasibility**: Can 80x24 terminal handle base view + 4 tank viewports?
2. **Viewport sizes**: Original shows ~29x15 for base + ~29x7 for each tank - total is wider than 80 cols
3. **Reduced viewport mode**: Should we offer 1-tank-at-a-time view as fallback?
4. **Refresh rate**: How often do we redraw? Every input? Fixed tick?

#### Real-Time Combat
5. **Tick rate**: What's the game simulation rate? 10Hz? 20Hz?
6. **Input processing**: Numpad movement + SHIFT+numpad for firing - how do we handle modifier keys over telnet/SSH?
7. **Projectile travel**: Do phasers/lasers travel visually or instant-hit?
8. **Seeker AI**: How smart is the seeking algorithm? Pathfinding around obstacles?

#### Multiplayer Sync
9. **Authority model**: Server-authoritative for all combat, or trust clients?
10. **Collision detection**: Server validates all hits, or client reports?
11. **Communication jamming**: How is this implemented? Random garbage injection?

#### Ghost Mode
12. **Ghost controls**: Same as tank controls but invulnerable?
13. **Ghost visibility**: Can living players see ghost positions?
14. **Ghost chat**: Separate channel or mixed with living players?

#### Matchmaking
15. **Lobby system**: How do players find games? Public list? Invite codes?
16. **Player count scaling**: Spec shows pod destruction thresholds vary by player count - dynamic mid-game if player leaves?
17. **Minimum players**: Can 2 players start, or need more?

#### Disconnection
18. **Mid-game disconnect**: Base becomes AI? Instant elimination? Pause game?
19. **Rejoin possibility**: Can disconnected player rejoin their base?

### Assumptions (if not clarified)
- Viewport layout will need redesign for 80x24 (spec layout exceeds standard terminal width)
- Server-authoritative combat with client prediction for movement
- 10Hz tick rate for game simulation
- Public lobby with game browser, no matchmaking
- Disconnected player's base becomes neutral/AI-controlled
- Modifier keys (SHIFT) may not work - need alternative control scheme

### Conflicts with Architecture Doc
- **Terminal target of 80x24**: Original spec layout is approximately 60x30 for the full display
- **Character-by-character input**: Original uses SHIFT+numpad which requires special handling
- No mention of:
  - Multi-viewport rendering
  - Real-time game loops
  - Projectile/physics simulation
  - Sound system (spec mentions distinct sounds)

### Complexity Estimate: **COMPLEX** (4-6 weeks)
- Multi-viewport rendering is unprecedented in current codebase
- Real-time combat with 6 players and 24 tanks total
- Multiple weapon systems with different behaviors
- Ghost mode adds second game layer
- Control scheme needs adaptation for terminal constraints

---

## 3. Depths of Diablo

### Summary
A multiplayer roguelite inspired by Diablo 1-2 with procedural dungeons, 3 character classes (Warrior, Rogue, Sorcerer), randomized loot with affixes, permadeath with meta-progression, and 1-4 player co-op. Features 20-floor dungeons with boss fights every 5 floors.

### Key Implementation Questions

#### Combat Model
1. **Real-time or turn-based?**: Spec says "Real-time Sync" but doesn't clarify combat
2. **If real-time**: What's the action speed? Cooldowns? Attack animation timing?
3. **If turn-based in multiplayer**: Simultaneous turns? Sequential? Timer per turn?
4. **Targeting system**: Auto-attack nearest? Manual target selection?

#### Multiplayer Sync
5. **Dungeon authority**: Server generates dungeon, or client with shared seed?
6. **Loot distribution**: Individual loot drops? Shared pool? FFA?
7. **Combat resolution**: Server calculates all damage? Client prediction?

#### Session Management
8. **Can strangers join?**: Random matchmaking or friends-only?
9. **Mid-dungeon join**: Can players join a run in progress?
10. **Player scaling**: Does difficulty scale with player count?

#### Dungeon Generation
11. **Daily dungeon**: Same seed for everyone like Summit, or random per run?
12. **Seed sharing**: Can players share a seed to replay specific dungeons?

#### Permadeath & Progression
13. **Party wipe handling**: All players dead = run ends for everyone?
14. **Partial wipe**: Dead players spectate? Can be revived?
15. **Meta-progression scope**: Per-account or per-character class?

#### Town Hub
16. **Town persistence**: One town per player, or shared town?
17. **Town between runs**: Do players return to town hub between dungeon runs?
18. **Stash sharing**: Can party members access each other's stash?

#### Disconnection
19. **Character fate**: Disconnected character removed? AI takeover? Frozen?
20. **Rejoin with loot**: If rejoin allowed, does character keep items collected?

### Assumptions (if not clarified)
- Turn-based combat (easier to sync, matches BBS aesthetic)
- Server-authoritative dungeon generation
- Individual loot drops (each player sees their own)
- Friends-only via invite code
- Dead players become spectators until party wipes or reaches town
- Meta-progression is per-account
- Each player has their own town instance

### Conflicts with Architecture Doc
- **Instance management**: Architecture doc mentions "Per-party dungeon instances" but no implementation guidance
- **Real-time Sync**: Mentioned in spec but unclear if combat is real-time
- No guidance on:
  - Procedural dungeon generation patterns
  - Loot table / affix system
  - Class ability systems
  - Meta-progression storage

### Complexity Estimate: **COMPLEX** (4-6 weeks)
- Procedural dungeon generation with multiple floor themes
- 3 classes with 4 skills each = 12 unique abilities
- Loot system with rarity tiers and affixes
- Meta-progression system
- Multiplayer co-op adds significant complexity
- Could be 6+ weeks if real-time combat

---

## 4. Acromania

### Summary
A party word game for 3-16 players inspired by Acrophobia. Players receive random acronyms and must invent fitting phrases. Anonymous voting determines winners. Features speed bonuses, category themes, face-off final rounds, and 10-round games with escalating difficulty.

### Key Implementation Questions

#### Player Count
1. **Minimum to start**: 3 players required - what if someone leaves mid-game?
2. **Bot fill-in**: Can bots submit entries to maintain player count?
3. **Dynamic join/leave**: Can players join between rounds? Leave gracefully?

#### Timer Synchronization
4. **Clock sync**: How do we ensure all players see same countdown?
5. **Submission cutoff**: Server time authoritative? Grace period for latency?
6. **Phase transitions**: Automatic or triggered when all players ready?

#### Content Moderation
7. **Profanity filter**: Built-in? Configurable? Community-reported?
8. **Invalid submissions**: Auto-reject, or let voters decide?
9. **Offensive voting**: Can submissions be reported during voting?

#### Voting Mechanics
10. **Vote visibility**: Do players see vote counts in real-time or only at reveal?
11. **Tie-breaking**: What if two submissions tie for votes?
12. **Self-vote prevention**: UI hides own submission, or server rejects?

#### Lobby System
13. **Game creation**: Anyone can create? Only certain users?
14. **Private games**: Invite-only option?
15. **Spectators**: Can non-players watch and chat?

#### Categories
16. **Category selection**: Random? Host chooses? Voted on?
17. **Custom categories**: Can players create custom categories?

### Assumptions (if not clarified)
- Game continues with 2 players if someone leaves (reduced but playable)
- No bots - human players only
- Server time is authoritative for all timers
- Basic profanity filter built-in
- Ties broken by submission speed
- Anyone can create games
- Spectators allowed with chat disabled during rounds

### Conflicts with Architecture Doc
- **Real-time timers**: Architecture doc doesn't address synchronized timers
- **Lobby system**: Mentioned as needed for real-time multiplayer but no implementation details
- **Content filtering**: Not mentioned in architecture doc at all

### Complexity Estimate: **MODERATE** (1-2 weeks)
- Simplest of the four games mechanically
- No spatial/physics simulation
- Main challenges: timer sync, content moderation, lobby system
- Can reuse lobby patterns from other multiplayer games

---

## Cross-Cutting Questions

### WebSocket Infrastructure (All 4 Games)

1. **Room/lobby management**: Need shared infrastructure for all multiplayer games
2. **Message protocol**: Define standard message format (JSON? MessagePack? Custom?)
3. **Connection pooling**: How many concurrent games supported?
4. **Heartbeat/keepalive**: Detect dead connections how quickly?

### Matchmaking Infrastructure

5. **Global lobby service**: One service for all games, or per-game?
6. **Friend system**: Is there a BBS-wide friend list for invites?
7. **Game browser**: Public games visible to all users?

### Disconnection Policy (All 4 Games)

8. **Grace period**: How long before declaring player disconnected?
9. **Reconnection window**: Allow rejoin within X minutes?
10. **Partial game save**: Can interrupted games be resumed later?

### Daily Seed System (Summit, Depths of Diablo)

11. **Seed generation**: Central server generates, or deterministic from date?
12. **Timezone handling**: UTC standard, or regional seeds?
13. **Leaderboard scope**: Daily leaderboard per mountain/dungeon?

### Spectator Mode (All 4 Games)

14. **Universal spectator system**: Build once, use for all games?
15. **Delay for competitive**: Add delay to prevent cheating?
16. **Spectator chat**: Separate channel?

### Real-Time Sync Architecture (Summit, Tanks, Depths of Diablo)

17. **Authority model**: Full server authority, or client prediction?
18. **State reconciliation**: How to handle desyncs?
19. **Bandwidth limits**: Max update rate per player?
20. **Latency compensation**: Rollback? Interpolation? Accept jitter?

---

## Recommended Implementation Order

1. **Acromania** (1-2 weeks) - Simplest multiplayer, establishes lobby/timer patterns
2. **Summit** (2-3 weeks) - Co-op focus, can reuse lobby system
3. **Depths of Diablo** (4-6 weeks) - Complex but can start simple (single-player first)
4. **Tanks: Blitzkrieg** (4-6 weeks) - Most complex, needs multi-viewport rendering

### Pre-requisite Infrastructure
Before any of these games:
- WebSocket room management system
- Lobby service with invite codes
- Timer synchronization protocol
- Spectator mode framework

---

## Architecture Gaps to Address

The current `GAME_ARCHITECTURE.md` needs expansion for:

1. **Real-time multiplayer patterns**
   - WebSocket room lifecycle
   - Message protocol specification
   - State synchronization strategies

2. **Lobby system**
   - Game creation flow
   - Player join/leave handling
   - Ready-up mechanics

3. **Disconnection handling**
   - Detection mechanisms
   - Recovery options
   - AI takeover patterns

4. **Multi-viewport rendering**
   - Terminal subdivision
   - Independent viewport updates
   - Focus/selection indication

5. **Timer synchronization**
   - Server-authoritative timers
   - Client countdown display
   - Latency compensation

---

## Questions Requiring User Input Before Implementation

### Critical (Blocks Design)
1. Real-time vs turn-based combat for Depths of Diablo?
2. Terminal viewport layout for Tanks (80x24 constraint)?
3. Matchmaking: strangers or friends-only for all games?
4. Daily seed timezone handling?

### Important (Affects Scope)
5. Bot fill-in for Acromania when players leave?
6. Spectator mode for all games or specific ones?
7. Disconnection policy: strict (forfeit) or lenient (rejoin)?
8. Content moderation approach for Acromania?

### Nice to Know (Optimization)
9. Expected concurrent player counts per game?
10. Latency tolerance targets?
11. Mobile/slow terminal support priority?

---

*Document generated for planning purposes. Answers to these questions will significantly impact architecture decisions and timeline estimates.*
