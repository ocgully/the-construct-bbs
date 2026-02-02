//! Expand map borders with a specified tile

use clap::Parser;
use serde_json::Value;
use std::fs;

#[derive(Parser)]
#[command(name = "expand-map")]
#[command(about = "Expand map borders with a specified tile")]
struct Args {
    /// Input map JSON file
    input: String,

    /// Number of tiles to add on each edge
    #[arg(short, long)]
    border: u32,

    /// Tile name to use for border (e.g., "water1", "empty")
    #[arg(short, long)]
    tile: String,

    /// Output JSON file (overwrites input if not specified)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = expand_map(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn expand_map(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&args.input)?;
    let mut map: Value = serde_json::from_str(&content)?;

    let old_width = map["dimensions"]["width"].as_u64().ok_or("Missing width")? as usize;
    let old_height = map["dimensions"]["height"].as_u64().ok_or("Missing height")? as usize;

    let new_width = old_width + (args.border as usize * 2);
    let new_height = old_height + (args.border as usize * 2);

    // Clone tiles first to avoid borrow conflict
    let tiles = map["tiles"].as_array().ok_or("Missing tiles")?.clone();

    // Find or add tile to tileset
    let border_tile_idx = {
        let tileset = map["tileset"].as_array_mut().ok_or("Missing tileset")?;
        tileset
            .iter()
            .position(|t| t.as_str() == Some(&args.tile))
            .unwrap_or_else(|| {
                tileset.push(serde_json::json!(args.tile));
                tileset.len() - 1
            })
    };

    // Create new tiles array with border
    let mut new_tiles: Vec<Vec<Value>> = Vec::with_capacity(new_height);

    // Add top border rows
    for _ in 0..args.border {
        new_tiles.push(vec![serde_json::json!(border_tile_idx); new_width]);
    }

    // Add original rows with left/right border
    for row in tiles.iter() {
        let mut new_row: Vec<Value> = Vec::with_capacity(new_width);

        // Left border
        for _ in 0..args.border {
            new_row.push(serde_json::json!(border_tile_idx));
        }

        // Original tiles
        if let Some(row_arr) = row.as_array() {
            for tile_val in row_arr {
                new_row.push(tile_val.clone());
            }
        }

        // Right border
        for _ in 0..args.border {
            new_row.push(serde_json::json!(border_tile_idx));
        }

        new_tiles.push(new_row);
    }

    // Add bottom border rows
    for _ in 0..args.border {
        new_tiles.push(vec![serde_json::json!(border_tile_idx); new_width]);
    }

    // Update map dimensions
    map["dimensions"]["width"] = serde_json::json!(new_width);
    map["dimensions"]["height"] = serde_json::json!(new_height);
    map["tiles"] = serde_json::json!(new_tiles);

    // Shift attribute_overrides by border offset
    if let Some(overrides) = map.get_mut("attribute_overrides") {
        if let Some(arr) = overrides.as_array_mut() {
            for entry in arr.iter_mut() {
                if let (Some(x), Some(y)) = (entry["x"].as_i64(), entry["y"].as_i64()) {
                    entry["x"] = serde_json::json!(x + args.border as i64);
                    entry["y"] = serde_json::json!(y + args.border as i64);
                }
            }
        }
    }

    // Shift spawn point by border offset
    if let Some(spawn) = map.get_mut("spawn") {
        if let (Some(x), Some(y)) = (spawn["x"].as_i64(), spawn["y"].as_i64()) {
            spawn["x"] = serde_json::json!(x + args.border as i64);
            spawn["y"] = serde_json::json!(y + args.border as i64);
        }
    }

    // Save updated map
    let output_path = args.output.as_ref().unwrap_or(&args.input);
    let json = serde_json::to_string_pretty(&map)?;
    fs::write(output_path, json)?;

    println!(
        "Expanded map with {} tiles of '{}' border: {}x{} -> {}x{}: {} -> {}",
        args.border, args.tile, old_width, old_height, new_width, new_height,
        args.input, output_path
    );

    Ok(())
}
