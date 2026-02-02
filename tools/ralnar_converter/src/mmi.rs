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
        if data.len() < 5 {
            return Err(MmiError::Parse {
                line: 0,
                message: "file too short".to_string(),
            });
        }

        // Header format (QBasic GET array):
        // Line 1: width * 8 (e.g., 160 for 20px wide)
        // Line 2: height (e.g., 20)
        // Lines 3-202: packed pixel data (200 integers = 400 pixels)
        // Line 203 (or last): tile attribute code

        let width_raw = data[0];
        let height = data[1] as u32;

        // Width is stored as width * 8 in QBasic GET format
        let width = if width_raw == 160 {
            TILE_WIDTH
        } else {
            (width_raw / 8).max(1) as u32
        };

        let expected_pixels = (width * height) as usize;

        // Parse packed pixel data starting from line 3 (index 2)
        // Each integer contains 2 pixels: low byte = first pixel, high byte = second pixel
        let mut pixels = Vec::with_capacity(expected_pixels);
        let mut pixel_attributes = Vec::with_capacity(expected_pixels);

        // Process packed pixel pairs - start at index 2, not 4!
        let pixel_data = &data[2..];
        let needed_ints = (expected_pixels + 1) / 2; // 200 integers for 400 pixels

        for (i, &value) in pixel_data.iter().enumerate() {
            if pixels.len() >= expected_pixels {
                break;
            }

            // Check if this is the attribute line (last line, typically small value 1-9)
            if i >= needed_ints {
                break;
            }

            // Extract two pixels from packed 16-bit value
            // Low byte = first pixel, high byte = second pixel
            let pixel1 = (value & 0xFF) as i16;
            let pixel2 = ((value >> 8) & 0xFF) as i16;

            // First pixel
            if pixels.len() < expected_pixels {
                if pixel1 == 0 && value == 0 {
                    pixels.push(-1); // Transparent
                } else {
                    pixels.push(pixel1);
                }
                pixel_attributes.push(1); // Default attribute
            }

            // Second pixel
            if pixels.len() < expected_pixels {
                if pixel2 == 0 && value == 0 {
                    pixels.push(-1); // Transparent
                } else {
                    pixels.push(pixel2);
                }
                pixel_attributes.push(1);
            }
        }

        // Pad with transparent if needed
        while pixels.len() < expected_pixels {
            pixels.push(-1);
            pixel_attributes.push(0);
        }

        // Get tile attribute from last data value (after pixel data)
        let attr_index = 2 + needed_ints;
        let primary_attr = if attr_index < data.len() {
            let attr_val = data[attr_index];
            if attr_val >= 1 && attr_val <= 9 {
                attr_val as u8
            } else {
                1
            }
        } else {
            1
        };

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
        // Row-major order: pixel[x, y] = data[y * width + x]
        let index = (y * self.width + x) as usize;
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
