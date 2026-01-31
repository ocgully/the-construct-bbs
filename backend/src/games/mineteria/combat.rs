//! Combat system for Mineteria
//!
//! Handles monster spawning, combat mechanics, and loot.

use rand::Rng;
use serde::{Deserialize, Serialize};
use super::data::{MonsterType, Monster, ItemType, ToolType, is_daytime};
use super::state::GameState;

/// An active monster in the world
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveMonster {
    pub monster_type: MonsterType,
    pub health: i32,
    pub max_health: i32,
    pub x: i32,
    pub y: i32,
}

impl ActiveMonster {
    pub fn new(monster_type: MonsterType, x: i32, y: i32) -> Self {
        let stats = monster_type.get_monster();
        Self {
            monster_type,
            health: stats.max_health,
            max_health: stats.max_health,
            x,
            y,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }

    pub fn get_stats(&self) -> Monster {
        self.monster_type.get_monster()
    }
}

/// Result of a combat action
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub player_damage_dealt: i32,
    pub monster_damage_dealt: i32,
    pub monster_killed: bool,
    pub player_killed: bool,
    pub xp_gained: i32,
    pub loot: Vec<(ItemType, u8)>,
    pub message: String,
}

/// Calculate player's attack damage
pub fn calculate_player_damage(state: &GameState) -> i32 {
    // Base damage
    let mut damage = 1;

    // Check for weapon in selected slot
    if let Some(slot) = state.inventory.selected_item() {
        if let Some(ref tool) = slot.tool_data {
            if tool.tool_type == ToolType::Sword {
                damage += 4 + tool.material.damage_bonus();
            } else {
                // Other tools do less damage
                damage += 2 + tool.material.damage_bonus() / 2;
            }
        }
    }

    // Level bonus
    damage += state.level / 5;

    damage
}

/// Calculate monster's attack damage
pub fn calculate_monster_damage(monster: &Monster) -> i32 {
    monster.damage
}

/// Perform a combat round
pub fn combat_round(state: &mut GameState, monster: &mut ActiveMonster) -> CombatResult {
    let mut rng = rand::thread_rng();

    let monster_stats = monster.get_stats();

    // Player attacks first
    let player_base_damage = calculate_player_damage(state);

    // Random variance (+/- 20%)
    let variance = rng.gen_range(-20..=20) as f32 / 100.0;
    let player_damage = ((player_base_damage as f32) * (1.0 + variance)).round() as i32;

    // Apply defense
    let actual_player_damage = (player_damage - monster_stats.defense / 2).max(1);
    monster.health -= actual_player_damage;

    // Use weapon durability
    state.inventory.use_selected_tool();

    let mut result = CombatResult {
        player_damage_dealt: actual_player_damage,
        monster_damage_dealt: 0,
        monster_killed: false,
        player_killed: false,
        xp_gained: 0,
        loot: Vec::new(),
        message: String::new(),
    };

    if monster.is_dead() {
        result.monster_killed = true;
        result.xp_gained = monster_stats.xp_reward;
        result.loot = generate_loot(&monster_stats);
        state.stats.monsters_killed += 1;

        result.message = format!(
            "You dealt {} damage and slayed the {}! (+{} XP)",
            actual_player_damage,
            monster_stats.name,
            result.xp_gained
        );

        // Add XP
        if state.add_experience(result.xp_gained) {
            result.message.push_str(&format!(" LEVEL UP! You are now level {}!", state.level));
        }

        // Add loot to inventory
        for (item, count) in &result.loot {
            state.inventory.add_item(*item, *count);
        }

        return result;
    }

    // Monster attacks
    let monster_base_damage = calculate_monster_damage(&monster_stats);
    let monster_variance = rng.gen_range(-20..=20) as f32 / 100.0;
    let monster_damage = ((monster_base_damage as f32) * (1.0 + monster_variance)).round() as i32;

    // Player defense (from level)
    let player_defense = state.level / 3;
    let actual_monster_damage = (monster_damage - player_defense).max(1);

    result.player_killed = state.take_damage(actual_monster_damage);
    result.monster_damage_dealt = actual_monster_damage;

    if result.player_killed {
        result.message = format!(
            "You dealt {} damage but the {} struck back for {} damage. You died!",
            actual_player_damage,
            monster_stats.name,
            actual_monster_damage
        );
    } else {
        result.message = format!(
            "You dealt {} damage. The {} struck back for {} damage. ({}/{} HP)",
            actual_player_damage,
            monster_stats.name,
            actual_monster_damage,
            state.health,
            state.max_health
        );
    }

    result
}

/// Generate loot from a killed monster
fn generate_loot(monster: &Monster) -> Vec<(ItemType, u8)> {
    let mut rng = rand::thread_rng();
    let mut loot = Vec::new();

    // Common drops
    match monster.monster_type {
        MonsterType::Zombie => {
            if rng.gen_bool(0.5) {
                loot.push((ItemType::RawMeat, 1));
            }
        }
        MonsterType::Skeleton => {
            loot.push((ItemType::Arrow, rng.gen_range(1..=3)));
            if rng.gen_bool(0.1) {
                loot.push((ItemType::Bow, 1));
            }
        }
        MonsterType::Spider => {
            // Could drop string (not implemented)
        }
        MonsterType::Slime => {
            // Could drop slimeball (not implemented)
        }
        MonsterType::Creeper => {
            // Gunpowder (not implemented)
        }
        MonsterType::Bat => {
            // Nothing
        }
        MonsterType::GiantSpider => {
            if rng.gen_bool(0.3) {
                loot.push((ItemType::Diamond, 1));
            }
        }
        MonsterType::UndeadKing => {
            loot.push((ItemType::Diamond, rng.gen_range(3..=5)));
            loot.push((ItemType::GoldIngot, rng.gen_range(5..=10)));
        }
    }

    // Rare drops
    if rng.gen_bool(0.05) {
        loot.push((ItemType::IronIngot, 1));
    }

    loot
}

/// Check if monsters should spawn at this location
pub fn should_spawn_monster(state: &GameState, x: i32, y: i32) -> Option<MonsterType> {
    let mut rng = rand::thread_rng();

    // Don't spawn too close to player
    let dist_x = (x - state.position.x).abs();
    let dist_y = (y - state.position.y).abs();
    if dist_x < 5 && dist_y < 5 {
        return None;
    }
    if dist_x > 30 || dist_y > 30 {
        return None; // Too far
    }

    let is_underground = y < 60;
    let is_night = !is_daytime(state.world_tick);

    // Spawn chance
    if rng.gen_range(0..1000) > 5 {
        return None;
    }

    let monster_types: Vec<MonsterType> = if is_underground {
        vec![
            MonsterType::Zombie,
            MonsterType::Skeleton,
            MonsterType::Spider,
            MonsterType::Slime,
            MonsterType::Bat,
        ]
    } else if is_night {
        vec![
            MonsterType::Zombie,
            MonsterType::Skeleton,
            MonsterType::Spider,
            MonsterType::Creeper,
        ]
    } else {
        return None; // No surface spawns during day
    };

    if monster_types.is_empty() {
        return None;
    }

    let monster = monster_types[rng.gen_range(0..monster_types.len())];

    // Check spawn conditions
    let stats = monster.get_monster();
    if is_underground && !stats.spawns_underground {
        return None;
    }
    if is_night && !stats.spawns_at_night && !is_underground {
        return None;
    }

    Some(monster)
}

/// Attempt to flee from combat
pub fn attempt_flee(state: &GameState, monster: &Monster) -> (bool, i32) {
    let mut rng = rand::thread_rng();

    // Base 50% chance, modified by level difference
    let level_diff = state.level - (monster.xp_reward / 10);
    let flee_chance = 50 + level_diff * 5;
    let flee_chance = flee_chance.clamp(10, 90);

    let roll = rng.gen_range(0..100);
    let success = roll < flee_chance;

    let damage_taken = if !success {
        // Failed flee, monster gets a free hit
        let base = monster.damage;
        let variance = rng.gen_range(-20..=20) as f32 / 100.0;
        ((base as f32) * (1.0 + variance)).round() as i32
    } else {
        0
    };

    (success, damage_taken)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::mineteria::data::ToolMaterial;

    #[test]
    fn test_active_monster() {
        let monster = ActiveMonster::new(MonsterType::Zombie, 10, 20);
        assert_eq!(monster.health, 20);
        assert!(!monster.is_dead());
    }

    #[test]
    fn test_player_damage_calculation() {
        let state = GameState::new(0);
        let damage = calculate_player_damage(&state);
        assert!(damage >= 1);
    }

    #[test]
    fn test_player_damage_with_sword() {
        let mut state = GameState::new(0);
        state.inventory.add_item(ItemType::Tool(ToolType::Sword, ToolMaterial::Iron), 1);
        state.inventory.select_slot(0);

        let damage = calculate_player_damage(&state);
        assert!(damage >= 8); // 1 base + 4 sword + 3 iron bonus
    }

    #[test]
    fn test_combat_round() {
        let mut state = GameState::new(0);
        state.health = 20;

        let mut monster = ActiveMonster::new(MonsterType::Slime, 5, 5);

        let result = combat_round(&mut state, &mut monster);

        assert!(result.player_damage_dealt > 0);
        // Slime has low health, might be killed
        if !result.monster_killed {
            assert!(result.monster_damage_dealt >= 0);
        }
    }

    #[test]
    fn test_monster_kill_rewards() {
        let mut state = GameState::new(0);
        state.inventory.add_item(ItemType::Tool(ToolType::Sword, ToolMaterial::Diamond), 1);
        state.inventory.select_slot(0);

        let mut monster = ActiveMonster::new(MonsterType::Slime, 5, 5);
        monster.health = 1; // About to die

        let result = combat_round(&mut state, &mut monster);

        assert!(result.monster_killed);
        assert!(result.xp_gained > 0);
    }

    #[test]
    fn test_flee_mechanic() {
        let state = GameState::new(0);
        let monster_stats = MonsterType::Slime.get_monster();

        // Run multiple times to test randomness
        let mut successes = 0;
        for _ in 0..100 {
            let (success, _) = attempt_flee(&state, &monster_stats);
            if success {
                successes += 1;
            }
        }

        // Should have some successes and some failures
        assert!(successes > 10);
        assert!(successes < 90);
    }

    #[test]
    fn test_loot_generation() {
        let monster = MonsterType::UndeadKing.get_monster();
        let loot = generate_loot(&monster);

        // Boss should always drop something
        assert!(!loot.is_empty());
    }
}
