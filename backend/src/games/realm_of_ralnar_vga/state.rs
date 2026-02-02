//! Game state for Realm of Ralnar VGA

use serde::{Deserialize, Serialize};

/// Player character state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub level: u8,
    pub experience: u32,
    pub gold: u32,

    // Stats
    pub max_hp: u16,
    pub current_hp: u16,
    pub max_mp: u16,
    pub current_mp: u16,
    pub strength: u8,
    pub defense: u8,
    pub agility: u8,
    pub magic: u8,

    // Equipment slots
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub shield: Option<String>,
    pub accessory: Option<String>,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            name: "Hero".to_string(),
            level: 1,
            experience: 0,
            gold: 100,
            max_hp: 50,
            current_hp: 50,
            max_mp: 10,
            current_mp: 10,
            strength: 10,
            defense: 10,
            agility: 10,
            magic: 5,
            weapon: None,
            armor: None,
            shield: None,
            accessory: None,
        }
    }
}

/// Player position and movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub map_name: String,
    pub x: u32,
    pub y: u32,
    pub facing: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Party inventory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    pub quantity: u16,
}

/// Quest/story progress flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestProgress {
    pub flags: std::collections::HashMap<String, bool>,
    pub chapter: u8,
}

/// Vehicle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vehicle {
    None,
    Ship,
    Airship,
}

/// Full game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Party members (up to 4)
    pub party: Vec<Character>,

    /// Current position
    pub position: Position,

    /// Shared inventory
    pub inventory: Inventory,

    /// Quest progress
    pub quest: QuestProgress,

    /// Current vehicle
    pub vehicle: Vehicle,

    /// Play time in seconds
    pub play_time: u64,

    /// Steps taken (for random encounters)
    pub step_count: u32,

    /// Chests opened (by map:x:y key)
    pub opened_chests: std::collections::HashSet<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            party: vec![Character::default()],
            position: Position {
                map_name: "TOWN1".to_string(),
                x: 17,
                y: 17,
                facing: Direction::Down,
            },
            inventory: Inventory::default(),
            quest: QuestProgress::default(),
            vehicle: Vehicle::None,
            play_time: 0,
            step_count: 0,
            opened_chests: std::collections::HashSet::new(),
        }
    }
}

impl GameState {
    pub fn new(player_name: &str) -> Self {
        let mut state = Self::default();
        state.party[0].name = player_name.to_string();
        state
    }

    /// Get the lead character
    pub fn hero(&self) -> &Character {
        &self.party[0]
    }

    /// Get mutable lead character
    pub fn hero_mut(&mut self) -> &mut Character {
        &mut self.party[0]
    }

    /// Check if party can travel by sea
    pub fn has_ship(&self) -> bool {
        self.vehicle == Vehicle::Ship || self.vehicle == Vehicle::Airship
    }

    /// Check if party can fly
    pub fn has_airship(&self) -> bool {
        self.vehicle == Vehicle::Airship
    }

    /// Record a step (for random encounter calculation)
    pub fn take_step(&mut self) {
        self.step_count += 1;
    }

    /// Mark a chest as opened
    pub fn open_chest(&mut self, map: &str, x: u32, y: u32) {
        self.opened_chests.insert(format!("{}:{}:{}", map, x, y));
    }

    /// Check if chest was already opened
    pub fn is_chest_opened(&self, map: &str, x: u32, y: u32) -> bool {
        self.opened_chests.contains(&format!("{}:{}:{}", map, x, y))
    }
}
