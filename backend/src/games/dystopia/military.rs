//! Military operations - attacks, thief ops, and magic

use super::state::ProvinceState;
use super::data::{AttackType, ThiefOp, SpellType, get_race};
use rand::Rng;

/// Result of a military attack
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for attack result reporting
pub struct AttackResult {
    pub success: bool,
    pub message: String,
    pub land_captured: u32,
    pub gold_stolen: i64,
    pub buildings_destroyed: u32,
    pub peasants_killed: u32,
    pub our_losses: MilitaryLosses,
    pub enemy_losses: MilitaryLosses,
}

/// Military losses in combat
#[derive(Debug, Clone, Default)]
pub struct MilitaryLosses {
    pub soldiers: u32,
    pub archers: u32,
    pub knights: u32,
}

/// Result of a thief operation
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for thief operation reporting
pub struct ThiefOpResult {
    pub success: bool,
    pub message: String,
    pub thieves_lost: u32,
    pub gold_stolen: i64,
    pub buildings_destroyed: u32,
    pub peasants_kidnapped: u32,
}

/// Result of a spell cast
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields reserved for spell result reporting
pub struct SpellResult {
    pub success: bool,
    pub message: String,
    pub runes_spent: i64,
}

/// Calculate attack strength for an army
pub fn calculate_attack_strength(attacker: &ProvinceState, army_size_percent: u32) -> u32 {
    let percent = army_size_percent.min(100) as f64 / 100.0;

    let soldiers = (attacker.military.soldiers as f64 * percent) as u32;
    let archers = (attacker.military.archers as f64 * percent) as u32;
    let knights = (attacker.military.knights as f64 * percent) as u32;

    let mut strength = 0u32;
    strength += soldiers * 3;  // Soldier attack
    strength += archers * 4;   // Archer attack
    strength += knights * 7;   // Knight attack

    // Apply race bonus
    if let Some(race) = get_race(&attacker.race) {
        strength = (strength * race.attack_bonus) / 100;
    }

    // Apply military science
    strength = (strength * (100 + attacker.sciences.military)) / 100;

    strength
}

/// Execute an attack on another province (simulated)
/// In a real implementation, this would query the database for the target
pub fn execute_attack(
    attacker: &mut ProvinceState,
    _target_id: i64,
    attack_type: AttackType,
    army_size_percent: u32,
) -> AttackResult {
    let mut rng = rand::thread_rng();

    // Calculate our attack strength
    let our_strength = calculate_attack_strength(attacker, army_size_percent);

    // Simulate enemy defense (would be fetched from DB in reality)
    let enemy_defense: u32 = rng.gen_range(100..500);
    let attack_ratio = our_strength as f64 / enemy_defense as f64;

    // Determine success (>1.0 ratio needed to win)
    let success = attack_ratio > 1.0 && rng.gen_bool((attack_ratio - 1.0).min(0.9));

    // Calculate army being sent
    let percent = army_size_percent.min(100) as f64 / 100.0;
    let soldiers_sent = (attacker.military.soldiers as f64 * percent) as u32;
    let archers_sent = (attacker.military.archers as f64 * percent) as u32;
    let knights_sent = (attacker.military.knights as f64 * percent) as u32;

    // Calculate losses (always some losses in combat)
    let loss_percent = if success {
        rng.gen_range(5..15) as f64 / 100.0
    } else {
        rng.gen_range(15..40) as f64 / 100.0
    };

    let our_losses = MilitaryLosses {
        soldiers: (soldiers_sent as f64 * loss_percent) as u32,
        archers: (archers_sent as f64 * loss_percent) as u32,
        knights: (knights_sent as f64 * loss_percent) as u32,
    };

    // Apply losses to attacker
    attacker.military.soldiers = attacker.military.soldiers.saturating_sub(our_losses.soldiers);
    attacker.military.archers = attacker.military.archers.saturating_sub(our_losses.archers);
    attacker.military.knights = attacker.military.knights.saturating_sub(our_losses.knights);

    // Track deployed troops
    let remaining_soldiers = soldiers_sent - our_losses.soldiers;
    let remaining_archers = archers_sent - our_losses.archers;
    let remaining_knights = knights_sent - our_losses.knights;

    attacker.military.deployed_soldiers += remaining_soldiers;
    attacker.military.deployed_archers += remaining_archers;
    attacker.military.deployed_knights += remaining_knights;

    // Update stats
    attacker.stats.attacks_sent += 1;
    if success {
        attacker.stats.attacks_won += 1;
    }

    // Calculate gains based on attack type
    let (land_captured, gold_stolen, buildings_destroyed, peasants_killed) = if success {
        match attack_type {
            AttackType::TraditionalMarch => {
                let land = rng.gen_range(5..20);
                attacker.land += land;
                attacker.stats.land_captured += land;
                (land, 0, 0, 0)
            }
            AttackType::Raid => {
                let gold = rng.gen_range(1000..5000);
                attacker.resources.gold += gold;
                attacker.stats.gold_earned += gold;
                (0, gold, 0, 0)
            }
            AttackType::Plunder => {
                let buildings = rng.gen_range(1..5);
                (0, 0, buildings, 0)
            }
            AttackType::Massacre => {
                let peasants = rng.gen_range(50..200);
                (0, 0, 0, peasants)
            }
            AttackType::Learn => {
                // Would transfer science in real implementation
                (0, 0, 0, 0)
            }
        }
    } else {
        (0, 0, 0, 0)
    };

    let message = if success {
        format!(
            "Victory! Your forces triumphed against the enemy. {} {}.",
            match attack_type {
                AttackType::TraditionalMarch => format!("Captured {} acres", land_captured),
                AttackType::Raid => format!("Stole {} gold", gold_stolen),
                AttackType::Plunder => format!("Destroyed {} buildings", buildings_destroyed),
                AttackType::Massacre => format!("Killed {} peasants", peasants_killed),
                AttackType::Learn => "Stole enemy knowledge".to_string(),
            },
            format!("Lost {} soldiers, {} archers, {} knights.", our_losses.soldiers, our_losses.archers, our_losses.knights)
        )
    } else {
        format!(
            "Defeat! Your attack was repelled. Lost {} soldiers, {} archers, {} knights.",
            our_losses.soldiers, our_losses.archers, our_losses.knights
        )
    };

    AttackResult {
        success,
        message,
        land_captured,
        gold_stolen,
        buildings_destroyed,
        peasants_killed,
        our_losses,
        enemy_losses: MilitaryLosses::default(),
    }
}

/// Execute a thief operation
pub fn execute_thief_op(
    attacker: &mut ProvinceState,
    _target_id: i64,
    op_type: ThiefOp,
    thieves_sent: u32,
) -> ThiefOpResult {
    let mut rng = rand::thread_rng();

    if attacker.military.thieves < thieves_sent {
        return ThiefOpResult {
            success: false,
            message: "Not enough thieves available.".to_string(),
            thieves_lost: 0,
            gold_stolen: 0,
            buildings_destroyed: 0,
            peasants_kidnapped: 0,
        };
    }

    // Calculate success chance based on thieves and crime science
    let base_chance = 50 + (attacker.sciences.crime as i32);
    let bonus = get_race(&attacker.race)
        .map(|r| r.thief_bonus as i32 - 100)
        .unwrap_or(0);
    let success_chance = (base_chance + bonus).min(90).max(10) as u32;

    let success = rng.gen_range(0..100) < success_chance;

    // Calculate losses (caught thieves)
    let loss_percent = if success {
        rng.gen_range(5..15)
    } else {
        rng.gen_range(20..50)
    };
    let thieves_lost = (thieves_sent * loss_percent) / 100;
    attacker.military.thieves = attacker.military.thieves.saturating_sub(thieves_lost);

    attacker.stats.thief_ops += 1;

    let (gold_stolen, buildings_destroyed, peasants_kidnapped) = if success {
        match op_type {
            ThiefOp::StealGold => {
                let gold = rng.gen_range(500..3000);
                attacker.resources.gold += gold;
                attacker.stats.gold_earned += gold;
                (gold, 0, 0)
            }
            ThiefOp::Sabotage => (0, rng.gen_range(1..3), 0),
            ThiefOp::Kidnap => {
                let peasants = rng.gen_range(10..50);
                attacker.resources.prisoners += peasants;
                (0, 0, peasants)
            }
            ThiefOp::Assassinate => (0, 0, 0), // Would kill specialists
            ThiefOp::PropagandaWar => (0, 0, 0), // Would reduce morale
            ThiefOp::IntelGather => (0, 0, 0), // Would return province info
        }
    } else {
        (0, 0, 0)
    };

    let message = if success {
        format!(
            "Operation successful! {} thieves lost. {}",
            thieves_lost,
            match op_type {
                ThiefOp::StealGold => format!("Stole {} gold.", gold_stolen),
                ThiefOp::Sabotage => format!("Destroyed {} buildings.", buildings_destroyed),
                ThiefOp::Kidnap => format!("Kidnapped {} peasants.", peasants_kidnapped),
                ThiefOp::Assassinate => "Eliminated key specialists.".to_string(),
                ThiefOp::PropagandaWar => "Spread dissent among the populace.".to_string(),
                ThiefOp::IntelGather => "Gathered intelligence on the target.".to_string(),
            }
        )
    } else {
        format!(
            "Operation failed! {} thieves were caught and executed.",
            thieves_lost
        )
    };

    ThiefOpResult {
        success,
        message,
        thieves_lost,
        gold_stolen,
        buildings_destroyed,
        peasants_kidnapped,
    }
}

/// Cast a spell
pub fn cast_spell(
    caster: &mut ProvinceState,
    _target_id: Option<i64>,
    spell: SpellType,
) -> SpellResult {
    let mut rng = rand::thread_rng();

    let rune_cost = spell.rune_cost() as i64;

    if caster.resources.runes < rune_cost {
        return SpellResult {
            success: false,
            message: format!("Insufficient runes. Need {}, have {}.", rune_cost, caster.resources.runes),
            runes_spent: 0,
        };
    }

    // Calculate success chance based on wizards and channeling science
    let base_chance = 60 + (caster.sciences.channeling as i32);
    let wizard_bonus = (caster.military.wizards as i32).min(50);
    let race_bonus = get_race(&caster.race)
        .map(|r| r.magic_bonus as i32 - 100)
        .unwrap_or(0) / 2;
    let success_chance = (base_chance + wizard_bonus + race_bonus).min(95).max(20) as u32;

    let success = rng.gen_range(0..100) < success_chance;

    // Always spend runes
    caster.resources.runes -= rune_cost;
    caster.stats.spells_cast += 1;

    let message = if success {
        match spell {
            SpellType::Shield => {
                caster.effects.push(super::state::ActiveEffect {
                    effect_type: "shield".to_string(),
                    remaining_ticks: 24,
                    magnitude: 20,
                });
                "Shield spell cast! Defense increased by 20% for 24 ticks.".to_string()
            }
            SpellType::Prosperity => {
                caster.effects.push(super::state::ActiveEffect {
                    effect_type: "prosperity".to_string(),
                    remaining_ticks: 24,
                    magnitude: 15,
                });
                "Prosperity spell cast! Income increased by 15% for 24 ticks.".to_string()
            }
            SpellType::Haste => {
                caster.effects.push(super::state::ActiveEffect {
                    effect_type: "haste".to_string(),
                    remaining_ticks: 12,
                    magnitude: 25,
                });
                "Haste spell cast! Training speed increased by 25% for 12 ticks.".to_string()
            }
            SpellType::Heal => {
                // Restore some military units
                let healed = rng.gen_range(10..30);
                caster.military.soldiers += healed;
                format!("Heal spell cast! {} soldiers restored.", healed)
            }
            SpellType::Clairvoyance => "Clairvoyance cast! Target province revealed.".to_string(),
            SpellType::Fireball => format!("Fireball launched! Enemy troops incinerated."),
            SpellType::Lightning => format!("Lightning struck! Enemy buildings damaged."),
            SpellType::Plague => format!("Plague unleashed! Enemy peasants sickened."),
            SpellType::Drought => format!("Drought caused! Enemy food production halted."),
            SpellType::Barrier => {
                caster.effects.push(super::state::ActiveEffect {
                    effect_type: "barrier".to_string(),
                    remaining_ticks: 24,
                    magnitude: 50,
                });
                "Magic barrier raised! Spell resistance increased.".to_string()
            }
        }
    } else {
        "The spell fizzled! The arcane energies dissipated harmlessly.".to_string()
    };

    SpellResult {
        success,
        message,
        runes_spent: rune_cost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_attack_strength() {
        let state = ProvinceState::new(
            "Test".to_string(),
            "orc".to_string(), // Attack bonus
            "warrior".to_string(),
        );

        let strength = calculate_attack_strength(&state, 100);
        assert!(strength > 0);

        // 50% army should be roughly half strength
        let half_strength = calculate_attack_strength(&state, 50);
        assert!(half_strength < strength);
        assert!(half_strength > strength / 3); // Not exactly half due to rounding
    }

    #[test]
    fn test_attack_affects_state() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "warrior".to_string(),
        );

        let initial_soldiers = state.military.soldiers;
        let _result = execute_attack(&mut state, 1, AttackType::TraditionalMarch, 50);

        // Should have some losses
        assert!(state.military.soldiers <= initial_soldiers);
        assert_eq!(state.stats.attacks_sent, 1);
    }

    #[test]
    fn test_thief_op() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "faery".to_string(), // Thief bonus
            "rogue".to_string(),
        );

        let _result = execute_thief_op(&mut state, 1, ThiefOp::IntelGather, 5);
        assert_eq!(state.stats.thief_ops, 1);
    }

    #[test]
    fn test_spell_casting() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "elf".to_string(), // Magic bonus
            "mystic".to_string(),
        );

        let initial_runes = state.resources.runes;
        let result = cast_spell(&mut state, None, SpellType::Shield);

        assert!(state.resources.runes < initial_runes);
        assert!(result.runes_spent > 0);
    }

    #[test]
    fn test_insufficient_runes() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "warrior".to_string(),
        );
        state.resources.runes = 0;

        let result = cast_spell(&mut state, None, SpellType::Fireball);
        assert!(!result.success);
        assert_eq!(result.runes_spent, 0);
    }
}
