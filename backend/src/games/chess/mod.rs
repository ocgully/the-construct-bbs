//! Chess - Async Multiplayer Chess Game
//!
//! A full chess implementation with:
//! - Complete chess rules (castling, en passant, promotion, check/checkmate)
//! - Async multiplayer with move notifications
//! - ELO rating system
//! - Open games list, ELO-based matching, and direct challenges
//! - 3-day move timeout = forfeit
//!
//! Note: This module is implemented but not yet integrated into the main BBS.
#![allow(dead_code)]

pub mod board;
pub mod moves;
pub mod render;
pub mod screen;
pub mod state;

#[cfg(test)]
mod tests;

// Re-export types used externally
pub use board::Board;
pub use moves::Move;
pub use state::{GameState, GameStatus, PlayerColor, MatchmakingMode};
pub use screen::{GameScreen, ChessAction, ChessFlow};
