//! Summit - Cooperative Mountain Climbing Game
//!
//! A real-time cooperative climbing game where 1-4 scouts must scale
//! a procedurally generated mountain. Features weather systems, stamina
//! management, gear crafting, and daily seeded mountains.
//!
//! Key mechanics:
//! - Stamina: Current (regenerates) and Max (permanent damage from falls)
//! - 4 Biomes: Beach, Jungle, Alpine, Volcanic
//! - 30 Questionable Foods with benefits AND side effects
//! - Climbing items: Ropes, pitons, grappling hooks, etc.
//! - Campfires between biomes for rest and revival

#![allow(dead_code)]

pub mod data;
pub mod mountain;
pub mod state;
pub mod screen;
pub mod render;
pub mod lobby;

// Re-export types used externally
pub use state::PlayerStats;
pub use screen::SummitFlow;
