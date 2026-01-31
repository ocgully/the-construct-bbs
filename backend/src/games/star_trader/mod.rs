//! Star Trader - Space Trading/Empire Game
//!
//! Inspired by Trade Wars 2002, the legendary BBS door game.
//! Command a starship in a galaxy of sectors, trading Fuel Ore, Organics,
//! and Equipment between ports. Build your empire through trading, combat,
//! and colonization.
//!
//! Structure:
//! - mod.rs (this file - public exports)
//! - data.rs (static game data - ships, sectors, commodities)
//! - state.rs (GameState - player's persistent state)
//! - screen.rs (GameScreen enum + StarTraderFlow state machine)
//! - render.rs (ANSI rendering functions)
//! - galaxy.rs (galaxy generation and navigation)
//! - combat.rs (ship combat system)
//! - economy.rs (trading and port mechanics)
//! - corporation.rs (corp management)

// Work-in-progress module - suppress warnings for unused code
#![allow(dead_code)]
#![allow(unused)]

pub mod data;
pub mod state;
pub mod screen;
pub mod render;
pub mod galaxy;
pub mod combat;
pub mod economy;
pub mod corporation;

pub use state::GameState;
pub use screen::StarTraderFlow;
