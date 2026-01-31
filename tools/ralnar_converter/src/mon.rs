//! MON (Monster) sprite file converter
//!
//! MON files contain monster sprites from the original Realm of Ralnar game.
//! Format (based on analysis of MON_EDIT.BAS and file structure):
//!
//! Header (8 bytes):
//! - Bytes 0-1: Version (LE16, always 0x0001)
//! - Bytes 2-3: Frame count (LE16, typically 1, 2, or 4)
//! - Bytes 4-5: Width * 8 (LE16, divide by 8 to get actual width)
//! - Bytes 6-7: Height (LE16)
//!
//! Pixel data follows:
//! - Raw palette indices
//! - 0xFF (255) = transparent
//! - Width * Height * FrameCount total pixels

use crate::palette::palette_to_rgba;
use image::{ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Header size in bytes
const HEADER_SIZE: usize = 8;

/// Transparent pixel value
const TRANSPARENT: u8 = 0xFF;

/// Errors that can occur when processing MON files
#[derive(Error, Debug)]
pub enum MonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File too short: expected at least {expected} bytes, got {actual}")]
    FileTooShort { expected: usize, actual: usize },

    #[error("Invalid version: expected 1, got {0}")]
    InvalidVersion(u16),

    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u16, height: u16 },

    #[error("Invalid frame count: {0}")]
    InvalidFrameCount(u16),

    #[error("Insufficient pixel data: expected {expected}, got {actual}")]
    InsufficientPixelData { expected: usize, actual: usize },

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Read a little-endian u16 from a byte slice
fn read_le16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

/// Metadata for a MON sprite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonMetadata {
    pub name: String,
    pub version: u16,
    pub frame_count: u16,
    pub width: u16,
    pub height: u16,
    /// Total pixel count per frame
    pub pixels_per_frame: usize,
}

/// A single animation frame
#[derive(Debug, Clone)]
pub struct MonFrame {
    /// Pixel data as palette indices (255 = transparent)
    pub pixels: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl MonFrame {
    /// Get pixel at (x, y), returns None for transparent
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        let pixel = self.pixels.get(idx).copied()?;
        if pixel == TRANSPARENT {
            None
        } else {
            Some(pixel)
        }
    }

    /// Convert to RGBA image
    pub fn to_image(&self) -> RgbaImage {
        let mut img: RgbaImage =
            ImageBuffer::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let pixel = self.pixels.get(idx).copied().unwrap_or(TRANSPARENT);

                let color = if pixel == TRANSPARENT {
                    Rgba([0, 0, 0, 0])
                } else {
                    let (r, g, b, a) = palette_to_rgba(pixel);
                    Rgba([r, g, b, a])
                };
                img.put_pixel(x as u32, y as u32, color);
            }
        }

        img
    }
}

/// A parsed MON sprite with animation frames
#[derive(Debug, Clone)]
pub struct MonSprite {
    /// Sprite name (from filename)
    pub name: String,
    /// File version (usually 1)
    pub version: u16,
    /// Animation frames
    pub frames: Vec<MonFrame>,
    /// Width of each frame
    pub width: u16,
    /// Height of each frame
    pub height: u16,
}

impl MonSprite {
    /// Parse a MON file from the given path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MonError> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let data = fs::read(path)?;
        Self::parse_binary(&data, name)
    }

    /// Parse MON data from binary content
    fn parse_binary(data: &[u8], name: String) -> Result<Self, MonError> {
        if data.len() < HEADER_SIZE {
            return Err(MonError::FileTooShort {
                expected: HEADER_SIZE,
                actual: data.len(),
            });
        }

        // Parse header
        let version = read_le16(data, 0);
        let frame_count = read_le16(data, 2);
        let width_raw = read_le16(data, 4);
        let height = read_le16(data, 6);

        // Width is stored as width * 8, divide to get actual
        let width = width_raw / 8;

        // Validate header
        if version != 1 {
            // Be lenient - some files might have different versions
            // Just log but continue
        }

        if width == 0 || height == 0 || width > 1000 || height > 1000 {
            return Err(MonError::InvalidDimensions { width, height });
        }

        if frame_count == 0 || frame_count > 100 {
            return Err(MonError::InvalidFrameCount(frame_count));
        }

        let pixels_per_frame = (width as usize) * (height as usize);
        let total_pixels = pixels_per_frame * (frame_count as usize);
        let expected_size = HEADER_SIZE + total_pixels;

        if data.len() < expected_size {
            // Try with actual width instead of divided
            let alt_width = width_raw;
            let alt_pixels_per_frame = (alt_width as usize) * (height as usize);
            let alt_expected = HEADER_SIZE + alt_pixels_per_frame * (frame_count as usize);

            if data.len() >= alt_expected {
                // Use the raw width
                return Self::parse_with_dimensions(data, name, version, frame_count, alt_width, height);
            }

            return Err(MonError::InsufficientPixelData {
                expected: total_pixels,
                actual: data.len() - HEADER_SIZE,
            });
        }

        Self::parse_with_dimensions(data, name, version, frame_count, width, height)
    }

    fn parse_with_dimensions(
        data: &[u8],
        name: String,
        version: u16,
        frame_count: u16,
        width: u16,
        height: u16,
    ) -> Result<Self, MonError> {
        let pixels_per_frame = (width as usize) * (height as usize);
        let mut frames = Vec::with_capacity(frame_count as usize);
        let mut offset = HEADER_SIZE;

        for _ in 0..frame_count {
            let end = offset + pixels_per_frame;
            let pixels = if end <= data.len() {
                data[offset..end].to_vec()
            } else {
                // Pad with transparent if not enough data
                let mut pixels = data[offset..].to_vec();
                pixels.resize(pixels_per_frame, TRANSPARENT);
                pixels
            };

            frames.push(MonFrame {
                pixels,
                width,
                height,
            });

            offset = end;
        }

        Ok(MonSprite {
            name,
            version,
            frames,
            width,
            height,
        })
    }

    /// Parse from byte slice (for testing)
    pub fn from_bytes(data: &[u8], name: &str) -> Result<Self, MonError> {
        Self::parse_binary(data, name.to_string())
    }

    /// Get metadata for JSON export
    pub fn get_metadata(&self) -> MonMetadata {
        MonMetadata {
            name: self.name.clone(),
            version: self.version,
            frame_count: self.frames.len() as u16,
            width: self.width,
            height: self.height,
            pixels_per_frame: (self.width as usize) * (self.height as usize),
        }
    }

    /// Convert all frames to a single sprite sheet image
    /// Frames are arranged horizontally
    pub fn to_sprite_sheet(&self) -> RgbaImage {
        let total_width = (self.width as u32) * (self.frames.len() as u32);
        let height = self.height as u32;

        let mut sheet: RgbaImage = ImageBuffer::new(total_width, height);

        for (frame_idx, frame) in self.frames.iter().enumerate() {
            let frame_img = frame.to_image();
            let x_offset = (frame_idx as u32) * (self.width as u32);

            for y in 0..height {
                for x in 0..self.width as u32 {
                    let pixel = frame_img.get_pixel(x, y);
                    sheet.put_pixel(x + x_offset, y, *pixel);
                }
            }
        }

        sheet
    }

    /// Save as PNG file(s)
    /// If multiple frames, saves as sprite sheet
    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> Result<(), MonError> {
        if self.frames.len() == 1 {
            self.frames[0].to_image().save(path)?;
        } else {
            self.to_sprite_sheet().save(path)?;
        }
        Ok(())
    }

    /// Save individual frames as separate PNGs
    pub fn save_frames<P: AsRef<Path>>(&self, base_path: P) -> Result<Vec<String>, MonError> {
        let base = base_path.as_ref();
        let stem = base
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("frame");
        let ext = base
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");
        let parent = base.parent().unwrap_or(Path::new("."));

        let mut paths = Vec::new();

        for (i, frame) in self.frames.iter().enumerate() {
            let filename = if self.frames.len() == 1 {
                format!("{}.{}", stem, ext)
            } else {
                format!("{}_{}.{}", stem, i, ext)
            };
            let path = parent.join(&filename);
            frame.to_image().save(&path)?;
            paths.push(filename);
        }

        Ok(paths)
    }

    /// Save metadata as JSON
    pub fn save_metadata<P: AsRef<Path>>(&self, path: P) -> Result<(), MonError> {
        let metadata = self.get_metadata();
        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mon_data(
        version: u16,
        frame_count: u16,
        width: u16,
        height: u16,
        pixel_value: u8,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(&version.to_le_bytes());
        data.extend_from_slice(&frame_count.to_le_bytes());
        data.extend_from_slice(&(width * 8).to_le_bytes()); // Width is stored * 8
        data.extend_from_slice(&height.to_le_bytes());

        // Pixel data
        let pixels_per_frame = (width as usize) * (height as usize);
        for _ in 0..(frame_count as usize) {
            for _ in 0..pixels_per_frame {
                data.push(pixel_value);
            }
        }

        data
    }

    #[test]
    fn test_parse_single_frame() {
        let data = create_test_mon_data(1, 1, 8, 8, 5);
        let sprite = MonSprite::from_bytes(&data, "test").unwrap();

        assert_eq!(sprite.name, "test");
        assert_eq!(sprite.version, 1);
        assert_eq!(sprite.frames.len(), 1);
        assert_eq!(sprite.width, 8);
        assert_eq!(sprite.height, 8);
    }

    #[test]
    fn test_parse_multiple_frames() {
        let data = create_test_mon_data(1, 4, 16, 16, 2);
        let sprite = MonSprite::from_bytes(&data, "test").unwrap();

        assert_eq!(sprite.frames.len(), 4);
    }

    #[test]
    fn test_frame_get_pixel() {
        let data = create_test_mon_data(1, 1, 4, 4, 10);
        let sprite = MonSprite::from_bytes(&data, "test").unwrap();

        assert_eq!(sprite.frames[0].get_pixel(0, 0), Some(10));
        assert_eq!(sprite.frames[0].get_pixel(3, 3), Some(10));
        assert_eq!(sprite.frames[0].get_pixel(4, 0), None); // Out of bounds
    }

    #[test]
    fn test_transparent_pixel() {
        let mut data = create_test_mon_data(1, 1, 4, 4, 0);
        // Set first pixel to transparent
        data[HEADER_SIZE] = TRANSPARENT;

        let sprite = MonSprite::from_bytes(&data, "test").unwrap();
        assert_eq!(sprite.frames[0].get_pixel(0, 0), None);
    }

    #[test]
    fn test_to_image() {
        let data = create_test_mon_data(1, 1, 4, 4, 2); // Green
        let sprite = MonSprite::from_bytes(&data, "test").unwrap();
        let img = sprite.frames[0].to_image();

        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);

        let pixel = img.get_pixel(0, 0);
        assert_eq!(pixel.0[3], 255); // Opaque
    }

    #[test]
    fn test_sprite_sheet() {
        let data = create_test_mon_data(1, 2, 8, 8, 5);
        let sprite = MonSprite::from_bytes(&data, "test").unwrap();
        let sheet = sprite.to_sprite_sheet();

        assert_eq!(sheet.width(), 16); // 2 frames * 8 width
        assert_eq!(sheet.height(), 8);
    }

    #[test]
    fn test_metadata() {
        let data = create_test_mon_data(1, 2, 16, 24, 0);
        let sprite = MonSprite::from_bytes(&data, "monster").unwrap();
        let metadata = sprite.get_metadata();

        assert_eq!(metadata.name, "monster");
        assert_eq!(metadata.frame_count, 2);
        assert_eq!(metadata.width, 16);
        assert_eq!(metadata.height, 24);
    }

    #[test]
    fn test_file_too_short() {
        let data = vec![0, 0, 0, 0]; // Only 4 bytes
        let result = MonSprite::from_bytes(&data, "test");
        assert!(matches!(result, Err(MonError::FileTooShort { .. })));
    }
}
