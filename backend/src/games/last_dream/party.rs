//! Party and character management for Last Dream
//! Handles party of up to 4 characters, stats, equipment, and abilities

use serde::{Deserialize, Serialize};
use super::data::{ClassType, EquipmentSlot, get_equipment, get_spell};

/// Maximum party size
pub const MAX_PARTY_SIZE: usize = 4;

/// A single character in the party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub class: CharacterClass,

    // Level and experience
    pub level: u8,
    pub exp: u64,
    pub exp_to_next: u64,

    // Current and max stats
    pub hp: u32,
    pub hp_max: u32,
    pub mp: u32,
    pub mp_max: u32,

    // Base stats (before equipment)
    pub strength: u32,
    pub agility: u32,
    pub intelligence: u32,
    pub vitality: u32,
    pub luck: u32,

    // Equipment
    pub equipment: Equipment,

    // Known spells
    pub spells: Vec<String>,

    // Status effects
    pub status: StatusEffects,

    // ATB gauge (0-100)
    pub atb_gauge: u32,

    // Is character in front or back row?
    pub front_row: bool,
}

/// Character class (serializable version)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClass {
    Warrior,
    Thief,
    Mage,
    Cleric,
    Monk,
    Knight,
}

impl CharacterClass {
    pub fn to_class_type(&self) -> ClassType {
        match self {
            CharacterClass::Warrior => ClassType::Warrior,
            CharacterClass::Thief => ClassType::Thief,
            CharacterClass::Mage => ClassType::Mage,
            CharacterClass::Cleric => ClassType::Cleric,
            CharacterClass::Monk => ClassType::Monk,
            CharacterClass::Knight => ClassType::Knight,
        }
    }

    pub fn from_class_type(ct: ClassType) -> Self {
        match ct {
            ClassType::Warrior => CharacterClass::Warrior,
            ClassType::Thief => CharacterClass::Thief,
            ClassType::Mage => CharacterClass::Mage,
            ClassType::Cleric => CharacterClass::Cleric,
            ClassType::Monk => CharacterClass::Monk,
            ClassType::Knight => CharacterClass::Knight,
        }
    }

    pub fn name(&self) -> &'static str {
        self.to_class_type().name()
    }
}

/// Equipment slots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub shield: Option<String>,
    pub helmet: Option<String>,
    pub accessory: Option<String>,
}

impl Equipment {
    /// Get equipment key for a slot
    pub fn get(&self, slot: EquipmentSlot) -> Option<&str> {
        match slot {
            EquipmentSlot::Weapon => self.weapon.as_deref(),
            EquipmentSlot::Armor => self.armor.as_deref(),
            EquipmentSlot::Shield => self.shield.as_deref(),
            EquipmentSlot::Helmet => self.helmet.as_deref(),
            EquipmentSlot::Accessory => self.accessory.as_deref(),
        }
    }

    /// Set equipment for a slot
    pub fn set(&mut self, slot: EquipmentSlot, key: Option<String>) {
        match slot {
            EquipmentSlot::Weapon => self.weapon = key,
            EquipmentSlot::Armor => self.armor = key,
            EquipmentSlot::Shield => self.shield = key,
            EquipmentSlot::Helmet => self.helmet = key,
            EquipmentSlot::Accessory => self.accessory = key,
        }
    }

    /// Calculate total stat bonuses from equipment
    pub fn total_stats(&self) -> (i32, i32, i32, i32) {
        // Returns (attack, defense, magic, speed)
        let mut attack = 0i32;
        let mut defense = 0i32;
        let mut magic = 0i32;
        let mut speed = 0i32;

        for slot in &[
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Shield,
            EquipmentSlot::Helmet,
            EquipmentSlot::Accessory,
        ] {
            if let Some(key) = self.get(*slot) {
                if let Some(equip) = get_equipment(key) {
                    attack += equip.attack as i32;
                    defense += equip.defense as i32;
                    magic += equip.magic;
                    speed += equip.speed;
                }
            }
        }

        (attack, defense, magic, speed)
    }
}

/// Status effects on a character
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusEffects {
    pub dead: bool,
    pub poisoned: bool,
    pub sleeping: bool,
    pub paralyzed: bool,
    pub protected: bool,
    pub shelled: bool,
    pub focused: bool, // For monk's Focus ability
}

impl StatusEffects {
    pub fn clear_all(&mut self) {
        *self = Self::default();
    }

    pub fn can_act(&self) -> bool {
        !self.dead && !self.sleeping && !self.paralyzed
    }
}

impl Character {
    /// Create a new character with starting stats
    pub fn new(name: String, class: CharacterClass) -> Self {
        let class_type = class.to_class_type();
        let (hp, mp, str, agi, int, vit, luck) = class_type.base_stats();
        let spells = class_type.spells_at_level(1)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            name,
            class,
            level: 1,
            exp: 0,
            exp_to_next: 100,
            hp,
            hp_max: hp,
            mp,
            mp_max: mp,
            strength: str,
            agility: agi,
            intelligence: int,
            vitality: vit,
            luck,
            equipment: Equipment::default(),
            spells,
            status: StatusEffects::default(),
            atb_gauge: 0,
            front_row: true,
        }
    }

    /// Check if character is alive
    pub fn is_alive(&self) -> bool {
        !self.status.dead && self.hp > 0
    }

    /// Get total attack power (strength + weapon)
    pub fn attack_power(&self) -> u32 {
        let (weapon_atk, _, _, _) = self.equipment.total_stats();
        let base = self.strength + weapon_atk.max(0) as u32;

        // Monk bonus: higher damage when unarmed
        if self.class == CharacterClass::Monk && self.equipment.weapon.is_none() {
            base + (self.level as u32 * 3)
        } else {
            base
        }
    }

    /// Get total defense (vitality + armor)
    pub fn defense_power(&self) -> u32 {
        let (_, armor_def, _, _) = self.equipment.total_stats();
        self.vitality + armor_def.max(0) as u32
    }

    /// Get total magic power (intelligence + equipment)
    pub fn magic_power(&self) -> u32 {
        let (_, _, magic_bonus, _) = self.equipment.total_stats();
        self.intelligence + magic_bonus.max(0) as u32
    }

    /// Get effective speed (agility + equipment)
    pub fn effective_speed(&self) -> u32 {
        let (_, _, _, speed_bonus) = self.equipment.total_stats();
        (self.agility as i32 + speed_bonus).max(1) as u32
    }

    /// Calculate damage dealt
    pub fn calculate_damage(&self) -> u32 {
        let base = self.attack_power();
        let multiplier = if self.status.focused { 2 } else { 1 };
        base * multiplier
    }

    /// Take damage, return actual damage taken
    pub fn take_damage(&mut self, amount: u32) -> u32 {
        let defense = self.defense_power();
        let reduced = amount.saturating_sub(defense / 2);
        let actual = reduced.max(1);

        if actual >= self.hp {
            self.hp = 0;
            self.status.dead = true;
        } else {
            self.hp -= actual;
        }

        actual
    }

    /// Heal HP
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// Restore MP
    pub fn restore_mp(&mut self, amount: u32) {
        self.mp = (self.mp + amount).min(self.mp_max);
    }

    /// Revive from KO
    pub fn revive(&mut self, hp_percent: u32) {
        self.status.dead = false;
        self.hp = (self.hp_max * hp_percent / 100).max(1);
    }

    /// Use MP for a spell
    pub fn use_mp(&mut self, amount: u32) -> bool {
        if self.mp >= amount {
            self.mp -= amount;
            true
        } else {
            false
        }
    }

    /// Check if character knows a spell
    pub fn knows_spell(&self, spell_key: &str) -> bool {
        self.spells.iter().any(|s| s == spell_key)
    }

    /// Add experience and check for level up
    pub fn add_exp(&mut self, amount: u64) -> Option<LevelUpResult> {
        self.exp += amount;

        if self.exp >= self.exp_to_next {
            self.level_up()
        } else {
            None
        }
    }

    /// Level up the character
    fn level_up(&mut self) -> Option<LevelUpResult> {
        self.level += 1;
        self.exp -= self.exp_to_next;
        self.exp_to_next = Self::exp_for_level(self.level + 1);

        let class_type = self.class.to_class_type();
        let (hp_gain, mp_gain, str_gain, agi_gain, int_gain, vit_gain, luck_gain) =
            class_type.level_up_stats();

        // Apply stat gains
        self.hp_max += hp_gain;
        self.mp_max += mp_gain;
        self.strength += str_gain;
        self.agility += agi_gain;
        self.intelligence += int_gain;
        self.vitality += vit_gain;
        self.luck += luck_gain;

        // Full heal on level up
        self.hp = self.hp_max;
        self.mp = self.mp_max;

        // Learn new spells
        let new_spells: Vec<String> = class_type.spells_at_level(self.level)
            .into_iter()
            .filter(|s| !self.knows_spell(s))
            .map(|s| s.to_string())
            .collect();

        for spell in &new_spells {
            self.spells.push(spell.clone());
        }

        Some(LevelUpResult {
            new_level: self.level,
            hp_gained: hp_gain,
            mp_gained: mp_gain,
            new_spells,
        })
    }

    /// Calculate EXP needed for a level
    fn exp_for_level(level: u8) -> u64 {
        // Exponential growth
        ((level as u64).pow(2) * 50) + ((level as u64) * 50)
    }

    /// Update ATB gauge based on speed
    pub fn tick_atb(&mut self) -> bool {
        if !self.status.can_act() {
            return false;
        }

        let speed = self.effective_speed();
        self.atb_gauge = (self.atb_gauge + speed).min(100);
        self.atb_gauge >= 100
    }

    /// Reset ATB gauge after taking action
    pub fn reset_atb(&mut self) {
        self.atb_gauge = 0;
        self.status.focused = false; // Clear focus after any action
    }

    /// Get list of usable spells with MP costs
    pub fn usable_spells(&self) -> Vec<(&str, u32)> {
        self.spells.iter()
            .filter_map(|spell_key| {
                get_spell(spell_key).map(|spell| (spell.name, spell.mp_cost))
            })
            .collect()
    }
}

/// Result of leveling up
#[derive(Debug, Clone)]
pub struct LevelUpResult {
    pub new_level: u8,
    pub hp_gained: u32,
    pub mp_gained: u32,
    pub new_spells: Vec<String>,
}

/// The player's party
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Party {
    pub members: Vec<Character>,
}

impl Party {
    pub fn new() -> Self {
        Self { members: Vec::new() }
    }

    /// Add a character to the party
    pub fn add_member(&mut self, character: Character) -> bool {
        if self.members.len() < MAX_PARTY_SIZE {
            self.members.push(character);
            true
        } else {
            false
        }
    }

    /// Get character by index
    pub fn get(&self, index: usize) -> Option<&Character> {
        self.members.get(index)
    }

    /// Get mutable character by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Character> {
        self.members.get_mut(index)
    }

    /// Check if party is alive
    pub fn is_alive(&self) -> bool {
        self.members.iter().any(|c| c.is_alive())
    }

    /// Get all living party members
    pub fn living_members(&self) -> Vec<&Character> {
        self.members.iter().filter(|c| c.is_alive()).collect()
    }

    /// Fully heal all party members
    pub fn full_heal(&mut self) {
        for member in &mut self.members {
            member.hp = member.hp_max;
            member.mp = member.mp_max;
            member.status.clear_all();
        }
    }

    /// Distribute experience to all living members
    pub fn distribute_exp(&mut self, total_exp: u64) -> Vec<(String, Option<LevelUpResult>)> {
        let living_count = self.living_members().len() as u64;
        if living_count == 0 {
            return Vec::new();
        }

        let per_member = total_exp / living_count;
        let mut results = Vec::new();

        for member in &mut self.members {
            if member.is_alive() {
                let level_up = member.add_exp(per_member);
                results.push((member.name.clone(), level_up));
            }
        }

        results
    }

    /// Get the party leader (first member)
    pub fn leader(&self) -> Option<&Character> {
        self.members.first()
    }

    /// Swap positions of two party members
    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.members.len() && b < self.members.len() {
            self.members.swap(a, b);
        }
    }

    /// Rest at an inn (partial heal)
    pub fn rest_at_inn(&mut self) {
        for member in &mut self.members {
            if !member.status.dead {
                member.hp = member.hp_max;
                member.mp = member.mp_max;
                member.status.poisoned = false;
            }
        }
    }

    /// Rest with tent (partial recovery)
    pub fn rest_with_tent(&mut self) {
        for member in &mut self.members {
            if !member.status.dead {
                member.hp = (member.hp + member.hp_max / 2).min(member.hp_max);
                member.mp = (member.mp + member.mp_max / 4).min(member.mp_max);
            }
        }
    }

    /// Get total party level (for encounter scaling)
    pub fn average_level(&self) -> u8 {
        if self.members.is_empty() {
            return 1;
        }
        let total: u32 = self.members.iter().map(|c| c.level as u32).sum();
        (total / self.members.len() as u32) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let warrior = Character::new("Hero".to_string(), CharacterClass::Warrior);
        assert_eq!(warrior.level, 1);
        assert!(warrior.hp > 0);
        assert!(warrior.strength > 10);
    }

    #[test]
    fn test_mage_starts_with_spells() {
        let mage = Character::new("Wizard".to_string(), CharacterClass::Mage);
        assert!(mage.knows_spell("fire"));
    }

    #[test]
    fn test_damage_calculation() {
        let mut warrior = Character::new("Hero".to_string(), CharacterClass::Warrior);
        let initial_hp = warrior.hp;
        warrior.take_damage(10);
        assert!(warrior.hp < initial_hp);
    }

    #[test]
    fn test_party_management() {
        let mut party = Party::new();
        assert!(party.add_member(Character::new("Hero1".to_string(), CharacterClass::Warrior)));
        assert!(party.add_member(Character::new("Hero2".to_string(), CharacterClass::Mage)));
        assert_eq!(party.members.len(), 2);
    }

    #[test]
    fn test_party_size_limit() {
        let mut party = Party::new();
        for i in 0..MAX_PARTY_SIZE {
            assert!(party.add_member(Character::new(format!("Hero{}", i), CharacterClass::Warrior)));
        }
        assert!(!party.add_member(Character::new("Extra".to_string(), CharacterClass::Warrior)));
    }

    #[test]
    fn test_level_up() {
        let mut character = Character::new("Hero".to_string(), CharacterClass::Warrior);
        let old_hp = character.hp_max;
        let result = character.add_exp(150);
        assert!(result.is_some());
        assert_eq!(character.level, 2);
        assert!(character.hp_max > old_hp);
    }

    #[test]
    fn test_equipment_stats() {
        let mut equip = Equipment::default();
        equip.weapon = Some("wooden_sword".to_string());
        let (attack, _, _, _) = equip.total_stats();
        assert!(attack > 0);
    }

    #[test]
    fn test_monk_unarmed_bonus() {
        let monk = Character::new("Monk".to_string(), CharacterClass::Monk);
        let attack = monk.attack_power();
        assert!(attack > monk.strength); // Should have bonus damage
    }

    #[test]
    fn test_atb_system() {
        let mut character = Character::new("Hero".to_string(), CharacterClass::Thief);
        assert_eq!(character.atb_gauge, 0);

        // Tick ATB multiple times
        for _ in 0..10 {
            character.tick_atb();
        }

        assert!(character.atb_gauge > 0);
    }
}
