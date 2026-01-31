//! Game state for Ultimo
//!
//! Contains the character and game state that gets persisted to the database.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position in the world
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub zone: String,
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(zone: &str, x: i32, y: i32) -> Self {
        Self {
            zone: zone.to_string(),
            x,
            y,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            zone: "britain".to_string(),
            x: 30,
            y: 20,
        }
    }
}

/// An item in inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_key: String,
    pub quantity: u32,
    /// For equipment tracking durability, enchantments, etc.
    pub durability: Option<u32>,
    pub max_durability: Option<u32>,
}

impl InventoryItem {
    pub fn new(item_key: &str, quantity: u32) -> Self {
        Self {
            item_key: item_key.to_string(),
            quantity,
            durability: None,
            max_durability: None,
        }
    }

    pub fn with_durability(item_key: &str, durability: u32) -> Self {
        Self {
            item_key: item_key.to_string(),
            quantity: 1,
            durability: Some(durability),
            max_durability: Some(durability),
        }
    }
}

/// Quest progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestProgress {
    pub quest_key: String,
    pub started_at: String,
    /// For kill quests: monsters killed
    pub kills: u32,
    /// For collect quests: items collected (tracked separately)
    pub collected: u32,
    /// For visit quests: visited the zone
    pub visited: bool,
    pub completed: bool,
}

impl QuestProgress {
    pub fn new(quest_key: &str) -> Self {
        Self {
            quest_key: quest_key.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            kills: 0,
            collected: 0,
            visited: false,
            completed: false,
        }
    }
}

/// Player character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    /// User ID from BBS
    pub user_id: i64,
    pub created_at: String,
    pub last_login: String,

    // Base stats
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,

    // Derived stats
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub stamina: i32,
    pub max_stamina: i32,

    // Position
    pub position: Position,

    // Equipment slots
    pub equipped_weapon: Option<String>,
    pub equipped_armor: Option<String>,
    pub equipped_shield: Option<String>,

    // Inventory
    pub inventory: Vec<InventoryItem>,
    pub max_inventory_slots: u32,

    // Currency
    pub gold: i64,
    pub bank_gold: i64,

    // Skills (skill_key -> level 0-100)
    pub skills: HashMap<String, u32>,

    // Experience
    pub total_xp: i64,

    // Quests
    pub active_quests: Vec<QuestProgress>,
    pub completed_quests: Vec<String>,

    // Combat stats
    pub kills: HashMap<String, u32>, // monster_key -> count
    pub deaths: u32,
    pub pvp_kills: u32,
    pub pvp_deaths: u32,

    // Crafting
    pub recipes_known: Vec<String>,

    // Housing
    pub house_id: Option<i64>,

    // Miscellaneous
    pub title: Option<String>,
    pub guild_id: Option<i64>,

    // Romance (same-sex supported)
    pub partner_name: Option<String>,
    pub partner_user_id: Option<i64>,

    // Is character dead and needs resurrection
    pub is_dead: bool,

    // Last message to display
    #[serde(default)]
    pub last_message: Option<String>,
}

impl Character {
    pub fn new(name: &str, user_id: i64) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        // Starting skills
        let mut skills = HashMap::new();
        skills.insert("wrestling".to_string(), 20);
        skills.insert("healing".to_string(), 10);

        Self {
            name: name.to_string(),
            user_id,
            created_at: now.clone(),
            last_login: now,

            // Starting stats (can be customized during creation)
            strength: 10,
            dexterity: 10,
            intelligence: 10,

            // Derived stats
            hp: 50,
            max_hp: 50,
            mana: 10,
            max_mana: 10,
            stamina: 50,
            max_stamina: 50,

            position: Position::default(),

            equipped_weapon: None,
            equipped_armor: None,
            equipped_shield: None,

            inventory: vec![
                InventoryItem::new("bread", 5),
                InventoryItem::new("bandage", 10),
                InventoryItem::new("gold_coin", 100),
            ],
            max_inventory_slots: 50,

            gold: 100,
            bank_gold: 0,

            skills,

            total_xp: 0,

            active_quests: Vec::new(),
            completed_quests: Vec::new(),

            kills: HashMap::new(),
            deaths: 0,
            pvp_kills: 0,
            pvp_deaths: 0,

            recipes_known: Vec::new(),

            house_id: None,
            title: None,
            guild_id: None,

            partner_name: None,
            partner_user_id: None,

            is_dead: false,
            last_message: None,
        }
    }

    /// Get skill level (0 if not learned)
    pub fn get_skill(&self, skill_key: &str) -> u32 {
        self.skills.get(skill_key).copied().unwrap_or(0)
    }

    /// Attempt to gain skill (use-based progression)
    /// Returns true if skill increased
    pub fn try_skill_gain(&mut self, skill_key: &str, difficulty: u32) -> bool {
        let current = self.get_skill(skill_key);
        if current >= 100 {
            return false; // Maxed out
        }

        // Gain chance decreases as skill increases
        // Higher difficulty relative to skill = better chance
        let gain_chance = if difficulty > current {
            50 // Easy gain when doing something hard
        } else if difficulty + 20 > current {
            25 // Moderate gain
        } else {
            5 // Hard to gain from easy tasks
        };

        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen_range(0..100) < gain_chance {
            let gain = rng.gen_range(1..=3).min(100 - current);
            self.skills.insert(skill_key.to_string(), current + gain);
            return true;
        }

        false
    }

    /// Calculate effective level based on total XP
    pub fn level(&self) -> u32 {
        // Simple level formula: level = sqrt(total_xp / 100)
        ((self.total_xp as f64 / 100.0).sqrt() as u32).max(1)
    }

    /// Add XP and return true if leveled up
    pub fn add_xp(&mut self, amount: i64) -> bool {
        let old_level = self.level();
        self.total_xp += amount;
        let new_level = self.level();

        if new_level > old_level {
            // Level up bonuses
            self.max_hp += 5;
            self.hp = self.max_hp;
            self.max_mana += 2;
            self.mana = self.max_mana;
            self.max_stamina += 3;
            self.stamina = self.max_stamina;
            true
        } else {
            false
        }
    }

    /// Get item count in inventory
    pub fn get_item_count(&self, item_key: &str) -> u32 {
        self.inventory
            .iter()
            .filter(|i| i.item_key == item_key)
            .map(|i| i.quantity)
            .sum()
    }

    /// Add item to inventory
    pub fn add_item(&mut self, item_key: &str, quantity: u32) -> bool {
        // Check if item is stackable
        if let Some(item_def) = super::data::get_item(item_key) {
            if item_def.stackable {
                // Find existing stack
                if let Some(slot) = self.inventory.iter_mut().find(|i| i.item_key == item_key) {
                    slot.quantity += quantity;
                    return true;
                }
            }
        }

        // Check inventory space
        if self.inventory.len() as u32 >= self.max_inventory_slots {
            return false;
        }

        self.inventory.push(InventoryItem::new(item_key, quantity));
        true
    }

    /// Remove item from inventory
    pub fn remove_item(&mut self, item_key: &str, quantity: u32) -> bool {
        let mut remaining = quantity;

        for slot in self.inventory.iter_mut() {
            if slot.item_key == item_key && remaining > 0 {
                let take = slot.quantity.min(remaining);
                slot.quantity -= take;
                remaining -= take;
            }
        }

        // Remove empty slots
        self.inventory.retain(|i| i.quantity > 0);

        remaining == 0
    }

    /// Calculate total weight of inventory
    pub fn inventory_weight(&self) -> u32 {
        self.inventory
            .iter()
            .map(|slot| {
                super::data::get_item(&slot.item_key)
                    .map(|item| item.weight * slot.quantity)
                    .unwrap_or(0)
            })
            .sum()
    }

    /// Calculate attack power
    pub fn attack_power(&self) -> i32 {
        let base = self.strength / 2;
        let weapon_power = self
            .equipped_weapon
            .as_ref()
            .and_then(|key| super::data::get_item(key))
            .map(|item| item.power)
            .unwrap_or(3); // Bare fists

        let skill_bonus = if let Some(ref weapon_key) = self.equipped_weapon {
            if let Some(item) = super::data::get_item(weapon_key) {
                if let Some(skill) = item.required_skill {
                    self.get_skill(skill) as i32 / 10
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            self.get_skill("wrestling") as i32 / 10
        };

        let tactics_bonus = self.get_skill("tactics") as i32 / 20;

        base + weapon_power + skill_bonus + tactics_bonus
    }

    /// Calculate defense
    pub fn defense(&self) -> i32 {
        let base = self.dexterity / 4;

        let armor_defense = self
            .equipped_armor
            .as_ref()
            .and_then(|key| super::data::get_item(key))
            .map(|item| item.power)
            .unwrap_or(0);

        let shield_defense = self
            .equipped_shield
            .as_ref()
            .and_then(|key| super::data::get_item(key))
            .map(|item| item.power)
            .unwrap_or(0);

        let parry_bonus = self.get_skill("parrying") as i32 / 20;

        base + armor_defense + shield_defense + parry_bonus
    }

    /// Check if character can cast a spell
    pub fn can_cast(&self, spell_key: &str) -> bool {
        if let Some(spell) = super::data::get_spell(spell_key) {
            if self.mana < spell.mana_cost {
                return false;
            }
            if self.get_skill("magery") < spell.required_magery {
                return false;
            }
            // Check reagents
            for (reagent, amount) in spell.reagents {
                if self.get_item_count(reagent) < *amount {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Die and mark as dead
    pub fn die(&mut self) {
        self.is_dead = true;
        self.deaths += 1;
        // Drop some gold on death
        let gold_lost = self.gold / 4;
        self.gold -= gold_lost;
    }

    /// Resurrect at a shrine/healer
    pub fn resurrect(&mut self) {
        self.is_dead = false;
        self.hp = self.max_hp / 4; // Resurrect with 25% HP
        self.mana = self.max_mana / 4;
        self.stamina = self.max_stamina / 4;
    }

    /// Full heal (at healer NPC)
    pub fn full_heal(&mut self) {
        self.hp = self.max_hp;
        self.mana = self.max_mana;
        self.stamina = self.max_stamina;
    }

    /// Check quest progress and update
    pub fn update_quest_kill(&mut self, monster_key: &str) {
        for quest in self.active_quests.iter_mut() {
            if let Some(quest_def) = super::data::get_quest(&quest.quest_key) {
                if let Some((target, _count)) = &quest_def.requirements.kill_monsters {
                    if *target == monster_key {
                        quest.kills += 1;
                    }
                }
            }
        }
    }

    /// Check if a quest is completable
    pub fn can_complete_quest(&self, quest_key: &str) -> bool {
        if let Some(progress) = self.active_quests.iter().find(|q| q.quest_key == quest_key) {
            if let Some(quest_def) = super::data::get_quest(quest_key) {
                // Check kill requirement
                if let Some((_, count)) = &quest_def.requirements.kill_monsters {
                    if progress.kills < *count {
                        return false;
                    }
                }
                // Check collect requirement
                if let Some((item, count)) = &quest_def.requirements.collect_items {
                    if self.get_item_count(item) < *count {
                        return false;
                    }
                }
                // Check visit requirement
                if quest_def.requirements.visit_zone.is_some() && !progress.visited {
                    return false;
                }
                // Check skill requirement
                if let Some((skill, level)) = &quest_def.requirements.skill_level {
                    if self.get_skill(skill) < *level {
                        return false;
                    }
                }
                return true;
            }
        }
        false
    }

    /// Complete a quest and claim rewards
    pub fn complete_quest(&mut self, quest_key: &str) -> bool {
        if !self.can_complete_quest(quest_key) {
            return false;
        }

        if let Some(quest_def) = super::data::get_quest(quest_key) {
            // Remove collected items if required
            if let Some((item, count)) = &quest_def.requirements.collect_items {
                self.remove_item(item, *count);
            }

            // Grant rewards
            self.gold += quest_def.reward.gold;
            self.add_xp(quest_def.reward.xp);

            for (item_key, qty) in quest_def.reward.items {
                self.add_item(item_key, *qty);
            }

            // Move quest to completed
            self.active_quests.retain(|q| q.quest_key != quest_key);
            self.completed_quests.push(quest_key.to_string());

            return true;
        }

        false
    }
}

/// Global game state (for the persistent world)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    /// Current character being played
    pub character: Option<Character>,
    /// World time (in-game hours since start)
    pub world_time: u64,
    /// Active monsters in current zone
    pub zone_monsters: Vec<super::data::Monster>,
    /// Other players visible in zone (populated from DB)
    pub visible_players: Vec<VisiblePlayer>,
    /// Current combat state
    pub combat: Option<CombatState>,
    /// UI selection states
    pub selected_item: Option<usize>,
    pub selected_npc: Option<String>,
    pub shop_mode: bool,
    pub input_buffer: String,
}

/// Simplified player info for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisiblePlayer {
    pub name: String,
    pub level: u32,
    pub x: i32,
    pub y: i32,
    pub guild: Option<String>,
}

/// Active combat state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub monster: super::data::Monster,
    pub player_acted: bool,
    pub combat_log: Vec<String>,
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_character(character: Character) -> Self {
        Self {
            character: Some(character),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let char = Character::new("TestHero", 1);
        assert_eq!(char.name, "TestHero");
        assert_eq!(char.user_id, 1);
        assert_eq!(char.level(), 1);
        assert_eq!(char.gold, 100);
    }

    #[test]
    fn test_skill_system() {
        let mut char = Character::new("Warrior", 1);
        char.skills.insert("swordsmanship".to_string(), 50);
        assert_eq!(char.get_skill("swordsmanship"), 50);
        assert_eq!(char.get_skill("magery"), 0);
    }

    #[test]
    fn test_xp_and_leveling() {
        let mut char = Character::new("Hero", 1);
        assert_eq!(char.level(), 1);

        char.add_xp(400); // Should be level 2
        assert_eq!(char.level(), 2);

        char.add_xp(500); // Should be level 3
        assert_eq!(char.level(), 3);
    }

    #[test]
    fn test_inventory() {
        let mut char = Character::new("Merchant", 1);

        // Starting inventory
        assert!(char.get_item_count("bread") > 0);

        // Add items
        char.add_item("iron_ore", 10);
        assert_eq!(char.get_item_count("iron_ore"), 10);

        // Remove items
        char.remove_item("iron_ore", 5);
        assert_eq!(char.get_item_count("iron_ore"), 5);
    }

    #[test]
    fn test_death_and_resurrection() {
        let mut char = Character::new("Mortal", 1);
        char.gold = 100;

        char.die();
        assert!(char.is_dead);
        assert_eq!(char.deaths, 1);
        assert_eq!(char.gold, 75); // Lost 25%

        char.resurrect();
        assert!(!char.is_dead);
        assert_eq!(char.hp, char.max_hp / 4);
    }

    #[test]
    fn test_position() {
        let pos = Position::new("britain", 10, 20);
        assert_eq!(pos.zone, "britain");
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    }

    #[test]
    fn test_serialization() {
        let char = Character::new("SaveTest", 1);
        let json = serde_json::to_string(&char).unwrap();
        let restored: Character = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, char.name);
        assert_eq!(restored.gold, char.gold);
    }
}
