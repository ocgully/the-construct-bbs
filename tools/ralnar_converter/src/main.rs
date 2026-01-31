//! Realm of Ralnar Asset Converter CLI
//!
//! Batch converts original game assets to modern formats (PNG, JSON).

use clap::{Parser, Subcommand};
use ralnar_converter::{
    mmi::MmiTile, mmm::MmmMap, mon::MonSprite, nmf::NmfMap, pic::PicTile,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    }
}

fn convert_all(source: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Converting assets from {} to {}", source.display(), output.display());

    // Create output directories
    let pics_out = output.join("tiles");
    let mmi_out = output.join("mmi");
    let maps_out = output.join("maps");
    let monsters_out = output.join("monsters");

    fs::create_dir_all(&pics_out)?;
    fs::create_dir_all(&mmi_out)?;
    fs::create_dir_all(&maps_out)?;
    fs::create_dir_all(&monsters_out)?;

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

    // Convert PIC files
    let pics_dir = source.join("pics");
    if pics_dir.exists() {
        println!("\nConverting PIC files from {}...", pics_dir.display());
        convert_pics(&pics_dir, &pics_out, &mut manifest)?;
    }

    // Convert MMI files
    let mmi_dir = source.join("mmi");
    if mmi_dir.exists() {
        println!("\nConverting MMI files from {}...", mmi_dir.display());
        convert_mmis(&mmi_dir, &mmi_out, &mut manifest)?;
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
        convert_monsters(&monster_dir, &monsters_out, &mut manifest)?;
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

fn convert_pics(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e.to_ascii_uppercase()) == Some("PIC".into()) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let out_path = output.join(format!("{}.png", stem.to_lowercase()));

            match PicTile::from_file(&path) {
                Ok(tile) => {
                    tile.save_png(&out_path)?;
                    manifest.tiles.push(TileEntry {
                        name: stem.to_lowercase(),
                        source: path.display().to_string(),
                        output: out_path.display().to_string(),
                    });
                    manifest.stats.pic_count += 1;
                    print!(".");
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    manifest.stats.pic_errors += 1;
                }
            }
        }
    }
    println!();
    Ok(())
}

fn convert_mmis(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e.to_ascii_uppercase()) == Some("MMI".into()) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let png_path = output.join(format!("{}.png", stem.to_lowercase()));
            let json_path = output.join(format!("{}.json", stem.to_lowercase()));

            match MmiTile::from_file(&path) {
                Ok(tile) => {
                    tile.save_png(&png_path)?;
                    tile.save_metadata(&json_path)?;
                    let metadata = tile.get_metadata();
                    manifest.mmi_tiles.push(MmiEntry {
                        name: stem.to_lowercase(),
                        source: path.display().to_string(),
                        image: png_path.display().to_string(),
                        metadata: json_path.display().to_string(),
                        attribute_code: metadata.attribute_code,
                    });
                    manifest.stats.mmi_count += 1;
                    print!(".");
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    manifest.stats.mmi_errors += 1;
                }
            }
        }
    }
    println!();
    Ok(())
}

fn convert_maps(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().map(|e| e.to_ascii_uppercase());

        if ext == Some("MMM".into()) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let out_path = output.join(format!("{}.json", stem.to_lowercase()));

            match MmmMap::from_file(&path) {
                Ok(map) => {
                    map.save_json(&out_path)?;
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
                    map.save_json(&out_path)?;
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

fn convert_monsters(
    source: &Path,
    output: &Path,
    manifest: &mut AssetManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e.to_ascii_uppercase()) == Some("MON".into()) {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let png_path = output.join(format!("{}.png", stem.to_lowercase()));
            let json_path = output.join(format!("{}.json", stem.to_lowercase()));

            match MonSprite::from_file(&path) {
                Ok(sprite) => {
                    sprite.save_png(&png_path)?;
                    sprite.save_metadata(&json_path)?;
                    let metadata = sprite.get_metadata();
                    manifest.monsters.push(MonsterEntry {
                        name: stem.to_lowercase(),
                        source: path.display().to_string(),
                        output: png_path.display().to_string(),
                        frames: metadata.frame_count,
                        width: metadata.width,
                        height: metadata.height,
                    });
                    manifest.stats.mon_count += 1;
                    print!(".");
                }
                Err(e) => {
                    eprintln!("\nError converting {}: {}", path.display(), e);
                    manifest.stats.mon_errors += 1;
                }
            }
        }
    }
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
