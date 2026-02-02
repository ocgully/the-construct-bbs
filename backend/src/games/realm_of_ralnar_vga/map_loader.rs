//! Map loader for Realm of Ralnar VGA
//!
//! Loads converted JSON map files and tile assets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Tile registry mapping index to name
pub struct TileRegistry {
    names: Vec<String>,
}

impl TileRegistry {
    /// Load from MMIFILES.TXT format
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let names: Vec<String> = content
            .lines()
            .map(|line| {
                line.trim()
                    .trim_end_matches(".MMI")
                    .trim_end_matches(".mmi")
                    .to_lowercase()
            })
            .filter(|s| !s.is_empty())
            .collect();
        Ok(Self { names })
    }

    /// Create from explicit list
    pub fn from_names(names: Vec<String>) -> Self {
        Self { names }
    }

    /// Get tile name from 1-based index
    pub fn get_name(&self, index: u16) -> Option<&str> {
        if index == 0 || index as usize > self.names.len() {
            return None;
        }
        Some(&self.names[(index - 1) as usize])
    }

    /// Get total tile count
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Raw tile data from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMapTile {
    pub tile_index: u16,
    pub attribute: u8,
}

/// Raw map data from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMap {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<RawMapTile>,
}

/// Processed tile with resolved name
#[derive(Debug, Clone)]
pub struct MapTile {
    pub tile_name: String,
    pub tile_index: u16,
    pub attribute: TileAttribute,
}

/// Tile attribute flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileAttribute {
    /// Passable on foot
    Land,
    /// Requires boat
    Water,
    /// Slow movement
    Slow,
    /// Damages player
    Damaging,
    /// Cannot pass
    Blocked,
    /// Town entrance
    Town,
    /// Cave entrance
    Cave,
    /// Dock (board/leave ship)
    Dock,
    /// Any movement type passes
    Universal,
}

impl From<u8> for TileAttribute {
    fn from(code: u8) -> Self {
        match code {
            1 => TileAttribute::Land,
            2 => TileAttribute::Water,
            3 => TileAttribute::Slow,
            4 => TileAttribute::Damaging,
            5 => TileAttribute::Blocked,
            6 => TileAttribute::Town,
            7 => TileAttribute::Cave,
            8 => TileAttribute::Dock,
            9 => TileAttribute::Universal,
            _ => TileAttribute::Blocked,
        }
    }
}

/// Loaded map with resolved tile names
#[derive(Debug)]
pub struct Map {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<MapTile>,
    /// Unique tile names used in this map
    pub tileset: Vec<String>,
}

impl Map {
    /// Load map from JSON file
    pub fn from_file<P: AsRef<Path>>(path: P, registry: &TileRegistry) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let raw: RawMap = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Self::from_raw(raw, registry)
    }

    /// Convert raw map using tile registry
    pub fn from_raw(raw: RawMap, registry: &TileRegistry) -> std::io::Result<Self> {
        let mut tileset_set = std::collections::HashSet::new();
        let mut tiles = Vec::with_capacity(raw.tiles.len());

        for raw_tile in &raw.tiles {
            let tile_name = registry
                .get_name(raw_tile.tile_index)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("tile_{}", raw_tile.tile_index));

            tileset_set.insert(tile_name.clone());

            tiles.push(MapTile {
                tile_name,
                tile_index: raw_tile.tile_index,
                attribute: raw_tile.attribute.into(),
            });
        }

        let tileset: Vec<String> = tileset_set.into_iter().collect();

        Ok(Self {
            name: raw.name,
            width: raw.width,
            height: raw.height,
            tiles,
            tileset,
        })
    }

    /// Get tile at (x, y) position
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&MapTile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.tiles.get(idx)
    }

    /// Check if position is passable on foot
    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        self.get_tile(x, y)
            .map(|t| matches!(t.attribute,
                TileAttribute::Land |
                TileAttribute::Slow |
                TileAttribute::Town |
                TileAttribute::Cave |
                TileAttribute::Dock |
                TileAttribute::Universal
            ))
            .unwrap_or(false)
    }

    /// Check if position requires a ship
    pub fn is_water(&self, x: u32, y: u32) -> bool {
        self.get_tile(x, y)
            .map(|t| t.attribute == TileAttribute::Water)
            .unwrap_or(false)
    }
}

/// Map cache for loaded maps
#[derive(Default)]
pub struct MapCache {
    maps: HashMap<String, Map>,
    registry: Option<TileRegistry>,
}

impl MapCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_registry(registry: TileRegistry) -> Self {
        Self {
            maps: HashMap::new(),
            registry: Some(registry),
        }
    }

    pub fn set_registry(&mut self, registry: TileRegistry) {
        self.registry = Some(registry);
    }

    /// Load and cache a map
    pub fn load_map<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<&Map> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_uppercase();

        if !self.maps.contains_key(&name) {
            let registry = self.registry.as_ref().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "No tile registry loaded")
            })?;
            let map = Map::from_file(path, registry)?;
            self.maps.insert(name.clone(), map);
        }

        Ok(self.maps.get(&name).unwrap())
    }

    /// Get cached map by name
    pub fn get_map(&self, name: &str) -> Option<&Map> {
        self.maps.get(&name.to_uppercase())
    }
}
