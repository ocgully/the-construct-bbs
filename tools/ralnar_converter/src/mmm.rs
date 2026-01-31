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

        // Parse tile/attribute pairs starting from line 4
        // Format: tile_index on one line, attribute on the next
        // Note: Line 4 might be a header value (often "0") that should be skipped
        let data_lines = &lines[3..];
        let mut i = 0;

        // Check if line 4 is a header (value "0" alone)
        if !data_lines.is_empty() && data_lines[0].trim() == "0" {
            i = 1; // Skip the header
        }

        while i + 1 < data_lines.len() && tiles.len() < expected_tiles {
            let tile_line = data_lines[i].trim();
            let attr_line = data_lines[i + 1].trim();

            if tile_line.is_empty() || attr_line.is_empty() {
                i += 1;
                continue;
            }

            let tile_index: u16 = tile_line.parse().map_err(|_| MmmError::Parse {
                line: 4 + i,
                message: format!("invalid tile index: '{}'", tile_line),
            })?;

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

        // Don't fail if we have fewer tiles than expected - just pad with empty
        while tiles.len() < expected_tiles {
            tiles.push(MapTile {
                tile_index: 0,
                attribute: 1,
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

/// Compact map format for export
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

        // Check first tile
        assert_eq!(map.tiles[0].tile_index, 1);
        assert_eq!(map.tiles[0].attribute, 1);

        // Check last tile
        assert_eq!(map.tiles[5].tile_index, 6);
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

        let tile_00 = map.get_tile(0, 0).unwrap();
        assert_eq!(tile_00.tile_index, 10);

        let tile_10 = map.get_tile(1, 0).unwrap();
        assert_eq!(tile_10.tile_index, 20);

        let tile_01 = map.get_tile(0, 1).unwrap();
        assert_eq!(tile_01.tile_index, 30);

        let tile_11 = map.get_tile(1, 1).unwrap();
        assert_eq!(tile_11.tile_index, 40);

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

        assert_eq!(compact.tile_indices, vec![1, 2, 3, 4]);
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
