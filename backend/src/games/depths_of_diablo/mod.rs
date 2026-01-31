//! Depths of Diablo - Real-Time Roguelite Dungeon Crawler
//!
//! A multiplayer roguelite inspired by Diablo 1-2, featuring:
//! - Real-time combat (not turn-based)
//! - Procedurally generated dungeons from daily seed
//! - Character classes: Warrior, Rogue, Sorcerer
//! - Randomized loot with affixes
//! - Permadeath with meta-progression
//! - 1-4 player co-op with lobby/matchmaking
//!
//! Uses __depths_of_diablo__ sentinel for session routing.

#![allow(dead_code)]

pub mod combat;
pub mod data;
pub mod dungeon;
pub mod items;
pub mod lobby;
pub mod render;
pub mod screen;
pub mod state;

// Re-export types used externally
pub use screen::DiabloFlow;
