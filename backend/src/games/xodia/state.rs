//! Character and game state for Xodia
//!
//! Contains the serializable player state including stats, inventory,
//! location, and progression.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Primary character class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Warrior,
    Mage,
    Rogue,
    Cleric,
}

impl CharacterClass {
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Mage => "Mage",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Cleric => "Cleric",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "A master of martial combat. High strength and constitution.",
            CharacterClass::Mage => "A wielder of arcane power. High intelligence and magical ability.",
            CharacterClass::Rogue => "A shadow in the night. High dexterity and cunning.",
            CharacterClass::Cleric => "A divine servant. Balanced stats with healing magic.",
        }
    }

    /// Stat modifiers for each class (STR, DEX, CON, INT, WIS, CHA)
    pub fn stat_modifiers(&self) -> (i32, i32, i32, i32, i32, i32) {
        match self {
            CharacterClass::Warrior => (4, 0, 2, -2, 0, 0),
            CharacterClass::Mage => (-2, 0, -1, 4, 2, 0),
            CharacterClass::Rogue => (0, 4, 0, 1, 0, 0),
            CharacterClass::Cleric => (0, 0, 1, 0, 3, 2),
        }
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Character stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

impl CharacterStats {
    pub fn new() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }

    /// Apply class modifiers
    pub fn apply_class_modifiers(&mut self, class: CharacterClass) {
        let (str_mod, dex_mod, con_mod, int_mod, wis_mod, cha_mod) = class.stat_modifiers();
        self.strength += str_mod;
        self.dexterity += dex_mod;
        self.constitution += con_mod;
        self.intelligence += int_mod;
        self.wisdom += wis_mod;
        self.charisma += cha_mod;
    }

    /// Calculate stat modifier (D&D style: (stat - 10) / 2)
    pub fn modifier(&self, stat: &str) -> i32 {
        let value = match stat {
            "strength" | "str" => self.strength,
            "dexterity" | "dex" => self.dexterity,
            "constitution" | "con" => self.constitution,
            "intelligence" | "int" => self.intelligence,
            "wisdom" | "wis" => self.wisdom,
            "charisma" | "cha" => self.charisma,
            _ => 10,
        };
        (value - 10) / 2
    }
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self::new()
    }
}

/// An item in the player's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub key: String,
    pub name: String,
    pub quantity: u32,
    pub equipped: bool,
}

/// Equipment slots
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub shield: Option<String>,
    pub accessory: Option<String>,
}

/// Quest progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    pub quest_id: String,
    pub stage: u32,
    pub completed: bool,
    pub data: HashMap<String, String>,
}

/// NPC relationship tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRelation {
    pub npc_id: String,
    pub reputation: i32,  // -100 to 100
    pub met: bool,
    pub last_interaction: Option<String>,
}

/// The complete player game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // Character identity
    pub character_name: String,
    pub class: CharacterClass,
    pub handle: Option<String>,

    // Stats and level
    pub stats: CharacterStats,
    pub level: u32,
    pub experience: u64,

    // Health and resources
    pub health: i32,
    pub max_health: i32,
    pub mana: i32,
    pub max_mana: i32,

    // Location
    pub current_room_id: String,
    pub current_region: String,

    // Inventory and equipment
    pub inventory: Vec<InventoryItem>,
    pub equipment: Equipment,
    pub gold: i64,
    pub carry_weight: f32,
    pub max_carry_weight: f32,

    // Progression
    pub quests: Vec<QuestProgress>,
    pub main_quest_stage: u32,
    pub discovered_rooms: Vec<String>,

    // Social
    pub npc_relations: HashMap<String, NpcRelation>,

    // Session state
    pub in_combat: bool,
    pub combat_target: Option<String>,

    // UI state
    #[serde(default)]
    pub last_message: Option<String>,
    #[serde(default)]
    pub last_llm_response: Option<String>,

    // Timestamps
    pub created_at: String,
    pub last_played: String,
    pub total_playtime_seconds: u64,
}

impl GameState {
    /// Create a new character
    pub fn new(name: &str, class: CharacterClass) -> Self {
        let mut stats = CharacterStats::new();
        stats.apply_class_modifiers(class);

        // Calculate derived stats
        let max_health = 50 + (stats.constitution * 5);
        let max_mana = match class {
            CharacterClass::Mage => 30 + (stats.intelligence * 3),
            CharacterClass::Cleric => 20 + (stats.wisdom * 2),
            _ => 10 + stats.wisdom,
        };

        let max_carry_weight = 50.0 + (stats.strength as f32 * 5.0);

        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            character_name: name.to_string(),
            class,
            handle: None,

            stats,
            level: 1,
            experience: 0,

            health: max_health,
            max_health,
            mana: max_mana,
            max_mana,

            current_room_id: "misthollow_square".to_string(),
            current_region: "misthollow".to_string(),

            inventory: Vec::new(),
            equipment: Equipment::default(),
            gold: 50,
            carry_weight: 0.0,
            max_carry_weight,

            quests: Vec::new(),
            main_quest_stage: 0,
            discovered_rooms: vec!["misthollow_square".to_string()],

            npc_relations: HashMap::new(),

            in_combat: false,
            combat_target: None,

            last_message: None,
            last_llm_response: None,

            created_at: now.clone(),
            last_played: now,
            total_playtime_seconds: 0,
        }
    }

    /// Calculate attack power
    pub fn attack_power(&self) -> i32 {
        let base = self.stats.modifier("strength");
        let weapon_bonus = self.get_weapon_bonus();
        (base + weapon_bonus).max(1)
    }

    /// Calculate defense
    pub fn defense(&self) -> i32 {
        let base = 10 + self.stats.modifier("dexterity");
        let armor_bonus = self.get_armor_bonus();
        base + armor_bonus
    }

    /// Get weapon damage bonus
    fn get_weapon_bonus(&self) -> i32 {
        if let Some(ref weapon_key) = self.equipment.weapon {
            if let Some(template) = super::data::get_item_template(weapon_key) {
                return template.stat_bonus;
            }
        }
        0
    }

    /// Get armor defense bonus
    fn get_armor_bonus(&self) -> i32 {
        let mut bonus = 0;
        if let Some(ref armor_key) = self.equipment.armor {
            if let Some(template) = super::data::get_item_template(armor_key) {
                bonus += template.stat_bonus;
            }
        }
        if let Some(ref shield_key) = self.equipment.shield {
            if let Some(template) = super::data::get_item_template(shield_key) {
                bonus += template.stat_bonus;
            }
        }
        bonus
    }

    /// Check if player has an item
    pub fn has_item(&self, key: &str) -> bool {
        self.inventory.iter().any(|i| i.key == key && i.quantity > 0)
    }

    /// Get item quantity
    pub fn item_quantity(&self, key: &str) -> u32 {
        self.inventory.iter()
            .find(|i| i.key == key)
            .map(|i| i.quantity)
            .unwrap_or(0)
    }

    /// Add item to inventory
    pub fn add_item(&mut self, key: &str, name: &str, quantity: u32, weight: f32) -> bool {
        let total_weight = weight * quantity as f32;
        if self.carry_weight + total_weight > self.max_carry_weight {
            return false;
        }

        self.carry_weight += total_weight;

        if let Some(item) = self.inventory.iter_mut().find(|i| i.key == key) {
            item.quantity += quantity;
        } else {
            self.inventory.push(InventoryItem {
                key: key.to_string(),
                name: name.to_string(),
                quantity,
                equipped: false,
            });
        }
        true
    }

    /// Remove item from inventory
    pub fn remove_item(&mut self, key: &str, quantity: u32) -> bool {
        if let Some(idx) = self.inventory.iter().position(|i| i.key == key) {
            let item = &mut self.inventory[idx];
            if item.quantity >= quantity {
                if let Some(template) = super::data::get_item_template(key) {
                    self.carry_weight -= template.weight * quantity as f32;
                }
                item.quantity -= quantity;
                if item.quantity == 0 {
                    self.inventory.remove(idx);
                }
                return true;
            }
        }
        false
    }

    /// Equip an item
    pub fn equip_item(&mut self, key: &str) -> Result<String, String> {
        if !self.has_item(key) {
            return Err("You don't have that item.".to_string());
        }

        let template = super::data::get_item_template(key)
            .ok_or("Unknown item.")?;

        match template.item_type {
            super::data::ItemType::Weapon => {
                // Unequip current weapon if any
                if let Some(ref old) = self.equipment.weapon {
                    if let Some(item) = self.inventory.iter_mut().find(|i| &i.key == old) {
                        item.equipped = false;
                    }
                }
                self.equipment.weapon = Some(key.to_string());
            }
            super::data::ItemType::Armor => {
                if let Some(ref old) = self.equipment.armor {
                    if let Some(item) = self.inventory.iter_mut().find(|i| &i.key == old) {
                        item.equipped = false;
                    }
                }
                self.equipment.armor = Some(key.to_string());
            }
            super::data::ItemType::Shield => {
                if let Some(ref old) = self.equipment.shield {
                    if let Some(item) = self.inventory.iter_mut().find(|i| &i.key == old) {
                        item.equipped = false;
                    }
                }
                self.equipment.shield = Some(key.to_string());
            }
            _ => return Err("That item cannot be equipped.".to_string()),
        }

        if let Some(item) = self.inventory.iter_mut().find(|i| i.key == key) {
            item.equipped = true;
        }

        Ok(format!("Equipped {}.", template.name))
    }

    /// Take damage
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.health = (self.health - amount).max(0);
        self.health <= 0
    }

    /// Heal
    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Restore mana
    pub fn restore_mana(&mut self, amount: i32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, amount: u64) -> Option<u32> {
        self.experience += amount;

        let required = super::data::LEVEL_XP_REQUIREMENTS
            .get(self.level as usize)
            .copied()
            .unwrap_or(u64::MAX);

        if self.experience >= required && self.level < 20 {
            self.level += 1;
            // Increase max HP/MP on level up
            self.max_health += 5 + self.stats.modifier("constitution");
            self.health = self.max_health;

            match self.class {
                CharacterClass::Mage => {
                    self.max_mana += 5 + self.stats.modifier("intelligence");
                }
                CharacterClass::Cleric => {
                    self.max_mana += 3 + self.stats.modifier("wisdom");
                }
                _ => {
                    self.max_mana += 2;
                }
            }
            self.mana = self.max_mana;

            Some(self.level)
        } else {
            None
        }
    }

    /// Discover a new room
    pub fn discover_room(&mut self, room_id: &str) -> bool {
        if !self.discovered_rooms.contains(&room_id.to_string()) {
            self.discovered_rooms.push(room_id.to_string());
            true
        } else {
            false
        }
    }

    /// Update last played timestamp
    pub fn update_timestamp(&mut self) {
        self.last_played = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_character_warrior() {
        let state = GameState::new("TestHero", CharacterClass::Warrior);
        assert_eq!(state.character_name, "TestHero");
        assert_eq!(state.class, CharacterClass::Warrior);
        assert_eq!(state.level, 1);
        assert!(state.stats.strength > 10); // Warriors get STR bonus
        assert!(state.health > 0);
    }

    #[test]
    fn test_new_character_mage() {
        let state = GameState::new("TestMage", CharacterClass::Mage);
        assert!(state.stats.intelligence > 10); // Mages get INT bonus
        assert!(state.max_mana > state.max_health / 2); // Mages have more mana
    }

    #[test]
    fn test_inventory_operations() {
        let mut state = GameState::new("Test", CharacterClass::Warrior);

        // Add item
        assert!(state.add_item("health_potion", "Health Potion", 3, 0.2));
        assert_eq!(state.item_quantity("health_potion"), 3);
        assert!(state.has_item("health_potion"));

        // Remove item
        assert!(state.remove_item("health_potion", 1));
        assert_eq!(state.item_quantity("health_potion"), 2);

        // Remove all
        assert!(state.remove_item("health_potion", 2));
        assert!(!state.has_item("health_potion"));
    }

    #[test]
    fn test_damage_and_healing() {
        let mut state = GameState::new("Test", CharacterClass::Warrior);
        let initial_health = state.health;

        state.take_damage(20);
        assert_eq!(state.health, initial_health - 20);

        state.heal(10);
        assert_eq!(state.health, initial_health - 10);

        // Heal beyond max
        state.heal(1000);
        assert_eq!(state.health, state.max_health);
    }

    #[test]
    fn test_experience_and_level_up() {
        let mut state = GameState::new("Test", CharacterClass::Warrior);
        assert_eq!(state.level, 1);

        // Add enough XP to level up
        let new_level = state.add_experience(150);
        assert!(new_level.is_some());
        assert_eq!(state.level, 2);
    }

    #[test]
    fn test_stat_modifiers() {
        let stats = CharacterStats {
            strength: 16,
            dexterity: 14,
            constitution: 12,
            intelligence: 8,
            wisdom: 10,
            charisma: 13,
        };

        assert_eq!(stats.modifier("strength"), 3);
        assert_eq!(stats.modifier("dexterity"), 2);
        assert_eq!(stats.modifier("constitution"), 1);
        assert_eq!(stats.modifier("intelligence"), -1);
        assert_eq!(stats.modifier("wisdom"), 0);
        assert_eq!(stats.modifier("charisma"), 1);
    }

    #[test]
    fn test_room_discovery() {
        let mut state = GameState::new("Test", CharacterClass::Warrior);

        // Starting room should be discovered
        assert!(state.discovered_rooms.contains(&"misthollow_square".to_string()));

        // Discover new room
        assert!(state.discover_room("misthollow_elder"));
        assert!(state.discovered_rooms.contains(&"misthollow_elder".to_string()));

        // Already discovered returns false
        assert!(!state.discover_room("misthollow_elder"));
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new("SerializeTest", CharacterClass::Cleric);
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.character_name, restored.character_name);
        assert_eq!(state.class, restored.class);
        assert_eq!(state.level, restored.level);
        assert_eq!(state.gold, restored.gold);
    }
}
