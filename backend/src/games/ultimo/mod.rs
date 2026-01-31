//! Ultimo - MMO-style persistent world RPG
//!
//! An Ultima Online-inspired multiplayer RPG where players share a persistent
//! world, develop skills through use, engage in PvE and PvP combat, craft items,
//! own housing, and trade with other players.
//!
//! Key features:
//! - Skill-based progression (no classes)
//! - Persistent shared world
//! - Real-time multiplayer (players see each other)
//! - Player housing and property ownership
//! - Crafting professions
//! - Player economy with trading
//! - PvP zones

// Game module under development - code is complete but not yet integrated
#![allow(dead_code)]

pub mod combat;
pub mod crafting;
pub mod data;
pub mod economy;
pub mod housing;
pub mod render;
pub mod screen;
pub mod skills;
pub mod state;
pub mod world;

// Re-export main types
pub use screen::UltimoFlow;
pub use state::GameState;
