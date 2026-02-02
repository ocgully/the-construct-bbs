//! Realm of Ralnar Asset Converter CLI
//!
//! Batch converts original game assets to modern formats (PNG, JSON).
//! Outputs images at 1x-5x scales using nearest-neighbor interpolation.

use clap::{Parser, Subcommand};
use ralnar_converter::{
    mmi::MmiTile, mmm::{MapTile, MmmMap}, mon::MonSprite, nmf::NmfMap, pic::PicTile,
    scaling::{save_scaled_images, ScaledOutput, SCALE_FACTORS},
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Realm of Ralnar Asset Converter
#[derive(Parser)]
#[command(name = "ralnar_converter")]
#[command(about = "Convert Realm of Ralnar (1996) game assets to modern formats")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert all assets from source directories
    ConvertAll {
        /// Source directory containing original assets
        #[arg(short, long)]
        source: PathBuf,

        /// Output directory for converted assets
        #[arg(short, long, default_value = "assets/converted")]
        output: PathBuf,
    },

    /// Render a map to PNG using tile images
    RenderMap {
        /// Input map JSON file
        input: PathBuf,

        /// Assets directory containing tiles/
        #[arg(short, long)]
        assets: PathBuf,

        /// Output PNG file (or directory for all scales)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Render all scales (1x-5x)
        #[arg(long)]
        all_scales: bool,
    },

    /// Render all maps to PNG
    RenderAllMaps {
        /// Assets directory containing maps/ and tiles/
        #[arg(short, long)]
        assets: PathBuf,

        /// Output directory for rendered maps
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Convert a single PIC file
    Pic {
        /// Input PIC file
        input: PathBuf,

        /// Output PNG file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert a single MMI file
    Mmi {
        /// Input MMI file
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert a single MMM map file
    Mmm {
        /// Input MMM file
        input: PathBuf,

        /// Output JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert a single NMF binary map file
    Nmf {
        /// Input NMF file
        input: PathBuf,

        /// Output JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Convert a single MON monster sprite file
    Mon {
        /// Input MON file
        input: PathBuf,

        /// Output PNG file (or directory for individual frames)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Save individual frames instead of sprite sheet
        #[arg(long)]
        frames: bool,
    },

    /// Shift/move a map by X and Y tiles with wrapping
    ShiftMap {
        /// Input map JSON file
        input: PathBuf,

        /// X offset (positive = right, negative = left)
        #[arg(short = 'x', long, default_value = "0")]
        shift_x: i32,

        /// Y offset (positive = down, negative = up)
        #[arg(short = 'y', long, default_value = "0")]
        shift_y: i32,

        /// Output JSON file (overwrites input if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Expand map borders with a specified tile
    ExpandMap {
        /// Input map JSON file
        input: PathBuf,

        /// Number of tiles to add on each edge
        #[arg(short, long)]
        border: u32,

        /// Tile name to use for border (e.g., "water1", "empty")
        #[arg(short, long)]
        tile: String,

        /// Output JSON file (overwrites input if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Asset manifest for tracking converted files
#[derive(Debug, Serialize, Deserialize)]
struct AssetManifest {
    /// Conversion timestamp
    pub timestamp: String,
    /// Source directory
    pub source_dir: String,
    /// Converted tiles from PIC files
    pub tiles: Vec<TileEntry>,
    /// Converted tiles from MMI files (with metadata)
    pub mmi_tiles: Vec<MmiEntry>,
    /// Converted maps from MMM files
    pub mmm_maps: Vec<MapEntry>,
    /// Converted maps from NMF files
    pub nmf_maps: Vec<MapEntry>,
    /// Converted monster sprites
    pub monsters: Vec<MonsterEntry>,
    /// Conversion statistics
    pub stats: ConversionStats,
}

#[derive(Debug, Serialize, Deserialize)]
struct TileEntry {
    pub name: String,
    pub source: String,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MmiEntry {
    pub name: String,
    pub source: String,
    pub image: String,
    pub metadata: String,
    pub attribute_code: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct MapEntry {
    pub name: String,
    pub source: String,
    pub output: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MonsterEntry {
    pub name: String,
    pub source: String,
    pub output: String,
    pub frames: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConversionStats {
    pub pic_count: usize,
    pub pic_errors: usize,
    pub mmi_count: usize,
    pub mmi_errors: usize,
    pub mmm_count: usize,
    pub mmm_errors: usize,
    pub nmf_count: usize,
    pub nmf_errors: usize,
    pub mon_count: usize,
    pub mon_errors: usize,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ConvertAll { source, output } => {
            if let Err(e) = convert_all(&source, &output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::RenderMap {
            input,
            assets,
            output,
            all_scales,
        } => {
            if let Err(e) = render_map(&input, &assets, output.as_deref(), all_scales) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::RenderAllMaps { assets, output } => {
            if let Err(e) = render_all_maps(&assets, &output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Pic { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension("png"));
            match PicTile::from_file(&input) {
                Ok(tile) => {
                    if let Err(e) = tile.save_png(&output) {
                        eprintln!("Error saving PNG: {}", e);
                        std::process::exit(1);
                    }
                    println!("Converted {} -> {}", input.display(), output.display());
                }
                Err(e) => {
                    eprintln!("Error reading PIC: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Mmi { input, output } => {
            let out_dir = output.unwrap_or_else(|| PathBuf::from("."));
            let stem = input.file_stem().unwrap().to_str().unwrap();
            let png_path = out_dir.join(format!("{}.png", stem));
            let json_path = out_dir.join(format!("{}.json", stem));

            match MmiTile::from_file(&input) {
                Ok(tile) => {
                    if let Err(e) = tile.save_png(&png_path) {
                        eprintln!("Error saving PNG: {}", e);
                        std::process::exit(1);
                    }
                    if let Err(e) = tile.save_metadata(&json_path) {
                        eprintln!("Error saving metadata: {}", e);
                        std::process::exit(1);
                    }
                    println!(
                        "Converted {} -> {}, {}",
                        input.display(),
                        png_path.display(),
                        json_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("Error reading MMI: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Mmm { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension("json"));
            match MmmMap::from_file(&input) {
                Ok(map) => {
                    if let Err(e) = map.save_json(&output) {
                        eprintln!("Error saving JSON: {}", e);
                        std::process::exit(1);
                    }
                    println!("Converted {} -> {}", input.display(), output.display());
                }
                Err(e) => {
                    eprintln!("Error reading MMM: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Nmf { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension("json"));
            match NmfMap::from_file(&input) {
                Ok(map) => {
                    if let Err(e) = map.save_json(&output) {
                        eprintln!("Error saving JSON: {}", e);
                        std::process::exit(1);
                    }
                    println!("Converted {} -> {}", input.display(), output.display());
                }
                Err(e) => {
                    eprintln!("Error reading NMF: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Mon {
            input,
            output,
            frames,
        } => {
            let output = output.unwrap_or_else(|| input.with_extension("png"));
            match MonSprite::from_file(&input) {
                Ok(sprite) => {
                    if frames {
                        match sprite.save_frames(&output) {
                            Ok(files) => {
                                println!("Converted {} -> {} frames", input.display(), files.len());
                            }
                            Err(e) => {
                                eprintln!("Error saving frames: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        if let Err(e) = sprite.save_png(&output) {
                            eprintln!("Error saving PNG: {}", e);
                            std::process::exit(1);
                        }
                        println!("Converted {} -> {}", input.display(), output.display());
                    }
                }
                Err(e) => {
                    eprintln!("Error reading MON: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ShiftMap {
            input,
            shift_x,
            shift_y,
            output,
        } => {
            let output = output.unwrap_or_else(|| input.clone());
            if let Err(e) = shift_map(&input, shift_x, shift_y, &output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::ExpandMap {
            input,
            border,
            tile,
            output,
        } => {
            let output = output.unwrap_or_else(|| input.clone());
            if let Err(e) = expand_map(&input, border, &tile, &output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn convert_all(source: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Converting assets from {} to {}", source.display(), output.display());
    println!("Output scales: 1x, 2x, 3x, 4x, 5x (nearest-neighbor)");

    // Create output directories with scale subdirs
    let tiles_out = output.join("tiles");
    let sprites_out = output.join("sprites");
    let maps_out = output.join("maps");
    let monsters_out = output.join("monsters");
    let metadata_out = output.join("metadata");

    // Create scale directories
    for scale in SCALE_FACTORS {
        fs::create_dir_all(tiles_out.join(format!("{}x", scale)))?;
        fs::create_dir_all(sprites_out.join(format!("{}x", scale)))?;
        fs::create_dir_all(monsters_out.join(format!("{}x", scale)))?;
    }
    fs::create_dir_all(&maps_out)?;
    fs::create_dir_all(&metadata_out)?;

    let mut manifest = AssetManifest {
        timestamp: chrono_lite_now(),
        source_dir: source.display().to_string(),
        tiles: Vec::new(),
        mmi_tiles: Vec::new(),
        mmm_maps: Vec::new(),
        nmf_maps: Vec::new(),
        monsters: Vec::new(),
        stats: ConversionStats::default(),
    };

    // Convert PIC files (sprites)
    let pics_dir = source.join("pics");
    if pics_dir.exists() {
        println!("\nConverting PIC files (sprites) from {}...", pics_dir.display());
        convert_pics_scaled(&pics_dir, &sprites_out, &mut manifest)?;
    }

    // Convert MMI files (tiles with attributes)
    let mmi_dir = source.join("mmi");
    if mmi_dir.exists() {
        println!("\nConverting MMI files (tiles) from {}...", mmi_dir.display());
        convert_mmis_scaled(&mmi_dir, &tiles_out, &metadata_out, &mut manifest)?;
    }

    // Convert map files
    let maps_dir = source.join("maps");
    if maps_dir.exists() {
        println!("\nConverting map files from {}...", maps_dir.display());
        convert_maps(&maps_dir, &maps_out, &mut manifest)?;
    }

    // Convert monster files
    let monster_dir = source.join("monster");
    if monster_dir.exists() {
        println!("\nConverting monster files from {}...", monster_dir.display());
        convert_monsters_scaled(&monster_dir, &monsters_out, &mut manifest)?;
    }

    // Save manifest
    let manifest_path = output.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    println!("\n=== Conversion Complete ===");
    println!("PIC tiles: {} converted, {} errors", manifest.stats.pic_count, manifest.stats.pic_errors);
    println!("MMI tiles: {} converted, {} errors", manifest.stats.mmi_count, manifest.stats.mmi_errors);
    println!("MMM maps:  {} converted, {} errors", manifest.stats.mmm_count, manifest.stats.mmm_errors);
    println!("NMF maps:  {} converted, {} errors", manifest.stats.nmf_count, manifest.stats.nmf_errors);
    println!("Monsters:  {} converted, {} errors", manifest.stats.mon_count, manifest.stats.mon_errors);
    println!("\nManifest saved to: {}", manifest_path.display());

    Ok(())
}

fn convert_pics_scaled(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = fs::read_dir(source)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x.to_ascii_uppercase()) == Some("PIC".into()))
        .collect();

    let count = AtomicUsize::new(0);
    let errors = AtomicUsize::new(0);

    let results: Vec<_> = files
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let stem = path.file_stem()?.to_str()?;
            let filename = format!("{}.png", stem.to_lowercase());

            match PicTile::from_file(&path) {
                Ok(tile) => {
                    let img = tile.to_image();
                    if save_scaled_images(&img, output, &filename, None).is_ok() {
                        count.fetch_add(1, Ordering::Relaxed);
                        print!(".");
                        Some(TileEntry {
                            name: stem.to_lowercase(),
                            source: path.display().to_string(),
                            output: filename,
                        })
                    } else {
                        errors.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    errors.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect();

    manifest.tiles.extend(results);
    manifest.stats.pic_count = count.load(Ordering::Relaxed);
    manifest.stats.pic_errors = errors.load(Ordering::Relaxed);
    println!();
    Ok(())
}

fn convert_mmis_scaled(
    source: &Path,
    tiles_output: &Path,
    metadata_output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = fs::read_dir(source)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().map(|x| x.to_ascii_uppercase()) == Some("MMI".into())
                && path.file_name().map(|n| n.to_str().unwrap_or("").to_uppercase()) != Some("MMIFILES.TXT".to_string())
        })
        .collect();

    let count = AtomicUsize::new(0);
    let errors = AtomicUsize::new(0);

    let results: Vec<_> = files
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let stem = path.file_stem()?.to_str()?;
            let filename = format!("{}.png", stem.to_lowercase());

            match MmiTile::from_file(&path) {
                Ok(tile) => {
                    let img = tile.to_image();
                    if save_scaled_images(&img, tiles_output, &filename, None).is_ok() {
                        // Save metadata
                        let json_path = metadata_output.join(format!("{}.json", stem.to_lowercase()));
                        let _ = tile.save_metadata(&json_path);

                        let metadata = tile.get_metadata();
                        count.fetch_add(1, Ordering::Relaxed);
                        print!(".");
                        Some(MmiEntry {
                            name: stem.to_lowercase(),
                            source: path.display().to_string(),
                            image: filename,
                            metadata: json_path.display().to_string(),
                            attribute_code: metadata.attribute_code,
                        })
                    } else {
                        errors.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    errors.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect();

    manifest.mmi_tiles.extend(results);
    manifest.stats.mmi_count = count.load(Ordering::Relaxed);
    manifest.stats.mmi_errors = errors.load(Ordering::Relaxed);
    println!();
    Ok(())
}

fn convert_maps(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    // Load tile registry from MMIFILES.TXT
    let mmi_dir = source.parent().unwrap_or(source).join("mmi");
    let registry_path = mmi_dir.join("MMIFILES.TXT");
    let tile_registry = if registry_path.exists() {
        ralnar_converter::load_tile_registry(&registry_path).unwrap_or_default()
    } else {
        Vec::new()
    };

    // PASS 1: Analyze all maps (MMM and NMF) to find most common attribute per tile
    println!("  Pass 1: Analyzing tile attributes across all maps...");
    let mut tile_attr_counts: HashMap<String, HashMap<u8, usize>> = HashMap::new();

    // Helper to count tile attributes from a map's tiles
    let count_tiles = |tiles: &[MapTile], tile_attr_counts: &mut HashMap<String, HashMap<u8, usize>>| {
        for tile in tiles {
            // Get tile name from registry (0-based index after NMF icon list remapping)
            let name = if (tile.tile_index as usize) < tile_registry.len() {
                tile_registry[tile.tile_index as usize].clone()
            } else {
                format!("tile_{}", tile.tile_index)
            };

            // Count this attribute usage
            tile_attr_counts
                .entry(name)
                .or_default()
                .entry(tile.attribute)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    };

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().map(|e| e.to_ascii_uppercase());

        if ext == Some("MMM".into()) {
            if let Ok(map) = MmmMap::from_file(&path) {
                count_tiles(&map.tiles, &mut tile_attr_counts);
            }
        } else if ext == Some("NMF".into()) {
            if let Ok(map) = NmfMap::from_file(&path) {
                count_tiles(&map.tiles, &mut tile_attr_counts);
            }
        }
    }

    // Compute most common attribute for each tile
    let mut tile_attributes: HashMap<String, u8> = HashMap::new();
    for (name, attr_counts) in &tile_attr_counts {
        if let Some((&most_common_attr, _)) = attr_counts.iter().max_by_key(|(_, count)| *count) {
            tile_attributes.insert(name.clone(), most_common_attr);
        }
    }

    // Update metadata files with computed defaults
    let metadata_dir = output.parent().unwrap_or(output).join("metadata");
    for (name, attr) in &tile_attributes {
        let meta_path = metadata_dir.join(format!("{}.json", name));
        if meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(mut meta) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = meta.as_object_mut() {
                        obj.insert("attribute_code".to_string(), serde_json::Value::Number((*attr).into()));
                        // Update the attributes object based on code
                        let attrs = serde_json::json!({
                            "land": *attr == 1 || *attr == 3 || *attr == 4 || *attr == 6 || *attr == 7 || *attr == 9,
                            "water": *attr == 2,
                            "slow": *attr == 3 || *attr == 4,
                            "damaging": *attr == 4,
                            "impassable": *attr == 5,
                            "town": *attr == 6,
                            "cave": *attr == 7,
                            "dock": *attr == 8,
                            "any_pass": *attr == 9
                        });
                        obj.insert("attributes".to_string(), attrs);
                        if let Ok(updated) = serde_json::to_string_pretty(&meta) {
                            let _ = fs::write(&meta_path, updated);
                        }
                    }
                }
            }
        }
    }

    println!("  Computed defaults for {} tiles from map usage", tile_attributes.len());

    // PASS 2: Generate maps using computed defaults
    println!("  Pass 2: Generating maps...");

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().map(|e| e.to_ascii_uppercase());

        if ext == Some("MMM".into()) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let out_path = output.join(format!("{}.json", stem.to_lowercase()));

            match MmmMap::from_file(&path) {
                Ok(map) => {
                    // Use spec-compliant format with per-map tileset
                    map.save_map_json(&out_path, &tile_registry, &tile_attributes)?;
                    manifest.mmm_maps.push(MapEntry {
                        name: stem.to_lowercase(),
                        source: path.display().to_string(),
                        output: out_path.display().to_string(),
                        width: map.width,
                        height: map.height,
                    });
                    manifest.stats.mmm_count += 1;
                    print!(".");
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    manifest.stats.mmm_errors += 1;
                }
            }
        } else if ext == Some("NMF".into()) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let out_path = output.join(format!("{}_nmf.json", stem.to_lowercase()));

            match NmfMap::from_file(&path) {
                Ok(map) => {
                    // Use spec-compliant format with per-map tileset and sparse overrides
                    map.save_map_json(&out_path, &tile_registry, &tile_attributes)?;
                    manifest.nmf_maps.push(MapEntry {
                        name: stem.to_lowercase(),
                        source: path.display().to_string(),
                        output: out_path.display().to_string(),
                        width: map.width,
                        height: map.height,
                    });
                    manifest.stats.nmf_count += 1;
                    print!(".");
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    manifest.stats.nmf_errors += 1;
                }
            }
        }
    }
    println!();
    Ok(())
}

fn convert_monsters_scaled(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = fs::read_dir(source)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x.to_ascii_uppercase()) == Some("MON".into()))
        .collect();

    let count = AtomicUsize::new(0);
    let errors = AtomicUsize::new(0);

    let results: Vec<_> = files
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let stem = path.file_stem()?.to_str()?;
            let filename = format!("{}.png", stem.to_lowercase());

            match MonSprite::from_file(&path) {
                Ok(sprite) => {
                    // Use sprite sheet for multi-frame, single image otherwise
                    let img = if sprite.frames.len() == 1 {
                        sprite.frames[0].to_image()
                    } else {
                        sprite.to_sprite_sheet()
                    };

                    if save_scaled_images(&img, output, &filename, None).is_ok() {
                        let metadata = sprite.get_metadata();
                        count.fetch_add(1, Ordering::Relaxed);
                        print!(".");
                        Some(MonsterEntry {
                            name: stem.to_lowercase(),
                            source: path.display().to_string(),
                            output: filename,
                            frames: metadata.frame_count,
                            width: metadata.width,
                            height: metadata.height,
                        })
                    } else {
                        errors.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    errors.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect();

    manifest.monsters.extend(results);
    manifest.stats.mon_count = count.load(Ordering::Relaxed);
    manifest.stats.mon_errors = errors.load(Ordering::Relaxed);
    println!();
    Ok(())
}

/// Simple timestamp without external chrono dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Render a single map to PNG
fn render_map(
    map_path: &Path,
    assets_dir: &Path,
    output: Option<&Path>,
    all_scales: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{RgbaImage, imageops};
    use std::collections::HashMap;

    // Load map JSON
    let map_content = fs::read_to_string(map_path)?;
    let map_json: serde_json::Value = serde_json::from_str(&map_content)?;

    let name = map_json["name"].as_str().unwrap_or("unknown");
    let width = map_json["dimensions"]["width"].as_u64().unwrap_or(0) as u32;
    let height = map_json["dimensions"]["height"].as_u64().unwrap_or(0) as u32;
    let tileset = map_json["tileset"].as_array().ok_or("Missing tileset")?;
    let tiles = map_json["tiles"].as_array().ok_or("Missing tiles")?;

    println!("Rendering map: {} ({}x{} tiles)", name, width, height);

    // Determine scales to render
    let scales: Vec<u32> = if all_scales {
        vec![1, 2, 3, 4, 5]
    } else {
        vec![1]
    };

    for scale in &scales {
        let tile_size = 20 * scale;
        let img_width = width * tile_size;
        let img_height = height * tile_size;

        println!("  Rendering {}x scale ({}x{} pixels)...", scale, img_width, img_height);

        // Load tile images for this scale
        let tiles_dir = assets_dir.join("tiles").join(format!("{}x", scale));
        let mut tile_cache: HashMap<String, RgbaImage> = HashMap::new();

        // Create output image
        let mut output_img = RgbaImage::new(img_width, img_height);

        // Render each tile
        for (y, row) in tiles.iter().enumerate() {
            if let Some(row_arr) = row.as_array() {
                for (x, tile_idx) in row_arr.iter().enumerate() {
                    let idx = tile_idx.as_u64().unwrap_or(0) as usize;

                    // Get tile name from tileset
                    let tile_name = if idx < tileset.len() {
                        tileset[idx].as_str().unwrap_or("unknown")
                    } else {
                        "unknown"
                    };

                    // Load tile image (with caching)
                    let tile_img = tile_cache.entry(tile_name.to_string()).or_insert_with(|| {
                        // Special case: "empty" tiles render as black
                        if tile_name == "empty" {
                            let mut black = RgbaImage::new(tile_size, tile_size);
                            for pixel in black.pixels_mut() {
                                *pixel = image::Rgba([0, 0, 0, 255]);
                            }
                            return black;
                        }

                        let tile_path = tiles_dir.join(format!("{}.png", tile_name));
                        if tile_path.exists() {
                            match image::open(&tile_path) {
                                Ok(img) => img.to_rgba8(),
                                Err(_) => {
                                    // Create magenta placeholder for missing tiles
                                    let mut placeholder = RgbaImage::new(tile_size, tile_size);
                                    for pixel in placeholder.pixels_mut() {
                                        *pixel = image::Rgba([255, 0, 255, 255]);
                                    }
                                    placeholder
                                }
                            }
                        } else {
                            // Create magenta placeholder for missing tiles
                            let mut placeholder = RgbaImage::new(tile_size, tile_size);
                            for pixel in placeholder.pixels_mut() {
                                *pixel = image::Rgba([255, 0, 255, 255]);
                            }
                            eprintln!("    Warning: Missing tile '{}' at ({}, {})", tile_name, x, y);
                            placeholder
                        }
                    });

                    // Copy tile to output image
                    let dest_x = (x as u32) * tile_size;
                    let dest_y = (y as u32) * tile_size;
                    imageops::overlay(&mut output_img, tile_img, dest_x as i64, dest_y as i64);
                }
            }
        }

        // Save output
        let stem = map_path.file_stem().unwrap().to_str().unwrap();
        let output_path = if let Some(out) = output {
            if all_scales {
                // Create directory structure for multiple scales
                let out_dir = out.parent().unwrap_or(Path::new("."));
                fs::create_dir_all(out_dir)?;
                out_dir.join(format!("{}_{}x.png", stem, scale))
            } else {
                out.to_path_buf()
            }
        } else {
            PathBuf::from(format!("{}_{}x.png", stem, scale))
        };

        output_img.save(&output_path)?;
        println!("    Saved: {}", output_path.display());
    }

    Ok(())
}

/// Render all maps to PNG
fn render_all_maps(
    assets_dir: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let maps_dir = assets_dir.join("maps");

    if !maps_dir.exists() {
        return Err(format!("Maps directory not found: {}", maps_dir.display()).into());
    }

    // Create output directories for each scale
    for scale in SCALE_FACTORS {
        fs::create_dir_all(output_dir.join(format!("{}x", scale)))?;
    }

    let mut count = 0;
    let mut errors = 0;

    for entry in fs::read_dir(&maps_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let stem = path.file_stem().unwrap().to_str().unwrap();

            match render_map_to_scales(&path, assets_dir, output_dir) {
                Ok(_) => {
                    count += 1;
                    println!("  Rendered: {}", stem);
                }
                Err(e) => {
                    eprintln!("  Error rendering {}: {}", stem, e);
                    errors += 1;
                }
            }
        }
    }

    println!("\n=== Rendering Complete ===");
    println!("Maps rendered: {}", count);
    println!("Errors: {}", errors);

    Ok(())
}

/// Render a map to all scales
fn render_map_to_scales(
    map_path: &Path,
    assets_dir: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{RgbaImage, imageops};
    use std::collections::HashMap;

    // Load map JSON
    let map_content = fs::read_to_string(map_path)?;
    let map_json: serde_json::Value = serde_json::from_str(&map_content)?;

    let name = map_json["name"].as_str().unwrap_or("unknown");
    let width = map_json["dimensions"]["width"].as_u64().unwrap_or(0) as u32;
    let height = map_json["dimensions"]["height"].as_u64().unwrap_or(0) as u32;
    let tileset = map_json["tileset"].as_array().ok_or("Missing tileset")?;
    let tiles = map_json["tiles"].as_array().ok_or("Missing tiles")?;

    let stem = map_path.file_stem().unwrap().to_str().unwrap();

    for scale in SCALE_FACTORS {
        let tile_size = 20 * scale;
        let img_width = width * tile_size;
        let img_height = height * tile_size;

        // Load tile images for this scale
        let tiles_dir = assets_dir.join("tiles").join(format!("{}x", scale));
        let mut tile_cache: HashMap<String, RgbaImage> = HashMap::new();

        // Create output image
        let mut output_img = RgbaImage::new(img_width, img_height);

        // Render each tile
        for (y, row) in tiles.iter().enumerate() {
            if let Some(row_arr) = row.as_array() {
                for (x, tile_idx) in row_arr.iter().enumerate() {
                    let idx = tile_idx.as_u64().unwrap_or(0) as usize;

                    // Get tile name from tileset
                    let tile_name = if idx < tileset.len() {
                        tileset[idx].as_str().unwrap_or("unknown")
                    } else {
                        "unknown"
                    };

                    // Load tile image (with caching)
                    let tile_img = tile_cache.entry(tile_name.to_string()).or_insert_with(|| {
                        // Special case: "empty" tiles render as black
                        if tile_name == "empty" {
                            let mut black = RgbaImage::new(tile_size, tile_size);
                            for pixel in black.pixels_mut() {
                                *pixel = image::Rgba([0, 0, 0, 255]);
                            }
                            return black;
                        }

                        let tile_path = tiles_dir.join(format!("{}.png", tile_name));
                        if tile_path.exists() {
                            match image::open(&tile_path) {
                                Ok(img) => img.to_rgba8(),
                                Err(_) => create_placeholder(tile_size),
                            }
                        } else {
                            create_placeholder(tile_size)
                        }
                    });

                    // Copy tile to output image
                    let dest_x = (x as u32) * tile_size;
                    let dest_y = (y as u32) * tile_size;
                    imageops::overlay(&mut output_img, tile_img, dest_x as i64, dest_y as i64);
                }
            }
        }

        // Save output
        let output_path = output_dir.join(format!("{}x", scale)).join(format!("{}.png", stem));
        output_img.save(&output_path)?;
    }

    Ok(())
}

/// Create a magenta placeholder tile for missing images
fn create_placeholder(size: u32) -> image::RgbaImage {
    let mut placeholder = image::RgbaImage::new(size, size);
    for pixel in placeholder.pixels_mut() {
        *pixel = image::Rgba([255, 0, 255, 255]);
    }
    placeholder
}

/// Shift/move a map by X and Y tiles with wrapping
fn shift_map(
    input: &Path,
    shift_x: i32,
    shift_y: i32,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load map JSON
    let content = fs::read_to_string(input)?;
    let mut map: serde_json::Value = serde_json::from_str(&content)?;

    let width = map["dimensions"]["width"].as_u64().ok_or("Missing width")? as i32;
    let height = map["dimensions"]["height"].as_u64().ok_or("Missing height")? as i32;

    let tiles = map["tiles"].as_array().ok_or("Missing tiles")?;

    // Create new tiles array with shifted positions
    let mut new_tiles: Vec<Vec<serde_json::Value>> = vec![vec![serde_json::json!(0); width as usize]; height as usize];

    for (old_y, row) in tiles.iter().enumerate() {
        if let Some(row_arr) = row.as_array() {
            for (old_x, tile_val) in row_arr.iter().enumerate() {
                // Calculate new position with wrapping
                let new_x = ((old_x as i32 + shift_x) % width + width) % width;
                let new_y = ((old_y as i32 + shift_y) % height + height) % height;
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
                    let new_x = ((x as i32 + shift_x) % width + width) % width;
                    let new_y = ((y as i32 + shift_y) % height + height) % height;
                    entry["x"] = serde_json::json!(new_x);
                    entry["y"] = serde_json::json!(new_y);
                }
            }
        }
    }

    // Also shift spawn point if present
    if let Some(spawn) = map.get_mut("spawn") {
        if let (Some(x), Some(y)) = (spawn["x"].as_i64(), spawn["y"].as_i64()) {
            let new_x = ((x as i32 + shift_x) % width + width) % width;
            let new_y = ((y as i32 + shift_y) % height + height) % height;
            spawn["x"] = serde_json::json!(new_x);
            spawn["y"] = serde_json::json!(new_y);
        }
    }

    // Save updated map
    let json = serde_json::to_string_pretty(&map)?;
    fs::write(output, json)?;

    println!(
        "Shifted map by ({}, {}) with wrapping: {} -> {}",
        shift_x, shift_y,
        input.display(),
        output.display()
    );

    Ok(())
}

/// Expand map borders with a specified tile
fn expand_map(
    input: &Path,
    border: u32,
    tile_name: &str,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load map JSON
    let content = fs::read_to_string(input)?;
    let mut map: serde_json::Value = serde_json::from_str(&content)?;

    let old_width = map["dimensions"]["width"].as_u64().ok_or("Missing width")? as usize;
    let old_height = map["dimensions"]["height"].as_u64().ok_or("Missing height")? as usize;

    let new_width = old_width + (border as usize * 2);
    let new_height = old_height + (border as usize * 2);

    // Clone tiles first to avoid borrow conflict
    let tiles = map["tiles"].as_array().ok_or("Missing tiles")?.clone();

    // Find or add tile to tileset
    let border_tile_idx = {
        let tileset = map["tileset"].as_array_mut().ok_or("Missing tileset")?;
        tileset
            .iter()
            .position(|t| t.as_str() == Some(tile_name))
            .unwrap_or_else(|| {
                tileset.push(serde_json::json!(tile_name));
                tileset.len() - 1
            })
    };

    // Create new tiles array with border
    let mut new_tiles: Vec<Vec<serde_json::Value>> = Vec::with_capacity(new_height);

    // Add top border rows
    for _ in 0..border {
        new_tiles.push(vec![serde_json::json!(border_tile_idx); new_width]);
    }

    // Add original rows with left/right border
    for row in tiles.iter() {
        let mut new_row: Vec<serde_json::Value> = Vec::with_capacity(new_width);

        // Left border
        for _ in 0..border {
            new_row.push(serde_json::json!(border_tile_idx));
        }

        // Original tiles
        if let Some(row_arr) = row.as_array() {
            for tile_val in row_arr {
                new_row.push(tile_val.clone());
            }
        }

        // Right border
        for _ in 0..border {
            new_row.push(serde_json::json!(border_tile_idx));
        }

        new_tiles.push(new_row);
    }

    // Add bottom border rows
    for _ in 0..border {
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
                    entry["x"] = serde_json::json!(x + border as i64);
                    entry["y"] = serde_json::json!(y + border as i64);
                }
            }
        }
    }

    // Shift spawn point by border offset
    if let Some(spawn) = map.get_mut("spawn") {
        if let (Some(x), Some(y)) = (spawn["x"].as_i64(), spawn["y"].as_i64()) {
            spawn["x"] = serde_json::json!(x + border as i64);
            spawn["y"] = serde_json::json!(y + border as i64);
        }
    }

    // Save updated map
    let json = serde_json::to_string_pretty(&map)?;
    fs::write(output, json)?;

    println!(
        "Expanded map with {} tiles of '{}' border: {}x{} -> {}x{}: {} -> {}",
        border, tile_name,
        old_width, old_height,
        new_width, new_height,
        input.display(),
        output.display()
    );

    Ok(())
}
