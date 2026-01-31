//! Real-time combat system for Depths of Diablo
//!
//! Implements continuous action combat with skills, cooldowns, and damage calculation.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::data::get_skill;
use super::dungeon::Monster;
use super::items::{AffixStat, Item};
use super::state::Character;

/// Combat action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatAction {
    Attack,
    UseSkill(String),
    UsePotion(bool), // true = health, false = mana
    Move(i32, i32),  // dx, dy
    Idle,
}

/// Result of a combat tick
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub player_damage_dealt: i32,
    pub player_damage_taken: i32,
    pub monsters_killed: Vec<u64>,
    pub xp_gained: i32,
    pub messages: Vec<String>,
    pub player_died: bool,
    pub skill_used: Option<String>,
}

impl CombatResult {
    pub fn new() -> Self {
        CombatResult {
            player_damage_dealt: 0,
            player_damage_taken: 0,
            monsters_killed: Vec::new(),
            xp_gained: 0,
            messages: Vec::new(),
            player_died: false,
            skill_used: None,
        }
    }
}

impl Default for CombatResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Skill instance with cooldown tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub key: String,
    pub last_used_ms: u64,
}

impl Skill {
    /// Special value indicating skill has never been used
    const NEVER_USED: u64 = u64::MAX;

    pub fn new(key: &str) -> Self {
        Skill {
            key: key.to_string(),
            last_used_ms: Self::NEVER_USED,
        }
    }

    pub fn is_ready(&self, current_time_ms: u64) -> bool {
        // If skill has never been used, it's ready
        if self.last_used_ms == Self::NEVER_USED {
            return get_skill(&self.key).is_some(); // Ready if skill exists
        }

        if let Some(def) = get_skill(&self.key) {
            current_time_ms >= self.last_used_ms + def.cooldown_ms
        } else {
            false
        }
    }

    pub fn cooldown_remaining(&self, current_time_ms: u64) -> u64 {
        if let Some(def) = get_skill(&self.key) {
            let ready_at = self.last_used_ms + def.cooldown_ms;
            if current_time_ms >= ready_at {
                0
            } else {
                ready_at - current_time_ms
            }
        } else {
            0
        }
    }
}

/// Skill effects that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    Damage { amount: i32 },
    AreaDamage { amount: i32, radius: usize },
    Buff { stat: BuffType, duration_ms: u64 },
    Heal { amount: i32 },
    Teleport,
    Stun { duration_ms: u64 },
    Poison { damage_per_sec: i32, duration_ms: u64 },
}

/// Types of buffs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffType {
    DamageBoost,
    ArmorBoost,
    SpeedBoost,
    Invulnerability,
}

/// Active buff on a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBuff {
    pub buff_type: BuffType,
    pub value: i32,
    pub expires_at_ms: u64,
}

/// The real-time combat engine
#[derive(Debug)]
pub struct CombatEngine {
    /// Time since combat started (in ms)
    pub combat_time_ms: u64,
    /// Last update time
    last_update: Instant,
    /// Attack cooldown tracking (u64::MAX means never attacked)
    pub last_attack_ms: u64,
    /// Base attack speed (ms between attacks)
    pub attack_interval_ms: u64,
}

impl CombatEngine {
    /// Special value indicating never attacked - allows immediate first attack
    const NEVER_ATTACKED: u64 = u64::MAX;

    pub fn new() -> Self {
        CombatEngine {
            combat_time_ms: 0,
            last_update: Instant::now(),
            last_attack_ms: Self::NEVER_ATTACKED,
            attack_interval_ms: 1000, // 1 attack per second base
        }
    }

    /// Update combat time
    pub fn update(&mut self) {
        let elapsed = self.last_update.elapsed();
        self.combat_time_ms += elapsed.as_millis() as u64;
        self.last_update = Instant::now();
    }

    /// Calculate player damage
    pub fn calculate_player_damage(character: &Character, equipped: &[&Item]) -> i32 {
        let base_stats = character.class.base_stats();

        // Base damage from strength
        let mut damage = base_stats.strength + character.level as i32 * 2;

        // Add weapon damage
        for item in equipped {
            damage += item.total_damage();
            damage += item.get_stat_bonus(AffixStat::Strength) / 2;
        }

        damage.max(1)
    }

    /// Calculate player armor
    pub fn calculate_player_armor(character: &Character, equipped: &[&Item]) -> i32 {
        let base_stats = character.class.base_stats();
        let mut armor = base_stats.armor;

        for item in equipped {
            armor += item.total_armor();
        }

        armor
    }

    /// Calculate critical hit chance
    pub fn calculate_crit_chance(character: &Character, equipped: &[&Item]) -> i32 {
        let base_stats = character.class.base_stats();
        let mut crit = base_stats.dexterity / 5; // 1% per 5 dex

        for item in equipped {
            crit += item.get_stat_bonus(AffixStat::CritChance);
        }

        crit.min(75) // Cap at 75%
    }

    /// Check if attack can happen (cooldown)
    pub fn can_attack(&self, attack_speed_bonus: i32) -> bool {
        // If never attacked, can attack immediately
        if self.last_attack_ms == Self::NEVER_ATTACKED {
            return true;
        }

        let modified_interval =
            (self.attack_interval_ms as i32 * 100 / (100 + attack_speed_bonus)).max(200) as u64;
        self.combat_time_ms >= self.last_attack_ms + modified_interval
    }

    /// Process a player attack
    pub fn process_attack(
        &mut self,
        character: &mut Character,
        target: &mut Monster,
        equipped: &[&Item],
    ) -> CombatResult {
        let mut result = CombatResult::new();

        // Calculate damage
        let base_damage = Self::calculate_player_damage(character, equipped);
        let crit_chance = Self::calculate_crit_chance(character, equipped);

        let mut rng = rand::thread_rng();
        let is_crit = rng.gen_range(0..100) < crit_chance;

        let damage = if is_crit {
            let crit_damage = base_damage * 2;
            result
                .messages
                .push(format!("CRITICAL HIT! {} damage!", crit_damage));
            crit_damage
        } else {
            base_damage
        };

        // Apply damage reduction from monster armor
        let monster_stats = target.monster_type.stats();
        let damage_reduction = monster_stats.armor as f32 / (monster_stats.armor as f32 + 100.0);
        let final_damage = ((damage as f32) * (1.0 - damage_reduction)) as i32;

        target.take_damage(final_damage);
        result.player_damage_dealt = final_damage;

        // Life steal
        let life_steal: i32 = equipped
            .iter()
            .map(|i| i.get_stat_bonus(AffixStat::LifeSteal))
            .sum();
        if life_steal > 0 {
            let heal = final_damage * life_steal / 100;
            character.health = (character.health + heal).min(character.max_health);
        }

        // Check if monster died
        if !target.is_alive() {
            result.monsters_killed.push(target.id);
            result.xp_gained = monster_stats.xp;
            result
                .messages
                .push(format!("{} slain! +{} XP", target.monster_type.name(), monster_stats.xp));
        }

        self.last_attack_ms = self.combat_time_ms;
        result
    }

    /// Process a skill use (works with externally-passed skill)
    pub fn process_skill(
        &mut self,
        character: &mut Character,
        skill: &mut Skill,
        targets: &mut [&mut Monster],
        equipped: &[&Item],
    ) -> CombatResult {
        self.process_skill_with_ref(character, skill, targets, equipped)
    }

    /// Process a skill use by skill index (for use when skill is in character's list)
    pub fn process_skill_by_index(
        &mut self,
        character: &mut Character,
        skill_index: usize,
        targets: &mut [&mut Monster],
        equipped: &[&Item],
    ) -> CombatResult {
        // Get skill info without holding borrow
        let (skill_key, skill_ready) = match character.skills.get(skill_index) {
            Some(s) => (s.key.clone(), s.is_ready(self.combat_time_ms)),
            None => return CombatResult::new(),
        };

        let skill_def = match get_skill(&skill_key) {
            Some(def) => def,
            None => return CombatResult::new(),
        };

        let mut result = CombatResult::new();

        // Check cooldown
        if !skill_ready {
            result
                .messages
                .push(format!("{} is on cooldown!", skill_def.name));
            return result;
        }

        // Check mana
        if character.mana < skill_def.mana_cost {
            result.messages.push("Not enough mana!".to_string());
            return result;
        }

        // Use mana and update skill
        character.mana -= skill_def.mana_cost;
        if let Some(skill) = character.skills.get_mut(skill_index) {
            skill.last_used_ms = self.combat_time_ms;
        }
        result.skill_used = Some(skill_key.clone());

        // Process the skill effects
        self.apply_skill_effects(&skill_key, character, targets, equipped, &mut result);

        result
    }

    /// Process a skill with a mutable skill reference
    fn process_skill_with_ref(
        &mut self,
        character: &mut Character,
        skill: &mut Skill,
        targets: &mut [&mut Monster],
        equipped: &[&Item],
    ) -> CombatResult {
        let mut result = CombatResult::new();

        let skill_key = skill.key.clone();
        let skill_def = match get_skill(&skill_key) {
            Some(def) => def,
            None => return result,
        };

        // Check cooldown
        if !skill.is_ready(self.combat_time_ms) {
            result
                .messages
                .push(format!("{} is on cooldown!", skill_def.name));
            return result;
        }

        // Check mana
        if character.mana < skill_def.mana_cost {
            result.messages.push("Not enough mana!".to_string());
            return result;
        }

        // Use mana and update skill
        character.mana -= skill_def.mana_cost;
        skill.last_used_ms = self.combat_time_ms;
        result.skill_used = Some(skill_key.clone());

        // Process the skill effects
        self.apply_skill_effects(&skill_key, character, targets, equipped, &mut result);

        result
    }

    /// Apply skill effects (shared logic)
    fn apply_skill_effects(
        &self,
        skill_key: &str,
        character: &Character,
        targets: &mut [&mut Monster],
        equipped: &[&Item],
        result: &mut CombatResult,
    ) {
        let skill_def = match get_skill(skill_key) {
            Some(def) => def,
            None => return,
        };

        // Calculate base damage
        let base_damage = Self::calculate_player_damage(character, equipped);
        let skill_damage = base_damage * skill_def.damage_multiplier / 100;

        // Apply skill effects based on skill type
        match skill_key {
            "bash" => {
                // Single target with stun
                if let Some(target) = targets.first_mut() {
                    target.take_damage(skill_damage);
                    result.player_damage_dealt = skill_damage;
                    result
                        .messages
                        .push(format!("Bash! {} damage!", skill_damage));

                    if !target.is_alive() {
                        result.monsters_killed.push(target.id);
                        result.xp_gained += target.monster_type.stats().xp;
                    }
                }
            }
            "whirlwind" => {
                // AoE damage
                for target in targets.iter_mut() {
                    target.take_damage(skill_damage);
                    result.player_damage_dealt += skill_damage;

                    if !target.is_alive() {
                        result.monsters_killed.push(target.id);
                        result.xp_gained += target.monster_type.stats().xp;
                    }
                }
                result
                    .messages
                    .push(format!("Whirlwind! {} total damage!", result.player_damage_dealt));
            }
            "battle_cry" => {
                // Buff - handled by caller
                result
                    .messages
                    .push("Battle Cry! Damage increased!".to_string());
            }
            "iron_skin" => {
                // Defense buff - handled by caller
                result
                    .messages
                    .push("Iron Skin! Defense increased!".to_string());
            }
            "multishot" => {
                // Hit multiple targets
                let hit_count = targets.len().min(3);
                for target in targets.iter_mut().take(hit_count) {
                    target.take_damage(skill_damage);
                    result.player_damage_dealt += skill_damage;

                    if !target.is_alive() {
                        result.monsters_killed.push(target.id);
                        result.xp_gained += target.monster_type.stats().xp;
                    }
                }
                result
                    .messages
                    .push(format!("Multishot! {} arrows!", hit_count));
            }
            "trap" => {
                result.messages.push("Trap placed!".to_string());
            }
            "shadow_step" => {
                result.messages.push("Shadow Step!".to_string());
            }
            "poison_strike" => {
                if let Some(target) = targets.first_mut() {
                    target.take_damage(skill_damage);
                    result.player_damage_dealt = skill_damage;
                    result.messages.push(format!(
                        "Poison Strike! {} damage + poison!",
                        skill_damage
                    ));

                    if !target.is_alive() {
                        result.monsters_killed.push(target.id);
                        result.xp_gained += target.monster_type.stats().xp;
                    }
                }
            }
            "fireball" => {
                // High damage single target
                if let Some(target) = targets.first_mut() {
                    target.take_damage(skill_damage);
                    result.player_damage_dealt = skill_damage;
                    result
                        .messages
                        .push(format!("Fireball! {} fire damage!", skill_damage));

                    if !target.is_alive() {
                        result.monsters_killed.push(target.id);
                        result.xp_gained += target.monster_type.stats().xp;
                    }
                }
            }
            "frost_nova" => {
                // AoE with freeze
                for target in targets.iter_mut() {
                    target.take_damage(skill_damage);
                    result.player_damage_dealt += skill_damage;

                    if !target.is_alive() {
                        result.monsters_killed.push(target.id);
                        result.xp_gained += target.monster_type.stats().xp;
                    }
                }
                result.messages.push("Frost Nova! Enemies frozen!".to_string());
            }
            "teleport" => {
                result.messages.push("Teleport!".to_string());
            }
            "chain_lightning" => {
                // Bounces between targets
                let base_chain_damage = skill_damage;
                for (i, target) in targets.iter_mut().enumerate() {
                    let bounce_damage = base_chain_damage * (100 - i as i32 * 20) / 100;
                    if bounce_damage > 0 {
                        target.take_damage(bounce_damage);
                        result.player_damage_dealt += bounce_damage;

                        if !target.is_alive() {
                            result.monsters_killed.push(target.id);
                            result.xp_gained += target.monster_type.stats().xp;
                        }
                    }
                }
                result
                    .messages
                    .push(format!("Chain Lightning! {} total damage!", result.player_damage_dealt));
            }
            _ => {}
        }
    }

    /// Process monster attacks on player
    pub fn process_monster_attacks(
        &mut self,
        character: &mut Character,
        monsters: &mut [&mut Monster],
        player_armor: i32,
    ) -> CombatResult {
        let mut result = CombatResult::new();
        let mut rng = rand::thread_rng();

        for monster in monsters.iter_mut() {
            if !monster.is_alive() {
                continue;
            }

            let stats = monster.monster_type.stats();

            // Check attack cooldown
            let attack_interval = 2000 * 100 / stats.speed.max(1) as u64;
            if self.combat_time_ms < monster.last_attack_ms + attack_interval {
                continue;
            }

            // Calculate damage
            let base_damage = stats.damage + rng.gen_range(-2..=2);

            // Armor reduction
            let damage_reduction = player_armor as f32 / (player_armor as f32 + 100.0);
            let final_damage = ((base_damage as f32) * (1.0 - damage_reduction)).max(1.0) as i32;

            character.health -= final_damage;
            result.player_damage_taken += final_damage;
            monster.last_attack_ms = self.combat_time_ms;

            result.messages.push(format!(
                "{} hits you for {} damage!",
                monster.monster_type.name(),
                final_damage
            ));

            if character.health <= 0 {
                result.player_died = true;
                result.messages.push("You have died!".to_string());
                break;
            }
        }

        result
    }
}

impl Default for CombatEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::depths_of_diablo::data::{CharacterClass, MonsterType};
    use crate::games::depths_of_diablo::items::{Affix, Item, ItemRarity, ItemType};

    // Helper function to create a test item with specific stats
    fn create_test_weapon(damage: i32, life_steal: i32, crit_chance: i32) -> Item {
        let mut affixes = Vec::new();
        if damage > 0 {
            affixes.push(Affix {
                name: "Sharp".to_string(),
                stat: AffixStat::Damage,
                value: damage,
                is_prefix: true,
            });
        }
        if life_steal > 0 {
            affixes.push(Affix {
                name: "of the Vampire".to_string(),
                stat: AffixStat::LifeSteal,
                value: life_steal,
                is_prefix: false,
            });
        }
        if crit_chance > 0 {
            affixes.push(Affix {
                name: "of Precision".to_string(),
                stat: AffixStat::CritChance,
                value: crit_chance,
                is_prefix: false,
            });
        }
        Item {
            id: 1,
            item_type: ItemType::Sword,
            rarity: ItemRarity::Rare,
            name: "Test Sword".to_string(),
            base_damage: 10,
            base_armor: 0,
            affixes,
            level_req: 1,
            floor_found: 1,
        }
    }

    fn create_test_armor(armor: i32) -> Item {
        let mut affixes = Vec::new();
        if armor > 0 {
            affixes.push(Affix {
                name: "Sturdy".to_string(),
                stat: AffixStat::Armor,
                value: armor,
                is_prefix: true,
            });
        }
        Item {
            id: 2,
            item_type: ItemType::Chest,
            rarity: ItemRarity::Magic,
            name: "Test Armor".to_string(),
            base_damage: 0,
            base_armor: 15,
            affixes,
            level_req: 1,
            floor_found: 1,
        }
    }

    fn create_test_monster() -> Monster {
        Monster::new(1, MonsterType::Zombie, 5, 5)
    }

    // ===== Skill Tests =====

    #[test]
    fn test_skill_cooldown() {
        let mut skill = Skill::new("bash");
        assert!(skill.is_ready(0)); // New skill is ready

        skill.last_used_ms = 0; // Simulate skill was used at time 0
        assert!(!skill.is_ready(1000)); // Bash has 2000ms cooldown, so not ready at 1000ms
        assert!(skill.is_ready(2500)); // Ready at 2500ms (after cooldown)
    }

    #[test]
    fn test_skill_new() {
        let skill = Skill::new("fireball");
        assert_eq!(skill.key, "fireball");
        assert_eq!(skill.last_used_ms, Skill::NEVER_USED);
    }

    #[test]
    fn test_skill_cooldown_remaining() {
        let mut skill = Skill::new("bash");
        skill.last_used_ms = 1000;

        // Bash has 2000ms cooldown, so at time 2000, 1000ms remaining
        assert_eq!(skill.cooldown_remaining(2000), 1000);

        // At time 3000, should be ready (0 remaining)
        assert_eq!(skill.cooldown_remaining(3000), 0);

        // At time 4000, still 0
        assert_eq!(skill.cooldown_remaining(4000), 0);
    }

    #[test]
    fn test_skill_cooldown_remaining_unknown_skill() {
        let skill = Skill::new("unknown_skill");
        assert_eq!(skill.cooldown_remaining(1000), 0);
    }

    #[test]
    fn test_skill_is_ready_unknown_skill() {
        let skill = Skill::new("unknown_skill");
        // Unknown skill should never be ready
        assert!(!skill.is_ready(0));
        assert!(!skill.is_ready(100000));
    }

    // ===== CombatResult Tests =====

    #[test]
    fn test_combat_result_default() {
        let result = CombatResult::new();
        assert_eq!(result.player_damage_dealt, 0);
        assert_eq!(result.player_damage_taken, 0);
        assert!(!result.player_died);
    }

    #[test]
    fn test_combat_result_default_trait() {
        let result = CombatResult::default();
        assert_eq!(result.player_damage_dealt, 0);
        assert!(result.monsters_killed.is_empty());
        assert!(result.messages.is_empty());
        assert!(result.skill_used.is_none());
    }

    // ===== CombatEngine Tests =====

    #[test]
    fn test_combat_engine_new() {
        let engine = CombatEngine::new();
        assert_eq!(engine.combat_time_ms, 0);
        // NEVER_ATTACKED sentinel allows immediate first attack
        assert_eq!(engine.last_attack_ms, CombatEngine::NEVER_ATTACKED);
        assert_eq!(engine.attack_interval_ms, 1000);
    }

    #[test]
    fn test_combat_engine_default() {
        let engine = CombatEngine::default();
        assert_eq!(engine.combat_time_ms, 0);
        assert_eq!(engine.attack_interval_ms, 1000);
    }

    #[test]
    fn test_calculate_damage() {
        let mut character = Character::new("Test", CharacterClass::Warrior);
        let damage = CombatEngine::calculate_player_damage(&character, &[]);
        assert!(damage > 0);

        // Higher level = more damage
        character.level = 10;
        let higher_damage = CombatEngine::calculate_player_damage(&character, &[]);
        assert!(higher_damage > damage);
    }

    #[test]
    fn test_calculate_damage_with_weapon() {
        let character = Character::new("Test", CharacterClass::Warrior);
        let weapon = create_test_weapon(20, 0, 0);

        let damage_no_weapon = CombatEngine::calculate_player_damage(&character, &[]);
        let damage_with_weapon = CombatEngine::calculate_player_damage(&character, &[&weapon]);

        assert!(damage_with_weapon > damage_no_weapon);
    }

    #[test]
    fn test_calculate_damage_with_strength_bonus() {
        let character = Character::new("Test", CharacterClass::Warrior);

        // Create weapon with strength bonus
        let mut weapon = create_test_weapon(10, 0, 0);
        weapon.affixes.push(Affix {
            name: "of the Bear".to_string(),
            stat: AffixStat::Strength,
            value: 10,
            is_prefix: false,
        });

        let damage_basic = CombatEngine::calculate_player_damage(&character, &[]);
        let damage_str = CombatEngine::calculate_player_damage(&character, &[&weapon]);

        // Strength bonus adds damage (divided by 2)
        assert!(damage_str > damage_basic);
    }

    #[test]
    fn test_calculate_damage_minimum() {
        // Even with very low stats, damage should be at least 1
        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.level = 1;

        let damage = CombatEngine::calculate_player_damage(&character, &[]);
        assert!(damage >= 1);
    }

    #[test]
    fn test_calculate_armor() {
        let character = Character::new("Test", CharacterClass::Warrior);
        let armor = CombatEngine::calculate_player_armor(&character, &[]);

        // Warrior base armor is 20
        assert_eq!(armor, 20);
    }

    #[test]
    fn test_calculate_armor_with_equipment() {
        let character = Character::new("Test", CharacterClass::Warrior);
        let chest = create_test_armor(10);

        let armor_no_equip = CombatEngine::calculate_player_armor(&character, &[]);
        let armor_with_equip = CombatEngine::calculate_player_armor(&character, &[&chest]);

        // Should have base + item base_armor + affix armor
        assert!(armor_with_equip > armor_no_equip);
        assert_eq!(armor_with_equip, 20 + 15 + 10); // base + item base + affix
    }

    #[test]
    fn test_calculate_crit_chance() {
        let character = Character::new("Test", CharacterClass::Warrior);
        let crit = CombatEngine::calculate_crit_chance(&character, &[]);

        // Warrior has 15 dex, so 15/5 = 3% crit
        assert_eq!(crit, 3);
    }

    #[test]
    fn test_calculate_crit_chance_rogue() {
        let character = Character::new("Test", CharacterClass::Rogue);
        let crit = CombatEngine::calculate_crit_chance(&character, &[]);

        // Rogue has 25 dex, so 25/5 = 5% crit
        assert_eq!(crit, 5);
    }

    #[test]
    fn test_calculate_crit_chance_with_equipment() {
        let character = Character::new("Test", CharacterClass::Warrior);
        let weapon = create_test_weapon(0, 0, 20);

        let crit = CombatEngine::calculate_crit_chance(&character, &[&weapon]);

        // 3% base + 20% from equipment = 23%
        assert_eq!(crit, 23);
    }

    #[test]
    fn test_calculate_crit_chance_capped() {
        let character = Character::new("Test", CharacterClass::Rogue);
        let weapon = create_test_weapon(0, 0, 100);

        let crit = CombatEngine::calculate_crit_chance(&character, &[&weapon]);

        // Should be capped at 75%
        assert_eq!(crit, 75);
    }

    #[test]
    fn test_attack_cooldown() {
        let engine = CombatEngine::new();
        assert!(engine.can_attack(0)); // Fresh engine can attack

        // With speed bonus, interval is shorter
        let fast_engine = CombatEngine {
            combat_time_ms: 500,
            last_attack_ms: 0,
            attack_interval_ms: 1000,
            ..CombatEngine::new()
        };
        assert!(!fast_engine.can_attack(0)); // 500ms not enough
        assert!(fast_engine.can_attack(100)); // With 100% bonus, only 500ms needed
    }

    #[test]
    fn test_can_attack_minimum_interval() {
        // Attack speed bonus should not reduce interval below 200ms
        let engine = CombatEngine {
            combat_time_ms: 150,
            last_attack_ms: 0,
            attack_interval_ms: 1000,
            ..CombatEngine::new()
        };

        // Even with huge speed bonus, minimum interval is 200ms
        assert!(!engine.can_attack(1000));
    }

    #[test]
    fn test_can_attack_at_exact_interval() {
        let engine = CombatEngine {
            combat_time_ms: 1000,
            last_attack_ms: 0,
            attack_interval_ms: 1000,
            ..CombatEngine::new()
        };

        assert!(engine.can_attack(0));
    }

    // ===== Attack Processing Tests =====

    #[test]
    fn test_process_attack_basic() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut monster = create_test_monster();

        let initial_health = monster.health;
        let result = engine.process_attack(&mut character, &mut monster, &[]);

        assert!(result.player_damage_dealt > 0);
        assert!(monster.health < initial_health);
        assert_eq!(engine.last_attack_ms, engine.combat_time_ms);
    }

    #[test]
    fn test_process_attack_kills_monster() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.level = 50; // High level for high damage

        let mut monster = create_test_monster();
        monster.health = 1; // Almost dead

        let result = engine.process_attack(&mut character, &mut monster, &[]);

        assert!(!monster.is_alive());
        assert!(result.monsters_killed.contains(&monster.id));
        assert!(result.xp_gained > 0);
        assert!(result.messages.iter().any(|m| m.contains("slain")));
    }

    #[test]
    fn test_process_attack_life_steal() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.health = 50; // Damaged

        let mut monster = create_test_monster();
        let weapon = create_test_weapon(20, 10, 0); // 10% life steal

        let initial_health = character.health;
        let result = engine.process_attack(&mut character, &mut monster, &[&weapon]);

        // Should heal based on damage dealt
        if result.player_damage_dealt > 0 {
            let expected_heal = result.player_damage_dealt * 10 / 100;
            if expected_heal > 0 {
                assert!(character.health > initial_health);
            }
        }
    }

    #[test]
    fn test_process_attack_life_steal_caps_at_max() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.health = character.max_health - 1; // Almost full

        let mut monster = create_test_monster();
        let weapon = create_test_weapon(50, 50, 0); // 50% life steal

        engine.process_attack(&mut character, &mut monster, &[&weapon]);

        // Should not exceed max health
        assert!(character.health <= character.max_health);
    }

    #[test]
    fn test_process_attack_monster_armor_reduces_damage() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);

        // Create two monsters with different armor
        let mut zombie = Monster::new(1, MonsterType::Zombie, 5, 5); // 2 armor
        let mut blood_knight = Monster::new(2, MonsterType::BloodKnight, 6, 6); // 35 armor

        let zombie_initial = zombie.health;
        let bk_initial = blood_knight.health;

        engine.combat_time_ms = 0;
        engine.last_attack_ms = 0;
        let _zombie_result = engine.process_attack(&mut character, &mut zombie, &[]);

        engine.combat_time_ms = 2000;
        engine.last_attack_ms = 0;
        let _bk_result = engine.process_attack(&mut character, &mut blood_knight, &[]);

        let zombie_damage = zombie_initial - zombie.health;
        let bk_damage = bk_initial - blood_knight.health;

        // Blood Knight with higher armor should take less damage
        assert!(bk_damage < zombie_damage);
    }

    // ===== Skill Processing Tests =====

    #[test]
    fn test_process_skill_bash() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut skill = Skill::new("bash");
        let mut monster = create_test_monster();

        let initial_health = monster.health;
        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        assert!(result.player_damage_dealt > 0);
        assert!(monster.health < initial_health);
        assert_eq!(result.skill_used, Some("bash".to_string()));
        assert!(result.messages.iter().any(|m| m.contains("Bash")));
    }

    #[test]
    fn test_process_skill_whirlwind_aoe() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.mana = 100;
        let mut skill = Skill::new("whirlwind");

        let mut monster1 = Monster::new(1, MonsterType::Zombie, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::Skeleton, 6, 6);

        let m1_initial = monster1.health;
        let m2_initial = monster2.health;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster1, &mut monster2],
            &[],
        );

        // Both monsters should take damage
        assert!(monster1.health < m1_initial);
        assert!(monster2.health < m2_initial);
        assert!(result.messages.iter().any(|m| m.contains("Whirlwind")));
    }

    #[test]
    fn test_process_skill_multishot() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Rogue);
        character.mana = 100;
        let mut skill = Skill::new("multishot");

        let mut monster1 = Monster::new(1, MonsterType::Zombie, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::Skeleton, 6, 6);
        let mut monster3 = Monster::new(3, MonsterType::FallenOne, 7, 7);
        let mut monster4 = Monster::new(4, MonsterType::Zombie, 8, 8);

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster1, &mut monster2, &mut monster3, &mut monster4],
            &[],
        );

        // Multishot hits up to 3 targets
        assert!(result.messages.iter().any(|m| m.contains("Multishot")));
        assert!(result.messages.iter().any(|m| m.contains("3 arrows")));
    }

    #[test]
    fn test_process_skill_fireball() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.mana = 100;
        let mut skill = Skill::new("fireball");
        let mut monster = create_test_monster();

        let initial_health = monster.health;
        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        assert!(result.player_damage_dealt > 0);
        assert!(monster.health < initial_health);
        assert!(result.messages.iter().any(|m| m.contains("Fireball")));
    }

    #[test]
    fn test_process_skill_frost_nova_aoe() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.mana = 100;
        let mut skill = Skill::new("frost_nova");

        let mut monster1 = Monster::new(1, MonsterType::Zombie, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::Skeleton, 6, 6);

        let m1_initial = monster1.health;
        let m2_initial = monster2.health;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster1, &mut monster2],
            &[],
        );

        assert!(monster1.health < m1_initial);
        assert!(monster2.health < m2_initial);
        assert!(result.messages.iter().any(|m| m.contains("Frost Nova")));
    }

    #[test]
    fn test_process_skill_chain_lightning() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.mana = 100;
        let mut skill = Skill::new("chain_lightning");

        let mut monster1 = Monster::new(1, MonsterType::Zombie, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::Skeleton, 6, 6);
        let mut monster3 = Monster::new(3, MonsterType::FallenOne, 7, 7);

        let m1_initial = monster1.health;
        let m2_initial = monster2.health;
        let m3_initial = monster3.health;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster1, &mut monster2, &mut monster3],
            &[],
        );

        // All should take damage, but decreasing amounts
        assert!(monster1.health < m1_initial);
        assert!(monster2.health < m2_initial);
        assert!(monster3.health < m3_initial);

        // Damage should decrease for each bounce
        let m1_damage = m1_initial - monster1.health;
        let m2_damage = m2_initial - monster2.health;
        let m3_damage = m3_initial - monster3.health;

        assert!(m1_damage >= m2_damage);
        assert!(m2_damage >= m3_damage);
        assert!(result.messages.iter().any(|m| m.contains("Chain Lightning")));
    }

    #[test]
    fn test_process_skill_teleport() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.mana = 100;
        let mut skill = Skill::new("teleport");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert!(result.messages.iter().any(|m| m.contains("Teleport")));
        assert_eq!(result.skill_used, Some("teleport".to_string()));
    }

    #[test]
    fn test_process_skill_battle_cry() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.mana = 100;
        let mut skill = Skill::new("battle_cry");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert!(result.messages.iter().any(|m| m.contains("Battle Cry")));
    }

    #[test]
    fn test_process_skill_iron_skin() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.mana = 100;
        let mut skill = Skill::new("iron_skin");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert!(result.messages.iter().any(|m| m.contains("Iron Skin")));
    }

    #[test]
    fn test_process_skill_trap() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Rogue);
        character.mana = 100;
        let mut skill = Skill::new("trap");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert!(result.messages.iter().any(|m| m.contains("Trap")));
    }

    #[test]
    fn test_process_skill_shadow_step() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Rogue);
        character.mana = 100;
        let mut skill = Skill::new("shadow_step");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert!(result.messages.iter().any(|m| m.contains("Shadow Step")));
    }

    #[test]
    fn test_process_skill_poison_strike() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Rogue);
        character.mana = 100;
        let mut skill = Skill::new("poison_strike");
        let mut monster = create_test_monster();

        let initial_health = monster.health;
        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        assert!(monster.health < initial_health);
        assert!(result.messages.iter().any(|m| m.contains("Poison Strike")));
    }

    #[test]
    fn test_process_skill_on_cooldown() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 1000;

        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut skill = Skill::new("bash");
        skill.last_used_ms = 500; // Used 500ms ago, but bash has 2000ms cooldown

        let mut monster = create_test_monster();
        let initial_health = monster.health;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        // Should not deal damage because skill is on cooldown
        assert_eq!(result.player_damage_dealt, 0);
        assert_eq!(monster.health, initial_health);
        assert!(result.messages.iter().any(|m| m.contains("cooldown")));
    }

    #[test]
    fn test_process_skill_not_enough_mana() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.mana = 0; // No mana

        let mut skill = Skill::new("bash");
        let mut monster = create_test_monster();
        let initial_health = monster.health;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        assert_eq!(result.player_damage_dealt, 0);
        assert_eq!(monster.health, initial_health);
        assert!(result.messages.iter().any(|m| m.contains("mana")));
    }

    #[test]
    fn test_process_skill_uses_mana() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.mana = 30;

        let mut skill = Skill::new("bash"); // Costs 5 mana

        engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert_eq!(character.mana, 25);
    }

    #[test]
    fn test_process_skill_updates_cooldown() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 5000;

        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut skill = Skill::new("bash");
        skill.last_used_ms = 0;

        engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        assert_eq!(skill.last_used_ms, 5000);
    }

    #[test]
    fn test_process_skill_unknown_skill() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut skill = Skill::new("unknown_skill");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [],
            &[],
        );

        // Unknown skill should return empty result
        assert_eq!(result.player_damage_dealt, 0);
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_process_skill_kills_monster() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        character.level = 50; // High level for high damage

        let mut skill = Skill::new("bash");
        let mut monster = create_test_monster();
        monster.health = 1;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        assert!(!monster.is_alive());
        assert!(result.monsters_killed.contains(&monster.id));
        assert!(result.xp_gained > 0);
    }

    // ===== Monster Attack Tests =====

    #[test]
    fn test_process_monster_attacks() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 5000; // Enough time for monsters to attack

        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut monster = create_test_monster();

        let initial_health = character.health;
        let result = engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster],
            20, // player armor
        );

        assert!(result.player_damage_taken > 0);
        assert!(character.health < initial_health);
        assert!(result.messages.iter().any(|m| m.contains("hits you")));
    }

    #[test]
    fn test_process_monster_attacks_cooldown() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 100; // Not enough time

        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut monster = create_test_monster();
        monster.last_attack_ms = 0;

        let initial_health = character.health;
        let result = engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster],
            20,
        );

        // Monster should not attack yet
        assert_eq!(result.player_damage_taken, 0);
        assert_eq!(character.health, initial_health);
    }

    #[test]
    fn test_process_monster_attacks_dead_monster_skipped() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 5000;

        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut monster = create_test_monster();
        monster.health = 0; // Dead

        let initial_health = character.health;
        let result = engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster],
            20,
        );

        assert_eq!(result.player_damage_taken, 0);
        assert_eq!(character.health, initial_health);
    }

    #[test]
    fn test_process_monster_attacks_armor_reduces_damage() {
        // Run multiple iterations to average out randomness in damage calculation
        // Monster base damage is 5 with +/- 2 random variation (3-7 range)
        // Low armor (0): ~0% reduction, High armor (500): ~83% reduction
        // Even with worst-case rolls, high armor should consistently take less damage
        let mut total_low = 0;
        let mut total_high = 0;
        let iterations = 10;

        for i in 0..iterations {
            let mut engine = CombatEngine::new();
            engine.combat_time_ms = 5000 + (i as u64 * 10000);

            let mut char_low_armor = Character::new("Test", CharacterClass::Sorcerer);
            let mut char_high_armor = Character::new("Test", CharacterClass::Warrior);

            let mut monster1 = create_test_monster();
            let mut monster2 = create_test_monster();

            let result_low = engine.process_monster_attacks(
                &mut char_low_armor,
                &mut [&mut monster1],
                0, // No armor
            );

            engine.combat_time_ms += 5000;
            monster2.last_attack_ms = 0;

            let result_high = engine.process_monster_attacks(
                &mut char_high_armor,
                &mut [&mut monster2],
                500, // Very high armor (83% reduction)
            );

            total_low += result_low.player_damage_taken;
            total_high += result_high.player_damage_taken;
        }

        // Higher armor should result in less total damage over multiple attacks
        assert!(
            total_high < total_low,
            "High armor total ({}) should be less than low armor total ({})",
            total_high,
            total_low
        );
    }

    #[test]
    fn test_process_monster_attacks_player_death() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 5000;

        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.health = 1; // Almost dead

        let mut monster = Monster::new(1, MonsterType::DiabloBoss, 5, 5); // High damage boss

        let result = engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster],
            0, // No armor
        );

        assert!(result.player_died);
        assert!(result.messages.iter().any(|m| m.contains("died")));
    }

    #[test]
    fn test_process_monster_attacks_multiple_monsters() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 10000;

        let mut character = Character::new("Test", CharacterClass::Warrior);

        let mut monster1 = Monster::new(1, MonsterType::Zombie, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::Skeleton, 6, 6);

        let result = engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster1, &mut monster2],
            10,
        );

        // Both monsters should have attacked
        assert!(result.messages.len() >= 2);
        assert!(result.messages.iter().any(|m| m.contains("Zombie")));
        assert!(result.messages.iter().any(|m| m.contains("Skeleton")));
    }

    #[test]
    fn test_process_monster_attacks_updates_last_attack() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 5000;

        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut monster = create_test_monster();
        monster.last_attack_ms = 0;

        engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster],
            20,
        );

        assert_eq!(monster.last_attack_ms, 5000);
    }

    #[test]
    fn test_process_monster_attacks_stops_on_death() {
        let mut engine = CombatEngine::new();
        engine.combat_time_ms = 10000;

        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.health = 1;

        // Create a high damage boss that will kill in one hit
        let mut monster1 = Monster::new(1, MonsterType::DiabloBoss, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::DiabloBoss, 6, 6);

        let result = engine.process_monster_attacks(
            &mut character,
            &mut [&mut monster1, &mut monster2],
            0,
        );

        assert!(result.player_died);
        // Should stop processing after death
        assert!(result.messages.iter().any(|m| m.contains("died")));
    }

    // ===== Buff Type Tests =====

    #[test]
    fn test_buff_type_variants() {
        let damage = BuffType::DamageBoost;
        let armor = BuffType::ArmorBoost;
        let speed = BuffType::SpeedBoost;
        let invuln = BuffType::Invulnerability;

        assert_eq!(damage, BuffType::DamageBoost);
        assert_eq!(armor, BuffType::ArmorBoost);
        assert_eq!(speed, BuffType::SpeedBoost);
        assert_eq!(invuln, BuffType::Invulnerability);
        assert_ne!(damage, armor);
    }

    #[test]
    fn test_active_buff_creation() {
        let buff = ActiveBuff {
            buff_type: BuffType::DamageBoost,
            value: 25,
            expires_at_ms: 10000,
        };

        assert_eq!(buff.buff_type, BuffType::DamageBoost);
        assert_eq!(buff.value, 25);
        assert_eq!(buff.expires_at_ms, 10000);
    }

    // ===== Skill Effect Tests =====

    #[test]
    fn test_skill_effect_variants() {
        let damage = SkillEffect::Damage { amount: 100 };
        let area = SkillEffect::AreaDamage { amount: 50, radius: 3 };
        let buff = SkillEffect::Buff {
            stat: BuffType::DamageBoost,
            duration_ms: 5000
        };
        let heal = SkillEffect::Heal { amount: 50 };
        let teleport = SkillEffect::Teleport;
        let stun = SkillEffect::Stun { duration_ms: 2000 };
        let poison = SkillEffect::Poison {
            damage_per_sec: 10,
            duration_ms: 5000
        };

        // Just verify they can be created
        match damage {
            SkillEffect::Damage { amount } => assert_eq!(amount, 100),
            _ => panic!("Wrong variant"),
        }
        match area {
            SkillEffect::AreaDamage { amount, radius } => {
                assert_eq!(amount, 50);
                assert_eq!(radius, 3);
            }
            _ => panic!("Wrong variant"),
        }
        match buff {
            SkillEffect::Buff { stat, duration_ms } => {
                assert_eq!(stat, BuffType::DamageBoost);
                assert_eq!(duration_ms, 5000);
            }
            _ => panic!("Wrong variant"),
        }
        match heal {
            SkillEffect::Heal { amount } => assert_eq!(amount, 50),
            _ => panic!("Wrong variant"),
        }
        assert!(matches!(teleport, SkillEffect::Teleport));
        match stun {
            SkillEffect::Stun { duration_ms } => assert_eq!(duration_ms, 2000),
            _ => panic!("Wrong variant"),
        }
        match poison {
            SkillEffect::Poison { damage_per_sec, duration_ms } => {
                assert_eq!(damage_per_sec, 10);
                assert_eq!(duration_ms, 5000);
            }
            _ => panic!("Wrong variant"),
        }
    }

    // ===== Combat Action Tests =====

    #[test]
    fn test_combat_action_variants() {
        let attack = CombatAction::Attack;
        let skill = CombatAction::UseSkill("fireball".to_string());
        let health_pot = CombatAction::UsePotion(true);
        let mana_pot = CombatAction::UsePotion(false);
        let move_action = CombatAction::Move(1, -1);
        let idle = CombatAction::Idle;

        assert_eq!(attack, CombatAction::Attack);
        assert_eq!(skill, CombatAction::UseSkill("fireball".to_string()));
        assert_eq!(health_pot, CombatAction::UsePotion(true));
        assert_eq!(mana_pot, CombatAction::UsePotion(false));
        assert_eq!(move_action, CombatAction::Move(1, -1));
        assert_eq!(idle, CombatAction::Idle);
    }

    // ===== Edge Case Tests =====

    #[test]
    fn test_process_skill_no_targets() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut skill = Skill::new("bash");

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [], // No targets
            &[],
        );

        // Should still use mana and go on cooldown, but deal no damage
        assert_eq!(result.player_damage_dealt, 0);
    }

    #[test]
    fn test_multishot_fewer_than_three_targets() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Rogue);
        character.mana = 100;
        let mut skill = Skill::new("multishot");

        let mut monster1 = Monster::new(1, MonsterType::Zombie, 5, 5);
        let mut monster2 = Monster::new(2, MonsterType::Skeleton, 6, 6);

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster1, &mut monster2],
            &[],
        );

        // Should hit both targets (less than max of 3)
        assert!(result.messages.iter().any(|m| m.contains("2 arrows")));
    }

    #[test]
    fn test_chain_lightning_single_target() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Sorcerer);
        character.mana = 100;
        let mut skill = Skill::new("chain_lightning");

        let mut monster = create_test_monster();
        let initial_health = monster.health;

        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[],
        );

        // Should still work with single target
        assert!(monster.health < initial_health);
        assert!(result.player_damage_dealt > 0);
    }

    #[test]
    fn test_process_skill_with_equipment_bonus() {
        let mut engine = CombatEngine::new();
        let mut character = Character::new("Test", CharacterClass::Warrior);
        let mut skill = Skill::new("bash");
        let mut monster = create_test_monster();

        let weapon = create_test_weapon(50, 0, 0);

        let initial_health = monster.health;
        let result = engine.process_skill(
            &mut character,
            &mut skill,
            &mut [&mut monster],
            &[&weapon],
        );

        // With weapon, should deal more damage
        assert!(result.player_damage_dealt > 0);
        assert!(monster.health < initial_health);
    }
}
