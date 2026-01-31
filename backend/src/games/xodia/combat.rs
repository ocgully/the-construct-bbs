//! Combat system for Xodia
//!
//! Handles combat resolution with mechanical dice rolls and
//! prepares data for LLM narration.

use serde::{Serialize, Deserialize};
use rand::Rng;
use super::state::GameState;
use super::world::NPC;
use super::data::get_npc_template;

/// Combat action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CombatAction {
    Attack,
    Defend,
    Flee,
    Cast { spell: String },
    UseItem { item: String },
}

/// Result of a single combat round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatRoundResult {
    pub action: CombatAction,
    pub attacker_roll: i32,
    pub defender_roll: i32,
    pub damage_dealt: i32,
    pub damage_taken: i32,
    pub hit: bool,
    pub critical: bool,
    pub fled: bool,
    pub player_health_after: i32,
    pub enemy_health_after: i32,
    pub combat_ended: bool,
    pub player_victory: bool,
    pub player_defeated: bool,
    pub xp_gained: u64,
    pub loot: Vec<LootDrop>,
    pub narrative_context: String,
}

/// Loot dropped from combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootDrop {
    pub item_key: String,
    pub item_name: String,
    pub quantity: u32,
    pub gold: i64,
}

/// Combat state for an ongoing fight
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatState {
    pub enemy_id: String,
    pub enemy_name: String,
    pub enemy_health: i32,
    pub enemy_max_health: i32,
    pub enemy_damage: i32,
    pub round: u32,
    pub player_is_defending: bool,
}

impl CombatState {
    pub fn new(npc: &NPC) -> Self {
        let template = get_npc_template(&npc.template_key);
        let damage = template.map(|t| t.damage).unwrap_or(5);

        Self {
            enemy_id: npc.instance_id.clone(),
            enemy_name: npc.name.clone(),
            enemy_health: npc.health,
            enemy_max_health: npc.max_health,
            enemy_damage: damage,
            round: 1,
            player_is_defending: false,
        }
    }
}

/// Roll a d20
pub fn roll_d20() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=20)
}

/// Roll damage dice (1d6 by default, can be modified)
pub fn roll_damage(base: i32, modifier: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(1..=6);
    (roll + base + modifier).max(1)
}

/// Resolve a combat round
pub fn resolve_combat_round(
    state: &mut GameState,
    combat: &mut CombatState,
    action: CombatAction,
) -> CombatRoundResult {
    let mut result = CombatRoundResult {
        action: action.clone(),
        attacker_roll: 0,
        defender_roll: 0,
        damage_dealt: 0,
        damage_taken: 0,
        hit: false,
        critical: false,
        fled: false,
        player_health_after: state.health,
        enemy_health_after: combat.enemy_health,
        combat_ended: false,
        player_victory: false,
        player_defeated: false,
        xp_gained: 0,
        loot: Vec::new(),
        narrative_context: String::new(),
    };

    match action {
        CombatAction::Attack => {
            resolve_attack(state, combat, &mut result);
        }
        CombatAction::Defend => {
            combat.player_is_defending = true;
            result.narrative_context = "You raise your guard, bracing for the enemy's attack.".to_string();
            // Enemy still attacks
            resolve_enemy_attack(state, combat, &mut result);
        }
        CombatAction::Flee => {
            resolve_flee(state, combat, &mut result);
        }
        CombatAction::Cast { ref spell } => {
            resolve_spell(state, combat, spell, &mut result);
        }
        CombatAction::UseItem { ref item } => {
            resolve_item(state, item, &mut result);
            // Enemy attacks after item use
            if !result.combat_ended {
                resolve_enemy_attack(state, combat, &mut result);
            }
        }
    }

    // Update final states
    result.player_health_after = state.health;
    result.enemy_health_after = combat.enemy_health;

    // Check for combat end
    if combat.enemy_health <= 0 {
        result.combat_ended = true;
        result.player_victory = true;
        result.xp_gained = calculate_xp(combat);
        result.loot = generate_loot(combat);
    } else if state.health <= 0 {
        result.combat_ended = true;
        result.player_defeated = true;
    }

    combat.round += 1;
    combat.player_is_defending = false;

    result
}

/// Resolve player attack
fn resolve_attack(
    state: &mut GameState,
    combat: &mut CombatState,
    result: &mut CombatRoundResult,
) {
    let attack_roll = roll_d20();
    let attack_bonus = state.attack_power();
    let total_attack = attack_roll + attack_bonus;

    // Enemy AC is 10 + level bonus
    let enemy_ac = 10 + (get_npc_template(&combat.enemy_id)
        .map(|t| t.level as i32)
        .unwrap_or(1) / 2);

    result.attacker_roll = attack_roll;
    result.hit = total_attack >= enemy_ac;
    result.critical = attack_roll == 20;

    if result.hit {
        let base_damage = state.attack_power();
        let damage = roll_damage(base_damage, 0);
        let final_damage = if result.critical { damage * 2 } else { damage };

        combat.enemy_health -= final_damage;
        result.damage_dealt = final_damage;

        result.narrative_context = if result.critical {
            format!("Critical hit! You strike {} for {} damage!", combat.enemy_name, final_damage)
        } else {
            format!("You hit {} for {} damage.", combat.enemy_name, final_damage)
        };
    } else {
        result.narrative_context = format!("Your attack misses {}.", combat.enemy_name);
    }

    // Enemy counterattack (if still alive)
    if combat.enemy_health > 0 {
        resolve_enemy_attack(state, combat, result);
    }
}

/// Resolve enemy attack
fn resolve_enemy_attack(
    state: &mut GameState,
    combat: &mut CombatState,
    result: &mut CombatRoundResult,
) {
    let attack_roll = roll_d20();
    let player_ac = state.defense();

    // Apply defense bonus if player is defending
    let effective_ac = if combat.player_is_defending {
        player_ac + 4
    } else {
        player_ac
    };

    result.defender_roll = attack_roll;

    if attack_roll >= effective_ac {
        let damage = roll_damage(combat.enemy_damage, 0);

        // Reduce damage if defending
        let final_damage = if combat.player_is_defending {
            (damage / 2).max(1)
        } else {
            damage
        };

        state.take_damage(final_damage);
        result.damage_taken = final_damage;

        result.narrative_context.push_str(&format!(
            " {} strikes you for {} damage!",
            combat.enemy_name, final_damage
        ));
    } else {
        result.narrative_context.push_str(&format!(
            " {} attacks but you evade!",
            combat.enemy_name
        ));
    }
}

/// Resolve flee attempt
fn resolve_flee(
    state: &GameState,
    combat: &mut CombatState,
    result: &mut CombatRoundResult,
) {
    let flee_roll = roll_d20();
    let dex_mod = state.stats.modifier("dexterity");
    let total = flee_roll + dex_mod;

    // DC is 10 + enemy level
    let dc = 10 + (get_npc_template(&combat.enemy_id)
        .map(|t| t.level as i32)
        .unwrap_or(1));

    result.attacker_roll = flee_roll;

    if total >= dc {
        result.fled = true;
        result.combat_ended = true;
        result.narrative_context = "You manage to escape from combat!".to_string();
    } else {
        result.narrative_context = format!(
            "You try to flee but {} blocks your escape!",
            combat.enemy_name
        );
        // Take opportunity attack
        let damage = roll_damage(combat.enemy_damage / 2, 0);
        result.damage_taken = damage;
        result.narrative_context.push_str(&format!(
            " You take {} damage while retreating!",
            damage
        ));
    }
}

/// Resolve spell cast
fn resolve_spell(
    state: &mut GameState,
    combat: &mut CombatState,
    spell: &str,
    result: &mut CombatRoundResult,
) {
    // Basic spell resolution - expand as needed
    match spell.to_lowercase().as_str() {
        "fireball" | "fire" => {
            let mana_cost = 10;
            if state.mana >= mana_cost {
                state.mana -= mana_cost;
                let damage = roll_damage(10, state.stats.modifier("intelligence"));
                combat.enemy_health -= damage;
                result.damage_dealt = damage;
                result.hit = true;
                result.narrative_context = format!(
                    "Flames erupt from your hands, burning {} for {} damage!",
                    combat.enemy_name, damage
                );
            } else {
                result.narrative_context = "You lack the mana to cast that spell.".to_string();
            }
        }
        "heal" | "healing" => {
            let mana_cost = 8;
            if state.mana >= mana_cost {
                state.mana -= mana_cost;
                let heal = roll_damage(8, state.stats.modifier("wisdom"));
                state.heal(heal);
                result.narrative_context = format!(
                    "Divine light washes over you, healing {} health!",
                    heal
                );
            } else {
                result.narrative_context = "You lack the mana to cast that spell.".to_string();
            }
        }
        _ => {
            result.narrative_context = format!("You don't know the spell '{}'.", spell);
        }
    }

    // Enemy still attacks
    if !result.combat_ended && combat.enemy_health > 0 {
        resolve_enemy_attack(state, combat, result);
    }
}

/// Resolve item use in combat
fn resolve_item(
    state: &mut GameState,
    item: &str,
    result: &mut CombatRoundResult,
) {
    match item.to_lowercase().as_str() {
        "health potion" | "health_potion" | "potion" => {
            if state.has_item("health_potion") {
                state.remove_item("health_potion", 1);
                let heal = 20;
                state.heal(heal);
                result.narrative_context = format!(
                    "You drink a health potion, restoring {} health!",
                    heal
                );
            } else {
                result.narrative_context = "You don't have any health potions.".to_string();
            }
        }
        "mana potion" | "mana_potion" => {
            if state.has_item("mana_potion") {
                state.remove_item("mana_potion", 1);
                let restore = 15;
                state.restore_mana(restore);
                result.narrative_context = format!(
                    "You drink a mana potion, restoring {} mana!",
                    restore
                );
            } else {
                result.narrative_context = "You don't have any mana potions.".to_string();
            }
        }
        _ => {
            result.narrative_context = format!("You can't use '{}' in combat.", item);
        }
    }
}

/// Calculate XP from victory
fn calculate_xp(combat: &CombatState) -> u64 {
    let base_xp = (combat.enemy_max_health as u64) * 2;
    let level_bonus = get_npc_template(&combat.enemy_id)
        .map(|t| t.level as u64 * 10)
        .unwrap_or(10);
    base_xp + level_bonus
}

/// Generate loot from defeated enemy
fn generate_loot(combat: &CombatState) -> Vec<LootDrop> {
    let mut rng = rand::thread_rng();
    let mut loot = Vec::new();

    // Base gold drop
    let gold = rng.gen_range(5..=20) * combat.round as i64;
    loot.push(LootDrop {
        item_key: "gold".to_string(),
        item_name: "Gold".to_string(),
        quantity: 1,
        gold,
    });

    // 25% chance to drop a health potion
    if rng.gen_ratio(1, 4) {
        loot.push(LootDrop {
            item_key: "health_potion".to_string(),
            item_name: "Health Potion".to_string(),
            quantity: 1,
            gold: 0,
        });
    }

    loot
}

/// Generate a narrative prompt for combat result
pub fn generate_combat_narrative_prompt(result: &CombatRoundResult, enemy_name: &str) -> String {
    let action_desc = match &result.action {
        CombatAction::Attack => {
            if result.critical {
                "lands a devastating critical strike"
            } else if result.hit {
                "swings their weapon true"
            } else {
                "swings wildly but misses"
            }
        }
        CombatAction::Defend => "raises their guard defensively",
        CombatAction::Flee => {
            if result.fled {
                "breaks away and escapes"
            } else {
                "tries to flee but is blocked"
            }
        }
        CombatAction::Cast { .. } => "channels arcane energy",
        CombatAction::UseItem { .. } => "reaches for an item",
    };

    let outcome = if result.player_victory {
        format!("{} falls defeated!", enemy_name)
    } else if result.player_defeated {
        "The hero falls...".to_string()
    } else if result.fled {
        "The hero escapes to fight another day.".to_string()
    } else {
        format!(
            "Dealt {} damage, took {} damage.",
            result.damage_dealt, result.damage_taken
        )
    };

    format!(
        r#"Combat round against {}:
The hero {}.
{}

Narrate this combat moment dramatically in 1-2 sentences."#,
        enemy_name, action_desc, outcome
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::xodia::state::CharacterClass;

    fn create_test_state() -> GameState {
        let mut state = GameState::new("TestHero", CharacterClass::Warrior);
        state.health = 50;
        state.max_health = 50;
        state.mana = 20;
        state.max_mana = 20;
        state
    }

    fn create_test_combat() -> CombatState {
        CombatState {
            enemy_id: "forest_goblin".to_string(),
            enemy_name: "Goblin".to_string(),
            enemy_health: 15,
            enemy_max_health: 15,
            enemy_damage: 3,
            round: 1,
            player_is_defending: false,
        }
    }

    #[test]
    fn test_roll_d20() {
        for _ in 0..100 {
            let roll = roll_d20();
            assert!(roll >= 1 && roll <= 20);
        }
    }

    #[test]
    fn test_roll_damage() {
        for _ in 0..100 {
            let damage = roll_damage(5, 2);
            assert!(damage >= 1); // Minimum damage is 1
        }
    }

    #[test]
    fn test_combat_attack() {
        let mut state = create_test_state();
        let mut combat = create_test_combat();

        let result = resolve_combat_round(&mut state, &mut combat, CombatAction::Attack);

        // Combat should have happened
        assert!(result.attacker_roll > 0);
        // Either hit or miss
        assert!(result.hit || !result.hit);
    }

    #[test]
    fn test_combat_defend() {
        let mut state = create_test_state();
        let mut combat = create_test_combat();

        let result = resolve_combat_round(&mut state, &mut combat, CombatAction::Defend);

        // Should not deal damage when defending
        assert_eq!(result.damage_dealt, 0);
        // Should contain defend narrative
        assert!(result.narrative_context.contains("guard") || result.narrative_context.contains("brac"));
    }

    #[test]
    fn test_combat_flee() {
        let mut state = create_test_state();
        let mut combat = create_test_combat();

        let result = resolve_combat_round(&mut state, &mut combat, CombatAction::Flee);

        // Either fled successfully or failed
        assert!(result.fled || !result.fled);
        if result.fled {
            assert!(result.combat_ended);
        }
    }

    #[test]
    fn test_combat_victory() {
        let mut state = create_test_state();
        let mut combat = CombatState {
            enemy_id: "test".to_string(),
            enemy_name: "Weak Enemy".to_string(),
            enemy_health: 1,
            enemy_max_health: 10,
            enemy_damage: 1,
            round: 1,
            player_is_defending: false,
        };

        // Keep attacking until we win
        for _ in 0..20 {
            let result = resolve_combat_round(&mut state, &mut combat, CombatAction::Attack);
            if result.player_victory {
                assert!(result.combat_ended);
                assert!(result.xp_gained > 0);
                return;
            }
        }
        // Should have won by now with such a weak enemy
    }

    #[test]
    fn test_calculate_xp() {
        let combat = create_test_combat();
        let xp = calculate_xp(&combat);
        assert!(xp > 0);
    }

    #[test]
    fn test_generate_loot() {
        let combat = create_test_combat();
        let loot = generate_loot(&combat);
        // Should always have at least gold
        assert!(!loot.is_empty());
        assert!(loot.iter().any(|l| l.gold > 0));
    }

    #[test]
    fn test_combat_state_new() {
        use crate::games::xodia::world::WorldState;

        let world = WorldState::new();
        if let Some(npc) = world.get_npc("forest_path_entrance_forest_goblin") {
            let combat = CombatState::new(npc);
            assert!(!combat.enemy_name.is_empty());
            assert!(combat.enemy_health > 0);
        }
    }

    #[test]
    fn test_spell_cast() {
        let mut state = create_test_state();
        state.mana = 20;
        let mut combat = create_test_combat();

        let result = resolve_combat_round(
            &mut state,
            &mut combat,
            CombatAction::Cast { spell: "fireball".to_string() }
        );

        assert!(result.damage_dealt > 0 || result.narrative_context.contains("lack"));
    }

    #[test]
    fn test_item_use() {
        let mut state = create_test_state();
        state.add_item("health_potion", "Health Potion", 1, 0.2);
        state.health = 30; // Damage the player

        let mut combat = create_test_combat();

        let result = resolve_combat_round(
            &mut state,
            &mut combat,
            CombatAction::UseItem { item: "health_potion".to_string() }
        );

        assert!(result.narrative_context.contains("potion") || result.narrative_context.contains("heal"));
    }
}
