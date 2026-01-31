//! Sudoku - Daily Puzzle Game
//!
//! A classic 9x9 Sudoku puzzle, refreshing daily at midnight Eastern.
//! All players get the same puzzle each day (date-seeded generation).
//! Tracks streaks, completion times, and pause days.
//!
//! Note: This module is implemented but not yet integrated into the main BBS.
#![allow(dead_code)]

pub mod generator;
pub mod render;
pub mod screen;
pub mod state;

// Re-export types used externally
pub use screen::{GameScreen, SudokuAction, SudokuFlow};
pub use state::GameState;
