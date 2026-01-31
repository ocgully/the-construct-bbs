//! Game state for Depths of Diablo
//!
//! Contains character state, run state, and meta-progression.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::combat::{ActiveBuff, Skill};
use super::data::CharacterClass;
use super::dungeon::Dungeon;
use super::items::{AffixStat, EquipSlot, Item};

/// A player character in a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub class: CharacterClass,
    pub level: u32,
    pub experience: i32,
    pub exp_to_next: i32,

    // Combat stats
    pub health: i32,
    pub max_health: i32,
    pub mana: i32,
    pub max_mana: i32,

    // Attributes
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
    pub vitality: i32,

    // Position
    pub x: usize,
    pub y: usize,

    // Inventory and equipment
    pub inventory: Vec<Item>,
    pub equipment: HashMap<String, Item>, // slot name -> item
    pub gold: i32,

    // Skills
    pub skills: Vec<Skill>,
    pub active_skill_index: usize,

    // Buffs
    pub active_buffs: Vec<ActiveBuff>,

    // Potions (quick slots)
    pub health_potions: u32,
    pub mana_potions: u32,
}

impl Character {
    pub fn new(name: &str, class: CharacterClass) -> Self {
        let stats = class.base_stats();
        let starting_skills: Vec<Skill> = class
            .starting_skills()
            .iter()
            .map(|s| Skill::new(s))
            .collect();

        Character {
            name: name.to_string(),
            class,
            level: 1,
            experience: 0,
            exp_to_next: 100,

            health: stats.health,
            max_health: stats.health,
            mana: stats.mana,
            max_mana: stats.mana,

            strength: stats.strength,
            dexterity: stats.dexterity,
            intelligence: stats.intelligence,
            vitality: stats.vitality,

            x: 0,
            y: 0,

            inventory: Vec::new(),
            equipment: HashMap::new(),
            gold: 100,

            skills: starting_skills,
            active_skill_index: 0,

            active_buffs: Vec::new(),

            health_potions: 3,
            mana_potions: 2,
        }
    }

    /// Calculate total stat from base + equipment
    pub fn total_strength(&self) -> i32 {
        self.strength + self.equipment_bonus(AffixStat::Strength)
    }

    pub fn total_dexterity(&self) -> i32 {
        self.dexterity + self.equipment_bonus(AffixStat::Dexterity)
    }

    pub fn total_intelligence(&self) -> i32 {
        self.intelligence + self.equipment_bonus(AffixStat::Intelligence)
    }

    pub fn total_vitality(&self) -> i32 {
        self.vitality + self.equipment_bonus(AffixStat::Vitality)
    }

    /// Get stat bonus from all equipment
    fn equipment_bonus(&self, stat: AffixStat) -> i32 {
        self.equipment
            .values()
            .map(|item| item.get_stat_bonus(stat))
            .sum()
    }

    /// Get equipped items as references
    pub fn equipped_items(&self) -> Vec<&Item> {
        self.equipment.values().collect()
    }

    /// Equip an item
    pub fn equip(&mut self, item: Item) -> Option<Item> {
        let slot = item.slot()?;
        let slot_name = format!("{:?}", slot);

        // Handle two-handed weapons
        if slot == EquipSlot::TwoHand {
            // Remove main and off hand
            let main = self.equipment.remove("MainHand");
            let off = self.equipment.remove("OffHand");

            self.equipment.insert(slot_name, item);

            // Return first unequipped item to inventory
            if let Some(m) = main {
                self.inventory.push(m);
            }
            return off;
        }

        self.equipment.insert(slot_name, item)
    }

    /// Unequip an item by slot
    pub fn unequip(&mut self, slot: &str) -> Option<Item> {
        self.equipment.remove(slot)
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, xp: i32) -> bool {
        self.experience += xp;
        let mut leveled_up = false;

        while self.experience >= self.exp_to_next {
            self.experience -= self.exp_to_next;
            self.level_up();
            leveled_up = true;
        }

        leveled_up
    }

    /// Level up the character
    fn level_up(&mut self) {
        self.level += 1;
        self.exp_to_next = (self.exp_to_next as f32 * 1.5) as i32;

        // Stat gains per level based on class
        let (str_gain, dex_gain, int_gain, vit_gain) = match self.class {
            CharacterClass::Warrior => (3, 1, 1, 2),
            CharacterClass::Rogue => (1, 3, 1, 1),
            CharacterClass::Sorcerer => (1, 1, 3, 1),
        };

        self.strength += str_gain;
        self.dexterity += dex_gain;
        self.intelligence += int_gain;
        self.vitality += vit_gain;

        // Update max health/mana
        let stats = self.class.base_stats();
        self.max_health = stats.health + self.vitality * 3 + (self.level as i32 - 1) * 5;
        self.max_mana = stats.mana + self.intelligence * 2 + (self.level as i32 - 1) * 3;

        // Full heal on level up
        self.health = self.max_health;
        self.mana = self.max_mana;
    }

    /// Use health potion
    pub fn use_health_potion(&mut self) -> bool {
        if self.health_potions > 0 && self.health < self.max_health {
            self.health_potions -= 1;
            let heal = self.max_health / 3; // Heal 33%
            self.health = (self.health + heal).min(self.max_health);
            true
        } else {
            false
        }
    }

    /// Use mana potion
    pub fn use_mana_potion(&mut self) -> bool {
        if self.mana_potions > 0 && self.mana < self.max_mana {
            self.mana_potions -= 1;
            let restore = self.max_mana / 3; // Restore 33%
            self.mana = (self.mana + restore).min(self.max_mana);
            true
        } else {
            false
        }
    }

    /// Get active skill
    pub fn active_skill(&self) -> Option<&Skill> {
        self.skills.get(self.active_skill_index)
    }

    /// Get active skill mutable
    pub fn active_skill_mut(&mut self) -> Option<&mut Skill> {
        self.skills.get_mut(self.active_skill_index)
    }

    /// Cycle to next skill
    pub fn next_skill(&mut self) {
        if !self.skills.is_empty() {
            self.active_skill_index = (self.active_skill_index + 1) % self.skills.len();
        }
    }

    /// Is character alive
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }
}

/// State of a dungeon run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: u64,
    pub seed: u64,
    pub current_floor: u32,
    pub max_floor_reached: u32,
    pub start_time_ms: u64,
    pub elapsed_time_ms: u64,
    pub in_town: bool,
    pub run_active: bool,
    pub run_completed: bool,
    pub run_failed: bool,
}

impl RunState {
    pub fn new(run_id: u64, seed: u64) -> Self {
        RunState {
            run_id,
            seed,
            current_floor: 1,
            max_floor_reached: 1,
            start_time_ms: 0,
            elapsed_time_ms: 0,
            in_town: true,
            run_active: true,
            run_completed: false,
            run_failed: false,
        }
    }

    pub fn descend(&mut self) {
        if self.current_floor < 20 {
            self.current_floor += 1;
            self.max_floor_reached = self.max_floor_reached.max(self.current_floor);
        }
    }

    pub fn ascend(&mut self) {
        if self.current_floor > 1 {
            self.current_floor -= 1;
        } else {
            self.in_town = true;
        }
    }

    pub fn enter_dungeon(&mut self) {
        self.in_town = false;
    }

    pub fn return_to_town(&mut self) {
        self.in_town = true;
    }

    pub fn complete_run(&mut self) {
        self.run_active = false;
        self.run_completed = true;
    }

    pub fn fail_run(&mut self) {
        self.run_active = false;
        self.run_failed = true;
    }

    /// Calculate soul essence reward
    pub fn soul_essence_reward(&self) -> i64 {
        let floor_bonus = self.max_floor_reached as i64 * 10;
        let completion_bonus = if self.run_completed { 500 } else { 0 };
        floor_bonus + completion_bonus
    }
}

/// Meta-progression that persists between runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaProgress {
    pub soul_essence: i64,
    pub highest_floor_ever: u32,
    pub total_runs: u32,
    pub successful_runs: u32,
    pub total_kills: u64,
    pub total_gold_earned: i64,

    // Unlocks
    pub town_upgrades: HashMap<String, u32>, // upgrade_key -> level
    pub unlocked_classes: Vec<CharacterClass>,
    pub permanent_bonuses: HashMap<String, i32>, // stat -> bonus

    // Stash (items kept between runs)
    pub stash: Vec<Item>,
}

impl MetaProgress {
    pub fn new() -> Self {
        MetaProgress {
            soul_essence: 0,
            highest_floor_ever: 0,
            total_runs: 0,
            successful_runs: 0,
            total_kills: 0,
            total_gold_earned: 0,

            town_upgrades: HashMap::new(),
            unlocked_classes: vec![CharacterClass::Warrior], // Start with warrior
            permanent_bonuses: HashMap::new(),

            stash: Vec::new(),
        }
    }

    /// Unlock a class
    pub fn unlock_class(&mut self, class: CharacterClass) -> bool {
        if self.unlocked_classes.contains(&class) {
            return false;
        }

        let cost = match class {
            CharacterClass::Warrior => 0,   // Free
            CharacterClass::Rogue => 200,   // 200 soul essence
            CharacterClass::Sorcerer => 200,
        };

        if self.soul_essence >= cost {
            self.soul_essence -= cost;
            self.unlocked_classes.push(class);
            true
        } else {
            false
        }
    }

    /// Upgrade a town facility
    pub fn upgrade_town(&mut self, upgrade_key: &str, cost: i64) -> bool {
        if self.soul_essence >= cost {
            self.soul_essence -= cost;
            let level = self.town_upgrades.entry(upgrade_key.to_string()).or_insert(0);
            *level += 1;
            true
        } else {
            false
        }
    }

    /// Get town upgrade level
    pub fn get_upgrade_level(&self, upgrade_key: &str) -> u32 {
        self.town_upgrades.get(upgrade_key).copied().unwrap_or(0)
    }

    /// Add permanent stat bonus
    pub fn add_permanent_bonus(&mut self, stat: &str, bonus: i32, cost: i64) -> bool {
        if self.soul_essence >= cost {
            self.soul_essence -= cost;
            let current = self.permanent_bonuses.entry(stat.to_string()).or_insert(0);
            *current += bonus;
            true
        } else {
            false
        }
    }

    /// Get permanent bonus for a stat
    pub fn get_permanent_bonus(&self, stat: &str) -> i32 {
        self.permanent_bonuses.get(stat).copied().unwrap_or(0)
    }

    /// Record run completion
    pub fn record_run(&mut self, run: &RunState, kills: u64, gold: i64) {
        self.total_runs += 1;
        if run.run_completed {
            self.successful_runs += 1;
        }
        self.highest_floor_ever = self.highest_floor_ever.max(run.max_floor_reached);
        self.total_kills += kills;
        self.total_gold_earned += gold;
        self.soul_essence += run.soul_essence_reward();
    }
}

impl Default for MetaProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete game state for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub user_id: i64,
    pub handle: String,

    // Current character (if in run)
    pub character: Option<Character>,

    // Current run state
    pub run: Option<RunState>,

    // Current dungeon (transient, regenerated from seed)
    #[serde(skip)]
    pub dungeon: Option<Dungeon>,

    // Meta progression
    pub meta: MetaProgress,

    // Session info
    pub last_save_time: Option<String>,
    pub messages: Vec<String>,
}

impl GameState {
    pub fn new(user_id: i64, handle: &str) -> Self {
        GameState {
            user_id,
            handle: handle.to_string(),
            character: None,
            run: None,
            dungeon: None,
            meta: MetaProgress::new(),
            last_save_time: None,
            messages: Vec::new(),
        }
    }

    /// Start a new run
    pub fn start_run(&mut self, character_name: &str, class: CharacterClass, seed: u64) {
        let mut character = Character::new(character_name, class);

        // Apply permanent bonuses
        character.strength += self.meta.get_permanent_bonus("strength");
        character.dexterity += self.meta.get_permanent_bonus("dexterity");
        character.intelligence += self.meta.get_permanent_bonus("intelligence");
        character.vitality += self.meta.get_permanent_bonus("vitality");

        // Starting equipment based on town upgrades
        let blacksmith_level = self.meta.get_upgrade_level("blacksmith");
        if blacksmith_level > 0 {
            // Give better starting gear
            character.health_potions += blacksmith_level;
        }

        self.character = Some(character);
        self.run = Some(RunState::new(self.meta.total_runs as u64 + 1, seed));

        // Generate first floor dungeon
        self.dungeon = Some(Dungeon::generate(1, seed));
    }

    /// End current run
    pub fn end_run(&mut self, success: bool) {
        if let Some(ref mut run) = self.run {
            if success {
                run.complete_run();
            } else {
                run.fail_run();
            }

            // Calculate rewards
            let kills = 0u64; // TODO: track kills
            let gold = self.character.as_ref().map(|c| c.gold as i64).unwrap_or(0);
            self.meta.record_run(run, kills, gold);
        }

        self.character = None;
        self.run = None;
        self.dungeon = None;
    }

    /// Enter dungeon from town
    pub fn enter_dungeon(&mut self) {
        if let Some(ref mut run) = self.run {
            run.enter_dungeon();

            // Generate or regenerate dungeon for current floor
            self.dungeon = Some(Dungeon::generate(run.current_floor, run.seed));

            // Place character at start
            if let (Some(ref mut char), Some(ref dungeon)) = (&mut self.character, &self.dungeon) {
                char.x = dungeon.start_pos.0;
                char.y = dungeon.start_pos.1;
            }
        }
    }

    /// Return to town
    pub fn return_to_town(&mut self) {
        if let Some(ref mut run) = self.run {
            run.return_to_town();
        }
        self.dungeon = None;
    }

    /// Descend to next floor
    pub fn descend(&mut self) {
        if let Some(ref mut run) = self.run {
            run.descend();

            // Generate new floor
            self.dungeon = Some(Dungeon::generate(run.current_floor, run.seed));

            // Place character at start
            if let (Some(ref mut char), Some(ref dungeon)) = (&mut self.character, &self.dungeon) {
                char.x = dungeon.start_pos.0;
                char.y = dungeon.start_pos.1;
            }
        }
    }

    /// Add a message to display
    pub fn add_message(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
        // Keep last 5 messages
        if self.messages.len() > 5 {
            self.messages.remove(0);
        }
    }

    /// Clear messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Is in an active run
    pub fn is_in_run(&self) -> bool {
        self.run.as_ref().map(|r| r.run_active).unwrap_or(false)
    }

    /// Is in town
    pub fn is_in_town(&self) -> bool {
        self.run.as_ref().map(|r| r.in_town).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let char = Character::new("TestHero", CharacterClass::Warrior);
        assert_eq!(char.name, "TestHero");
        assert_eq!(char.class, CharacterClass::Warrior);
        assert_eq!(char.level, 1);
        assert!(char.health > 0);
    }

    #[test]
    fn test_level_up() {
        let mut char = Character::new("Test", CharacterClass::Warrior);
        let old_health = char.max_health;
        let old_strength = char.strength;

        char.add_experience(150); // More than needed to level

        assert_eq!(char.level, 2);
        assert!(char.max_health > old_health);
        assert!(char.strength > old_strength);
    }

    #[test]
    fn test_potions() {
        let mut char = Character::new("Test", CharacterClass::Warrior);
        char.health = 50;

        assert!(char.use_health_potion());
        assert!(char.health > 50);
        assert_eq!(char.health_potions, 2);
    }

    #[test]
    fn test_run_state() {
        let mut run = RunState::new(1, 12345);
        assert!(run.in_town);
        assert_eq!(run.current_floor, 1);

        run.enter_dungeon();
        assert!(!run.in_town);

        run.descend();
        assert_eq!(run.current_floor, 2);

        run.return_to_town();
        assert!(run.in_town);
    }

    #[test]
    fn test_meta_progression() {
        let mut meta = MetaProgress::new();
        assert_eq!(meta.unlocked_classes.len(), 1); // Only warrior

        meta.soul_essence = 300;
        assert!(meta.unlock_class(CharacterClass::Rogue));
        assert_eq!(meta.unlocked_classes.len(), 2);
        assert_eq!(meta.soul_essence, 100);
    }

    #[test]
    fn test_game_state_run() {
        let mut state = GameState::new(1, "TestPlayer");

        state.start_run("Hero", CharacterClass::Warrior, 12345);
        assert!(state.is_in_run());
        assert!(state.is_in_town());

        state.enter_dungeon();
        assert!(!state.is_in_town());
        assert!(state.dungeon.is_some());

        state.end_run(false);
        assert!(!state.is_in_run());
        assert_eq!(state.meta.total_runs, 1);
    }

    #[test]
    fn test_soul_essence_reward() {
        let mut run = RunState::new(1, 12345);
        run.max_floor_reached = 10;

        // Failed run
        assert_eq!(run.soul_essence_reward(), 100); // 10 * 10

        // Completed run
        run.complete_run();
        assert_eq!(run.soul_essence_reward(), 600); // 100 + 500
    }
}
