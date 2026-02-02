//! Realm of Ralnar VGA - Graphics-based JRPG
//!
//! A faithful port of the 1996 QBasic RPG to web-based VGA Mode 13h (320x200, 256 colors).
//! Unlike the ASCII version, this renders actual pixel art using the original game assets.
//!
//! # Architecture
//!
//! This module renders to a virtual 320x200 framebuffer which is:
//! - Scaled up using nearest-neighbor interpolation
//! - Streamed to the client as PNG or raw pixel data
//! - Displayed on an HTML5 Canvas element
//!
//! # Assets
//!
//! Pre-converted PNG assets at multiple scales (1x-5x) in `assets/`:
//! - `tiles/` - 20x20 map tiles from MMI files
//! - `sprites/` - 20x20 character/object sprites from PIC files
//! - `monsters/` - Variable-size monster sprites from MON files
//! - `maps/` - JSON map data from MMM/NMF files

pub mod data;
pub mod palette;
pub mod render;
pub mod screen;
pub mod state;

pub use render::RalnarVgaRenderer;
pub use screen::GameScreen;
pub use state::GameState;

/// Virtual display dimensions (VGA Mode 13h)
pub const DISPLAY_WIDTH: u32 = 320;
pub const DISPLAY_HEIGHT: u32 = 200;

/// Tile dimensions
pub const TILE_SIZE: u32 = 20;

/// Visible tiles on screen
pub const VIEW_TILES_X: u32 = DISPLAY_WIDTH / TILE_SIZE;  // 16
pub const VIEW_TILES_Y: u32 = DISPLAY_HEIGHT / TILE_SIZE; // 10
