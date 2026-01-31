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
//! # Usage
//!
//! ```no_run
//! use ralnar_converter::{pic::PicTile, mmi::MmiTile, mmm::MmmMap, nmf::NmfMap, mon::MonSprite};
//!
//! // Convert a PIC file to PNG
//! let tile = PicTile::from_file("tile.pic").unwrap();
//! tile.save_png("tile.png").unwrap();
//!
//! // Convert an MMI file to PNG + JSON
//! let mmi = MmiTile::from_file("tile.mmi").unwrap();
//! mmi.save_png("tile.png").unwrap();
//! mmi.save_metadata("tile.json").unwrap();
//!
//! // Convert a map file
//! let map = MmmMap::from_file("world.mmm").unwrap();
//! map.save_json("world.json").unwrap();
//!
//! // Convert a monster sprite
//! let sprite = MonSprite::from_file("spider.mon").unwrap();
//! sprite.save_png("spider.png").unwrap();
//! ```

pub mod mmi;
pub mod mmm;
pub mod mon;
pub mod nmf;
pub mod palette;
pub mod pic;

pub use mmi::{MmiError, MmiMetadata, MmiTile, TileAttributes};
pub use mmm::{MapTile, MmmError, MmmMap, MmmMapCompact};
pub use mon::{MonError, MonFrame, MonMetadata, MonSprite};
pub use nmf::{NmfError, NmfMap};
pub use palette::{palette_to_rgb, palette_to_rgba, vga6_to_rgb8, VGA_PALETTE};
pub use pic::{PicError, PicTile, TILE_HEIGHT, TILE_PIXELS, TILE_WIDTH};
