//! Shift/move a map by X and Y tiles with wrapping

use clap::Parser;
use serde_json::Value;
use std::fs;

#[derive(Parser)]
#[command(name = "shift-map")]
#[command(about = "Shift/move a map by X and Y tiles with wrapping")]
struct Args {
    /// Input map JSON file
    input: String,

    /// X offset (positive = right, negative = left)
    #[arg(short = 'x', long, default_value = "0", allow_hyphen_values = true)]
    shift_x: i32,

    /// Y offset (positive = down, negative = up)
    #[arg(short = 'y', long, default_value = "0", allow_hyphen_values = true)]
    shift_y: i32,

    /// Output JSON file (overwrites input if not specified)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = shift_map(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn shift_map(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&args.input)?;
    let mut map: Value = serde_json::from_str(&content)?;

    let width = map["dimensions"]["width"].as_u64().ok_or("Missing width")? as i32;
    let height = map["dimensions"]["height"].as_u64().ok_or("Missing height")? as i32;

    let tiles = map["tiles"].as_array().ok_or("Missing tiles")?;

    // Create new tiles array with shifted positions
    let mut new_tiles: Vec<Vec<Value>> = vec![vec![serde_json::json!(0); width as usize]; height as usize];

    for (old_y, row) in tiles.iter().enumerate() {
        if let Some(row_arr) = row.as_array() {
            for (old_x, tile_val) in row_arr.iter().enumerate() {
                // Calculate new position with wrapping
                let new_x = ((old_x as i32 + args.shift_x) % width + width) % width;
                let new_y = ((old_y as i32 + args.shift_y) % height + height) % height;
                new_tiles[new_y as usize][new_x as usize] = tile_val.clone();
            }
        }
    }

    // Update tiles in map
    map["tiles"] = serde_json::json!(new_tiles);

    // Also shift attribute_overrides if present
    if let Some(overrides) = map.get_mut("attribute_overrides") {
        if let Some(arr) = overrides.as_array_mut() {
            for entry in arr.iter_mut() {
                if let (Some(x), Some(y)) = (entry["x"].as_i64(), entry["y"].as_i64()) {
                    let new_x = ((x as i32 + args.shift_x) % width + width) % width;
                    let new_y = ((y as i32 + args.shift_y) % height + height) % height;
                    entry["x"] = serde_json::json!(new_x);
                    entry["y"] = serde_json::json!(new_y);
                }
            }
        }
    }

    // Also shift spawn point if present
    if let Some(spawn) = map.get_mut("spawn") {
        if let (Some(x), Some(y)) = (spawn["x"].as_i64(), spawn["y"].as_i64()) {
            let new_x = ((x as i32 + args.shift_x) % width + width) % width;
            let new_y = ((y as i32 + args.shift_y) % height + height) % height;
            spawn["x"] = serde_json::json!(new_x);
            spawn["y"] = serde_json::json!(new_y);
        }
    }

    // Save updated map
    let output_path = args.output.as_ref().unwrap_or(&args.input);
    let json = serde_json::to_string_pretty(&map)?;
    fs::write(output_path, json)?;

    println!(
        "Shifted map by ({}, {}) with wrapping: {} -> {}",
        args.shift_x, args.shift_y, args.input, output_path
    );

    Ok(())
}
