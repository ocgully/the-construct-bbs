//! Dragon Slayer - Legend of the Red Dragon style RPG
//!
//! A medieval RPG where players fight monsters in the forest,
//! level up by defeating masters, romance other players/NPCs,
//! and ultimately slay the Red Dragon terrorizing the town.

pub mod combat;
pub mod data;
pub mod events;
pub mod igm;
pub mod render;
pub mod romance;
pub mod screen;
pub mod state;

// Re-export types used externally
pub use state::GameState;
pub use screen::{GameScreen, DragonSlayerAction, DragonSlayerFlow};
