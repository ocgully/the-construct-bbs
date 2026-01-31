//! Magic system for Morningmist
//! Spell casting, the Fountain of Scrolls, and spell effects

#![allow(dead_code)]

use rand::Rng;
use super::data::{Spell, SpellType, get_spell, get_spell_by_incantation, RoomSpecial};
use super::state::GameState;

/// Result of attempting to cast a spell
#[derive(Debug, Clone)]
pub struct SpellCast {
    pub success: bool,
    pub message: String,
    pub effect: SpellEffect,
}

/// Effect of a spell
#[derive(Debug, Clone)]
pub enum SpellEffect {
    /// No effect (spell failed or utility)
    None,
    /// Damage dealt to target
    Damage(u32),
    /// Healing applied
    Healing(u32),
    /// Shield applied with power
    Shield(u32),
    /// Light spell cast
    Light,
    /// Teleport to location
    Teleport(String),
    /// Magic detected
    DetectMagic(Vec<String>),
    /// Ward placed
    Ward,
    /// Special puzzle effect
    PuzzleSolved(String),
}

/// Player's spellbook
#[derive(Debug, Clone, Default)]
pub struct SpellBook {
    pub known_spells: Vec<String>,
}

impl SpellBook {
    pub fn new() -> Self {
        Self {
            known_spells: vec!["light".to_string()],
        }
    }

    pub fn knows(&self, spell_key: &str) -> bool {
        self.known_spells.contains(&spell_key.to_string())
    }

    pub fn learn(&mut self, spell_key: &str) -> bool {
        if self.knows(spell_key) {
            return false;
        }
        self.known_spells.push(spell_key.to_string());
        true
    }

    pub fn get_known(&self) -> Vec<&'static Spell> {
        self.known_spells
            .iter()
            .filter_map(|key| get_spell(key))
            .collect()
    }
}

/// Attempt to cast a spell by incantation
pub fn cast_spell(
    incantation: &str,
    state: &mut GameState,
    current_room_special: Option<RoomSpecial>,
) -> SpellCast {
    // Find the spell by incantation
    let spell = match get_spell_by_incantation(incantation) {
        Some(s) => s,
        None => {
            return SpellCast {
                success: false,
                message: "The words hold no power. Perhaps the incantation is wrong?".to_string(),
                effect: SpellEffect::None,
            };
        }
    };

    // Check if player knows the spell
    if !state.knows_spell(spell.key) {
        return SpellCast {
            success: false,
            message: format!("You don't know how to cast {}. Find a scroll to learn it.", spell.name),
            effect: SpellEffect::None,
        };
    }

    // Check level requirement
    if state.level < spell.required_level {
        return SpellCast {
            success: false,
            message: format!(
                "You are not powerful enough to cast {}. Requires level {}.",
                spell.name, spell.required_level
            ),
            effect: SpellEffect::None,
        };
    }

    // Check mana
    if state.mana < spell.mana_cost {
        return SpellCast {
            success: false,
            message: format!(
                "Not enough mana! {} requires {} mana, you have {}.",
                spell.name, spell.mana_cost, state.mana
            ),
            effect: SpellEffect::None,
        };
    }

    // Consume mana
    state.use_mana(spell.mana_cost);

    // Apply spell effect
    match spell.spell_type {
        SpellType::Combat => {
            // Combat spells only work in combat
            if state.combat.is_some() {
                let power = calculate_spell_power(spell.power, state);
                SpellCast {
                    success: true,
                    message: format!("You cast {}! It deals {} damage!", spell.name, power),
                    effect: SpellEffect::Damage(power),
                }
            } else {
                // Refund mana if not in combat
                state.restore_mana(spell.mana_cost);
                SpellCast {
                    success: false,
                    message: "There is nothing to attack here.".to_string(),
                    effect: SpellEffect::None,
                }
            }
        }
        SpellType::Healing => {
            let power = calculate_spell_power(spell.power, state);
            state.heal(power);
            SpellCast {
                success: true,
                message: format!("You cast {}! Restored {} health.", spell.name, power),
                effect: SpellEffect::Healing(power),
            }
        }
        SpellType::Defense => {
            if spell.key == "shield" {
                let power = calculate_spell_power(spell.power, state);
                state.active_effects.insert("shield".to_string(), 5);  // 5 turns
                SpellCast {
                    success: true,
                    message: format!("A magical shield surrounds you! ({} damage reduction for 5 turns)", power),
                    effect: SpellEffect::Shield(power),
                }
            } else if spell.key == "ward" {
                SpellCast {
                    success: true,
                    message: "You place a protective ward on this location.".to_string(),
                    effect: SpellEffect::Ward,
                }
            } else {
                SpellCast {
                    success: true,
                    message: format!("You cast {}.", spell.name),
                    effect: SpellEffect::None,
                }
            }
        }
        SpellType::Utility => {
            match spell.key {
                "light" => {
                    state.active_effects.insert("light".to_string(), 10);
                    SpellCast {
                        success: true,
                        message: "A soft glow emanates from your hand, illuminating the area.".to_string(),
                        effect: SpellEffect::Light,
                    }
                }
                "teleport" => {
                    // Teleport to village (simple version)
                    SpellCast {
                        success: true,
                        message: "Reality bends around you... you find yourself in the village square.".to_string(),
                        effect: SpellEffect::Teleport("village_square".to_string()),
                    }
                }
                "detect_magic" => {
                    // Detect hidden items/passages in current room
                    SpellCast {
                        success: true,
                        message: "Your senses expand... you detect magical energies nearby.".to_string(),
                        effect: SpellEffect::DetectMagic(vec![]),  // Would be populated by room
                    }
                }
                "glory_tashanna" => {
                    // Special puzzle spell - only works at altar
                    if current_room_special == Some(RoomSpecial::Altar) {
                        SpellCast {
                            success: true,
                            message: "The altar glows with divine light! Tashanna smiles upon you!".to_string(),
                            effect: SpellEffect::PuzzleSolved("altar_blessing".to_string()),
                        }
                    } else {
                        SpellCast {
                            success: true,
                            message: "You chant the sacred words, but nothing happens here.".to_string(),
                            effect: SpellEffect::None,
                        }
                    }
                }
                _ => SpellCast {
                    success: true,
                    message: format!("You cast {}.", spell.name),
                    effect: SpellEffect::None,
                }
            }
        }
    }
}

/// Calculate spell power with equipment bonuses
fn calculate_spell_power(base_power: u32, state: &GameState) -> u32 {
    let level_bonus = state.level as u32 * 2;
    let equipment_bonus = state.equipment_power_bonus();

    // Apply percentage bonus from enchanted staff
    let multiplier = if state.equipped_weapon.as_deref() == Some("enchanted_staff") {
        1.25
    } else {
        1.0
    };

    ((base_power + level_bonus + equipment_bonus) as f32 * multiplier) as u32
}

/// Apply combat spell damage to monster
pub fn apply_combat_spell(state: &mut GameState, damage: u32) -> Option<(bool, u32, u64, i64)> {
    if let Some(ref mut combat) = state.combat {
        if damage >= combat.monster_hp {
            let xp = 0u64;  // Will be set by caller from monster data
            let gold = 0i64;
            combat.monster_hp = 0;
            Some((true, 0, xp, gold))  // Monster defeated
        } else {
            combat.monster_hp -= damage;
            Some((false, combat.monster_hp, 0, 0))  // Monster still alive
        }
    } else {
        None
    }
}

/// Use the Fountain of Scrolls to create a random spell scroll
pub fn use_fountain(state: &mut GameState) -> Result<String, String> {
    // Need 3 pine cones
    let pine_cones = state.item_count("pine_cone");
    if pine_cones < 3 {
        return Err(format!(
            "You need 3 pine cones to use the fountain. You have {}.",
            pine_cones
        ));
    }

    // Remove pine cones
    state.remove_item("pine_cone", 3);

    // Determine scroll based on level and chance
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(0..100);

    let scroll_key = match state.level {
        1 => {
            // Low level: mostly basic scrolls
            match roll {
                0..=40 => "scroll_light",
                41..=70 => "scroll_heal",
                71..=90 => "scroll_shield",
                _ => "scroll_fireball",
            }
        }
        2..=3 => {
            match roll {
                0..=20 => "scroll_light",
                21..=40 => "scroll_heal",
                41..=60 => "scroll_shield",
                61..=80 => "scroll_fireball",
                81..=95 => "scroll_teleport",
                _ => "scroll_inferno",
            }
        }
        4..=5 => {
            match roll {
                0..=15 => "scroll_heal",
                16..=30 => "scroll_fireball",
                31..=50 => "scroll_teleport",
                51..=70 => "scroll_inferno",
                71..=90 => "scroll_arcane_blast",
                _ => "scroll_resurrection",
            }
        }
        _ => {
            // High level: rare scrolls more common
            match roll {
                0..=20 => "scroll_teleport",
                21..=45 => "scroll_inferno",
                46..=75 => "scroll_arcane_blast",
                _ => "scroll_resurrection",
            }
        }
    };

    // Construct scroll item key
    let scroll_item = scroll_key.to_string();

    // Add to inventory (check capacity)
    if state.add_item(&scroll_item, 1) {
        let scroll_name = match scroll_key {
            "scroll_light" => "Scroll of Light",
            "scroll_heal" => "Scroll of Healing",
            "scroll_shield" => "Scroll of Shield",
            "scroll_fireball" => "Scroll of Fireball",
            "scroll_teleport" => "Scroll of Teleport",
            "scroll_inferno" => "Scroll of Inferno",
            "scroll_arcane_blast" => "Scroll of Arcane Blast",
            "scroll_resurrection" => "Scroll of Resurrection",
            _ => "Mysterious Scroll",
        };

        Ok(format!(
            "You throw the pine cones into the shimmering water...\n\
             The fountain glows brilliantly!\n\
             A scroll materializes before you: {}!",
            scroll_name
        ))
    } else {
        // Inventory full - refund pine cones
        state.add_item("pine_cone", 3);
        Err("Your inventory is full! Make room and try again.".to_string())
    }
}

/// Use a scroll to learn a spell
pub fn use_scroll(state: &mut GameState, scroll_key: &str) -> Result<String, String> {
    // Check if we have the scroll
    if !state.has_item(scroll_key) {
        return Err("You don't have that scroll.".to_string());
    }

    // Map scroll to spell
    let spell_key = match scroll_key {
        "scroll_light" => "light",
        "scroll_heal" => "heal",
        "scroll_shield" => "shield",
        "scroll_fireball" => "fireball",
        "scroll_teleport" => "teleport",
        "scroll_inferno" => "inferno",
        "scroll_arcane_blast" => "arcane_blast",
        "scroll_resurrection" => "resurrection",
        _ => return Err("This scroll contains no spell.".to_string()),
    };

    // Check level requirement
    if let Some(spell) = get_spell(spell_key) {
        if state.level < spell.required_level {
            return Err(format!(
                "You cannot learn {} yet. Requires level {}.",
                spell.name, spell.required_level
            ));
        }

        // Check if already known
        if state.knows_spell(spell_key) {
            return Err(format!("You already know {}.", spell.name));
        }

        // Consume scroll
        state.remove_item(scroll_key, 1);

        // Learn spell
        state.learn_spell(spell_key);

        Ok(format!(
            "You study the scroll intently...\n\
             The words burn into your mind as the scroll crumbles to dust.\n\
             You have learned {}!\n\
             Incantation: \"{}\"",
            spell.name, spell.incantation
        ))
    } else {
        Err("The spell on this scroll is corrupted.".to_string())
    }
}

/// Get list of known spells with details
pub fn format_spellbook(state: &GameState) -> Vec<(String, String, u32, String)> {
    state.known_spells
        .iter()
        .filter_map(|key| {
            get_spell(key).map(|spell| {
                (
                    spell.name.to_string(),
                    spell.incantation.to_string(),
                    spell.mana_cost,
                    spell.description.to_string(),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spellbook() {
        let mut book = SpellBook::new();
        assert!(book.knows("light"));
        assert!(!book.knows("fireball"));

        book.learn("fireball");
        assert!(book.knows("fireball"));
    }

    #[test]
    fn test_cast_unknown_spell() {
        let mut state = GameState::new("Test");
        state.known_spells.clear();  // Remove starting spell

        let result = cast_spell("luminos", &mut state, None);
        assert!(!result.success);
    }

    #[test]
    fn test_cast_not_enough_mana() {
        let mut state = GameState::new("Test");
        state.mana = 0;

        let result = cast_spell("luminos", &mut state, None);
        assert!(!result.success);
        assert!(result.message.contains("mana"));
    }

    #[test]
    fn test_cast_light() {
        let mut state = GameState::new("Test");
        state.mana = 30;

        let result = cast_spell("luminos", &mut state, None);
        assert!(result.success);
        assert!(state.active_effects.contains_key("light"));
    }

    #[test]
    fn test_cast_heal() {
        let mut state = GameState::new("Test");
        state.learn_spell("heal");
        state.health = 20;
        state.max_health = 50;
        state.mana = 30;

        let result = cast_spell("vitae restauro", &mut state, None);
        assert!(result.success);
        assert!(state.health > 20);
    }

    #[test]
    fn test_use_fountain_no_pine_cones() {
        let mut state = GameState::new("Test");

        let result = use_fountain(&mut state);
        assert!(result.is_err());
    }

    #[test]
    fn test_use_fountain_success() {
        let mut state = GameState::new("Test");
        state.add_item("pine_cone", 5);

        let result = use_fountain(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.item_count("pine_cone"), 2);
    }

    #[test]
    fn test_use_scroll() {
        let mut state = GameState::new("Test");
        state.add_item("scroll_shield", 1);
        state.level = 2;  // Meet level requirement

        let result = use_scroll(&mut state, "scroll_shield");
        assert!(result.is_ok());
        assert!(state.knows_spell("shield"));
        assert!(!state.has_item("scroll_shield"));
    }

    #[test]
    fn test_glory_tashanna_at_altar() {
        let mut state = GameState::new("Test");
        state.learn_spell("glory_tashanna");
        state.level = 3;
        state.mana = 10;

        let result = cast_spell("glory be to tashanna", &mut state, Some(RoomSpecial::Altar));
        assert!(result.success);
        assert!(matches!(result.effect, SpellEffect::PuzzleSolved(_)));
    }
}
