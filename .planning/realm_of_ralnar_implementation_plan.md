# Realm of Ralnar Implementation Plan

## Overview
Port of 1996 QuickBasic JRPG by Brian and Christopher Gulliver to Rust for BBS integration.

## Parallel Implementation Streams

### Stream 1: Asset Conversion Tools
**Agent Task:** Create tools crate with converters for all original file formats
- VGA palette mapping (6-bit to 8-bit)
- pic2png: Convert .PIC files to PNG (20x20 tiles, -1 = transparent)
- mmi2png: Convert .MMI files to PNG with JSON metadata
- mmm2json: Convert .MMM text maps to JSON
- nmf2json: Convert .NMF binary maps to JSON
- mon2png: Convert .MON monster sprites to PNG
- batch_convert: Process all assets at once

### Stream 2: Core Game Foundation
**Agent Task:** Create base game structure following existing game patterns
- `backend/src/games/realm_of_ralnar/mod.rs` - Module exports
- `backend/src/games/realm_of_ralnar/screen.rs` - GameScreen enum
- `backend/src/games/realm_of_ralnar/state.rs` - GameState with party, inventory, world
- `backend/src/games/realm_of_ralnar/flow.rs` - RalnarFlow state machine
- `backend/src/services/realm_of_ralnar/mod.rs` - Service module
- `backend/src/services/realm_of_ralnar/db.rs` - SQLite database layer
- `backend/src/services/realm_of_ralnar/service.rs` - Service entry points
- `backend/src/services/realm_of_ralnar/session.rs` - WebSocket session handler

### Stream 3: Map & Movement System
**Agent Task:** Implement map loading, camera, and movement
- `map.rs` - Map data structures, loading from JSON
- `camera.rs` - Camera system with world wrap support
- `movement.rs` - Grid-based player movement
- `tile.rs` - Tile definitions and attributes
- Support for overworld wrap-around
- Map transitions between areas

### Stream 4: Party & Character System
**Agent Task:** Implement party management and character progression
- `party.rs` - Party management (max 4, brothers always together)
- `character.rs` - CharacterStats, equipment, leveling
- `equipment.rs` - Weapons, armor, accessories
- `skills.rs` - Learnable abilities by class
- `classes.rs` - Character classes (Warrior, Paladin, Cleric, etc.)

### Stream 5: Combat System
**Agent Task:** Implement FF1-style turn-based combat
- `combat.rs` - Battle state machine
- `battle_ai.rs` - Enemy AI with Guardian special patterns
- `magic.rs` - Spells, MP costs, elemental system
- `status.rs` - Status effects (poison, stone, etc.)
- `damage.rs` - Damage/hit calculations
- Random encounter system

### Stream 6: Dialogue & Interaction
**Agent Task:** Implement NPC and dialogue systems
- `dialogue.rs` - Dialogue tree parser and renderer
- `npc.rs` - NPC definitions, movement patterns
- `shop.rs` - Buy/sell interface
- `inn.rs` - Rest and save functionality
- `chest.rs` - Treasure chests and loot

### Stream 7: Story & Events
**Agent Task:** Implement story progression systems
- `story.rs` - Story flags, world phases
- `quest.rs` - Quest tracking
- `cutscene.rs` - Cutscene player
- `events.rs` - Map event triggers
- `dorl.rs` - Dorl's special dialogue and blessing system

### Stream 8: Render System
**Agent Task:** Create ANSI rendering for BBS display
- `render.rs` - Main render dispatcher
- VGA palette to ANSI color mapping
- Battle screen rendering
- Map/exploration rendering
- Menu rendering
- Dialogue box rendering

### Stream 9: Data Definitions
**Agent Task:** Define all game data
- `data/items.rs` - All items (weapons, armor, consumables)
- `data/enemies.rs` - All enemy types with stats
- `data/spells.rs` - All spells and magic
- `data/guardians.rs` - Guardian boss definitions
- `data/npcs.rs` - NPC dialogue and behavior
- `data/config.rs` - Game constants

## File Structure
```
backend/src/games/realm_of_ralnar/
├── mod.rs           # Module exports
├── screen.rs        # GameScreen enum
├── state.rs         # GameState
├── flow.rs          # RalnarFlow state machine
├── render.rs        # ANSI rendering
├── map.rs           # Map loading/camera
├── movement.rs      # Player movement
├── combat.rs        # Battle system
├── battle_ai.rs     # Enemy AI
├── magic.rs         # Spell system
├── party.rs         # Party management
├── character.rs     # Character stats
├── equipment.rs     # Equipment system
├── dialogue.rs      # Dialogue trees
├── npc.rs           # NPC system
├── shop.rs          # Shop system
├── story.rs         # Story progression
├── quest.rs         # Quest tracking
├── cutscene.rs      # Cutscene system
├── events.rs        # Event triggers
└── data/
    ├── mod.rs
    ├── items.rs
    ├── enemies.rs
    ├── spells.rs
    ├── guardians.rs
    └── config.rs

backend/src/services/realm_of_ralnar/
├── mod.rs
├── db.rs            # SQLite persistence
├── service.rs       # Entry points
└── session.rs       # WebSocket handler

tools/ralnar_converter/
├── Cargo.toml
└── src/
    ├── main.rs      # Batch converter
    ├── palette.rs   # VGA palette
    ├── pic.rs       # PIC converter
    ├── mmi.rs       # MMI converter
    ├── mmm.rs       # MMM converter
    ├── nmf.rs       # NMF converter
    └── mon.rs       # MON converter
```

## Key Design Decisions

1. **BBS-First Design:** All rendering through ANSI, no SDL2/graphics
2. **Session-Based:** Each player has isolated game state
3. **Persistence:** SQLite for saves, like other BBS games
4. **Asset Loading:** Convert original assets to JSON/PNG at build time
5. **Combat:** FF1-style menu-based turn combat
6. **Story Integration:** Full support for the Dorl twist narrative

## Testing Requirements

1. **Unit Tests:** For each module (state, combat, movement, etc.)
2. **E2E Tests:** Playwright tests like other BBS games
3. **Integration Tests:** Service layer tests with temp database

## Dependencies (from existing games)
- serde/serde_json for serialization
- sqlx for database
- rand for combat RNG
- tempfile for tests
