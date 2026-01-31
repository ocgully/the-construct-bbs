//! Player movement system for Realm of Ralnar
//!
//! Handles grid-based movement, collision detection, and movement-triggered events.

use super::map::{EventTrigger, Map, MapType, MovementMode, Tile};
use super::state::Direction;
use serde::{Deserialize, Serialize};

/// Result of attempting to move
#[derive(Debug, Clone, PartialEq)]
pub enum MovementResult {
    /// Movement succeeded
    Success,
    /// Movement blocked by terrain
    Blocked,
    /// Movement blocked by an NPC
    NpcBlocking(String),
    /// Triggered a map event
    TriggerEvent(u32),
    /// Transition to another map
    MapTransition {
        map_id: String,
        x: u32,
        y: u32,
    },
    /// Encountered random enemies
    RandomEncounter(Vec<String>),
    /// Stepped on a damage tile
    DamageTile(u8),
}

/// Player position state for movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPosition {
    /// Current X coordinate
    pub x: u32,
    /// Current Y coordinate
    pub y: u32,
    /// Direction player is facing
    pub direction: Direction,
    /// Current movement mode
    pub movement_mode: MovementMode,
}

impl PlayerPosition {
    /// Create a new player position
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            direction: Direction::Down,
            movement_mode: MovementMode::Walking,
        }
    }
}

impl Default for PlayerPosition {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Movement system for handling player movement
pub struct MovementSystem;

impl MovementSystem {
    /// Try to move the player in a direction
    pub fn try_move(
        position: &mut PlayerPosition,
        map: &Map,
        direction: Direction,
    ) -> MovementResult {
        let (dx, dy) = direction.delta();
        let new_x = position.x as i32 + dx;
        let new_y = position.y as i32 + dy;

        // Check for map connection first (before bounds check) when moving out of bounds
        // This allows transitions at map edges even when the target tile is out of bounds
        if let Some(connection) = map.get_connection(position.x, position.y, direction) {
            // Check if we're at the edge and should transition
            let at_edge = match direction {
                Direction::Up => new_y < 0 || (map.map_type != MapType::Overworld && new_y == 0),
                Direction::Down => {
                    new_y >= map.height as i32
                        || (map.map_type != MapType::Overworld && new_y == map.height as i32 - 1)
                }
                Direction::Left => new_x < 0 || (map.map_type != MapType::Overworld && new_x == 0),
                Direction::Right => {
                    new_x >= map.width as i32
                        || (map.map_type != MapType::Overworld && new_x == map.width as i32 - 1)
                }
            };

            if at_edge || map.map_type != MapType::Overworld {
                return MovementResult::MapTransition {
                    map_id: connection.target_map.clone(),
                    x: connection.target_x,
                    y: connection.target_y,
                };
            }
        }

        // Check map bounds (with wrap for overworld)
        let tile = match map.get_tile(new_x, new_y) {
            Some(t) => t,
            None => return MovementResult::Blocked,
        };

        // Check passability based on movement mode
        if !Self::can_pass(tile, position.movement_mode) {
            return MovementResult::Blocked;
        }

        // Check for NPCs blocking
        let final_x = Self::wrap_coordinate(new_x, map.width, map.map_type);
        let final_y = Self::wrap_coordinate(new_y, map.height, map.map_type);

        if let Some(npc) = map.get_npc_at(final_x, final_y) {
            return MovementResult::NpcBlocking(npc.npc_id.clone());
        }

        // Apply movement
        position.direction = direction;
        position.x = final_x;
        position.y = final_y;

        // Check for events
        for event in map.get_step_events_at(position.x, position.y) {
            if event.trigger == EventTrigger::OnStep {
                return MovementResult::TriggerEvent(event.id);
            }
        }

        // Check for random encounter
        if let Some(enemies) = map.check_random_encounter() {
            return MovementResult::RandomEncounter(enemies);
        }

        // Check for damage tiles
        if tile.attributes.damage > 0 {
            return MovementResult::DamageTile(tile.attributes.damage);
        }

        MovementResult::Success
    }

    /// Check if a tile can be passed with the given movement mode
    fn can_pass(tile: &Tile, mode: MovementMode) -> bool {
        match mode {
            MovementMode::Walking => tile.attributes.passable && !tile.attributes.water,
            MovementMode::Ship => tile.attributes.water,
            MovementMode::Airship => true, // Can fly over anything
        }
    }

    /// Wrap a coordinate for overworld, or clamp for bounded maps
    fn wrap_coordinate(coord: i32, size: u32, map_type: MapType) -> u32 {
        if map_type == MapType::Overworld {
            ((coord % size as i32) + size as i32) as u32 % size
        } else {
            coord.max(0) as u32
        }
    }

    /// Turn player to face a direction without moving
    pub fn turn(position: &mut PlayerPosition, direction: Direction) {
        position.direction = direction;
    }

    /// Check if player can interact with something in front of them
    pub fn get_interaction_target(position: &PlayerPosition, map: &Map) -> Option<InteractionTarget> {
        let (dx, dy) = position.direction.delta();
        let target_x = position.x as i32 + dx;
        let target_y = position.y as i32 + dy;

        // Wrap coordinates for overworld
        let target_x = Self::wrap_coordinate(target_x, map.width, map.map_type);
        let target_y = Self::wrap_coordinate(target_y, map.height, map.map_type);

        // Check for NPC
        if let Some(npc) = map.get_npc_at(target_x, target_y) {
            return Some(InteractionTarget::Npc(npc.npc_id.clone()));
        }

        // Check for interact event
        if let Some(event) = map.get_event_at(target_x, target_y) {
            if event.trigger == EventTrigger::OnInteract {
                return Some(InteractionTarget::Event(event.id));
            }
        }

        None
    }
}

/// What the player can interact with
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionTarget {
    /// An NPC
    Npc(String),
    /// A map event
    Event(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::realm_of_ralnar::map::{MapEvent, EventEffect, NpcSpawn, MapConnection};

    fn create_test_map() -> Map {
        let mut map = Map::new("test".to_string(), 10, 10);
        map.map_type = MapType::Town;

        // Add a tree at (5, 3)
        map.tiles[3][5] = Tile::tree();

        // Add water at (7, 5)
        map.tiles[5][7] = Tile::water();

        map
    }

    #[test]
    fn test_move_success() {
        let map = create_test_map();
        let mut pos = PlayerPosition::new(5, 5);

        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 4);
        assert_eq!(pos.direction, Direction::Up);
    }

    #[test]
    fn test_move_all_directions() {
        let map = create_test_map();

        // Test moving up
        let mut pos = PlayerPosition::new(2, 2);
        MovementSystem::try_move(&mut pos, &map, Direction::Up);
        assert_eq!((pos.x, pos.y), (2, 1));

        // Test moving down
        let mut pos = PlayerPosition::new(2, 2);
        MovementSystem::try_move(&mut pos, &map, Direction::Down);
        assert_eq!((pos.x, pos.y), (2, 3));

        // Test moving left
        let mut pos = PlayerPosition::new(2, 2);
        MovementSystem::try_move(&mut pos, &map, Direction::Left);
        assert_eq!((pos.x, pos.y), (1, 2));

        // Test moving right
        let mut pos = PlayerPosition::new(2, 2);
        MovementSystem::try_move(&mut pos, &map, Direction::Right);
        assert_eq!((pos.x, pos.y), (3, 2));
    }

    #[test]
    fn test_move_blocked_by_terrain() {
        let map = create_test_map();
        // Tree is at (5, 3), try to move into it from (5, 4)
        let mut pos = PlayerPosition::new(5, 4);

        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);
        assert_eq!(result, MovementResult::Blocked);
        // Position should not change
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 4);
    }

    #[test]
    fn test_move_blocked_by_water_when_walking() {
        let map = create_test_map();
        // Water is at (7, 5), try to move into it from (6, 5)
        let mut pos = PlayerPosition::new(6, 5);

        let result = MovementSystem::try_move(&mut pos, &map, Direction::Right);
        assert_eq!(result, MovementResult::Blocked);
        assert_eq!(pos.x, 6);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_move_blocked_at_edge() {
        let map = create_test_map();
        let mut pos = PlayerPosition::new(0, 5);

        let result = MovementSystem::try_move(&mut pos, &map, Direction::Left);
        assert_eq!(result, MovementResult::Blocked);
        assert_eq!(pos.x, 0);
    }

    #[test]
    fn test_move_npc_blocking() {
        let mut map = create_test_map();
        map.npcs.push(NpcSpawn {
            npc_id: "guard".to_string(),
            x: 4,
            y: 5,
            direction: Direction::Down,
            stationary: true,
        });

        let mut pos = PlayerPosition::new(5, 5);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Left);

        assert_eq!(result, MovementResult::NpcBlocking("guard".to_string()));
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_move_triggers_event() {
        let mut map = create_test_map();
        map.events.push(MapEvent {
            id: 42,
            x: 5,
            y: 4,
            trigger: EventTrigger::OnStep,
            effect: EventEffect::SavePoint,
            one_time: false,
            flag_id: None,
        });

        let mut pos = PlayerPosition::new(5, 5);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);

        assert_eq!(result, MovementResult::TriggerEvent(42));
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 4);
    }

    #[test]
    fn test_move_damage_tile() {
        let mut map = create_test_map();
        map.tiles[4][5] = Tile::lava();

        let mut pos = PlayerPosition::new(5, 5);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);

        assert_eq!(result, MovementResult::DamageTile(10));
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 4);
    }

    #[test]
    fn test_map_transition() {
        let mut map = create_test_map();
        map.connections.push(MapConnection {
            direction: Direction::Up,
            x_range: (0, 9),
            y_range: (0, 0),
            target_map: "dungeon".to_string(),
            target_x: 5,
            target_y: 9,
        });

        let mut pos = PlayerPosition::new(5, 0);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);

        match result {
            MovementResult::MapTransition { map_id, x, y } => {
                assert_eq!(map_id, "dungeon");
                assert_eq!(x, 5);
                assert_eq!(y, 9);
            }
            _ => panic!("Expected MapTransition, got {:?}", result),
        }
    }

    #[test]
    fn test_overworld_wrap_movement() {
        let mut map = Map::new("world".to_string(), 10, 10);
        map.map_type = MapType::Overworld;

        // Move left from x=0 should wrap to x=9
        let mut pos = PlayerPosition::new(0, 5);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Left);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.x, 9);

        // Move right from x=9 should wrap to x=0
        let mut pos = PlayerPosition::new(9, 5);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Right);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.x, 0);

        // Move up from y=0 should wrap to y=9
        let mut pos = PlayerPosition::new(5, 0);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.y, 9);

        // Move down from y=9 should wrap to y=0
        let mut pos = PlayerPosition::new(5, 9);
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Down);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_ship_movement_on_water() {
        let mut map = create_test_map();
        // Make a water area
        map.tiles[5][5] = Tile::water();
        map.tiles[5][6] = Tile::water();

        let mut pos = PlayerPosition::new(5, 5);
        pos.movement_mode = MovementMode::Ship;

        // Can move to adjacent water
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Right);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.x, 6);
    }

    #[test]
    fn test_ship_blocked_by_land() {
        let mut map = create_test_map();
        map.tiles[5][5] = Tile::water();
        map.tiles[5][6] = Tile::grass();

        let mut pos = PlayerPosition::new(5, 5);
        pos.movement_mode = MovementMode::Ship;

        // Ship can't go on land
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Right);
        assert_eq!(result, MovementResult::Blocked);
    }

    #[test]
    fn test_airship_can_fly_anywhere() {
        let map = create_test_map();
        let mut pos = PlayerPosition::new(5, 5);
        pos.movement_mode = MovementMode::Airship;

        // Can fly over tree at (5, 3) - from (5, 4) go up
        let mut pos = PlayerPosition::new(5, 4);
        pos.movement_mode = MovementMode::Airship;
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Up);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.y, 3);

        // Can fly over water at (7, 5)
        let mut pos = PlayerPosition::new(6, 5);
        pos.movement_mode = MovementMode::Airship;
        let result = MovementSystem::try_move(&mut pos, &map, Direction::Right);
        assert_eq!(result, MovementResult::Success);
        assert_eq!(pos.x, 7);
    }

    #[test]
    fn test_turn_without_moving() {
        let mut pos = PlayerPosition::new(5, 5);
        pos.direction = Direction::Down;

        MovementSystem::turn(&mut pos, Direction::Left);
        assert_eq!(pos.direction, Direction::Left);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_interaction_target_npc() {
        let mut map = create_test_map();
        map.npcs.push(NpcSpawn {
            npc_id: "shopkeeper".to_string(),
            x: 5,
            y: 4,
            direction: Direction::Down,
            stationary: true,
        });

        let mut pos = PlayerPosition::new(5, 5);
        pos.direction = Direction::Up;

        let target = MovementSystem::get_interaction_target(&pos, &map);
        assert_eq!(target, Some(InteractionTarget::Npc("shopkeeper".to_string())));
    }

    #[test]
    fn test_interaction_target_event() {
        let mut map = create_test_map();
        map.events.push(MapEvent {
            id: 99,
            x: 5,
            y: 4,
            trigger: EventTrigger::OnInteract,
            effect: EventEffect::Chest {
                item_id: "gold".to_string(),
                quantity: 100,
            },
            one_time: true,
            flag_id: Some("chest_opened".to_string()),
        });

        let mut pos = PlayerPosition::new(5, 5);
        pos.direction = Direction::Up;

        let target = MovementSystem::get_interaction_target(&pos, &map);
        assert_eq!(target, Some(InteractionTarget::Event(99)));
    }

    #[test]
    fn test_interaction_target_none() {
        let map = create_test_map();
        let mut pos = PlayerPosition::new(5, 5);
        pos.direction = Direction::Up;

        let target = MovementSystem::get_interaction_target(&pos, &map);
        assert_eq!(target, None);
    }

    #[test]
    fn test_player_position_default() {
        let pos: PlayerPosition = Default::default();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
        assert_eq!(pos.direction, Direction::Down);
        assert_eq!(pos.movement_mode, MovementMode::Walking);
    }

    #[test]
    fn test_player_position_serialization() {
        let pos = PlayerPosition {
            x: 10,
            y: 20,
            direction: Direction::Left,
            movement_mode: MovementMode::Ship,
        };

        let json = serde_json::to_string(&pos).unwrap();
        let restored: PlayerPosition = serde_json::from_str(&json).unwrap();

        assert_eq!(pos.x, restored.x);
        assert_eq!(pos.y, restored.y);
        assert_eq!(pos.direction, restored.direction);
        assert_eq!(pos.movement_mode, restored.movement_mode);
    }
}
