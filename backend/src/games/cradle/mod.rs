//! Cradle - Infinite Progression RPG
//!
//! An incremental/idle progression game inspired by the Cradle book series.
//! Players advance through 15+ tiers from Unsouled to Void and beyond,
//! combining Sacred Arts aspects to create unique cultivation paths.
//!
//! Key features:
//! - Multiple progression layers (resources -> upgrades -> prestige -> meta-prestige)
//! - Idle gains while offline (catchup calculation)
//! - Build paths with meaningful choices
//! - Respec system with significant cost
//! - Prestige resets with permanent bonuses
//! - Unlockable mechanics over time
//! - Mentor system with guidance

// Game module under development - code is complete but not yet integrated
#![allow(dead_code)]

pub mod data;
pub mod economy;
pub mod events;
pub mod render;
pub mod screen;
pub mod state;
pub mod tick;

// Re-export types used by service layer
pub use state::GameState;
pub use screen::CradleFlow;
pub use render::format_power;
