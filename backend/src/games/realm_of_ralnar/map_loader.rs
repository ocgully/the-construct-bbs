//! Map loading utilities for Realm of Ralnar
//!
//! Handles loading maps from JSON files and creating test maps.

use super::map::{
    Direction, EventEffect, EventTrigger, Map, MapConnection, MapEvent, MapType, NpcSpawn, Tile,
    TileAttributes,
};
use std::path::Path;

/// Map loader for reading map data from files
pub struct MapLoader;

impl MapLoader {
    /// Load a map from a JSON file
    pub fn load_from_json(path: &Path) -> Result<Map, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read map file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse map JSON: {}", e))
    }

    /// Load a map from a JSON string
    pub fn load_from_string(json: &str) -> Result<Map, String> {
        serde_json::from_str(json).map_err(|e| format!("Failed to parse map JSON: {}", e))
    }

    /// Save a map to a JSON file
    pub fn save_to_json(map: &Map, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(map)
            .map_err(|e| format!("Failed to serialize map: {}", e))?;

        std::fs::write(path, json).map_err(|e| format!("Failed to write map file: {}", e))
    }

    /// Create a simple test map for development
    pub fn create_test_map() -> Map {
        let mut tiles = vec![vec![Tile::grass(); 20]; 20];

        // Add some trees around the edges
        for i in 0..20 {
            tiles[0][i] = Tile::tree();
            tiles[19][i] = Tile::tree();
            tiles[i][0] = Tile::tree();
            tiles[i][19] = Tile::tree();
        }

        // Add a small pond
        tiles[5][5] = Tile::water();
        tiles[5][6] = Tile::water();
        tiles[6][5] = Tile::water();
        tiles[6][6] = Tile::water();

        // Add some mountains
        tiles[12][15] = Tile::mountain();
        tiles[13][15] = Tile::mountain();
        tiles[12][16] = Tile::mountain();

        Map {
            id: "test".to_string(),
            name: "Test Map".to_string(),
            width: 20,
            height: 20,
            tiles,
            npcs: vec![NpcSpawn {
                npc_id: "old_man".to_string(),
                x: 8,
                y: 8,
                direction: Direction::Down,
                stationary: true,
            }],
            events: vec![MapEvent {
                id: 1,
                x: 10,
                y: 10,
                trigger: EventTrigger::OnInteract,
                effect: EventEffect::SavePoint,
                one_time: false,
                flag_id: None,
            }],
            spawn_point: (10, 10),
            map_type: MapType::Town,
            encounter_rate: 0,
            enemy_table: vec![],
            music_id: Some("town_theme".to_string()),
            connections: vec![],
        }
    }

    /// Create a sample overworld map
    pub fn create_sample_overworld() -> Map {
        let width = 64;
        let height = 64;
        let mut tiles = vec![vec![Tile::grass(); width]; height];

        // Create terrain patterns
        for y in 0..height {
            for x in 0..width {
                // Ocean around edges
                if x < 3 || x >= width - 3 || y < 3 || y >= height - 3 {
                    tiles[y][x] = Tile::water();
                }
                // Mountain range in the middle
                else if y >= 20 && y <= 25 && x >= 10 && x <= 50 {
                    tiles[y][x] = Tile::mountain();
                }
                // Forest in the north
                else if y >= 5 && y <= 15 && x >= 15 && x <= 45 && (x + y) % 3 == 0 {
                    tiles[y][x] = Tile::tree();
                }
                // Desert in the south
                else if y >= 45 && y <= 55 && x >= 20 && x <= 40 {
                    tiles[y][x] = Tile {
                        base_id: 40,
                        overlay_id: None,
                        attributes: TileAttributes {
                            passable: true,
                            ..Default::default()
                        },
                    };
                }
                // Lake
                else if y >= 35 && y <= 42 && x >= 8 && x <= 18 {
                    tiles[y][x] = Tile::water();
                }
            }
        }

        // Clear paths through mountains
        for x in 28..32 {
            for y in 20..26 {
                tiles[y][x] = Tile::grass();
            }
        }

        Map {
            id: "overworld".to_string(),
            name: "Realm of Ralnar".to_string(),
            width: width as u32,
            height: height as u32,
            tiles,
            npcs: vec![],
            events: vec![
                // Town entrance
                MapEvent {
                    id: 1,
                    x: 30,
                    y: 30,
                    trigger: EventTrigger::OnStep,
                    effect: EventEffect::Teleport {
                        map: "starting_town".to_string(),
                        x: 10,
                        y: 18,
                    },
                    one_time: false,
                    flag_id: None,
                },
            ],
            spawn_point: (30, 32),
            map_type: MapType::Overworld,
            encounter_rate: 15,
            enemy_table: vec![
                "slime".to_string(),
                "goblin".to_string(),
                "wolf".to_string(),
            ],
            music_id: Some("overworld_theme".to_string()),
            connections: vec![],
        }
    }

    /// Create a sample town map
    pub fn create_sample_town() -> Map {
        let width = 24;
        let height = 20;
        let mut tiles = vec![vec![Tile::grass(); width]; height];

        // Ground/path tiles
        let path = Tile {
            base_id: 80,
            overlay_id: None,
            attributes: TileAttributes {
                passable: true,
                ..Default::default()
            },
        };

        // House tiles
        let house = Tile {
            base_id: 110,
            overlay_id: None,
            attributes: TileAttributes {
                passable: false,
                ..Default::default()
            },
        };

        // Main street (horizontal)
        for x in 0..width {
            tiles[10][x] = path.clone();
        }

        // Cross street (vertical)
        for y in 0..height {
            tiles[y][12] = path.clone();
        }

        // Buildings
        // Inn (top left)
        for y in 3..7 {
            for x in 3..8 {
                tiles[y][x] = house.clone();
            }
        }

        // Shop (top right)
        for y in 3..7 {
            for x in 16..21 {
                tiles[y][x] = house.clone();
            }
        }

        // Houses (bottom)
        for y in 13..17 {
            for x in 3..7 {
                tiles[y][x] = house.clone();
            }
        }

        for y in 13..17 {
            for x in 17..21 {
                tiles[y][x] = house.clone();
            }
        }

        // Fountain in center
        tiles[10][12] = Tile::water();

        // Trees around edges
        for i in 0..width {
            if tiles[0][i].attributes.passable && tiles[0][i].base_id != 80 {
                tiles[0][i] = Tile::tree();
            }
            if tiles[height - 1][i].attributes.passable && tiles[height - 1][i].base_id != 80 {
                tiles[height - 1][i] = Tile::tree();
            }
        }
        for i in 0..height {
            if tiles[i][0].attributes.passable && tiles[i][0].base_id != 80 {
                tiles[i][0] = Tile::tree();
            }
            if tiles[i][width - 1].attributes.passable && tiles[i][width - 1].base_id != 80 {
                tiles[i][width - 1] = Tile::tree();
            }
        }

        Map {
            id: "starting_town".to_string(),
            name: "Willowbrook Village".to_string(),
            width: width as u32,
            height: height as u32,
            tiles,
            npcs: vec![
                NpcSpawn {
                    npc_id: "innkeeper".to_string(),
                    x: 5,
                    y: 7,
                    direction: Direction::Down,
                    stationary: true,
                },
                NpcSpawn {
                    npc_id: "shopkeeper".to_string(),
                    x: 18,
                    y: 7,
                    direction: Direction::Down,
                    stationary: true,
                },
                NpcSpawn {
                    npc_id: "villager".to_string(),
                    x: 10,
                    y: 10,
                    direction: Direction::Right,
                    stationary: false,
                },
            ],
            events: vec![
                MapEvent {
                    id: 1,
                    x: 5,
                    y: 6,
                    trigger: EventTrigger::OnInteract,
                    effect: EventEffect::Dialogue {
                        dialogue_id: "inn_welcome".to_string(),
                    },
                    one_time: false,
                    flag_id: None,
                },
                MapEvent {
                    id: 2,
                    x: 18,
                    y: 6,
                    trigger: EventTrigger::OnInteract,
                    effect: EventEffect::Dialogue {
                        dialogue_id: "shop_browse".to_string(),
                    },
                    one_time: false,
                    flag_id: None,
                },
                MapEvent {
                    id: 3,
                    x: 10,
                    y: 10,
                    trigger: EventTrigger::OnInteract,
                    effect: EventEffect::SavePoint,
                    one_time: false,
                    flag_id: None,
                },
            ],
            spawn_point: (12, 18),
            map_type: MapType::Town,
            encounter_rate: 0,
            enemy_table: vec![],
            music_id: Some("village_theme".to_string()),
            connections: vec![MapConnection {
                direction: Direction::Down,
                x_range: (10, 14),
                y_range: (19, 19),
                target_map: "overworld".to_string(),
                target_x: 30,
                target_y: 30,
            }],
        }
    }

    /// Create a sample dungeon map
    pub fn create_sample_dungeon() -> Map {
        let width = 30;
        let height = 25;
        let mut tiles = vec![vec![Tile::wall(); width]; height];

        // Carve out rooms and corridors
        // Starting room
        for y in 2..8 {
            for x in 2..10 {
                tiles[y][x] = Tile::floor();
            }
        }

        // Corridor
        for x in 10..20 {
            tiles[5][x] = Tile::floor();
        }

        // Second room
        for y in 2..10 {
            for x in 20..28 {
                tiles[y][x] = Tile::floor();
            }
        }

        // Vertical corridor
        for y in 10..18 {
            tiles[y][24] = Tile::floor();
        }

        // Boss room
        for y in 18..23 {
            for x in 18..28 {
                tiles[y][x] = Tile::floor();
            }
        }

        // Treasure room
        for y in 12..16 {
            for x in 4..12 {
                tiles[y][x] = Tile::floor();
            }
        }

        // Corridor to treasure
        for y in 8..12 {
            tiles[y][6] = Tile::floor();
        }

        // Stairs
        tiles[4][4] = Tile {
            base_id: 251, // Stairs up
            overlay_id: None,
            attributes: TileAttributes {
                passable: true,
                ..Default::default()
            },
        };

        // Lava trap
        tiles[20][22] = Tile::lava();
        tiles[20][23] = Tile::lava();

        Map {
            id: "dark_cave".to_string(),
            name: "Dark Cave".to_string(),
            width: width as u32,
            height: height as u32,
            tiles,
            npcs: vec![],
            events: vec![
                // Entrance
                MapEvent {
                    id: 1,
                    x: 4,
                    y: 4,
                    trigger: EventTrigger::OnStep,
                    effect: EventEffect::Teleport {
                        map: "overworld".to_string(),
                        x: 45,
                        y: 22,
                    },
                    one_time: false,
                    flag_id: None,
                },
                // Treasure chest
                MapEvent {
                    id: 2,
                    x: 8,
                    y: 14,
                    trigger: EventTrigger::OnInteract,
                    effect: EventEffect::Chest {
                        item_id: "ancient_sword".to_string(),
                        quantity: 1,
                    },
                    one_time: true,
                    flag_id: Some("cave_chest_1".to_string()),
                },
                // Boss trigger
                MapEvent {
                    id: 3,
                    x: 23,
                    y: 20,
                    trigger: EventTrigger::OnStep,
                    effect: EventEffect::StartBattle {
                        enemies: vec!["cave_boss".to_string()],
                        boss: true,
                    },
                    one_time: true,
                    flag_id: Some("cave_boss_defeated".to_string()),
                },
            ],
            spawn_point: (4, 5),
            map_type: MapType::Dungeon,
            encounter_rate: 20,
            enemy_table: vec!["bat".to_string(), "skeleton".to_string()],
            music_id: Some("dungeon_theme".to_string()),
            connections: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_map() {
        let map = MapLoader::create_test_map();

        assert_eq!(map.id, "test");
        assert_eq!(map.width, 20);
        assert_eq!(map.height, 20);
        assert_eq!(map.tiles.len(), 20);
        assert_eq!(map.tiles[0].len(), 20);
        assert_eq!(map.spawn_point, (10, 10));
        assert_eq!(map.map_type, MapType::Town);

        // Check that edges are trees
        assert_eq!(map.tiles[0][5].base_id, 30); // Tree
        assert_eq!(map.tiles[19][5].base_id, 30);

        // Check pond
        assert_eq!(map.tiles[5][5].base_id, 10); // Water
    }

    #[test]
    fn test_create_sample_overworld() {
        let map = MapLoader::create_sample_overworld();

        assert_eq!(map.id, "overworld");
        assert_eq!(map.width, 64);
        assert_eq!(map.height, 64);
        assert_eq!(map.map_type, MapType::Overworld);
        assert!(map.encounter_rate > 0);
        assert!(!map.enemy_table.is_empty());

        // Check ocean border
        assert!(map.tiles[0][0].attributes.water);
        assert!(map.tiles[0][63].attributes.water);
    }

    #[test]
    fn test_create_sample_town() {
        let map = MapLoader::create_sample_town();

        assert_eq!(map.id, "starting_town");
        assert_eq!(map.map_type, MapType::Town);
        assert_eq!(map.encounter_rate, 0); // No encounters in town

        // Should have NPCs
        assert!(!map.npcs.is_empty());

        // Should have events
        assert!(!map.events.is_empty());

        // Should have connection back to overworld
        assert!(!map.connections.is_empty());
    }

    #[test]
    fn test_create_sample_dungeon() {
        let map = MapLoader::create_sample_dungeon();

        assert_eq!(map.id, "dark_cave");
        assert_eq!(map.map_type, MapType::Dungeon);
        assert!(map.encounter_rate > 0);

        // Should have walls and floors
        let wall_count = map
            .tiles
            .iter()
            .flatten()
            .filter(|t| t.base_id == 210)
            .count();
        let floor_count = map
            .tiles
            .iter()
            .flatten()
            .filter(|t| t.base_id == 200)
            .count();

        assert!(wall_count > 0);
        assert!(floor_count > 0);

        // Should have events (chest, boss, entrance)
        assert!(map.events.len() >= 3);
    }

    #[test]
    fn test_load_from_string() {
        let map = MapLoader::create_test_map();
        let json = serde_json::to_string(&map).unwrap();

        let loaded = MapLoader::load_from_string(&json).unwrap();

        assert_eq!(loaded.id, map.id);
        assert_eq!(loaded.width, map.width);
        assert_eq!(loaded.height, map.height);
    }

    #[test]
    fn test_load_invalid_json() {
        let result = MapLoader::load_from_string("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_map_serialization_roundtrip() {
        let original = MapLoader::create_sample_town();
        let json = serde_json::to_string_pretty(&original).unwrap();
        let restored = MapLoader::load_from_string(&json).unwrap();

        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.width, restored.width);
        assert_eq!(original.height, restored.height);
        assert_eq!(original.spawn_point, restored.spawn_point);
        assert_eq!(original.map_type, restored.map_type);
        assert_eq!(original.encounter_rate, restored.encounter_rate);
        assert_eq!(original.npcs.len(), restored.npcs.len());
        assert_eq!(original.events.len(), restored.events.len());
        assert_eq!(original.connections.len(), restored.connections.len());
    }

    #[test]
    fn test_all_sample_maps_are_navigable() {
        let maps = vec![
            MapLoader::create_test_map(),
            MapLoader::create_sample_overworld(),
            MapLoader::create_sample_town(),
            MapLoader::create_sample_dungeon(),
        ];

        for map in maps {
            // Spawn point should be passable
            let (sx, sy) = map.spawn_point;
            let spawn_tile = map.get_tile(sx as i32, sy as i32);
            assert!(
                spawn_tile.is_some(),
                "Map {} has no tile at spawn point",
                map.id
            );
            assert!(
                spawn_tile.unwrap().attributes.passable,
                "Map {} spawn point is not passable",
                map.id
            );
        }
    }

    #[test]
    fn test_dungeon_has_lava_damage_tiles() {
        let map = MapLoader::create_sample_dungeon();

        let lava_tiles: Vec<_> = map
            .tiles
            .iter()
            .flatten()
            .filter(|t| t.attributes.damage > 0)
            .collect();

        assert!(!lava_tiles.is_empty(), "Dungeon should have damage tiles");
    }

    #[test]
    fn test_town_has_fountain() {
        let map = MapLoader::create_sample_town();

        // There should be a water tile (fountain)
        let water_tiles: Vec<_> = map
            .tiles
            .iter()
            .flatten()
            .filter(|t| t.attributes.water)
            .collect();

        assert!(!water_tiles.is_empty(), "Town should have a fountain");
    }

    #[test]
    fn test_events_have_unique_ids() {
        let map = MapLoader::create_sample_dungeon();

        let mut ids: Vec<u32> = map.events.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();

        assert_eq!(
            ids.len(),
            map.events.len(),
            "Event IDs should be unique"
        );
    }

    #[test]
    fn test_boss_event_is_one_time() {
        let map = MapLoader::create_sample_dungeon();

        let boss_event = map.events.iter().find(|e| {
            matches!(
                &e.effect,
                EventEffect::StartBattle { boss: true, .. }
            )
        });

        assert!(boss_event.is_some(), "Should have a boss event");
        assert!(
            boss_event.unwrap().one_time,
            "Boss event should be one-time"
        );
        assert!(
            boss_event.unwrap().flag_id.is_some(),
            "Boss event should have a flag ID"
        );
    }
}
