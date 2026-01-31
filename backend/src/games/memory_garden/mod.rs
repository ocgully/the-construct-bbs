//! Memory Garden - Communal digital garden for the BBS
//!
//! A social/journaling feature where users leave daily memories
//! (280 character limit). The garden grows with user contributions
//! and system-generated milestone memories.
//!
//! This is NOT a game but a BBS main menu feature, structured
//! similarly for consistency with the codebase patterns.
//!
//! Note: This module is implemented but not yet integrated into the main BBS.
#![allow(dead_code)]

pub mod render;
pub mod screen;
pub mod state;

pub use screen::{GardenAction, GardenFlow};
pub use state::{Memory, MilestoneType, FlagResolution};
