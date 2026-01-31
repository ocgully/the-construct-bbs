//! NMF (New Map Format) binary file converter
//!
//! NMF files are binary map files from the original Realm of Ralnar game.
//! Format (based on analysis of MAPCONV.BAS):
//! - All values are little-endian 16-bit integers
//! - First: Map dimensions and metadata
//! - Then: Tile data (tile index + attribute pairs)

use crate::mmm::{MapTile, MmmMap, MmmMapCompact};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when processing NMF files
#[derive(Error, Debug)]
pub enum NmfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File too short: expected at least {expected} bytes, got {actual}")]
    FileTooShort { expected: usize, actual: usize },

    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u16, height: u16 },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Read a little-endian u16 from a byte slice
fn read_le16(data: &[u8], offset: usize) -> u16 {
    if offset + 1 >= data.len() {
        return 0;
    }
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

/// A parsed NMF map (uses same structure as MMM for compatibility)
pub struct NmfMap {
    inner: MmmMap,
}

impl NmfMap {
    /// Parse an NMF file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, NmfError> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let data = fs::read(path)?;
        Self::parse_binary(&data, name)
    }

    /// Parse NMF data from binary content
    fn parse_binary(data: &[u8], name: String) -> Result<Self, NmfError> {
        // Need at least header: name (string), width (2), height (2)
        // The format appears to have the name first, then dimensions

        if data.len() < 8 {
            return Err(NmfError::FileTooShort {
                expected: 8,
                actual: data.len(),
            });
        }

        // Try to find where the binary data starts
        // NMF files may have a text name prefix followed by binary data
        // Or they could be fully binary

        // Look for the pattern: the file seems to start with the map name
        // followed by binary dimension data

        // Find where numeric data might start
        let mut offset = 0;

        // Skip any text prefix (map name)
        // Look for where we get reasonable dimension values
        while offset + 4 <= data.len() {
            let width = read_le16(data, offset);
            let height = read_le16(data, offset + 2);

            // Check if these look like valid dimensions (1-500 range)
            if width > 0 && width <= 500 && height > 0 && height <= 500 {
                // This might be the start of dimensions
                break;
            }
            offset += 1;
        }

        // If we couldn't find valid dimensions, try starting from beginning
        if offset + 4 > data.len() {
            offset = 0;
        }

        let width = read_le16(data, offset);
        let height = read_le16(data, offset + 2);

        if width == 0 || height == 0 || width > 1000 || height > 1000 {
            return Err(NmfError::InvalidDimensions { width, height });
        }

        let expected_tiles = (width as usize) * (height as usize);
        let mut tiles = Vec::with_capacity(expected_tiles);

        // Skip past dimensions to tile data
        let mut data_offset = offset + 4;

        // Each tile is: tile_index (u16) + attribute (u16)
        // But the actual format might pack them differently

        // Based on MAPCONV.BAS analysis, the format alternates:
        // tile_index, attribute, tile_index, attribute...

        while tiles.len() < expected_tiles && data_offset + 4 <= data.len() {
            let tile_index = read_le16(data, data_offset);
            let attribute = read_le16(data, data_offset + 2);

            tiles.push(MapTile {
                tile_index,
                attribute: attribute.min(255) as u8,
            });

            data_offset += 4;
        }

        // Pad with empty tiles if needed
        while tiles.len() < expected_tiles {
            tiles.push(MapTile {
                tile_index: 0,
                attribute: 1,
            });
        }

        Ok(NmfMap {
            inner: MmmMap {
                name,
                width: width as u32,
                height: height as u32,
                tiles,
            },
        })
    }

    /// Parse from byte slice (for testing)
    pub fn from_bytes(data: &[u8], name: &str) -> Result<Self, NmfError> {
        Self::parse_binary(data, name.to_string())
    }

    /// Get the inner map
    pub fn into_inner(self) -> MmmMap {
        self.inner
    }

    /// Get a reference to the inner map
    pub fn as_mmm(&self) -> &MmmMap {
        &self.inner
    }

    /// Get the tile at (x, y) coordinates
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&MapTile> {
        self.inner.get_tile(x, y)
    }

    /// Save as JSON file
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<(), NmfError> {
        let json = serde_json::to_string_pretty(&self.inner)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Get compact format for export
    pub fn to_compact(&self) -> MmmMapCompact {
        MmmMapCompact::from(&self.inner)
    }
}

impl std::ops::Deref for NmfMap {
    type Target = MmmMap;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_nmf_data(width: u16, height: u16, tiles: &[(u16, u16)]) -> Vec<u8> {
        let mut data = Vec::new();

        // Write dimensions as little-endian
        data.extend_from_slice(&width.to_le_bytes());
        data.extend_from_slice(&height.to_le_bytes());

        // Write tile data
        for (tile_idx, attr) in tiles {
            data.extend_from_slice(&tile_idx.to_le_bytes());
            data.extend_from_slice(&attr.to_le_bytes());
        }

        data
    }

    #[test]
    fn test_parse_simple_nmf() {
        let tiles = vec![(1, 1), (2, 2), (3, 3), (4, 4)];
        let data = create_test_nmf_data(2, 2, &tiles);

        let map = NmfMap::from_bytes(&data, "test").unwrap();

        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.tiles.len(), 4);

        assert_eq!(map.tiles[0].tile_index, 1);
        assert_eq!(map.tiles[0].attribute, 1);
    }

    #[test]
    fn test_get_tile() {
        let tiles = vec![(10, 1), (20, 2), (30, 3), (40, 4)];
        let data = create_test_nmf_data(2, 2, &tiles);

        let map = NmfMap::from_bytes(&data, "test").unwrap();

        let tile = map.get_tile(0, 0).unwrap();
        assert_eq!(tile.tile_index, 10);

        let tile = map.get_tile(1, 0).unwrap();
        assert_eq!(tile.tile_index, 20);
    }

    #[test]
    fn test_file_too_short() {
        let data = vec![0, 0, 0]; // Only 3 bytes
        let result = NmfMap::from_bytes(&data, "test");
        assert!(matches!(result, Err(NmfError::FileTooShort { .. })));
    }

    #[test]
    fn test_invalid_dimensions() {
        // Create data with 0x0 dimensions
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let result = NmfMap::from_bytes(&data, "test");
        assert!(matches!(result, Err(NmfError::InvalidDimensions { .. })));
    }

    #[test]
    fn test_to_compact() {
        let tiles = vec![(1, 5), (2, 6)];
        let data = create_test_nmf_data(2, 1, &tiles);

        let map = NmfMap::from_bytes(&data, "test").unwrap();
        let compact = map.to_compact();

        assert_eq!(compact.tile_indices, vec![1, 2]);
        assert_eq!(compact.attributes, vec![5, 6]);
    }
}
