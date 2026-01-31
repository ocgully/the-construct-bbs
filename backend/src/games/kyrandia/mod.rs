//! Realm of Kyrandia - Text Adventure RPG
//!
//! A multi-player text adventure inspired by the classic Kyrandia BBS game.
//! Players explore a fairy tale realm, learn magic, solve puzzles, and
//! battle to become the Arch-Mage of Legends.
//!
//! Features:
//! - Hybrid input: Menu navigation + typed spell incantations
//! - Four regions: Village, Dark Forest, Golden Forest, Dragon Castle
//! - Magic system with spell scrolls and the Fountain of Scrolls
//! - Puzzle solving with cryptic clues
//! - Multiplayer interaction (chat, trade, duel)
//! - IGM (In-Game Module) support for extensibility

pub mod combat;
pub mod data;
pub mod igm;
pub mod magic;
pub mod parser;
pub mod puzzles;
pub mod render;
pub mod romance;
pub mod screen;
pub mod state;
pub mod world;

// Re-export types used externally
pub use state::GameState;
pub use screen::KyrandiaFlow;
