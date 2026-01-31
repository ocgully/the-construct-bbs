//! Combat system for Kyrandia
//! Handles monster encounters and PvP duels

#![allow(dead_code)]

use rand::Rng;
use super::data::{Monster, get_monster, get_monsters_for_region, Region};
use super::state::{GameState, CombatState};

/// Result of a combat action
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub message: String,
    pub player_damage: u32,
    pub monster_damage: u32,
    pub combat_ended: bool,
    pub victory: bool,
    pub xp_gained: u64,
    pub gold_gained: i64,
    pub items_dropped: Vec<String>,
}

/// Combat action types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatAction {
    Attack,
    CastSpell,
    UseItem,
    Flee,
}

/// Start a combat encounter with a random monster from the region
pub fn start_random_encounter(state: &mut GameState, region: Region) -> Option<String> {
    let monsters = get_monsters_for_region(region);
    if monsters.is_empty() {
        return None;
    }

    let mut rng = rand::thread_rng();
    let monster = monsters[rng.gen_range(0..monsters.len())];

    state.combat = Some(CombatState {
        monster_key: monster.key.to_string(),
        monster_hp: monster.hp,
        monster_max_hp: monster.hp,
        player_turn: true,
        shield_active: state.active_effects.contains_key("shield"),
        shield_power: state.active_effects.get("shield").map(|_| 20).unwrap_or(0),
    });

    Some(format!(
        "A {} appears before you!\n\n\
         {}\n\n\
         HP: {}/{}",
        monster.name, monster.description, monster.hp, monster.hp
    ))
}

/// Start combat with a specific monster
pub fn start_combat_with(state: &mut GameState, monster_key: &str) -> Option<String> {
    let monster = get_monster(monster_key)?;

    state.combat = Some(CombatState {
        monster_key: monster.key.to_string(),
        monster_hp: monster.hp,
        monster_max_hp: monster.hp,
        player_turn: true,
        shield_active: state.active_effects.contains_key("shield"),
        shield_power: state.active_effects.get("shield").map(|_| 20).unwrap_or(0),
    });

    Some(format!(
        "A {} appears before you!\n\n\
         {}\n\n\
         HP: {}/{}",
        monster.name, monster.description, monster.hp, monster.hp
    ))
}

/// Process player attack
pub fn player_attack(state: &mut GameState) -> CombatResult {
    // Get immutable values before mutable borrow
    let player_defense = state.defense();
    let player_attack = state.attack_power();

    let combat = match &mut state.combat {
        Some(c) => c,
        None => return CombatResult {
            message: "You're not in combat!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        },
    };

    let monster = match get_monster(&combat.monster_key) {
        Some(m) => m,
        None => return CombatResult {
            message: "The monster has vanished!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        },
    };

    let mut rng = rand::thread_rng();
    let mut message = String::new();

    // Player attack (use pre-calculated player_attack)
    let player_roll = rng.gen_range(0..player_attack);
    let monster_defense = monster.defense;
    let player_damage_dealt = if player_roll > monster_defense {
        player_roll - monster_defense
    } else {
        1  // Minimum damage
    };

    message.push_str(&format!(
        "You attack the {}! You deal {} damage.\n",
        monster.name, player_damage_dealt
    ));

    // Apply damage to monster
    let combat = state.combat.as_mut().unwrap();
    if player_damage_dealt >= combat.monster_hp {
        combat.monster_hp = 0;
        // Victory!
        let items_dropped = calculate_drops(monster);
        let xp = monster.xp_reward;
        let gold = monster.gold_reward;

        // Add rewards
        state.add_xp(xp);
        state.add_gold(gold);
        state.monsters_killed += 1;

        // Add dropped items
        for item in &items_dropped {
            state.add_item(item, 1);
        }

        // End combat
        state.combat = None;

        message.push_str(&format!(
            "\nThe {} falls defeated!\n\
             You gain {} XP and {} gold!",
            monster.name, xp, gold
        ));

        if !items_dropped.is_empty() {
            message.push_str(&format!("\nDropped: {}", items_dropped.join(", ")));
        }

        return CombatResult {
            message,
            player_damage: 0,
            monster_damage: player_damage_dealt,
            combat_ended: true,
            victory: true,
            xp_gained: xp,
            gold_gained: gold,
            items_dropped,
        };
    } else {
        combat.monster_hp -= player_damage_dealt;
    }

    message.push_str(&format!(
        "{} HP: {}/{}\n\n",
        monster.name, combat.monster_hp, combat.monster_max_hp
    ));

    // Monster counterattack (use pre-calculated player_defense)
    let monster_roll = rng.gen_range(0..monster.attack);

    // Apply shield if active
    let shield_reduction = if combat.shield_active {
        combat.shield_power
    } else {
        0
    };

    let damage_to_player = if monster_roll > player_defense + shield_reduction {
        monster_roll - player_defense - shield_reduction
    } else {
        0
    };

    if damage_to_player > 0 {
        message.push_str(&format!(
            "The {} attacks you for {} damage!",
            monster.name, damage_to_player
        ));

        let died = state.damage(damage_to_player);
        if died {
            state.combat = None;
            message.push_str("\n\nYou have been defeated!");

            return CombatResult {
                message,
                player_damage: damage_to_player,
                monster_damage: player_damage_dealt,
                combat_ended: true,
                victory: false,
                xp_gained: 0,
                gold_gained: 0,
                items_dropped: vec![],
            };
        }
    } else {
        message.push_str(&format!(
            "The {} attacks but you block it!",
            monster.name
        ));
    }

    CombatResult {
        message,
        player_damage: damage_to_player,
        monster_damage: player_damage_dealt,
        combat_ended: false,
        victory: false,
        xp_gained: 0,
        gold_gained: 0,
        items_dropped: vec![],
    }
}

/// Apply spell damage to monster in combat
pub fn apply_spell_damage(state: &mut GameState, damage: u32) -> CombatResult {
    // Get player defense before mutable borrow
    let player_defense = state.defense();

    let combat = match &mut state.combat {
        Some(c) => c,
        None => return CombatResult {
            message: "You're not in combat!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        },
    };

    let monster = match get_monster(&combat.monster_key) {
        Some(m) => m,
        None => return CombatResult {
            message: "The monster has vanished!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        },
    };

    // Apply spell damage (ignores defense for spells)
    if damage >= combat.monster_hp {
        combat.monster_hp = 0;

        let items_dropped = calculate_drops(monster);
        let xp = monster.xp_reward;
        let gold = monster.gold_reward;

        state.add_xp(xp);
        state.add_gold(gold);
        state.monsters_killed += 1;

        for item in &items_dropped {
            state.add_item(item, 1);
        }

        state.combat = None;

        let message = format!(
            "Your spell engulfs the {}!\n\
             The {} falls defeated!\n\
             You gain {} XP and {} gold!",
            monster.name, monster.name, xp, gold
        );

        return CombatResult {
            message,
            player_damage: 0,
            monster_damage: damage,
            combat_ended: true,
            victory: true,
            xp_gained: xp,
            gold_gained: gold,
            items_dropped,
        };
    }

    combat.monster_hp -= damage;

    // Monster counterattack (use pre-calculated player_defense)
    let mut rng = rand::thread_rng();
    let monster_roll = rng.gen_range(0..monster.attack);

    let shield_reduction = if combat.shield_active {
        combat.shield_power
    } else {
        0
    };

    let damage_to_player = if monster_roll > player_defense + shield_reduction {
        monster_roll - player_defense - shield_reduction
    } else {
        0
    };

    let mut message = format!(
        "Your spell hits the {} for {} damage!\n\
         {} HP: {}/{}\n\n",
        monster.name, damage, monster.name, combat.monster_hp, combat.monster_max_hp
    );

    if damage_to_player > 0 {
        message.push_str(&format!(
            "The {} counterattacks for {} damage!",
            monster.name, damage_to_player
        ));

        let died = state.damage(damage_to_player);
        if died {
            state.combat = None;
            message.push_str("\n\nYou have been defeated!");

            return CombatResult {
                message,
                player_damage: damage_to_player,
                monster_damage: damage,
                combat_ended: true,
                victory: false,
                xp_gained: 0,
                gold_gained: 0,
                items_dropped: vec![],
            };
        }
    } else {
        message.push_str(&format!("The {} attacks but you dodge!", monster.name));
    }

    CombatResult {
        message,
        player_damage: damage_to_player,
        monster_damage: damage,
        combat_ended: false,
        victory: false,
        xp_gained: 0,
        gold_gained: 0,
        items_dropped: vec![],
    }
}

/// Attempt to flee from combat
pub fn attempt_flee(state: &mut GameState) -> CombatResult {
    let combat = match &state.combat {
        Some(c) => c,
        None => return CombatResult {
            message: "You're not in combat!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        },
    };

    let monster = match get_monster(&combat.monster_key) {
        Some(m) => m,
        None => return CombatResult {
            message: "The monster has vanished!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        },
    };

    let mut rng = rand::thread_rng();

    // Flee chance based on player level vs monster
    let flee_chance = 40 + (state.level as i32 * 5);
    let roll = rng.gen_range(0..100);

    if roll < flee_chance {
        state.combat = None;
        CombatResult {
            message: "You flee from battle!".to_string(),
            player_damage: 0,
            monster_damage: 0,
            combat_ended: true,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        }
    } else {
        // Failed to flee, monster gets a free attack
        let monster_roll = rng.gen_range(0..monster.attack);
        let player_defense = state.defense();
        let damage = if monster_roll > player_defense {
            monster_roll - player_defense
        } else {
            1
        };

        let message = format!(
            "You try to flee but the {} blocks your escape!\n\
             It attacks you for {} damage!",
            monster.name, damage
        );

        let died = state.damage(damage);
        if died {
            state.combat = None;
            return CombatResult {
                message: format!("{}\n\nYou have been defeated!", message),
                player_damage: damage,
                monster_damage: 0,
                combat_ended: true,
                victory: false,
                xp_gained: 0,
                gold_gained: 0,
                items_dropped: vec![],
            };
        }

        CombatResult {
            message,
            player_damage: damage,
            monster_damage: 0,
            combat_ended: false,
            victory: false,
            xp_gained: 0,
            gold_gained: 0,
            items_dropped: vec![],
        }
    }
}

/// Calculate item drops from defeated monster
fn calculate_drops(monster: &Monster) -> Vec<String> {
    let mut drops = Vec::new();
    let mut rng = rand::thread_rng();

    for (item_key, chance) in monster.drops {
        let roll = rng.gen_range(0..100);
        if roll < *chance as i32 {
            drops.push(item_key.to_string());
        }
    }

    drops
}

/// Check if there should be a random encounter
pub fn check_random_encounter(region: Region, level: u8) -> bool {
    let mut rng = rand::thread_rng();

    // Encounter rate based on region
    let base_rate = match region {
        Region::Village => 5,      // Low rate in village
        Region::DarkForest => 25,  // Higher in dangerous areas
        Region::GoldenForest => 20,
        Region::DragonCastle => 30,
    };

    // Lower encounter rate at higher levels (experienced adventurers)
    let rate = (base_rate as i32 - level as i32 * 2).max(5);

    rng.gen_range(0..100) < rate
}

/// Get combat status display
pub fn get_combat_status(state: &GameState) -> Option<String> {
    let combat = state.combat.as_ref()?;
    let monster = get_monster(&combat.monster_key)?;

    Some(format!(
        "{}\n\
         Monster HP: {}/{}\n\
         Your HP: {}/{}\n\
         Your Mana: {}/{}",
        monster.name,
        combat.monster_hp, combat.monster_max_hp,
        state.health, state.max_health,
        state.mana, state.max_mana
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_random_encounter() {
        let mut state = GameState::new("Test");
        let msg = start_random_encounter(&mut state, Region::Village);

        assert!(msg.is_some());
        assert!(state.combat.is_some());
    }

    #[test]
    fn test_start_combat_with() {
        let mut state = GameState::new("Test");
        let msg = start_combat_with(&mut state, "rat");

        assert!(msg.is_some());
        assert!(state.combat.is_some());
        assert_eq!(state.combat.as_ref().unwrap().monster_key, "rat");
    }

    #[test]
    fn test_player_attack() {
        let mut state = GameState::new("Test");
        start_combat_with(&mut state, "rat");

        let result = player_attack(&mut state);

        assert!(result.monster_damage > 0 || result.combat_ended);
    }

    #[test]
    fn test_attempt_flee() {
        let mut state = GameState::new("Test");
        start_combat_with(&mut state, "rat");

        // Try multiple times (flee has chance element)
        for _ in 0..20 {
            if state.combat.is_none() {
                start_combat_with(&mut state, "rat");
            }
            attempt_flee(&mut state);
        }

        // Just verify it doesn't panic
    }

    #[test]
    fn test_combat_victory() {
        let mut state = GameState::new("Test");
        state.level = 7;  // High level
        start_combat_with(&mut state, "rat");

        // Attack until victory (rat has low HP)
        for _ in 0..10 {
            if state.combat.is_none() {
                break;
            }
            player_attack(&mut state);
        }

        // Should have won against a rat at high level
        assert!(state.monsters_killed > 0 || state.combat.is_none());
    }

    #[test]
    fn test_check_random_encounter() {
        // Just verify it works without panicking
        for _ in 0..100 {
            let _ = check_random_encounter(Region::DarkForest, 3);
        }
    }
}
