//! Master of Cygnus - 4X Space Strategy Game
//!
//! A Master of Orion 1 inspired 4X game where players lead space-faring
//! civilizations to dominate the Cygnus constellation through exploration,
//! expansion, exploitation, and extermination.
//!
//! Features:
//! - Galaxy generation with procedural star systems
//! - Colony management and population growth
//! - Technology research across 6 fields
//! - Custom ship design and fleet combat
//! - Async multiplayer with 72-hour turn timeout
//! - AI takeover for timed-out/forfeited players

// Work-in-progress module - suppress warnings for unused code
#![allow(dead_code)]
#![allow(unused)]

pub mod data;
pub mod state;
pub mod galaxy;
pub mod tech;
pub mod ships;
pub mod combat;
pub mod ai;
pub mod screen;
pub mod render;

// Re-export main types used externally
pub use state::GameState;
pub use screen::MocFlow;
