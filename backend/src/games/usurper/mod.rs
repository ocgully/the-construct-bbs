//! Usurper - Dark Fantasy RPG with Drugs/Steroids Mechanics
//!
//! A multiplayer hack-n-slash RPG inspired by the classic Usurper BBS door game.
//! Players explore 100+ dungeons in the mountain of Durunghins, fight monsters,
//! battle other players, and can use steroids/drugs that boost stats but risk
//! mental stability.
//!
//! Key Features:
//! - 100+ dungeon levels from mountain top to unfathomable depths
//! - Solo or team play (clans/teams)
//! - PvP combat with XP loss for killing much lower level players
//! - Political system (become King, rise to godhood)
//! - Romance system (marry, stat bonuses, divorce, same-sex OK)
//! - Drugs/steroids boost stats but risk mental stability (psychosis at 0)
//! - 10+ equipment slots
//! - IGM (In-Game Module) support for extensibility

// Work-in-progress module - suppress warnings for unused code
#![allow(dead_code)]
#![allow(unused)]

pub mod data;
pub mod state;
pub mod screen;
pub mod render;
pub mod combat;
pub mod substances;
pub mod romance;
pub mod igm;

// Re-export types used externally
pub use state::GameState;
pub use screen::UsurperFlow;
