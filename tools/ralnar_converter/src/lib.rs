//! Realm of Ralnar Asset Conversion Library
//!
//! This crate provides converters for the various asset formats used in
//! the 1996 QuickBasic JRPG "Realm of Ralnar".
//!
//! # Supported Formats
//!
//! - **PIC**: 20x20 pixel tiles stored as text files with VGA palette indices
//! - **MMI**: Tiles with attribute metadata (passability, terrain type)
//! - **MMM**: Text-based map files with tile and attribute data
//! - **NMF**: Binary map files (new map format)
//! - **MON**: Monster sprites with animation frames
//!
//! # Multi-Scale Output
//!
//! All image converters support outputting at multiple scales (1x-5x) using
//! nearest-neighbor interpolation to preserve pixel art crispness.

pub mod mmi;
pub mod mmm;
pub mod mon;
pub mod nmf;
pub mod palette;
pub mod pic;
pub mod scaling;
pub mod tileset;

pub use mmi::{MmiError, MmiMetadata, MmiTile, TileAttributes};
pub use mmm::{MapTile, MmmError, MmmMap, MmmMapCompact};
pub use mon::{MonError, MonFrame, MonMetadata, MonSprite};
pub use nmf::{NmfError, NmfMap};
pub use palette::{palette_to_rgb, palette_to_rgba, vga6_to_rgb8, VGA_PALETTE};
pub use pic::{PicError, PicTile, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH};
pub use scaling::{save_scaled_images, scale_image, SCALE_FACTORS};
pub use tileset::load_tile_registry;
