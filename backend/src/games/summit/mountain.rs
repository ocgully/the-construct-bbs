//! Mountain generation for Summit
//!
//! Procedurally generates the daily mountain from a seed.
//! The same seed produces the same mountain layout worldwide.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::data::{BiomeType, HazardType, ItemType, HAZARDS};

// ============================================================================
// TILE TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Air,
    Rock,
    Climbable,
    Ledge,
    Campfire,
    Luggage,
    SecretArea,
    // Biome-specific
    Sand,
    Palm,
    Vine,
    Waterfall,
    Snow,
    Ice,
    Lava,
    AshCloud,
}

impl TileType {
    pub fn is_solid(&self) -> bool {
        matches!(self, TileType::Rock | TileType::Sand | TileType::Palm |
                 TileType::Snow | TileType::Ice | TileType::Lava)
    }

    pub fn is_climbable(&self) -> bool {
        matches!(self, TileType::Climbable | TileType::Ledge | TileType::Vine |
                 TileType::Rock | TileType::Snow | TileType::Ice)
    }

    pub fn is_rest_point(&self) -> bool {
        matches!(self, TileType::Ledge | TileType::Campfire)
    }

    pub fn character(&self, biome: BiomeType) -> char {
        match self {
            TileType::Air => ' ',
            TileType::Rock => match biome {
                BiomeType::Beach => '#',
                BiomeType::Jungle => '%',
                BiomeType::Alpine => '^',
                BiomeType::Volcanic => '&',
            },
            TileType::Climbable => '.',
            TileType::Ledge => '=',
            TileType::Campfire => '*',
            TileType::Luggage => 'L',
            TileType::SecretArea => '?',
            TileType::Sand => ',',
            TileType::Palm => 'T',
            TileType::Vine => '|',
            TileType::Waterfall => '~',
            TileType::Snow => '.',
            TileType::Ice => '-',
            TileType::Lava => '~',
            TileType::AshCloud => ':',
        }
    }
}

// ============================================================================
// MOUNTAIN TILE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainTile {
    pub tile_type: TileType,
    pub hazard: Option<HazardType>,
    pub item_spawn: Option<ItemType>,
    pub food_spawn: Option<u32>,  // Food ID
}

impl MountainTile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            hazard: None,
            item_spawn: None,
            food_spawn: None,
        }
    }

    pub fn with_hazard(mut self, hazard: HazardType) -> Self {
        self.hazard = Some(hazard);
        self
    }

    pub fn with_item(mut self, item: ItemType) -> Self {
        self.item_spawn = Some(item);
        self
    }

    pub fn with_food(mut self, food_id: u32) -> Self {
        self.food_spawn = Some(food_id);
        self
    }
}

// ============================================================================
// MOUNTAIN
// ============================================================================

pub const MOUNTAIN_WIDTH: usize = 80;
pub const MOUNTAIN_HEIGHT: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mountain {
    pub seed: u64,
    pub date: String,
    tiles: Vec<Vec<MountainTile>>,  // [y][x]
    pub campfire_positions: Vec<(i32, i32)>,
}

impl Mountain {
    /// Generate a new mountain from a seed
    pub fn generate(seed: u64, date: String) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        // Initialize with air
        let mut tiles: Vec<Vec<MountainTile>> = (0..MOUNTAIN_HEIGHT)
            .map(|_| {
                (0..MOUNTAIN_WIDTH)
                    .map(|_| MountainTile::new(TileType::Air))
                    .collect()
            })
            .collect();

        let mut campfire_positions = Vec::new();

        // Generate terrain for each biome
        for y in 0..MOUNTAIN_HEIGHT {
            let biome = Self::biome_at(y);
            Self::generate_row(&mut tiles[y], y, biome, &mut rng);
        }

        // Place campfires between biomes
        let campfire_heights = [24, 49, 74];
        for &y in &campfire_heights {
            let x = rng.gen_range(30..50);
            tiles[y][x] = MountainTile::new(TileType::Campfire);
            // Create ledge around campfire
            if x > 0 {
                tiles[y][x - 1] = MountainTile::new(TileType::Ledge);
            }
            if x < MOUNTAIN_WIDTH - 1 {
                tiles[y][x + 1] = MountainTile::new(TileType::Ledge);
            }
            campfire_positions.push((x as i32, y as i32));
        }

        // Place hazards
        Self::place_hazards(&mut tiles, &mut rng);

        // Place items and food
        Self::place_loot(&mut tiles, &mut rng);

        // Ensure summit is reachable
        tiles[MOUNTAIN_HEIGHT - 1][40] = MountainTile::new(TileType::Ledge);

        Mountain {
            seed,
            date,
            tiles,
            campfire_positions,
        }
    }

    fn biome_at(y: usize) -> BiomeType {
        match y {
            0..=24 => BiomeType::Beach,
            25..=49 => BiomeType::Jungle,
            50..=74 => BiomeType::Alpine,
            _ => BiomeType::Volcanic,
        }
    }

    fn generate_row(row: &mut Vec<MountainTile>, y: usize, biome: BiomeType, rng: &mut ChaCha8Rng) {
        // Mountain shape: narrower at top
        let width_factor = 1.0 - (y as f64 / MOUNTAIN_HEIGHT as f64) * 0.5;
        let mountain_width = (MOUNTAIN_WIDTH as f64 * width_factor) as usize;
        let start_x = (MOUNTAIN_WIDTH - mountain_width) / 2;
        let end_x = start_x + mountain_width;

        for x in 0..MOUNTAIN_WIDTH {
            if x < start_x || x >= end_x {
                row[x] = MountainTile::new(TileType::Air);
                continue;
            }

            // Edge is rock
            if x == start_x || x == end_x - 1 {
                row[x] = MountainTile::new(match biome {
                    BiomeType::Beach => TileType::Sand,
                    BiomeType::Jungle => TileType::Rock,
                    BiomeType::Alpine => TileType::Snow,
                    BiomeType::Volcanic => TileType::Rock,
                });
                continue;
            }

            // Interior terrain
            let roll: u32 = rng.gen_range(0..100);
            let tile_type = match biome {
                BiomeType::Beach => {
                    if roll < 60 { TileType::Air }
                    else if roll < 80 { TileType::Climbable }
                    else if roll < 90 { TileType::Ledge }
                    else if roll < 95 { TileType::Sand }
                    else { TileType::Palm }
                }
                BiomeType::Jungle => {
                    if roll < 50 { TileType::Air }
                    else if roll < 70 { TileType::Climbable }
                    else if roll < 80 { TileType::Vine }
                    else if roll < 90 { TileType::Ledge }
                    else if roll < 95 { TileType::Rock }
                    else { TileType::Waterfall }
                }
                BiomeType::Alpine => {
                    if roll < 45 { TileType::Air }
                    else if roll < 65 { TileType::Climbable }
                    else if roll < 75 { TileType::Snow }
                    else if roll < 85 { TileType::Ice }
                    else { TileType::Ledge }
                }
                BiomeType::Volcanic => {
                    if roll < 40 { TileType::Air }
                    else if roll < 60 { TileType::Climbable }
                    else if roll < 70 { TileType::Rock }
                    else if roll < 80 { TileType::Ledge }
                    else if roll < 90 { TileType::AshCloud }
                    else { TileType::Lava }
                }
            };

            row[x] = MountainTile::new(tile_type);
        }
    }

    fn place_hazards(tiles: &mut Vec<Vec<MountainTile>>, rng: &mut ChaCha8Rng) {
        for y in 0..MOUNTAIN_HEIGHT {
            let biome = Self::biome_at(y);
            let hazard_chance = match biome {
                BiomeType::Beach => 5,
                BiomeType::Jungle => 10,
                BiomeType::Alpine => 15,
                BiomeType::Volcanic => 20,
            };

            for x in 0..MOUNTAIN_WIDTH {
                if tiles[y][x].tile_type == TileType::Campfire {
                    continue; // No hazards at campfires
                }

                if rng.gen_range(0..100) < hazard_chance {
                    // Pick a hazard for this biome
                    let biome_hazards: Vec<_> = HAZARDS.iter()
                        .filter(|h| h.biome == biome)
                        .collect();

                    if !biome_hazards.is_empty() {
                        let hazard = biome_hazards[rng.gen_range(0..biome_hazards.len())];
                        tiles[y][x].hazard = Some(hazard.hazard_type);
                    }
                }
            }
        }
    }

    fn place_loot(tiles: &mut Vec<Vec<MountainTile>>, rng: &mut ChaCha8Rng) {
        use super::data::{CLIMBING_ITEMS, FOODS};

        for y in 0..MOUNTAIN_HEIGHT {
            for x in 0..MOUNTAIN_WIDTH {
                // Luggage (contains items/food)
                if tiles[y][x].tile_type == TileType::Air || tiles[y][x].tile_type == TileType::Climbable {
                    if rng.gen_range(0..100) < 3 {
                        tiles[y][x] = MountainTile::new(TileType::Luggage);

                        // Random item or food
                        if rng.gen_bool(0.5) {
                            // Item spawn
                            let items: Vec<_> = CLIMBING_ITEMS.iter()
                                .filter(|i| rng.gen_range(0..100) < i.rarity)
                                .collect();
                            if !items.is_empty() {
                                let item = items[rng.gen_range(0..items.len())];
                                tiles[y][x].item_spawn = Some(item.item_type);
                            }
                        } else {
                            // Food spawn
                            let biome = Self::biome_at(y);
                            let foods: Vec<_> = FOODS.iter()
                                .filter(|f| {
                                    // Filter by rarity and biome
                                    if f.rarity == 0 { return false; } // Crafted only
                                    if rng.gen_range(0..100) >= f.rarity { return false; }
                                    // Special foods only in specific biomes
                                    if f.id == 27 && biome != BiomeType::Volcanic { return false; }
                                    if f.id == 30 && y < 95 { return false; } // Summit only
                                    true
                                })
                                .collect();
                            if !foods.is_empty() {
                                let food = foods[rng.gen_range(0..foods.len())];
                                tiles[y][x].food_spawn = Some(food.id);
                            }
                        }
                    }
                }
            }
        }

        // Place secret area (one per mountain)
        let secret_y = rng.gen_range(30..70);
        let secret_x = rng.gen_range(10..70);
        tiles[secret_y][secret_x] = MountainTile::new(TileType::SecretArea);
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<&MountainTile> {
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        if y >= MOUNTAIN_HEIGHT || x >= MOUNTAIN_WIDTH {
            return None;
        }
        Some(&self.tiles[y][x])
    }

    pub fn get_tile_mut(&mut self, x: i32, y: i32) -> Option<&mut MountainTile> {
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        if y >= MOUNTAIN_HEIGHT || x >= MOUNTAIN_WIDTH {
            return None;
        }
        Some(&mut self.tiles[y][x])
    }

    pub fn biome_at_height(&self, y: i32) -> BiomeType {
        Self::biome_at(y as usize)
    }

    /// Get a viewport of the mountain centered on a position
    pub fn get_viewport(&self, center_x: i32, center_y: i32, width: usize, height: usize) -> Vec<Vec<MountainTile>> {
        let half_w = width as i32 / 2;
        let half_h = height as i32 / 2;

        let start_x = center_x - half_w;
        let start_y = center_y - half_h;

        let default_tile = MountainTile::new(TileType::Air);

        (0..height)
            .map(|dy| {
                (0..width)
                    .map(|dx| {
                        let x = start_x + dx as i32;
                        let y = start_y + dy as i32;
                        self.get_tile(x, y).cloned().unwrap_or_else(|| default_tile.clone())
                    })
                    .collect()
            })
            .collect()
    }

    /// Check if a position is valid for movement
    pub fn can_move_to(&self, x: i32, y: i32) -> bool {
        if let Some(tile) = self.get_tile(x, y) {
            !tile.tile_type.is_solid() && tile.tile_type != TileType::Lava
        } else {
            false
        }
    }

    /// Check if position has something to grab onto
    pub fn can_grab(&self, x: i32, y: i32) -> bool {
        if let Some(tile) = self.get_tile(x, y) {
            tile.tile_type.is_climbable() || tile.tile_type == TileType::Vine
        } else {
            false
        }
    }

    /// Check if position is a rest point (fast stamina regen)
    pub fn is_rest_point(&self, x: i32, y: i32) -> bool {
        if let Some(tile) = self.get_tile(x, y) {
            tile.tile_type.is_rest_point()
        } else {
            false
        }
    }

    /// Get nearest campfire position
    pub fn nearest_campfire(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        self.campfire_positions.iter()
            .min_by_key(|(cx, cy)| {
                let dx = (cx - x).abs();
                let dy = (cy - y).abs();
                dx * dx + dy * dy
            })
            .copied()
    }
}

/// Generate a daily seed from a date string
pub fn daily_seed(date: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    date.hash(&mut hasher);
    "SUMMIT_DAILY_SEED".hash(&mut hasher);
    hasher.finish()
}

/// Get today's date in YYYY-MM-DD format (Eastern timezone)
pub fn today_date() -> String {
    use chrono::{Utc, FixedOffset, TimeZone};

    // Eastern timezone is UTC-5 (ignoring DST for simplicity)
    let eastern = FixedOffset::west_opt(5 * 3600).unwrap();
    let now = eastern.from_utc_datetime(&Utc::now().naive_utc());
    now.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_seed_deterministic() {
        let seed1 = daily_seed("2026-01-30");
        let seed2 = daily_seed("2026-01-30");
        assert_eq!(seed1, seed2);

        let seed3 = daily_seed("2026-01-31");
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_mountain_generation_deterministic() {
        let seed = daily_seed("2026-01-30");
        let m1 = Mountain::generate(seed, "2026-01-30".to_string());
        let m2 = Mountain::generate(seed, "2026-01-30".to_string());

        // Same seed should produce same campfire positions
        assert_eq!(m1.campfire_positions, m2.campfire_positions);
    }

    #[test]
    fn test_mountain_dimensions() {
        let mountain = Mountain::generate(12345, "2026-01-30".to_string());

        assert!(mountain.get_tile(0, 0).is_some());
        assert!(mountain.get_tile(79, 99).is_some());
        assert!(mountain.get_tile(80, 0).is_none());
        assert!(mountain.get_tile(0, 100).is_none());
        assert!(mountain.get_tile(-1, 0).is_none());
    }

    #[test]
    fn test_campfire_placement() {
        let mountain = Mountain::generate(12345, "2026-01-30".to_string());

        // Should have 3 campfires (between biomes)
        assert_eq!(mountain.campfire_positions.len(), 3);

        // Campfires should be at biome boundaries
        for (_, y) in &mountain.campfire_positions {
            assert!(*y == 24 || *y == 49 || *y == 74);
        }
    }

    #[test]
    fn test_biome_at_height() {
        let mountain = Mountain::generate(12345, "2026-01-30".to_string());

        assert_eq!(mountain.biome_at_height(0), BiomeType::Beach);
        assert_eq!(mountain.biome_at_height(24), BiomeType::Beach);
        assert_eq!(mountain.biome_at_height(25), BiomeType::Jungle);
        assert_eq!(mountain.biome_at_height(49), BiomeType::Jungle);
        assert_eq!(mountain.biome_at_height(50), BiomeType::Alpine);
        assert_eq!(mountain.biome_at_height(74), BiomeType::Alpine);
        assert_eq!(mountain.biome_at_height(75), BiomeType::Volcanic);
        assert_eq!(mountain.biome_at_height(99), BiomeType::Volcanic);
    }

    #[test]
    fn test_viewport() {
        let mountain = Mountain::generate(12345, "2026-01-30".to_string());

        let viewport = mountain.get_viewport(40, 50, 20, 10);
        assert_eq!(viewport.len(), 10);
        assert_eq!(viewport[0].len(), 20);
    }

    #[test]
    fn test_tile_properties() {
        assert!(TileType::Rock.is_solid());
        assert!(!TileType::Air.is_solid());
        assert!(TileType::Climbable.is_climbable());
        assert!(TileType::Ledge.is_rest_point());
        assert!(TileType::Campfire.is_rest_point());
    }

    #[test]
    fn test_summit_reachable() {
        let mountain = Mountain::generate(12345, "2026-01-30".to_string());

        // Summit should have a ledge at the center
        let tile = mountain.get_tile(40, 99).unwrap();
        assert_eq!(tile.tile_type, TileType::Ledge);
    }
}
