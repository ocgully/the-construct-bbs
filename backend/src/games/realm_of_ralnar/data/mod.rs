//! Data module for Realm of Ralnar
//!
//! Contains all game data definitions: items, enemies, spells, characters, and guardians.

pub mod config;
pub mod items;
pub mod enemies;
pub mod spells;
pub mod characters;
pub mod guardians;

#[allow(unused_imports)]
pub use config::*;
#[allow(unused_imports)]
pub use items::*;
#[allow(unused_imports)]
pub use enemies::*;
#[allow(unused_imports)]
pub use spells::*;
#[allow(unused_imports)]
pub use characters::*;
#[allow(unused_imports)]
pub use guardians::*;
