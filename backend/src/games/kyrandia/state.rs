//! Player state for Morningmist
//! Persisted in the database as JSON

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::data::{Region, MageRank, get_room};

/// Main player game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Player's display name
    pub name: String,

    /// Current level (1-7)
    pub level: u8,

    /// Experience points
    pub xp: u64,

    /// Current health
    pub health: u32,

    /// Maximum health
    pub max_health: u32,

    /// Current mana
    pub mana: u32,

    /// Maximum mana
    pub max_mana: u32,

    /// Gold currency
    pub gold: i64,

    /// Current room key
    pub current_room: String,

    /// Inventory: item_key -> quantity
    pub inventory: HashMap<String, u32>,

    /// Maximum inventory slots
    pub inventory_capacity: u32,

    /// Known spells (spell_key)
    pub known_spells: Vec<String>,

    /// Equipped weapon (item_key)
    pub equipped_weapon: Option<String>,

    /// Equipped armor (item_key)
    pub equipped_armor: Option<String>,

    /// Active spell effects (effect_key -> turns remaining)
    pub active_effects: HashMap<String, u32>,

    /// Puzzle progress: puzzle_key -> solved
    pub puzzles_solved: HashMap<String, bool>,

    /// Quest flags for story progression
    pub quest_flags: HashMap<String, String>,

    /// NPCs the player has talked to
    pub npcs_met: Vec<String>,

    /// Romance partner (user_id or NPC key)
    pub romance_partner: Option<String>,

    /// Romance level with partner (0-100)
    pub romance_level: u32,

    /// Daily turns remaining
    pub turns_remaining: u32,

    /// Last turn date (for daily reset)
    pub last_turn_date: String,

    /// Player kills (PvP)
    pub pvp_kills: u32,

    /// Player deaths (PvP)
    pub pvp_deaths: u32,

    /// Monsters defeated
    pub monsters_killed: u32,

    /// Times died (PvE)
    pub deaths: u32,

    /// Total gold earned
    pub total_gold_earned: i64,

    /// Has defeated the dragon
    pub dragon_defeated: bool,

    /// Has become Arch-Mage
    pub became_archmage: bool,

    /// Game completion time (if completed)
    pub completion_time: Option<String>,

    /// Active combat state (if in combat)
    #[serde(default)]
    pub combat: Option<CombatState>,

    /// Last message to display
    #[serde(default)]
    pub last_message: Option<String>,

    /// IGM module states
    #[serde(default)]
    pub igm_states: HashMap<String, HashMap<String, String>>,

    /// Input mode (for hybrid input)
    #[serde(default)]
    pub input_mode: InputMode,
}

/// Combat state during a battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub monster_key: String,
    pub monster_hp: u32,
    pub monster_max_hp: u32,
    pub player_turn: bool,
    pub shield_active: bool,
    pub shield_power: u32,
}

/// Input mode for hybrid system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InputMode {
    #[default]
    Menu,      // Standard menu-driven
    Text,      // Free text entry (for spells, puzzles)
    Combat,    // Combat menu
}

impl GameState {
    /// Create a new game for a player
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: 1,
            xp: 0,
            health: 50,
            max_health: 50,
            mana: 30,
            max_mana: 30,
            gold: 50,
            current_room: "village_square".to_string(),
            inventory: HashMap::new(),
            inventory_capacity: 10,
            known_spells: vec!["light".to_string()],  // Start with Light spell
            equipped_weapon: None,
            equipped_armor: None,
            active_effects: HashMap::new(),
            puzzles_solved: HashMap::new(),
            quest_flags: HashMap::new(),
            npcs_met: Vec::new(),
            romance_partner: None,
            romance_level: 0,
            turns_remaining: 20,
            last_turn_date: String::new(),
            pvp_kills: 0,
            pvp_deaths: 0,
            monsters_killed: 0,
            deaths: 0,
            total_gold_earned: 50,
            dragon_defeated: false,
            became_archmage: false,
            completion_time: None,
            combat: None,
            last_message: None,
            igm_states: HashMap::new(),
            input_mode: InputMode::Menu,
        }
    }

    /// Get current mage rank based on level
    pub fn rank(&self) -> MageRank {
        MageRank::from_level(self.level)
    }

    /// Get current region based on room
    pub fn current_region(&self) -> Region {
        get_room(&self.current_room)
            .map(|r| r.region)
            .unwrap_or(Region::Village)
    }

    /// Check if player has an item
    pub fn has_item(&self, key: &str) -> bool {
        self.inventory.get(key).copied().unwrap_or(0) > 0
    }

    /// Get item quantity
    pub fn item_count(&self, key: &str) -> u32 {
        self.inventory.get(key).copied().unwrap_or(0)
    }

    /// Add items to inventory
    pub fn add_item(&mut self, key: &str, count: u32) -> bool {
        let current_total: u32 = self.inventory.values().sum();
        if current_total + count > self.inventory_capacity {
            return false;  // Inventory full
        }

        *self.inventory.entry(key.to_string()).or_insert(0) += count;
        true
    }

    /// Remove items from inventory
    pub fn remove_item(&mut self, key: &str, count: u32) -> bool {
        let current = self.inventory.get(key).copied().unwrap_or(0);
        if current < count {
            return false;  // Not enough
        }

        if current == count {
            self.inventory.remove(key);
        } else {
            self.inventory.insert(key.to_string(), current - count);
        }
        true
    }

    /// Get total inventory count
    pub fn inventory_count(&self) -> u32 {
        self.inventory.values().sum()
    }

    /// Check if player knows a spell
    pub fn knows_spell(&self, key: &str) -> bool {
        self.known_spells.contains(&key.to_string())
    }

    /// Learn a new spell
    pub fn learn_spell(&mut self, key: &str) -> bool {
        if self.knows_spell(key) {
            return false;  // Already known
        }
        self.known_spells.push(key.to_string());
        true
    }

    /// Add experience points and check for level up
    pub fn add_xp(&mut self, amount: u64) -> bool {
        self.xp += amount;
        let next_level = self.level + 1;
        if next_level <= 7 {
            let required = MageRank::from_level(next_level).xp_required();
            if self.xp >= required {
                self.level = next_level;
                // Stat increases on level up
                self.max_health += 15;
                self.health = self.max_health;
                self.max_mana += 10;
                self.mana = self.max_mana;
                return true;  // Leveled up
            }
        }
        false
    }

    /// Add gold
    pub fn add_gold(&mut self, amount: i64) {
        self.gold += amount;
        if amount > 0 {
            self.total_gold_earned += amount;
        }
    }

    /// Heal the player
    pub fn heal(&mut self, amount: u32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Damage the player
    pub fn damage(&mut self, amount: u32) -> bool {
        if amount >= self.health {
            self.health = 0;
            self.deaths += 1;
            return true;  // Player died
        }
        self.health -= amount;
        false
    }

    /// Restore mana
    pub fn restore_mana(&mut self, amount: u32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    /// Use mana
    pub fn use_mana(&mut self, amount: u32) -> bool {
        if self.mana < amount {
            return false;
        }
        self.mana -= amount;
        true
    }

    /// Check if player can access a region
    pub fn can_access_region(&self, region: Region) -> bool {
        self.level >= region.required_level()
    }

    /// Check if player has met an NPC
    pub fn has_met_npc(&self, key: &str) -> bool {
        self.npcs_met.contains(&key.to_string())
    }

    /// Mark NPC as met
    pub fn meet_npc(&mut self, key: &str) {
        if !self.has_met_npc(key) {
            self.npcs_met.push(key.to_string());
        }
    }

    /// Check if a puzzle is solved
    pub fn is_puzzle_solved(&self, key: &str) -> bool {
        self.puzzles_solved.get(key).copied().unwrap_or(false)
    }

    /// Mark a puzzle as solved
    pub fn solve_puzzle(&mut self, key: &str) {
        self.puzzles_solved.insert(key.to_string(), true);
    }

    /// Get a quest flag
    pub fn get_flag(&self, key: &str) -> Option<&String> {
        self.quest_flags.get(key)
    }

    /// Set a quest flag
    pub fn set_flag(&mut self, key: &str, value: &str) {
        self.quest_flags.insert(key.to_string(), value.to_string());
    }

    /// Check for daily turn reset
    pub fn check_daily_reset(&mut self) -> bool {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if self.last_turn_date != today {
            self.last_turn_date = today;
            self.turns_remaining = 20;
            return true;  // Reset occurred
        }
        false
    }

    /// Use a turn
    pub fn use_turn(&mut self) -> bool {
        if self.turns_remaining == 0 {
            return false;
        }
        self.turns_remaining -= 1;
        true
    }

    /// Respawn after death
    pub fn respawn(&mut self) {
        self.health = self.max_health / 2;
        self.mana = self.max_mana / 2;
        self.current_room = "village_inn".to_string();
        self.combat = None;

        // Lose some gold on death
        let lost = self.gold / 10;
        self.gold -= lost;
    }

    /// Get power bonus from equipment
    pub fn equipment_power_bonus(&self) -> u32 {
        let weapon_bonus = match self.equipped_weapon.as_deref() {
            Some("wooden_staff") => 5,
            Some("crystal_wand") => 10,
            Some("enchanted_staff") => 20,
            _ => 0,
        };

        let armor_bonus = match self.equipped_armor.as_deref() {
            Some("apprentice_robe") => 2,
            Some("mage_robe") => 8,
            Some("archmage_robe") => 15,
            _ => 0,
        };

        weapon_bonus + armor_bonus
    }

    /// Get defense bonus from equipment
    pub fn equipment_defense_bonus(&self) -> u32 {
        match self.equipped_armor.as_deref() {
            Some("apprentice_robe") => 3,
            Some("mage_robe") => 8,
            Some("archmage_robe") => 20,
            _ => 0,
        }
    }

    /// Calculate total attack power
    pub fn attack_power(&self) -> u32 {
        let base = 5 + (self.level as u32 * 3);
        base + self.equipment_power_bonus()
    }

    /// Calculate total defense
    pub fn defense(&self) -> u32 {
        let base = 2 + (self.level as u32 * 2);
        base + self.equipment_defense_bonus()
    }

    /// Calculate XP needed for next level
    pub fn xp_to_next_level(&self) -> u64 {
        if self.level >= 7 {
            return 0;  // Max level
        }
        let next = MageRank::from_level(self.level + 1).xp_required();
        if next > self.xp { next - self.xp } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new("TestMage");
        assert_eq!(state.name, "TestMage");
        assert_eq!(state.level, 1);
        assert_eq!(state.health, 50);
        assert!(state.knows_spell("light"));
    }

    #[test]
    fn test_inventory_operations() {
        let mut state = GameState::new("Test");

        assert!(state.add_item("pine_cone", 3));
        assert_eq!(state.item_count("pine_cone"), 3);
        assert!(state.has_item("pine_cone"));

        assert!(state.remove_item("pine_cone", 2));
        assert_eq!(state.item_count("pine_cone"), 1);

        assert!(!state.remove_item("pine_cone", 5));  // Not enough
    }

    #[test]
    fn test_inventory_capacity() {
        let mut state = GameState::new("Test");
        state.inventory_capacity = 5;

        assert!(state.add_item("pine_cone", 3));
        assert!(state.add_item("health_potion", 2));
        assert!(!state.add_item("mana_potion", 1));  // Full
    }

    #[test]
    fn test_level_up() {
        let mut state = GameState::new("Test");
        let old_health = state.max_health;

        // Add enough XP to level up
        let leveled = state.add_xp(100);
        assert!(leveled);
        assert_eq!(state.level, 2);
        assert!(state.max_health > old_health);
    }

    #[test]
    fn test_combat_damage() {
        let mut state = GameState::new("Test");
        state.health = 50;

        let died = state.damage(30);
        assert!(!died);
        assert_eq!(state.health, 20);

        let died = state.damage(30);
        assert!(died);
        assert_eq!(state.health, 0);
    }

    #[test]
    fn test_mana_usage() {
        let mut state = GameState::new("Test");
        state.mana = 20;

        assert!(state.use_mana(10));
        assert_eq!(state.mana, 10);

        assert!(!state.use_mana(15));  // Not enough
        assert_eq!(state.mana, 10);
    }

    #[test]
    fn test_spell_learning() {
        let mut state = GameState::new("Test");

        assert!(!state.knows_spell("fireball"));
        assert!(state.learn_spell("fireball"));
        assert!(state.knows_spell("fireball"));
        assert!(!state.learn_spell("fireball"));  // Already known
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new("Test");
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.name, restored.name);
        assert_eq!(state.level, restored.level);
    }
}
