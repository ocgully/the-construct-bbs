//! Mineteria - 2D Sandbox Mining/Crafting Game
//!
//! A Terraria/Minecraft inspired game with:
//! - Procedural world generation with biomes
//! - Mining and block placement
//! - Crafting system with recipes
//! - Tool progression (wood -> stone -> iron -> etc.)
//! - Day/night cycle with monster spawning
//! - Inventory management
//! - Combat system

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
pub mod world;
#[allow(dead_code)]
pub mod crafting;
#[allow(dead_code)]
pub mod combat;

// Re-export types used externally
pub use state::GameState;
pub use screen::MineteriaFlow;
