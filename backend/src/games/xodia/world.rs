//! World state management for Xodia
//!
//! Handles rooms, NPCs, items, and the persistent world graph.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::data::{get_npc_template, get_item_template, NpcType};

/// A room in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub region: String,
    pub exits: HashMap<String, String>,  // direction -> room_id
    pub npcs: Vec<String>,               // npc instance IDs
    pub items: Vec<RoomItem>,            // items on the ground
    pub discovered_by: Vec<i64>,         // user IDs who have discovered this room
    pub created_by: Option<i64>,         // user ID who triggered generation (if LLM-generated)
    pub is_generated: bool,              // true if LLM-generated, false if predefined
}

/// An item instance in a room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomItem {
    pub instance_id: String,
    pub template_key: String,
    pub name: String,
    pub quantity: u32,
}

/// An NPC instance in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    pub instance_id: String,
    pub template_key: String,
    pub name: String,
    pub current_room: String,
    pub health: i32,
    pub max_health: i32,
    pub is_alive: bool,
    pub is_hostile: bool,
    pub dialogue_state: u32,
    pub inventory: Vec<RoomItem>,
    pub memory: Vec<NpcMemory>,  // Remembers interactions
}

/// Memory of an interaction for NPC personality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcMemory {
    pub user_id: i64,
    pub character_name: String,
    pub interaction_type: String,
    pub summary: String,
    pub timestamp: String,
}

/// Event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: String,
    pub room_id: String,
    pub actor_id: Option<i64>,
    pub actor_name: String,
    pub action: String,
    pub target: Option<String>,
    pub outcome: String,
    pub timestamp: String,
}

/// The complete world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub rooms: HashMap<String, Room>,
    pub npcs: HashMap<String, NPC>,
    pub events: Vec<WorldEvent>,
    pub generation_seed: u64,
}

impl WorldState {
    /// Create a new world with predefined rooms
    pub fn new() -> Self {
        let mut world = Self {
            rooms: HashMap::new(),
            npcs: HashMap::new(),
            events: Vec::new(),
            generation_seed: rand::random(),
        };

        // Initialize predefined rooms
        world.initialize_predefined_rooms();

        world
    }

    /// Initialize all predefined rooms from data
    fn initialize_predefined_rooms(&mut self) {
        for template in super::data::STARTING_ROOMS {
            let mut exits = HashMap::new();
            for (dir, room_id) in template.exits {
                exits.insert(dir.to_string(), room_id.to_string());
            }

            // Create room items
            let items: Vec<RoomItem> = template.items.iter()
                .filter_map(|item_key| {
                    get_item_template(item_key).map(|t| RoomItem {
                        instance_id: format!("{}_{}", template.id, item_key),
                        template_key: item_key.to_string(),
                        name: t.name.to_string(),
                        quantity: 1,
                    })
                })
                .collect();

            let room = Room {
                id: template.id.to_string(),
                name: template.name.to_string(),
                description: template.description.to_string(),
                region: template.region.to_string(),
                exits,
                npcs: template.npcs.iter().map(|s| s.to_string()).collect(),
                items,
                discovered_by: Vec::new(),
                created_by: None,
                is_generated: false,
            };

            self.rooms.insert(template.id.to_string(), room);

            // Create NPC instances
            for npc_key in template.npcs {
                if let Some(npc_template) = get_npc_template(npc_key) {
                    let npc = NPC {
                        instance_id: format!("{}_{}", template.id, npc_key),
                        template_key: npc_key.to_string(),
                        name: npc_template.name.to_string(),
                        current_room: template.id.to_string(),
                        health: npc_template.health,
                        max_health: npc_template.health,
                        is_alive: true,
                        is_hostile: matches!(npc_template.npc_type, NpcType::Hostile),
                        dialogue_state: 0,
                        inventory: Vec::new(),
                        memory: Vec::new(),
                    };
                    self.npcs.insert(npc.instance_id.clone(), npc);
                }
            }
        }
    }

    /// Get a room by ID
    pub fn get_room(&self, room_id: &str) -> Option<&Room> {
        self.rooms.get(room_id)
    }

    /// Get a mutable room by ID
    pub fn get_room_mut(&mut self, room_id: &str) -> Option<&mut Room> {
        self.rooms.get_mut(room_id)
    }

    /// Get an NPC by instance ID
    pub fn get_npc(&self, instance_id: &str) -> Option<&NPC> {
        self.npcs.get(instance_id)
    }

    /// Get a mutable NPC by instance ID
    pub fn get_npc_mut(&mut self, instance_id: &str) -> Option<&mut NPC> {
        self.npcs.get_mut(instance_id)
    }

    /// Get all NPCs in a room
    pub fn get_npcs_in_room(&self, room_id: &str) -> Vec<&NPC> {
        self.npcs.values()
            .filter(|npc| npc.current_room == room_id && npc.is_alive)
            .collect()
    }

    /// Get all hostile NPCs in a room
    pub fn get_hostile_npcs_in_room(&self, room_id: &str) -> Vec<&NPC> {
        self.get_npcs_in_room(room_id)
            .into_iter()
            .filter(|npc| npc.is_hostile)
            .collect()
    }

    /// Add a new room (for LLM-generated content)
    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id.clone(), room);
    }

    /// Add a new NPC instance
    pub fn add_npc(&mut self, npc: NPC) {
        self.npcs.insert(npc.instance_id.clone(), npc);
    }

    /// Remove an item from a room
    pub fn remove_room_item(&mut self, room_id: &str, item_instance_id: &str) -> Option<RoomItem> {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if let Some(idx) = room.items.iter().position(|i| i.instance_id == item_instance_id) {
                return Some(room.items.remove(idx));
            }
        }
        None
    }

    /// Add an item to a room
    pub fn add_room_item(&mut self, room_id: &str, item: RoomItem) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.items.push(item);
        }
    }

    /// Log a world event
    pub fn log_event(&mut self, event: WorldEvent) {
        // Keep last 1000 events
        if self.events.len() >= 1000 {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    /// Get recent events for a room
    pub fn get_room_events(&self, room_id: &str, limit: usize) -> Vec<&WorldEvent> {
        self.events.iter()
            .rev()
            .filter(|e| e.room_id == room_id)
            .take(limit)
            .collect()
    }

    /// Mark a room as discovered by a user
    pub fn mark_room_discovered(&mut self, room_id: &str, user_id: i64) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if !room.discovered_by.contains(&user_id) {
                room.discovered_by.push(user_id);
            }
        }
    }

    /// Check if a direction leads somewhere from a room
    pub fn get_exit(&self, room_id: &str, direction: &str) -> Option<&str> {
        self.rooms.get(room_id)
            .and_then(|room| room.exits.get(direction))
            .map(|s| s.as_str())
    }

    /// Get the description of a room with current state
    pub fn describe_room(&self, room_id: &str) -> Option<String> {
        let room = self.get_room(room_id)?;
        let npcs = self.get_npcs_in_room(room_id);

        let mut description = format!("{}\n\n{}", room.name, room.description);

        // List NPCs
        if !npcs.is_empty() {
            description.push_str("\n\nYou see: ");
            let npc_names: Vec<&str> = npcs.iter().map(|n| n.name.as_str()).collect();
            description.push_str(&npc_names.join(", "));
        }

        // List items
        if !room.items.is_empty() {
            description.push_str("\n\nOn the ground: ");
            let item_names: Vec<String> = room.items.iter()
                .map(|i| if i.quantity > 1 {
                    format!("{} (x{})", i.name, i.quantity)
                } else {
                    i.name.clone()
                })
                .collect();
            description.push_str(&item_names.join(", "));
        }

        // List exits
        if !room.exits.is_empty() {
            description.push_str("\n\nExits: ");
            let exits: Vec<&str> = room.exits.keys().map(|s| s.as_str()).collect();
            description.push_str(&exits.join(", "));
        }

        Some(description)
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a new room via LLM (returns a template for the LLM to fill)
pub fn generate_room_prompt(from_room: &Room, direction: &str, region: &str) -> String {
    format!(
        r#"Generate a new room connected to "{}" via the {} direction.
Region: {}
The room should fit the region's atmosphere and be consistent with the world of Xodia.

Respond in JSON format:
{{
  "name": "Room Name",
  "description": "2-3 sentences describing the room vividly",
  "suggested_exits": ["direction1", "direction2"],
  "suggested_npcs": ["npc_type or name"],
  "suggested_items": ["item_type or name"],
  "atmosphere": "brief mood description"
}}
"#,
        from_room.name, direction, region
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_initialization() {
        let world = WorldState::new();

        // Check starting rooms exist
        assert!(world.get_room("misthollow_square").is_some());
        assert!(world.get_room("misthollow_elder").is_some());
        assert!(world.get_room("forest_path_entrance").is_some());
    }

    #[test]
    fn test_room_exits() {
        let world = WorldState::new();

        // Check exits from main square
        let exit = world.get_exit("misthollow_square", "north");
        assert_eq!(exit, Some("misthollow_elder"));

        let exit = world.get_exit("misthollow_square", "invalid");
        assert!(exit.is_none());
    }

    #[test]
    fn test_npcs_in_room() {
        let world = WorldState::new();

        // Elder Mira should be in her cottage
        let npcs = world.get_npcs_in_room("misthollow_elder");
        assert!(!npcs.is_empty());
        assert!(npcs.iter().any(|n| n.name.contains("Mira")));
    }

    #[test]
    fn test_hostile_npcs() {
        let world = WorldState::new();

        // Forest entrance should have goblin
        let hostiles = world.get_hostile_npcs_in_room("forest_path_entrance");
        assert!(!hostiles.is_empty());
    }

    #[test]
    fn test_room_items() {
        let world = WorldState::new();

        // Smithy should have items
        let smithy = world.get_room("misthollow_smithy").unwrap();
        assert!(!smithy.items.is_empty());
    }

    #[test]
    fn test_room_discovery() {
        let mut world = WorldState::new();

        world.mark_room_discovered("misthollow_square", 1);
        let room = world.get_room("misthollow_square").unwrap();
        assert!(room.discovered_by.contains(&1));

        // Duplicate marking should not add twice
        world.mark_room_discovered("misthollow_square", 1);
        let room = world.get_room("misthollow_square").unwrap();
        assert_eq!(room.discovered_by.iter().filter(|&&id| id == 1).count(), 1);
    }

    #[test]
    fn test_item_removal() {
        let mut world = WorldState::new();

        // Get initial item count
        let initial_count = world.get_room("misthollow_smithy").unwrap().items.len();

        // Remove an item
        if let Some(room) = world.get_room("misthollow_smithy") {
            if let Some(item) = room.items.first() {
                let id = item.instance_id.clone();
                let removed = world.remove_room_item("misthollow_smithy", &id);
                assert!(removed.is_some());
            }
        }

        let new_count = world.get_room("misthollow_smithy").unwrap().items.len();
        assert_eq!(new_count, initial_count - 1);
    }

    #[test]
    fn test_event_logging() {
        let mut world = WorldState::new();

        let event = WorldEvent {
            id: "test_event_1".to_string(),
            room_id: "misthollow_square".to_string(),
            actor_id: Some(1),
            actor_name: "TestPlayer".to_string(),
            action: "look".to_string(),
            target: None,
            outcome: "Player looked around.".to_string(),
            timestamp: "2026-01-30 12:00:00".to_string(),
        };

        world.log_event(event);

        let events = world.get_room_events("misthollow_square", 10);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_describe_room() {
        let world = WorldState::new();

        let description = world.describe_room("misthollow_square");
        assert!(description.is_some());

        let desc = description.unwrap();
        assert!(desc.contains("Misthollow"));
        assert!(desc.contains("Exits:"));
    }
}
