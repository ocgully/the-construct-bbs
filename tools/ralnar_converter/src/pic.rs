//! PIC file format converter
//!
//! PIC files are text-based tile sprites from the original Realm of Ralnar game.
//! Format:
//! - 400 lines for a 20x20 pixel tile
//! - One integer per line
//! - -1 = transparent pixel
//! - 0-255 = VGA palette index
//! - Reading order is column-major (x first, then y)

use crate::palette::palette_to_rgba;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

/// Tile dimensions for PIC files
pub const TILE_WIDTH: u32 = 20;
pub const TILE_HEIGHT: u32 = 20;
pub const TILE_PIXELS: usize = (TILE_WIDTH * TILE_HEIGHT) as usize;

/// Errors that can occur when processing PIC files
#[derive(Error, Debug)]
pub enum PicError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error on line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Invalid pixel count: expected {expected}, got {actual}")]
    InvalidPixelCount { expected: usize, actual: usize },

    #[error("Invalid palette index: {0}")]
    InvalidPaletteIndex(i32),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}

/// A parsed PIC tile
#[derive(Debug, Clone)]
pub struct PicTile {
    /// Pixel data in column-major order (as stored in file)
    /// Values: -1 for transparent, 0-255 for palette index
    pub pixels: Vec<i16>,
    /// Source filename (without extension)
    pub name: String,
}

impl PicTile {
    /// Parse a PIC file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PicError> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut pixels = Vec::with_capacity(TILE_PIXELS);

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Parse the integer value
            let value: i32 = trimmed.parse().map_err(|_| PicError::Parse {
                line: line_num + 1,
                message: format!("invalid integer: '{}'", trimmed),
            })?;

            // Validate range
            if value < -1 || value > 255 {
                return Err(PicError::InvalidPaletteIndex(value));
            }

            pixels.push(value as i16);

            // Stop after we have enough pixels
            if pixels.len() >= TILE_PIXELS {
                break;
            }
        }

        if pixels.len() != TILE_PIXELS {
            return Err(PicError::InvalidPixelCount {
                expected: TILE_PIXELS,
                actual: pixels.len(),
            });
        }

        Ok(PicTile { pixels, name })
    }

    /// Parse a PIC file from a string (for testing)
    pub fn from_str(content: &str, name: &str) -> Result<Self, PicError> {
        let mut pixels = Vec::with_capacity(TILE_PIXELS);

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let value: i32 = trimmed.parse().map_err(|_| PicError::Parse {
                line: line_num + 1,
                message: format!("invalid integer: '{}'", trimmed),
            })?;

            if value < -1 || value > 255 {
                return Err(PicError::InvalidPaletteIndex(value));
            }

            pixels.push(value as i16);

            if pixels.len() >= TILE_PIXELS {
                break;
            }
        }

        if pixels.len() != TILE_PIXELS {
            return Err(PicError::InvalidPixelCount {
                expected: TILE_PIXELS,
                actual: pixels.len(),
            });
        }

        Ok(PicTile {
            pixels,
            name: name.to_string(),
        })
    }

    /// Get the pixel value at (x, y) coordinates
    /// Returns -1 for transparent, 0-255 for palette index
    pub fn get_pixel(&self, x: u32, y: u32) -> i16 {
        // Data is stored in column-major order:
        // File reads sequentially as: (0,0), (0,1), (0,2)...(0,19), (1,0), (1,1)...
        // So pixel[x, y] = data[x * height + y]
        let index = (x * TILE_HEIGHT + y) as usize;
        self.pixels[index]
    }

    /// Convert to RGBA image
    pub fn to_image(&self) -> RgbaImage {
        let mut img: RgbaImage = ImageBuffer::new(TILE_WIDTH, TILE_HEIGHT);

        for x in 0..TILE_WIDTH {
            for y in 0..TILE_HEIGHT {
                let pixel_value = self.get_pixel(x, y);
                let color = if pixel_value == -1 {
                    // Transparent
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

    /// Save as PNG file
    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), PicError> {
        let img = self.to_image();
        img.save(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pic_content() -> String {
        // Create a simple 20x20 test pattern
        // All transparent except for a few colored pixels
        // Column-major: index = x * height + y
        let mut lines = Vec::new();
        for i in 0..TILE_PIXELS {
            if i == 0 {
                lines.push("2".to_string()); // Green at (0, 0): index 0*20+0 = 0
            } else if i == 21 {
                lines.push("4".to_string()); // Red at (1, 1): index 1*20+1 = 21
            } else {
                lines.push("-1".to_string()); // Transparent
            }
        }
        lines.join("\n")
    }

    #[test]
    fn test_parse_pic_content() {
        let content = create_test_pic_content();
        let tile = PicTile::from_str(&content, "test").unwrap();

        assert_eq!(tile.name, "test");
        assert_eq!(tile.pixels.len(), TILE_PIXELS);
    }

    #[test]
    fn test_get_pixel() {
        let content = create_test_pic_content();
        let tile = PicTile::from_str(&content, "test").unwrap();

        // Check specific pixels
        assert_eq!(tile.get_pixel(0, 0), 2); // Green
        assert_eq!(tile.get_pixel(1, 1), 4); // Red
        assert_eq!(tile.get_pixel(5, 5), -1); // Transparent
    }

    #[test]
    fn test_to_image() {
        let content = create_test_pic_content();
        let tile = PicTile::from_str(&content, "test").unwrap();
        let img = tile.to_image();

        assert_eq!(img.width(), TILE_WIDTH);
        assert_eq!(img.height(), TILE_HEIGHT);

        // Check transparent pixel
        let transparent_pixel = img.get_pixel(5, 5);
        assert_eq!(transparent_pixel.0[3], 0); // Alpha = 0

        // Check colored pixel (0, 0) should be green
        let green_pixel = img.get_pixel(0, 0);
        assert_eq!(green_pixel.0[3], 255); // Alpha = 255 (opaque)
        assert!(green_pixel.0[1] > green_pixel.0[0]); // G > R
        assert!(green_pixel.0[1] > green_pixel.0[2]); // G > B
    }

    #[test]
    fn test_invalid_pixel_count() {
        let content = "0\n1\n2"; // Only 3 pixels
        let result = PicTile::from_str(content, "test");
        assert!(matches!(result, Err(PicError::InvalidPixelCount { .. })));
    }

    #[test]
    fn test_invalid_palette_index() {
        let mut lines: Vec<String> = (0..TILE_PIXELS - 1).map(|_| "-1".to_string()).collect();
        lines.push("300".to_string()); // Invalid index
        let content = lines.join("\n");
        let result = PicTile::from_str(&content, "test");
        assert!(matches!(result, Err(PicError::InvalidPaletteIndex(300))));
    }

    #[test]
    fn test_parse_error() {
        let mut lines: Vec<String> = (0..TILE_PIXELS - 1).map(|_| "-1".to_string()).collect();
        lines.push("not_a_number".to_string());
        let content = lines.join("\n");
        let result = PicTile::from_str(&content, "test");
        assert!(matches!(result, Err(PicError::Parse { .. })));
    }
}
