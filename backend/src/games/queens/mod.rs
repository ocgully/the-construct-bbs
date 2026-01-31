//! Queens - Daily N-Queens Puzzle
//!
//! A daily puzzle game where players place N queens on an NxN grid
//! with colored regions. Each region must contain exactly one queen,
//! and no two queens can attack each other (same row, column, or diagonal).
//!
//! Features:
//! - Same puzzle worldwide each day (date-seeded)
//! - One attempt per day
//! - Streak tracking with 3 automatic pause days per week
//! - Hint system
//! - Timer and leaderboards
//!
//! Note: This module is implemented but not yet integrated into the main BBS.
#![allow(dead_code)]

pub mod data;
pub mod puzzle;
pub mod render;
pub mod screen;
pub mod state;

// Re-export types used externally
pub use screen::{GameScreen, QueensFlow};
pub use state::{GameState, PlayerStats};
