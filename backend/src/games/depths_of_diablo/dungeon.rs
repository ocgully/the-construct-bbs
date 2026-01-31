//! Procedural dungeon generation for Depths of Diablo
//!
//! Generates dungeons from daily seeds using BSP (Binary Space Partitioning).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use super::data::MonsterType;
use super::items::Item;

/// Dungeon theme based on floor depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DungeonTheme {
    Cathedral,  // Floors 1-5
    Catacombs,  // Floors 6-10
    Caves,      // Floors 11-15
    Hell,       // Floors 16-20
}

impl DungeonTheme {
    pub fn for_floor(floor: u32) -> Self {
        match floor {
            1..=5 => DungeonTheme::Cathedral,
            6..=10 => DungeonTheme::Catacombs,
            11..=15 => DungeonTheme::Caves,
            _ => DungeonTheme::Hell,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DungeonTheme::Cathedral => "Cathedral",
            DungeonTheme::Catacombs => "Catacombs",
            DungeonTheme::Caves => "Caves",
            DungeonTheme::Hell => "Hell",
        }
    }

    pub fn floor_char(&self) -> char {
        match self {
            DungeonTheme::Cathedral => '.',
            DungeonTheme::Catacombs => ',',
            DungeonTheme::Caves => '`',
            DungeonTheme::Hell => '~',
        }
    }

    pub fn wall_char(&self) -> char {
        match self {
            DungeonTheme::Cathedral => '#',
            DungeonTheme::Catacombs => '%',
            DungeonTheme::Caves => '*',
            DungeonTheme::Hell => '&',
        }
    }
}

/// Types of rooms in the dungeon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    Start,     // Player spawn point
    Standard,  // Normal room with enemies
    Treasure,  // Extra loot
    Shrine,    // Buff shrine
    Boss,      // Floor boss
    Exit,      // Stairs down
}

/// A tile in the dungeon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
    Door,
    StairsUp,
    StairsDown,
    Chest,
    Shrine,
    Trap,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        !matches!(self, Tile::Wall)
    }

    pub fn char(&self, theme: DungeonTheme) -> char {
        match self {
            Tile::Wall => theme.wall_char(),
            Tile::Floor => theme.floor_char(),
            Tile::Door => '+',
            Tile::StairsUp => '<',
            Tile::StairsDown => '>',
            Tile::Chest => '$',
            Tile::Shrine => '*',
            Tile::Trap => '^',
        }
    }
}

/// A room in the dungeon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: usize,
    pub room_type: RoomType,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub connected_to: Vec<usize>,
}

impl Room {
    pub fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }
}

/// A monster instance in the dungeon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub id: u64,
    pub monster_type: MonsterType,
    pub x: usize,
    pub y: usize,
    pub health: i32,
    pub max_health: i32,
    pub last_attack_ms: u64,
}

impl Monster {
    pub fn new(id: u64, monster_type: MonsterType, x: usize, y: usize) -> Self {
        let stats = monster_type.stats();
        Monster {
            id,
            monster_type,
            x,
            y,
            health: stats.health,
            max_health: stats.health,
            last_attack_ms: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.health = (self.health - damage).max(0);
    }
}

/// A dropped item in the dungeon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedItem {
    pub item: Item,
    pub x: usize,
    pub y: usize,
}

/// The complete dungeon for a floor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub floor: u32,
    pub theme: DungeonTheme,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub rooms: Vec<Room>,
    pub monsters: Vec<Monster>,
    pub items: Vec<DroppedItem>,
    pub start_pos: (usize, usize),
    pub exit_pos: (usize, usize),
    pub explored: Vec<Vec<bool>>,
    next_monster_id: u64,
    next_item_id: u64,
}

impl Dungeon {
    /// Generate a dungeon for a given floor and seed
    pub fn generate(floor: u32, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed + floor as u64);
        let theme = DungeonTheme::for_floor(floor);

        // Dungeon size scales with floor
        let base_size = 40;
        let size_bonus = (floor as usize / 5) * 5;
        let width = base_size + size_bonus + rng.gen_range(0..10);
        let height = base_size + size_bonus + rng.gen_range(0..10);

        // Initialize with walls
        let mut tiles = vec![vec![Tile::Wall; width]; height];
        let mut rooms = Vec::new();
        let explored = vec![vec![false; width]; height];

        // Generate rooms using BSP
        let room_count = 5 + floor as usize / 2;
        let min_room_size = 4;
        let max_room_size = 10;

        for i in 0..room_count * 2 {
            // Try more times than needed
            let room_w = rng.gen_range(min_room_size..=max_room_size);
            let room_h = rng.gen_range(min_room_size..=max_room_size);
            let room_x = rng.gen_range(1..width - room_w - 1);
            let room_y = rng.gen_range(1..height - room_h - 1);

            let new_room = Room {
                id: i,
                room_type: RoomType::Standard,
                x: room_x,
                y: room_y,
                width: room_w,
                height: room_h,
                connected_to: Vec::new(),
            };

            // Check for overlap
            let overlaps = rooms.iter().any(|r: &Room| new_room.intersects(r));
            if !overlaps {
                rooms.push(new_room);
                if rooms.len() >= room_count {
                    break;
                }
            }
        }

        // Carve out rooms
        for room in &rooms {
            for y in room.y..room.y + room.height {
                for x in room.x..room.x + room.width {
                    tiles[y][x] = Tile::Floor;
                }
            }
        }

        // Connect rooms with corridors
        for i in 1..rooms.len() {
            let (x1, y1) = rooms[i - 1].center();
            let (x2, y2) = rooms[i].center();

            // L-shaped corridor
            if rng.gen_bool(0.5) {
                Self::carve_h_corridor(&mut tiles, x1, x2, y1);
                Self::carve_v_corridor(&mut tiles, y1, y2, x2);
            } else {
                Self::carve_v_corridor(&mut tiles, y1, y2, x1);
                Self::carve_h_corridor(&mut tiles, x1, x2, y2);
            }
        }

        // Assign room types
        let room_count = rooms.len();
        if room_count > 0 {
            rooms[0].room_type = RoomType::Start;
            rooms[room_count - 1].room_type = RoomType::Exit;

            // Boss room on boss floors
            if MonsterType::boss_for_floor(floor).is_some() && room_count > 2 {
                rooms[room_count - 2].room_type = RoomType::Boss;
            }

            // Random special rooms
            for i in 1..rooms.len().saturating_sub(2) {
                match rng.gen_range(0..10) {
                    0..=1 => rooms[i].room_type = RoomType::Treasure,
                    2 => rooms[i].room_type = RoomType::Shrine,
                    _ => {}
                }
            }
        }

        // Place stairs
        let start_pos = if !rooms.is_empty() {
            rooms[0].center()
        } else {
            (width / 2, height / 2)
        };

        let exit_pos = if !rooms.is_empty() {
            rooms[rooms.len() - 1].center()
        } else {
            (width / 2 + 5, height / 2 + 5)
        };

        if floor > 1 {
            tiles[start_pos.1][start_pos.0] = Tile::StairsUp;
        }
        tiles[exit_pos.1][exit_pos.0] = Tile::StairsDown;

        // Place shrines and chests
        for room in &rooms {
            match room.room_type {
                RoomType::Treasure => {
                    let (cx, cy) = room.center();
                    tiles[cy][cx] = Tile::Chest;
                }
                RoomType::Shrine => {
                    let (cx, cy) = room.center();
                    tiles[cy][cx] = Tile::Shrine;
                }
                _ => {}
            }
        }

        let mut dungeon = Dungeon {
            floor,
            theme,
            width,
            height,
            tiles,
            rooms,
            monsters: Vec::new(),
            items: Vec::new(),
            start_pos,
            exit_pos,
            explored,
            next_monster_id: 1,
            next_item_id: 1,
        };

        // Spawn monsters
        dungeon.spawn_monsters(&mut rng);

        dungeon
    }

    fn carve_h_corridor(tiles: &mut [Vec<Tile>], x1: usize, x2: usize, y: usize) {
        let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in start..=end {
            if y < tiles.len() && x < tiles[y].len() {
                tiles[y][x] = Tile::Floor;
            }
        }
    }

    fn carve_v_corridor(tiles: &mut [Vec<Tile>], y1: usize, y2: usize, x: usize) {
        let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in start..=end {
            if y < tiles.len() && x < tiles[y].len() {
                tiles[y][x] = Tile::Floor;
            }
        }
    }

    fn spawn_monsters(&mut self, rng: &mut ChaCha8Rng) {
        let monster_types = MonsterType::for_floor(self.floor);

        for room in &self.rooms {
            match room.room_type {
                RoomType::Start => continue, // No monsters in start room
                RoomType::Boss => {
                    // Spawn boss
                    if let Some(boss_type) = MonsterType::boss_for_floor(self.floor) {
                        let (cx, cy) = room.center();
                        self.monsters.push(Monster::new(
                            self.next_monster_id,
                            boss_type,
                            cx,
                            cy,
                        ));
                        self.next_monster_id += 1;
                    }
                }
                _ => {
                    // Spawn regular monsters
                    let monster_count = rng.gen_range(1..=3) + self.floor as usize / 5;
                    for _ in 0..monster_count {
                        let mx = room.x + rng.gen_range(1..room.width.saturating_sub(1).max(1));
                        let my = room.y + rng.gen_range(1..room.height.saturating_sub(1).max(1));

                        // Don't spawn on stairs
                        if (mx, my) == self.start_pos || (mx, my) == self.exit_pos {
                            continue;
                        }

                        let monster_type =
                            monster_types[rng.gen_range(0..monster_types.len())].clone();
                        self.monsters.push(Monster::new(
                            self.next_monster_id,
                            monster_type,
                            mx,
                            my,
                        ));
                        self.next_monster_id += 1;
                    }
                }
            }
        }
    }

    /// Check if a position is walkable
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if y >= self.height || x >= self.width {
            return false;
        }
        self.tiles[y][x].is_walkable()
    }

    /// Get tile at position
    pub fn get_tile(&self, x: usize, y: usize) -> Option<Tile> {
        if y >= self.height || x >= self.width {
            return None;
        }
        Some(self.tiles[y][x])
    }

    /// Get monster at position
    pub fn get_monster_at(&self, x: usize, y: usize) -> Option<&Monster> {
        self.monsters.iter().find(|m| m.x == x && m.y == y && m.is_alive())
    }

    /// Get mutable monster at position
    pub fn get_monster_at_mut(&mut self, x: usize, y: usize) -> Option<&mut Monster> {
        self.monsters.iter_mut().find(|m| m.x == x && m.y == y && m.is_alive())
    }

    /// Get monsters in radius
    pub fn get_monsters_in_radius(&self, x: usize, y: usize, radius: usize) -> Vec<&Monster> {
        self.monsters
            .iter()
            .filter(|m| {
                m.is_alive()
                    && (m.x as i32 - x as i32).unsigned_abs() as usize <= radius
                    && (m.y as i32 - y as i32).unsigned_abs() as usize <= radius
            })
            .collect()
    }

    /// Get items at position
    pub fn get_items_at(&self, x: usize, y: usize) -> Vec<&DroppedItem> {
        self.items.iter().filter(|i| i.x == x && i.y == y).collect()
    }

    /// Pick up item at position
    pub fn pickup_item(&mut self, x: usize, y: usize) -> Option<Item> {
        let idx = self.items.iter().position(|i| i.x == x && i.y == y)?;
        Some(self.items.remove(idx).item)
    }

    /// Drop item at position
    pub fn drop_item(&mut self, item: Item, x: usize, y: usize) {
        self.items.push(DroppedItem { item, x, y });
    }

    /// Spawn loot from killed monster
    pub fn spawn_loot(&mut self, x: usize, y: usize, luck_bonus: i32) {
        let mut rng = rand::thread_rng();

        // Chance to drop item
        if rng.gen_ratio(1, 4) {
            let item = Item::generate(self.next_item_id, self.floor, luck_bonus);
            self.next_item_id += 1;
            self.drop_item(item, x, y);
        }

        // Chance to drop potions
        if rng.gen_ratio(1, 3) {
            let is_health = rng.gen_bool(0.6);
            let potion = Item::generate_potion(self.next_item_id, is_health);
            self.next_item_id += 1;
            self.drop_item(potion, x, y);
        }
    }

    /// Mark area as explored (field of view)
    pub fn explore(&mut self, x: usize, y: usize, radius: usize) {
        let x = x as i32;
        let y = y as i32;
        let radius = radius as i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < self.width
                    && (ny as usize) < self.height
                {
                    // Simple circular FOV
                    if dx * dx + dy * dy <= radius * radius {
                        self.explored[ny as usize][nx as usize] = true;
                    }
                }
            }
        }
    }

    /// Check if position is explored
    pub fn is_explored(&self, x: usize, y: usize) -> bool {
        if y >= self.height || x >= self.width {
            return false;
        }
        self.explored[y][x]
    }

    /// Count alive monsters
    pub fn alive_monster_count(&self) -> usize {
        self.monsters.iter().filter(|m| m.is_alive()).count()
    }

    /// Check if floor is cleared (all monsters dead)
    pub fn is_cleared(&self) -> bool {
        self.alive_monster_count() == 0
    }
}

/// Generate daily seed from date
pub fn daily_seed() -> u64 {
    use chrono::{Datelike, Local};
    let now = Local::now();
    let date_num = now.year() as u64 * 10000 + now.month() as u64 * 100 + now.day() as u64;
    // Simple hash
    date_num.wrapping_mul(2654435761) ^ 0xDEADBEEF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_generation() {
        let dungeon = Dungeon::generate(1, 12345);
        assert!(dungeon.width >= 40);
        assert!(dungeon.height >= 40);
        assert!(!dungeon.rooms.is_empty());
    }

    #[test]
    fn test_dungeon_walkable() {
        let dungeon = Dungeon::generate(1, 12345);
        // Start position should be walkable
        assert!(dungeon.is_walkable(dungeon.start_pos.0, dungeon.start_pos.1));
    }

    #[test]
    fn test_theme_for_floor() {
        assert_eq!(DungeonTheme::for_floor(1), DungeonTheme::Cathedral);
        assert_eq!(DungeonTheme::for_floor(5), DungeonTheme::Cathedral);
        assert_eq!(DungeonTheme::for_floor(6), DungeonTheme::Catacombs);
        assert_eq!(DungeonTheme::for_floor(16), DungeonTheme::Hell);
    }

    #[test]
    fn test_monster_spawning() {
        let dungeon = Dungeon::generate(5, 12345);
        // Should have some monsters
        assert!(!dungeon.monsters.is_empty());

        // Boss floor should have boss
        let boss_dungeon = Dungeon::generate(5, 12345);
        let has_boss = boss_dungeon
            .monsters
            .iter()
            .any(|m| m.monster_type.is_boss());
        assert!(has_boss);
    }

    #[test]
    fn test_explore() {
        let mut dungeon = Dungeon::generate(1, 12345);
        let (sx, sy) = dungeon.start_pos;

        // Initially not explored
        assert!(!dungeon.is_explored(sx, sy));

        // Explore around start
        dungeon.explore(sx, sy, 5);

        // Now should be explored
        assert!(dungeon.is_explored(sx, sy));
    }

    #[test]
    fn test_daily_seed() {
        let seed1 = daily_seed();
        let seed2 = daily_seed();
        // Same day = same seed
        assert_eq!(seed1, seed2);
    }
}
