//! Z-level terrain system for Fortress
//!
//! Implements a multi-level underground terrain with mining and digging.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use rand::Rng;

/// Tile types in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    // Natural terrain
    Empty,          // Air/empty space
    Soil,           // Diggable dirt
    Stone,          // Hard rock, needs mining
    Ore(OreType),   // Ore deposits
    Gem,            // Gem deposits
    Water,          // Underground water
    Lava,           // Dangerous!
    Tree,           // Surface trees
    Grass,          // Surface grass
    Shrub,          // Bushes and plants

    // Man-made
    Floor,          // Dug-out floor
    Wall,           // Constructed wall
    Door,           // Passable door
    Stairs,         // Up/down stairs
    Ramp,           // Slope between levels
    Stockpile,      // Storage designation
    Workshop,       // Workshop area
    Farm,           // Farming plot
}

/// Types of ore deposits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OreType {
    Iron,
    Copper,
    Gold,
    Silver,
    Coal,
}

impl OreType {
    pub fn resource_key(&self) -> &'static str {
        match self {
            OreType::Iron => "iron_ore",
            OreType::Copper => "copper_ore",
            OreType::Gold => "gold_ore",
            OreType::Silver => "silver_ore",
            OreType::Coal => "coal",
        }
    }
}

/// A single tile with type and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub revealed: bool,
    pub designated: bool,     // Marked for digging
    pub room_id: Option<u32>, // Room this tile belongs to
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            tile_type: TileType::Stone,
            revealed: false,
            designated: false,
            room_id: None,
        }
    }
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            revealed: false,
            designated: false,
            room_id: None,
        }
    }

    pub fn is_passable(&self) -> bool {
        matches!(
            self.tile_type,
            TileType::Empty
                | TileType::Floor
                | TileType::Door
                | TileType::Stairs
                | TileType::Ramp
                | TileType::Stockpile
                | TileType::Workshop
                | TileType::Farm
                | TileType::Grass
        )
    }

    pub fn is_diggable(&self) -> bool {
        matches!(
            self.tile_type,
            TileType::Soil | TileType::Stone | TileType::Ore(_) | TileType::Gem
        )
    }

    /// ASCII character for rendering
    pub fn char(&self) -> char {
        if !self.revealed {
            return ' ';
        }

        match self.tile_type {
            TileType::Empty => ' ',
            TileType::Soil => '.',
            TileType::Stone => '#',
            TileType::Ore(OreType::Iron) => '%',
            TileType::Ore(OreType::Copper) => '%',
            TileType::Ore(OreType::Gold) => '$',
            TileType::Ore(OreType::Silver) => '%',
            TileType::Ore(OreType::Coal) => '%',
            TileType::Gem => '*',
            TileType::Water => '~',
            TileType::Lava => '&',
            TileType::Tree => 'T',
            TileType::Grass => '"',
            TileType::Shrub => ',',
            TileType::Floor => '.',
            TileType::Wall => '#',
            TileType::Door => '+',
            TileType::Stairs => 'X',
            TileType::Ramp => '/',
            TileType::Stockpile => '_',
            TileType::Workshop => 'W',
            TileType::Farm => '~',
        }
    }
}

/// Multi-level terrain map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub width: u32,
    pub height: u32,
    pub depth: u32,   // Number of z-levels
    levels: Vec<Vec<Vec<Tile>>>, // z -> y -> x
    #[serde(default)]
    pub fortress_entrance: (u32, u32), // x, y on surface
}

impl Terrain {
    /// Create new terrain with procedural generation
    pub fn new(width: u32, height: u32, depth: u32, seed: u64) -> Self {
        use rand::{Rng, SeedableRng};
        use rand::rngs::StdRng;

        let mut rng = StdRng::seed_from_u64(seed);
        let mut levels = Vec::with_capacity(depth as usize);

        // Generate each z-level
        for z in 0..depth {
            let mut level = Vec::with_capacity(height as usize);

            for y in 0..height {
                let mut row = Vec::with_capacity(width as usize);

                for x in 0..width {
                    let tile_type = if z == 0 {
                        // Surface level
                        Self::generate_surface_tile(&mut rng, x, y, width, height)
                    } else {
                        // Underground level
                        Self::generate_underground_tile(&mut rng, x, y, z, width, height)
                    };

                    let mut tile = Tile::new(tile_type);
                    // Surface is always revealed
                    if z == 0 {
                        tile.revealed = true;
                    }
                    row.push(tile);
                }
                level.push(row);
            }
            levels.push(level);
        }

        // Place fortress entrance near center of map
        let entrance_x = width / 2 + rng.gen_range(0..5) as u32;
        let entrance_y = height / 2 + rng.gen_range(0..5) as u32;

        Self {
            width,
            height,
            depth,
            levels,
            fortress_entrance: (entrance_x, entrance_y),
        }
    }

    fn generate_surface_tile<R: Rng>(rng: &mut R, x: u32, y: u32, width: u32, height: u32) -> TileType {
        // Border with trees
        if x < 2 || x >= width - 2 || y < 2 || y >= height - 2 {
            return TileType::Tree;
        }

        // Random terrain
        match rng.gen_range(0..100) {
            0..=60 => TileType::Grass,
            61..=75 => TileType::Tree,
            76..=85 => TileType::Shrub,
            86..=90 => TileType::Soil,
            91..=95 => TileType::Stone,
            _ => TileType::Water,
        }
    }

    fn generate_underground_tile<R: Rng>(rng: &mut R, _x: u32, _y: u32, z: u32, _width: u32, _height: u32) -> TileType {
        // Deeper levels have more ore and gems
        let depth_bonus = z as i32 * 2;

        match rng.gen_range(0..100) + depth_bonus {
            0..=10 => TileType::Soil,
            11..=80 => TileType::Stone,
            81..=88 => TileType::Ore(OreType::Iron),
            89..=93 => TileType::Ore(OreType::Copper),
            94..=96 => TileType::Ore(OreType::Coal),
            97..=100 => TileType::Ore(OreType::Gold),
            101..=105 => TileType::Gem,
            106..=108 => TileType::Water,
            _ => if rng.gen_bool(0.01) { TileType::Lava } else { TileType::Gem },
        }
    }

    /// Get tile at position
    pub fn get(&self, x: u32, y: u32, z: u32) -> Option<&Tile> {
        if x >= self.width || y >= self.height || z >= self.depth {
            return None;
        }
        Some(&self.levels[z as usize][y as usize][x as usize])
    }

    /// Get mutable tile at position
    pub fn get_mut(&mut self, x: u32, y: u32, z: u32) -> Option<&mut Tile> {
        if x >= self.width || y >= self.height || z >= self.depth {
            return None;
        }
        Some(&mut self.levels[z as usize][y as usize][x as usize])
    }

    /// Dig a tile, returning resources extracted
    pub fn dig(&mut self, x: u32, y: u32, z: u32) -> Option<Vec<(&'static str, u32)>> {
        let tile = self.get_mut(x, y, z)?;

        if !tile.is_diggable() {
            return None;
        }

        let resources = match tile.tile_type {
            TileType::Soil => vec![],
            TileType::Stone => vec![("stone", 1)],
            TileType::Ore(ore_type) => vec![(ore_type.resource_key(), 2)],
            TileType::Gem => vec![("gem", 1)],
            _ => vec![],
        };

        tile.tile_type = TileType::Floor;
        tile.revealed = true;
        tile.designated = false;

        // Reveal adjacent tiles
        self.reveal_adjacent(x, y, z);

        Some(resources)
    }

    /// Reveal tiles adjacent to a position
    pub fn reveal_adjacent(&mut self, x: u32, y: u32, z: u32) {
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                let nx = (x as i32 + dx) as u32;
                let ny = (y as i32 + dy) as u32;

                if let Some(tile) = self.get_mut(nx, ny, z) {
                    tile.revealed = true;
                }
            }
        }
    }

    /// Designate a tile for digging
    pub fn designate(&mut self, x: u32, y: u32, z: u32) -> bool {
        if let Some(tile) = self.get_mut(x, y, z) {
            if tile.is_diggable() && tile.revealed {
                tile.designated = true;
                return true;
            }
        }
        false
    }

    /// Place a constructed wall
    pub fn build_wall(&mut self, x: u32, y: u32, z: u32) -> bool {
        if let Some(tile) = self.get_mut(x, y, z) {
            if tile.tile_type == TileType::Floor || tile.tile_type == TileType::Empty {
                tile.tile_type = TileType::Wall;
                return true;
            }
        }
        false
    }

    /// Place stairs connecting two levels
    pub fn build_stairs(&mut self, x: u32, y: u32, z: u32) -> bool {
        // Check both levels exist
        if z >= self.depth - 1 {
            return false;
        }

        let upper = self.get(x, y, z);
        let lower = self.get(x, y, z + 1);

        if let (Some(u), Some(l)) = (upper, lower) {
            if u.is_passable() && l.tile_type == TileType::Floor {
                if let Some(tile) = self.get_mut(x, y, z) {
                    tile.tile_type = TileType::Stairs;
                }
                if let Some(tile) = self.get_mut(x, y, z + 1) {
                    tile.tile_type = TileType::Stairs;
                }
                return true;
            }
        }
        false
    }

    /// Set tile as stockpile
    pub fn set_stockpile(&mut self, x: u32, y: u32, z: u32) -> bool {
        if let Some(tile) = self.get_mut(x, y, z) {
            if tile.tile_type == TileType::Floor {
                tile.tile_type = TileType::Stockpile;
                return true;
            }
        }
        false
    }

    /// Set tile as farm
    pub fn set_farm(&mut self, x: u32, y: u32, z: u32) -> bool {
        if let Some(tile) = self.get_mut(x, y, z) {
            if tile.tile_type == TileType::Soil || tile.tile_type == TileType::Floor {
                tile.tile_type = TileType::Farm;
                return true;
            }
        }
        false
    }

    /// Get all designated tiles for digging
    pub fn get_dig_designations(&self, z: u32) -> Vec<(u32, u32)> {
        let mut result = Vec::new();
        if z >= self.depth {
            return result;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(tile) = self.get(x, y, z) {
                    if tile.designated {
                        result.push((x, y));
                    }
                }
            }
        }
        result
    }

    /// Count tiles of a specific type on a level
    pub fn count_tiles(&self, z: u32, tile_type: TileType) -> u32 {
        if z >= self.depth {
            return 0;
        }

        let mut count = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(tile) = self.get(x, y, z) {
                    if tile.tile_type == tile_type {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Find path between two points (simple A* implementation)
    pub fn find_path(&self, from: (u32, u32, u32), to: (u32, u32, u32)) -> Option<Vec<(u32, u32, u32)>> {
        use std::collections::{BinaryHeap, HashSet};
        use std::cmp::Ordering;

        #[derive(Eq, PartialEq)]
        struct Node {
            pos: (u32, u32, u32),
            cost: u32,
            estimate: u32,
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                (other.cost + other.estimate).cmp(&(self.cost + self.estimate))
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut open = BinaryHeap::new();
        let mut closed = HashSet::new();
        let mut came_from: HashMap<(u32, u32, u32), (u32, u32, u32)> = HashMap::new();

        open.push(Node {
            pos: from,
            cost: 0,
            estimate: Self::heuristic(from, to),
        });

        while let Some(current) = open.pop() {
            if current.pos == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut pos = to;
                while let Some(&prev) = came_from.get(&pos) {
                    path.push(prev);
                    pos = prev;
                }
                path.reverse();
                return Some(path);
            }

            if closed.contains(&current.pos) {
                continue;
            }
            closed.insert(current.pos);

            // Check neighbors
            for neighbor in self.get_neighbors(current.pos) {
                if closed.contains(&neighbor) {
                    continue;
                }

                came_from.insert(neighbor, current.pos);
                open.push(Node {
                    pos: neighbor,
                    cost: current.cost + 1,
                    estimate: Self::heuristic(neighbor, to),
                });
            }
        }

        None
    }

    fn heuristic(from: (u32, u32, u32), to: (u32, u32, u32)) -> u32 {
        let dx = (from.0 as i32 - to.0 as i32).unsigned_abs();
        let dy = (from.1 as i32 - to.1 as i32).unsigned_abs();
        let dz = (from.2 as i32 - to.2 as i32).unsigned_abs();
        dx + dy + dz
    }

    fn get_neighbors(&self, pos: (u32, u32, u32)) -> Vec<(u32, u32, u32)> {
        let mut neighbors = Vec::new();
        let (x, y, z) = pos;

        // Horizontal neighbors
        for (dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 {
                let nx = nx as u32;
                let ny = ny as u32;
                if let Some(tile) = self.get(nx, ny, z) {
                    if tile.is_passable() {
                        neighbors.push((nx, ny, z));
                    }
                }
            }
        }

        // Vertical neighbors (stairs)
        if let Some(tile) = self.get(x, y, z) {
            if tile.tile_type == TileType::Stairs {
                if z > 0 {
                    if let Some(above) = self.get(x, y, z - 1) {
                        if above.tile_type == TileType::Stairs || above.is_passable() {
                            neighbors.push((x, y, z - 1));
                        }
                    }
                }
                if z < self.depth - 1 {
                    if let Some(below) = self.get(x, y, z + 1) {
                        if below.tile_type == TileType::Stairs || below.is_passable() {
                            neighbors.push((x, y, z + 1));
                        }
                    }
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_creation() {
        let terrain = Terrain::new(50, 50, 5, 12345);
        assert_eq!(terrain.width, 50);
        assert_eq!(terrain.height, 50);
        assert_eq!(terrain.depth, 5);
    }

    #[test]
    fn test_tile_passability() {
        let floor = Tile::new(TileType::Floor);
        let wall = Tile::new(TileType::Stone);

        assert!(floor.is_passable());
        assert!(!wall.is_passable());
    }

    #[test]
    fn test_digging() {
        let mut terrain = Terrain::new(10, 10, 3, 42);

        // Reveal and designate a stone tile
        if let Some(tile) = terrain.get_mut(5, 5, 1) {
            tile.tile_type = TileType::Stone;
            tile.revealed = true;
        }

        terrain.designate(5, 5, 1);
        let resources = terrain.dig(5, 5, 1);

        assert!(resources.is_some());
        let tile = terrain.get(5, 5, 1).unwrap();
        assert_eq!(tile.tile_type, TileType::Floor);
    }

    #[test]
    fn test_tile_chars() {
        let mut floor = Tile::new(TileType::Floor);
        floor.revealed = true;
        assert_eq!(floor.char(), '.');

        let mut stone = Tile::new(TileType::Stone);
        stone.revealed = true;
        assert_eq!(stone.char(), '#');

        let unrevealed = Tile::new(TileType::Stone);
        assert_eq!(unrevealed.char(), ' ');
    }

    #[test]
    fn test_ore_types() {
        assert_eq!(OreType::Iron.resource_key(), "iron_ore");
        assert_eq!(OreType::Gold.resource_key(), "gold_ore");
    }
}
