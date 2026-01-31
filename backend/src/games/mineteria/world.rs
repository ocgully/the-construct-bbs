//! World generation and management for Mineteria
//!
//! Procedural world generation with biomes, terrain, and resources.

use serde::{Deserialize, Serialize};
use noise::{NoiseFn, Perlin};
use super::data::{BlockType, Biome};
use super::state::{Position, GameState};

/// Size of a chunk (in blocks)
pub const CHUNK_SIZE: i32 = 16;

/// World height limits
pub const WORLD_HEIGHT: i32 = 256;
pub const SEA_LEVEL: i32 = 64;
pub const SURFACE_LEVEL: i32 = 80;
pub const BEDROCK_LEVEL: i32 = 0;

/// A tile in the world
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub block: BlockType,
    pub light_level: u8,
    pub background: bool, // true = background wall, can't be walked through
}

impl Tile {
    pub fn new(block: BlockType) -> Self {
        Self {
            block,
            light_level: 0,
            background: false,
        }
    }

    pub fn air() -> Self {
        Self::new(BlockType::Air)
    }
}

/// A chunk of the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub x: i32,
    pub y: i32,
    pub tiles: Vec<Vec<Tile>>,
    pub biome: Biome,
}

impl Chunk {
    pub fn new(x: i32, y: i32, biome: Biome) -> Self {
        let tiles = vec![vec![Tile::air(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        Self { x, y, tiles, biome }
    }

    pub fn get_tile(&self, local_x: usize, local_y: usize) -> &Tile {
        &self.tiles[local_y][local_x]
    }

    pub fn set_tile(&mut self, local_x: usize, local_y: usize, tile: Tile) {
        self.tiles[local_y][local_x] = tile;
    }
}

/// World generator
pub struct WorldGenerator {
    seed: u64,
    terrain_noise: Perlin,
    cave_noise: Perlin,
    biome_noise: Perlin,
    ore_noise: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        let terrain = Perlin::new(seed as u32);
        let cave = Perlin::new((seed.wrapping_add(1)) as u32);
        let biome = Perlin::new((seed.wrapping_add(2)) as u32);
        let ore = Perlin::new((seed.wrapping_add(3)) as u32);

        Self {
            seed,
            terrain_noise: terrain,
            cave_noise: cave,
            biome_noise: biome,
            ore_noise: ore,
        }
    }

    /// Determine biome at world position
    pub fn get_biome(&self, world_x: i32) -> Biome {
        let scale = 0.005;
        let noise = self.biome_noise.get([world_x as f64 * scale, 0.0]);
        let temperature = self.biome_noise.get([world_x as f64 * scale, 100.0]);

        // Map noise to biomes
        if noise < -0.5 {
            if temperature > 0.3 {
                Biome::Desert
            } else if temperature < -0.3 {
                Biome::Tundra
            } else {
                Biome::Plains
            }
        } else if noise < 0.0 {
            if temperature < -0.2 {
                Biome::Mountains
            } else {
                Biome::Forest
            }
        } else if noise < 0.3 {
            Biome::Swamp
        } else {
            Biome::Ocean
        }
    }

    /// Get terrain height at world position
    pub fn get_terrain_height(&self, world_x: i32) -> i32 {
        let scale = 0.02;
        let noise = self.terrain_noise.get([world_x as f64 * scale, 0.0]);

        // Base height with noise
        let base = SURFACE_LEVEL as f64;
        let variation = noise * 20.0;

        // Biome-specific adjustments with smoothing
        // Sample biome modifier over a range and average for smooth transitions
        let biome_mod = self.get_smoothed_biome_modifier(world_x);

        (base + variation + biome_mod).round() as i32
    }

    /// Get smoothed biome modifier to prevent sudden terrain jumps at biome boundaries
    fn get_smoothed_biome_modifier(&self, world_x: i32) -> f64 {
        // Sample biome modifiers over a range and weight by distance
        let sample_range = 16; // Sample 16 blocks in each direction
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for offset in -sample_range..=sample_range {
            let sample_x = world_x + offset;
            let biome = self.get_biome(sample_x);
            let modifier = match biome {
                Biome::Mountains => 30.0,
                Biome::Ocean => -20.0,
                Biome::Swamp => -5.0,
                _ => 0.0,
            };

            // Weight by inverse distance (closer samples have more influence)
            let distance = offset.abs() as f64;
            let weight = 1.0 / (1.0 + distance * 0.5);

            weighted_sum += modifier * weight;
            weight_total += weight;
        }

        weighted_sum / weight_total
    }

    /// Check if position is a cave
    pub fn is_cave(&self, world_x: i32, world_y: i32) -> bool {
        if world_y > SURFACE_LEVEL - 10 {
            return false; // No caves near surface
        }

        let scale = 0.05;
        let noise = self.cave_noise.get([world_x as f64 * scale, world_y as f64 * scale]);

        // Larger caves deeper down
        let depth_factor = ((SURFACE_LEVEL - world_y) as f64 / 50.0).min(1.0);
        let threshold = 0.4 - (depth_factor * 0.1);

        noise > threshold
    }

    /// Check for ore at position
    pub fn get_ore(&self, world_x: i32, world_y: i32) -> Option<BlockType> {
        // Ores only spawn underground
        if world_y >= SURFACE_LEVEL - 20 {
            return None;
        }

        let scale = 0.1;
        let noise = self.ore_noise.get([world_x as f64 * scale, world_y as f64 * scale]);

        // Different ores at different depths
        let depth = SURFACE_LEVEL - world_y;

        if noise > 0.7 {
            if depth > 100 && noise > 0.9 {
                Some(BlockType::DiamondOre)
            } else if depth > 60 && noise > 0.85 {
                Some(BlockType::GoldOre)
            } else if depth > 30 {
                Some(BlockType::IronOre)
            } else if depth > 10 {
                Some(BlockType::CopperOre)
            } else {
                Some(BlockType::CoalOre)
            }
        } else {
            None
        }
    }

    /// Generate a chunk
    pub fn generate_chunk(&self, chunk_x: i32, chunk_y: i32) -> Chunk {
        let world_x_start = chunk_x * CHUNK_SIZE;
        let world_y_start = chunk_y * CHUNK_SIZE;

        // Determine biome from center of chunk
        let biome = self.get_biome(world_x_start + CHUNK_SIZE / 2);

        let mut chunk = Chunk::new(chunk_x, chunk_y, biome);

        for local_y in 0..CHUNK_SIZE as usize {
            for local_x in 0..CHUNK_SIZE as usize {
                let world_x = world_x_start + local_x as i32;
                let world_y = world_y_start + local_y as i32;

                let block = self.generate_block(world_x, world_y, &biome);
                chunk.set_tile(local_x, local_y, Tile::new(block));
            }
        }

        // Add trees on surface
        self.add_features(&mut chunk, world_x_start, world_y_start, &biome);

        chunk
    }

    /// Generate a single block at world position
    fn generate_block(&self, world_x: i32, world_y: i32, biome: &Biome) -> BlockType {
        let terrain_height = self.get_terrain_height(world_x);

        // Bedrock at bottom
        if world_y <= BEDROCK_LEVEL {
            return BlockType::Bedrock;
        }

        // Above terrain
        if world_y > terrain_height {
            // Water in oceans below sea level
            if *biome == Biome::Ocean && world_y <= SEA_LEVEL {
                return BlockType::Water;
            }
            return BlockType::Air;
        }

        // Surface block
        if world_y == terrain_height {
            return biome.surface_block();
        }

        // Subsurface (few blocks below surface)
        if world_y > terrain_height - 5 {
            return biome.subsurface_block();
        }

        // Check for caves
        if self.is_cave(world_x, world_y) {
            // Lava in deep caves
            if world_y < 20 && rand_from_coords(world_x, world_y, self.seed) % 10 == 0 {
                return BlockType::Lava;
            }
            return BlockType::Air;
        }

        // Check for ores
        if let Some(ore) = self.get_ore(world_x, world_y) {
            return ore;
        }

        // Default stone
        BlockType::Stone
    }

    /// Add surface features (trees, cacti, etc.)
    fn add_features(&self, chunk: &mut Chunk, world_x_start: i32, world_y_start: i32, biome: &Biome) {
        if !biome.has_trees() && *biome != Biome::Desert {
            return;
        }

        for local_x in 2..(CHUNK_SIZE - 2) as usize {
            let world_x = world_x_start + local_x as i32;
            let terrain_height = self.get_terrain_height(world_x);

            // Check if terrain surface is in this chunk
            let local_y = terrain_height - world_y_start;
            if local_y < 0 || local_y >= CHUNK_SIZE {
                continue;
            }

            // Random tree placement
            let rand = rand_from_coords(world_x, terrain_height, self.seed);
            if rand % 15 != 0 {
                continue;
            }

            match biome {
                Biome::Desert => {
                    // Cacti
                    let height = (rand % 3 + 2) as usize;
                    for h in 1..=height {
                        let y = local_y as usize - h;
                        if y > 0 && y < CHUNK_SIZE as usize {
                            chunk.set_tile(local_x, y, Tile::new(BlockType::Cactus));
                        }
                    }
                }
                _ => {
                    // Trees
                    let trunk_height = (rand % 3 + 4) as usize;

                    // Trunk
                    for h in 1..=trunk_height {
                        let y = local_y as usize - h;
                        if y > 0 && y < CHUNK_SIZE as usize {
                            chunk.set_tile(local_x, y, Tile::new(BlockType::Wood));
                        }
                    }

                    // Leaves (simple sphere-ish shape)
                    let top_y = local_y as i32 - trunk_height as i32;
                    for dy in -2i32..=0 {
                        for dx in -2i32..=2 {
                            if dx.abs() + dy.abs() > 3 {
                                continue;
                            }
                            let lx = local_x as i32 + dx;
                            let ly = top_y + dy - world_y_start;
                            if lx >= 0
                                && lx < CHUNK_SIZE
                                && ly >= 0
                                && ly < CHUNK_SIZE
                            {
                                let current = chunk.get_tile(lx as usize, ly as usize);
                                if current.block == BlockType::Air {
                                    chunk.set_tile(lx as usize, ly as usize, Tile::new(BlockType::Leaves));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// World structure for runtime
pub struct World {
    pub seed: u64,
    generator: WorldGenerator,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            generator: WorldGenerator::new(seed),
        }
    }

    /// Get block at world position, considering player modifications
    pub fn get_block(&self, state: &GameState, x: i32, y: i32) -> BlockType {
        // Check player modifications first
        if let Some(block) = state.get_modified_block(x, y) {
            return block;
        }

        // Generate procedurally
        let biome = self.generator.get_biome(x);
        self.generator.generate_block(x, y, &biome)
    }

    /// Get terrain height at x position
    pub fn get_terrain_height(&self, x: i32) -> i32 {
        self.generator.get_terrain_height(x)
    }

    /// Get biome at x position
    pub fn get_biome(&self, x: i32) -> Biome {
        self.generator.get_biome(x)
    }

    /// Check if a position is solid (can't walk through)
    pub fn is_solid(&self, state: &GameState, x: i32, y: i32) -> bool {
        let block = self.get_block(state, x, y);
        block.get_block().solid
    }

    /// Get visible area around player
    pub fn get_visible_area(&self, state: &GameState, width: i32, height: i32) -> Vec<Vec<BlockType>> {
        let half_w = width / 2;
        let half_h = height / 2;

        let mut area = Vec::with_capacity(height as usize);

        for dy in -half_h..half_h {
            let mut row = Vec::with_capacity(width as usize);
            for dx in -half_w..half_w {
                let x = state.position.x + dx;
                let y = state.position.y - dy; // Y is inverted for display
                row.push(self.get_block(state, x, y));
            }
            area.push(row);
        }

        area
    }

    /// Find spawn point near x=0
    pub fn find_spawn_point(&self) -> Position {
        // Find surface level near x=0
        let surface_y = self.get_terrain_height(0);
        Position::new(0, surface_y + 1)
    }
}

/// Simple deterministic random from coordinates
fn rand_from_coords(x: i32, y: i32, seed: u64) -> u64 {
    let mut h = seed;
    h ^= x as u64;
    h = h.wrapping_mul(0x517cc1b727220a95);
    h ^= y as u64;
    h = h.wrapping_mul(0x517cc1b727220a95);
    h ^= h >> 32;
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_generation() {
        let world = World::new(12345);
        let state = GameState::new(12345);

        // Surface should have grass/sand/snow depending on biome
        let surface_y = world.get_terrain_height(0);
        let surface_block = world.get_block(&state, 0, surface_y);
        assert!(matches!(
            surface_block,
            BlockType::Grass | BlockType::Sand | BlockType::Snow | BlockType::Stone
        ));

        // Underground should have stone
        let deep_block = world.get_block(&state, 0, 20);
        assert!(matches!(
            deep_block,
            BlockType::Stone | BlockType::Air | BlockType::Lava |
            BlockType::CoalOre | BlockType::IronOre | BlockType::GoldOre |
            BlockType::DiamondOre | BlockType::CopperOre
        ));

        // Bedrock at bottom
        let bedrock = world.get_block(&state, 0, 0);
        assert_eq!(bedrock, BlockType::Bedrock);
    }

    #[test]
    fn test_modified_blocks() {
        let world = World::new(12345);
        let mut state = GameState::new(12345);

        // Place a torch
        state.set_modified_block(10, 50, BlockType::Torch);
        let block = world.get_block(&state, 10, 50);
        assert_eq!(block, BlockType::Torch);
    }

    #[test]
    fn test_biome_variation() {
        let world = World::new(12345);

        // Check biomes vary across world
        let mut biomes = std::collections::HashSet::new();
        for x in (-1000..1000).step_by(100) {
            biomes.insert(world.get_biome(x));
        }
        // Should have multiple biome types
        assert!(biomes.len() > 1);
    }

    #[test]
    fn test_terrain_continuity() {
        let world = World::new(12345);

        // Terrain should be continuous (no sudden jumps)
        let mut prev_height = world.get_terrain_height(0);
        for x in 1..100 {
            let height = world.get_terrain_height(x);
            let diff = (height - prev_height).abs();
            assert!(diff <= 3, "Terrain jump of {} at x={}", diff, x);
            prev_height = height;
        }
    }

    #[test]
    fn test_visible_area() {
        let world = World::new(12345);
        let state = GameState::new(12345);

        let area = world.get_visible_area(&state, 10, 10);
        assert_eq!(area.len(), 10);
        assert_eq!(area[0].len(), 10);
    }
}
