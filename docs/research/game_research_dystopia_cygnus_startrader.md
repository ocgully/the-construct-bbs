# Game Research: Dystopia, Master of Cygnus, Star Trader

**Research Date:** 2026-01-30
**Researcher:** Claude Agent
**Reference Implementation:** `backend/src/games/grand_theft_meth/`

---

## Executive Summary

All three games are **Complex** async multiplayer strategy games with shared worlds. They have significant overlap in technical requirements (turn systems, multiplayer coordination, economy simulation) but differ substantially in theme and mechanics. Key decision points center around:

1. **Tick/Turn timing** - How often do things happen? Real-time? Scheduled?
2. **Game round duration** - Weeks? Months? Perpetual?
3. **Team formation rules** - How many players? How do teams form?
4. **AI/NPC behavior** - How sophisticated should Ferrengi/enemy factions be?
5. **Cross-game integration** - Do these games share any data or economy?

---

## Game 1: Dystopia

### Summary
A kingdom management game where players manage provinces within kingdoms. Players build structures, train military, research magic, and engage in inter-kingdom warfare. Ages (rounds) last multiple weeks with hourly resource ticks. Heavily inspired by web-based kingdom management games like Utopia.

### Key Implementation Questions

#### Tick System
1. **Hourly tick implementation**: Should ticks be:
   - Real-time background job running every hour?
   - Calculated on-demand when player logs in (catchup calculation)?
   - Hybrid (background for active kingdoms, catchup for inactive)?

2. **Tick offset**: Should all kingdoms tick at the same time, or stagger to reduce server load?

3. **What happens during a tick?**
   - Resources accumulate (gold from peasants, food from farms, runes from towers)
   - Military upkeep deducted
   - Construction/training progress
   - Spell durations decrease
   - Does notoriety/protection decay happen hourly or daily?

#### Age/Round System
4. **Age duration**: Spec says "multi-week" - specifically how long?
   - 4 weeks (28 days)?
   - 6 weeks?
   - 8 weeks?
   - Configurable per age?

5. **Age transitions**:
   - What happens between ages? Full reset? Partial carryover?
   - Is there downtime between ages for signup?
   - Can players join mid-age?

6. **Win conditions**: How is the winning kingdom determined?
   - Total land controlled?
   - Combined net worth?
   - War victories?
   - Point system combining multiple factors?

#### Kingdom Formation
7. **Kingdom size**: Spec says 10-25 players. Questions:
   - What's the minimum to start? (5? 10?)
   - Can kingdoms recruit mid-age?
   - What happens if a kingdom drops below minimum?
   - Is there a queue/draft system or open signup?

8. **King mechanics**:
   - How is the king chosen? Founder? Voted?
   - What powers does the king have?
   - Can king be deposed?

9. **Kingdom coordination**:
   - In-game messaging system required?
   - War declarations - who can declare? King only? Majority vote?

#### Combat & Warfare
10. **Attack resolution timing**:
    - Instant resolution?
    - Travel time based on distance?
    - Queued attacks resolve at next tick?

11. **Attack limits**:
    - How many attacks per day/hour?
    - Cooldown between attacks on same target?
    - Any restriction on attacking much smaller provinces?

#### Protection System
12. **New player protection**:
    - Duration? (24 hours? 48? Until first attack?)
    - Does it break on any offensive action?
    - Can protected players still interact with kingdom?

#### Race/Personality System
13. **Balance**: Are races/personalities pre-defined in spec? If not:
    - How many races? (4-8 typical)
    - Are bonuses percentage-based or flat?
    - Can players see opponents' races?

### Assumptions (If Not Clarified)

1. **Tick system**: Implement background hourly tick job for all kingdoms, with catchup calculation on login for any missed ticks.

2. **Age duration**: Default 6 weeks (42 days), configurable.

3. **Kingdom size**: Minimum 5 to start, soft cap at 25, must maintain minimum 3 active players or kingdom disbands.

4. **Attack resolution**: Instant, but travel time of 1-4 hours before troops return (can't attack again until return).

5. **Protection**: 48 hours for new players, breaks on any offensive action (attack, theft, spell cast on enemy).

6. **Races**: Start with 4 races (Human, Elf, Dwarf, Orc analog) with distinct bonuses.

7. **Win condition**: Combined kingdom net worth (land * development factor + military value + treasury).

### Architecture Conflicts

1. **Database per game vs shared tables**: The architecture doc says "separate database per game" but Dystopia has complex inter-player relationships. Consider:
   - `data/dystopia.db` containing all tables
   - Ages, kingdoms, provinces, attacks, messages all in one DB
   - This is fine, just more complex schema than single-player games

2. **Real-time sync for kingdoms**: Architecture mentions WebSocket for real-time but Dystopia is async. However:
   - Kingdom chat would benefit from real-time
   - Attack notifications could be real-time
   - May need hybrid approach

3. **Daily limits vs hourly ticks**: GTM reference uses daily actions (5/day). Dystopia has hourly ticks but probably still needs daily limits on certain actions (attacks, spells). Need to reconcile these two time systems.

### Complexity Estimate

**High Complexity** - 3-4 weeks for core, 2+ weeks for polish

- Hourly tick system with catchup logic
- Kingdom management and coordination
- Combat system with army composition
- Magic/spell system
- Age lifecycle management
- Inter-kingdom messaging
- Leaderboards per age + all-time

---

## Game 2: Master of Cygnus

### Summary
A 4X space strategy game inspired by Master of Orion 1. Players lead civilizations to dominate a galaxy through exploration, expansion, exploitation, and extermination. Async turn-based with 72-hour turn deadlines. Features colony management, tech research, ship design, and fleet combat.

### Key Implementation Questions

#### Turn System
1. **Turn resolution model**: Spec says "if both online, resolve immediately; otherwise 72hr timeout"
   - How do we detect "both online"? WebSocket presence?
   - Is it "all players" for 3+ player games?
   - Can players retract orders before resolution?

2. **Turn timeout handling**:
   - 3 timeouts = forfeit - is this consecutive or total?
   - What happens on forfeit? AI takes over? Instant loss?
   - Can players rejoin after timeout?

3. **Simultaneous vs sequential turns**:
   - All players submit orders, then resolve together (like Diplomacy)?
   - Or sequential turns (like Civ)?

#### Game Setup
4. **Game creation**:
   - How do players find/create games? Lobby system?
   - Can games be private (invite-only)?
   - Rating/skill matching?

5. **Player count**:
   - Minimum players to start? (2?)
   - Maximum players? (8? 16?)
   - Can AI fill empty slots?

6. **Galaxy generation**:
   - Spec says 20-50 stars. How is size determined?
   - Deterministic from seed (replayable)?
   - Fair start positions guaranteed?

#### Colony Management
7. **Production queue**:
   - How many items can be queued?
   - Can queue be reordered?
   - What happens when resources insufficient?

8. **Population growth**:
   - Formula for growth rate?
   - Max population per planet type?
   - Can population be transferred between colonies?

#### Technology
9. **Tech tree structure**:
   - Fixed tech tree or procedural choices (like MOO1)?
   - Tech trading between players?
   - Tech stealing through espionage?

10. **Research allocation**:
    - Percentage-based across fields?
    - All-or-nothing per field?
    - Colony-specific research buildings?

#### Ship Design & Combat
11. **Ship design validation**:
    - Real-time validation as player designs?
    - Can designs be copied/shared?
    - Retrofit existing ships or build new?

12. **Combat resolution**:
    - Automated resolution with report?
    - Turn-by-turn tactical combat?
    - Can players observe combat in progress?

13. **Fleet movement**:
    - ETA calculation - fixed or depends on engine tech?
    - Can fleets be redirected mid-journey?
    - Supply/range limits?

#### Victory Conditions
14. **Victory types**:
    - Conquest: What percentage of galaxy needed?
    - Council vote: When does council convene? Who votes?
    - Technology: Which tech triggers victory?

### Assumptions (If Not Clarified)

1. **Turn resolution**: Simultaneous submission, resolves when all submit OR 72hr deadline. All players in same game must submit.

2. **Timeout penalty**: 3 consecutive timeouts = AI takes over empire (not forfeit). Player can request to return.

3. **Player count**: 2-8 players, games start when creator triggers "begin" with minimum 2.

4. **Galaxy size**: Small (20-25 stars) for 2-3 players, Medium (30-40) for 4-6, Large (45-50) for 7-8.

5. **Tech tree**: Fixed tree with 6 branches, MOO1-style "pick 1 of 3" choices at certain tiers.

6. **Combat**: Automated resolution with detailed report. No real-time tactical combat (too complex for BBS async).

7. **Victory**: Conquest (control 2/3 of inhabited planets), Council (every 50 turns, majority of population), Tech (reach level 10 in any field).

### Architecture Conflicts

1. **Game instance management**: Unlike single-player games, MOC needs:
   - Multiple concurrent game instances
   - Game lobby/matchmaking system
   - Per-game state (galaxy, empires, turns)

   Schema needs `games` table + per-game foreign keys on all state tables.

2. **Turn processor**: Needs background job to:
   - Check for turn deadlines
   - Process simultaneous order resolution
   - Handle combat calculations
   - Send notifications

   This is more complex than GTM's simple daily progression.

3. **State size**: Galaxy + all empires + fleets + colonies could be large JSON. May need to split state across tables rather than single `state_json` blob.

### Complexity Estimate

**Very High Complexity** - 4-6 weeks for core, 2-3 weeks for polish

- Turn-based async coordination (most complex)
- Galaxy generation and pathfinding
- Tech tree and research system
- Ship designer with component validation
- Combat resolver (could be a week alone)
- Colony production queues
- Diplomacy system
- Multiple victory condition tracking

---

## Game 3: Star Trader (Trade Wars Clone)

### Summary
Space trading game inspired by Trade Wars 2002. Players command starships, trade commodities between ports, upgrade ships, colonize planets, build starbases, and form corporations. Features the Ferrengi enemy faction. Turn-based with daily action limits (500-1000 turns/day).

### Key Implementation Questions

#### Galaxy & Navigation
1. **Galaxy generation**:
   - How are warps determined? Random? Guaranteed connectivity?
   - One-way warps - how common? How to ensure not trapped?
   - Is the galaxy shared by all players or per-session?

2. **Galaxy persistence**:
   - One eternal galaxy (like original TW2002)?
   - Periodic resets (weeks/months)?
   - Multiple concurrent galaxies?

3. **Sector discovery**:
   - All sectors known from start?
   - Fog of war until visited?
   - Scanner tech reveals nearby sectors?

#### Turn System
4. **Turn costs**:
   - Spec says 500-1000 turns/day. What's the actual default?
   - Turn costs: Move (1-3), Trade (2), Combat (varies) - what's "varies"?
   - Building (5+) - what's the maximum?

5. **Turn banking**:
   - Can excess turns be saved?
   - Maximum banked turns?
   - Do banked turns expire?

6. **Turn reset**:
   - Midnight UTC? Player's local midnight?
   - All at once or staggered?

#### Trading Economy
7. **Port supply/demand**:
   - How does supply regenerate?
   - Does demand change based on player trading?
   - Is economy simulation real-time or tick-based?

8. **Price volatility**:
   - How much can prices fluctuate?
   - Are there price events (like GTM's price spikes)?
   - Can players manipulate markets?

9. **Haggling**:
   - Skill-based? Random? Both?
   - Does haggling cost turns?

#### Ships & Combat
10. **Ship acquisition**:
    - Purchase at StarDock only?
    - Trade-in value for old ship?
    - Can ships be customized/upgraded incrementally?

11. **Fighter mechanics**:
    - How do fighters attack? Automatic?
    - Can fighters be deployed in sectors for defense?
    - Fighter production on planets - how fast?

12. **Escape pod**:
    - Where do you respawn?
    - What resources (if any) are saved?
    - Cooldown before playing again?

#### Planets & Starbases
13. **Planet colonization**:
    - How many colonists needed to start?
    - Growth rate formula?
    - Can planets be attacked/conquered?

14. **Starbase construction**:
    - Resource requirements?
    - Build time?
    - Can starbases be destroyed?

15. **Planet/starbase production**:
    - Tick-based or continuous?
    - Production rates configurable?

#### Corporations
16. **Corporation formation**:
    - Minimum members? (2?)
    - Maximum members? (Unlimited? Capped?)
    - Entry fee or requirements?

17. **Corporation mechanics**:
    - How is treasury shared?
    - Can members withdraw from treasury?
    - How are planet/starbase ownership shared?

18. **Corporation warfare**:
    - Formal war declarations needed?
    - Can members attack corp property?
    - Espionage mechanics?

#### Ferrengi AI
19. **Ferrengi behavior**:
    - Patrol patterns - random or strategic?
    - Aggression level - always attack? Probability-based?
    - Do Ferrengi adapt to player strategies?

20. **Ferrengi economy**:
    - Do they trade? Produce resources?
    - Can players interact peacefully with Ferrengi?

#### Game Rounds
21. **Round duration**:
    - Perpetual universe?
    - Scheduled resets (monthly? Quarterly)?
    - Victory conditions that end rounds?

### Assumptions (If Not Clarified)

1. **Galaxy**: Shared persistent galaxy, 5000 sectors default, monthly optional resets.

2. **Turns**: 750 turns/day default, can bank up to 500, resets at midnight UTC.

3. **Turn costs**: Move (2), Trade (3), Combat round (5), Planet landing (10), Building (varies by type).

4. **Economy**: Tick every 15 minutes for port supply regeneration. Player trading affects supply/demand.

5. **Corporations**: Minimum 2 members, maximum 10, shared treasury with withdrawal limits (50%/day).

6. **Ferrengi**: Patrol randomly, 75% chance to attack on encounter, difficulty scales with player rank.

7. **Game rounds**: Perpetual with optional monthly "seasons" that have leaderboard resets but persistent galaxy.

### Architecture Conflicts

1. **Shared world vs per-player state**: Unlike GTM (single-player), Star Trader has:
   - Shared galaxy (all players see same sectors)
   - Player-owned assets (ships, planets, starbases)
   - Corporation state

   Needs careful locking/transactions for concurrent access.

2. **Real-time economy**: Port supply/demand should tick even when no one is playing. Need background job similar to Dystopia.

3. **Large state**: Galaxy of 5000+ sectors is too big for single JSON blob. Need proper relational tables:
   - `sectors` table with warps
   - `ports` table with inventory
   - `planets` table
   - etc.

4. **Ferrengi AI**: Need either:
   - Background job moving Ferrengi ships
   - On-demand generation when player enters sector
   - Recommend: hybrid (spawn on sector entry, persist until destroyed)

### Complexity Estimate

**High Complexity** - 3-4 weeks for core, 2-3 weeks for polish

- Galaxy generation with connectivity validation
- Trading economy with supply/demand simulation
- Multiple ship types and combat
- Planet colonization and production
- Corporation system
- Ferrengi AI
- StarDock special location
- Turn management system

---

## Cross-Cutting Questions (Multiple Games)

### Shared Infrastructure

1. **Background tick jobs**:
   - Dystopia: hourly ticks
   - Star Trader: 15-minute economy ticks
   - Master of Cygnus: turn deadline checker

   **Question**: Single job scheduler or per-game? How to handle server restarts?

2. **Notification system**:
   - All three games need async notifications
   - "Your kingdom was attacked", "Your turn is due", "Your ship was destroyed"

   **Question**: In-app notifications only? Email? Push? All optional?

3. **Cross-game identity**:
   - Do players have separate characters per game?
   - Or same identity (username) across all?

   **Assumption**: Same BBS user account, but separate in-game state per game.

### Economy & Balance

4. **Starting resources**:
   - Should all three use similar "debt + small cash" model like GTM?
   - Or game-specific starting conditions?

   **Assumption**: Game-specific. GTM's debt model fits crime theme, not space games.

5. **Economy tuning**:
   - Static formulas from spec?
   - Or configurable for balancing?

   **Assumption**: Configurable via constants in `data.rs`, not hard-coded.

### Multiplayer Coordination

6. **Player matching**:
   - Dystopia: Kingdom signup/draft
   - MOC: Game lobby
   - Star Trader: Shared world

   **Question**: Unified lobby system or per-game?

7. **Inactive player handling**:
   - How long before considered inactive?
   - Auto-forfeit vs AI takeover vs protection?

   **Assumption**: 7 days inactive = AI takeover (can reclaim), 30 days = account pruned from game.

8. **Griefing prevention**:
   - All three allow attacking other players
   - What prevents veteran players from crushing newbies?

   **Assumption**: Protection period for new players, matchmaking by power level, possibly "newbie sectors" in Star Trader.

### Cross-Game Integration

9. **Do Star Trader and Dystopia share anything?**
   - Same universe/lore?
   - Cross-game achievements?
   - Shared currency?

   **Assumption**: No cross-game integration initially. Can add achievements later.

10. **Universe continuity**:
    - Are these games in same fictional universe?
    - Does it matter for implementation?

    **Assumption**: Separate universes, no narrative connection needed.

### Technical Standards

11. **State serialization**:
    - GTM uses single `state_json` blob
    - These games have much more state

    **Question**: Continue with JSON blob, or normalized relational tables?

    **Recommendation**: Normalized tables for shared world state, JSON for player-specific preferences/settings.

12. **Testing multiplayer**:
    - GTM tests are single-player focused
    - How to test async multiplayer scenarios?

    **Recommendation**: Need test harness that can simulate multiple players, time advancement, and concurrent actions.

---

## Recommended Implementation Order

1. **Star Trader** (First)
   - Most similar to GTM (trading focus)
   - Can start simple and add features
   - Establishes multiplayer patterns for others

2. **Dystopia** (Second)
   - Uses patterns from Star Trader
   - Kingdom system more complex
   - Age/round lifecycle management

3. **Master of Cygnus** (Third)
   - Most complex turn resolution
   - Benefits from learnings of other two
   - Ship designer is unique complexity

---

## Summary of Required User Input

### Must Have Answers Before Implementation

| Question | Game | Impact |
|----------|------|--------|
| Tick implementation (real-time vs catchup) | Dystopia, Star Trader | Architecture decision |
| Turn resolution model (simultaneous vs sequential) | MOC | Core gameplay |
| Galaxy persistence (perpetual vs resets) | Star Trader | Data model |
| Age duration | Dystopia | Game balance |
| Corporation max size | Star Trader | Balance |
| Kingdom min/max size | Dystopia | Balance |
| Ferrengi AI complexity | Star Trader | Scope |
| Background job scheduler approach | All | Infrastructure |
| Cross-game integration | All | Scope |

### Nice to Have Clarifications

| Question | Game | Default Assumption |
|----------|------|-------------------|
| Turn banking limits | Star Trader | 500 max |
| Tech tree structure | MOC | Fixed with choices |
| Race/personality count | Dystopia | 4 races |
| Victory conditions | MOC, Dystopia | Multiple options |
| Notification channels | All | In-app only |

---

*Document prepared for user review. Please provide feedback on questions and validate or correct assumptions before implementation begins.*
