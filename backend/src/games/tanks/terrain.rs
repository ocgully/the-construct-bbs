//! Terrain generation and destruction for Tanks
//!
//! Generates procedural terrain and handles destruction from explosions.

use rand::Rng;
use super::data::{FIELD_WIDTH, FIELD_HEIGHT};

/// Terrain types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Air,
    Dirt,
    Rock,       // Harder to destroy
    Bedrock,    // Indestructible (bottom layer)
}

impl TerrainType {
    pub fn symbol(&self) -> char {
        match self {
            TerrainType::Air => ' ',
            TerrainType::Dirt => '#',
            TerrainType::Rock => '%',
            TerrainType::Bedrock => '=',
        }
    }

    pub fn is_solid(&self) -> bool {
        !matches!(self, TerrainType::Air)
    }

    pub fn is_destructible(&self) -> bool {
        matches!(self, TerrainType::Dirt | TerrainType::Rock)
    }

    /// How much damage is needed to destroy this terrain type
    pub fn toughness(&self) -> u32 {
        match self {
            TerrainType::Air => 0,
            TerrainType::Dirt => 1,
            TerrainType::Rock => 3,
            TerrainType::Bedrock => u32::MAX,
        }
    }
}

/// Terrain grid
#[derive(Debug, Clone)]
pub struct Terrain {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<TerrainType>>,
}

impl Terrain {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![vec![TerrainType::Air; width]; height],
        }
    }

    /// Get terrain at position (0,0 is top-left)
    pub fn get(&self, x: usize, y: usize) -> TerrainType {
        if x >= self.width || y >= self.height {
            return TerrainType::Air;
        }
        self.grid[y][x]
    }

    /// Set terrain at position
    pub fn set(&mut self, x: usize, y: usize, terrain_type: TerrainType) {
        if x < self.width && y < self.height {
            self.grid[y][x] = terrain_type;
        }
    }

    /// Get the ground level at a given x position (y coordinate of first solid from top)
    pub fn ground_level(&self, x: usize) -> usize {
        if x >= self.width {
            return self.height;
        }
        for y in 0..self.height {
            if self.grid[y][x].is_solid() {
                return y;
            }
        }
        self.height
    }

    /// Check if position is solid (for collision)
    pub fn is_solid(&self, x: usize, y: usize) -> bool {
        self.get(x, y).is_solid()
    }

    /// Check if position is in bounds
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    /// Destroy terrain in a circular area
    /// Returns list of destroyed coordinates
    pub fn destroy_circle(&mut self, center_x: i32, center_y: i32, radius: u32) -> Vec<(usize, usize)> {
        let mut destroyed = Vec::new();
        let r = radius as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= (r * r) {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                        let xu = x as usize;
                        let yu = y as usize;
                        if self.grid[yu][xu].is_destructible() {
                            self.grid[yu][xu] = TerrainType::Air;
                            destroyed.push((xu, yu));
                        }
                    }
                }
            }
        }

        destroyed
    }

    /// Add terrain in a circular area (for dirt bomb)
    pub fn add_circle(&mut self, center_x: i32, center_y: i32, radius: u32) {
        let r = radius as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= (r * r) {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                        let xu = x as usize;
                        let yu = y as usize;
                        if self.grid[yu][xu] == TerrainType::Air {
                            self.grid[yu][xu] = TerrainType::Dirt;
                        }
                    }
                }
            }
        }
    }

    /// Apply gravity - terrain falls down if unsupported
    /// Returns true if any terrain moved
    pub fn apply_gravity(&mut self) -> bool {
        let mut moved = false;

        // Process from bottom up
        for y in (0..self.height - 1).rev() {
            for x in 0..self.width {
                if self.grid[y][x] == TerrainType::Dirt {
                    // Check if there's air below
                    let mut drop_y = y;
                    while drop_y + 1 < self.height && self.grid[drop_y + 1][x] == TerrainType::Air {
                        drop_y += 1;
                    }
                    if drop_y != y {
                        self.grid[drop_y][x] = TerrainType::Dirt;
                        self.grid[y][x] = TerrainType::Air;
                        moved = true;
                    }
                }
            }
        }

        moved
    }
}

/// Terrain generation style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerrainStyle {
    Hills,
    Mountains,
    Valleys,
    Flat,
    Random,
}

/// Generate terrain with a given style and seed
pub fn generate_terrain(style: TerrainStyle, seed: u64) -> Terrain {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut terrain = Terrain::new(FIELD_WIDTH, FIELD_HEIGHT);

    // Generate height map
    let heights = match style {
        TerrainStyle::Hills => generate_hills(&mut rng),
        TerrainStyle::Mountains => generate_mountains(&mut rng),
        TerrainStyle::Valleys => generate_valleys(&mut rng),
        TerrainStyle::Flat => generate_flat(&mut rng),
        TerrainStyle::Random => {
            let styles = [TerrainStyle::Hills, TerrainStyle::Mountains, TerrainStyle::Valleys];
            let style = styles[rng.gen_range(0..styles.len())];
            return generate_terrain(style, seed);
        }
    };

    // Fill terrain based on heights
    for x in 0..FIELD_WIDTH {
        let ground_y = heights[x].min(FIELD_HEIGHT - 1);

        for y in ground_y..FIELD_HEIGHT {
            if y >= FIELD_HEIGHT - 1 {
                terrain.set(x, y, TerrainType::Bedrock);
            } else if y >= FIELD_HEIGHT - 2 || rng.gen_bool(0.1) {
                terrain.set(x, y, TerrainType::Rock);
            } else {
                terrain.set(x, y, TerrainType::Dirt);
            }
        }
    }

    terrain
}

fn generate_hills<R: Rng>(rng: &mut R) -> Vec<usize> {
    let mut heights = vec![FIELD_HEIGHT / 2; FIELD_WIDTH];
    let base_height = FIELD_HEIGHT / 2;

    // Generate several smooth hills using sine waves
    let num_hills = rng.gen_range(2..5);
    for _ in 0..num_hills {
        let amplitude = rng.gen_range(2..6) as f64;
        let frequency = rng.gen_range(8..20) as f64;
        let phase = rng.gen_range(0.0..std::f64::consts::TAU);

        for x in 0..FIELD_WIDTH {
            let wave = (amplitude * (x as f64 / frequency + phase).sin()) as i32;
            heights[x] = ((base_height as i32 + wave).max(3) as usize).min(FIELD_HEIGHT - 3);
        }
    }

    heights
}

fn generate_mountains<R: Rng>(rng: &mut R) -> Vec<usize> {
    let mut heights = vec![FIELD_HEIGHT - 5; FIELD_WIDTH];

    // Generate peaks
    let num_peaks = rng.gen_range(2..4);
    for _ in 0..num_peaks {
        let peak_x = rng.gen_range(5..FIELD_WIDTH - 5);
        let peak_height = rng.gen_range(3..8);
        let width = rng.gen_range(8..15);

        for x in 0..FIELD_WIDTH {
            let dist = (x as i32 - peak_x as i32).abs() as f64;
            if dist < width as f64 {
                let height_add = ((1.0 - dist / width as f64) * peak_height as f64) as usize;
                heights[x] = heights[x].saturating_sub(height_add).max(2);
            }
        }
    }

    heights
}

fn generate_valleys<R: Rng>(rng: &mut R) -> Vec<usize> {
    let mut heights = vec![FIELD_HEIGHT / 3; FIELD_WIDTH];

    // Generate valleys (inverted peaks)
    let num_valleys = rng.gen_range(1..3);
    for _ in 0..num_valleys {
        let valley_x = rng.gen_range(10..FIELD_WIDTH - 10);
        let valley_depth = rng.gen_range(3..6);
        let width = rng.gen_range(10..20);

        for x in 0..FIELD_WIDTH {
            let dist = (x as i32 - valley_x as i32).abs() as f64;
            if dist < width as f64 {
                let depth_add = ((1.0 - dist / width as f64) * valley_depth as f64) as usize;
                heights[x] = (heights[x] + depth_add).min(FIELD_HEIGHT - 3);
            }
        }
    }

    heights
}

fn generate_flat<R: Rng>(rng: &mut R) -> Vec<usize> {
    let base = rng.gen_range(FIELD_HEIGHT / 2..FIELD_HEIGHT - 4);
    let mut heights = vec![base; FIELD_WIDTH];

    // Add small random variation
    for x in 0..FIELD_WIDTH {
        let variation = rng.gen_range(-1..=1);
        heights[x] = ((base as i32 + variation).max(3) as usize).min(FIELD_HEIGHT - 3);
    }

    heights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_creation() {
        let terrain = Terrain::new(80, 20);
        assert_eq!(terrain.width, 80);
        assert_eq!(terrain.height, 20);
        assert_eq!(terrain.get(0, 0), TerrainType::Air);
    }

    #[test]
    fn test_terrain_set_get() {
        let mut terrain = Terrain::new(10, 10);
        terrain.set(5, 5, TerrainType::Dirt);
        assert_eq!(terrain.get(5, 5), TerrainType::Dirt);
        assert!(terrain.is_solid(5, 5));
    }

    #[test]
    fn test_destroy_circle() {
        let mut terrain = Terrain::new(20, 20);
        // Fill with dirt
        for y in 10..20 {
            for x in 0..20 {
                terrain.set(x, y, TerrainType::Dirt);
            }
        }

        let destroyed = terrain.destroy_circle(10, 15, 3);
        assert!(!destroyed.is_empty());
        assert_eq!(terrain.get(10, 15), TerrainType::Air);
    }

    #[test]
    fn test_bedrock_indestructible() {
        let mut terrain = Terrain::new(10, 10);
        terrain.set(5, 9, TerrainType::Bedrock);
        let destroyed = terrain.destroy_circle(5, 9, 2);

        // Bedrock should not be destroyed
        assert_eq!(terrain.get(5, 9), TerrainType::Bedrock);
        assert!(!destroyed.contains(&(5, 9)));
    }

    #[test]
    fn test_ground_level() {
        let mut terrain = Terrain::new(10, 10);
        // Air from 0-5, dirt from 6-9
        for y in 6..10 {
            terrain.set(3, y, TerrainType::Dirt);
        }
        assert_eq!(terrain.ground_level(3), 6);
        assert_eq!(terrain.ground_level(0), 10); // No ground
    }

    #[test]
    fn test_generate_terrain() {
        let terrain = generate_terrain(TerrainStyle::Hills, 12345);
        assert_eq!(terrain.width, FIELD_WIDTH);
        assert_eq!(terrain.height, FIELD_HEIGHT);

        // Should have some ground
        let has_ground = (0..FIELD_WIDTH).any(|x| terrain.ground_level(x) < FIELD_HEIGHT);
        assert!(has_ground);
    }

    #[test]
    fn test_gravity() {
        let mut terrain = Terrain::new(10, 10);
        terrain.set(5, 3, TerrainType::Dirt);
        // Leave 4-7 as air
        terrain.set(5, 8, TerrainType::Bedrock);

        let moved = terrain.apply_gravity();
        assert!(moved);
        assert_eq!(terrain.get(5, 3), TerrainType::Air);
        assert_eq!(terrain.get(5, 7), TerrainType::Dirt); // Fell to above bedrock
    }
}
