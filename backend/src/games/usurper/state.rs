//! Game state for Usurper
//!
//! Contains the serializable player state including character stats,
//! equipment, substance effects, romance status, and progression.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::data::{CharacterClass, EquipmentSlot};

/// Main game state - persisted to database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // Character identity
    pub handle: Option<String>,
    pub character_name: String,
    pub class: CharacterClassSer,
    pub level: u32,
    pub experience: u64,
    pub experience_to_next: u64,

    // Primary stats (base values)
    pub strength: u32,
    pub agility: u32,
    pub vitality: u32,
    pub intelligence: u32,
    pub charisma: u32,

    // Derived stats
    pub hp: u32,
    pub max_hp: u32,
    pub mental_stability: i32,  // Can go negative (psychosis)
    pub max_mental_stability: i32,

    // Resources
    pub gold: u64,
    pub bank_gold: u64,

    // Location
    pub current_dungeon_level: u32,
    pub in_town: bool,

    // Daily limits
    pub turns_remaining: u32,
    pub last_turn_date: String,  // "YYYY-MM-DD"

    // Equipment (slot -> item key)
    pub equipment: Equipment,

    // Inventory (item key -> count)
    pub inventory: HashMap<String, u32>,

    // Substance tracking
    pub active_effects: Vec<ActiveSubstanceEffect>,
    pub addictions: HashMap<String, u32>,  // substance key -> addiction level

    // Romance
    pub romance_status: RomanceStatus,

    // PvP
    pub pvp_kills: u32,
    pub pvp_deaths: u32,
    pub reputation: i32,  // Good/Evil alignment
    pub notoriety: u32,   // PvP fame

    // Political
    pub is_king: bool,
    pub godhood_level: u32,  // 0 = mortal, 1+ = divine ranks

    // Clan/Team
    pub clan_id: Option<String>,
    pub clan_role: Option<String>,

    // Quest progress
    pub quests_completed: Vec<String>,
    pub current_quest: Option<String>,
    pub supreme_being_defeated: bool,

    // Stats tracking
    pub monsters_killed: u64,
    pub total_gold_earned: u64,
    pub deaths: u32,
    pub deepest_dungeon: u32,
    pub days_played: u32,

    // Transient state
    #[serde(default)]
    pub last_message: Option<String>,

    // Game state
    pub game_over: bool,
    pub game_over_reason: Option<String>,
}

/// Serializable version of CharacterClass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterClassSer {
    Warrior,
    Rogue,
    Mage,
    Cleric,
    Berserker,
}

impl From<CharacterClass> for CharacterClassSer {
    fn from(c: CharacterClass) -> Self {
        match c {
            CharacterClass::Warrior => CharacterClassSer::Warrior,
            CharacterClass::Rogue => CharacterClassSer::Rogue,
            CharacterClass::Mage => CharacterClassSer::Mage,
            CharacterClass::Cleric => CharacterClassSer::Cleric,
            CharacterClass::Berserker => CharacterClassSer::Berserker,
        }
    }
}

impl From<CharacterClassSer> for CharacterClass {
    fn from(c: CharacterClassSer) -> Self {
        match c {
            CharacterClassSer::Warrior => CharacterClass::Warrior,
            CharacterClassSer::Rogue => CharacterClass::Rogue,
            CharacterClassSer::Mage => CharacterClass::Mage,
            CharacterClassSer::Cleric => CharacterClass::Cleric,
            CharacterClassSer::Berserker => CharacterClass::Berserker,
        }
    }
}

impl CharacterClassSer {
    pub fn name(&self) -> &'static str {
        CharacterClass::from(*self).name()
    }
}

/// Equipment slots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub shield: Option<String>,
    pub helmet: Option<String>,
    pub armor: Option<String>,
    pub gloves: Option<String>,
    pub boots: Option<String>,
    pub ring_left: Option<String>,
    pub ring_right: Option<String>,
    pub amulet: Option<String>,
    pub cloak: Option<String>,
}

impl Equipment {
    pub fn get(&self, slot: EquipmentSlot) -> &Option<String> {
        match slot {
            EquipmentSlot::Weapon => &self.weapon,
            EquipmentSlot::Shield => &self.shield,
            EquipmentSlot::Helmet => &self.helmet,
            EquipmentSlot::Armor => &self.armor,
            EquipmentSlot::Gloves => &self.gloves,
            EquipmentSlot::Boots => &self.boots,
            EquipmentSlot::RingLeft => &self.ring_left,
            EquipmentSlot::RingRight => &self.ring_right,
            EquipmentSlot::Amulet => &self.amulet,
            EquipmentSlot::Cloak => &self.cloak,
        }
    }

    pub fn set(&mut self, slot: EquipmentSlot, item: Option<String>) {
        match slot {
            EquipmentSlot::Weapon => self.weapon = item,
            EquipmentSlot::Shield => self.shield = item,
            EquipmentSlot::Helmet => self.helmet = item,
            EquipmentSlot::Armor => self.armor = item,
            EquipmentSlot::Gloves => self.gloves = item,
            EquipmentSlot::Boots => self.boots = item,
            EquipmentSlot::RingLeft => self.ring_left = item,
            EquipmentSlot::RingRight => self.ring_right = item,
            EquipmentSlot::Amulet => self.amulet = item,
            EquipmentSlot::Cloak => self.cloak = item,
        }
    }

    pub fn all_equipped(&self) -> Vec<(&'static str, &Option<String>)> {
        vec![
            ("Weapon", &self.weapon),
            ("Shield", &self.shield),
            ("Helmet", &self.helmet),
            ("Armor", &self.armor),
            ("Gloves", &self.gloves),
            ("Boots", &self.boots),
            ("Ring (L)", &self.ring_left),
            ("Ring (R)", &self.ring_right),
            ("Amulet", &self.amulet),
            ("Cloak", &self.cloak),
        ]
    }
}

/// Active substance effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSubstanceEffect {
    pub substance_key: String,
    pub turns_remaining: u32,
    pub effects: SubstanceEffects,
}

/// Computed effects from a substance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubstanceEffects {
    pub strength_mod: i32,
    pub agility_mod: i32,
    pub vitality_mod: i32,
    pub intelligence_mod: i32,
    pub damage_mod: i32,
    pub defense_mod: i32,
    pub action_bonus: u32,
    pub invincible: bool,
}

/// Romance status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RomanceStatus {
    pub partner_user_id: Option<i64>,
    pub partner_name: Option<String>,
    pub relationship_level: u32,  // 0 = single, 1 = dating, 2 = engaged, 3 = married
    pub marriage_date: Option<String>,
    pub stat_bonuses: RomanceStatBonuses,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RomanceStatBonuses {
    pub strength: i32,
    pub vitality: i32,
    pub charisma: i32,
    pub mental_stability: i32,
}

/// Computed character stats (with all modifiers applied)
#[derive(Debug, Clone, Default)]
pub struct CharacterStats {
    pub strength: i32,
    pub agility: i32,
    pub vitality: i32,
    pub intelligence: i32,
    pub charisma: i32,
    pub damage: i32,
    pub defense: i32,
    pub max_hp: i32,
    pub mental_stability: i32,
}

impl GameState {
    pub fn new(name: String, class: CharacterClass) -> Self {
        let (str, agi, vit, int, cha, mental) = class.base_stats();
        let max_hp = vit * 10 + 50;

        Self {
            handle: None,
            character_name: name,
            class: class.into(),
            level: 1,
            experience: 0,
            experience_to_next: 100,

            strength: str,
            agility: agi,
            vitality: vit,
            intelligence: int,
            charisma: cha,

            hp: max_hp,
            max_hp,
            mental_stability: mental as i32,
            max_mental_stability: 100,

            gold: 100,
            bank_gold: 0,

            current_dungeon_level: 1,
            in_town: true,

            turns_remaining: 20,
            last_turn_date: String::new(),

            equipment: Equipment::default(),
            inventory: HashMap::new(),
            active_effects: Vec::new(),
            addictions: HashMap::new(),

            romance_status: RomanceStatus::default(),

            pvp_kills: 0,
            pvp_deaths: 0,
            reputation: 0,
            notoriety: 0,

            is_king: false,
            godhood_level: 0,

            clan_id: None,
            clan_role: None,

            quests_completed: Vec::new(),
            current_quest: None,
            supreme_being_defeated: false,

            monsters_killed: 0,
            total_gold_earned: 100,
            deaths: 0,
            deepest_dungeon: 0,
            days_played: 0,

            last_message: None,
            game_over: false,
            game_over_reason: None,
        }
    }

    /// Calculate effective stats with all modifiers
    pub fn effective_stats(&self) -> CharacterStats {
        let mut stats = CharacterStats {
            strength: self.strength as i32,
            agility: self.agility as i32,
            vitality: self.vitality as i32,
            intelligence: self.intelligence as i32,
            charisma: self.charisma as i32,
            damage: 0,
            defense: 0,
            max_hp: self.max_hp as i32,
            mental_stability: self.mental_stability,
        };

        // Apply equipment bonuses
        for (_slot, item_key) in self.equipment.all_equipped() {
            if let Some(key) = item_key {
                if let Some(item) = super::data::get_equipment(key) {
                    stats.strength += item.stat_bonuses.strength;
                    stats.agility += item.stat_bonuses.agility;
                    stats.vitality += item.stat_bonuses.vitality;
                    stats.intelligence += item.stat_bonuses.intelligence;
                    stats.charisma += item.stat_bonuses.charisma;
                    stats.damage += item.stat_bonuses.damage;
                    stats.defense += item.stat_bonuses.defense;
                    stats.max_hp += item.stat_bonuses.hp_bonus;
                    stats.mental_stability += item.stat_bonuses.mental_stability;
                }
            }
        }

        // Apply active substance effects
        for effect in &self.active_effects {
            stats.strength += effect.effects.strength_mod;
            stats.agility += effect.effects.agility_mod;
            stats.vitality += effect.effects.vitality_mod;
            stats.intelligence += effect.effects.intelligence_mod;
            stats.damage += effect.effects.damage_mod;
            stats.defense += effect.effects.defense_mod;
        }

        // Apply romance bonuses
        if self.romance_status.relationship_level >= 2 {
            stats.strength += self.romance_status.stat_bonuses.strength;
            stats.vitality += self.romance_status.stat_bonuses.vitality;
            stats.charisma += self.romance_status.stat_bonuses.charisma;
            stats.mental_stability += self.romance_status.stat_bonuses.mental_stability;
        }

        // Apply base damage from strength
        stats.damage += stats.strength / 2;

        // Apply base defense from agility
        stats.defense += stats.agility / 3;

        stats
    }

    /// Get total damage (for display)
    pub fn total_damage(&self) -> i32 {
        self.effective_stats().damage
    }

    /// Get total defense (for display)
    pub fn total_defense(&self) -> i32 {
        self.effective_stats().defense
    }

    /// Check if player is currently invincible
    pub fn is_invincible(&self) -> bool {
        self.active_effects.iter().any(|e| e.effects.invincible)
    }

    /// Check if player is in psychosis (mental stability <= 0)
    pub fn is_in_psychosis(&self) -> bool {
        self.mental_stability <= 0
    }

    /// Apply damage to player
    pub fn take_damage(&mut self, amount: u32) {
        if self.is_invincible() {
            return;
        }
        self.hp = self.hp.saturating_sub(amount);
        if self.hp == 0 {
            self.deaths += 1;
            // Respawn in town with half HP
            self.hp = self.max_hp / 2;
            self.in_town = true;
            self.last_message = Some("You died and were resurrected in town...".to_string());
        }
    }

    /// Heal player
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, xp: u64) -> bool {
        self.experience += xp;
        if self.experience >= self.experience_to_next {
            self.level_up();
            return true;
        }
        false
    }

    /// Level up the character
    fn level_up(&mut self) {
        self.level += 1;
        self.experience -= self.experience_to_next;
        self.experience_to_next = (self.experience_to_next as f64 * 1.5) as u64;

        // Stat gains based on class
        let class: CharacterClass = self.class.into();
        match class {
            CharacterClass::Warrior => {
                self.strength += 3;
                self.vitality += 2;
                self.agility += 1;
            }
            CharacterClass::Rogue => {
                self.agility += 3;
                self.charisma += 2;
                self.strength += 1;
            }
            CharacterClass::Mage => {
                self.intelligence += 3;
                self.charisma += 1;
                self.agility += 1;
            }
            CharacterClass::Cleric => {
                self.vitality += 2;
                self.intelligence += 2;
                self.charisma += 2;
            }
            CharacterClass::Berserker => {
                self.strength += 4;
                self.vitality += 2;
            }
        }

        // Recalculate max HP
        self.max_hp = self.vitality * 10 + 50 + (self.level * 5);
        self.hp = self.max_hp; // Full heal on level up

        self.last_message = Some(format!("Level Up! You are now level {}!", self.level));
    }

    /// Add gold and track stats
    pub fn add_gold(&mut self, amount: u64) {
        self.gold += amount;
        self.total_gold_earned += amount;
    }

    /// Spend gold (returns false if insufficient)
    pub fn spend_gold(&mut self, amount: u64) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// Check and reset turns for a new day
    pub fn check_new_day(&mut self) -> bool {
        use chrono::Local;
        let today = Local::now().format("%Y-%m-%d").to_string();

        if self.last_turn_date != today {
            self.last_turn_date = today;
            self.turns_remaining = 20; // Reset daily turns
            self.days_played += 1;

            // Tick down substance effects
            self.active_effects.retain_mut(|effect| {
                if effect.turns_remaining > 0 {
                    effect.turns_remaining -= 1;
                    true
                } else {
                    false
                }
            });

            // Slight mental stability recovery overnight
            if self.mental_stability < self.max_mental_stability {
                self.mental_stability = (self.mental_stability + 5).min(self.max_mental_stability);
            }

            return true;
        }
        false
    }

    /// Use a turn
    pub fn use_turn(&mut self) -> bool {
        if self.turns_remaining > 0 {
            self.turns_remaining -= 1;
            true
        } else {
            false
        }
    }

    /// Get bonus actions from active effects
    pub fn bonus_actions(&self) -> u32 {
        self.active_effects.iter().map(|e| e.effects.action_bonus).sum()
    }

    /// Apply substance effect
    pub fn apply_substance(&mut self, key: &str) -> Result<String, String> {
        let substance = super::data::get_substance(key)
            .ok_or_else(|| "Unknown substance".to_string())?;

        // Apply mental cost
        self.mental_stability += substance.mental_cost;

        // Check for psychosis
        let psychosis_triggered = self.mental_stability <= 0 && !self.is_in_psychosis();

        // Add active effect
        let effect = ActiveSubstanceEffect {
            substance_key: key.to_string(),
            turns_remaining: substance.duration_turns,
            effects: SubstanceEffects {
                strength_mod: substance.effects.strength_mod,
                agility_mod: substance.effects.agility_mod,
                vitality_mod: substance.effects.vitality_mod,
                intelligence_mod: substance.effects.intelligence_mod,
                damage_mod: substance.effects.damage_mod,
                defense_mod: substance.effects.defense_mod,
                action_bonus: substance.effects.action_bonus,
                invincible: substance.effects.invincible_turns > 0,
            },
        };
        self.active_effects.push(effect);

        // Apply instant healing
        if substance.effects.healing > 0 {
            self.heal(substance.effects.healing as u32);
        }

        // Check for addiction
        let addiction_level = self.addictions.entry(key.to_string()).or_insert(0);
        if rand::random::<u32>() % 100 < substance.addiction_chance + (*addiction_level * 5) {
            *addiction_level += 1;
        }

        let mut msg = format!("{} takes effect!", substance.name);
        if psychosis_triggered {
            msg.push_str(" WARNING: You feel your mind slipping into madness...");
        }

        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_character() {
        let state = GameState::new("TestHero".to_string(), CharacterClass::Warrior);
        assert_eq!(state.level, 1);
        assert_eq!(state.strength, 15);
        assert_eq!(state.gold, 100);
        assert!(state.in_town);
    }

    #[test]
    fn test_level_up() {
        let mut state = GameState::new("TestHero".to_string(), CharacterClass::Warrior);
        let initial_str = state.strength;
        state.add_experience(100);
        assert_eq!(state.level, 2);
        assert!(state.strength > initial_str);
    }

    #[test]
    fn test_damage_and_death() {
        let mut state = GameState::new("TestHero".to_string(), CharacterClass::Warrior);
        state.hp = 10;
        state.take_damage(100);
        assert!(state.hp > 0); // Should respawn
        assert_eq!(state.deaths, 1);
        assert!(state.in_town);
    }

    #[test]
    fn test_mental_stability() {
        let mut state = GameState::new("TestHero".to_string(), CharacterClass::Warrior);
        state.mental_stability = 5;
        assert!(!state.is_in_psychosis());
        state.mental_stability = 0;
        assert!(state.is_in_psychosis());
    }

    #[test]
    fn test_equipment() {
        let mut equip = Equipment::default();
        equip.set(EquipmentSlot::Weapon, Some("rusty_sword".to_string()));
        assert_eq!(equip.weapon, Some("rusty_sword".to_string()));
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new("TestHero".to_string(), CharacterClass::Rogue);
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.character_name, restored.character_name);
        assert_eq!(state.agility, restored.agility);
    }
}
