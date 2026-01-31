//! Fortress - Colony Simulation Game
//!
//! A multiplayer colony simulation inspired by Dwarf Fortress with simplified
//! mechanics and cleaner ASCII visuals. Players manage dwarves with skills and
//! needs, build production chains, and defend against invasions.
//!
//! Key Features:
//! - Dwarf management with skills and needs (hunger, thirst, rest, mood)
//! - Job assignment and work orders
//! - Resource gathering (mining, woodcutting, farming)
//! - Crafting workshops and production chains
//! - Room designation (bedrooms, dining, workshops)
//! - Z-level terrain for underground digging
//! - Defense against periodic invasions

// Game module - code is intentionally reserved for future implementation
#[allow(dead_code)]
pub mod data;
#[allow(dead_code)]
pub mod state;
#[allow(dead_code)]
pub mod screen;
#[allow(dead_code)]
pub mod render;
#[allow(dead_code)]
pub mod terrain;
#[allow(dead_code)]
pub mod dwarves;
#[allow(dead_code)]
pub mod jobs;
#[allow(dead_code)]
pub mod tick;

// Re-export main types
pub use state::GameState;
pub use screen::{GameScreen, FortressFlow};
