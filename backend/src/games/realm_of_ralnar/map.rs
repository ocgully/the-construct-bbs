//! Map data structures for Realm of Ralnar
//!
//! Defines the tile-based map system including terrain, events, and NPCs.

use serde::{Deserialize, Serialize};

// Re-export Direction from state for convenience
pub use super::state::Direction;

/// A game map containing tiles, NPCs, and events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    /// Unique map identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Width in tiles
    pub width: u32,
    /// Height in tiles
    pub height: u32,
    /// 2D tile grid (row-major: tiles[y][x])
    pub tiles: Vec<Vec<Tile>>,
    /// NPCs spawned on this map
    pub npcs: Vec<NpcSpawn>,
    /// Triggered events on this map
    pub events: Vec<MapEvent>,
    /// Player spawn point (x, y)
    pub spawn_point: (u32, u32),
    /// Type of map (affects behavior)
    pub map_type: MapType,
    /// Random encounter rate (0-100)
    pub encounter_rate: u8,
    /// Enemy IDs that can spawn here
    pub enemy_table: Vec<String>,
    /// Background music ID
    pub music_id: Option<String>,
    /// Connections to other maps
    pub connections: Vec<MapConnection>,
}

/// A single tile on the map
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    /// Base tile ID for appearance
    pub base_id: u16,
    /// Optional overlay tile ID
    pub overlay_id: Option<u16>,
    /// Tile properties
    pub attributes: TileAttributes,
}

/// Properties that affect tile behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TileAttributes {
    /// Can walk on this tile
    pub passable: bool,
    /// Requires ship to cross
    pub water: bool,
    /// Requires airship to cross
    pub air_only: bool,
    /// Damage per step (lava, poison swamps, etc.)
    pub damage: u8,
    /// Associated event ID
    pub event_id: Option<u32>,
}

/// Type of map, affects wrapping and behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MapType {
    /// World map with wrapping edges
    Overworld,
    /// Town area
    Town,
    /// Dungeon with no random encounters on most tiles
    Dungeon,
    /// Interior building
    Interior,
    /// Sacred shrine area
    Shrine,
}

/// Connection to another map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConnection {
    /// Direction player must be moving
    pub direction: Direction,
    /// X coordinate range for this connection
    pub x_range: (u32, u32),
    /// Y coordinate range for this connection
    pub y_range: (u32, u32),
    /// Target map ID
    pub target_map: String,
    /// Target X coordinate
    pub target_x: u32,
    /// Target Y coordinate
    pub target_y: u32,
}

/// NPC spawn point on a map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSpawn {
    /// NPC ID
    pub npc_id: String,
    /// X position
    pub x: u32,
    /// Y position
    pub y: u32,
    /// Direction NPC is facing
    pub direction: Direction,
    /// Whether NPC can move around
    pub stationary: bool,
}

/// A triggered event on the map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapEvent {
    /// Unique event ID
    pub id: u32,
    /// X position
    pub x: u32,
    /// Y position
    pub y: u32,
    /// What triggers this event
    pub trigger: EventTrigger,
    /// What happens when triggered
    pub effect: EventEffect,
    /// Whether this event only triggers once
    pub one_time: bool,
    /// Flag ID for one-time events (tracks if already triggered)
    pub flag_id: Option<String>,
}

/// What triggers a map event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventTrigger {
    /// Triggered by stepping on the tile
    OnStep,
    /// Triggered by pressing interact key
    OnInteract,
    /// Triggered when entering the map
    OnEnter,
}

/// What happens when an event triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEffect {
    /// Teleport to another location
    Teleport { map: String, x: u32, y: u32 },
    /// Start a battle
    StartBattle { enemies: Vec<String>, boss: bool },
    /// Show dialogue
    Dialogue { dialogue_id: String },
    /// Open a treasure chest
    Chest { item_id: String, quantity: u32 },
    /// Save point
    SavePoint,
    /// Heal party
    Heal,
    /// Play a cutscene
    Cutscene { scene_id: String },
    /// Set a game flag
    SetFlag { flag: String, value: bool },
}

impl Map {
    /// Create a new empty map
    pub fn new(id: String, width: u32, height: u32) -> Self {
        let tiles = vec![vec![Tile::grass(); width as usize]; height as usize];
        Self {
            id,
            name: "Unnamed Map".to_string(),
            width,
            height,
            tiles,
            npcs: vec![],
            events: vec![],
            spawn_point: (width / 2, height / 2),
            map_type: MapType::Town,
            encounter_rate: 0,
            enemy_table: vec![],
            music_id: None,
            connections: vec![],
        }
    }

    /// Get a tile at coordinates, handling world wrap for overworld
    pub fn get_tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if self.map_type == MapType::Overworld {
            // World wrap
            let wx = ((x % self.width as i32) + self.width as i32) % self.width as i32;
            let wy = ((y % self.height as i32) + self.height as i32) % self.height as i32;
            Some(&self.tiles[wy as usize][wx as usize])
        } else {
            // Bounded
            if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                Some(&self.tiles[y as usize][x as usize])
            } else {
                None
            }
        }
    }

    /// Check if a position is passable for the given movement mode
    pub fn is_passable(&self, x: i32, y: i32, mode: MovementMode) -> bool {
        match self.get_tile(x, y) {
            Some(tile) => match mode {
                MovementMode::Walking => tile.attributes.passable && !tile.attributes.water,
                MovementMode::Ship => tile.attributes.water,
                MovementMode::Airship => true, // Airship can fly over anything
            },
            None => false,
        }
    }

    /// Get an event at the given position (if any)
    pub fn get_event_at(&self, x: u32, y: u32) -> Option<&MapEvent> {
        self.events.iter().find(|e| e.x == x && e.y == y)
    }

    /// Get events that trigger on step at the given position
    pub fn get_step_events_at(&self, x: u32, y: u32) -> Vec<&MapEvent> {
        self.events
            .iter()
            .filter(|e| e.x == x && e.y == y && e.trigger == EventTrigger::OnStep)
            .collect()
    }

    /// Get a map connection at the edge position moving in a direction
    pub fn get_connection(&self, x: u32, y: u32, direction: Direction) -> Option<&MapConnection> {
        self.connections.iter().find(|c| {
            c.direction == direction
                && x >= c.x_range.0
                && x <= c.x_range.1
                && y >= c.y_range.0
                && y <= c.y_range.1
        })
    }

    /// Roll for a random encounter
    pub fn check_random_encounter(&self) -> Option<Vec<String>> {
        if self.encounter_rate == 0 || self.enemy_table.is_empty() {
            return None;
        }

        // Random roll
        let roll: u8 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            % 100) as u8;

        if roll < self.encounter_rate {
            // Select 1-3 enemies from the table
            let enemy_count = (roll % 3) as usize + 1;
            let enemies: Vec<String> = self
                .enemy_table
                .iter()
                .cycle()
                .take(enemy_count.min(self.enemy_table.len()))
                .cloned()
                .collect();
            Some(enemies)
        } else {
            None
        }
    }

    /// Get NPC at a position
    pub fn get_npc_at(&self, x: u32, y: u32) -> Option<&NpcSpawn> {
        self.npcs.iter().find(|npc| npc.x == x && npc.y == y)
    }
}

impl Tile {
    /// Create a grass tile
    pub fn grass() -> Self {
        Tile {
            base_id: 0,
            overlay_id: None,
            attributes: TileAttributes {
                passable: true,
                ..Default::default()
            },
        }
    }

    /// Create a tree tile
    pub fn tree() -> Self {
        Tile {
            base_id: 30,
            overlay_id: None,
            attributes: TileAttributes {
                passable: false,
                ..Default::default()
            },
        }
    }

    /// Create a water tile
    pub fn water() -> Self {
        Tile {
            base_id: 10,
            overlay_id: None,
            attributes: TileAttributes {
                passable: true,
                water: true,
                ..Default::default()
            },
        }
    }

    /// Create a mountain tile
    pub fn mountain() -> Self {
        Tile {
            base_id: 20,
            overlay_id: None,
            attributes: TileAttributes {
                passable: false,
                ..Default::default()
            },
        }
    }

    /// Create a lava tile
    pub fn lava() -> Self {
        Tile {
            base_id: 50,
            overlay_id: None,
            attributes: TileAttributes {
                passable: true,
                damage: 10,
                ..Default::default()
            },
        }
    }

    /// Create a floor tile
    pub fn floor() -> Self {
        Tile {
            base_id: 200,
            overlay_id: None,
            attributes: TileAttributes {
                passable: true,
                ..Default::default()
            },
        }
    }

    /// Create a wall tile
    pub fn wall() -> Self {
        Tile {
            base_id: 210,
            overlay_id: None,
            attributes: TileAttributes {
                passable: false,
                ..Default::default()
            },
        }
    }
}

/// Mode of player movement
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MovementMode {
    /// Walking on foot
    #[default]
    Walking,
    /// Sailing on water
    Ship,
    /// Flying anywhere
    Airship,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let map = Map::new("test".to_string(), 20, 15);
        assert_eq!(map.id, "test");
        assert_eq!(map.width, 20);
        assert_eq!(map.height, 15);
        assert_eq!(map.tiles.len(), 15);
        assert_eq!(map.tiles[0].len(), 20);
    }

    #[test]
    fn test_get_tile_bounded() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = MapType::Town;
        map.tiles[5][5] = Tile::tree();

        // Valid position
        let tile = map.get_tile(5, 5);
        assert!(tile.is_some());
        assert_eq!(tile.unwrap().base_id, 30);

        // Out of bounds
        assert!(map.get_tile(-1, 0).is_none());
        assert!(map.get_tile(0, -1).is_none());
        assert!(map.get_tile(10, 5).is_none());
        assert!(map.get_tile(5, 10).is_none());
    }

    #[test]
    fn test_get_tile_overworld_wrap() {
        let mut map = Map::new("world".to_string(), 10, 10);
        map.map_type = MapType::Overworld;
        map.tiles[0][0] = Tile::tree();

        // Normal access
        let tile = map.get_tile(0, 0);
        assert!(tile.is_some());
        assert_eq!(tile.unwrap().base_id, 30);

        // Wrap positive
        let tile = map.get_tile(10, 10);
        assert!(tile.is_some());
        assert_eq!(tile.unwrap().base_id, 30);

        // Wrap negative
        let tile = map.get_tile(-10, -10);
        assert!(tile.is_some());
        assert_eq!(tile.unwrap().base_id, 30);
    }

    #[test]
    fn test_world_wrap_coordinates() {
        let mut map = Map::new("world".to_string(), 100, 100);
        map.map_type = MapType::Overworld;

        // Mark a specific tile
        map.tiles[50][50] = Tile::tree();

        // Access via wrapping
        assert_eq!(map.get_tile(50, 50).unwrap().base_id, 30);
        assert_eq!(map.get_tile(150, 150).unwrap().base_id, 30);
        assert_eq!(map.get_tile(-50, -50).unwrap().base_id, 30);
        assert_eq!(map.get_tile(250, 250).unwrap().base_id, 30);
    }

    #[test]
    fn test_is_passable_walking() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = MapType::Town;
        map.tiles[0][0] = Tile::grass();
        map.tiles[1][0] = Tile::tree();
        map.tiles[2][0] = Tile::water();

        assert!(map.is_passable(0, 0, MovementMode::Walking));
        assert!(!map.is_passable(0, 1, MovementMode::Walking)); // Tree blocks
        assert!(!map.is_passable(0, 2, MovementMode::Walking)); // Water blocks walking
    }

    #[test]
    fn test_is_passable_ship() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = MapType::Town;
        map.tiles[0][0] = Tile::grass();
        map.tiles[1][0] = Tile::water();

        assert!(!map.is_passable(0, 0, MovementMode::Ship)); // Grass blocks ship
        assert!(map.is_passable(0, 1, MovementMode::Ship)); // Water is passable
    }

    #[test]
    fn test_is_passable_airship() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = MapType::Town;
        map.tiles[0][0] = Tile::grass();
        map.tiles[1][0] = Tile::tree();
        map.tiles[2][0] = Tile::water();
        map.tiles[3][0] = Tile::mountain();

        // Airship can fly over everything
        assert!(map.is_passable(0, 0, MovementMode::Airship));
        assert!(map.is_passable(0, 1, MovementMode::Airship));
        assert!(map.is_passable(0, 2, MovementMode::Airship));
        assert!(map.is_passable(0, 3, MovementMode::Airship));
    }

    #[test]
    fn test_get_event_at() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.events.push(MapEvent {
            id: 1,
            x: 5,
            y: 5,
            trigger: EventTrigger::OnStep,
            effect: EventEffect::SavePoint,
            one_time: false,
            flag_id: None,
        });

        assert!(map.get_event_at(5, 5).is_some());
        assert_eq!(map.get_event_at(5, 5).unwrap().id, 1);
        assert!(map.get_event_at(0, 0).is_none());
    }

    #[test]
    fn test_get_connection() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.connections.push(MapConnection {
            direction: Direction::Up,
            x_range: (0, 9),
            y_range: (0, 0),
            target_map: "other_map".to_string(),
            target_x: 5,
            target_y: 9,
        });

        // Should find connection when at top edge moving up
        let conn = map.get_connection(5, 0, Direction::Up);
        assert!(conn.is_some());
        assert_eq!(conn.unwrap().target_map, "other_map");

        // Should not find when moving other directions
        assert!(map.get_connection(5, 0, Direction::Down).is_none());
        assert!(map.get_connection(5, 0, Direction::Left).is_none());

        // Should not find when at wrong position
        assert!(map.get_connection(5, 5, Direction::Up).is_none());
    }

    #[test]
    fn test_get_npc_at() {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.npcs.push(NpcSpawn {
            npc_id: "shopkeeper".to_string(),
            x: 3,
            y: 4,
            direction: Direction::Down,
            stationary: true,
        });

        assert!(map.get_npc_at(3, 4).is_some());
        assert_eq!(map.get_npc_at(3, 4).unwrap().npc_id, "shopkeeper");
        assert!(map.get_npc_at(0, 0).is_none());
    }

    #[test]
    fn test_direction_delta() {
        // Uses delta() from state::Direction
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn test_tile_constructors() {
        let grass = Tile::grass();
        assert!(grass.attributes.passable);
        assert!(!grass.attributes.water);

        let tree = Tile::tree();
        assert!(!tree.attributes.passable);

        let water = Tile::water();
        assert!(water.attributes.passable);
        assert!(water.attributes.water);

        let mountain = Tile::mountain();
        assert!(!mountain.attributes.passable);

        let lava = Tile::lava();
        assert!(lava.attributes.passable);
        assert_eq!(lava.attributes.damage, 10);

        let floor = Tile::floor();
        assert!(floor.attributes.passable);

        let wall = Tile::wall();
        assert!(!wall.attributes.passable);
    }

    #[test]
    fn test_movement_mode_default() {
        let mode: MovementMode = Default::default();
        assert_eq!(mode, MovementMode::Walking);
    }

    #[test]
    fn test_tile_serialization() {
        let tile = Tile::grass();
        let json = serde_json::to_string(&tile).unwrap();
        let restored: Tile = serde_json::from_str(&json).unwrap();
        assert_eq!(tile.base_id, restored.base_id);
        assert_eq!(tile.attributes.passable, restored.attributes.passable);
    }

    #[test]
    fn test_map_serialization() {
        let map = Map::new("test".to_string(), 5, 5);
        let json = serde_json::to_string(&map).unwrap();
        let restored: Map = serde_json::from_str(&json).unwrap();
        assert_eq!(map.id, restored.id);
        assert_eq!(map.width, restored.width);
        assert_eq!(map.height, restored.height);
    }
}
