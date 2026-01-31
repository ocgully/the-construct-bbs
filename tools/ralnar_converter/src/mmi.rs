//! MMI file format converter
//!
//! MMI files contain tile data with attributes from the original Realm of Ralnar game.
//! Format (based on analysis of MMICONV.BAS):
//! - Line 1: Always 160 (some kind of identifier)
//! - Line 2: XDIM (width, typically 20)
//! - Line 3: Reserved (0)
//! - Line 4: Reserved (0)
//! - Lines 5+: Combined pixel + attribute data (encoded integers)
//!
//! The data appears to encode both the pixel color and attribute information.
//! Based on the observed values, the format seems to use bit packing.

use crate::palette::palette_to_rgba;
use crate::pic::{TILE_HEIGHT, TILE_WIDTH};
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

/// Tile attribute flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileAttributes {
    /// 1 = Land (passable by foot)
    pub land: bool,
    /// 2 = Water (passable by boat)
    pub water: bool,
    /// 3 = Slow movement
    pub slow: bool,
    /// 4 = Damaging area (also slow)
    pub damaging: bool,
    /// 5 = Impassable
    pub impassable: bool,
    /// 6 = Town entrance
    pub town: bool,
    /// 7 = Cave entrance
    pub cave: bool,
    /// 8 = Dock
    pub dock: bool,
    /// 9 = Passable by all
    pub any_pass: bool,
}

impl Default for TileAttributes {
    fn default() -> Self {
        TileAttributes {
            land: true,
            water: false,
            slow: false,
            damaging: false,
            impassable: false,
            town: false,
            cave: false,
            dock: false,
            any_pass: false,
        }
    }
}

impl TileAttributes {
    /// Create from attribute code (1-9)
    pub fn from_code(code: u8) -> Self {
        match code {
            1 => TileAttributes {
                land: true,
                ..Default::default()
            },
            2 => TileAttributes {
                water: true,
                land: false,
                ..Default::default()
            },
            3 => TileAttributes {
                slow: true,
                ..Default::default()
            },
            4 => TileAttributes {
                damaging: true,
                slow: true,
                ..Default::default()
            },
            5 => TileAttributes {
                impassable: true,
                land: false,
                ..Default::default()
            },
            6 => TileAttributes {
                town: true,
                ..Default::default()
            },
            7 => TileAttributes {
                cave: true,
                ..Default::default()
            },
            8 => TileAttributes {
                dock: true,
                ..Default::default()
            },
            9 => TileAttributes {
                any_pass: true,
                ..Default::default()
            },
            _ => TileAttributes::default(),
        }
    }

    /// Convert to attribute code
    pub fn to_code(&self) -> u8 {
        if self.any_pass {
            9
        } else if self.dock {
            8
        } else if self.cave {
            7
        } else if self.town {
            6
        } else if self.impassable {
            5
        } else if self.damaging {
            4
        } else if self.slow {
            3
        } else if self.water {
            2
        } else {
            1 // Default to land
        }
    }
}

/// Errors that can occur when processing MMI files
#[derive(Error, Debug)]
pub enum MmiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error on line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Invalid header: expected identifier 160, got {0}")]
    InvalidHeader(i32),

    #[error("Invalid pixel count: expected {expected}, got {actual}")]
    InvalidPixelCount { expected: usize, actual: usize },

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Metadata for an MMI tile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MmiMetadata {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub attributes: TileAttributes,
    /// Primary attribute code (1-9)
    pub attribute_code: u8,
}

/// A parsed MMI tile with pixel and attribute data
#[derive(Debug, Clone)]
pub struct MmiTile {
    /// Pixel data as palette indices (-1 = transparent)
    pub pixels: Vec<i16>,
    /// Per-pixel attribute codes (if available)
    pub pixel_attributes: Vec<u8>,
    /// Primary tile attribute
    pub attributes: TileAttributes,
    /// Source filename (without extension)
    pub name: String,
    /// Tile dimensions
    pub width: u32,
    pub height: u32,
}

impl MmiTile {
    /// Parse an MMI file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MmiError> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines: Vec<i32> = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            let value: i32 = trimmed.parse().map_err(|_| MmiError::Parse {
                line: line_num + 1,
                message: format!("invalid integer: '{}'", trimmed),
            })?;

            lines.push(value);
        }

        Self::parse_mmi_data(&lines, name)
    }

    /// Parse MMI data from a vector of integers
    fn parse_mmi_data(data: &[i32], name: String) -> Result<Self, MmiError> {
        if data.len() < 4 {
            return Err(MmiError::Parse {
                line: 0,
                message: "file too short".to_string(),
            });
        }

        // Validate header (first value should be 160)
        // But we'll be lenient and accept any file
        let _header = data[0];
        let xdim = data[1] as u32;
        let _reserved1 = data[2];
        let _reserved2 = data[3];

        // Use standard tile dimensions if xdim is reasonable
        let width = if xdim == 20 { TILE_WIDTH } else { xdim };
        let height = TILE_HEIGHT;
        let expected_pixels = (width * height) as usize;

        // Parse pixel data starting from line 5 (index 4)
        // The MMI format encodes color and attribute together
        // Based on analysis: values seem to be encoded as combined pixel+attribute
        let mut pixels = Vec::with_capacity(expected_pixels);
        let mut pixel_attributes = Vec::with_capacity(expected_pixels);

        // Track the most common non-zero attribute for the tile's primary attribute
        let mut attribute_counts = [0u32; 10];

        for &value in data.iter().skip(4) {
            if pixels.len() >= expected_pixels {
                break;
            }

            // Decode the combined value
            // The encoding seems to be: low byte = color, high bits = attribute
            // But observing values like 514, 2562, 30583, we need to decode properly

            // Values observed: 0, 2, 512, 514, 631, 2562, 2570, 2679, 29440, 29556, 29811, 30466, 30583
            // 512 = 0x200, 514 = 0x202, 631 = 0x277
            // It seems the color is in the lower bits

            if value == 0 {
                // 0 typically means transparent or empty
                pixels.push(-1);
                pixel_attributes.push(0);
            } else {
                // Extract color: lower 8 bits seem to be the color index
                let color = (value & 0xFF) as i16;

                // Extract attribute: bits 9-12 seem to encode the attribute
                // Shift right by 9 bits and mask to get attribute code
                let attr = ((value >> 9) & 0xF) as u8;

                // If color is 0xFF or similar high values that don't make sense,
                // treat the whole value differently
                if color >= 0 && color <= 255 {
                    pixels.push(color);
                } else {
                    pixels.push(-1);
                }

                let attr_code = if attr > 0 && attr <= 9 { attr } else { 1 };
                pixel_attributes.push(attr_code);
                attribute_counts[attr_code as usize] += 1;
            }
        }

        // Pad with transparent if needed
        while pixels.len() < expected_pixels {
            pixels.push(-1);
            pixel_attributes.push(0);
        }

        // Determine primary attribute from most common non-zero
        let primary_attr = attribute_counts
            .iter()
            .enumerate()
            .skip(1)
            .max_by_key(|(_, &count)| count)
            .map(|(idx, _)| idx as u8)
            .unwrap_or(1);

        Ok(MmiTile {
            pixels,
            pixel_attributes,
            attributes: TileAttributes::from_code(primary_attr),
            name,
            width,
            height,
        })
    }

    /// Get the pixel value at (x, y) coordinates
    pub fn get_pixel(&self, x: u32, y: u32) -> i16 {
        // Column-major order like PIC files
        let index = (x * self.height + y) as usize;
        if index < self.pixels.len() {
            self.pixels[index]
        } else {
            -1
        }
    }

    /// Convert to RGBA image
    pub fn to_image(&self) -> RgbaImage {
        let mut img: RgbaImage = ImageBuffer::new(self.width, self.height);

        for x in 0..self.width {
            for y in 0..self.height {
                let pixel_value = self.get_pixel(x, y);
                let color = if pixel_value == -1 {
                    Rgba([0, 0, 0, 0])
                } else {
                    let (r, g, b, a) = palette_to_rgba(pixel_value as u8);
                    Rgba([r, g, b, a])
                };
                img.put_pixel(x, y, color);
            }
        }

        img
    }

    /// Get metadata for JSON export
    pub fn get_metadata(&self) -> MmiMetadata {
        MmiMetadata {
            name: self.name.clone(),
            width: self.width,
            height: self.height,
            attributes: self.attributes,
            attribute_code: self.attributes.to_code(),
        }
    }

    /// Save as PNG file
    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), MmiError> {
        let img = self.to_image();
        img.save(path)?;
        Ok(())
    }

    /// Save metadata as JSON
    pub fn save_metadata<P: AsRef<Path>>(&self, path: P) -> Result<(), MmiError> {
        let metadata = self.get_metadata();
        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_attributes_from_code() {
        let land = TileAttributes::from_code(1);
        assert!(land.land);
        assert!(!land.water);

        let water = TileAttributes::from_code(2);
        assert!(water.water);
        assert!(!water.land);

        let impassable = TileAttributes::from_code(5);
        assert!(impassable.impassable);
        assert!(!impassable.land);
    }

    #[test]
    fn test_tile_attributes_to_code() {
        assert_eq!(TileAttributes::from_code(1).to_code(), 1);
        assert_eq!(TileAttributes::from_code(5).to_code(), 5);
        assert_eq!(TileAttributes::from_code(9).to_code(), 9);
    }

    #[test]
    fn test_parse_mmi_data() {
        // Minimal MMI data: header, xdim, reserved, reserved, then pixel data
        let mut data = vec![160, 20, 0, 0];
        // Add enough pixel data for 20x20
        for _ in 0..400 {
            data.push(2); // All green pixels with attribute 0
        }

        let tile = MmiTile::parse_mmi_data(&data, "test".to_string()).unwrap();
        assert_eq!(tile.name, "test");
        assert_eq!(tile.width, 20);
        assert_eq!(tile.height, 20);
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = MmiMetadata {
            name: "test".to_string(),
            width: 20,
            height: 20,
            attributes: TileAttributes::from_code(1),
            attribute_code: 1,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"land\":true"));
    }
}
