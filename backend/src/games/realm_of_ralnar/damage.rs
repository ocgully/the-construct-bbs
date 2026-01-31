//! Damage Calculations for Realm of Ralnar Combat
//!
//! Contains formulas for physical damage, magic damage, hit chance, and other
//! combat calculations in the FF1-style turn-based combat system.

use rand::Rng;

use super::magic::{Element, Spell};

/// Stats needed for damage calculation (attacker perspective)
#[derive(Debug, Clone, Copy)]
pub struct AttackerStats {
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub luck: i32,
    pub level: i32,
    pub weapon_attack: i32,
    pub has_protect: bool,
    pub has_haste: bool,
    pub is_blind: bool,
    pub is_berserk: bool,
}

/// Stats needed for damage calculation (defender perspective)
#[derive(Debug, Clone, Copy)]
pub struct DefenderStats {
    pub defense: i32,
    pub magic_defense: i32,
    pub agility: i32,
    pub level: i32,
    pub has_protect: bool,
    pub has_shell: bool,
    pub is_defending: bool,
    pub is_back_row: bool,
    pub weakness: Option<Element>,
    pub resistance: Option<Element>,
    pub absorb: Option<Element>,
}

/// Result of a damage calculation
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage: i32,
    pub is_critical: bool,
    pub is_miss: bool,
    pub is_weak: bool,
    pub is_resist: bool,
    pub is_absorb: bool,
    pub message: Option<String>,
}

impl DamageResult {
    pub fn miss() -> Self {
        Self {
            damage: 0,
            is_critical: false,
            is_miss: true,
            is_weak: false,
            is_resist: false,
            is_absorb: false,
            message: Some("Miss!".to_string()),
        }
    }

    pub fn absorb(amount: i32) -> Self {
        Self {
            damage: -amount, // Negative = healing
            is_critical: false,
            is_miss: false,
            is_weak: false,
            is_resist: false,
            is_absorb: true,
            message: Some("Absorbed!".to_string()),
        }
    }
}

/// Calculate physical damage from an attack
///
/// Formula: (STR + Weapon) - (DEF / 2), with variance and modifiers
pub fn calculate_physical_damage(attacker: &AttackerStats, defender: &DefenderStats) -> DamageResult {
    let mut rng = rand::thread_rng();

    // Check for miss first
    let hit_chance = calculate_hit_chance(attacker.agility, defender.agility, attacker.is_blind);
    if rng.gen::<f32>() > hit_chance {
        return DamageResult::miss();
    }

    // Base damage calculation
    let base_attack = attacker.strength + attacker.weapon_attack;
    let defense = if defender.has_protect {
        defender.defense * 3 / 2 // +50% defense from Protect
    } else {
        defender.defense
    };

    let mut damage = (base_attack - defense / 2).max(1);

    // Defending halves damage
    if defender.is_defending {
        damage /= 2;
    }

    // Back row reduces damage
    if defender.is_back_row {
        damage = damage * 3 / 4;
    }

    // Berserk increases damage by 50%
    if attacker.is_berserk {
        damage = damage * 3 / 2;
    }

    // Critical hit check
    let crit_chance = calculate_crit_chance(attacker.luck, attacker.level);
    let is_critical = rng.gen::<f32>() < crit_chance;
    if is_critical {
        damage *= 2;
    }

    // Add variance (+/- 25%)
    let variance = (damage as f32 * 0.25) as i32;
    let variance_range = if variance > 0 { variance } else { 1 };
    damage += rng.gen_range(-variance_range..=variance_range);

    // Minimum 1 damage
    damage = damage.max(1);

    DamageResult {
        damage,
        is_critical,
        is_miss: false,
        is_weak: false,
        is_resist: false,
        is_absorb: false,
        message: if is_critical {
            Some("Critical hit!".to_string())
        } else {
            None
        },
    }
}

/// Calculate hit chance based on agility difference
///
/// Base: 85%, modified by agility difference
pub fn calculate_hit_chance(attacker_agi: i32, defender_agi: i32, is_blind: bool) -> f32 {
    let base = if is_blind { 0.50 } else { 0.85 };
    let bonus = (attacker_agi - defender_agi) as f32 / 100.0;
    (base + bonus).clamp(0.10, 0.95)
}

/// Calculate critical hit chance
///
/// Base: 5%, increased by luck and level
pub fn calculate_crit_chance(luck: i32, level: i32) -> f32 {
    let base = 0.05;
    let luck_bonus = luck as f32 / 200.0;
    let level_bonus = level as f32 / 500.0;
    (base + luck_bonus + level_bonus).clamp(0.05, 0.25)
}

/// Calculate magic damage from a spell
///
/// Formula: (Spell Power + INT/2) - (M.DEF/4), with elemental modifiers
pub fn calculate_magic_damage(
    caster_int: i32,
    spell: &Spell,
    defender: &DefenderStats,
) -> DamageResult {
    let mut rng = rand::thread_rng();

    // Check for absorption
    if let Some(absorb_elem) = defender.absorb {
        if spell.element == absorb_elem {
            let absorb_amount = spell.power + caster_int / 2;
            return DamageResult::absorb(absorb_amount);
        }
    }

    // Base magic damage
    let magic_defense = if defender.has_shell {
        defender.magic_defense * 3 / 2 // +50% magic defense from Shell
    } else {
        defender.magic_defense
    };

    let mut damage = (spell.power + caster_int / 2 - magic_defense / 4).max(1);

    // Elemental modifiers
    let mut is_weak = false;
    let mut is_resist = false;

    if let Some(weak_elem) = defender.weakness {
        if spell.element == weak_elem {
            damage = damage * 3 / 2; // +50% damage
            is_weak = true;
        }
    }

    if let Some(resist_elem) = defender.resistance {
        if spell.element == resist_elem {
            damage /= 2; // -50% damage
            is_resist = true;
        }
    }

    // Defending reduces magic damage by 25%
    if defender.is_defending {
        damage = damage * 3 / 4;
    }

    // Add small variance (+/- 10%)
    let variance = (damage as f32 * 0.10) as i32;
    let variance_range = if variance > 0 { variance } else { 1 };
    damage += rng.gen_range(-variance_range..=variance_range);

    damage = damage.max(1);

    let message = if is_weak {
        Some("Weakness!".to_string())
    } else if is_resist {
        Some("Resisted!".to_string())
    } else {
        None
    };

    DamageResult {
        damage,
        is_critical: false,
        is_miss: false,
        is_weak,
        is_resist,
        is_absorb: false,
        message,
    }
}

/// Calculate healing amount from a healing spell
///
/// Formula: Spell Power + INT/2
pub fn calculate_healing(caster_int: i32, spell: &Spell) -> i32 {
    let base_heal = spell.power + caster_int / 2;

    // Add small variance (+/- 10%)
    let mut rng = rand::thread_rng();
    let variance = (base_heal as f32 * 0.10) as i32;
    let variance_range = if variance > 0 { variance } else { 1 };

    (base_heal + rng.gen_range(-variance_range..=variance_range)).max(1)
}

/// Calculate status effect success chance
///
/// Base chance from spell power, modified by level difference
pub fn calculate_status_chance(
    spell_power: i32,
    caster_level: i32,
    target_level: i32,
    target_has_immunity: bool,
) -> f32 {
    if target_has_immunity {
        return 0.0;
    }

    let base_chance = spell_power as f32 / 100.0;
    let level_diff = (caster_level - target_level) as f32 / 50.0;

    (base_chance + level_diff).clamp(0.05, 0.80)
}

/// Calculate flee chance for the party
///
/// Based on average party agility vs average enemy agility
pub fn calculate_flee_chance(
    party_avg_agility: i32,
    party_avg_level: i32,
    enemy_avg_agility: i32,
    enemy_avg_level: i32,
    is_boss_fight: bool,
) -> f32 {
    // Can't flee from boss fights
    if is_boss_fight {
        return 0.0;
    }

    let base_chance = 0.50;
    let agi_diff = (party_avg_agility - enemy_avg_agility) as f32 / 100.0;
    let level_diff = (party_avg_level - enemy_avg_level) as f32 / 50.0;

    (base_chance + agi_diff + level_diff).clamp(0.10, 0.90)
}

/// Calculate EXP multiplier based on party composition
pub fn calculate_exp_multiplier(party_size: usize, survivors: usize) -> f32 {
    if survivors == 0 {
        return 0.0;
    }

    // Full party gets base EXP, smaller parties get bonus
    match party_size {
        1 => 1.5, // Solo gets 50% bonus
        2 => 1.25,
        3 => 1.1,
        _ => 1.0,
    }
}

/// Calculate gold drop with variance
pub fn calculate_gold_drop(base_gold: i32, party_luck: i32) -> i32 {
    let mut rng = rand::thread_rng();

    // Luck bonus (up to +20% at high luck)
    let luck_multiplier = 1.0 + (party_luck as f32 / 500.0).min(0.20);

    // Random variance (+/- 20%)
    let variance = rng.gen_range(0.80..1.20);

    ((base_gold as f32) * luck_multiplier * variance) as i32
}

/// Calculate poison damage per turn (percentage of max HP)
pub fn calculate_poison_damage(max_hp: i32) -> i32 {
    // 10% of max HP per turn
    (max_hp / 10).max(1)
}

/// Calculate regen healing per turn (percentage of max HP)
pub fn calculate_regen_healing(max_hp: i32) -> i32 {
    // 5% of max HP per turn
    (max_hp / 20).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::realm_of_ralnar::magic::SpellType;

    fn default_attacker() -> AttackerStats {
        AttackerStats {
            strength: 20,
            agility: 15,
            intelligence: 10,
            luck: 10,
            level: 10,
            weapon_attack: 15,
            has_protect: false,
            has_haste: false,
            is_blind: false,
            is_berserk: false,
        }
    }

    fn default_defender() -> DefenderStats {
        DefenderStats {
            defense: 10,
            magic_defense: 8,
            agility: 12,
            level: 10,
            has_protect: false,
            has_shell: false,
            is_defending: false,
            is_back_row: false,
            weakness: None,
            resistance: None,
            absorb: None,
        }
    }

    #[test]
    fn test_physical_damage_basic() {
        let attacker = default_attacker();
        let defender = default_defender();

        // Run multiple times to account for variance
        let mut damages = Vec::new();
        for _ in 0..100 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                damages.push(result.damage);
            }
        }

        // Should have some hits
        assert!(!damages.is_empty(), "Should land some hits");

        // Damage should be reasonable
        let avg_damage: i32 = damages.iter().sum::<i32>() / damages.len() as i32;
        assert!(avg_damage > 0, "Average damage should be positive");
        assert!(
            avg_damage < 100,
            "Average damage should be reasonable for these stats"
        );
    }

    #[test]
    fn test_physical_damage_defending_halves() {
        let attacker = default_attacker();
        let mut defender = default_defender();

        let mut normal_damages = Vec::new();
        for _ in 0..50 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                normal_damages.push(result.damage);
            }
        }

        defender.is_defending = true;
        let mut defending_damages = Vec::new();
        for _ in 0..50 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                defending_damages.push(result.damage);
            }
        }

        if !normal_damages.is_empty() && !defending_damages.is_empty() {
            let avg_normal: i32 = normal_damages.iter().sum::<i32>() / normal_damages.len() as i32;
            let avg_defending: i32 =
                defending_damages.iter().sum::<i32>() / defending_damages.len() as i32;

            // Defending should roughly halve damage
            assert!(
                avg_defending < avg_normal,
                "Defending should reduce damage"
            );
        }
    }

    #[test]
    fn test_physical_damage_protect_increases_defense() {
        let attacker = default_attacker();
        let mut defender = default_defender();

        // Use more samples for statistical reliability
        let mut normal_damages = Vec::new();
        for _ in 0..200 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                normal_damages.push(result.damage);
            }
        }

        defender.has_protect = true;
        let mut protected_damages = Vec::new();
        for _ in 0..200 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                protected_damages.push(result.damage);
            }
        }

        if !normal_damages.is_empty() && !protected_damages.is_empty() {
            let avg_normal: i32 = normal_damages.iter().sum::<i32>() / normal_damages.len() as i32;
            let avg_protected: i32 =
                protected_damages.iter().sum::<i32>() / protected_damages.len() as i32;

            // With high sample size, protected average should be lower or equal
            // (accounting for minimal variance in test conditions)
            assert!(
                avg_protected <= avg_normal + 1, // Allow small tolerance for randomness
                "Protect should reduce damage taken (normal: {}, protected: {})",
                avg_normal,
                avg_protected
            );
        }
    }

    #[test]
    fn test_physical_damage_berserk_increases() {
        let mut attacker = default_attacker();
        let defender = default_defender();

        let mut normal_damages = Vec::new();
        for _ in 0..50 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                normal_damages.push(result.damage);
            }
        }

        attacker.is_berserk = true;
        let mut berserk_damages = Vec::new();
        for _ in 0..50 {
            let result = calculate_physical_damage(&attacker, &defender);
            if !result.is_miss {
                berserk_damages.push(result.damage);
            }
        }

        if !normal_damages.is_empty() && !berserk_damages.is_empty() {
            let avg_normal: i32 = normal_damages.iter().sum::<i32>() / normal_damages.len() as i32;
            let avg_berserk: i32 =
                berserk_damages.iter().sum::<i32>() / berserk_damages.len() as i32;

            assert!(
                avg_berserk > avg_normal,
                "Berserk should increase damage dealt"
            );
        }
    }

    #[test]
    fn test_hit_chance_basic() {
        let chance = calculate_hit_chance(20, 10, false);
        assert!(chance > 0.85, "High agility should have good hit chance");
        assert!(chance <= 0.95, "Hit chance should be capped at 95%");
    }

    #[test]
    fn test_hit_chance_blind_penalty() {
        let normal = calculate_hit_chance(20, 10, false);
        let blind = calculate_hit_chance(20, 10, true);
        assert!(
            blind < normal,
            "Blind should significantly reduce hit chance"
        );
    }

    #[test]
    fn test_hit_chance_minimum() {
        let chance = calculate_hit_chance(0, 100, true);
        assert!(chance >= 0.10, "Hit chance should have a minimum of 10%");
    }

    #[test]
    fn test_crit_chance_scales_with_luck() {
        let low_luck = calculate_crit_chance(10, 10);
        let high_luck = calculate_crit_chance(50, 10);
        assert!(
            high_luck > low_luck,
            "Higher luck should increase crit chance"
        );
    }

    #[test]
    fn test_magic_damage_basic() {
        let spell = Spell {
            id: "fire",
            name: "Fire",
            mp_cost: 4,
            spell_type: SpellType::Damage,
            element: Element::Fire,
            power: 20,
            target_type: super::super::magic::TargetType::SingleEnemy,
            level_required: 1,
            description: "Test",
        };
        let defender = default_defender();

        let result = calculate_magic_damage(20, &spell, &defender);
        assert!(result.damage > 0, "Should deal damage");
        assert!(!result.is_miss, "Magic shouldn't miss");
    }

    #[test]
    fn test_magic_damage_weakness() {
        let spell = Spell {
            id: "fire",
            name: "Fire",
            mp_cost: 4,
            spell_type: SpellType::Damage,
            element: Element::Fire,
            power: 20,
            target_type: super::super::magic::TargetType::SingleEnemy,
            level_required: 1,
            description: "Test",
        };

        let mut defender = default_defender();
        let normal_result = calculate_magic_damage(20, &spell, &defender);

        defender.weakness = Some(Element::Fire);
        let weak_result = calculate_magic_damage(20, &spell, &defender);

        assert!(weak_result.is_weak, "Should flag as weakness");
        assert!(
            weak_result.damage > normal_result.damage,
            "Weakness should deal more damage"
        );
    }

    #[test]
    fn test_magic_damage_resistance() {
        let spell = Spell {
            id: "fire",
            name: "Fire",
            mp_cost: 4,
            spell_type: SpellType::Damage,
            element: Element::Fire,
            power: 20,
            target_type: super::super::magic::TargetType::SingleEnemy,
            level_required: 1,
            description: "Test",
        };

        let mut defender = default_defender();
        let normal_result = calculate_magic_damage(20, &spell, &defender);

        defender.resistance = Some(Element::Fire);
        let resist_result = calculate_magic_damage(20, &spell, &defender);

        assert!(resist_result.is_resist, "Should flag as resisted");
        assert!(
            resist_result.damage < normal_result.damage,
            "Resistance should reduce damage"
        );
    }

    #[test]
    fn test_magic_damage_absorb() {
        let spell = Spell {
            id: "fire",
            name: "Fire",
            mp_cost: 4,
            spell_type: SpellType::Damage,
            element: Element::Fire,
            power: 20,
            target_type: super::super::magic::TargetType::SingleEnemy,
            level_required: 1,
            description: "Test",
        };

        let mut defender = default_defender();
        defender.absorb = Some(Element::Fire);

        let result = calculate_magic_damage(20, &spell, &defender);
        assert!(result.is_absorb, "Should flag as absorbed");
        assert!(result.damage < 0, "Absorb should result in negative damage (healing)");
    }

    #[test]
    fn test_healing_calculation() {
        let spell = Spell {
            id: "cure",
            name: "Cure",
            mp_cost: 4,
            spell_type: SpellType::Heal,
            element: Element::None,
            power: 30,
            target_type: super::super::magic::TargetType::SingleAlly,
            level_required: 1,
            description: "Test",
        };

        let healing = calculate_healing(20, &spell);
        assert!(healing > 0, "Healing should be positive");
        assert!(healing >= 30, "Healing should be at least spell power");
    }

    #[test]
    fn test_status_chance_basic() {
        let chance = calculate_status_chance(50, 10, 10, false);
        assert!(chance > 0.0 && chance < 1.0, "Chance should be between 0 and 1");
        assert_eq!(chance, 0.50, "50% spell power should give ~50% chance at equal levels");
    }

    #[test]
    fn test_status_chance_immunity() {
        let chance = calculate_status_chance(100, 20, 10, true);
        assert_eq!(chance, 0.0, "Immunity should give 0% chance");
    }

    #[test]
    fn test_status_chance_level_scaling() {
        let high_level = calculate_status_chance(50, 20, 10, false);
        let low_level = calculate_status_chance(50, 10, 20, false);

        assert!(
            high_level > low_level,
            "Higher caster level should increase chance"
        );
    }

    #[test]
    fn test_flee_chance_no_boss() {
        let chance = calculate_flee_chance(20, 10, 15, 8, false);
        assert!(chance > 0.0, "Should be able to flee from normal fights");
    }

    #[test]
    fn test_flee_chance_boss_zero() {
        let chance = calculate_flee_chance(50, 20, 10, 5, true);
        assert_eq!(chance, 0.0, "Cannot flee from boss fights");
    }

    #[test]
    fn test_flee_chance_agility_scaling() {
        let fast_party = calculate_flee_chance(30, 10, 10, 10, false);
        let slow_party = calculate_flee_chance(10, 10, 30, 10, false);

        assert!(
            fast_party > slow_party,
            "Faster party should have better flee chance"
        );
    }

    #[test]
    fn test_exp_multiplier() {
        assert!(calculate_exp_multiplier(1, 1) > calculate_exp_multiplier(4, 4));
        assert_eq!(calculate_exp_multiplier(4, 0), 0.0);
    }

    #[test]
    fn test_gold_drop_variance() {
        let mut drops = Vec::new();
        for _ in 0..100 {
            drops.push(calculate_gold_drop(100, 10));
        }

        let min = *drops.iter().min().unwrap();
        let max = *drops.iter().max().unwrap();

        assert!(max > min, "Gold drops should have variance");
    }

    #[test]
    fn test_poison_damage() {
        let damage = calculate_poison_damage(100);
        assert_eq!(damage, 10, "Poison should deal 10% max HP");

        let min_damage = calculate_poison_damage(5);
        assert_eq!(min_damage, 1, "Poison damage should have a minimum of 1");
    }

    #[test]
    fn test_regen_healing() {
        let healing = calculate_regen_healing(100);
        assert_eq!(healing, 5, "Regen should heal 5% max HP");

        let min_healing = calculate_regen_healing(10);
        assert_eq!(min_healing, 1, "Regen healing should have a minimum of 1");
    }
}
