# Epic Games Research: Questions & Assumptions

This document consolidates implementation questions, assumptions, and concerns for the following games assigned for research:

1. **Last Dream** - JRPG with simulation twist
2. **Mineteria** - Sandbox/survival/crafting
3. **Fortress** - Colony simulation (Dwarf Fortress inspired)
4. **Ultimo** - MMORPG (Ultima 1-6 inspired)
5. **Cradle** - Infinite progression RPG
6. **Xodia the Living MUD** - LLM-powered MUD

Reference implementation studied: `backend/src/games/grand_theft_meth/`

---

## 1. Last Dream (JRPG)

### Summary
A classic JRPG in Final Fantasy 1-2 style with turn-based combat, party system, overworld exploration, towns/dungeons, and transportation progression (walk -> ship -> airship). The twist: the world is revealed to be a simulation in the final act, with subtle "glitches" and breadcrumbs hidden throughout the game.

### Key Implementation Questions

**Simulation Twist:**
1. How subtle should the breadcrumbs be? The spec lists several (NPCs using anachronistic words, "FILE NOT FOUND" bookshelf, child asking about "computers"). Should these be:
   - Extremely rare (1-2 per playthrough, easily missed)?
   - Moderate (one per major area, attentive players notice)?
   - Obvious (clear pattern for players looking for it)?

2. When should the twist be revealed? Options:
   - Only at final boss defeat (true ending only)
   - Multiple endings (bad ending hides it, good ending reveals it)
   - Post-credits scene for any completion

3. Should there be a "New Game+" mode where the simulation elements become more visible/interactive?

**Scope & Content:**
4. How many dungeons/towns are planned for MVP vs full version?
   - Spec mentions 4 acts, 3 continents, multiple crystal dungeons
   - MVP: 1 continent, 2 dungeons, 1 crystal boss?
   - Full: All 4 acts?

5. Party size is "up to 4" - is this 4 fixed story characters or player-recruited?

6. Magic system scope: How many spells per tier? How many tiers?

**Technical:**
7. Maps are described as "pre-designed stored as data files" - what format?
   - JSON tile arrays?
   - Custom text format?
   - Should there be a map editor?

8. Save system: Single slot per user or multiple slots (spec shows `slot_number`)?

### Assumptions (if not clarified)

- Breadcrumbs will be moderate difficulty (one per area, 5-7 total, missable but findable)
- Twist revealed only in true ending after defeating final boss
- MVP scope: 2 dungeons, 3 towns, 1 continent, 1 crystal, ship unlock
- 4 fixed story party members (classic FF1 style - player picks classes at start)
- 3 spell tiers with 4 spells each = 12 total spells (simple)
- Maps stored as JSON with tile arrays
- Single save slot per user (can delete and restart)
- No New Game+ for MVP

### Architecture Conflicts

- None major. Follows standard single-player turn-based pattern like Grand Theft Meth
- Will need more complex combat system (party-based rather than solo)
- Map rendering more complex (multi-screen world maps vs single location)

### Complexity Estimate

**Spec says: Complex**
**My estimate: Complex to Epic (6-10 weeks for MVP)**

Justification: Party combat, multi-map world, transportation modes, and the subtle breadcrumb system require significant content authoring beyond code.

---

## 2. Mineteria (Sandbox/Survival)

### Summary
A 2D procedurally generated sandbox in the Minecraft/Terraria vein. Mining, crafting, building, day/night cycle, combat, biomes. Persistent player worlds.

### Key Implementation Questions

**World Storage & Performance:**
1. What chunk size for world storage? Spec mentions chunks but no dimensions.
   - 16x16 tiles per chunk (Minecraft-style)?
   - 32x32? 64x64?

2. How large can a world be? Limits needed?
   - Horizontal: 1000 chunks? 10000?
   - Vertical: Fixed depth (100 tiles) or variable?

3. Chunk loading strategy:
   - Load surrounding 3x3 chunks?
   - Lazy load on approach?
   - How to handle simultaneous players if multiplayer is added?

4. How to compress/store tile data efficiently?
   - Run-length encoding?
   - Delta compression?
   - Just raw JSON (might be huge)?

**Crafting & Content:**
5. How many items/blocks for MVP?
   - Spec shows simplified tree (Wood -> Stone -> Iron -> Advanced)
   - Full Terraria has 5000+ items - what's realistic?
   - MVP: ~50 items? 100?

6. How many biomes for MVP? Spec lists 5 (Forest, Desert, Tundra, Swamp, Mountains)
   - MVP: 2-3 biomes?

7. Day/night cycle length? Real-time minutes? Action-based?

**Combat & Survival:**
8. Combat system depth?
   - Simple bump-attack?
   - Weapons with different behaviors?
   - Enemy AI complexity?

9. Hunger/survival mechanics - how punishing?
   - Death on starvation?
   - Debuffs only?

**Multiplayer:**
10. Spec says "Single-player (Multiplayer stretch)" - is multiplayer a goal?
    - If yes, how many concurrent players per world?
    - Shared world or instance per player?

### Assumptions (if not clarified)

- Chunks are 32x32 tiles
- World limit: 500 chunks horizontal, 200 tiles vertical (reasonable for BBS)
- Load 5x5 chunk area around player
- Store chunks as compressed binary blobs in SQLite
- MVP: 50 items, 3 biomes, 10 enemy types, 5 ore types
- Day/night cycle: 5 real minutes per in-game day
- Simple bump-attack combat with weapon damage modifiers
- Hunger causes debuffs, not death (no forced death from forgetting to eat)
- Single-player only for MVP, multiplayer deferred

### Architecture Conflicts

- **Real-time input requirement** - BBS is request/response, not real-time. How to handle:
  - Movement should be instant (server-side position update)
  - Mining/crafting can be action-based (click to start, time to complete)

- **Rendering complexity** - Need to render a viewport of tiles every frame
  - May need optimized rendering (only send changes, not full screen)

- **Database pressure** - Chunk saves could be frequent
  - Need batching/debouncing strategy
  - Consider separate SQLite file per world

### Complexity Estimate

**Spec says: Epic**
**My estimate: Epic (12-20 weeks)**

Justification: Procedural world generation, chunk management, crafting systems, real-time-ish gameplay, and significant content (items, recipes, enemies) make this one of the largest undertakings.

---

## 3. Fortress (Colony Simulation)

### Summary
A simplified Dwarf Fortress - colony management with dwarves, resources, production chains, threats. All player fortresses exist in a shared world for trading and diplomacy.

### Key Implementation Questions

**Shared World Model:**
1. How large is the shared world map?
   - Grid of claimable tiles?
   - How many tiles per player (fortress footprint)?
   - How do fortresses interact (distance-based trade costs)?

2. Can fortresses attack each other? Spec is unclear on PvP.
   - Trade-only interaction?
   - Raids possible?
   - War declarations?

3. How to prevent griefing in shared world?
   - Protection period for new players?
   - Limits on aggressive actions?

**Dwarf Simulation:**
4. How many dwarves per fortress? Limits?
   - Start with 7 like DF?
   - Max 50? 100? Unlimited?

5. How complex is dwarf AI?
   - Simple task queue (do next available task)?
   - Needs priority system?
   - Moods/personality affecting work?

6. Death and replacement - how do you get new dwarves?
   - Immigration waves?
   - Breeding?
   - Purchasing?

**Simplification Level:**
7. How simplified vs original DF?
   - Full fluid simulation? No?
   - Temperature? No?
   - Detailed combat? Simplified?

8. Threat system - spec says "simplified, seasonal raids"
   - Is this just scheduled combat events?
   - How to scale difficulty with fortress value?

**Tick System:**
9. How often do ticks occur?
   - Real-time (every second)?
   - Turn-based (player action triggers tick)?
   - Scheduled (every 5 minutes)?

10. What happens when player is offline?
    - Pause simulation?
    - Continue with AI?
    - Simplified "background mode"?

### Assumptions (if not clarified)

- World map is 1000x1000 grid, each fortress claims 5x5 tiles
- No direct PvP attacks, trade-only multiplayer
- 2-week protection period for new fortresses
- Start with 7 dwarves, max 50, new dwarves arrive via immigration events
- Simplified AI: task queue with priority, no complex moods
- Dwarves can die but can be replaced through immigration
- No fluid/temperature simulation, simplified combat
- Seasonal raids are scripted events with difficulty based on fortress wealth
- Tick every 30 seconds when online, pause when offline (or slow background mode)

### Architecture Conflicts

- **Tick system** - Need background job for world simulation
  - Not just on-demand like GTM
  - May need separate simulation process/thread

- **Shared world state** - Multiple players modifying same world
  - Need careful transaction handling
  - Race conditions on trade/diplomacy

- **Complexity creep** - DF is famously complex; need strict feature limits

### Complexity Estimate

**Spec says: Epic**
**My estimate: Epic (15-25 weeks)**

Justification: Simulation systems (dwarves, resources, production), shared world coordination, tick system, and the inherent complexity of colony management even when simplified.

---

## 4. Ultimo (MMORPG)

### Summary
A persistent multiplayer RPG where all players share the same world. Tile-based ASCII rendering, class system, skill progression, quests, dungeons, economy, and optional PvP zones. Inspired by Ultima 1-6.

### Key Implementation Questions

**Concurrent Players:**
1. How many concurrent players expected in the same world?
   - 10? 50? 100? 500?
   - This dramatically affects architecture

2. Is sharding needed?
   - Single world server?
   - Multiple world instances?
   - Regional sharding?

3. How to handle player density in popular areas (towns)?
   - Limit players per screen?
   - Instance towns per N players?

**Real-time Synchronization:**
4. How real-time does "see other players" need to be?
   - Position updates every second?
   - Every 100ms?
   - Only on screen refresh?

5. Combat: Is it real-time or turn-based?
   - Spec says "turn-based when engaged"
   - Does this pause the world? Instance the combat?
   - What if multiple players attack same enemy?

**World Scale:**
6. How large is the overworld?
   - 100x100 tiles? 500x500? Larger?
   - How many dungeons?
   - How many towns?

7. Are dungeons instanced or shared?
   - Shared: Multiple parties in same dungeon
   - Instanced: Private dungeon per party
   - Spec suggests "instanced if needed" - what triggers it?

**Persistence:**
8. NPC state - do NPCs remember things permanently?
   - Quest givers track who completed quests
   - Merchants have shared or per-player inventory?

9. Economy - player trading concerns
   - Trade scams prevention?
   - Auction house or direct trade only?
   - Gold sinks to prevent inflation?

**Content:**
10. How much pre-authored content vs procedural?
    - Dungeons: Hand-crafted or generated?
    - Quests: Static or dynamic?
    - Monsters: Fixed spawns or procedural?

### Assumptions (if not clarified)

- Target 50 concurrent players, single world, no sharding for MVP
- Position updates every 500ms (acceptable for BBS latency)
- Combat is turn-based, instances the combat (like Pokemon) - world continues around you
- Overworld is 200x200 tiles, 5 towns, 10 dungeons for MVP
- Dungeons are shared (party sees same monsters/state)
- NPCs have per-player quest state but shared world state
- Direct trade only for MVP, gold sinks via equipment degradation and taxes
- Hand-crafted dungeons and quests, procedural monster spawns

### Architecture Conflicts

- **Real-time sync** - This is the biggest challenge
  - Need WebSocket layer for position broadcasts
  - May need separate game server process
  - State reconciliation between DB and memory

- **Database pressure** - Constant position updates
  - May need Redis/in-memory cache for live state
  - SQLite might not handle write load
  - Consider position updates as transient (not persisted every update)

- **Turn-based in real-time world** - Awkward hybrid
  - Need combat instancing to not block world
  - Enemy "claimed" system to prevent steal?

### Complexity Estimate

**Spec says: Epic**
**My estimate: Epic+ (20-30 weeks)**

Justification: MMO infrastructure is fundamentally different from single-player games. Real-time sync, shared world state, concurrent player handling, and content volume make this the most architecturally challenging game.

---

## 5. Cradle (Infinite Progression)

### Summary
Inspired by the Cradle book series - an infinite progression system where players advance through 15+ tiers, each tier expanding the scope of the world. Strategic build choices matter, wrong combinations lead to plateaus, mentor system provides guidance.

### Key Implementation Questions

**Tier System:**
1. 15+ tiers seems massive. Expected time per tier?
   - Early tiers (0-5): Minutes to hours?
   - Mid tiers (6-10): Days?
   - Late tiers (11-15): Weeks?
   - If it takes months to reach high tiers, is that intentional?

2. How to keep early game engaging if late game is the "real" game?
   - Front-load content in tiers 0-5?
   - Or make early tiers very fast?

3. Can players ever "finish" or is it truly infinite?
   - Is tier 15 the cap or is there "???" beyond?
   - Prestige/rebirth system?

**The Puzzle Mechanic:**
4. "Wrong build choices lead to diminishing returns" - how harsh?
   - Soft ceiling (slow progress)?
   - Hard wall (literally cannot advance)?
   - Fixable via rebirth/reset mechanics?

5. How transparent is the build system?
   - Hidden formulas (discover through play)?
   - Visible requirements (can plan builds)?
   - Community will solve this instantly - is that okay?

6. Mentor system - how does it work?
   - Always available guidance?
   - Limited hints per day?
   - LLM-powered or scripted responses?

**World Expansion:**
7. World "expands" with tier - how literal?
   - New areas unlock as you tier up?
   - Same areas but new content appears?
   - Procedurally generated higher-tier content?

8. Can high-tier players interact with low-tier players?
   - Mentorship mechanics?
   - One-shotting low-tier content?
   - Separate instances by tier range?

**Procedural Content:**
9. Spec mentions "higher tiers need generated content" - how much?
   - Tiers 0-10 hand-crafted, 11+ generated?
   - What's generated (enemies? locations? items?)?

10. Path system - how many paths for MVP?
    - Spec shows 10 aspects, multiple path combinations
    - That's potentially hundreds of paths
    - MVP: 5-10 hand-crafted paths with known ceilings?

### Assumptions (if not clarified)

- Time per tier: 0-5 = hours, 6-10 = days, 11-15 = weeks each
- Tier 15 is soft cap, "???" exists but is post-launch content
- Build mistakes create soft ceilings (50% slower progress) not hard walls
- Build requirements partially visible (can see what path needs, not exact formulas)
- Mentors provide 3 free hints per day, then require in-game currency
- World unlocks new areas at tier thresholds (2, 5, 8, 11, 14)
- High-tier players can visit low areas but get no XP, can mentor
- Tiers 0-10 hand-crafted, 11+ has procedural variations
- MVP: 5 paths with clear progressions, 10 aspects for mixing

### Architecture Conflicts

- **Progression calculator complexity** - Need algorithm to determine max tier for any build
  - Could be expensive computation
  - May need caching/precomputation

- **Mentor AI** - If LLM-powered, cost and latency concerns
  - Could use cached responses for common situations
  - Fallback to scripted hints

- **Infinite content** - Need strategy for generating late-game content
  - Template-based generation?
  - Parameter scaling?

### Complexity Estimate

**Spec says: Epic**
**My estimate: Epic (12-18 weeks for MVP with tiers 0-10)**

Justification: The core systems (tiering, paths, techniques) need careful balancing. Content volume is high but can be data-driven. Late-game procedural content deferred to post-MVP.

---

## 6. Xodia the Living MUD (LLM-Powered MUD)

### Summary
A MUD where an LLM acts as Dungeon Master, game mechanics provide rules, and the world grows organically as players explore. Natural language input processed through micro-LLM for intent classification, mapped to canonical actions. Persistent shared world.

### Key Implementation Questions

**LLM Integration:**
1. Which model for DM narration?
   - Claude API (high quality, cost)?
   - GPT-4 (alternative)?
   - Local model (Llama, Mistral)?
   - Different models for different use cases?

2. Which model for intent classification (micro-LLM)?
   - Same as DM model (expensive)?
   - Smaller/cheaper model (Claude Haiku)?
   - Local classifier model?
   - Fine-tuned small model?

3. Cost per request estimate?
   - If Claude Opus for narration: ~$0.01-0.05 per action
   - If Haiku for classification: ~$0.0001 per classification
   - At 100 actions/player/session, 100 players/day = 10,000 actions = $100-500/day?

4. What happens when LLM API is down or rate limited?
   - Fallback to scripted responses?
   - Queue actions for later?
   - "The mists prevent you from acting" message?

**MCP Tools:**
5. Build custom MCP tools or use existing?
   - Custom: Full control, more work
   - Existing: What's available for world state, entity registry?
   - Hybrid approach?

6. What MCP tools are absolutely required for MVP?
   - World state (locations, exits, contents)
   - Character state (stats, inventory, position)
   - Entity registry (named things)
   - Event log (history)

7. How does MCP tool latency affect gameplay?
   - Each DM response may need 3-5 tool calls
   - Sequential: 5 * 500ms = 2.5s delay
   - Parallel where possible?

**Consistency:**
8. How to prevent LLM hallucinating non-existent items/NPCs?
   - Strict tool-use requirements?
   - Post-validation of responses?
   - Fine-tuned model that doesn't hallucinate?

9. How much context to provide LLM per request?
   - Full character state every time?
   - Recent location history?
   - All NPCs in area?
   - Context window limits?

10. How to handle LLM generating conflicting information?
    - Newer info overrides older?
    - Canonical source of truth in DB?
    - Manual admin cleanup?

**Multiplayer:**
11. Multiple players in same location - how does DM handle?
    - Separate narration per player?
    - Shared narration visible to all?
    - Turn order for actions?

12. Real-time or turn-based interaction between players?

**Procedural World:**
13. How much is pre-authored vs generated?
    - Spec shows 7 "anchor" regions
    - Everything between is generated
    - How to ensure generated content is interesting?

14. Can generated content be bad enough to need cleanup?
    - LLM generates nonsensical room?
    - Admin tools to edit/delete?
    - Player flagging system?

### Assumptions (if not clarified)

- DM narration: Claude Sonnet (balance of quality/cost), ~$0.01 per action
- Intent classification: Claude Haiku, ~$0.0001 per action
- Estimated cost: $50-100/day at moderate usage
- API failure fallback: Queue actions, generic "the mists swirl" responses
- Build custom MCP tools for world/entity/event/character management
- LLM cannot create entities without tool approval (no hallucination)
- Context: Character + current location + recent 5 actions + relevant NPCs
- Multiplayer: Shared narration, quasi-turn-based (actions resolve in order received)
- 7 anchor regions hand-crafted, connections between are LLM-generated on first visit
- Admin tools to flag/edit/delete bad generated content

### Architecture Conflicts

- **LLM dependency** - Core gameplay requires LLM
  - This is fundamentally different from other games
  - Offline play impossible
  - Cost is ongoing operational expense

- **Latency** - LLM response time (1-5 seconds) affects UX
  - Need "thinking" indicator
  - Consider streaming responses

- **State synchronization** - LLM must agree with DB
  - DB is source of truth
  - LLM reads from MCP tools, writes through MCP tools
  - Never trust raw LLM output for state changes

- **Cost management** - Need rate limiting, caching
  - Similar actions could use cached responses
  - "Look around" descriptions could be cached

### Complexity Estimate

**Spec says: Epic**
**My estimate: Epic++ (25-40 weeks)**

Justification: Novel architecture (LLM as core component), custom MCP tool development, prompt engineering for consistency, cost optimization, and the inherent unpredictability of LLM outputs. This is R&D territory, not just implementation.

---

## Cross-Cutting Questions (All Games)

### Timeline & Prioritization

1. **Which games should be tackled first?**
   - Complexity order: Last Dream < Cradle < Mineteria < Fortress < Ultimo < Xodia
   - Value order: Depends on user interest?

2. **What's the definition of "MVP" for Epic-complexity games?**
   - Playable loop with 1-2 hours of content?
   - Feature-complete but content-light?
   - Vertical slice of full vision?

3. **Are these games blocking on each other or independent?**
   - Can work in parallel?
   - Shared systems to build first (multiplayer infrastructure)?

### Resource & Infrastructure

4. **Multiplayer infrastructure** - Ultimo, Fortress, Xodia all need shared world
   - Build shared infrastructure once?
   - Three different approaches?

5. **LLM costs for Xodia** - Is there a budget?
   - Monthly cap?
   - Per-user limits?
   - Premium feature?

6. **Testing strategy** - Playwright E2E for LLM-dependent game?
   - Mock LLM responses?
   - Live tests with cost budget?
   - Deterministic seed mode?

### Content & Authoring

7. **Map/content editor tools needed?**
   - Last Dream needs map editor
   - Fortress needs dwarf behavior editor?
   - Or all content as code/JSON?

8. **How much content authoring time vs coding time?**
   - These games are content-heavy
   - Programmer time != game completeness

9. **Community content** - Any plans for user-generated content?
   - Player-built structures in Mineteria?
   - User quests in Xodia?

### Realistic Timeline Expectations

Based on my analysis:

| Game | Spec Complexity | My MVP Estimate | Notes |
|------|-----------------|-----------------|-------|
| Last Dream | Complex | 6-10 weeks | Party combat, content authoring |
| Mineteria | Epic | 12-20 weeks | Chunk system, procedural gen |
| Fortress | Epic | 15-25 weeks | Simulation complexity |
| Ultimo | Epic | 20-30 weeks | MMO infrastructure |
| Cradle | Epic | 12-18 weeks | Balancing challenge |
| Xodia | Epic | 25-40 weeks | LLM integration uncertainty |

**Total: 90-143 weeks of work for all 6 games to MVP.**

This suggests either:
- Significant team parallelization
- Multi-year roadmap
- Aggressive scope reduction per game
- Prioritization of 2-3 games initially

---

## Recommended Next Steps

1. **User to clarify** the key questions marked above for each game
2. **Prioritize** which 1-2 games to tackle first
3. **Define MVP scope** more precisely for chosen games
4. **Build shared infrastructure** (multiplayer, LLM if Xodia chosen) first
5. **Create content authoring plan** alongside code plan
6. **Set cost/resource budgets** for LLM and infrastructure

---

*Document generated: 2026-01-30*
*Reference: Grand Theft Meth implementation patterns*
