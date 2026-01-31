//! Acromania - Multiplayer Acronym Party Game
//!
//! A party word game inspired by Acrophobia (Berkeley Systems/Jellyvision).
//! Players are given a random acronym and must invent clever, funny, or absurd
//! phrases that fit the letters. Everyone votes on the best submission.
//!
//! Features:
//! - 3-16 players per game
//! - 10 rounds with escalating difficulty (3-7 letter acronyms)
//! - Anonymous submissions during voting
//! - Speed bonuses for fast submissions
//! - Optional category themes
//! - Real-time synchronized timers

pub mod data;
pub mod game;
pub mod lobby;
pub mod profanity;
pub mod render;
pub mod scoring;
pub mod state;

// Re-export types used externally
pub use game::{AcroGame, GamePhase};
pub use lobby::AcroLobby;
pub use render::*;
