//! Xodia service module
//!
//! Session routing and persistence for the Xodia LLM-powered MUD.

pub mod db;
pub mod service;

pub use db::XodiaDb;
pub use service::{SENTINEL, start_game, save_game_state, render_screen, handle_action, get_llm_config};
