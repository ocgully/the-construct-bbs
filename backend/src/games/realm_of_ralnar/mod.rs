//! Realm of Ralnar - Classic JRPG (Final Fantasy 1 style)
//!
//! A fantasy adventure where the player leads a party through the Realm of Ralnar,
//! destroying elemental shrines, recruiting companions, and uncovering the mystery
//! behind the dark forces threatening the world.
//!
//! ## Story Systems
//!
//! The game features a complex story involving Dorl manipulating the brothers
//! into killing the Five Guardians:
//!
//! - `story` - Story flag management and world phase tracking
//! - `events` - Event system for triggering story progression
//! - `cutscene` - Cutscene system for story sequences
//! - `predefined_events` - All major story events and cutscenes
//! - `dorl` - Special handling for Dorl's manipulations
//!
//! Structure:
//! - mod.rs (this file - public exports)
//! - screen.rs (GameScreen enum - game states)
//! - state.rs (GameState - player's persistent state, Party, PartyMember)
//! - flow.rs (RalnarFlow state machine + RalnarAction)
//! - status.rs (StatusEffect system for combat)
//! - magic.rs (Spell definitions and magic system)
//! - combat.rs (FF1-style turn-based combat system)
//! - battle_ai.rs (Enemy AI behaviors, including Guardian AI)
//! - damage.rs (Damage calculations for combat)
//! - map.rs (Map data structures, tiles, events)
//! - movement.rs (Player movement system)
//! - camera.rs (Viewport/camera for map rendering)
//! - tile_ascii.rs (ASCII rendering of tiles for BBS)
//! - map_loader.rs (Loading maps from JSON files)
//! - story.rs (Story state and world phase management)
//! - events.rs (Event conditions, effects, and system)
//! - cutscene.rs (Cutscene definitions and player)
//! - predefined_events.rs (Key story events and cutscenes)
//! - dorl.rs (Dorl's manipulation system)
//! - data/ (game data: config, items, enemies, spells, etc.)

#![allow(dead_code)]

pub mod battle_ai;
pub mod camera;
pub mod combat;
pub mod cutscene;
pub mod damage;
pub mod data;
pub mod dialogue;
pub mod dorl;
pub mod events;
pub mod flow;
pub mod magic;
pub mod map;
pub mod map_loader;
pub mod movement;
pub mod npc;
pub mod predefined_dialogue;
pub mod predefined_events;
pub mod quest;
pub mod render;
pub mod screen;
pub mod shop;
pub mod state;
pub mod status;
pub mod story;
pub mod tile_ascii;

// Re-export types used externally
#[allow(unused_imports)]
pub use camera::Camera;
#[allow(unused_imports)]
pub use combat::{
    BattleAction, BattleActor, BattleCharacter, BattleEnemy, BattleEvent, BattlePhase,
    BattleRewards, BattleRow, BattleState, EnemySpawn,
};
#[allow(unused_imports)]
pub use flow::{RalnarAction, RalnarFlow};
#[allow(unused_imports)]
pub use map::{Map, MapType, MovementMode, Tile};
#[allow(unused_imports)]
pub use map_loader::MapLoader;
#[allow(unused_imports)]
pub use movement::{MovementResult, MovementSystem, PlayerPosition};
#[allow(unused_imports)]
pub use screen::GameScreen;
#[allow(unused_imports)]
pub use state::{
    CharacterClass, CharacterStats, Direction, Equipment, GameState, Inventory, Party,
    PartyMember,
};
#[allow(unused_imports)]
pub use tile_ascii::{render_map_ascii, render_map_ascii_color, tile_to_ascii};

// Dialogue and NPC system exports
#[allow(unused_imports)]
pub use dialogue::{
    Condition, DialogueBuilder, DialogueChoice, DialogueEffect, DialogueNode, DialogueState,
    DialogueTree,
};
#[allow(unused_imports)]
pub use npc::{DirectionExt, MovementPattern, NPC, NPCRegistry, Schedule, ScheduleEntry};
#[allow(unused_imports)]
pub use predefined_dialogue::{get_dialogue, get_npc_dialogue};
#[allow(unused_imports)]
pub use quest::{Quest, QuestLog, QuestObjective, QuestRequirement, QuestReward, QuestState};
#[allow(unused_imports)]
pub use shop::{Shop, ShopAction, ShopItem, ShopMode, ShopState, ShopType};

// Story system exports
#[allow(unused_imports)]
pub use cutscene::{
    Cutscene, CutsceneAction, CutsceneDialogue, CutscenePlayer, CutsceneResult, CutsceneScene,
};
#[allow(unused_imports)]
pub use dorl::DorlSystem;
#[allow(unused_imports)]
pub use events::{
    EventCondition, EventEffect, EventResult, EventSystem, GameEvent,
    GameStateForEvents, Party as EventParty,
};
#[allow(unused_imports)]
pub use story::{flags, StoryState, WorldPhase};
