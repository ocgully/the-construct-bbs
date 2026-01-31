//! Xodia - The Living MUD
//!
//! An LLM-powered MUD where the AI functions as a Dungeon Master.
//! The game mechanics provide the rules and foundation for what can be done,
//! while the LLM provides narrative descriptions and dynamic responses.
//!
//! Key Features:
//! - Natural language command parsing
//! - LLM-generated narrative responses
//! - Persistent world state (rooms, NPCs, items)
//! - Character stats, inventory, and progression
//! - Combat resolution (mechanical + LLM narration)
//! - Support for Ollama (local) and cloud APIs (OpenAI/Anthropic)

// Game module under development - code is complete but not yet integrated
#![allow(dead_code)]

pub mod data;
pub mod llm;
pub mod parser;
pub mod render;
pub mod screen;
pub mod state;
pub mod world;
pub mod combat;

// Re-export commonly used types
pub use state::GameState;
pub use screen::{GameScreen, XodiaAction, XodiaFlow};
