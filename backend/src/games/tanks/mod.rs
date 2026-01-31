//! Tanks - Real-time artillery game like Scorched Earth
//!
//! A real-time multiplayer artillery game where players control tanks on destructible terrain.
//! Features projectile physics with wind, terrain destruction, and multi-tank switching.
//!
//! Structure:
//! - mod.rs (this file) - Public exports
//! - data.rs - Static game data (weapons, items, terrain types)
//! - state.rs - TankState and GameState structs
//! - physics.rs - Projectile simulation, wind, collision detection
//! - terrain.rs - Terrain generation and destruction
//! - lobby.rs - Lobby and matchmaking system
//! - render.rs - ANSI rendering functions
//! - screen.rs - GameScreen enum + Flow state machine

#![allow(dead_code)]

pub mod data;
pub mod lobby;
pub mod physics;
pub mod render;
pub mod screen;
pub mod state;
pub mod terrain;

// Re-export commonly used types
pub use lobby::TanksLobby;
pub use render::{render_tanks_menu, render_leaderboard};
pub use screen::{TanksScreen, TanksAction, TanksFlow};
