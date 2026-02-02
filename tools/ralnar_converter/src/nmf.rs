//! NMF (New Map Format) binary file converter
//!
//! NMF files are binary map files from the original Realm of Ralnar game.
//! Format (based on reverse engineering):
//! - Bytes 0-1: Width (u16 little-endian)
//! - Bytes 2-3: Height (u16 little-endian)
//! - Bytes 4-11: Icon list (4 u16 entries) - preloaded tile indices
//! - Bytes 12+: Tile data as (tile_index u16, attribute u16) pairs
//!
//! Tile index interpretation:
//! - The tile indices are 0-based references into MMIFILES.TXT
//! - Both MMM and NMF use 1-based tile indices that we convert to 0-based

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

    #[error("Invalid tile count: expected {expected}, got {actual}")]
    InvalidTileCount { expected: usize, actual: usize },

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
        // NMF format: width(2) + height(2) + icon_list(8) + tile_data
        // Minimum: 12 bytes header
        if data.len() < 12 {
            return Err(NmfError::FileTooShort {
                expected: 12,
                actual: data.len(),
            });
        }

        // Read dimensions from bytes 0-3
        let width = read_le16(data, 0);
        let height = read_le16(data, 2);

        if width == 0 || height == 0 || width > 1000 || height > 1000 {
            return Err(NmfError::InvalidDimensions { width, height });
        }

        // Bytes 4-11: Icon list (4 u16 entries) - skip for now (preload hint)

        let expected_tiles = (width as usize) * (height as usize);
        let mut tiles = Vec::with_capacity(expected_tiles);

        // Tile data starts at byte 12
        // Each tile is 4 bytes: tile_index (u16) + attribute (u16)
        let mut data_offset = 12;

        while tiles.len() < expected_tiles && data_offset + 4 <= data.len() {
            let raw_tile_index = read_le16(data, data_offset);
            let attribute = read_le16(data, data_offset + 2);

            // NMF uses 1-based tile indices like MMM - convert to 0-based
            let tile_index = if raw_tile_index > 0 {
                raw_tile_index - 1
            } else {
                0
            };

            tiles.push(MapTile {
                tile_index,
                attribute: attribute.min(255) as u8,
            });

            data_offset += 4;
        }

        // Fail if tile count doesn't match expected
        if tiles.len() != expected_tiles {
            return Err(NmfError::InvalidTileCount {
                expected: expected_tiles,
                actual: tiles.len(),
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

    /// Save as JSON file (raw format - use save_map_json for optimized format)
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<(), NmfError> {
        let json = serde_json::to_string_pretty(&self.inner)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Save as spec-compliant JSON with compact row formatting and sparse attribute overrides
    pub fn save_map_json<P: AsRef<Path>>(
        &self,
        path: P,
        tile_registry: &[String],
        tile_attributes: &std::collections::HashMap<String, u8>,
    ) -> Result<(), NmfError> {
        self.inner
            .save_map_json(path, tile_registry, tile_attributes)
            .map_err(|e| NmfError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
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

        // Write dimensions as little-endian (bytes 0-3)
        data.extend_from_slice(&width.to_le_bytes());
        data.extend_from_slice(&height.to_le_bytes());

        // Write icon list bytes 4-11 (placeholder)
        data.extend_from_slice(&0u16.to_le_bytes()); // bytes 4-5
        data.extend_from_slice(&0u16.to_le_bytes()); // bytes 6-7
        data.extend_from_slice(&0u16.to_le_bytes()); // bytes 8-9
        data.extend_from_slice(&0u16.to_le_bytes()); // bytes 10-11

        // Write tile data (bytes 12+)
        // Each tile is 4 bytes: tile_index (u16) + attribute (u16)
        for (tile_idx, attr) in tiles {
            data.extend_from_slice(&tile_idx.to_le_bytes());
            data.extend_from_slice(&attr.to_le_bytes());
        }

        data
    }

    #[test]
    fn test_parse_simple_nmf() {
        // Tiles are (1-based tile index, attribute) - like MMM format
        let tiles: Vec<(u16, u16)> = vec![(1, 1), (2, 2), (3, 3), (4, 4)];
        let data = create_test_nmf_data(2, 2, &tiles);

        let map = NmfMap::from_bytes(&data, "test").unwrap();

        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.tiles.len(), 4);

        // 1-based input becomes 0-based: 1 -> 0
        assert_eq!(map.tiles[0].tile_index, 0);
        assert_eq!(map.tiles[0].attribute, 1);
    }

    #[test]
    fn test_get_tile() {
        // Tiles are (1-based tile index, attribute) - like MMM format
        let tiles: Vec<(u16, u16)> = vec![(10, 1), (20, 2), (30, 3), (40, 4)];
        let data = create_test_nmf_data(2, 2, &tiles);

        let map = NmfMap::from_bytes(&data, "test").unwrap();

        // 1-based input becomes 0-based: 10 -> 9, 20 -> 19
        let tile = map.get_tile(0, 0).unwrap();
        assert_eq!(tile.tile_index, 9);

        let tile = map.get_tile(1, 0).unwrap();
        assert_eq!(tile.tile_index, 19);
    }

    #[test]
    fn test_file_too_short() {
        let data = vec![0, 0, 0]; // Only 3 bytes
        let result = NmfMap::from_bytes(&data, "test");
        assert!(matches!(result, Err(NmfError::FileTooShort { .. })));
    }

    #[test]
    fn test_invalid_dimensions() {
        // Create data with 0x0 dimensions (need 12 bytes minimum: 4 dim + 8 icon list)
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = NmfMap::from_bytes(&data, "test");
        assert!(matches!(result, Err(NmfError::InvalidDimensions { .. })));
    }

    #[test]
    fn test_to_compact() {
        // Tiles are (1-based tile index, attribute) - like MMM format
        let tiles: Vec<(u16, u16)> = vec![(1, 5), (2, 6)];
        let data = create_test_nmf_data(2, 1, &tiles);

        let map = NmfMap::from_bytes(&data, "test").unwrap();
        let compact = map.to_compact();

        // 1-based input becomes 0-based: 1 -> 0, 2 -> 1
        assert_eq!(compact.tile_indices, vec![0, 1]);
        assert_eq!(compact.attributes, vec![5, 6]);
    }
}
