//! Combat system for Dragon Slayer
//! Handles monster fights, master challenges, PvP, and dragon battles

use rand::prelude::*;
use super::state::GameState;
use super::data::{Monster, Master};

/// Current combat state
#[derive(Debug, Clone)]
pub struct CombatState {
    pub enemy_type: EnemyType,
    pub enemy_name: String,
    pub enemy_hp: u32,
    pub enemy_max_hp: u32,
    pub enemy_strength: u32,
    pub enemy_defense: u32,
    pub gold_reward: i64,
    pub xp_reward: i64,
    /// Player is dodging next attack
    pub player_dodging: bool,
    /// Combat log for display
    pub combat_log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyType {
    Monster { key: String },
    Master { level: u8 },
    #[allow(dead_code)]
    Player { user_id: i64, handle: String },
    RedDragon,
}

#[derive(Debug, Clone)]
pub enum CombatAction {
    Attack,
    Run,
    UseSkill { skill_key: String },
    #[allow(dead_code)]
    UseItem { item_key: String },
}

#[derive(Debug, Clone)]
pub enum CombatResult {
    /// Combat continues
    Continue,
    /// Player won the fight
    Victory {
        xp_gained: i64,
        gold_gained: i64,
        message: String,
    },
    /// Player defeated (not necessarily dead - fairy might save)
    Defeat {
        message: String,
    },
    /// Player fled successfully
    Fled {
        message: String,
    },
    /// Player failed to flee
    FledFailed {
        #[allow(dead_code)]
        damage_taken: u32,
        message: String,
    },
    /// Dragon slain - special ending
    DragonSlain {
        message: String,
    },
    /// Master defeated - level up available
    MasterDefeated {
        #[allow(dead_code)]
        level: u8,
        message: String,
    },
}

impl CombatState {
    /// Create combat state for a monster encounter
    pub fn from_monster(monster: &Monster, player_level: u8) -> Self {
        let mut rng = thread_rng();

        // Scale monster HP based on player level difference
        let level_scale = ((player_level as i32 - monster.level_min as i32).max(0) as u32) / 2;
        let hp = monster.hp_base + monster.hp_per_level * level_scale + rng.gen_range(0..10);

        // Calculate rewards
        let gold = rng.gen_range(monster.gold_min..=monster.gold_max);
        let xp = monster.xp_base + (level_scale as i64 * 5);

        Self {
            enemy_type: EnemyType::Monster { key: monster.key.to_string() },
            enemy_name: monster.name.to_string(),
            enemy_hp: hp,
            enemy_max_hp: hp,
            enemy_strength: monster.strength,
            enemy_defense: monster.defense,
            gold_reward: gold,
            xp_reward: xp,
            player_dodging: false,
            combat_log: vec![format!("A {} attacks you!", monster.name)],
        }
    }

    /// Create combat state for a master challenge
    pub fn from_master(master: &Master) -> Self {
        Self {
            enemy_type: EnemyType::Master { level: master.level },
            enemy_name: master.name.to_string(),
            enemy_hp: master.hp,
            enemy_max_hp: master.hp,
            enemy_strength: master.strength,
            enemy_defense: master.defense,
            gold_reward: 0,
            xp_reward: 0,
            player_dodging: false,
            combat_log: vec![format!("{} stands ready!", master.name)],
        }
    }

    /// Create combat state for the Red Dragon
    pub fn red_dragon() -> Self {
        let (hp, str, def) = super::data::get_red_dragon_stats();
        Self {
            enemy_type: EnemyType::RedDragon,
            enemy_name: "THE RED DRAGON".to_string(),
            enemy_hp: hp,
            enemy_max_hp: hp,
            enemy_strength: str,
            enemy_defense: def,
            gold_reward: 100_000,
            xp_reward: 0, // Dragon slaying resets character
            player_dodging: false,
            combat_log: vec![
                "The ground shakes as the Red Dragon appears!".to_string(),
                "Its scales gleam like blood. Children's screams echo in the distance.".to_string(),
            ],
        }
    }
}

/// Process a combat action and return the result
pub fn process_combat(
    state: &mut GameState,
    combat: &mut CombatState,
    action: CombatAction,
) -> CombatResult {
    let mut rng = thread_rng();

    match action {
        CombatAction::Attack => {
            // Player attacks
            let player_attack = state.attack_power();
            let damage = calculate_damage(player_attack, combat.enemy_defense, &mut rng);
            let is_critical = rng.gen_range(0..100) < 10;
            let final_damage = if is_critical { damage * 2 } else { damage };

            combat.enemy_hp = combat.enemy_hp.saturating_sub(final_damage);

            if is_critical {
                combat.combat_log.push(format!(
                    "CRITICAL HIT! You deal {} damage to {}!",
                    final_damage, combat.enemy_name
                ));
            } else {
                combat.combat_log.push(format!(
                    "You deal {} damage to {}.",
                    final_damage, combat.enemy_name
                ));
            }

            // Check if enemy is dead
            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }

            // Enemy counterattacks
            enemy_turn(state, combat, &mut rng)
        }

        CombatAction::Run => {
            // Can't run from masters or dragon
            if matches!(combat.enemy_type, EnemyType::Master { .. } | EnemyType::RedDragon) {
                combat.combat_log.push("There is no escape from this battle!".to_string());
                return enemy_turn(state, combat, &mut rng);
            }

            // 60% base flee chance, modified by level difference
            let flee_chance = 60 + (state.level as i32 * 2);
            if rng.gen_range(0..100) < flee_chance {
                CombatResult::Fled {
                    message: "You flee into the forest!".to_string(),
                }
            } else {
                combat.combat_log.push("You fail to escape!".to_string());
                let result = enemy_turn(state, combat, &mut rng);
                if matches!(result, CombatResult::Continue) {
                    CombatResult::FledFailed {
                        damage_taken: 0, // Damage applied in enemy_turn
                        message: "Couldn't get away!".to_string(),
                    }
                } else {
                    result
                }
            }
        }

        CombatAction::UseSkill { skill_key } => {
            process_skill(state, combat, &skill_key, &mut rng)
        }

        CombatAction::UseItem { item_key: _ } => {
            // TODO: Implement item usage
            combat.combat_log.push("You fumble with your pack.".to_string());
            enemy_turn(state, combat, &mut rng)
        }
    }
}

/// Calculate damage with randomness
fn calculate_damage(attack: u32, defense: u32, rng: &mut ThreadRng) -> u32 {
    let base = attack.saturating_sub(defense / 2);
    let variance = rng.gen_range(0..=base / 4 + 1);
    (base + variance).max(1)
}

/// Enemy takes their turn
fn enemy_turn(state: &mut GameState, combat: &mut CombatState, rng: &mut ThreadRng) -> CombatResult {
    // Check if player is dodging
    if combat.player_dodging {
        combat.player_dodging = false;
        combat.combat_log.push(format!("{} attacks but you dodge!", combat.enemy_name));
        return CombatResult::Continue;
    }

    // Enemy attacks
    let player_defense = state.defense_power();
    let damage = calculate_damage(combat.enemy_strength, player_defense, rng);
    let is_critical = rng.gen_range(0..100) < 5;
    let final_damage = if is_critical { damage * 2 } else { damage };

    state.hp_current = state.hp_current.saturating_sub(final_damage);

    if is_critical {
        combat.combat_log.push(format!(
            "CRITICAL! {} deals {} damage to you!",
            combat.enemy_name, final_damage
        ));
    } else {
        combat.combat_log.push(format!(
            "{} deals {} damage to you.",
            combat.enemy_name, final_damage
        ));
    }

    // Check if player is defeated
    if state.hp_current == 0 {
        state.die();
        CombatResult::Defeat {
            message: format!("You have been slain by {}!", combat.enemy_name),
        }
    } else {
        CombatResult::Continue
    }
}

/// Handle victory
fn handle_victory(state: &mut GameState, combat: &CombatState) -> CombatResult {
    match &combat.enemy_type {
        EnemyType::Monster { .. } => {
            let xp = combat.xp_reward;
            let gold = combat.gold_reward;

            state.experience += xp;
            state.gold_pocket += gold;
            state.kills += 1;

            CombatResult::Victory {
                xp_gained: xp,
                gold_gained: gold,
                message: format!("You defeated {}!", combat.enemy_name),
            }
        }

        EnemyType::Master { level } => {
            CombatResult::MasterDefeated {
                level: *level,
                message: format!("{} acknowledges your skill!", combat.enemy_name),
            }
        }

        EnemyType::Player { handle, .. } => {
            let xp = 500 * (state.level as i64);
            state.experience += xp;
            state.kills += 1;

            CombatResult::Victory {
                xp_gained: xp,
                gold_gained: 0,
                message: format!("You defeated {}!", handle),
            }
        }

        EnemyType::RedDragon => {
            state.dragon_kills += 1;
            CombatResult::DragonSlain {
                message: "THE RED DRAGON HAS BEEN SLAIN! The children are saved!".to_string(),
            }
        }
    }
}

/// Process skill usage in combat
fn process_skill(
    state: &mut GameState,
    combat: &mut CombatState,
    skill_key: &str,
    rng: &mut ThreadRng,
) -> CombatResult {
    // Check if player has and can use the skill
    if !state.has_skill(skill_key) {
        combat.combat_log.push("You don't know that skill!".to_string());
        return enemy_turn(state, combat, rng);
    }

    if !state.can_use_skill(skill_key) {
        combat.combat_log.push("You've used that skill too many times today!".to_string());
        return enemy_turn(state, combat, rng);
    }

    state.use_skill(skill_key);

    match skill_key {
        "power_strike" => {
            let damage = state.attack_power() * 2;
            let final_damage = calculate_damage(damage, combat.enemy_defense, rng);
            combat.enemy_hp = combat.enemy_hp.saturating_sub(final_damage);
            combat.combat_log.push(format!(
                "POWER STRIKE! You deal {} damage!",
                final_damage
            ));

            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }
            enemy_turn(state, combat, rng)
        }

        "death_wish" => {
            // Sacrifice 20% HP for 3x damage
            let hp_cost = state.hp_max / 5;
            state.hp_current = state.hp_current.saturating_sub(hp_cost);

            if state.hp_current == 0 {
                state.die();
                return CombatResult::Defeat {
                    message: "Your death wish came true... literally.".to_string(),
                };
            }

            let damage = state.attack_power() * 3;
            let final_damage = calculate_damage(damage, combat.enemy_defense, rng);
            combat.enemy_hp = combat.enemy_hp.saturating_sub(final_damage);
            combat.combat_log.push(format!(
                "DEATH WISH! You sacrifice {} HP and deal {} damage!",
                hp_cost, final_damage
            ));

            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }
            enemy_turn(state, combat, rng)
        }

        "assault" => {
            // 5x damage ultimate
            let damage = state.attack_power() * 5;
            let final_damage = calculate_damage(damage, combat.enemy_defense, rng);
            combat.enemy_hp = combat.enemy_hp.saturating_sub(final_damage);
            combat.combat_log.push(format!(
                "ASSAULT! Devastating blow deals {} damage!",
                final_damage
            ));

            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }
            enemy_turn(state, combat, rng)
        }

        "fireball" => {
            // Magic damage ignores some defense
            let base_damage = 20 + (state.level as u32 * 5);
            let damage = base_damage + rng.gen_range(0..10);
            combat.enemy_hp = combat.enemy_hp.saturating_sub(damage);
            combat.combat_log.push(format!(
                "FIREBALL! Flames engulf {} for {} damage!",
                combat.enemy_name, damage
            ));

            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }
            enemy_turn(state, combat, rng)
        }

        "heal" => {
            let heal = state.hp_max * 30 / 100;
            state.hp_current = (state.hp_current + heal).min(state.hp_max);
            combat.combat_log.push(format!(
                "You heal for {} HP!",
                heal
            ));
            enemy_turn(state, combat, rng)
        }

        "lightning" => {
            // Strong magic attack
            let base_damage = 40 + (state.level as u32 * 8);
            let damage = base_damage + rng.gen_range(0..15);
            combat.enemy_hp = combat.enemy_hp.saturating_sub(damage);
            combat.combat_log.push(format!(
                "LIGHTNING BOLT! {} takes {} damage!",
                combat.enemy_name, damage
            ));

            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }
            enemy_turn(state, combat, rng)
        }

        "pick_pocket" => {
            // Steal gold from monster
            if matches!(combat.enemy_type, EnemyType::Monster { .. }) {
                let stolen = rng.gen_range(10..50 * state.level as i64);
                state.gold_pocket += stolen;
                combat.combat_log.push(format!(
                    "You pick {}'s pocket for {} gold!",
                    combat.enemy_name, stolen
                ));
            } else {
                combat.combat_log.push("There's nothing to steal!".to_string());
            }
            enemy_turn(state, combat, rng)
        }

        "sneak_attack" => {
            // 2.5x damage
            let damage = (state.attack_power() * 5) / 2;
            let final_damage = calculate_damage(damage, combat.enemy_defense, rng);
            combat.enemy_hp = combat.enemy_hp.saturating_sub(final_damage);
            combat.combat_log.push(format!(
                "SNEAK ATTACK! You strike from the shadows for {} damage!",
                final_damage
            ));

            if combat.enemy_hp == 0 {
                return handle_victory(state, combat);
            }
            enemy_turn(state, combat, rng)
        }

        "dodge" => {
            combat.player_dodging = true;
            combat.combat_log.push("You prepare to dodge the next attack!".to_string());
            enemy_turn(state, combat, rng)
        }

        "fairy_catch" => {
            // Attempt to catch a fairy (out of combat effect)
            if rng.gen_range(0..100) < 20 {
                state.has_fairy = true;
                state.fairy_uses = 1;
                combat.combat_log.push("A fairy appears! You catch it carefully.".to_string());
            } else {
                combat.combat_log.push("You search but find no fairies...".to_string());
            }
            enemy_turn(state, combat, rng)
        }

        _ => {
            combat.combat_log.push("Unknown skill!".to_string());
            enemy_turn(state, combat, rng)
        }
    }
}

/// Calculate XP penalty for killing much lower level players
#[allow(dead_code)]
pub fn pvp_xp_penalty(attacker_level: u8, defender_level: u8) -> f32 {
    let level_diff = attacker_level.saturating_sub(defender_level);
    match level_diff {
        0..=2 => 1.0,      // No penalty
        3..=4 => 0.5,      // 50% reduction
        5..=6 => 0.25,     // 75% reduction
        _ => 0.1,          // 90% reduction (griefing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::dragon_slayer::state::Sex;

    #[test]
    fn test_combat_state_from_monster() {
        let monster = super::super::data::get_monster("slime").unwrap();
        let combat = CombatState::from_monster(monster, 1);

        assert_eq!(combat.enemy_name, "Small Slime");
        assert!(combat.enemy_hp > 0);
        assert!(combat.gold_reward >= 1);
    }

    #[test]
    fn test_damage_calculation() {
        let mut rng = thread_rng();

        let damage = calculate_damage(100, 20, &mut rng);
        assert!(damage > 0);
        // With attack=100, defense=20: base = 90, plus variance up to ~23
        // So max damage is approximately base + base/4 = ~113
        assert!(damage <= 150); // Reasonable upper bound
    }

    #[test]
    fn test_basic_combat() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        state.stats.strength = 50;
        state.equipment.weapon = "dagger".to_string();

        let monster = super::super::data::get_monster("slime").unwrap();
        let mut combat = CombatState::from_monster(monster, 1);

        // Attack until victory or defeat
        for _ in 0..100 {
            let result = process_combat(&mut state, &mut combat, CombatAction::Attack);
            match result {
                CombatResult::Victory { .. } => {
                    assert!(combat.enemy_hp == 0);
                    return;
                }
                CombatResult::Defeat { .. } => {
                    assert!(state.hp_current == 0 || state.is_dead);
                    return;
                }
                CombatResult::Continue => continue,
                _ => panic!("Unexpected result"),
            }
        }
    }

    #[test]
    fn test_pvp_penalty() {
        assert_eq!(pvp_xp_penalty(5, 5), 1.0);
        assert_eq!(pvp_xp_penalty(10, 5), 0.25);
        assert_eq!(pvp_xp_penalty(12, 2), 0.1);
    }

    #[test]
    fn test_flee() {
        let mut state = GameState::new("Hero".to_string(), Sex::Male);
        let monster = super::super::data::get_monster("slime").unwrap();
        let mut combat = CombatState::from_monster(monster, 1);

        // Try fleeing multiple times (random chance)
        for _ in 0..10 {
            let result = process_combat(&mut state, &mut combat, CombatAction::Run);
            match result {
                CombatResult::Fled { .. } => return, // Success
                CombatResult::FledFailed { .. } | CombatResult::Continue => {
                    // Reset combat for next attempt
                    combat = CombatState::from_monster(monster, 1);
                }
                CombatResult::Defeat { .. } => return, // Got killed while fleeing
                _ => {}
            }
        }
    }
}
