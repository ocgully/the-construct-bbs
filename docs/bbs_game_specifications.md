# BBS Door Games & Features - Complete Specification Document
## For Parallel Implementation via Claude Code

---

# GAME: Sudoku
**Genre:** Puzzle
**Type:** Single-player
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Simple

### Core Concept
Classic 9x9 Sudoku puzzle, refreshing daily at midnight UTC. All players get the same puzzle each day. Streak tracking for consecutive days completed. Similar to LinkedIn's daily games.

### Key Features
- **Daily Puzzle**: Same puzzle for all players, seeded by date
- **Difficulty Levels**: Easy, Medium, Hard (selectable or rotating)
- **Streak Tracking**: Consecutive days played
- **Timer**: Track completion time
- **Pencil Marks**: Toggle candidate numbers in cells
- **Validation**: Check for errors, highlight conflicts
- **Leaderboards**: Fastest times, longest streaks

### ASCII Display
```
┌───────┬───────┬───────┐
│ 5 3 · │ · 7 · │ · · · │
│ 6 · · │ 1 9 5 │ · · · │
│ · 9 8 │ · · · │ · 6 · │
├───────┼───────┼───────┤
│ 8 · · │ · 6 · │ · · 3 │
│ 4 · · │ 8 · 3 │ · · 1 │
│ 7 · · │ · 2 · │ · · 6 │
├───────┼───────┼───────┤
│ · 6 · │ · · · │ 2 8 · │
│ · · · │ 4 1 9 │ · · 5 │
│ · · · │ · 8 · │ · 7 9 │
└───────┴───────┴───────┘
Row: 1  Col: 3  │ Enter number (1-9) or 0 to clear
```

### Technical Requirements
- **Database**: User streaks, completion times, daily seeds
- **Puzzle Generation**: Deterministic generator from daily seed
- **Solver**: Verify unique solution exists

### Data Model
```
sudoku_daily:
  - date (PK)
  - difficulty
  - seed
  - puzzle (81 chars, 0=empty)
  - solution (81 chars)

sudoku_players:
  - user_id
  - current_streak
  - longest_streak
  - last_played_date
  - games_completed
  - best_time_seconds

sudoku_completions:
  - user_id
  - date
  - time_seconds
  - difficulty
```


---

# GAME: Queens
**Genre:** Puzzle
**Type:** Single-player
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Simple

### Core Concept
Place N queens on an NxN board such that no two queens attack each other, with additional daily constraints (colored regions, blocked squares). Similar to LinkedIn's Queens game. Daily puzzle with streak tracking.

### Key Features
- **Daily Puzzle**: Unique constraints each day
- **Colored Regions**: Each region must contain exactly one queen
- **No Attacks**: No two queens share row, column, or diagonal
- **Streak Tracking**: Consecutive days completed
- **Hint System**: Reveal one safe placement
- **Timer**: Track completion time

### ASCII Display
```
    1   2   3   4   5
  ┌───┬───┬───┬───┬───┐
A │ R │ R │ B │ B │ B │
  ├───┼───┼───┼───┼───┤
B │ R │ R │ B │ G │ G │
  ├───┼───┼───┼───┼───┤
C │ Y │ Y │ Y │ G │ G │
  ├───┼───┼───┼───┼───┤
D │ Y │ P │ P │ P │ G │
  ├───┼───┼───┼───┼───┤
E │ Y │ P │ P │ P │ P │
  └───┴───┴───┴───┴───┘
  
Regions: R=Red B=Blue G=Green Y=Yellow P=Purple
Place one ♛ per region, no attacks allowed.
Position (e.g., A1): _
```

### Puzzle Generation
```
1. Generate N colored regions on NxN grid
2. Ensure exactly one valid queen placement exists
3. Regions must be contiguous
4. Seed by date for consistency
```

### Technical Requirements
- **Database**: User streaks, daily puzzles
- **Generator**: Create valid puzzles with unique solutions
- **Validator**: Check queen placements

### Data Model
```
queens_daily:
  - date (PK)
  - size (5-8)
  - regions (JSON - color per cell)
  - solution (JSON - queen positions)

queens_players:
  - user_id
  - current_streak
  - longest_streak
  - last_played_date
  - games_completed
  - best_time_seconds

queens_completions:
  - user_id
  - date
  - time_seconds
  - hints_used
```


---

---

# BBS FEATURE: Memory Garden
**Type:** Social / Journaling
**Platform:** BBS Main Menu Option
**Complexity:** Simple

### Core Concept
A communal digital garden accessible from the BBS main menu where users leave daily memories (tweet-length text). The garden grows organically with user contributions and system-generated memories for milestones. Entering the garden shows recent memories; users can explore by page or date.

### Key Features
- Daily memory submission (280 character limit, 1 per user per day)
- Random sampling of recent memories on entry
- Paginated exploration (newest first or by specific date)
- System-generated milestone memories (user counts, session counts, total usage time)
- Starter memory: "I was born" dated 1/25/2026
- ASCII garden aesthetic with decorative borders

### Technical Requirements
- **Database**: Memories table, milestone tracking, usage statistics
- **Milestone Triggers**: Check on each session for 10x thresholds

### Data Model
```
memories:
  - id
  - user_id (null for system)
  - content (varchar 280)
  - created_date
  - is_system_generated (bool)
  - milestone_type (nullable: users|sessions|time)

bbs_stats:
  - total_users
  - total_sessions
  - total_time_seconds
  - last_user_milestone (10, 100, 1000...)
  - last_session_milestone
  - last_time_milestone
```

### Milestone Logic
```
Milestones trigger at: 10, 100, 1000, 10000, 100000, 1000000
Check: floor(log10(current)) > floor(log10(previous))
```


---

# GAME: Mineteria
**Genre:** Sandbox / Survival / Crafting
**Type:** Single-player (Multiplayer stretch)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Epic

### Core Concept
A 2D procedurally generated sandbox game in the spirit of early Minecraft/Terraria. Players explore, mine resources, craft items, and survive in an ASCII-rendered world. Persistent worlds with day/night cycles, biomes, and progression systems.

### Key Features
- **Procedural World Generation**: Seed-based terrain with biomes (forest, desert, caves, ocean)
- **Mining System**: Dig through terrain, collect resources (stone, ore, gems)
- **Crafting System**: Combine resources into tools, weapons, structures
- **Inventory Management**: Limited slots, storage chests
- **Day/Night Cycle**: Monsters spawn at night
- **Building**: Place blocks, create structures
- **Combat**: Basic melee/ranged against creatures

### World Generation
```
Layers (top to bottom):
- Sky (empty, birds)
- Surface (grass, trees, structures)
- Soil (dirt, roots)
- Stone (rock, ores)
- Deep Stone (rare ores, gems)
- Bedrock (unbreakable)

Biomes: Forest | Desert | Tundra | Swamp | Mountains
```

### Crafting Tree (Simplified)
```
Wood -> Planks -> Workbench
Stone + Wood -> Stone Tools
Iron Ore -> Furnace -> Iron Ingot -> Iron Tools
Iron + Gold + Gems -> Advanced Items
```

### Technical Requirements
- **Database**: World chunks, player inventory, placed structures
- **Chunk System**: Load/unload world sections for performance
- **Real-time Input**: Responsive movement and actions

### Data Model
```
worlds:
  - id, seed, name, created_at

world_chunks:
  - world_id, chunk_x, chunk_y
  - tile_data (compressed binary)
  - modified_at

players_worlds:
  - user_id, world_id
  - position_x, position_y
  - inventory (JSON)
  - health, hunger
  -
items:
  - id, name, type, stack_max
  - recipe (JSON nullable)
```

### ASCII Rendering
```
Tiles:
  . = Air       # = Stone      ~ = Water
  " = Grass     @ = Player     T = Tree
  = = Wood      O = Ore        * = Gem
  M = Monster   D = Door       [ = Chest
```


---

# GAME: Ultimo
**Genre:** MMORPG
**Type:** Multiplayer (Persistent World)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Epic

### Core Concept
A persistent multiplayer RPG inspired by Ultima 1-6, rendered entirely in ASCII art. All players share the same world, can see each other, trade, party up, and compete. Features an overworld, towns, dungeons, and a class/skill system.

### Key Features
- **Persistent Shared World**: All players exist in same world instance
- **ASCII Rendering**: Tile-based world with character sprites
- **Class System**: Warrior, Mage, Rogue, Cleric
- **Skill Progression**: Use-based skill improvement
- **Quests**: NPC-given tasks with rewards
- **Dungeons**: Instanced or shared dangerous areas
- **Economy**: Player trading, shops, currency
- **PvP Zones**: Optional player combat areas

### World Structure
```
Overworld (large tile map):
  - Towns (safe zones, shops, NPCs)
  - Wilderness (monsters, resources)
  - Dungeon Entrances
  - Points of Interest

Dungeons (separate maps):
  - Multiple floors
  - Increasing difficulty
  - Boss rooms
  - Treasure
```

### Combat System
```
Turn-based when engaged:
  - Attack (physical damage based on weapon + STR)
  - Cast (spell from memorized list, uses MP)
  - Use Item (potions, scrolls)
  - Flee (DEX check)

Stats: STR, DEX, INT, VIT
Derived: HP, MP, Attack, Defense, Speed
```

### Technical Requirements
- **Database**: Player characters, world state, NPC states, quest progress
- **Real-time Sync**: Player positions visible to others
- **Instance Management**: Dungeon instances if needed

### Data Model
```
characters:
  - id, user_id, name, class
  - level, experience
  - str, dex, int, vit
  - hp_current, hp_max, mp_current, mp_max
  - position_map, position_x, position_y
  - gold, inventory (JSON)
  - skills (JSON)

maps:
  - id, name, type (overworld|town|dungeon)
  - width, height, tile_data
  - spawn_x, spawn_y

npcs:
  - id, map_id, name, type (merchant|quest|enemy)
  - position_x, position_y
  - dialogue (JSON)
  - shop_inventory (JSON nullable)

quests:
  - id, name, giver_npc_id
  - requirements (JSON)
  - rewards (JSON)

character_quests:
  - character_id, quest_id, status, progress (JSON)
```


---

# GAME: Last Dream
**Genre:** JRPG
**Type:** Single-player
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
A classic JRPG in the style of Final Fantasy 1-2 (US), rendered in ASCII. Features an overworld map, towns, dungeons, vehicles/transportation modes, and turn-based party combat. Set in a medieval fantasy world with crystals, kingdoms, and ancient evils. The story concludes with the revelation that the world is a simulation and the hero awakens - but this twist is hidden, with only subtle breadcrumbs and glitches hinting at the truth throughout the journey.

### Key Features
- **Overworld Map**: Traversable world with varied terrain
- **Towns & Dungeons**: Distinct locations with NPCs, shops, encounters
- **Party System**: Up to 4 characters with distinct classes
- **Turn-Based Combat**: FF1-style with Attack, Magic, Item, Run
- **Transportation**: Walking -> Ship -> Airship progression
- **Equipment**: Weapons, armor, accessories per character
- **Magic System**: Spell levels, MP-based casting

### World Structure
```
Transportation unlocks:
  Walking: Starting continent
  Ship: Ocean travel, new continents
  Airship: Anywhere (except mountains)

Map Types:
  Overworld: . grass, ^ mountain, ~ water, # forest, O town, X dungeon
  Town: Buildings, NPCs, shops
  Dungeon: Multi-floor, encounters, puzzles, boss
```

### Combat System
```
Party vs Enemy Group (turn-based)
Turn order by Speed stat

Actions:
  FIGHT - Physical attack single target
  MAGIC - Cast known spell (costs MP)
  ITEM  - Use consumable
  RUN   - Attempt escape (may fail)

Elements: Fire, Ice, Lightning, Holy, Dark
Status: Poison, Sleep, Paralysis, Death
```

### Class Definitions
```
WARRIOR - High HP, STR. Heavy weapons/armor. No magic.
THIEF   - High Speed, luck. Can steal. Light weapons.
MAGE    - High INT, MP. Offensive spells. Fragile.
CLERIC  - Healing/support magic. Medium combat.
MONK    - Unarmed specialist. High damage unequipped.
KNIGHT  - Balanced. Some white magic.
```

### Technical Requirements
- **Database**: Save game state, character progression
- **Map Data**: Pre-designed maps stored as data files
- **Encounter Tables**: Random battles by region

### Data Model
```
save_games:
  - id, user_id, slot_number
  - party (JSON array of character states)
  - position_map, position_x, position_y
  - story_flags (JSON)
  - playtime_seconds
  - gold

character_state:
  - name, class, level, exp
  - hp, hp_max, mp, mp_max
  - str, agi, int, vit, luck
  - equipment (weapon, armor, helm, accessory)
  - spells_known (array)
  - status_effects

maps:
  - id, name, type
  - tile_data, encounter_rate
  - connections (JSON - which edges lead where)

story_events:
  - id, trigger_condition
  - dialogue, rewards, flags_set
```

### Story Framework (Simulation Twist - Hidden)
```
Surface Story (What Players Experience):
  The Four Crystals that maintain world balance are fading.
  Darkness spreads from the Void. Heroes must restore the Crystals.

Act 1: The Awakening
  - Start in Cornelia village (name feels... familiar?)
  - Crystal of Earth dims, earthquakes begin
  - Gather initial party of four Warriors of Light
  - First dungeon: Chaos Shrine
  - BREADCRUMB: NPC says "Sometimes I dream I'm somewhere else entirely"
  
Act 2: The Journey
  - Obtain ship from grateful king
  - Visit 3 continents seeking Crystal guardians
  - Each Crystal dungeon + elemental boss
  - BREADCRUMB: A "glitch" - NPC repeats dialogue oddly, then says "Sorry, where was I?"
  - BREADCRUMB: Ancient text mentions "the Architects who dreamed the world"

Act 3: The Darkness
  - Crystals restored but Void still grows
  - Airship obtained from ancient civilization
  - Learn the "Void" is actually the simulation degrading
  - BREADCRUMB: Flying over ocean, briefly see a grid pattern in the water
  - Final dungeon: The Rift (geometry becomes impossible)

Act 4: The Awakening (True Ending)
  - Defeat the Void Lord (a corrupted process)
  - World begins to "shut down" - NPCs freeze
  - Party reaches the "Core" - a terminal
  - Hero touches it and WAKES UP
  - Brief scene: hospital bed, year 2XXX, "Simulation complete"
  - Credits roll over the medieval world, now frozen
  
Breadcrumbs (Subtle, Missable):
  - NPCs occasionally use anachronistic words then correct themselves
  - One bookshelf contains "ERROR: FILE NOT FOUND"
  - A child asks "What's a 'computer'? I dreamed that word."
  - Optional dungeon has perfectly geometric, artificial-feeling rooms
  - Final boss says "You cannot delete me. I AM the process."
```


---

# GAME: Cradle
**Genre:** Incremental / Progression RPG
**Type:** Single-player (with leaderboards)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Epic

### Core Concept
Inspired by the Cradle book series, an infinite progression game where characters advance through tiers and titles, with the scope of their world expanding as they grow. From village nobody to multiversal power. Features a deep puzzle element where wrong build choices lead to diminishing returns, requiring strategic pivots and mentor guidance.

### Progression Tiers
```
Tier 0:  Unsouled     - Unawakened, no power
Tier 1:  Copper       - Village level threats
Tier 2:  Iron         - Town level
Tier 3:  Jade         - City level
Tier 4:  Gold         - Regional level
Tier 5:  Lord         - National level
Tier 6:  Overlord     - Continental level
Tier 7:  Sage         - World level
Tier 8:  Herald       - Pocket dimension level
Tier 9:  Monarch      - Solar system level
Tier 10: Dreadgod     - Galactic quadrant level
Tier 11: Abidan       - Galaxy level
Tier 12: Judge        - Universal level
Tier 13: God          - Multiversal level
Tier 14: Void         - Beyond existence itself
Tier 15: ???          - Beyond comprehension
```

### Sacred Arts (Paths)
```
Core Aspects: Force, Life, Shadow, Light, Fire, Water, Earth, Wind, Time, Space

Paths combine 1-3 aspects:
  Pure Force: Raw power, direct damage
  Blackflame (Fire + Destruction): High damage, self-harm
  White Fox (Light + Dreams): Illusions, mind manipulation
  Hollow King (Shadow + Force): Void attacks, defense
  
Each Path has:
  - Cycling techniques (passive advancement)
  - Enforcer techniques (body enhancement)
  - Striker techniques (ranged attacks)
  - Ruler techniques (environment manipulation)
  - Forger techniques (construct creation)
```

### The Puzzle Element
```
Problem: Wrong combinations plateau early
Example: Pure Force + scattered investments = stuck at Gold

Diminishing Returns Triggers:
  - Aspect incompatibility
  - Unbalanced technique spread
  - Resource misallocation
  - Wrong mentor advice followed

Solutions:
  - Consume compatible treasures
  - Undergo rebirth (reset some progress)
  - Find Path-specific inheritance
  - Mentor guidance to correct course
```

### Mentor System
```
Mentors change as you progress:
  Tier 0-2:  Village Elder (basic guidance)
  Tier 3-4:  Sect Master (path refinement)
  Tier 5-6:  Wandering Sage (deep secrets)
  Tier 7-8:  Ascended Being (cosmic truths)
  Tier 9-11: Abidan Guides (multiverse navigation)
  Tier 12+:  The Void whispers (cryptic, dangerous wisdom)

Mentor interactions:
  - Offer hints when stuck
  - Provide quests for advancement
  - Warn about dangerous choices
  - Reveal hidden paths
```

### World Expansion
```
Tier 0-2:   Starting Village -> Valley -> Kingdom
Tier 3-5:   Continent -> World -> Other continents
Tier 6-8:   Pocket worlds -> Other planets -> Solar system
Tier 9-11:  Galaxy sectors -> Full galaxy -> Other galaxies
Tier 12-14: Universe -> Multiverse -> The Void beyond
Tier 15+:   ???
```

### Gameplay Loop
```
1. Train/Cycle - Build power slowly
2. Challenge - Fight enemies at your tier
3. Acquire - Resources, treasures, techniques
4. Advance - Meet tier requirements, break through
5. Explore - New areas unlock
6. Story - Tournaments, trials, guild events
7. Mentor - Get guidance, avoid plateaus
```

### Technical Requirements
- **Database**: Deep character state, world exploration state
- **Path Calculator**: Algorithm to determine advancement ceiling
- **Mentor AI**: Rule-based or LLM-assisted guidance
- **Procedural Content**: Higher tiers need generated content

### Data Model
```
characters:
  - id, user_id, name
  - tier, title, power_level
  - aspects (JSON array)
  - techniques (JSON array with levels)
  - resources (madra, spirit fruit, etc.)
  - mentor_id, mentor_relationship

world_state:
  - character_id
  - discovered_locations (JSON)
  - current_location
  - available_challenges

paths:
  - id, name, aspects_required
  - technique_tree (JSON)
  - max_tier_without_special (ceiling)
  - advancement_requirements

mentors:
  - id, name, tier_range
  - personality, guidance_style
  - hint_database (JSON)
```


---

# GAME: Dystopia
**Genre:** Strategy / Nation Building
**Type:** Multiplayer (Asynchronous)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
A BBS adaptation of the web-based kingdom management genre. Players manage provinces within kingdoms, building military, economy, and magic. Ages (rounds) last weeks, with kingdom coordination and inter-kingdom warfare.

### Key Features
- **Province Management**: Build structures, train units, research magic
- **Kingdom Teams**: 10-25 players form kingdoms
- **Age System**: Multi-week rounds with win conditions
- **Resource Management**: Gold, food, runes, population
- **Military**: Train troops, attack other provinces
- **Magic**: Cast spells for offense, defense, information
- **Race/Class**: Choose race (bonuses) and personality (playstyle)

### Province Resources
```
Land      - Required for buildings
Peasants  - Base population, produce gold
Gold      - Universal currency
Food      - Feeds military
Runes     - Powers magic
Soldiers  - Basic military unit
Specialists:
  - Thieves (covert ops)
  - Wizards (magic)
  - Elite units (race-specific)
```

### Buildings
```
Homes      - Increase population cap
Farms      - Produce food
Banks      - Increase gold income
Training Grounds - Train troops faster
Barracks   - Reduce military upkeep
Forts      - Defensive bonus
Towers     - Magic power
Thieves' Dens - Thief operations
```

### Combat System
```
Attack Types:
  Traditional March - Capture land
  Raid - Steal resources
  Plunder - Destroy buildings
  Massacre - Kill peasants
  Learn - Steal science

Defense: Forts + troops + magic protection
Generals increase army effectiveness
```

### Technical Requirements
- **Database**: Province state, kingdom membership, war logs
- **Tick System**: Hourly resource updates
- **Protection**: New player shields
- **Age Management**: Start/end ages, scoring

### Data Model
```
ages:
  - id, start_date, end_date, winner_kingdom_id

kingdoms:
  - id, age_id, name, motto
  - king_user_id

provinces:
  - id, kingdom_id, user_id
  - name, race, personality
  - land, peasants, gold, food, runes
  - military (JSON)
  - buildings (JSON)
  - science (JSON)
  - protection_until

attacks:
  - id, attacker_id, defender_id
  - type, result (JSON)
  - timestamp

kingdom_messages:
  - kingdom_id, user_id, message, timestamp
```


---

# GAME: Depths of Diablo
**Genre:** Roguelite / Action RPG
**Type:** Multiplayer (Co-op)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
A multiplayer roguelite inspired by Diablo 1-2, featuring procedurally generated dungeons, randomized loot, character classes, and permadeath with meta-progression. Multiple players can explore the same dungeon instance together.

### Key Features
- **Procedural Dungeons**: Each run generates new layouts
- **Class System**: Warrior, Rogue, Sorcerer (expandable)
- **Loot System**: Randomized affixes, rarity tiers
- **Multiplayer**: 1-4 players per dungeon run
- **Permadeath**: Death ends run, but unlock meta-rewards
- **Town Hub**: Persistent between runs, upgrade facilities
- **Boss Floors**: Every 5 floors, major boss fight

### Classes
```
WARRIOR
  - High HP, armor
  - Melee focused
  - Skills: Bash, Whirlwind, Battle Cry, Iron Skin

ROGUE
  - High speed, crit
  - Ranged/melee hybrid  
  - Skills: Multishot, Trap, Shadow Step, Poison Strike

SORCERER
  - High mana, damage
  - Spell focused
  - Skills: Fireball, Frost Nova, Teleport, Chain Lightning
```

### Loot System
```
Rarity: Common (white) -> Magic (blue) -> Rare (yellow) -> Unique (gold)

Affixes:
  Prefix: +Damage, +HP, +Armor, +Element
  Suffix: of Speed, of the Leech, of Frost, of Plenty

Item Types:
  Weapons: Sword, Axe, Bow, Staff, Dagger
  Armor: Helm, Chest, Gloves, Boots
  Jewelry: Ring, Amulet
  Consumables: Potions, Scrolls
```

### Dungeon Generation
```
Floor Themes by Depth:
  1-5:   Cathedral (undead, fallen)
  6-10:  Catacombs (goatmen, skeletons)
  11-15: Caves (beasts, demons)
  16-20: Hell (demons, bosses)

Room Types:
  - Standard (enemies)
  - Treasure (chests, higher loot)
  - Shrine (temporary buffs)
  - Boss (floor guardian)
```

### Meta-Progression
```
After each run, earn:
  - Soul Essence (based on depth reached)
  
Spend on:
  - Unlock new classes
  - Town upgrades (better starting gear, new shops)
  - Permanent stat bonuses
  - New item types in loot pool
```

### Technical Requirements
- **Database**: Character builds, meta-progression, leaderboards
- **Instance Management**: Per-party dungeon instances
- **Real-time Sync**: Player positions, combat in multiplayer

### Data Model
```
players_meta:
  - user_id
  - soul_essence
  - unlocks (JSON)
  - highest_floor
  - total_runs

runs:
  - id, created_at
  - dungeon_seed
  - floor_reached
  - players (JSON array of user_ids)

run_characters:
  - run_id, user_id
  - class, level
  - stats, skills (JSON)
  - equipment (JSON)
  - status (alive|dead)

town:
  - user_id
  - upgrades (JSON)
  - stash (JSON - items between runs)
```


---

# GAME: Fortress
**Genre:** Colony Simulation
**Type:** Multiplayer (Shared World)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Epic

### Core Concept
A multiplayer colony simulation inspired by Dwarf Fortress but with simplified mechanics and cleaner ASCII visuals. Players establish and manage fortresses, with all fortresses existing in the same world. Trade, diplomacy, and conflict between player colonies.

### Key Features
- **Colony Management**: Direct dwarves to dig, build, craft
- **Resource Gathering**: Mining, woodcutting, farming
- **Production Chains**: Raw materials -> crafted goods
- **Dwarf Needs**: Food, drink, sleep, happiness
- **Threats**: Periodic invasions, creatures, cave-ins
- **Multiplayer World**: Trade and interact with other fortresses
- **Simplified UI**: Cleaner than DF, menu-driven actions

### Core Systems
```
DWARVES
  - Skills: Mining, Crafting, Farming, Combat, etc.
  - Needs: Hunger, Thirst, Rest, Social
  - Moods: Happy dwarves work faster
  
RESOURCES
  - Stone: Walls, furniture, crafts
  - Wood: Construction, fuel
  - Metal: Tools, weapons, armor
  - Food: Farms, hunting, fishing
  - Drink: Brewing required

PRODUCTION
  Wood + Workshop = Furniture
  Ore + Smelter = Metal bars
  Metal + Forge = Tools/Weapons
  Grain + Still = Alcohol
```

### Simplified Threat System
```
Instead of full DF simulation:
  - Seasonal raids (predictable)
  - Depth-based creatures (deeper = scarier)
  - Simple happiness threshold (below = tantrum)
```

### Technical Requirements
- **Database**: Fortress state, world map, dwarf states
- **Tick System**: Actions resolve over time
- **World Persistence**: All fortresses in shared map

### Data Model
```
world_map:
  - x, y, terrain, claimed_by (user_id nullable)

fortresses:
  - id, user_id, name
  - position_x, position_y
  - z_levels_dug
  - resources (JSON)
  - buildings (JSON)
  
dwarves:
  - id, fortress_id, name
  - skills (JSON)
  - current_task
  - needs (hunger, thirst, rest, mood)
  - status

tasks:
  - id, fortress_id, type
  - priority, assigned_dwarf_id
  - target_location, progress
```


---

# GAME: Summit
**Genre:** Cooperative Climbing / Survival
**Type:** Multiplayer (1-4 players Co-op)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Moderate

### Core Concept
An online co-op climbing game where the slightest mistake can spell your doom. As a group of lost nature scouts, your only hope of rescue from a mysterious island is to scale the mountain at its center. The mountain changes every 24 hours. Help each other up ledges, place ropes and climbing spikes to make the way easier for those who come after, scavenge for questionable food to survive, and manage your injuries carefully - every setback limits your stamina, making it harder to climb. Do you have what it takes to reach the Summit?

### Key Features
- **Co-op Climbing**: Up to 4 players (3 friends + you)
- **4 Biomes**: Each with life-threatening obstacles
- **30 Questionable Foods**: Scavenge to survive (benefits AND side effects)
- **Climbing Items**: Ropes, spikes, pitons, the mysterious Anti-Rope, and more
- **Campfires & Marshmallows**: Rest zones between biomes
- **Character Customization**: Unlock cosmetics for your scout
- **Dozens of Badges**: Show off your survival prowess
- **Daily Mountains**: New layout every 24 hours

### Display Layout
```
╔═══════════════════════════════════════════════════════════════╗
║  SUMMIT - Day 47 - Alpine Biome                    🏕️ Campfire ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║    ▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲                           ║
║   ▲░░░░░░░▲▲▲░░░░░░░░▲▲▲▲░░░░░▲▲▲                           ║
║  ▲░░░░░░░░░▲░░░░░@░░░░░▲░░░░░░░▲▲   @=You  ◆=Scout          ║
║  ▲░░░╔═════════╗░░░░░░░░░░░░░░░░▲   ▲=Rock █=Ledge          ║
║  ▲░░░║🔥CAMP  🔥║░░░░░░░░░░░░░░░░▲   │=Rope ░=Climbable      ║
║  ▲░░░╚═════════╝░░░░░◆░░░░░░░░░▲▲   ≈=Ice  ~=Wind           ║
║  ▲▲░░░░░░░░░░░░░░░░░░│░░░░░░░▲▲▲                            ║
║  ▲▲▲░░░░░░░░░░░░░░░░░│░░░░░▲▲▲▲▲                            ║
║  ▲▲▲▲▲░░░░░░░░░░░░░░░│░░░▲▲▲▲▲▲▲                            ║
║  ▲▲▲▲▲▲▲░░░░░░░░░░░░░│░▲▲▲▲▲▲▲▲▲                            ║
║                                                               ║
╠═══════════════════════════════════════════════════════════════╣
║  YOU: Scout123            │  PARTNER: ClimbBro42              ║
║  Stamina: ████████░░░░░░░ │  Stamina: ██████████████░░        ║
║  Max:     ███████████████ │  Max:     ████████████████        ║
║  Status:  Hungry, Cold    │  Status:  OK                      ║
║  Items:   Rope(2) Piton(1)│  Items:   Snack(3) Rope(1)        ║
╠═══════════════════════════════════════════════════════════════╣
║  [WASD] Move  [E] Use Item  [R] Deploy Rope  [P] Place Piton  ║
║  [SPACE] Grab/Climb  [H] Help Partner  [TAB] Inventory        ║
╚═══════════════════════════════════════════════════════════════╝
```

### Stamina System
```
CURRENT STAMINA (regenerates when resting)
  - Depletes with every move/action
  - Regenerates slowly when holding onto something stable
  - Regenerates quickly at pitons and campfires
  - Reaches zero = you fall

MAXIMUM STAMINA (permanent within run)
  - Reduced when you fall
  - Reduced by injuries
  - Reduced by status effects (poison, exhaustion)
  - Every setback limits your stamina, making it harder to climb
  - CANNOT be restored (except revival at campfire altar)

Status Effects (reduce max stamina over time):
  - Hungry: -1 max stamina per 30 seconds
  - Cold: -1 max stamina per 20 seconds (Alpine/Volcanic)
  - Poisoned: -2 max stamina per 10 seconds
  - Exhausted: Stamina regeneration halved
```

### The 4 Biomes
```
BEACH (Difficulty: Easy)
  - Sandy cliffs, palm trees, gentle slopes
  - Tutorial area, forgiving falls
  - Hazards: Crabs, loose sand, tide pools
  - Learn the basics before the real climb begins

JUNGLE (Difficulty: Medium)
  - Dense vegetation, vines, waterfalls
  - Reduced visibility, slippery surfaces
  - Hazards: Poison plants, sudden drops, snakes
  - First real test of your climbing skills

ALPINE (Difficulty: Hard)  
  - Snow, ice, exposed rock faces
  - Cold status effect if not moving
  - Hazards: Ice slides, brittle rocks, avalanches, wind gusts
  - The mountain takes no prisoners

VOLCANIC (Difficulty: Extreme)
  - Unstable terrain, lava vents, ash clouds
  - Heat and toxic fumes
  - Hazards: Collapsing platforms, eruptions, steam vents
  - Final push to the Summit
```

### Climbing Items
```
ROPE
  - Deploy to create climbable path
  - Hangs down from attachment point
  - Make the way easier for those who come after
  - Found in luggage scattered across mountain

PITON / CLIMBING SPIKE
  - Place on any climbable surface
  - Creates rest point (fast stamina regen)
  - Essential for the harder biomes
  - Share with teammates

ROPE CANNON
  - Fire rope at distant anchor point
  - Creates long-distance path
  - Rare but incredibly useful

ANTI-ROPE (Mysterious)
  - ???
  - Effects unknown until discovered
  - One of the mountain's secrets

GRAPPLING HOOK
  - Pull yourself to anchor points
  - Can grab distant items
  - Limited uses

CLIMBING GLOVES
  - Reduce stamina cost on rough surfaces
  - Prevent slipping on ice
  - Durability decreases with use

SAFETY HARNESS
  - Reduces fall damage
  - Won't prevent falls, but softens the blow
```

### The 30 Questionable Foods
```
ENERGY & STAMINA:
  1.  Energy Drink      + Instant 50% stamina, - crash after 60s
  2.  Sports Gel        + Steady stamina regen for 30s
  3.  Coffee Thermos    + Stamina regen boost, - jitters (shaky movement)
  4.  Protein Bar       + 25% stamina, satisfies hunger
  5.  Trail Mix         + 20% stamina, light hunger relief

HUNGER & HEALING:
  6.  Mystery Meat      + 30% stamina, satisfies hunger, ? 20% food poisoning
  7.  Canned Beans      + Full hunger relief, - gas (makes noise, alerts hazards)
  8.  Dried Fruit       + Cures hunger, safe choice
  9.  Beef Jerky        + Hunger + 15% stamina
  10. MRE Pack          + Full hunger, 40% stamina, - heavy (slows you)

SPECIAL EFFECTS:
  11. Lollipop          + Unlimited stamina 15s, - severe exhaustion after
  12. Milk              + 10s invulnerability, extremely rare
  13. Hot Cocoa         + Cures cold, 25% stamina, cozy feeling
  14. Warm Soup         + Cures cold, hunger, and 30% stamina

QUESTIONABLE CHOICES:
  15. Strange Mushroom  + May restore 20% MAX stamina (!), ? may hallucinate
  16. Suspicious Berry  ? Random: healing OR poison OR energy boost
  17. Mystery Can       ? Could be anything
  18. Gas Station Sushi + 50% stamina if it doesn't kill you (50/50)
  19. Week-Old Sandwich + Desperate times, 30% poison chance
  20. Found Candy       + Small stamina boost, probably fine

CAMPFIRE SPECIALS:
  21. Roasted Marshmallow  + 15% stamina, morale boost, tastes like victory
  22. S'more             + 25% stamina, requires chocolate + graham + marshmallow
  23. Hot Dog            + Hunger relief, 20% stamina
  24. Campfire Coffee    + Major stamina regen, requires rest to drink

RARE & EXOTIC:
  25. Golden Apple       + Restores 30% MAX stamina, legendary
  26. Ancient Ration     + Full restore but tastes terrible
  27. Glowing Fruit      + ??? Found only in volcanic biome
  28. Scout's Emergency Chocolate  + Clutch 40% stamina restore
  29. Mystery Pill       ? Massive gamble - could be miracle or disaster
  30. The Forbidden Snack  + Found at Summit, grants badge, effects unknown
```

### Campfires & Marshmallows
```
CAMPFIRE MECHANICS:
  - Located between each biome
  - Safe zone - no hazards can reach you
  - Rapid stamina regeneration
  - Cook food for enhanced effects
  - Roast marshmallows for stamina boost
  - Revive downed teammates at altar

MARSHMALLOW ROASTING:
  - Mini-game: hold over fire without burning
  - Perfect roast = 20% stamina + morale
  - Burnt = 5% stamina, sad
  - On fire = 0 stamina, very sad
  - Can make S'mores if you have ingredients
```

### Cooperative Mechanics
```
HELPING UP
  - Stand at ledge edge, press H
  - Partner can grab your hand
  - Pull them up (costs YOUR stamina)
  - Need a Hand? You'll need to rely on your friends!

ITEM SHARING
  - Drop items for teammates
  - Coordinate who carries what
  - Share food when teammate is low

ROPE PLACEMENT
  - Place ropes and climbing spikes for those who come after
  - "Trail-blazer" role opens the path
  - Teamwork makes the climb possible

REVIVAL
  - Fallen teammate becomes "downed"
  - Carry them to campfire altar
  - Revival restores them at 50% max stamina
  - If all scouts downed = expedition over
```

### Hazards by Biome
```
BEACH:
  - Crabs (pinch, minor damage)
  - Loose sand (sliding)
  - Tide pools (slip hazard)
  - Seagulls (steal food!)

JUNGLE:
  - Poison plants (poison status)
  - Snakes (bite = poison)
  - Sudden drops (hidden cliffs)
  - Slippery vines (fall risk)

ALPINE:
  - Wind gusts (push off ledges)
  - Ice slides (uncontrollable descent)
  - Brittle rocks (collapse when grabbed)
  - Avalanches (scripted events, find cover!)
  - Extreme cold (status effect)

VOLCANIC:
  - Steam vents (periodic damage)
  - Collapsing platforms (timing puzzles)
  - Lava flows (instant death)
  - Ash clouds (reduced visibility)
  - Eruption events (chaos!)
```

### Character Customization
```
UNLOCKABLE COSMETICS:
  Scout Uniforms:
    - Classic Green (default)
    - Wilderness Brown
    - Arctic White
    - Volcanic Red
    - Golden Scout (prestige)

  Hats & Headgear:
    - Scout Cap
    - Headlamp
    - Climbing Helmet
    - Bandana
    - Mysterious Hood

  Backpacks:
    - Standard Issue
    - Oversized
    - Minimalist
    - Decorated with badges
    - Tiny (cosmetic only)

  Accessories:
    - Sunglasses
    - Goggles
    - Face Paint
    - Scarf
    - Lucky Charm

  Rope Colors:
    - Standard Tan
    - Safety Orange
    - Camo Green
    - Neon Pink
    - Rainbow
```

### Badges (Dozens to Earn)
```
PROGRESSION BADGES:
  🏔️ First Summit - Complete any mountain
  🏔️🏔️ Veteran Scout - Complete 10 mountains
  🏔️🏔️🏔️ Mountain Master - Complete 50 mountains
  ⭐ All Biomes - Reach all 4 biomes in one run
  👑 True Scout - Complete with no deaths

SKILL BADGES:
  ⚡ Speed Climber - Summit in under 15 minutes
  🪶 Featherfoot - Summit without falling
  💪 Solo Summit - Complete alone
  🤝 Team Player - Revive 10 teammates total
  🧗 Trailblazer - Place 100 ropes total

FOOD BADGES:
  🍽️ Adventurous Eater - Try 15 different foods
  🍽️🍽️ Iron Stomach - Try all 30 foods
  🤢 Survivor - Recover from food poisoning 5 times
  🔥 Marshmallow Master - Perfect roast 20 times
  🍫 S'more Connoisseur - Make 10 s'mores

DISCOVERY BADGES:
  🔍 Explorer - Find a secret area
  📦 Scavenger - Open 100 luggage containers
  ❓ Anti-Rope User - Discover what Anti-Rope does
  🌋 Lava Surfer - Survive volcanic biome hazard
  🏆 The Forbidden Snack - Eat it at the Summit

CHALLENGE BADGES:
  ☠️ Barely Alive - Summit with 1% max stamina
  🏃 Speedrun - Summit in under 10 minutes
  🎒 Minimalist - Summit using only 3 items
  🦸 The Carry - Revive same teammate 5 times in one run
  🌟 Perfectionist - Collect all other badges
```

### Technical Requirements
- **Database**: Daily seeds, player stats, badge progress, cosmetic unlocks
- **Real-time Sync**: Player positions, item states, hazard triggers
- **Procedural Generation**: Deterministic mountain from daily seed
- **Physics Simulation**: Stamina-based movement, fall detection

### Data Model
```
daily_mountains:
  - date (PK)
  - seed
  - difficulty_modifier
  - special_events (JSON)

runs:
  - id, date, created_at
  - players (JSON array of user_ids)
  - status (active|summit|failed)
  - duration_seconds
  - biome_reached (1-4)

run_players:
  - run_id, user_id
  - max_stamina_remaining
  - falls_count
  - items_used (JSON)
  - foods_eaten (JSON)
  - reached_summit (bool)
  - revives_given, revives_received

player_stats:
  - user_id
  - total_runs, summits, falls
  - fastest_summit_seconds
  - foods_tried (bitmask for 30 foods)
  - badges_earned (JSON)
  - cosmetics_unlocked (JSON)
  - equipped_cosmetics (JSON)

leaderboard_daily:
  - date, user_id
  - best_time_seconds
  - party_size
```

### Run Flow
```
1. STAGING
   - Scouts join lobby (up to 4)
   - Customize characters
   - See today's mountain preview
   - Ready up when prepared

2. CRASH LANDING
   - Cinematic: plane crashes on beach
   - Scouts spawn at wreckage
   - Scavenge initial supplies from luggage
   - "Your only hope of rescue is to reach the Summit!"

3. ASCENT
   - Climb through 4 biomes
   - Scavenge, help teammates, survive
   - Rest at campfires, roast marshmallows
   - The mountain changes every day

4. SUMMIT OR DOOM
   - Reach the Summit = rescue helicopter!
   - All scouts downed = expedition failed
   - Every mistake can spell your doom

5. RESULTS
   - Show stats (falls, items, foods, time)
   - Award badges earned
   - Unlock cosmetics
   - Update daily leaderboards
```


---

# GAME: Tanks: Blitzkrieg
**Genre:** Real-time Multiplayer Combat
**Type:** Multiplayer (2-6 players)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
Based on Flash Attack V2.2 by Tim Stryker (Galacticomm, 1989). Players command military bases on procedurally generated islands covered with mountains, forests, and lakes. Each player controls 4 tanks simultaneously via split-screen viewports, plus their base view. Objective: destroy all enemy bases. Real-time combat with phasers, mines, lasers, neutrons, and seekers. Features ghost mode for eliminated players, in-game radio communications, and defensive shields.

**Reference Materials Available**: Original FA22.DOC documentation and gameplay screenshot provided.

### Key Features
- **Split-Screen Display**: Base window + 4 tank windows simultaneously
- **Procedural Islands**: 65,536 unique islands (seed-based), varying in size, terrain density
- **Base System**: 8 pods form each base; damage threshold varies by player count
- **Tank Control**: Select 1-4 tanks, move via numpad, return to base for repairs/resupply
- **Weapons**: Phasers (tank), Mines (tank), Pods (tank), Lasers (base), Neutrons (base), Seekers (base)
- **Defensive Shields**: 8 uses per game, 2-second invulnerability
- **Condition System**: GREEN → YELLOW → RED → GONE
- **Ghost Mode**: Eliminated players become spirits, can scout and communicate
- **Radio Communications**: Real-time chat, jamming possible
- **Post-Game Review**: Full island view with all mines revealed

### Display Layout (from original)
```
┌─────────────────────────────┬─────────────────────────────┐
│          BASE               │  Tank 1    Phasers    Cond  │
│  ≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈   │  ┌─────┐   ████████  GREEN │
│  ≈≈≈≈≈≈≈≈nn≈≈≈≈≈nnnnn≈≈≈   │  │nn   │   Mines  Pods  X  Y│
│  ≈≈≈≈≈≈≈≈≈nn≈≈≈≈≈≈nn≈≈≈≈   │  │nn≈≈ │   █████  ███  95 69│
│  ≈≈≈≈≈≈≈≈≈≈≈≈≈nn≈≈≈nn≈≈≈   │  └─────┘         Fuel: 500  │
│  ≈≈≈≈≈≈≈≈≈nn≈≈nn≈≈≈≈≈≈≈≈   ├─────────────────────────────┤
│  ≈≈████≈≈nn≈≈≈≈≈≈≈≈≈≈≈≈≈   │  Tank 2    Phasers    Cond  │
│  ≈≈████≈≈≈≈≈≈≈≈≈≈≈nnnnn≈   │  ┌─────┐   ████████  GREEN │
│  ≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈nn≈≈   │  │  @  │   Mines  Pods  X  Y│
│  ≈nnnnn≈≈≈≈≈≈≈≈nnnnn≈≈≈≈   │  │     │   █████  ███ 104 69│
│  ≈≈≈nn≈≈≈≈≈≈≈≈≈≈≈nn≈≈≈≈≈   │  └─────┘         Fuel: 500  │
│  ≈≈≈≈≈≈≈nn≈≈≈≈≈≈≈≈≈≈≈≈≈≈   ├─────────────────────────────┤
│                             │  Tank 3    Phasers    Cond  │
│  Lasers   Neutrons   Cond   │  ┌─────┐   ████████  GREEN │
│   24        ██       GREEN  │  │nnnnn│   Mines  Pods  X  Y│
├─────────────────────────────│  │≈≈≈≈≈│   █████  ███  95 73│
│ Tanks  DS   X    Y          │  └─────┘         Fuel: 500  │
│  18    8   99   71          ├─────────────────────────────┤
│ Angle   Range   Seekers     │  Tank 4    Phasers    Cond  │
│  90.0   10.0      9         │  ┌─────┐   ████████  GREEN │
├─────────────────────────────│  │nnnnn│   Mines  Pods  X  Y│
│ Communications              │  │     │   █████  ███ 104 69│
│ >                           │  └─────┘         Fuel: 500  │
└─────────────────────────────┴─────────────────────────────┘
```

### Terrain Types
```
≈ or . = Open ground (full movement)
n or # = Mountain (impassable, blocks lasers, reduced to rubble by laser)
^ = Forest/Trees (passable, destroyed by phasers)
~ = Water/Lake (passable but tank drowns immediately after)
@ = Your tank
█ = Base pod (8 form a base in 2x4 pattern)
+ = Your mine (visible to you only)
  = Enemy mine (invisible until post-game)
* = Laser beam
░ = Rubble (1/8 chance to enter)
▓ = Neurubble (damages tank on contact, impassable)
```

### Map Specifications
```
Coordinate System:
  X: 0-190 (horizontal, left to right)
  Y: 0-70 (vertical, top to bottom)
  Origin (0,0): Ocean, upper-left corner

Island Generation (65536 variants):
  - Size varies (40% larger in V2.2)
  - Terrain density varies
  - Some islands segmented (disjoint parts)
  - Mountains, forests, lakes randomly distributed
```

### Base Structure
```
Base layout (8 pods):
    ████
    ████

Destruction thresholds by player count:
  2 players: 1 pod destroyed = eliminated
  3 players: 2 pods destroyed = eliminated
  4 players: 3 pods destroyed = eliminated
  5 players: 4 pods destroyed = eliminated
  6 players: 5 pods destroyed = eliminated

As players are eliminated, thresholds rise (ripple elimination possible)
```

### Weapons Systems

**PHASERS (Tank)**
- Fire in 8 directions via numpad (SHIFT+1-9, excluding 5)
- Straight line until hitting obstacle or window edge
- Destroys: Trees, tanks (1 condition level), pods
- Does NOT destroy: Mountains, rubble, neurubble, water
- Can detonate mines at window edge
- Limited ammo, resupply at base center

**MINES (Tank)**
- Press 'M' then move away to plant
- Visible to owner as red '+', invisible to enemies
- 14 mines per tank, unlimited resupply at base
- Hitting mine = 2 condition levels damage (GREEN→RED, else→GONE)
- Tactics: Double-deep lines, rings around base, random exploration markers

**PODS (Tank)**
- Press 'P' then move away to place
- Creates decoy bases, concealment patterns, or repairs damaged base
- 500 fuel units per tank

**LASERS (Base)**
- 24 shots per game (no resupply)
- Unlimited range, set angle via 'A' key (0-360°, clockwise from north)
- Fire with 'L' key
- Vaporizes everything EXCEPT mountains (which become rubble)
- Only one beam in air at a time
- Aspect ratio: squares taller than wide, affects visual angles

**NEUTRONS (Base)**
- Area effect weapon (~11x5 square explosion zone)
- Set angle 'A' and range 'R', fire with 'N'
- Falling whistle sound during flight (game continues)
- Destroys everything, leaves random terrain + neurubble
- Limited supply

**SEEKERS (Base)**
- 9 per game
- Press 'S' when enemy tank visible in base window
- Spirals outward from base center
- Hits ANY tank (friend or foe)
- Safe zones: 2 center base squares + 8 "shoulder" squares
- Only one seeker airborne at a time
- Can be outrun or dodged by timing

**DEFENSIVE SHIELDS (Base)**
- 8 uses per game
- Press 'D' for ~2 seconds invulnerability
- Base blinks blue while active
- Critical for surviving neutron attacks

### Condition System
```
GREEN  - Full health
YELLOW - Damaged (movement/weapons still normal)
RED    - Critical (50% chance movement commands fail)
GONE   - Destroyed

Damage sources:
  Phaser hit: -1 level
  Mine: -2 levels
  Neurubble contact: -1 level
  Seeker: -1 level (assumed)

Repair: Return tank to center 2 squares of your base
```

### Tank Management
```
Selection:
  'T' + 1/2/3/4 = Select single tank
  'TT' + digit = Gang up additional tank
  'TA' = Select all four tanks

Resources per tank:
  - Fuel: 500 units (1 per move attempt)
  - Mines: 14
  - Phasers: Limited
  - Pods: For building

Tanks per game: 18 total (replacements when destroyed)
When out of tanks: window turns red, unusable (unless ghost)
```

### Ghost Mode (3+ player games)
```
When eliminated (not last 2 players):
  - All 4 windows become "ghost windows"
  - Free movement through all terrain
  - Cannot be damaged
  - Can see all bases, tanks, terrain
  - Can communicate via radio
  - Purpose: Help speed up endgame as "artillery spotter"

Tactics:
  - Guide remaining players' laser/neutron aim
  - Reveal base locations
  - Can mislead, jam, or help any player
  - Players can request ghost assistance or ignore
```

### Communications
```
Press 'C' to enter transmit mode
Type message, press Enter to send
All players receive all transmissions
Jamming possible (garbage characters, backspaces)
```

### Controls Summary
```
Movement:     Numpad 1-9 (NUMLOCK off), 5=center
Phasers:      SHIFT + Numpad 1-9
Tank Select:  T + 1/2/3/4, TT + digit (gang), TA (all)
Mine:         M (then move away)
Pod:          P (then move away)
Laser Angle:  A (enter degrees)
Laser Fire:   L
Neutron:      A (angle), R (range), N (fire)
Seeker:       S
Shields:      D
Communicate:  C (then type, Enter to send)
Sound Toggle: ALT-V
```

### Technical Requirements
- **Database**: Game sessions, player states, island seeds, unit positions
- **Real-time Networking**: Fast position/action sync (original used BBS modem)
- **Procedural Generation**: Deterministic island from 16-bit seed
- **Multi-viewport Rendering**: Base + 4 tank windows simultaneously
- **Sound System**: Distinct sounds for movement, weapons, alerts

### Data Model
```
games:
  - id, created_at
  - island_seed (0-65535)
  - status (lobby|active|review|complete)
  - winner_user_id

players_games:
  - game_id, user_id
  - base_x, base_y
  - base_pods_remaining (8 max)
  - condition (green|yellow|red|gone|ghost)
  - lasers_remaining (24 max)
  - neutrons_remaining
  - seekers_remaining (9 max)
  - shields_remaining (8 max)
  - tanks_remaining (18 max)

tanks:
  - id, game_id, user_id, slot (1-4)
  - x, y
  - condition (green|yellow|red|gone)
  - fuel (500 max)
  - mines (14 max)
  - phasers_remaining
  - pods_remaining

mines:
  - game_id, user_id
  - x, y

structures:
  - game_id, user_id
  - type (pod|decoy_pod)
  - x, y

terrain_changes:
  - game_id, x, y
  - new_type (rubble|neurubble|empty)
```


---

# GAME: Master of Cygnus
**Genre:** 4X Strategy
**Type:** Multiplayer (Turn-based Async)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
A Master of Orion 1 inspired 4X game. Players lead space-faring civilizations to dominate the Cygnus constellation through exploration, expansion, exploitation, and extermination. Turn-based with async multiplayer support - if both players are online, turns resolve immediately; otherwise wait up to 3 days per turn.

### Key Features
- **Galaxy Generation**: Procedural star systems, one planet per star (like MOO1)
- **Colony Management**: Build structures, manage population
- **Technology Research**: Tech trees unlock ships, buildings, weapons
- **Ship Design**: Customize ships with researched components
- **Fleet Combat**: Tactical space battles
- **Diplomacy**: Treaties, trade, war declarations
- **Victory Conditions**: Conquest, Council vote, Technology

### Galaxy Structure
```
Stars: 20-50 per game (size setting)
Each star has exactly 1 planet (simplified like MOO1)

Planet Types:
  Terran   - Ideal, high pop capacity
  Ocean    - Good, needs tech
  Arid     - Moderate capacity
  Tundra   - Poor capacity
  Barren   - Colonizable with tech
  Toxic    - Advanced tech only
  Gas Giant - Uninhabitable, fuel harvesting only
```

### Colony Buildings
```
Basic:
  Colony Base, Factory, Lab, Farm
  
Advanced:
  Planetary Shield, Shipyard, Deep Core Mine
  
Wonders:
  Hyperspace Comm, Orbital Habitat, Star Forge
```

### Technology
```
Fields:
  Propulsion - Ship speed, range
  Weapons - Beam, missile, bomb damage
  Shields - Defense technology
  Planetology - Terraforming, farming
  Construction - Building speed, ship hulls
  Computers - Targeting, scanning
```

### Ship Design
```
Hull sizes: Scout, Destroyer, Cruiser, Battleship, Dreadnought

Components:
  - Engines (speed, range)
  - Weapons (beams, missiles, bombs)
  - Shields (defense)
  - Specials (cloak, teleporter, etc.)

Design limited by hull space and tech level
```

### Multiplayer Model
```
Game States:
  - WAITING_FOR_PLAYERS (lobby)
  - IN_PROGRESS
  - COMPLETED

Turn Resolution:
  - All players submit orders
  - When all submitted OR 72hr timeout -> resolve
  - Players who timeout 3x forfeit
  - If both online, can resolve immediately
```

### Technical Requirements
- **Database**: Game state, player empires, tech trees, fleets
- **Turn Processor**: Batch resolve all orders
- **Combat Resolver**: Deterministic battle outcomes

### Data Model
```
games:
  - id, name, galaxy_seed
  - settings (JSON)
  - turn_number, status
  - turn_deadline

empires:
  - id, game_id, user_id
  - race_name, color
  - research_allocation (JSON)
  - known_techs (JSON)
  - treasury

stars:
  - id, game_id, x, y, name
  - planets (JSON)
  
colonies:
  - id, star_id, planet_index
  - owner_empire_id
  - population, buildings (JSON)
  - production_queue (JSON)

fleets:
  - id, empire_id
  - location_star_id, destination_star_id
  - ships (JSON)
  - eta_turns

ship_designs:
  - id, empire_id, name
  - hull_size, components (JSON)

orders:
  - game_id, turn_number, empire_id
  - orders_data (JSON)
  - submitted_at
```


---

# GAME: Chess
**Genre:** Board Game
**Type:** Multiplayer (Turn-based Async)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Moderate

### Core Concept
Classic chess with both real-time and asynchronous play. Start a game alone with white's first move, wait for opponent, or join existing games. Full chess rules including castling, en passant, promotion. 3-day move timeout results in forfeit.

### Key Features
- **Full Chess Rules**: All standard moves and special rules
- **Async Play**: Make move, wait for opponent, continue later
- **Game Lobby**: See open games, create new ones
- **Move Notation**: Standard algebraic notation
- **Time Controls**: 3-day timeout per move (async)
- **Rating System**: ELO-based rankings
- **Game History**: Review past games move by move

### ASCII Board Display
```
    a b c d e f g h
  ┌─────────────────┐
8 │ ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ │ 8
7 │ ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ │ 7
6 │ · · · · · · · · │ 6
5 │ · · · · · · · · │ 5
4 │ · · · · ♙ · · · │ 4
3 │ · · · · · · · · │ 3
2 │ ♙ ♙ ♙ ♙ · ♙ ♙ ♙ │ 2
1 │ ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ │ 1
  └─────────────────┘
    a b c d e f g h

White to move. Enter move (e.g., Nf3): _
```

### Game States
```
WAITING   - White moved, waiting for black to join
ACTIVE    - Game in progress
CHECK     - King in check
CHECKMATE - Game over, winner determined  
STALEMATE - Draw
DRAW      - By agreement, repetition, or 50-move rule
FORFEIT   - Timeout or resignation
```

### Technical Requirements
- **Move Validation**: Legal move checker
- **Database**: Games, moves, players, ratings
- **Timeout Checker**: Background job for forfeits

### Data Model
```
chess_games:
  - id, created_at
  - white_user_id, black_user_id (nullable until joined)
  - status
  - result (white|black|draw nullable)
  - last_move_at
  - pgn (complete game notation)

chess_moves:
  - game_id, move_number
  - player (white|black)
  - notation (e.g., "e4", "Nxf7+")
  - fen_after (board state)
  - timestamp

chess_ratings:
  - user_id
  - rating (default 1200)
  - games_played
  - wins, losses, draws

open_games:
  - game_id (games waiting for opponent)
  - first_move (what white played)
```


---

# GAME: Xodia the Living MUD
**Genre:** MUD / Interactive Fiction / TTRPG
**Type:** Multiplayer (Persistent)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Epic

### Core Concept
A MUD where the LLM functions as a Dungeon Master, and the game mechanics provide the rules and foundation for what can be done - like a persistent session of D&D. The system maintains context for events, inventory, health, world state, and character progression. Natural language input is processed through a micro-LLM for intent classification, mapped to scripted game actions. The world grows organically as players explore and interact, with all discoveries becoming permanent canon.

### Key Features
- **LLM as Dungeon Master**: Describes scenes, reacts to actions, maintains narrative consistency
- **Game Mechanics as Rules**: Stats, inventory, combat, skills provide the "rulebook"
- **Persistent World Graph**: Every location, NPC, item persists and can be revisited
- **Natural Language Processing**: Micro-LLM classifies intent → mapped to canonical actions
- **Shared Reality**: All players exist in same world, actions affect everyone
- **Canonical Consistency**: MCP tools maintain names, locations, items, event history
- **Story Export**: Player journeys can be exported as narrative prose

### Architecture

**MCP Tools Required**
```
World State MCP:
  - get_location(location_id) → description, exits, npcs, items
  - create_location(parent_id, direction, details) → new location_id
  - update_location(location_id, changes)
  - get_connected_locations(location_id) → list of adjacent locations

Entity Registry MCP:
  - register_name(type, name) → unique_id (prevents duplicates)
  - lookup_entity(name_or_id) → entity details
  - list_entities(type, filters) → matching entities
  Types: character, npc, location, item, quest, event

Event Log MCP:
  - log_event(location_id, participants, action, outcome, timestamp)
  - get_events(location_id, time_range) → event history
  - get_character_history(character_id) → personal event log

Inventory/State MCP:
  - get_character(character_id) → stats, inventory, status, location
  - update_character(character_id, changes)
  - transfer_item(item_id, from, to)
```

**Natural Language Processing Pipeline**
```
Player Input: "I want to look around for hidden doors"
       ↓
Micro-LLM Intent Classifier:
  → Action: SEARCH
  → Target: ROOM
  → Modifier: hidden, doors
       ↓
Game Engine:
  → Roll perception check
  → Query room for hidden features
  → Determine success/failure
       ↓
DM LLM Narrator:
  → Receives: action, roll result, room state, character state
  → Generates: Vivid narrative description of outcome
       ↓
Player Output: "You run your fingers along the cold stone walls..."
```

### Game Mechanics (The Rulebook)

**Character Stats**
```
Primary:
  STR - Physical power, carry capacity, melee damage
  DEX - Agility, dodge, ranged attacks, stealth
  CON - Health, stamina, poison resistance
  INT - Magic power, puzzle solving, lore knowledge
  WIS - Perception, willpower, divine magic
  CHA - Persuasion, intimidation, bartering

Derived:
  HP = CON * 5 + Level * 3
  MP = INT * 3 + WIS * 2
  AC = 10 + DEX modifier + armor
  Initiative = DEX modifier + bonuses
```

**Action Resolution**
```
Skill checks: d20 + modifier vs DC
Combat: d20 + attack bonus vs AC
Damage: weapon dice + STR/DEX modifier
Saving throws: d20 + stat modifier vs effect DC

All rolls are server-side, narrated by DM LLM
```

**Canonical Actions (Micro-LLM maps to these)**
```
Movement:
  GO [direction/location]
  ENTER [portal/door/building]
  CLIMB [target]
  SWIM [direction]

Interaction:
  LOOK [target] / EXAMINE [target]
  TAKE [item]
  DROP [item]
  USE [item] [target]
  GIVE [item] TO [npc/player]
  OPEN [container/door]
  SEARCH [area/container]

Combat:
  ATTACK [target] WITH [weapon]
  CAST [spell] AT [target]
  DEFEND / DODGE
  FLEE [direction]

Social:
  TALK TO [npc]
  ASK [npc] ABOUT [topic]
  PERSUADE [npc] TO [action]
  INTIMIDATE [npc]
  TRADE WITH [npc]

Character:
  INVENTORY
  STATUS / CHARACTER
  EQUIP [item]
  REST / CAMP
  LEARN [skill/spell]
```

### World Structure

**The Land of Xodia**
```
Core Regions (Anchors - pre-defined):
  - Misthollow Village (starting zone)
  - The Whispering Woods (dark forest, fey creatures)
  - Saltmere Port (coastal trade hub, pirates)
  - The Sunken Kingdom (underwater ruins, ancient magic)
  - Dragon's Teeth Mountains (dwarven holds, dragons)
  - The Obsidian Desert (nomads, buried temples)
  - The Spire of Eternity (endgame goal)

Between anchors: Generated by DM LLM, becomes permanent
Each location tracks: description, exits, NPCs, items, events
```

**World Graph**
```
Nodes: Locations with unique IDs
Edges: Directional connections (north, south, portal, etc.)

When player moves to undefined direction:
  1. DM LLM generates new location (consistent with region)
  2. MCP registers location with unique ID
  3. Location becomes permanent world canon
  4. Other players can now visit this location
```

### Narrative Consistency

**DM LLM Context Window**
```
For each narration, DM receives:
  - Character state (stats, inventory, conditions)
  - Current location (description, contents, recent events)
  - Recent conversation history
  - Relevant world lore
  - Nearby NPC personalities and states
  - Active quests involving this character
  - Time of day, weather, ambient conditions
```

**Consistency Rules**
```
1. Named entities are ALWAYS looked up via MCP before use
2. New names MUST be registered before appearing in narration
3. Events are logged and can be referenced by future narrations
4. NPCs remember past interactions (stored in event log)
5. World state changes persist (broken door stays broken)
6. Dead NPCs stay dead (unless resurrection mechanics)
```

### Multiplayer Interaction
```
Multiple players in same location:
  - See each other's actions in real-time
  - Can interact, trade, party up
  - Combat can be PvP or cooperative

Players in different locations:
  - Actions may have distant effects (quest completion, etc.)
  - Can communicate via in-game messaging (letters, telepathy items)
```

### Story Export
```
At any point, player can export their journey:
  - All visited locations compiled
  - All significant actions and outcomes
  - NPC interactions and dialogue
  - Combat encounters and results
  
DM LLM generates:
  - Connecting narrative prose
  - Chapter breaks at major events
  - Formatted as readable story/book
```

### Technical Requirements
- **MCP Integration**: World state, entity registry, event log, character state
- **Micro-LLM**: Fast intent classification (can be local model)
- **DM LLM**: High-quality narrative generation (Claude API)
- **Database**: Graph database for world, relational for entities
- **Real-time**: WebSocket for multiplayer presence

### Data Model
```
locations:
  - id (unique), region_id
  - name, description
  - created_by_event_id
  - exits (JSON: {direction: location_id})

entities:
  - id, type (npc|item|quest|character)
  - name (unique within type)
  - data (JSON: type-specific attributes)
  - location_id (nullable)
  - created_at

events:
  - id, timestamp
  - location_id
  - actor_id, action, target_id
  - outcome, narrative_text

characters:
  - id, user_id, name
  - stats (JSON)
  - inventory (JSON)
  - location_id
  - conditions (JSON: poison, bless, etc.)
  - quest_log (JSON)

conversations:
  - character_id, npc_id
  - history (JSON: past exchanges)
  - relationship_score
```

### The Story of Xodia (Campaign Setting)
```
Long ago, the world was whole, united under the Light of the First Flame.
Then came the Sundering - the Flame was shattered into seven shards,
each falling to a corner of the world.

The Spire of Eternity holds the key to reuniting the shards.
But the path is treacherous, guarded by ancient evils awakened by the Sundering.

You are a Seeker, one called by dreams to find the Spire.
Your journey begins in Misthollow Village, where the last Keeper awaits...

This is not a railroad - players can ignore the main quest entirely.
The world is alive with side quests, factions, mysteries, and danger.
Every choice matters. Every action is remembered.
```

---

# GAME: The Usurper
**Genre:** Multiplayer Hack-n-Slash RPG
**Type:** Multiplayer (Asynchronous)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
A multiplayer hack-n-slash RPG inspired by the classic Usurper BBS door game. Players explore over 100 dungeons stretching from the top of the mountain of Durunghins to its deepest levels. Work alone as a lonewolf adventurer or form teams. Fight monsters, battle other players, become king, fall in love, have children, or descend into steroid-fueled madness. Ancient legends say something so powerful lies at the bottom that it can alter the destiny of the world.

### Key Features
- **100+ Dungeons**: Massive dungeon system from mountain top to unfathomable depths
- **Solo or Teams**: Play as lonewolf or form clans/teams
- **PvP Combat**: Attack other players, raid them while they sleep
- **Political System**: Become King, rise to godhood
- **Romance System**: Fall in love, marry, have children
- **Drugs & Steroids**: Enhance your stats but risk mental stability
- **Equipment**: 10+ equipment slots (weapons, armor, rings, helmets, etc.)
- **Story Quests**: Optional quests for bonus experience and gold
- **The Supreme Being**: Ultimate boss at the deepest level

### Dungeon Structure
```
THE MOUNTAIN OF DURUNGHINS

Level 1-10:    Surface Caves (Training grounds)
Level 11-25:   Upper Dungeons (Intermediate challenges)
Level 26-50:   Deep Caves (Serious threats)
Level 51-75:   The Abyss (Nightmare creatures)
Level 76-100:  The Depths (No one returns)
Level ???:     The Bottom (The Supreme Being awaits)

Each level contains:
  - Multiple rooms to explore
  - Monsters scaling with depth
  - Treasure and equipment
  - Traps and hazards
  - Connections to adjacent levels
```

### Character Stats
```
Primary Stats:
  Strength     - Physical damage, carry capacity
  Agility      - Speed, dodge chance, critical hits
  Vitality     - Health points, poison resistance
  Intelligence - Magic power, puzzle solving
  Charisma     - NPC interactions, romance success
  
Special Stats:
  Mental Stability - Affected by drugs/steroids
  Reputation      - Good/Evil alignment
  Notoriety       - PvP fame

Derived:
  HP = Vitality * 10 + Level * 5
  Attack = Strength + Weapon Bonus
  Defense = Agility + Armor Bonus
```

### Equipment System
```
Equipment Slots (10+):
  - Weapon (Main hand)
  - Shield (Off hand)
  - Helmet
  - Armor (Chest)
  - Gloves
  - Boots
  - Ring (Left)
  - Ring (Right)
  - Amulet
  - Cloak

Equipment Quality:
  Common → Uncommon → Rare → Epic → Legendary
  
Found in dungeons, bought from shops, or stolen from players
```

### Combat System
```
Turn-based combat:
  ATTACK  - Physical strike
  DEFEND  - Reduce incoming damage
  SKILL   - Use learned ability
  ITEM    - Use consumable
  FLEE    - Attempt escape (may fail)

Combat modifiers:
  - Weapon type vs armor type
  - Status effects (poison, stun, etc.)
  - Steroid bonuses (if applicable)
  - Team bonuses (if in party)
```

### Drugs & Steroids
```
STEROIDS (Risk vs Reward):
  + Massive stat boosts
  + Temporary invincibility feelings
  - Reduces Mental Stability
  - Addiction mechanics
  - Psychosis if Mental Stability hits 0
  
Types:
  Basic Steroids    - +10 STR, -5 Mental
  Power Enhancers   - +20 STR, -10 Mental
  Rage Inducers     - +30 ATK, -15 Mental
  Experimental      - Random effects

PSYCHOSIS:
  When Mental Stability = 0:
  - Character goes berserk
  - May attack allies
  - May lose items
  - Requires rehabilitation
```

### Team System
```
CLANS/TEAMS:
  - Form teams with other players
  - Share dungeon progress
  - Coordinate attacks on rivals
  - Pool resources
  - Team base with storage

TEAM ACTIONS:
  - Joint dungeon raids
  - Coordinated PvP attacks
  - Resource sharing
  - Territory control
```

### Romance & Family
```
ROMANCE:
  - Flirt with NPCs or other players
  - Build relationship over time
  - Marriage proposals
  - Wedding ceremonies

FAMILY:
  - Have children (stat inheritance)
  - Children can become playable
  - Family dynasty tracking
  - Inheritance when character dies
```

### Political System
```
RANKS:
  Commoner → Citizen → Noble → Baron → Count → Duke → Prince → KING

BECOMING KING:
  - Accumulate wealth and power
  - Gain political support
  - Challenge current King
  - Win in combat or election

KING POWERS:
  - Set taxes
  - Declare wars
  - Grant titles
  - Access to royal treasury
```

### The Supreme Being
```
Located at the absolute bottom of Durunghins.
  - Requires massive preparation
  - Bank health potions (10,000+ recommended)
  - Choose your approach carefully
  - Victory grants legendary rewards
  - Can alter the destiny of the world
```

### Technical Requirements
- **Database**: Player characters, equipment, dungeon state, political system
- **Turn System**: Limited actions per day
- **PvP Tracking**: Combat logs, revenge mechanics

### Data Model
```
characters:
  - id, user_id, name
  - stats (str, agi, vit, int, cha)
  - mental_stability
  - level, experience, gold
  - equipment (JSON)
  - inventory (JSON)
  - team_id (nullable)
  - spouse_id (nullable)
  - political_rank

teams:
  - id, name, leader_id
  - treasury, territory
  - members (array)

dungeon_progress:
  - character_id
  - deepest_level_reached
  - current_level
  - discovered_rooms (JSON)

children:
  - id, parent1_id, parent2_id
  - inherited_stats
  - playable (bool)
```

---

# GAME: Dragon Slayer
**Genre:** Medieval RPG
**Type:** Multiplayer (Asynchronous)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Moderate

### Core Concept
Inspired by Legend of the Red Dragon (LORD), one of the most beloved BBS door games ever. A red dragon is terrorizing the town, and children are disappearing. Players fight monsters in the forest to gain experience, level up by defeating masters at the training grounds, and ultimately slay the dragon. Features romance, flirting, player battles, and the social dynamics that made LORD legendary.

### Key Features
- **12 Level Progression**: Fight through masters to reach level 12
- **Three Skill Paths**: Death Knight, Mystic, Thief (can learn all three)
- **Forest Combat**: Daily battles against scaling monsters
- **PvP Combat**: Attack other players for massive XP
- **Romance System**: Flirt, date, marry other players
- **Daily Turn Limit**: Strategic use of limited daily actions
- **The Red Dragon**: Ultimate goal - find and slay the dragon
- **IGM Support**: Extensible with additional modules

### Game Flow
```
DAILY LOOP:
  1. Wake up at the Inn
  2. Visit the Forest (fight monsters)
  3. Visit Town locations
  4. Challenge other players (limited fights)
  5. Train at Turgon's (if enough XP)
  6. Romance/social activities
  7. Return to Inn

Limited Actions Per Day:
  - Forest fights: ~15-20
  - Player attacks: 1-3
  - Training attempts: Unlimited (if qualified)
```

### Locations
```
THE TOWN:
  [I] The Inn           - Sleep, heal, save game
  [F] The Forest        - Fight monsters, random events
  [T] Turgon's Training - Level up by defeating masters
  [W] The Weapons Shop  - Buy/sell weapons
  [A] The Armor Shop    - Buy/sell armor
  [H] Healers           - Restore HP (costs gold)
  [B] The Bank          - Store gold safely
  [K] King's Court      - Daily news, messages
  [O] Other Places      - IGM locations
  [V] Violet's House    - Romance NPC
  [S] Slaughter Arena   - PvP battles
```

### Level & Master System
```
Level progression (must defeat master to advance):

Lv  Master              HP      STR    Required XP
1   Halder              15      10     100
2   Buga                30      15     400
3   Atsuko Sensei       50      20     1,000
4   Sandtiger           80      30     4,000
5   Sparhawk            150     50     10,000
6   Aladdin             250     75     40,000
7   Prince Caspian      400     100    100,000
8   Gandalf             600     150    400,000
9   Turgon (Master)     1000    200    1,000,000
10  Merlin              2000    350    4,000,000
11  Pellinore           4000    500    10,000,000
12  (No Master)         -       -      40,000,000

At Level 12: Can attempt to find and slay the Red Dragon
```

### Three Skill Paths
```
DEATH KNIGHT:
  - Attack skills
  - Power Strike (2x damage)
  - Death Wish (sacrifice HP for damage)
  - Ultimate: Assault (massive damage)

MYSTICAL SKILLS:
  - Magic abilities
  - Fireball (ranged damage)
  - Heal (restore HP)
  - Ultimate: Mystical Transport

THIEVING SKILLS:
  - Subterfuge
  - Pick Pocket (steal gold)
  - Sneak Attack (bonus damage)
  - Ultimate: Fairy Catch (various uses)

Players can learn skills from ALL THREE paths!
Skill points earned per level up (max 40 per skill)
```

### Forest System
```
MONSTERS (scale with level):
  Level 1: Small Slimes, Rats
  Level 3: Goblins, Wolves
  Level 5: Orcs, Dark Elves
  Level 7: Trolls, Necromancers
  Level 9: Dragons, Demons
  Level 11: Ancient Evils
  
RANDOM EVENTS:
  - Find gold/gems
  - Encounter special NPCs
  - Discover secrets (Jennie codes)
  - Ambushed by other players
  - Fairy encounters
```

### Combat
```
OPTIONS:
  [A]ttack     - Standard physical attack
  [S]kill      - Use learned skill
  [R]un        - Attempt to flee
  [H]eal       - Use healing spell/item

DAMAGE = (Weapon + Strength) - Enemy Defense
Critical hits: 10% chance for 2x damage

DEATH:
  - Lose 10% of experience
  - Lose some gold (unless banked)
  - Respawn at Inn next day
  - Fairy can revive (if held)
```

### Romance System
```
ROMANTIC TARGETS:
  - Violet (NPC barmaid)
  - Seth the Bard (NPC)
  - Other players

ROMANCE PROGRESSION:
  1. Flirt (build relationship)
  2. Date (special events)
  3. Propose (requires charm + gold)
  4. Marriage (status + bonuses)
  5. Children (stat bonuses)

CHARM stat affects romance success
Daily flirt limit prevents spam
```

### PvP System
```
PLAYER ATTACKS:
  - Limited per day (1-3)
  - Can only attack lower or same level
  - Massive XP rewards for victory
  - Can use skills in PvP
  - Sleeping players can be attacked
  
PROTECTION:
  - Stay at Inn for safety
  - Bank gold to prevent theft
  - Higher level = harder to kill
```

### The Red Dragon
```
FINDING THE DRAGON:
  - Must be Level 12
  - Search the forest (random encounter)
  - Can take multiple days to find

THE BATTLE:
  - Dragon has massive HP
  - Requires preparation (potions, equipment)
  - Skills are crucial
  - Fairy can save from death once

VICTORY:
  - Character is reset (keeps some stats)
  - Name on Hall of Fame
  - Special title
  - Bragging rights
```

### Technical Requirements
- **Database**: Characters, equipment, daily state, romance
- **Turn Tracking**: Limited daily actions
- **IGM System**: Extensible module support

### Data Model
```
characters:
  - id, user_id, name, sex
  - level, experience, gold_pocket, gold_bank
  - hp_current, hp_max
  - strength, defense, charm
  - weapon_id, armor_id
  - deaths, kills
  - skill_death_knight, skill_mystic, skill_thief
  - skill_uses_today (JSON)
  - spouse_id
  - forest_fights_today, player_fights_today
  - dragon_kills

daily_state:
  - character_id, date
  - actions_remaining
  - already_fought (player IDs)

monsters:
  - id, name, level_min, level_max
  - hp, strength, defense
  - gold_drop_min, gold_drop_max
  - xp_reward

masters:
  - id, name, level
  - hp, strength, defense
  - quote_on_defeat
```

---

# GAME: Realm of Kyrandia
**Genre:** Multi-player Text Adventure RPG
**Type:** Multiplayer (Persistent)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
Inspired by the classic Kyrandia BBS game by Galacticomm. Travel to a fairy tale land of magic and mystery, realm of the goddess Tashanna, the "Lady of Legends." Begin as a young apprentice from a small village seeking to grow in knowledge, wisdom, and power. Your goal: become the "Arch-Mage of Legends" — the mightiest wizard of Kyrandia. Cross the lands, face perils, solve mystical enigmas, battle rival mages, and conquer your own mind, body, and soul.

### Key Features
- **Multi-player Adventure**: Many players explore simultaneously
- **Spell Casting System**: Learn and cast spells to progress
- **Puzzle Solving**: Mystical enigmas block your path
- **Player Interaction**: Help, battle, trade, or chat
- **Four Regions**: Village → Dark Forest → Golden Forest → Dragon Castle
- **Leveling System**: Grow from Apprentice to Arch-Mage
- **Item Manipulation**: Carry, use, combine items
- **Social Gameplay**: The journey is shared

### World Structure
```
KYRANDIA REGIONS:

THE VILLAGE (Starting Area)
  - Humble beginnings
  - Basic shops and training
  - Learn fundamental spells
  - NPC tutors
  
THE DARK FOREST
  - Dangerous creatures
  - Hidden paths
  - Mystical plants and ingredients
  - Rival mages prowl
  
THE GOLDEN FOREST
  - Ancient magic
  - The Fountain of Scrolls
  - Harder puzzles
  - Powerful items
  
DRAGON CASTLE
  - Final region
  - The vicious dragon guardian
  - Ultimate challenges
  - Arch-Mage chamber
```

### Magic System
```
SPELL TYPES:
  Combat Spells:
    - Fireball (damage)
    - Lightning Bolt (high damage)
    - Ice Shard (slow enemy)
    
  Utility Spells:
    - Light (illuminate dark areas)
    - Teleport (limited range)
    - Detect Magic (reveal hidden)
    
  Defensive Spells:
    - Shield (reduce damage)
    - Heal (restore HP)
    - Ward (prevent entry)

LEARNING SPELLS:
  - Find spell scrolls
  - Study at libraries
  - Taught by NPCs
  - Generated at the Fountain

SPELL POWER:
  - Grows with level
  - Requires mana
  - Some require components
```

### The Fountain of Scrolls
```
SCROLL GENERATION:
  1. Find pine cones in forest
  2. Carry to the Fountain
  3. Throw pine cones in (3 per scroll)
  4. Random scroll appears
  
Scroll types vary - some required for progression
Time-consuming but essential mechanic
```

### Puzzle System
```
PROGRESSION PUZZLES:
  Each level transition requires solving a puzzle
  
Example (Level 2→3):
  - Stand in specific location
  - Enter: "chant glory be to tashanna"
  - Nothing hints at this directly
  
PUZZLE TYPES:
  - Word/phrase puzzles
  - Item combination puzzles
  - Location-specific actions
  - Timed sequences
  - Multi-player cooperation puzzles

HINTS:
  - NPCs give cryptic clues
  - Library books contain lore
  - Other players can share knowledge
```

### Character Progression
```
RANKS:
  Lv 1: Apprentice
  Lv 2: Initiate
  Lv 3: Acolyte
  Lv 4: Mage
  Lv 5: Sorcerer
  Lv 6: Wizard
  Lv 7: Arch-Mage of Legends (WINNER)

STATS:
  - Health (HP)
  - Mana (MP)
  - Wisdom (puzzle hints)
  - Power (spell strength)
  - Speed (combat order)
```

### Combat
```
ENCOUNTERS:
  - Forest creatures
  - Rival player mages
  - Guardian monsters
  - The Dragon (final boss)

COMBAT FLOW:
  1. Encounter begins
  2. Choose: Attack, Cast Spell, Use Item, Flee
  3. Speed determines turn order
  4. Combat resolves
  5. Victor gains XP/loot

DEATH:
  - Respawn at village
  - Lose some items
  - Keep level progress
```

### Items
```
ITEM TYPES:
  - Spell components
  - Healing potions
  - Scrolls
  - Weapons/Armor
  - Quest items
  - Pine cones (for Fountain)

ITEM MANIPULATION:
  Commands: TAKE, DROP, USE, EXAMINE, COMBINE
  Limited inventory space
  Some items are heavy
```

### Multiplayer Interaction
```
SOCIAL COMMANDS:
  SAY [message]    - Speak to nearby players
  WHISPER [player] - Private message
  EMOTE [action]   - Roleplay actions
  TRADE [player]   - Exchange items
  DUEL [player]    - Challenge to combat

COOPERATION:
  - Share puzzle solutions
  - Team up against monsters
  - Trade spell scrolls
  - Guide new players
```

### Becoming Arch-Mage
```
REQUIREMENTS:
  1. Reach Level 6 (Wizard)
  2. Collect all required items
  3. Solve the final puzzle sequence
  4. Enter the Arch-Mage chamber
  5. Complete the ritual
  
FIRST TO COMPLETE = ARCH-MAGE OF LEGENDS
  - Name immortalized
  - Special powers granted
  - Can reset for new game
```

### Technical Requirements
- **Database**: World state, player positions, items, spells
- **Text Parser**: Process player commands
- **Multi-player Sync**: Real-time interactions
- **Puzzle State**: Track solved/unsolved per player

### Data Model
```
characters:
  - id, user_id, name
  - level, experience
  - hp, hp_max, mana, mana_max
  - wisdom, power, speed
  - location_id
  - inventory (JSON)
  - spells_known (JSON)
  - puzzles_solved (JSON)

locations:
  - id, region, name
  - description
  - exits (JSON: direction → location_id)
  - items_present (JSON)
  - npcs_present (JSON)
  - puzzle_required (nullable)

spells:
  - id, name, type
  - mana_cost, damage
  - required_level
  - components (JSON)

puzzles:
  - id, location_id
  - solution_command
  - hint_text
  - unlocks (next area or item)
```

---

# GAME: Star Trader
**Genre:** Space Trading / Strategy
**Type:** Multiplayer (Asynchronous)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Complex

### Core Concept
Inspired by Trade Wars 2002, the legendary BBS door game. Command a starship in a galaxy of sectors, trading Fuel Ore, Organics, and Equipment between ports. Accumulate wealth to buy better ships, colonize planets, build starbases, and ultimately conquer the universe. Form corporations with other players, battle the Ferrengi, and rise to galactic dominance. Turn-based with daily action limits.

### Key Features
- **Galaxy Exploration**: Thousands of sectors to discover
- **Trading**: Buy low, sell high (Ore, Organics, Equipment)
- **Ship Upgrades**: Multiple ship classes with different capabilities
- **Combat**: Battle pirates, Ferrengi, and other players
- **Corporations**: Team up with other players
- **Planet Colonization**: Build your empire
- **Starbases**: Construct and defend
- **StarDock**: Central hub for trading and socializing
- **Daily Turns**: Strategic resource management

### Galaxy Structure
```
SECTOR TYPES:
  Empty Space     - Just warps to other sectors
  Port            - Trading post (buy/sell goods)
  Planet          - Colonizable worlds
  Starbase        - Player-built stations
  StarDock        - Central hub (special location)
  Ferrengi Space  - Dangerous enemy territory
  
GALAXY SIZE:
  Small:  1,000 sectors
  Medium: 5,000 sectors
  Large:  10,000+ sectors

WARPS:
  Each sector connects to 1-6 other sectors
  One-way warps exist (traps!)
  Mapping is crucial
```

### Trading System
```
COMMODITIES:
  Fuel Ore   - Mined from planets, powers ships
  Organics   - Food and biological materials
  Equipment  - Technology and machinery

PORT TYPES (Buy/Sell combinations):
  BBB - Buys all three (rare, profitable)
  BBS - Buys Ore/Org, Sells Equip
  BSB - Buys Ore/Equip, Sells Org
  SBB - Sells Ore, Buys Org/Equip
  SSB - Sells Ore/Org, Buys Equip
  SBS - Sells Ore/Equip, Buys Org
  BSS - Buys Ore, Sells Org/Equip
  SSS - Sells all three (dumping ground)

TRADING MECHANICS:
  - Haggle for better prices
  - Port supply/demand fluctuates
  - Overstocking crashes prices
  - Trade routes for steady profit
```

### Ships
```
Starting Ship:
  MERCHANT CRUISER
  - 20 cargo holds
  - 30 fighters
  - Basic weapons/shields
  - Warp drive

Upgradeable Ships (examples):
  SCOUT MARAUDER
  - Fast, small cargo
  - Reconnaissance
  
  CARGO FREIGHTER
  - 100+ cargo holds
  - Slow, vulnerable
  - Maximum profit potential
  
  COLONIAL FRIGATE
  - Colonist transport
  - Planet development
  
  IMPERIAL STARSHIP
  - Best combat stats
  - Federation commission required
  
SHIP STATS:
  - Cargo Holds (trading capacity)
  - Fighters (attack/defense)
  - Shield Points
  - Warp Speed
  - Scanner Range
```

### Combat System
```
COMBAT ENCOUNTERS:
  - Ferrengi patrols
  - Space pirates
  - Other players
  - Planetary defenses
  - Starbase defenses

COMBAT MECHANICS:
  Fighters attack fighters
  Remaining fighters attack ship
  Shield points absorb damage
  Destroyed ship = escape pod (lose everything)

WEAPONS:
  - Photon Torpedoes
  - Plasma Cannons
  - Mines (sector denial)
  - Fighter deployment
```

### Planets & Colonization
```
PLANET TYPES:
  Class M - Earth-like (best)
  Class L - Marginal
  Class K - Adaptable
  Class O - Oceanic
  Class H - Desert
  
COLONIZATION:
  1. Transport colonists
  2. Land on planet
  3. Build citadel (defense)
  4. Develop production
  
PLANET PRODUCTION:
  - Fuel Ore mining
  - Organics farming
  - Equipment manufacturing
  - Fighter construction
  
DEFENSE:
  - Planetary fighters
  - Atmospheric shields
  - Ground troops
```

### Corporations
```
FORMING A CORP:
  - Requires credits
  - Invite other players
  - Shared resources
  - Coordinated strategy

CORP BENEFITS:
  - Shared planet access
  - Combined fleet power
  - Corporate treasury
  - Territory control

CORP WARFARE:
  - Declare war on rival corps
  - Coordinated attacks
  - Economic warfare
  - Espionage
```

### StarDock
```
THE HUB:
  - Central trading post
  - Ship dealership
  - Equipment upgrades
  - Player interactions
  - News and announcements
  - Banking services

SPECIAL FEATURES:
  - Hardware Emporium (ship upgrades)
  - Ship Dealership (buy/sell ships)
  - Federation Commission (elite status)
  - Corporate Headquarters
  - Arena (PvP battles)
```

### The Ferrengi
```
ENEMY FACTION:
  - Control dangerous sectors
  - Aggressive patrols
  - Steal cargo
  - Bounties for destruction
  
FERRENGI SHIPS:
  - Ferrengi Scout
  - Ferrengi Marauder
  - Ferrengi Dreadnought
  
FERRENGAL:
  - Ferrengi homeworld
  - Extremely dangerous
  - Massive rewards if survived
```

### Turns & Time
```
DAILY TURNS:
  - Start with 500-1000 turns/day
  - Every action costs turns:
    - Move: 1-3 turns
    - Trade: 2 turns
    - Combat: varies
    - Building: 5+ turns
    
TURN STRATEGY:
  - Efficient trade routes
  - Minimize wasted movement
  - Plan attacks carefully
  - Bank excess turns (limited)
```

### Technical Requirements
- **Database**: Galaxy map, player ships, planets, starbases
- **Turn Tracking**: Daily limits, action costs
- **Economy Simulation**: Supply/demand at ports

### Data Model
```
galaxy:
  - sector_id (PK)
  - warps (JSON: array of connected sectors)
  - port_id (nullable)
  - planet_ids (JSON)
  - hazards (JSON)

ports:
  - id, sector_id, name
  - type (BBB, BBS, etc.)
  - ore_inventory, ore_price
  - org_inventory, org_price
  - equip_inventory, equip_price

players:
  - id, user_id, name
  - credits, turns_remaining
  - ship_type, cargo (JSON)
  - fighters, shields
  - sector_id
  - corporation_id
  - alignment (good/evil)
  - experience, rank

planets:
  - id, sector_id, owner_id
  - class, name
  - colonists
  - ore_production, org_production, equip_production
  - fighters, shields
  - citadel_level

corporations:
  - id, name, leader_id
  - treasury
  - member_ids (JSON)
  - territory (JSON: controlled sectors)

starbases:
  - id, sector_id, owner_id
  - name
  - fighters, shields
  - cargo_storage (JSON)
```

---

# GAME: Acromania
**Genre:** Party / Word Game
**Type:** Multiplayer (3-16 players)
**Platform:** BBS Door Game / ASCII Terminal
**Complexity:** Moderate

### Core Concept
A party word game inspired by Acrophobia (Berkeley Systems/Jellyvision). Players are given a random acronym and must invent clever, funny, or absurd phrases that fit the letters. Everyone votes on the best submission (can't vote for your own). Points awarded for votes received. Fast-paced rounds with escalating difficulty.

### Key Features
- **Acronym Generation**: Random 3-7 letter acronyms with optional category hints
- **Anonymous Submission**: Players don't know who wrote what until voting ends
- **Voting System**: Vote for favorite (not your own), points for votes received
- **Speed Bonus**: Faster submissions get bonus points
- **Category Rounds**: Optional themed rounds (Movies, Food, Tech, etc.)
- **Face-Off Rounds**: Top 2 players compete head-to-head
- **Lobby System**: Players can join between rounds

### Game Flow
```
LOBBY PHASE (30 sec between rounds)
  - Players join/leave
  - Show current standings
  - Countdown to next round

ACRONYM REVEAL (5 sec)
  ┌─────────────────────────────────────┐
  │                                     │
  │       Round 5 of 10                 │
  │                                     │
  │       Category: EXCUSES             │
  │                                     │
  │           W.T.F.L.                  │
  │                                     │
  │       60 seconds to submit!         │
  │                                     │
  └─────────────────────────────────────┘

SUBMISSION PHASE (60 sec)
  - Players type phrases matching acronym
  - Timer countdown visible
  - Early submission = speed bonus
  - Can edit until time expires or lock in

VOTING PHASE (30 sec)
  ┌─────────────────────────────────────┐
  │  Vote for your favorite W.T.F.L.   │
  │                                     │
  │  1. Weasels Typically Fear Llamas   │
  │  2. Why The Face, Larry?            │
  │  3. Went To Finland Last            │
  │  4. Washing Towels Feels Lame       │
  │  5. Witches Transform Frogs Lazily  │
  │                                     │
  │  Enter number (1-5): _              │
  │                                     │
  │  Time remaining: 22 seconds         │
  └─────────────────────────────────────┘
  
  - Submissions shown in random order
  - Cannot vote for own submission
  - Can change vote until time expires

RESULTS PHASE (10 sec)
  ┌─────────────────────────────────────┐
  │           RESULTS                   │
  │                                     │
  │  "Why The Face, Larry?"             │
  │  by PlayerOne - 4 votes (400 pts)   │
  │                                     │
  │  "Witches Transform Frogs Lazily"   │
  │  by xXSlayerXx - 2 votes (200 pts)  │
  │                                     │
  │  "Weasels Typically Fear Llamas"    │
  │  by CoolDude99 - 1 vote (100 pts)   │
  │  + Speed Bonus: 50 pts              │
  │                                     │
  │  No votes: Washing..., Went To...   │
  └─────────────────────────────────────┘
```

### Scoring System
```
Base Points:
  Each vote received: 100 points
  
Bonuses:
  Speed bonus: Up to 50 points (scales with submission time)
  Unanimous: 200 bonus if ALL players vote for you
  Participation: 10 points just for submitting

Face-Off Round (final round):
  Winner gets: 500 points
  Loser gets: 100 points
  
Penalties:
  No submission: 0 points, -25 for 3+ skips in a row
  Invalid submission (doesn't match letters): 0 points
```

### Acronym Generation
```
Length by round:
  Rounds 1-3:  3 letters
  Rounds 4-6:  4 letters
  Rounds 7-9:  5 letters
  Round 10:    6-7 letters (Face-Off)

Letter Selection:
  - Weighted by English frequency (avoid QXZJ heavy)
  - Ensure at least one vowel in 4+ letter acronyms
  - Avoid unpronounceable combinations
  
Categories (optional, announced with acronym):
  - Open (anything goes)
  - Movies / TV Shows
  - Food & Drink
  - Technology
  - Excuses
  - Pickup Lines
  - Warning Signs
  - Band Names
  - Book Titles
  - Conspiracy Theories
```

### Validation
```
Submission must:
  1. Have correct number of words
  2. Each word starts with correct letter (case insensitive)
  3. Not be blank/empty
  
Allowed:
  - Numbers written as words ("Four" for F)
  - Contractions count as one word ("Don't" for D)
  - Common abbreviations ("Dr." for D)
  
Not allowed:
  - Just the letter itself ("F is for F")
  - Offensive content (basic filter)
```

### Multiplayer Model
```
Game States:
  LOBBY     - Accepting players, between games
  STARTING  - Countdown to first round
  ACRONYM   - Showing the acronym
  SUBMIT    - Accepting submissions
  VOTE      - Voting phase
  RESULTS   - Showing round results
  FINAL     - Game over, final standings
  
Player can join: LOBBY, STARTING, or between rounds
Player leaves: Their submissions removed from current round
Minimum players: 3 (2 submissions + 1 voter minimum)
Maximum players: 16
```

### Technical Requirements
- **Database**: Game sessions, player scores, submission history
- **Real-time Sync**: Timer synchronization, vote counting
- **Content Filter**: Basic profanity/offensive content filter
- **Random Generation**: Weighted acronym generator

### Data Model
```
games:
  - id, created_at
  - status (lobby|active|complete)
  - current_round (1-10)
  - round_phase (acronym|submit|vote|results)
  - settings (JSON: rounds, time_limits, categories)

game_players:
  - game_id, user_id
  - score
  - joined_at
  - submissions_count
  - skips_in_row

rounds:
  - id, game_id, round_number
  - acronym
  - category (nullable)
  - phase_started_at

submissions:
  - id, round_id, user_id
  - text
  - submitted_at
  - is_valid
  - votes_received
  - points_earned

votes:
  - round_id, voter_user_id
  - submission_id
  - voted_at
```

### ASCII Display Themes
```
Standard:
  ╔═══════════════════════════════════════╗
  ║                                       ║
  ╚═══════════════════════════════════════╝

Retro BBS:
  +---------------------------------------+
  |                                       |
  +---------------------------------------+

Hacker:
  [=========================================]
  |                                         |
  [=========================================]
```

### Chat Integration
```
Between rounds, players can chat in lobby
During rounds, chat is disabled (prevents hints)
Post-game: Full chat enabled for discussion
```

---

# Implementation Notes for All Games

## Shared Infrastructure Needs

### Database
All games need persistent storage. Recommended: SQLite for simplicity, PostgreSQL for multiplayer scale.

### User System
Shared across all games:
```
users:
  - id, username, password_hash
  - created_at, last_login
  - total_play_time
```

### Session Management
Track active sessions for multiplayer games and statistics.

### ASCII Rendering
Consider shared library for:
- Box drawing
- Color codes (ANSI)
- Screen clearing
- Cursor positioning

## Per-Game Isolation
Despite shared infrastructure, each game should be:
- Separate crate/module
- Own database tables (prefixed)
- Independent game loop
- Standalone testable

## Recommended Implementation Order (by complexity)

### BBS Features
- Memory Garden (main menu option)

### Simple (1-2 days each)
1. Sudoku
2. Queens
3. Chess

### Moderate (1-2 weeks each)
4. Summit
5. Acromania
6. Dragon Slayer (LORD clone)
7. Depths of Diablo (start simple)
8. Dystopia (core mechanics)

### Complex (2-4 weeks each)
9. Master of Cygnus
10. The Usurper
11. Realm of Kyrandia
12. Star Trader (Trade Wars clone)
13. Last Dream
14. Fortress
15. Tanks: Blitzkrieg

### Epic (1-2 months each)
16. Mineteria
17. Ultimo (MMO)
18. Cradle
19. Xodia the Living MUD

---

# Quick Reference - Copy/Paste Commands

## For Claude Code - Game-by-Game

### Memory Garden (BBS Feature)
```
Create BBS main menu feature "Memory Garden". Users leave daily memories (280 char limit, 1/day). Shows random recent memories on entry. Browse by page or date. Starter memory "I was born" dated 1/25/2026. Auto-generate memories for milestones (10x users/sessions/time). ASCII garden aesthetic.
```

### Sudoku
```
Create daily puzzle game "Sudoku" for BBS. Classic 9x9 grid. Same puzzle for all users per day (date-seeded). Track streaks (consecutive days completed). Difficulty levels. Timer and leaderboards. ASCII grid with keyboard navigation.
```

### Queens
```
Create daily puzzle game "Queens" for BBS. Place N queens on NxN grid with colored regions - one queen per region, no attacks. Same puzzle for all users per day. Track streaks. Hint system. Timer and leaderboards.
```

### Chess
```
Create async multiplayer Chess for BBS. Full rules (castling, en passant, promotion). Create game with first move, wait for opponent. Join existing games. 3-day move timeout = forfeit. ASCII board display. ELO ratings.
```

### Tanks: Blitzkrieg
```
Create real-time multiplayer combat game "Tanks: Blitzkrieg" for BBS, based on Flash Attack V2.2. 2-6 players on procedural islands (65536 variants). Each player has base (8 pods) + 4 tanks viewed via split-screen (base window + 4 tank windows). Tanks have phasers, mines (14), pods, fuel (500). Base has lasers (24), neutrons, seekers (9), defensive shields (8). Condition system: GREEN→YELLOW→RED→GONE. Destroy enemy base pods to win (threshold varies by player count). Ghost mode for eliminated players in 3+ games. Radio communications with jamming. Coordinates 0-190 X, 0-70 Y. Reference: FA22.DOC and gameplay screenshot provided.
```

### Summit
```
Create cooperative climbing game "Summit" for BBS, inspired by PEAK. Up to 4 players (lost nature scouts) scale procedurally generated mountain on mysterious island. New layout every 24 hours. Stamina-based climbing - every move costs stamina, every setback limits your stamina making it harder to climb. 4 biomes with life-threatening obstacles. 30 questionable foods (benefits AND side effects). Climbing items: ropes, spikes, pitons, mysterious Anti-Rope. Campfires with marshmallow roasting mini-game. Help teammates up ledges, place ropes for those who come after. Character customization, dozens of badges, cosmetic unlocks. The slightest mistake can spell your doom!
```

### Master of Cygnus
```
Create 4X space strategy game "Master of Cygnus" for BBS, inspired by Master of Orion 1. Procedural galaxy with 1 planet per star (simplified like MOO1). Colony management. Tech research. Ship design. Fleet combat. Async multiplayer - turns resolve when all submit or 72hr timeout. 3 timeouts = forfeit.
```

### Depths of Diablo
```
Create multiplayer roguelite "Depths of Diablo" for BBS, inspired by Diablo 1-2. Procedural dungeons. Classes: Warrior, Rogue, Sorcerer. Randomized loot with affixes. Permadeath with meta-progression. 1-4 player co-op. Town hub persists between runs.
```

### Dystopia
```
Create kingdom management game "Dystopia" for BBS. Manage province in kingdom. Build structures, train military, research magic. Multi-week ages. Kingdom teams. Inter-kingdom warfare. Hourly resource ticks.
```

### Last Dream
```
Create JRPG "Last Dream" for BBS, styled after Final Fantasy 1-2. ASCII overworld, towns, dungeons. Party of 4, 6 classes. Turn-based combat (Fight/Magic/Item/Run). Transportation progression (walk->ship->airship). Equipment and magic systems. Story involves Four Crystals and the Void, with hidden simulation twist revealed at ending - subtle breadcrumbs throughout (NPCs glitching, anachronistic words, geometric anomalies).
```

### Mineteria
```
Create 2D sandbox game "Mineteria" for BBS, inspired by Minecraft/Terraria. Procedural world with biomes. Mining, crafting, building. Day/night cycle. Inventory management. Basic combat. Persistent worlds.
```

### Fortress
```
Create colony sim "Fortress" for BBS, simplified Dwarf Fortress. Manage dwarves with skills and needs. Dig, build, craft production chains. Resource management. Seasonal threats. Multiplayer world with trade between fortresses. Cleaner ASCII than DF.
```

### Ultimo
```
Create ASCII MMORPG "Ultimo" for BBS, inspired by Ultima 1-6. Persistent shared world. Classes: Warrior, Mage, Rogue, Cleric. Overworld, towns, dungeons. Quests, NPCs, economy. See other players in real-time. Turn-based combat.
```

### Cradle
```
Create infinite progression RPG "Cradle" for BBS, inspired by Cradle books. 15+ tiers from Unsouled (unawakened) to Void (beyond existence) and beyond. Sacred Arts paths combining aspects. Puzzle element - wrong builds plateau. Mentor system with hints. World expands with progression. Tournaments and trials.
```

### Xodia the Living MUD
```
Create persistent MUD "Xodia the Living MUD" for BBS where LLM acts as Dungeon Master. Game mechanics (stats, inventory, combat) provide the rulebook; LLM narrates outcomes. Natural language input processed via micro-LLM intent classifier → mapped to canonical actions → resolved by game engine → narrated by DM LLM. MCP tools for: world state (locations, exits), entity registry (unique names/IDs), event log (history), character state (stats, inventory). Persistent world graph - new locations become permanent canon. D20-based skill/combat resolution. Multiplayer in shared world. Story export as narrative prose.
```

### The Usurper
```
Create multiplayer hack-n-slash RPG "The Usurper" for BBS, inspired by classic Usurper. 100+ dungeons in the mountain of Durunghins. Solo lonewolf or form teams/clans. Fight monsters and other players. Political system (become King, rise to godhood). Romance system (marry, have children). Drugs/steroids boost stats but risk Mental Stability (psychosis at 0). 10+ equipment slots. Story quests. PvP raids while players sleep. Ultimate goal: reach the deepest level and defeat The Supreme Being.
```

### Dragon Slayer
```
Create medieval RPG "Dragon Slayer" for BBS, inspired by Legend of the Red Dragon (LORD). Red dragon terrorizes town, children disappearing. Fight monsters in forest for XP. 12 level progression - defeat masters at Turgon's Training to advance. Three skill paths: Death Knight, Mystic, Thief (can learn all three). PvP combat for massive XP. Romance system (flirt, marry other players). Daily turn limits. Ultimate goal: reach Level 12 and slay the Red Dragon. IGM module support.
```

### Realm of Kyrandia
```
Create multi-player text adventure RPG "Realm of Kyrandia" for BBS, inspired by Kyrandia (Major BBS). Fantasy world of goddess Tashanna. Begin as apprentice, goal is become Arch-Mage of Legends. Four regions: Village → Dark Forest → Golden Forest → Dragon Castle. Spell casting system. Puzzle solving (some obscure, requires specific commands at specific locations). Fountain of Scrolls (throw pine cones to generate random scrolls). Multiplayer interaction (help, battle, trade, chat). First to complete = Arch-Mage.
```

### Star Trader
```
Create space trading/strategy game "Star Trader" for BBS, inspired by Trade Wars 2002. Galaxy of thousands of sectors. Trade Fuel Ore, Organics, Equipment between ports (buy low, sell high). Multiple ship classes (Merchant Cruiser → Imperial Starship). Combat (Ferrengi, pirates, players). Corporations (team up with players). Planet colonization and starbase construction. StarDock central hub. Daily turn limits. Ultimate goal: accumulate wealth and conquer the universe.
```

### Acromania
```
Create party word game "Acromania" for BBS, inspired by Acrophobia. 3-16 players. Random acronym shown (3-7 letters), players submit phrases matching letters. Anonymous voting phase - can't vote for own. Points for votes received + speed bonus. 10 rounds, escalating length. Optional category themes (Movies, Excuses, Tech, etc.). Face-off final round between top 2. Lobby system between rounds. Real-time synchronized timers.
```
