//! Dystopia - Kingdom Management Strategy Game
//!
//! A BBS adaptation of the web-based kingdom management genre (inspired by Utopia).
//! Players manage provinces within kingdoms, building military, economy, and magic.
//! Ages (rounds) last weeks, with kingdom coordination and inter-kingdom warfare.

pub mod data;
pub mod economy;
pub mod military;
pub mod render;
pub mod screen;
pub mod state;
pub mod tick;

// Re-export types used externally
pub use state::ProvinceState;
pub use screen::{GameScreen, DystopiaFlow, DystopiaAction};
