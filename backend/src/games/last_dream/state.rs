//! Last Dream game state
//! Persistent player data that gets serialized to the database

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::party::Party;
use super::world::{Position, Transportation};

/// Inventory item with quantity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub key: String,
    pub quantity: u32,
}

/// The main game state for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Player's BBS handle
    pub handle: Option<String>,

    /// Party of characters
    pub party: Party,

    /// World position
    pub world_position: Position,
    /// Current location (if in a town/dungeon)
    pub current_location: Option<String>,
    /// Dungeon floor (if in dungeon)
    pub dungeon_floor: Option<u8>,
    /// Position within location
    pub location_position: Option<Position>,

    /// Transportation mode
    pub transport: Transportation,
    /// Ship location (parked)
    pub ship_position: Option<Position>,
    /// Airship location (parked)
    pub airship_position: Option<Position>,

    /// Gold
    pub gold: u32,

    /// Inventory
    pub inventory: Vec<InventoryItem>,

    /// Story progress flags
    pub story_flags: HashSet<String>,

    /// Opened chests (dungeon_key:floor:x:y)
    pub opened_chests: HashSet<String>,

    /// Play time in seconds
    pub play_time: u64,

    /// Total battles fought
    pub battles_fought: u32,
    /// Monsters defeated
    pub monsters_defeated: u32,

    /// Last save timestamp
    pub last_saved: Option<String>,

    /// Message to display
    pub last_message: Option<String>,

    /// Romance partner (if any) - supports same-sex
    pub romance_partner: Option<String>,
    pub romance_level: u8,

    /// Simulation hints seen (for tracking rarity)
    pub hints_seen: u8,
}

impl GameState {
    /// Create a new game state
    pub fn new() -> Self {
        Self {
            handle: None,
            party: Party::new(),
            world_position: Position::new(2, 5), // Near Cornelia
            current_location: None,
            dungeon_floor: None,
            location_position: None,
            transport: Transportation::Walking,
            ship_position: None,
            airship_position: None,
            gold: 500, // Starting gold
            inventory: vec![
                InventoryItem { key: "potion".to_string(), quantity: 3 },
            ],
            story_flags: HashSet::new(),
            opened_chests: HashSet::new(),
            play_time: 0,
            battles_fought: 0,
            monsters_defeated: 0,
            last_saved: None,
            last_message: None,
            romance_partner: None,
            romance_level: 0,
            hints_seen: 0,
        }
    }

    /// Check if player has a story flag
    pub fn has_flag(&self, flag: &str) -> bool {
        self.story_flags.contains(flag)
    }

    /// Set a story flag
    pub fn set_flag(&mut self, flag: &str) {
        self.story_flags.insert(flag.to_string());
    }

    /// Get item quantity in inventory
    pub fn item_count(&self, key: &str) -> u32 {
        self.inventory.iter()
            .find(|i| i.key == key)
            .map(|i| i.quantity)
            .unwrap_or(0)
    }

    /// Add items to inventory
    pub fn add_item(&mut self, key: &str, quantity: u32) {
        if let Some(item) = self.inventory.iter_mut().find(|i| i.key == key) {
            item.quantity += quantity;
        } else {
            self.inventory.push(InventoryItem {
                key: key.to_string(),
                quantity,
            });
        }
    }

    /// Remove items from inventory
    pub fn remove_item(&mut self, key: &str, quantity: u32) -> bool {
        if let Some(item) = self.inventory.iter_mut().find(|i| i.key == key) {
            if item.quantity >= quantity {
                item.quantity -= quantity;
                if item.quantity == 0 {
                    self.inventory.retain(|i| i.key != key);
                }
                return true;
            }
        }
        false
    }

    /// Add gold
    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }

    /// Spend gold
    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// Check if a chest has been opened
    pub fn is_chest_opened(&self, dungeon: &str, floor: u8, x: usize, y: usize) -> bool {
        let key = format!("{}:{}:{}:{}", dungeon, floor, x, y);
        self.opened_chests.contains(&key)
    }

    /// Mark a chest as opened
    pub fn open_chest(&mut self, dungeon: &str, floor: u8, x: usize, y: usize) {
        let key = format!("{}:{}:{}:{}", dungeon, floor, x, y);
        self.opened_chests.insert(key);
    }

    /// Can access a location based on story progress?
    pub fn can_access_location(&self, location_key: &str) -> bool {
        use super::world::get_location;

        if let Some(location) = get_location(location_key) {
            if let Some(required_flag) = location.story_flag_required {
                return self.has_flag(required_flag);
            }
            true
        } else {
            false
        }
    }

    /// Enter a location
    pub fn enter_location(&mut self, location_key: &str) {
        self.current_location = Some(location_key.to_string());
        self.location_position = Some(Position::new(5, 5)); // Default spawn
        self.dungeon_floor = None;
    }

    /// Exit to world map
    pub fn exit_location(&mut self) {
        self.current_location = None;
        self.location_position = None;
        self.dungeon_floor = None;
    }

    /// Enter dungeon floor
    pub fn enter_dungeon_floor(&mut self, floor: u8) {
        self.dungeon_floor = Some(floor);
        self.location_position = Some(Position::new(1, 1));
    }

    /// Is player on world map?
    pub fn is_on_world_map(&self) -> bool {
        self.current_location.is_none()
    }

    /// Is player in a dungeon?
    pub fn is_in_dungeon(&self) -> bool {
        if let Some(ref loc) = self.current_location {
            use super::world::{get_location, LocationType};
            if let Some(location) = get_location(loc) {
                return matches!(location.location_type, LocationType::Dungeon | LocationType::Cave);
            }
        }
        false
    }

    /// Get current area level for encounters
    pub fn area_level(&self) -> u8 {
        if let Some(ref loc) = self.current_location {
            use super::world::get_location;
            if let Some(location) = get_location(loc) {
                let base_level = location.area_level;
                // Add floor bonus for dungeons
                if let Some(floor) = self.dungeon_floor {
                    return base_level + floor * 2;
                }
                return base_level;
            }
        }

        // World map - based on distance from start
        let dist = ((self.world_position.x as i32 - 2).abs() +
                    (self.world_position.y as i32 - 5).abs()) as u8;
        (dist / 5).max(1)
    }

    /// Increment play time
    pub fn add_play_time(&mut self, seconds: u64) {
        self.play_time += seconds;
    }

    /// Format play time as HH:MM:SS
    pub fn formatted_play_time(&self) -> String {
        let hours = self.play_time / 3600;
        let minutes = (self.play_time % 3600) / 60;
        let seconds = self.play_time % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Track a battle
    pub fn record_battle(&mut self, monsters_killed: u32) {
        self.battles_fought += 1;
        self.monsters_defeated += monsters_killed;
    }

    /// Check for rare simulation hint
    pub fn check_simulation_hint(&mut self) -> Option<&'static str> {
        // Only show 1-2 hints per playthrough
        if self.hints_seen >= 2 {
            return None;
        }

        if let Some(hint) = super::data::maybe_get_simulation_hint() {
            self.hints_seen += 1;
            Some(hint)
        } else {
            None
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new();
        assert_eq!(state.gold, 500);
        assert!(state.party.members.is_empty());
        assert!(state.is_on_world_map());
    }

    #[test]
    fn test_inventory() {
        let mut state = GameState::new();
        assert_eq!(state.item_count("potion"), 3);

        state.add_item("ether", 2);
        assert_eq!(state.item_count("ether"), 2);

        state.remove_item("potion", 1);
        assert_eq!(state.item_count("potion"), 2);
    }

    #[test]
    fn test_gold() {
        let mut state = GameState::new();
        assert!(state.spend_gold(100));
        assert_eq!(state.gold, 400);

        assert!(!state.spend_gold(500));
        assert_eq!(state.gold, 400);
    }

    #[test]
    fn test_story_flags() {
        let mut state = GameState::new();
        assert!(!state.has_flag("ship_obtained"));

        state.set_flag("ship_obtained");
        assert!(state.has_flag("ship_obtained"));
    }

    #[test]
    fn test_location_entry() {
        let mut state = GameState::new();
        state.enter_location("cornelia");

        assert!(!state.is_on_world_map());
        assert_eq!(state.current_location.as_deref(), Some("cornelia"));

        state.exit_location();
        assert!(state.is_on_world_map());
    }

    #[test]
    fn test_dungeon_floor() {
        let mut state = GameState::new();
        state.enter_location("chaos_shrine");
        state.enter_dungeon_floor(3);

        assert!(state.is_in_dungeon());
        assert_eq!(state.dungeon_floor, Some(3));
    }

    #[test]
    fn test_play_time_format() {
        let mut state = GameState::new();
        state.play_time = 3723; // 1 hour, 2 minutes, 3 seconds
        assert_eq!(state.formatted_play_time(), "01:02:03");
    }

    #[test]
    fn test_chest_tracking() {
        let mut state = GameState::new();
        assert!(!state.is_chest_opened("chaos_shrine", 1, 5, 5));

        state.open_chest("chaos_shrine", 1, 5, 5);
        assert!(state.is_chest_opened("chaos_shrine", 1, 5, 5));
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new();
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.gold, restored.gold);
        assert_eq!(state.play_time, restored.play_time);
    }
}
