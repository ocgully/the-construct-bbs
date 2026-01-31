//! Dragon Slayer game state
//! Persistent player data that gets serialized to the database

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Default daily forest fights
pub const DEFAULT_DAILY_FIGHTS: u32 = 20;

/// Default daily player attacks
pub const DEFAULT_PLAYER_ATTACKS: u32 = 2;

/// The main game state for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Player's display handle
    pub handle: Option<String>,

    /// Character info
    pub char_name: String,
    pub sex: Sex,
    pub level: u8,
    pub experience: i64,

    /// Combat stats
    pub stats: PlayerStats,
    pub hp_current: u32,
    pub hp_max: u32,

    /// Equipment
    pub equipment: Equipment,

    /// Skills from all three paths (can learn all)
    pub skills: SkillTree,

    /// Gold
    pub gold_pocket: i64,
    pub gold_bank: i64,

    /// Romance
    pub romance: RomanceStatus,
    pub charm: u32,

    /// Daily limits
    pub forest_fights_today: u32,
    pub player_fights_today: u32,
    pub last_play_date: Option<String>,
    pub skill_uses_today: HashMap<String, u8>,

    /// Combat record
    pub kills: u32,
    pub deaths: u32,
    pub dragon_kills: u32,

    /// Special items
    pub has_fairy: bool,
    pub fairy_uses: u8,

    /// Death state
    pub is_dead: bool,

    /// Message to display
    pub last_message: Option<String>,

    /// Sysop-configurable settings
    pub daily_fights_max: u32,
    pub daily_attacks_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStats {
    pub strength: u32,
    pub defense: u32,
    pub vitality: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Equipment {
    pub weapon: String,      // weapon key
    pub armor: String,       // armor key
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillTree {
    pub death_knight: u8,    // 0-40 points
    pub mystic: u8,          // 0-40 points
    pub thief: u8,           // 0-40 points
    /// Available skill points to spend
    pub unspent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RomanceStatus {
    /// ID of spouse (player or NPC name)
    pub spouse: Option<String>,
    /// Relationship level with Violet (NPC)
    pub violet_affection: u8,
    /// Relationship level with Seth (NPC)
    pub seth_affection: u8,
    /// Marriage date
    pub married_date: Option<String>,
    /// Number of "flirt" actions today
    pub flirts_today: u8,
}

impl GameState {
    /// Create a new game state for a fresh character
    pub fn new(char_name: String, sex: Sex) -> Self {
        Self {
            handle: None,
            char_name,
            sex,
            level: 1,
            experience: 0,

            stats: PlayerStats {
                strength: 10,
                defense: 5,
                vitality: 10,
            },
            hp_current: 20,
            hp_max: 20,

            equipment: Equipment {
                weapon: "stick".to_string(),
                armor: "rags".to_string(),
            },

            skills: SkillTree::default(),

            gold_pocket: 100,
            gold_bank: 0,

            romance: RomanceStatus::default(),
            charm: 5,

            forest_fights_today: 0,
            player_fights_today: 0,
            last_play_date: None,
            skill_uses_today: HashMap::new(),

            kills: 0,
            deaths: 0,
            dragon_kills: 0,

            has_fairy: false,
            fairy_uses: 0,

            is_dead: false,
            last_message: None,

            daily_fights_max: DEFAULT_DAILY_FIGHTS,
            daily_attacks_max: DEFAULT_PLAYER_ATTACKS,
        }
    }

    /// Check if this is a new day (resets daily limits)
    /// Uses midnight Eastern time as the reset point
    pub fn check_new_day(&mut self) -> bool {
        use chrono_tz::America::New_York;
        use chrono::Utc;

        let now = Utc::now().with_timezone(&New_York);
        let today = now.format("%Y-%m-%d").to_string();

        match &self.last_play_date {
            Some(last_date) if last_date == &today => false,
            _ => {
                // Reset daily limits
                self.forest_fights_today = 0;
                self.player_fights_today = 0;
                self.romance.flirts_today = 0;
                self.skill_uses_today.clear();

                // Revive if dead
                if self.is_dead {
                    self.is_dead = false;
                    self.hp_current = self.hp_max;
                    self.last_message = Some("You wake at the Inn. A new day begins!".to_string());
                }

                self.last_play_date = Some(today);
                true
            }
        }
    }

    /// Get total attack power (strength + weapon damage)
    pub fn attack_power(&self) -> u32 {
        use super::data::get_weapon;
        let weapon_damage = get_weapon(&self.equipment.weapon)
            .map(|w| w.damage)
            .unwrap_or(1);
        self.stats.strength + weapon_damage
    }

    /// Get total defense (defense stat + armor defense)
    pub fn defense_power(&self) -> u32 {
        use super::data::get_armor;
        let armor_def = get_armor(&self.equipment.armor)
            .map(|a| a.defense)
            .unwrap_or(0);
        self.stats.defense + armor_def
    }

    /// Calculate HP for a given level
    pub fn hp_for_level(level: u8, vitality: u32) -> u32 {
        // Base 20 HP + 10 per level + vitality bonus
        20 + (level as u32 * 10) + (vitality * 2)
    }

    /// Get remaining forest fights for today
    pub fn forest_fights_remaining(&self) -> u32 {
        self.daily_fights_max.saturating_sub(self.forest_fights_today)
    }

    /// Get remaining player attacks for today
    pub fn player_attacks_remaining(&self) -> u32 {
        self.daily_attacks_max.saturating_sub(self.player_fights_today)
    }

    /// Use a forest fight
    pub fn use_forest_fight(&mut self) -> bool {
        if self.forest_fights_today < self.daily_fights_max {
            self.forest_fights_today += 1;
            true
        } else {
            false
        }
    }

    /// Use a player attack
    #[allow(dead_code)]
    pub fn use_player_attack(&mut self) -> bool {
        if self.player_fights_today < self.daily_attacks_max {
            self.player_fights_today += 1;
            true
        } else {
            false
        }
    }

    /// Check if player can use a skill today
    pub fn can_use_skill(&self, skill_key: &str) -> bool {
        use super::data::get_skill;
        if let Some(skill) = get_skill(skill_key) {
            let uses = self.skill_uses_today.get(skill_key).copied().unwrap_or(0);
            uses < skill.uses_per_day
        } else {
            false
        }
    }

    /// Use a skill (increments daily counter)
    pub fn use_skill(&mut self, skill_key: &str) {
        let uses = self.skill_uses_today.entry(skill_key.to_string()).or_insert(0);
        *uses += 1;
    }

    /// Check if player has enough skill points in a path to use a skill
    pub fn has_skill(&self, skill_key: &str) -> bool {
        use super::data::{get_skill, SkillPath};
        if let Some(skill) = get_skill(skill_key) {
            let points = match skill.path {
                SkillPath::DeathKnight => self.skills.death_knight,
                SkillPath::Mystic => self.skills.mystic,
                SkillPath::Thief => self.skills.thief,
            };
            points >= skill.level_required && self.level >= skill.level_required
        } else {
            false
        }
    }

    /// Award experience and handle level up
    #[allow(dead_code)]
    pub fn award_experience(&mut self, xp: i64) -> Option<String> {
        self.experience += xp;

        // Check for level up messages (but don't actually level up - need to beat master)
        use super::data::get_master;
        if let Some(master) = get_master(self.level) {
            if self.experience >= master.xp_required {
                return Some(format!(
                    "You have enough experience to challenge {}!",
                    master.name
                ));
            }
        }
        None
    }

    /// Level up after defeating a master
    pub fn level_up(&mut self) {
        self.level += 1;

        // Increase stats
        self.stats.strength += 5 + (self.level as u32);
        self.stats.defense += 3 + (self.level as u32 / 2);
        self.stats.vitality += 2;

        // Recalculate max HP
        self.hp_max = Self::hp_for_level(self.level, self.stats.vitality);
        self.hp_current = self.hp_max; // Full heal on level up

        // Award skill points
        self.skills.unspent += 2;

        // Increase charm
        self.charm += 1;
    }

    /// Handle player death
    pub fn die(&mut self) {
        self.is_dead = true;
        self.deaths += 1;

        // Lose 10% of experience
        let xp_loss = self.experience / 10;
        self.experience = self.experience.saturating_sub(xp_loss);

        // Lose gold in pocket (not banked)
        let gold_loss = self.gold_pocket / 2;
        self.gold_pocket -= gold_loss;

        // Fairy can revive once
        if self.has_fairy && self.fairy_uses > 0 {
            self.fairy_uses -= 1;
            self.is_dead = false;
            self.hp_current = self.hp_max / 2;
            self.last_message = Some("Your fairy sacrifices itself to save you!".to_string());

            if self.fairy_uses == 0 {
                self.has_fairy = false;
            }
        } else {
            self.last_message = Some(format!(
                "You have been slain! Lost {} XP and {} gold. Return tomorrow.",
                xp_loss, gold_loss
            ));
        }
    }

    /// Heal at the healer's hut
    pub fn heal(&mut self, cost_per_hp: i64) -> Result<i64, &'static str> {
        let damage = self.hp_max - self.hp_current;
        if damage == 0 {
            return Err("You are already at full health!");
        }

        let cost = damage as i64 * cost_per_hp;
        if self.gold_pocket < cost {
            // Heal what we can afford
            let affordable_hp = (self.gold_pocket / cost_per_hp) as u32;
            if affordable_hp == 0 {
                return Err("You don't have enough gold!");
            }
            self.hp_current += affordable_hp;
            let partial_cost = affordable_hp as i64 * cost_per_hp;
            self.gold_pocket -= partial_cost;
            return Ok(partial_cost);
        }

        self.hp_current = self.hp_max;
        self.gold_pocket -= cost;
        Ok(cost)
    }

    /// Rest at the inn (costs nothing, but uses up the day)
    pub fn rest_at_inn(&mut self) {
        // Partial heal based on remaining fights
        let heal_amount = (self.hp_max - self.hp_current) / 3;
        self.hp_current = (self.hp_current + heal_amount).min(self.hp_max);

        // Use all remaining fights for the day (forces return tomorrow)
        self.forest_fights_today = self.daily_fights_max;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_character() {
        let state = GameState::new("Hero".to_string(), Sex::Male);
        assert_eq!(state.level, 1);
        assert_eq!(state.experience, 0);
        assert_eq!(state.hp_current, 20);
        assert_eq!(state.equipment.weapon, "stick");
    }

    #[test]
    fn test_attack_power() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        // Stick does 1 damage, strength is 10
        assert_eq!(state.attack_power(), 11);

        state.equipment.weapon = "dagger".to_string();
        // Dagger does 3 damage
        assert_eq!(state.attack_power(), 13);
    }

    #[test]
    fn test_level_up() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        let old_str = state.stats.strength;
        let old_hp = state.hp_max;

        state.level_up();

        assert_eq!(state.level, 2);
        assert!(state.stats.strength > old_str);
        assert!(state.hp_max > old_hp);
        assert_eq!(state.hp_current, state.hp_max); // Full heal
        assert_eq!(state.skills.unspent, 2);
    }

    #[test]
    fn test_daily_fight_limit() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);

        for _ in 0..20 {
            assert!(state.use_forest_fight());
        }

        // 21st fight should fail
        assert!(!state.use_forest_fight());
    }

    #[test]
    fn test_death() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        state.experience = 100;
        state.gold_pocket = 1000;

        state.die();

        assert!(state.is_dead);
        assert_eq!(state.deaths, 1);
        assert_eq!(state.experience, 90); // Lost 10%
        assert_eq!(state.gold_pocket, 500); // Lost half
    }

    #[test]
    fn test_fairy_saves() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        state.has_fairy = true;
        state.fairy_uses = 1;

        state.die();

        assert!(!state.is_dead); // Fairy saved us
        assert!(!state.has_fairy); // Fairy is gone
    }

    #[test]
    fn test_heal() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        state.hp_current = 10; // Missing 10 HP
        state.gold_pocket = 100;

        let cost = state.heal(5).unwrap();
        assert_eq!(cost, 50); // 10 HP * 5 gold = 50
        assert_eq!(state.hp_current, 20);
        assert_eq!(state.gold_pocket, 50);
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new("TestHero".to_string(), Sex::Female);
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.char_name, restored.char_name);
        assert_eq!(state.level, restored.level);
        assert_eq!(state.sex, restored.sex);
    }
}
