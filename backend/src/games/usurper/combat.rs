//! Combat system for Usurper
//!
//! Handles monster encounters, damage calculation, and combat resolution.

use super::data::{Monster, MONSTERS, CharacterClass};
use super::state::GameState;

/// Result of a combat round
pub struct CombatResult {
    pub damage_to_monster: u32,
    pub damage_to_player: u32,
    pub message: String,
}

/// Get a random monster appropriate for the dungeon level
pub fn get_random_monster(dungeon_level: u32) -> &'static Monster {
    // Filter monsters by level range
    let eligible: Vec<_> = MONSTERS.iter()
        .filter(|m| m.min_level <= dungeon_level + 5 && m.max_level >= dungeon_level.saturating_sub(3))
        .collect();

    if eligible.is_empty() {
        // Fallback to first monster
        return &MONSTERS[0];
    }

    // Weight towards appropriate level monsters
    let idx = rand::random::<usize>() % eligible.len();
    eligible[idx]
}

/// Calculate monster HP scaled for dungeon level
pub fn calculate_monster_hp(monster: &Monster, dungeon_level: u32) -> u32 {
    let level_bonus = dungeon_level.saturating_sub(monster.min_level);
    monster.base_hp + (level_bonus * 5)
}

/// Calculate monster damage scaled for dungeon level
pub fn calculate_monster_damage(monster: &Monster, dungeon_level: u32) -> u32 {
    let level_bonus = dungeon_level.saturating_sub(monster.min_level);
    monster.base_damage + (level_bonus * 2)
}

/// Resolve a combat attack round
pub fn resolve_combat_round(state: &GameState, monster: &Monster, monster_hp: u32) -> CombatResult {
    let stats = state.effective_stats();

    // Player attack
    let player_base_damage = stats.damage.max(1) as u32;
    let player_roll = rand::random::<u32>() % 10;
    let player_damage = player_base_damage + player_roll;

    // Critical hit chance based on agility
    let crit_chance = 5 + (stats.agility as u32 / 5);
    let is_crit = rand::random::<u32>() % 100 < crit_chance;
    let final_player_damage = if is_crit { player_damage * 2 } else { player_damage };

    // Monster attack
    let monster_damage = calculate_monster_damage(monster, state.current_dungeon_level);
    let monster_roll = rand::random::<u32>() % 6;
    let raw_monster_damage = monster_damage + monster_roll;

    // Apply defense
    let defense = stats.defense.max(0) as u32;
    let final_monster_damage = raw_monster_damage.saturating_sub(defense / 2);

    // Check for invincibility
    let actual_damage = if state.is_invincible() { 0 } else { final_monster_damage };

    let message = if is_crit {
        format!("CRITICAL HIT! You deal {} damage!", final_player_damage)
    } else {
        format!("You deal {} damage.", final_player_damage)
    };

    CombatResult {
        damage_to_monster: final_player_damage.min(monster_hp),
        damage_to_player: actual_damage,
        message,
    }
}

/// Resolve a defend action
pub fn resolve_defend(state: &GameState, monster: &Monster, _monster_hp: u32) -> CombatResult {
    let stats = state.effective_stats();

    // Reduced monster damage when defending
    let monster_damage = calculate_monster_damage(monster, state.current_dungeon_level);
    let reduced_damage = monster_damage / 2;

    // Counter attack (small damage)
    let counter_damage = (stats.strength.max(0) as u32) / 3;

    CombatResult {
        damage_to_monster: counter_damage,
        damage_to_player: if state.is_invincible() { 0 } else { reduced_damage },
        message: "You brace for impact and counter-strike!".to_string(),
    }
}

/// Resolve a class skill
pub fn resolve_skill(state: &GameState, monster: &Monster, monster_hp: u32) -> CombatResult {
    let stats = state.effective_stats();
    let class: CharacterClass = state.class.into();

    let (damage_mult, message) = match class {
        CharacterClass::Warrior => {
            // Power Strike - high damage
            (2.0, "You unleash a devastating Power Strike!")
        }
        CharacterClass::Rogue => {
            // Backstab - guaranteed crit if successful
            let success = rand::random::<u32>() % 100 < 60 + stats.agility as u32;
            if success {
                (3.0, "Your Backstab finds its mark!")
            } else {
                (0.5, "Your Backstab misses!")
            }
        }
        CharacterClass::Mage => {
            // Fireball - INT-based damage
            let int_bonus = stats.intelligence as f32 / 10.0;
            (1.5 + int_bonus, "You hurl a blazing Fireball!")
        }
        CharacterClass::Cleric => {
            // Holy Smite - moderate damage, some healing
            // Note: healing handled separately
            (1.5, "Divine light burns the enemy!")
        }
        CharacterClass::Berserker => {
            // Rage - massive damage but takes damage too
            (3.5, "RAAAAGE! You strike in a frenzy!")
        }
    };

    let base_damage = stats.damage.max(1) as f32;
    let skill_damage = (base_damage * damage_mult) as u32;

    // Monster counter-attack
    let monster_damage = calculate_monster_damage(monster, state.current_dungeon_level);
    let self_damage = if class == CharacterClass::Berserker {
        monster_damage + 5 // Berserkers take extra damage from rage
    } else if class == CharacterClass::Cleric {
        monster_damage.saturating_sub(10) // Clerics take less
    } else {
        monster_damage
    };

    CombatResult {
        damage_to_monster: skill_damage.min(monster_hp),
        damage_to_player: if state.is_invincible() { 0 } else { self_damage },
        message: message.to_string(),
    }
}

/// Calculate XP reward for defeating a monster
pub fn calculate_xp_reward(monster: &Monster, player_level: u32) -> u64 {
    let base_xp = monster.xp_reward as u64;

    // Bonus for fighting higher level monsters
    let level_diff = (monster.min_level as i32) - (player_level as i32);
    let multiplier = if level_diff > 5 {
        1.5
    } else if level_diff > 0 {
        1.2
    } else if level_diff < -5 {
        0.5 // Reduced XP for much lower level monsters
    } else {
        1.0
    };

    (base_xp as f64 * multiplier) as u64
}

/// Calculate gold reward for defeating a monster
pub fn calculate_gold_reward(monster: &Monster) -> u64 {
    let (min, max) = monster.gold_drop;
    let range = max - min;
    (min + rand::random::<u32>() % range.max(1)) as u64
}

/// Calculate XP penalty for PvP kill (attacker kills much lower level)
pub fn calculate_pvp_xp_penalty(attacker_level: u32, victim_level: u32) -> u64 {
    if attacker_level > victim_level + 10 {
        // Severe penalty for killing much lower level players
        let level_diff = attacker_level - victim_level;
        (level_diff as u64) * 50
    } else if attacker_level > victim_level + 5 {
        // Moderate penalty
        100
    } else {
        0 // No penalty for fair fights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_selection() {
        let monster = get_random_monster(5);
        assert!(monster.min_level <= 10);
    }

    #[test]
    fn test_monster_hp_scaling() {
        let monster = &MONSTERS[0];
        let hp_low = calculate_monster_hp(monster, 1);
        let hp_high = calculate_monster_hp(monster, 10);
        assert!(hp_high >= hp_low);
    }

    #[test]
    fn test_xp_reward() {
        let monster = &MONSTERS[5]; // goblin
        let xp = calculate_xp_reward(monster, 5);
        assert!(xp > 0);
    }

    #[test]
    fn test_pvp_penalty() {
        let penalty = calculate_pvp_xp_penalty(50, 10);
        assert!(penalty > 0);

        let no_penalty = calculate_pvp_xp_penalty(10, 10);
        assert_eq!(no_penalty, 0);
    }

    #[test]
    fn test_combat_round() {
        let state = GameState::new("Test".to_string(), CharacterClass::Warrior);
        let monster = &MONSTERS[0];
        let result = resolve_combat_round(&state, monster, 20);
        assert!(result.damage_to_monster > 0);
    }
}
