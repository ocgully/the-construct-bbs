//! Combat system for Ultimo
//!
//! Handles PvE and PvP combat mechanics.

use super::data::{get_item, get_spell};
use super::state::{Character, CombatState};

/// Result of a combat action
#[derive(Debug, Clone, PartialEq)]
pub enum CombatResult {
    Continue,
    Victory,
    Defeat,
    Fled,
}

/// Player attacks the monster
pub fn player_attack(char: &mut Character, combat: &mut CombatState) -> CombatResult {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let player_attack = char.attack_power();
    let monster_defense = combat.monster.defense;

    // Hit chance based on weapon skill
    let weapon_skill = if let Some(ref weapon_key) = char.equipped_weapon {
        if let Some(item) = get_item(weapon_key) {
            if let Some(skill) = item.required_skill {
                char.get_skill(skill)
            } else {
                50
            }
        } else {
            50
        }
    } else {
        char.get_skill("wrestling")
    };

    let hit_chance = 50 + weapon_skill as i32 / 2;
    let hit_roll = rng.gen_range(0..100);

    if hit_roll < hit_chance {
        // Calculate damage
        let base_damage = player_attack - monster_defense / 2;
        let variance = rng.gen_range(-3..=3);
        let damage = (base_damage + variance).max(1);

        combat.monster.hp -= damage;
        combat
            .combat_log
            .push(format!("You hit for {} damage!", damage));

        // Try to gain weapon skill
        if let Some(ref weapon_key) = char.equipped_weapon {
            if let Some(item) = get_item(weapon_key) {
                if let Some(skill) = item.required_skill {
                    if char.try_skill_gain(skill, combat.monster.level) {
                        combat.combat_log.push(format!("{} skill increased!", skill));
                    }
                }
            }
        } else if char.try_skill_gain("wrestling", combat.monster.level) {
            combat.combat_log.push("Wrestling skill increased!".to_string());
        }

        // Tactics skill gain
        char.try_skill_gain("tactics", combat.monster.level);
    } else {
        combat.combat_log.push("You miss!".to_string());
    }

    // Check for monster death
    if combat.monster.hp <= 0 {
        combat.combat_log.push(format!(
            "You have slain the {}!",
            combat.monster.name
        ));
        return CombatResult::Victory;
    }

    // Monster's turn
    monster_attack(char, combat)
}

/// Player casts a spell
pub fn player_cast(
    char: &mut Character,
    combat: &mut CombatState,
    spell_key: &str,
) -> CombatResult {
    if !char.can_cast(spell_key) {
        combat
            .combat_log
            .push("Cannot cast that spell!".to_string());
        return CombatResult::Continue;
    }

    if let Some(spell) = get_spell(spell_key) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Consume mana and reagents
        char.mana -= spell.mana_cost;
        for (reagent, amount) in spell.reagents {
            char.remove_item(reagent, *amount);
        }

        // Calculate spell damage based on magery skill
        let skill_bonus = char.get_skill("magery") as f32 / 100.0;
        let eval_bonus = char.get_skill("eval_int") as f32 / 200.0;
        let variance = rng.gen_range(0.8..1.2);

        let damage =
            ((spell.power as f32) * (1.0 + skill_bonus + eval_bonus) * variance) as i32;

        if spell.name.contains("Heal") {
            // Healing spell
            char.hp = (char.hp + damage).min(char.max_hp);
            combat
                .combat_log
                .push(format!("You heal for {} HP!", damage));
        } else {
            // Damage spell
            combat.monster.hp -= damage;
            combat
                .combat_log
                .push(format!("{} hits for {} damage!", spell.name, damage));

            // Resist check from monster
            let resist = combat.monster.defense / 4;
            if resist > 0 {
                combat.monster.hp += resist;
                combat.combat_log.push(format!(
                    "{} resists {} damage.",
                    combat.monster.name, resist
                ));
            }
        }

        // Magery skill gain
        if char.try_skill_gain("magery", spell.required_magery) {
            combat.combat_log.push("Magery skill increased!".to_string());
        }

        // Check for monster death
        if combat.monster.hp <= 0 {
            combat.combat_log.push(format!(
                "You have slain the {}!",
                combat.monster.name
            ));
            return CombatResult::Victory;
        }

        // Monster's turn
        return monster_attack(char, combat);
    }

    CombatResult::Continue
}

/// Player uses an item
pub fn player_use_item(
    char: &mut Character,
    combat: &mut CombatState,
    item_key: &str,
) -> CombatResult {
    if char.get_item_count(item_key) == 0 {
        combat.combat_log.push("You don't have that item!".to_string());
        return CombatResult::Continue;
    }

    if let Some(item) = get_item(item_key) {
        // Use the item
        if item.name.contains("Heal") || item.name.contains("heal") {
            let heal = item.power;
            char.hp = (char.hp + heal).min(char.max_hp);
            char.remove_item(item_key, 1);
            combat.combat_log.push(format!("You drink a {} and recover {} HP!", item.name, heal));
        } else if item.name.contains("Mana") || item.name.contains("mana") {
            let restore = item.power;
            char.mana = (char.mana + restore).min(char.max_mana);
            char.remove_item(item_key, 1);
            combat.combat_log.push(format!("You drink a {} and recover {} mana!", item.name, restore));
        } else if item.name == "Bandage" {
            // Use healing skill
            let healing_skill = char.get_skill("healing");
            let heal = (healing_skill as i32 / 5).max(5);
            char.hp = (char.hp + heal).min(char.max_hp);
            char.remove_item(item_key, 1);
            combat.combat_log.push(format!("You bandage your wounds for {} HP.", heal));

            // Healing skill gain
            if char.try_skill_gain("healing", combat.monster.level) {
                combat.combat_log.push("Healing skill increased!".to_string());
            }
        } else {
            combat.combat_log.push("Can't use that in combat!".to_string());
            return CombatResult::Continue;
        }

        // Monster's turn
        return monster_attack(char, combat);
    }

    CombatResult::Continue
}

/// Player attempts to flee
pub fn player_flee(char: &mut Character, combat: &mut CombatState) -> CombatResult {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Flee chance based on dexterity and level difference
    let level_diff = char.level() as i32 - combat.monster.level as i32;
    let flee_chance = 30 + char.dexterity + level_diff * 5;

    if rng.gen_range(0..100) < flee_chance {
        combat.combat_log.push("You flee from combat!".to_string());
        CombatResult::Fled
    } else {
        combat.combat_log.push("You fail to escape!".to_string());
        monster_attack(char, combat)
    }
}

/// Monster attacks the player
fn monster_attack(char: &mut Character, combat: &mut CombatState) -> CombatResult {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Monster hit chance
    let hit_chance = 60 + combat.monster.level as i32;
    let hit_roll = rng.gen_range(0..100);

    if hit_roll < hit_chance {
        let player_defense = char.defense();
        let base_damage = combat.monster.damage - player_defense / 2;
        let variance = rng.gen_range(-2..=2);
        let damage = (base_damage + variance).max(1);

        char.hp -= damage;
        combat.combat_log.push(format!(
            "{} hits you for {} damage!",
            combat.monster.name, damage
        ));

        // Parrying skill gain
        if char.equipped_shield.is_some() {
            char.try_skill_gain("parrying", combat.monster.level);
        }

        // Resist spells skill gain (if monster does magic damage - simplified)
        char.try_skill_gain("resist_spells", combat.monster.level);

        // Check for player death
        if char.hp <= 0 {
            combat.combat_log.push("You have been slain!".to_string());
            return CombatResult::Defeat;
        }
    } else {
        combat.combat_log.push(format!(
            "{} misses!",
            combat.monster.name
        ));
    }

    CombatResult::Continue
}

/// Calculate PvP damage (for arena combat)
#[allow(dead_code)]
pub fn pvp_damage(attacker: &Character, defender: &Character) -> i32 {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let attack = attacker.attack_power();
    let defense = defender.defense();

    let base_damage = attack - defense / 2;
    let variance = rng.gen_range(-5..=5);

    (base_damage + variance).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::ultimo::data::Monster;

    fn create_test_character() -> Character {
        let mut char = Character::new("TestWarrior", 1);
        char.strength = 20;
        char.dexterity = 15;
        char.hp = 100;
        char.max_hp = 100;
        char
    }

    fn create_test_monster() -> Monster {
        Monster {
            template_key: "rat".to_string(),
            name: "Giant Rat".to_string(),
            level: 2,
            hp: 10,
            max_hp: 10,
            damage: 3,
            defense: 1,
            position: (0, 0),
        }
    }

    #[test]
    fn test_player_attack() {
        let mut char = create_test_character();
        let mut combat = CombatState {
            monster: create_test_monster(),
            player_acted: false,
            combat_log: Vec::new(),
        };

        // Attack multiple times to ensure damage is dealt
        for _ in 0..10 {
            let result = player_attack(&mut char, &mut combat);
            if result == CombatResult::Victory {
                break;
            }
        }

        // Monster should have taken damage
        assert!(combat.monster.hp < combat.monster.max_hp || combat.monster.hp <= 0);
    }

    #[test]
    fn test_player_flee() {
        let mut char = create_test_character();
        char.dexterity = 100; // High dex for easier flee

        let mut combat = CombatState {
            monster: create_test_monster(),
            player_acted: false,
            combat_log: Vec::new(),
        };

        // Try to flee multiple times
        let mut fled = false;
        for _ in 0..20 {
            let result = player_flee(&mut char, &mut combat);
            if result == CombatResult::Fled {
                fled = true;
                break;
            }
            // Reset HP for next attempt
            char.hp = char.max_hp;
        }

        assert!(fled, "Should be able to flee with high dexterity");
    }

    #[test]
    fn test_monster_can_kill_player() {
        let mut char = create_test_character();
        char.hp = 1; // Very low HP

        let mut combat = CombatState {
            monster: Monster {
                template_key: "dragon".to_string(),
                name: "Dragon".to_string(),
                level: 25,
                hp: 500,
                max_hp: 500,
                damage: 100,
                defense: 40,
                position: (0, 0),
            },
            player_acted: false,
            combat_log: Vec::new(),
        };

        let result = player_attack(&mut char, &mut combat);

        // Should either be Continue (if we missed or dragon missed) or Defeat
        assert!(matches!(
            result,
            CombatResult::Continue | CombatResult::Defeat
        ));
    }

    #[test]
    fn test_pvp_damage() {
        let attacker = create_test_character();
        let defender = create_test_character();

        let damage = pvp_damage(&attacker, &defender);
        assert!(damage >= 1);
    }
}
