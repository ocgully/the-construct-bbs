//! Realm of Ralnar - Game State
//! Persistent player data that gets serialized to the database

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::data::config;

/// Cardinal directions for player facing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Get the direction name for display
    pub fn name(&self) -> &'static str {
        match self {
            Direction::Up => "North",
            Direction::Down => "South",
            Direction::Left => "West",
            Direction::Right => "East",
        }
    }

    /// Get the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Get delta for movement
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Down
    }
}

/// Character class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    /// Herbert - The main hero, strong melee fighter
    Warrior,
    /// Valeran - Holy warrior, balanced with light magic
    Paladin,
    /// Sera - Healer and support magic
    Cleric,
    /// Korrath - Heavy armor tank
    Knight,
    /// Zanth - Offensive magic specialist
    Wizard,
    /// Captain John - Agile fighter, good with swords
    Swashbuckler,
    /// Nomodest - Quick, can steal, high luck
    Thief,
    /// Elder Morath - Ancient wisdom, powerful late-game
    Sage,
    /// Lyra - Ranged combat specialist
    Archer,
}

impl CharacterClass {
    /// Get the display name for this class
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Paladin => "Paladin",
            CharacterClass::Cleric => "Cleric",
            CharacterClass::Knight => "Knight",
            CharacterClass::Wizard => "Wizard",
            CharacterClass::Swashbuckler => "Swashbuckler",
            CharacterClass::Thief => "Thief",
            CharacterClass::Sage => "Sage",
            CharacterClass::Archer => "Archer",
        }
    }

    /// Get starting stats for this class: (hp, mp, str, agi, int, vit, luck)
    pub fn base_stats(&self) -> (i32, i32, i32, i32, i32, i32, i32) {
        match self {
            CharacterClass::Warrior => (45, 0, 14, 8, 4, 12, 6),
            CharacterClass::Paladin => (40, 15, 12, 7, 8, 10, 7),
            CharacterClass::Cleric => (30, 30, 6, 6, 12, 8, 8),
            CharacterClass::Knight => (50, 5, 10, 5, 5, 14, 5),
            CharacterClass::Wizard => (25, 40, 4, 7, 16, 5, 7),
            CharacterClass::Swashbuckler => (35, 5, 10, 14, 6, 8, 10),
            CharacterClass::Thief => (28, 0, 8, 16, 5, 6, 14),
            CharacterClass::Sage => (28, 35, 5, 6, 14, 6, 10),
            CharacterClass::Archer => (32, 10, 8, 14, 8, 7, 12),
        }
    }

    /// Get stat growth per level: (hp, mp, str, agi, int, vit, luck)
    pub fn level_up_stats(&self) -> (i32, i32, i32, i32, i32, i32, i32) {
        match self {
            CharacterClass::Warrior => (12, 0, 4, 2, 1, 3, 1),
            CharacterClass::Paladin => (10, 4, 3, 2, 2, 3, 2),
            CharacterClass::Cleric => (7, 7, 2, 2, 4, 2, 2),
            CharacterClass::Knight => (14, 1, 3, 1, 1, 4, 1),
            CharacterClass::Wizard => (5, 9, 1, 2, 5, 1, 2),
            CharacterClass::Swashbuckler => (9, 1, 3, 4, 2, 2, 3),
            CharacterClass::Thief => (6, 0, 2, 5, 1, 2, 4),
            CharacterClass::Sage => (6, 8, 1, 2, 4, 2, 3),
            CharacterClass::Archer => (8, 3, 2, 4, 2, 2, 3),
        }
    }
}

/// Base stats for a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub vitality: i32,
    pub luck: i32,
}

impl CharacterStats {
    /// Create new stats with given values
    pub fn new(strength: i32, agility: i32, intelligence: i32, vitality: i32, luck: i32) -> Self {
        Self {
            strength,
            agility,
            intelligence,
            vitality,
            luck,
        }
    }

    /// Create stats from class defaults
    pub fn from_class(class: CharacterClass) -> Self {
        let (_, _, str, agi, int, vit, luck) = class.base_stats();
        Self::new(str, agi, int, vit, luck)
    }
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self::new(10, 10, 10, 10, 10)
    }
}

/// Equipment slots for a character
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub shield: Option<String>,
    pub helmet: Option<String>,
    pub accessory: Option<String>,
}

impl Equipment {
    /// Check if any equipment is equipped
    pub fn has_equipment(&self) -> bool {
        self.weapon.is_some()
            || self.armor.is_some()
            || self.shield.is_some()
            || self.helmet.is_some()
            || self.accessory.is_some()
    }
}

/// A single party member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMember {
    /// Unique identifier for this character
    pub id: String,
    /// Display name
    pub name: String,
    /// Character class
    pub class: CharacterClass,
    /// Current level
    pub level: u8,
    /// Total experience points
    pub exp: u32,
    /// Current hit points
    pub hp: i32,
    /// Maximum hit points
    pub hp_max: i32,
    /// Current magic points
    pub mp: i32,
    /// Maximum magic points
    pub mp_max: i32,
    /// Base stats
    pub stats: CharacterStats,
    /// Equipped items
    pub equipment: Equipment,
    /// Whether this character is one of the brothers (Herbert or his brothers)
    /// Brothers cannot be removed from the party
    pub is_brother: bool,
}

impl PartyMember {
    /// Create a new party member with class-appropriate starting stats
    pub fn new(id: String, name: String, class: CharacterClass, is_brother: bool) -> Self {
        let (hp, mp, str, agi, int, vit, luck) = class.base_stats();

        Self {
            id,
            name,
            class,
            level: 1,
            exp: 0,
            hp,
            hp_max: hp,
            mp,
            mp_max: mp,
            stats: CharacterStats::new(str, agi, int, vit, luck),
            equipment: Equipment::default(),
            is_brother,
        }
    }

    /// Check if character is alive
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Get attack power (strength + weapon bonus)
    pub fn attack_power(&self) -> i32 {
        // Base attack from strength
        let base = self.stats.strength;
        // TODO: Add weapon bonus when equipment data is implemented
        base
    }

    /// Get defense power (vitality + armor bonus)
    pub fn defense_power(&self) -> i32 {
        let base = self.stats.vitality;
        // TODO: Add armor bonus when equipment data is implemented
        base
    }

    /// Get magic power
    pub fn magic_power(&self) -> i32 {
        self.stats.intelligence
    }

    /// Get speed for turn order
    pub fn speed(&self) -> i32 {
        self.stats.agility
    }

    /// Calculate experience needed for next level
    pub fn exp_to_next_level(&self) -> u32 {
        config::exp_for_level(self.level + 1)
    }

    /// Check if character can level up
    pub fn can_level_up(&self) -> bool {
        self.exp >= self.exp_to_next_level()
    }

    /// Apply level up, returns stat gains
    pub fn level_up(&mut self) -> Option<LevelUpGains> {
        if !self.can_level_up() {
            return None;
        }

        self.level += 1;
        let (hp_gain, mp_gain, str_gain, agi_gain, int_gain, vit_gain, luck_gain) =
            self.class.level_up_stats();

        self.hp_max += hp_gain;
        self.mp_max += mp_gain;
        self.stats.strength += str_gain;
        self.stats.agility += agi_gain;
        self.stats.intelligence += int_gain;
        self.stats.vitality += vit_gain;
        self.stats.luck += luck_gain;

        // Full heal on level up
        self.hp = self.hp_max;
        self.mp = self.mp_max;

        Some(LevelUpGains {
            new_level: self.level,
            hp: hp_gain,
            mp: mp_gain,
            strength: str_gain,
            agility: agi_gain,
            intelligence: int_gain,
            vitality: vit_gain,
            luck: luck_gain,
        })
    }

    /// Take damage, returns actual damage taken
    pub fn take_damage(&mut self, amount: i32) -> i32 {
        let actual = amount.min(self.hp);
        self.hp -= actual;
        actual
    }

    /// Heal HP
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// Restore MP
    pub fn restore_mp(&mut self, amount: i32) {
        self.mp = (self.mp + amount).min(self.mp_max);
    }

    /// Revive from KO with percentage of max HP
    pub fn revive(&mut self, hp_percent: u8) {
        if self.hp <= 0 {
            self.hp = ((self.hp_max as u32 * hp_percent as u32) / 100).max(1) as i32;
        }
    }
}

/// Level up stat gains for display
#[derive(Debug, Clone)]
pub struct LevelUpGains {
    pub new_level: u8,
    pub hp: i32,
    pub mp: i32,
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub vitality: i32,
    pub luck: i32,
}

/// The player's party
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Party {
    pub members: Vec<PartyMember>,
}

impl Party {
    /// Maximum party size
    pub const MAX_SIZE: usize = 4;

    /// Create an empty party
    pub fn new() -> Self {
        Self { members: Vec::new() }
    }

    /// Add a member to the party
    pub fn add_member(&mut self, member: PartyMember) -> bool {
        if self.members.len() < Self::MAX_SIZE {
            self.members.push(member);
            true
        } else {
            false
        }
    }

    /// Remove a member by ID (cannot remove brothers)
    pub fn remove_member(&mut self, id: &str) -> Result<PartyMember, &'static str> {
        let index = self.members.iter().position(|m| m.id == id);

        match index {
            Some(idx) => {
                if self.members[idx].is_brother {
                    Err("Cannot remove a brother from the party")
                } else {
                    Ok(self.members.remove(idx))
                }
            }
            None => Err("Member not found in party"),
        }
    }

    /// Get a member by ID
    pub fn get_member(&self, id: &str) -> Option<&PartyMember> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Get a mutable member by ID
    pub fn get_member_mut(&mut self, id: &str) -> Option<&mut PartyMember> {
        self.members.iter_mut().find(|m| m.id == id)
    }

    /// Get all living party members
    pub fn living_members(&self) -> Vec<&PartyMember> {
        self.members.iter().filter(|m| m.is_alive()).collect()
    }

    /// Check if party is alive
    pub fn is_alive(&self) -> bool {
        self.members.iter().any(|m| m.is_alive())
    }

    /// Get party leader (first member)
    pub fn leader(&self) -> Option<&PartyMember> {
        self.members.first()
    }

    /// Get average party level
    pub fn average_level(&self) -> u8 {
        if self.members.is_empty() {
            return 1;
        }
        let total: u32 = self.members.iter().map(|m| m.level as u32).sum();
        (total / self.members.len() as u32) as u8
    }

    /// Fully heal all party members
    pub fn full_heal(&mut self) {
        for member in &mut self.members {
            member.hp = member.hp_max;
            member.mp = member.mp_max;
        }
    }

    /// Rest at inn (heal HP/MP but don't revive)
    pub fn rest_at_inn(&mut self) {
        for member in &mut self.members {
            if member.is_alive() {
                member.hp = member.hp_max;
                member.mp = member.mp_max;
            }
        }
    }

    /// Distribute experience to living members
    pub fn distribute_exp(&mut self, total_exp: u32) -> Vec<(String, Option<LevelUpGains>)> {
        let living_count = self.living_members().len();
        if living_count == 0 {
            return Vec::new();
        }

        let per_member = total_exp / living_count as u32;
        let mut results = Vec::new();

        for member in &mut self.members {
            if member.is_alive() {
                member.exp += per_member;
                let level_up = member.level_up();
                results.push((member.name.clone(), level_up));
            }
        }

        results
    }
}

/// Inventory item with quantity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub key: String,
    pub quantity: u32,
}

/// Player inventory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<InventoryItem>,
}

impl Inventory {
    /// Create empty inventory
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Get quantity of an item
    pub fn count(&self, key: &str) -> u32 {
        self.items
            .iter()
            .find(|i| i.key == key)
            .map(|i| i.quantity)
            .unwrap_or(0)
    }

    /// Add items to inventory
    pub fn add(&mut self, key: &str, quantity: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.key == key) {
            item.quantity += quantity;
        } else {
            self.items.push(InventoryItem {
                key: key.to_string(),
                quantity,
            });
        }
    }

    /// Remove items from inventory
    pub fn remove(&mut self, key: &str, quantity: u32) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.key == key) {
            if item.quantity >= quantity {
                item.quantity -= quantity;
                if item.quantity == 0 {
                    self.items.retain(|i| i.key != key);
                }
                return true;
            }
        }
        false
    }

    /// Check if has at least this quantity
    pub fn has(&self, key: &str, quantity: u32) -> bool {
        self.count(key) >= quantity
    }
}

/// The main game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// User ID from BBS system
    pub user_id: i64,
    /// Player's party
    pub party: Party,
    /// Player's inventory
    pub inventory: Inventory,
    /// Gold currency
    pub gold: u32,
    /// Current map ID
    pub current_map: String,
    /// Position on current map (x, y)
    pub position: (u32, u32),
    /// Direction facing
    pub direction: Direction,
    /// Story progress flags
    pub story_flags: HashMap<String, bool>,
    /// Shrines destroyed (main quest progress) - 5 elemental shrines
    pub shrines_destroyed: [bool; 5],
    /// World phase (0-6) - affects enemy strength, available areas, etc.
    pub world_phase: u8,
    /// Total play time in seconds
    pub play_time_seconds: u64,
    /// Last message to display
    pub last_message: Option<String>,
}

impl GameState {
    /// Create a new game state for a user
    pub fn new(user_id: i64, player_name: String) -> Self {
        let mut party = Party::new();

        // Create Herbert, the main hero
        let herbert = PartyMember::new(
            "herbert".to_string(),
            player_name,
            CharacterClass::Warrior,
            true, // Is a brother
        );
        party.add_member(herbert);

        // Starting inventory
        let mut inventory = Inventory::new();
        inventory.add("potion", 3);

        Self {
            user_id,
            party,
            inventory,
            gold: 100,
            current_map: "starting_village".to_string(),
            position: (5, 5),
            direction: Direction::Down,
            story_flags: HashMap::new(),
            shrines_destroyed: [false; 5],
            world_phase: 0,
            play_time_seconds: 0,
            last_message: None,
        }
    }

    /// Check if a story flag is set
    pub fn has_flag(&self, flag: &str) -> bool {
        self.story_flags.get(flag).copied().unwrap_or(false)
    }

    /// Set a story flag
    pub fn set_flag(&mut self, flag: &str) {
        self.story_flags.insert(flag.to_string(), true);
    }

    /// Clear a story flag
    pub fn clear_flag(&mut self, flag: &str) {
        self.story_flags.remove(flag);
    }

    /// Get number of shrines destroyed
    pub fn shrines_destroyed_count(&self) -> usize {
        self.shrines_destroyed.iter().filter(|&&d| d).count()
    }

    /// Check if a specific shrine has been found/destroyed
    /// Shrine names: shrine_of_fire, shrine_of_water, shrine_of_earth, shrine_of_wind, shrine_of_shadows
    /// Also accepts short names: shrine_1 through shrine_5
    pub fn has_shrine(&self, shrine_name: &str) -> bool {
        self.shrine_index(shrine_name)
            .map(|i| self.shrines_destroyed[i])
            .unwrap_or(false)
    }

    /// Mark a shrine as found/destroyed
    /// Accepts both full names (shrine_of_fire) and short names (shrine_1)
    pub fn find_shrine(&mut self, shrine_name: &str) {
        if let Some(idx) = self.shrine_index(shrine_name) {
            self.shrines_destroyed[idx] = true;
        }
    }

    /// Get shrine index from name
    fn shrine_index(&self, shrine_name: &str) -> Option<usize> {
        match shrine_name {
            "shrine_of_fire" | "shrine_1" => Some(0),
            "shrine_of_water" | "shrine_2" => Some(1),
            "shrine_of_earth" | "shrine_3" => Some(2),
            "shrine_of_wind" | "shrine_4" => Some(3),
            "shrine_of_shadows" | "shrine_5" => Some(4),
            _ => None,
        }
    }

    /// Update world phase based on story progress
    pub fn update_world_phase(&mut self) {
        // World phases:
        // 0: Beginning - only starting area accessible
        // 1: After leaving starting village
        // 2: First shrine destroyed
        // 3: Three shrines destroyed
        // 4: All shrines destroyed
        // 5: Final dungeon unlocked
        // 6: Game complete
        let shrines = self.shrines_destroyed_count();
        self.world_phase = match shrines {
            0 if self.has_flag("left_village") => 1,
            0 => 0,
            1..=2 => 2,
            3..=4 => 3,
            5 if self.has_flag("final_dungeon") => 5,
            5 => 4,
            _ => self.world_phase,
        };
    }

    /// Add gold (with overflow protection)
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

    /// Move in a direction
    pub fn move_direction(&mut self, dir: Direction) {
        let (dx, dy) = dir.delta();
        self.position.0 = (self.position.0 as i32 + dx).max(0) as u32;
        self.position.1 = (self.position.1 as i32 + dy).max(0) as u32;
        self.direction = dir;
    }

    /// Add play time
    pub fn add_play_time(&mut self, seconds: u64) {
        self.play_time_seconds += seconds;
    }

    /// Format play time as HH:MM:SS
    pub fn formatted_play_time(&self) -> String {
        let hours = self.play_time_seconds / 3600;
        let minutes = (self.play_time_seconds % 3600) / 60;
        let seconds = self.play_time_seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn test_character_class_stats() {
        let (hp, mp, str, agi, int, vit, luck) = CharacterClass::Warrior.base_stats();
        assert!(hp > 40);
        assert!(str > 10);
        assert_eq!(mp, 0); // Warriors have no MP
    }

    #[test]
    fn test_party_member_creation() {
        let member = PartyMember::new(
            "test".to_string(),
            "Test Hero".to_string(),
            CharacterClass::Warrior,
            false,
        );
        assert_eq!(member.level, 1);
        assert!(member.hp > 0);
        assert!(member.is_alive());
    }

    #[test]
    fn test_party_member_damage_and_heal() {
        let mut member = PartyMember::new(
            "test".to_string(),
            "Test Hero".to_string(),
            CharacterClass::Warrior,
            false,
        );
        let initial_hp = member.hp;

        member.take_damage(10);
        assert_eq!(member.hp, initial_hp - 10);

        member.heal(5);
        assert_eq!(member.hp, initial_hp - 5);

        // Heal shouldn't exceed max
        member.heal(1000);
        assert_eq!(member.hp, member.hp_max);
    }

    #[test]
    fn test_party_management() {
        let mut party = Party::new();

        let hero = PartyMember::new(
            "hero".to_string(),
            "Hero".to_string(),
            CharacterClass::Warrior,
            true,
        );
        assert!(party.add_member(hero));

        let ally = PartyMember::new(
            "ally".to_string(),
            "Ally".to_string(),
            CharacterClass::Cleric,
            false,
        );
        assert!(party.add_member(ally));

        assert_eq!(party.members.len(), 2);
    }

    #[test]
    fn test_cannot_remove_brother() {
        let mut party = Party::new();
        let hero = PartyMember::new(
            "hero".to_string(),
            "Hero".to_string(),
            CharacterClass::Warrior,
            true, // Is a brother
        );
        party.add_member(hero);

        let result = party.remove_member("hero");
        assert!(result.is_err());
        assert_eq!(party.members.len(), 1);
    }

    #[test]
    fn test_can_remove_non_brother() {
        let mut party = Party::new();
        let ally = PartyMember::new(
            "ally".to_string(),
            "Ally".to_string(),
            CharacterClass::Cleric,
            false, // Not a brother
        );
        party.add_member(ally);

        let result = party.remove_member("ally");
        assert!(result.is_ok());
        assert_eq!(party.members.len(), 0);
    }

    #[test]
    fn test_party_size_limit() {
        let mut party = Party::new();

        for i in 0..Party::MAX_SIZE {
            let member = PartyMember::new(
                format!("member{}", i),
                format!("Member {}", i),
                CharacterClass::Warrior,
                false,
            );
            assert!(party.add_member(member));
        }

        // Should fail to add 5th member
        let extra = PartyMember::new(
            "extra".to_string(),
            "Extra".to_string(),
            CharacterClass::Warrior,
            false,
        );
        assert!(!party.add_member(extra));
    }

    #[test]
    fn test_inventory() {
        let mut inventory = Inventory::new();

        inventory.add("potion", 3);
        assert_eq!(inventory.count("potion"), 3);

        inventory.add("potion", 2);
        assert_eq!(inventory.count("potion"), 5);

        assert!(inventory.remove("potion", 2));
        assert_eq!(inventory.count("potion"), 3);

        assert!(!inventory.remove("potion", 10)); // Not enough
        assert_eq!(inventory.count("potion"), 3); // Unchanged
    }

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new(1, "Herbert".to_string());

        assert_eq!(state.user_id, 1);
        assert_eq!(state.gold, 100);
        assert_eq!(state.party.members.len(), 1);
        assert_eq!(state.party.members[0].name, "Herbert");
        assert!(state.party.members[0].is_brother);
        assert_eq!(state.world_phase, 0);
    }

    #[test]
    fn test_story_flags() {
        let mut state = GameState::new(1, "Herbert".to_string());

        assert!(!state.has_flag("test_flag"));
        state.set_flag("test_flag");
        assert!(state.has_flag("test_flag"));
        state.clear_flag("test_flag");
        assert!(!state.has_flag("test_flag"));
    }

    #[test]
    fn test_gold_operations() {
        let mut state = GameState::new(1, "Herbert".to_string());

        assert!(state.spend_gold(50));
        assert_eq!(state.gold, 50);

        assert!(!state.spend_gold(100)); // Not enough
        assert_eq!(state.gold, 50); // Unchanged

        state.add_gold(100);
        assert_eq!(state.gold, 150);
    }

    #[test]
    fn test_shrine_progress() {
        let mut state = GameState::new(1, "Herbert".to_string());

        assert_eq!(state.shrines_destroyed_count(), 0);

        state.shrines_destroyed[0] = true;
        state.shrines_destroyed[2] = true;
        assert_eq!(state.shrines_destroyed_count(), 2);
    }

    #[test]
    fn test_movement() {
        let mut state = GameState::new(1, "Herbert".to_string());
        state.position = (5, 5);

        state.move_direction(Direction::Up);
        assert_eq!(state.position, (5, 4));
        assert_eq!(state.direction, Direction::Up);

        state.move_direction(Direction::Right);
        assert_eq!(state.position, (6, 4));
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new(1, "Herbert".to_string());
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.user_id, restored.user_id);
        assert_eq!(state.gold, restored.gold);
        assert_eq!(state.party.members.len(), restored.party.members.len());
    }

    #[test]
    fn test_play_time_format() {
        let mut state = GameState::new(1, "Herbert".to_string());
        state.play_time_seconds = 3723; // 1:02:03
        assert_eq!(state.formatted_play_time(), "01:02:03");
    }

    #[test]
    fn test_level_up() {
        let mut member = PartyMember::new(
            "test".to_string(),
            "Test".to_string(),
            CharacterClass::Warrior,
            false,
        );

        // Give enough XP to level up
        member.exp = 150;
        assert!(member.can_level_up());

        let gains = member.level_up();
        assert!(gains.is_some());
        assert_eq!(member.level, 2);
        assert_eq!(member.hp, member.hp_max); // Full heal
    }
}
