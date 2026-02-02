//! Multi-scale image output for pixel art preservation
//!
//! Supports outputting images at 1x-5x scale using nearest-neighbor interpolation.

use image::{imageops::FilterType, RgbaImage};
use std::fs;
use std::path::Path;

/// Available scale factors for output
pub const SCALE_FACTORS: &[u32] = &[1, 2, 3, 4, 5];

/// Scale an image using nearest-neighbor interpolation
pub fn scale_image(img: &RgbaImage, scale: u32) -> RgbaImage {
    if scale == 1 {
        return img.clone();
    }

    let new_width = img.width() * scale;
    let new_height = img.height() * scale;

    image::imageops::resize(img, new_width, new_height, FilterType::Nearest)
}

/// Save an image at multiple scales
///
/// Creates subdirectories for each scale (1x/, 2x/, etc.) and saves the image
/// with the same filename in each.
///
/// # Arguments
/// * `img` - The source image at 1x scale
/// * `base_dir` - The base output directory (e.g., "assets/tiles")
/// * `filename` - The output filename (e.g., "tree.png")
/// * `scales` - Which scales to output (default: 1-5)
pub fn save_scaled_images(
    img: &RgbaImage,
    base_dir: &Path,
    filename: &str,
    scales: Option<&[u32]>,
) -> Result<Vec<String>, image::ImageError> {
    let scales = scales.unwrap_or(SCALE_FACTORS);
    let mut saved_paths = Vec::new();

    for &scale in scales {
        let scale_dir = base_dir.join(format!("{}x", scale));
        fs::create_dir_all(&scale_dir).ok();

        let output_path = scale_dir.join(filename);
        let scaled = scale_image(img, scale);
        scaled.save(&output_path)?;
        saved_paths.push(output_path.display().to_string());
    }

    Ok(saved_paths)
}

/// Batch save helper for converting multiple images
pub struct ScaledOutput {
    pub base_dir: std::path::PathBuf,
    pub scales: Vec<u32>,
}

impl ScaledOutput {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            scales: SCALE_FACTORS.to_vec(),
        }
    }

    pub fn with_scales(mut self, scales: &[u32]) -> Self {
        self.scales = scales.to_vec();
        self
    }

    pub fn save(&self, img: &RgbaImage, filename: &str) -> Result<Vec<String>, image::ImageError> {
        save_scaled_images(img, &self.base_dir, filename, Some(&self.scales))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn create_test_image(width: u32, height: u32) -> RgbaImage {
        let mut img = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.put_pixel(x, y, Rgba([x as u8, y as u8, 128, 255]));
            }
        }
        img
    }

    #[test]
    fn test_scale_1x() {
        let img = create_test_image(20, 20);
        let scaled = scale_image(&img, 1);
        assert_eq!(scaled.width(), 20);
        assert_eq!(scaled.height(), 20);
    }

    #[test]
    fn test_scale_2x() {
        let img = create_test_image(20, 20);
        let scaled = scale_image(&img, 2);
        assert_eq!(scaled.width(), 40);
        assert_eq!(scaled.height(), 40);
    }

    #[test]
    fn test_scale_5x() {
        let img = create_test_image(20, 20);
        let scaled = scale_image(&img, 5);
        assert_eq!(scaled.width(), 100);
        assert_eq!(scaled.height(), 100);
    }

    #[test]
    fn test_nearest_neighbor_preserves_pixels() {
        // Create a 2x2 image with distinct colors
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Red
        img.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // Green
        img.put_pixel(0, 1, Rgba([0, 0, 255, 255])); // Blue
        img.put_pixel(1, 1, Rgba([255, 255, 0, 255])); // Yellow

        let scaled = scale_image(&img, 2);

        // Check that each original pixel became a 2x2 block
        assert_eq!(scaled.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
        assert_eq!(scaled.get_pixel(1, 0), &Rgba([255, 0, 0, 255]));
        assert_eq!(scaled.get_pixel(0, 1), &Rgba([255, 0, 0, 255]));
        assert_eq!(scaled.get_pixel(1, 1), &Rgba([255, 0, 0, 255]));

        assert_eq!(scaled.get_pixel(2, 0), &Rgba([0, 255, 0, 255]));
        assert_eq!(scaled.get_pixel(3, 0), &Rgba([0, 255, 0, 255]));
    }
}
