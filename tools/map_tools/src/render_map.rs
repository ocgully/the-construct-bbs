//! Render JSON maps to PNG at multiple scales

use clap::Parser;
use image::{imageops, RgbaImage};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const TILE_SIZE: u32 = 20;
const SCALES: [u32; 5] = [1, 2, 3, 4, 5];

#[derive(Parser)]
#[command(name = "render-map")]
#[command(about = "Render JSON maps to PNG at multiple scales")]
struct Args {
    /// Input map JSON file or directory containing maps
    input: String,

    /// Assets directory containing tiles/
    #[arg(short, long)]
    assets: String,

    /// Output directory for renders
    #[arg(short, long)]
    output: String,

    /// Render all maps in directory
    #[arg(long)]
    all: bool,
}

#[derive(Deserialize)]
struct MapJson {
    name: String,
    dimensions: Dimensions,
    tileset: Vec<String>,
    tiles: Vec<Vec<u16>>,
}

#[derive(Deserialize)]
struct Dimensions {
    width: u32,
    height: u32,
}

fn main() {
    let args = Args::parse();

    if args.all || Path::new(&args.input).is_dir() {
        render_all_maps(&args.input, &args.assets, &args.output);
    } else {
        render_single_map(&args.input, &args.assets, &args.output);
    }
}

fn render_all_maps(input_dir: &str, assets_dir: &str, output_dir: &str) {
    let maps_dir = if Path::new(input_dir).join("maps").exists() {
        Path::new(input_dir).join("maps")
    } else {
        Path::new(input_dir).to_path_buf()
    };

    let entries: Vec<_> = fs::read_dir(&maps_dir)
        .expect("Failed to read maps directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .collect();

    for entry in &entries {
        let path = entry.path();
        if let Err(e) = render_single_map(path.to_str().unwrap(), assets_dir, output_dir) {
            eprintln!("Error rendering {}: {}", path.display(), e);
        } else {
            println!("  Rendered: {}", path.file_stem().unwrap().to_str().unwrap());
        }
    }

    println!("\n=== Rendering Complete ===");
    println!("Maps rendered: {}", entries.len());
}

fn render_single_map(input: &str, assets_dir: &str, output_dir: &str) -> Result<(), String> {
    let content = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let map: MapJson = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let width = map.dimensions.width;
    let height = map.dimensions.height;
    let map_name = map.name.to_lowercase();

    println!("Rendering map: {} ({}x{} tiles)", map.name, width, height);

    // Create output directories
    for scale in SCALES {
        let scale_dir = Path::new(output_dir).join(format!("{}x", scale));
        fs::create_dir_all(&scale_dir).map_err(|e| e.to_string())?;
    }

    // Pre-load tile cache
    let mut tile_cache: HashMap<(String, u32), RgbaImage> = HashMap::new();

    for scale in SCALES {
        let tile_size = TILE_SIZE * scale;
        let img_width = width * tile_size;
        let img_height = height * tile_size;

        println!("  Rendering {}x scale ({}x{} pixels)...", scale, img_width, img_height);

        let mut output_img = RgbaImage::new(img_width, img_height);

        for (y, row) in map.tiles.iter().enumerate() {
            for (x, &tile_idx) in row.iter().enumerate() {
                let tile_name = if (tile_idx as usize) < map.tileset.len() {
                    &map.tileset[tile_idx as usize]
                } else {
                    continue;
                };

                let tile_img = tile_cache
                    .entry((tile_name.clone(), scale))
                    .or_insert_with(|| {
                        load_tile(tile_name, assets_dir, scale, tile_size)
                    });

                let dest_x = (x as u32) * tile_size;
                let dest_y = (y as u32) * tile_size;
                imageops::overlay(&mut output_img, tile_img, dest_x as i64, dest_y as i64);
            }
        }

        let scale_dir = Path::new(output_dir).join(format!("{}x", scale));
        let output_path = scale_dir.join(format!("{}.png", map_name));
        output_img.save(&output_path).map_err(|e| e.to_string())?;
        println!("    Saved: {}", output_path.display());
    }

    Ok(())
}

fn load_tile(name: &str, assets_dir: &str, scale: u32, tile_size: u32) -> RgbaImage {
    // Handle empty tiles
    if name == "empty" {
        let mut black = RgbaImage::new(tile_size, tile_size);
        for pixel in black.pixels_mut() {
            *pixel = image::Rgba([0, 0, 0, 255]);
        }
        return black;
    }

    // Try to load from tiles directory
    let tile_path = Path::new(assets_dir)
        .join("tiles")
        .join(format!("{}x", scale))
        .join(format!("{}.png", name));

    if let Ok(img) = image::open(&tile_path) {
        return img.to_rgba8();
    }

    // Create magenta placeholder for missing tiles
    let mut placeholder = RgbaImage::new(tile_size, tile_size);
    for pixel in placeholder.pixels_mut() {
        *pixel = image::Rgba([255, 0, 255, 255]);
    }
    placeholder
}
