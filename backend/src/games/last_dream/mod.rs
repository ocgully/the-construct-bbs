//! Last Dream - Classic JRPG in the style of Final Fantasy 1-2
//!
//! A medieval fantasy RPG with crystals, kingdoms, and ancient evils.
//! Features an overworld map, towns, dungeons, and turn-based party combat.
//! The story concludes with the revelation that the world is a simulation -
//! but this twist is hidden, with only very rare, subtle breadcrumbs
//! hinting at the truth throughout the journey.

// Game module - code is intentionally reserved for future implementation
#[allow(dead_code)]
pub mod combat;
#[allow(dead_code)]
pub mod data;
#[allow(dead_code)]
pub mod party;
#[allow(dead_code)]
pub mod render;
#[allow(dead_code)]
pub mod screen;
#[allow(dead_code)]
pub mod state;
#[allow(dead_code)]
pub mod world;

// Re-export types used externally
pub use screen::LastDreamFlow;
pub use state::GameState;
