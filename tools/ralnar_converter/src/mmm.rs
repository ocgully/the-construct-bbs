//! MMM (Map Maker Map) text file format converter
//!
//! MMM files are text-based map files from the original Realm of Ralnar game.
//! Format (based on analysis of TOWN1.MMM and MAPMAKRB.BAS):
//! - Line 1: Map name in quotes (e.g., "TOWN1")
//! - Line 2: Map width (e.g., 35)
//! - Line 3: Map height (e.g., 35)
//! - Following lines: Alternating tile index and attribute pairs
//!   - Even lines (after header): tile index
//!   - Odd lines (after header): attribute code

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when processing MMM files
#[derive(Error, Debug)]
pub enum MmmError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error on line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Missing map name")]
    MissingName,

    #[error("Missing map dimensions")]
    MissingDimensions,

    #[error("Invalid tile count: expected {expected}, got {actual}")]
    InvalidTileCount { expected: usize, actual: usize },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Marker for empty/void tiles (raw index 0 in MMM means "no tile")
/// We use u16::MAX to distinguish from valid tile index 0
pub const EMPTY_TILE_MARKER: u16 = u16::MAX;

/// A single map tile with its index and attribute
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MapTile {
    /// Index into the tile/icon set
    pub tile_index: u16,
    /// Attribute code (1-9, defines passability)
    pub attribute: u8,
}

/// A parsed MMM map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmmMap {
    /// Map name (from the first line)
    pub name: String,
    /// Map width in tiles
    pub width: u32,
    /// Map height in tiles
    pub height: u32,
    /// Tile data in row-major order
    pub tiles: Vec<MapTile>,
}

impl MmmMap {
    /// Parse an MMM file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MmmError> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let lines: Vec<String> = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MmmError::Io(e))?;

        Self::parse_lines(&lines)
    }

    /// Parse MMM data from lines
    fn parse_lines(lines: &[String]) -> Result<Self, MmmError> {
        if lines.is_empty() {
            return Err(MmmError::MissingName);
        }

        // Line 1: Map name (in quotes)
        let name = lines[0]
            .trim()
            .trim_matches('"')
            .to_string();

        if name.is_empty() {
            return Err(MmmError::MissingName);
        }

        if lines.len() < 3 {
            return Err(MmmError::MissingDimensions);
        }

        // Line 2: Width
        let width: u32 = lines[1].trim().parse().map_err(|_| MmmError::Parse {
            line: 2,
            message: format!("invalid width: '{}'", lines[1].trim()),
        })?;

        // Line 3: Height
        let height: u32 = lines[2].trim().parse().map_err(|_| MmmError::Parse {
            line: 3,
            message: format!("invalid height: '{}'", lines[2].trim()),
        })?;

        let expected_tiles = (width * height) as usize;
        let mut tiles = Vec::with_capacity(expected_tiles);

        // Parse tile/attribute pairs
        // Format: tile_index on one line, attribute on the next
        // Some files have a header flag on line 4 (0 or 1) that must be skipped
        // Detect by checking if line count is odd (has header) or even (no header)
        let data_lines = &lines[3..];

        // Each tile needs 2 lines (index + attribute)
        // If data_lines is even: no header, exactly expected*2 lines
        // If data_lines is odd: has header, expected*2 + 1 lines
        let has_header = data_lines.len() % 2 == 1;
        let mut i = if has_header { 1 } else { 0 };

        while i + 1 < data_lines.len() && tiles.len() < expected_tiles {
            let tile_line = data_lines[i].trim();
            let attr_line = data_lines[i + 1].trim();

            if tile_line.is_empty() || attr_line.is_empty() {
                i += 1;
                continue;
            }

            let raw_tile_index: u16 = tile_line.parse().map_err(|_| MmmError::Parse {
                line: 4 + i,
                message: format!("invalid tile index: '{}'", tile_line),
            })?;

            // MMM files use 1-based tile indices - convert to 0-based
            // Special case: raw index 0 means "empty/no tile" (render as black)
            // We use EMPTY_TILE_MARKER (u16::MAX) to distinguish from valid tile index 0
            let tile_index = if raw_tile_index == 0 {
                EMPTY_TILE_MARKER // Empty tile - render as black
            } else {
                raw_tile_index - 1 // Convert 1-based to 0-based
            };

            // Attribute can be larger than 9 in some cases (extended attributes)
            // Store as u8 but allow parsing as u16 first
            let attr_raw: u16 = attr_line.parse().map_err(|_| MmmError::Parse {
                line: 5 + i,
                message: format!("invalid attribute: '{}'", attr_line),
            })?;
            let attribute = attr_raw.min(255) as u8;

            tiles.push(MapTile {
                tile_index,
                attribute,
            });

            i += 2;
        }

        if tiles.len() != expected_tiles {
            return Err(MmmError::InvalidTileCount {
                expected: expected_tiles,
                actual: tiles.len(),
            });
        }

        Ok(MmmMap {
            name,
            width,
            height,
            tiles,
        })
    }

    /// Parse from string content (for testing)
    pub fn from_str(content: &str) -> Result<Self, MmmError> {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Self::parse_lines(&lines)
    }

    /// Get the tile at (x, y) coordinates
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&MapTile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.tiles.get(index)
    }

    /// Save as JSON file
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<(), MmmError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Compact map format for export (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmmMapCompact {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// Tile indices as a flat array (row-major)
    pub tile_indices: Vec<u16>,
    /// Attributes as a flat array (row-major)
    pub attributes: Vec<u8>,
}

impl From<&MmmMap> for MmmMapCompact {
    fn from(map: &MmmMap) -> Self {
        MmmMapCompact {
            name: map.name.clone(),
            width: map.width,
            height: map.height,
            tile_indices: map.tiles.iter().map(|t| t.tile_index).collect(),
            attributes: map.tiles.iter().map(|t| t.attribute).collect(),
        }
    }
}

/// Map properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProperties {
    #[serde(rename = "type")]
    pub map_type: String,
    pub encounters_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music: Option<String>,
    pub world_wrap: bool,
}

impl Default for MapProperties {
    fn default() -> Self {
        Self {
            map_type: "town".to_string(),
            encounters_enabled: false,
            music: None,
            world_wrap: false,
        }
    }
}

/// Spawn point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub x: u32,
    pub y: u32,
    pub direction: String,
}

/// Per-map tile entry (references tile by name only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetEntry {
    pub name: String,
}

/// Sparse attribute override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeOverride {
    pub x: u32,
    pub y: u32,
    pub attribute: u8,
}

/// Spec-compliant map format with per-map tileset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapJson {
    pub name: String,
    pub version: u32,
    pub dimensions: MapDimensions,
    pub properties: MapProperties,
    pub spawn: SpawnPoint,
    /// Per-map tileset with tile names and default attributes
    pub tileset: Vec<TilesetEntry>,
    /// 2D tile indices into local tileset (row-major, inner vec is one row)
    pub tiles: Vec<Vec<u16>>,
    /// Sparse attribute overrides (only tiles with non-default attributes)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attribute_overrides: Vec<AttributeOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDimensions {
    pub width: u32,
    pub height: u32,
}

impl MmmMap {
    /// Convert to spec-compliant JSON format with tile registry
    /// tile_attributes contains the authoritative default attribute for each tile (from metadata)
    pub fn to_map_json(&self, tile_registry: &[String], tile_attributes: &std::collections::HashMap<String, u8>) -> MapJson {
        use std::collections::HashMap;

        // Build per-map tileset from tiles actually used
        // Also track each tile's metadata default attribute for override detection
        // Key is (tile_index, use_a_variant) to handle tiles 126-147 which have A variants
        let mut global_to_local: HashMap<(u16, bool), u16> = HashMap::new();
        let mut tileset: Vec<TilesetEntry> = Vec::new();
        let mut tile_defaults: Vec<u8> = Vec::new(); // Parallel to tileset - metadata default for each

        for tile in &self.tiles {
            // Tiles 125-146 (0-based) have "A" variants used when attribute is 5
            // These are tree/foliage tiles vs lava/fire tiles
            let use_a_variant = tile.tile_index >= 125 && tile.tile_index <= 146 && tile.attribute == 5;
            let key = (tile.tile_index, use_a_variant);

            if !global_to_local.contains_key(&key) {
                let local_idx = tileset.len() as u16;
                global_to_local.insert(key, local_idx);

                // Get tile name from registry
                // EMPTY_TILE_MARKER means "empty/no tile" (render as black)
                let name = if tile.tile_index == EMPTY_TILE_MARKER {
                    "empty".to_string()
                } else if (tile.tile_index as usize) < tile_registry.len() {
                    let base_name = tile_registry[tile.tile_index as usize].clone();
                    if use_a_variant {
                        format!("{}a", base_name)
                    } else {
                        base_name
                    }
                } else {
                    format!("tile_{}", tile.tile_index)
                };

                // Get default attribute from tile metadata (authoritative source)
                // Empty tiles have attribute 0 (impassable/void)
                let metadata_attr = if tile.tile_index == EMPTY_TILE_MARKER {
                    0
                } else {
                    tile_attributes.get(&name).copied().unwrap_or(1)
                };

                tileset.push(TilesetEntry { name });
                tile_defaults.push(metadata_attr);
            }
        }

        // Build 2D tile array with local indices and collect sparse overrides
        // Only add override when map attribute differs from tile's METADATA default
        // Source data is stored in column-major order (x varies fastest)
        let mut tiles_2d: Vec<Vec<u16>> = Vec::with_capacity(self.height as usize);
        let mut overrides: Vec<AttributeOverride> = Vec::new();

        for y in 0..self.height {
            let mut row: Vec<u16> = Vec::with_capacity(self.width as usize);

            for x in 0..self.width {
                // Column-major indexing: data stored as columns, not rows
                let idx = (x * self.height + y) as usize;
                let tile = &self.tiles[idx];
                // Use same composite key logic as when building tileset
                let use_a_variant = tile.tile_index >= 125 && tile.tile_index <= 146 && tile.attribute == 5;
                let key = (tile.tile_index, use_a_variant);
                let local_idx = global_to_local[&key];
                row.push(local_idx);

                // Only override if map attribute differs from tile's metadata default
                let metadata_default = tile_defaults[local_idx as usize];
                if tile.attribute != metadata_default {
                    overrides.push(AttributeOverride {
                        x,
                        y,
                        attribute: tile.attribute,
                    });
                }
            }

            tiles_2d.push(row);
        }

        // Determine map type from name
        let map_type = if self.name.to_uppercase().contains("TOWN") {
            "town"
        } else if self.name.to_uppercase().contains("CAVE") || self.name.to_uppercase().contains("DUNGEON") {
            "dungeon"
        } else if self.name.to_uppercase().contains("CASTLE") {
            "castle"
        } else if self.name.to_uppercase().contains("WORLD") {
            "overworld"
        } else {
            "interior"
        };

        MapJson {
            name: self.name.clone(),
            version: 1,
            dimensions: MapDimensions {
                width: self.width,
                height: self.height,
            },
            properties: MapProperties {
                map_type: map_type.to_string(),
                encounters_enabled: map_type == "overworld" || map_type == "dungeon",
                music: None,
                world_wrap: map_type == "overworld",
            },
            spawn: SpawnPoint {
                x: self.width / 2,
                y: self.height / 2,
                direction: "down".to_string(),
            },
            tileset,
            tiles: tiles_2d,
            attribute_overrides: overrides,
        }
    }

    /// Save as spec-compliant JSON with compact row formatting
    pub fn save_map_json<P: AsRef<Path>>(
        &self,
        path: P,
        tile_registry: &[String],
        tile_attributes: &std::collections::HashMap<String, u8>,
    ) -> Result<(), MmmError> {
        let map_json = self.to_map_json(tile_registry, tile_attributes);
        let json = format_map_json_compact(&map_json)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Format MapJson with compact row representation (each row on one line)
fn format_map_json_compact(map: &MapJson) -> Result<String, MmmError> {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"name\": {},\n", serde_json::to_string(&map.name)?));
    output.push_str(&format!("  \"version\": {},\n", map.version));

    // Dimensions
    output.push_str("  \"dimensions\": {\n");
    output.push_str(&format!("    \"width\": {},\n", map.dimensions.width));
    output.push_str(&format!("    \"height\": {}\n", map.dimensions.height));
    output.push_str("  },\n");

    // Properties
    output.push_str("  \"properties\": {\n");
    output.push_str(&format!("    \"type\": {},\n", serde_json::to_string(&map.properties.map_type)?));
    output.push_str(&format!("    \"encounters_enabled\": {},\n", map.properties.encounters_enabled));
    if let Some(ref music) = map.properties.music {
        output.push_str(&format!("    \"music\": {},\n", serde_json::to_string(music)?));
    }
    output.push_str(&format!("    \"world_wrap\": {}\n", map.properties.world_wrap));
    output.push_str("  },\n");

    // Spawn
    output.push_str("  \"spawn\": {\n");
    output.push_str(&format!("    \"x\": {},\n", map.spawn.x));
    output.push_str(&format!("    \"y\": {},\n", map.spawn.y));
    output.push_str(&format!("    \"direction\": {}\n", serde_json::to_string(&map.spawn.direction)?));
    output.push_str("  },\n");

    // Tileset (tile names only - attributes come from tile metadata)
    output.push_str("  \"tileset\": [\n");
    for (i, entry) in map.tileset.iter().enumerate() {
        let comma = if i < map.tileset.len() - 1 { "," } else { "" };
        output.push_str(&format!("    {}{}\n", serde_json::to_string(&entry.name)?, comma));
    }
    output.push_str("  ],\n");

    // Tiles - each row on one line
    output.push_str("  \"tiles\": [\n");
    for (i, row) in map.tiles.iter().enumerate() {
        let comma = if i < map.tiles.len() - 1 { "," } else { "" };
        let row_str: Vec<String> = row.iter().map(|n| n.to_string()).collect();
        output.push_str(&format!("    [{}]{}\n", row_str.join(","), comma));
    }
    output.push_str("  ]");

    // Sparse attribute overrides (only if any exist)
    if !map.attribute_overrides.is_empty() {
        output.push_str(",\n  \"attribute_overrides\": [\n");
        for (i, ovr) in map.attribute_overrides.iter().enumerate() {
            let comma = if i < map.attribute_overrides.len() - 1 { "," } else { "" };
            output.push_str(&format!(
                "    {{ \"x\": {}, \"y\": {}, \"attribute\": {} }}{}\n",
                ovr.x, ovr.y, ovr.attribute, comma
            ));
        }
        output.push_str("  ]");
    }

    output.push_str("\n}\n");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_map() {
        let content = r#""TEST"
3
2
1
1
2
1
3
1
4
1
5
1
6
1"#;
        let map = MmmMap::from_str(content).unwrap();

        assert_eq!(map.name, "TEST");
        assert_eq!(map.width, 3);
        assert_eq!(map.height, 2);
        assert_eq!(map.tiles.len(), 6);

        // Check first tile (1-based input becomes 0-based: 1 -> 0)
        assert_eq!(map.tiles[0].tile_index, 0);
        assert_eq!(map.tiles[0].attribute, 1);

        // Check last tile (1-based input becomes 0-based: 6 -> 5)
        assert_eq!(map.tiles[5].tile_index, 5);
        assert_eq!(map.tiles[5].attribute, 1);
    }

    #[test]
    fn test_get_tile() {
        let content = r#""TEST"
2
2
10
1
20
2
30
3
40
4"#;
        let map = MmmMap::from_str(content).unwrap();

        // 1-based input becomes 0-based: 10 -> 9, 20 -> 19, etc.
        let tile_00 = map.get_tile(0, 0).unwrap();
        assert_eq!(tile_00.tile_index, 9);

        let tile_10 = map.get_tile(1, 0).unwrap();
        assert_eq!(tile_10.tile_index, 19);

        let tile_01 = map.get_tile(0, 1).unwrap();
        assert_eq!(tile_01.tile_index, 29);

        let tile_11 = map.get_tile(1, 1).unwrap();
        assert_eq!(tile_11.tile_index, 39);

        assert!(map.get_tile(2, 0).is_none());
    }

    #[test]
    fn test_compact_format() {
        let content = r#""TEST"
2
2
1
5
2
6
3
7
4
8"#;
        let map = MmmMap::from_str(content).unwrap();
        let compact = MmmMapCompact::from(&map);

        // 1-based input becomes 0-based: 1->0, 2->1, 3->2, 4->3
        assert_eq!(compact.tile_indices, vec![0, 1, 2, 3]);
        assert_eq!(compact.attributes, vec![5, 6, 7, 8]);
    }

    #[test]
    fn test_missing_name() {
        let result = MmmMap::from_str("");
        assert!(matches!(result, Err(MmmError::MissingName)));
    }

    #[test]
    fn test_json_serialization() {
        let content = r#""TEST"
2
2
1
1
2
2
3
3
4
4"#;
        let map = MmmMap::from_str(content).unwrap();
        let json = serde_json::to_string(&map).unwrap();

        assert!(json.contains("\"name\":\"TEST\""));
        assert!(json.contains("\"width\":2"));
        assert!(json.contains("\"height\":2"));
    }
}
