//! Substance (drugs/steroids) system for Usurper
//!
//! Manages the application and effects of performance-enhancing substances.
//! Substances provide powerful stat boosts but risk mental stability.

use super::data::{Substance, SubstanceCategory, SUBSTANCES};
use super::state::GameState;

/// Result of using a substance
pub struct SubstanceResult {
    pub success: bool,
    pub message: String,
    pub psychosis_triggered: bool,
    pub overdose: bool,
}

/// Apply a substance to the player
pub fn apply_substance(state: &mut GameState, substance_key: &str) -> SubstanceResult {
    let substance = match super::data::get_substance(substance_key) {
        Some(s) => s,
        None => return SubstanceResult {
            success: false,
            message: "Unknown substance.".to_string(),
            psychosis_triggered: false,
            overdose: false,
        },
    };

    // Check for addiction effects
    let addiction_level = state.addictions.get(substance_key).copied().unwrap_or(0);
    let was_in_psychosis = state.is_in_psychosis();

    // Higher addiction = diminished effects but worse mental cost
    let effect_multiplier = 1.0 - (addiction_level as f32 * 0.05).min(0.5);
    let mental_cost_multiplier = 1.0 + (addiction_level as f32 * 0.1);

    // Apply mental cost
    let adjusted_mental_cost = (substance.mental_cost as f32 * mental_cost_multiplier) as i32;
    state.mental_stability += adjusted_mental_cost;

    // Check for overdose (using multiple substances of same category)
    let same_category_count = state.active_effects.iter()
        .filter(|e| {
            super::data::get_substance(&e.substance_key)
                .map(|s| s.category == substance.category)
                .unwrap_or(false)
        })
        .count();

    let overdose = same_category_count >= 2 && substance.category != SubstanceCategory::Alchemical;
    if overdose {
        // Overdose causes significant damage and mental degradation
        state.hp = state.hp.saturating_sub(state.max_hp / 4);
        state.mental_stability -= 20;
        return SubstanceResult {
            success: false,
            message: format!(
                "OVERDOSE! Mixing too many {}s causes severe harm!",
                substance.category.name().to_lowercase()
            ),
            psychosis_triggered: state.mental_stability <= 0 && !was_in_psychosis,
            overdose: true,
        };
    }

    // Apply instant healing
    if substance.effects.healing > 0 {
        let healing = (substance.effects.healing as f32 * effect_multiplier) as u32;
        state.heal(healing);
    }

    // Create active effect with diminished stats if addicted
    let effects = super::state::SubstanceEffects {
        strength_mod: (substance.effects.strength_mod as f32 * effect_multiplier) as i32,
        agility_mod: (substance.effects.agility_mod as f32 * effect_multiplier) as i32,
        vitality_mod: (substance.effects.vitality_mod as f32 * effect_multiplier) as i32,
        intelligence_mod: (substance.effects.intelligence_mod as f32 * effect_multiplier) as i32,
        damage_mod: (substance.effects.damage_mod as f32 * effect_multiplier) as i32,
        defense_mod: (substance.effects.defense_mod as f32 * effect_multiplier) as i32,
        action_bonus: substance.effects.action_bonus,
        invincible: substance.effects.invincible_turns > 0,
    };

    state.active_effects.push(super::state::ActiveSubstanceEffect {
        substance_key: substance_key.to_string(),
        turns_remaining: substance.duration_turns,
        effects,
    });

    // Addiction check
    let addiction_roll = rand::random::<u32>() % 100;
    let addiction_threshold = substance.addiction_chance + (addiction_level * 5);
    if addiction_roll < addiction_threshold {
        *state.addictions.entry(substance_key.to_string()).or_insert(0) += 1;
    }

    // Build result message
    let psychosis_triggered = state.mental_stability <= 0 && !was_in_psychosis;

    let mut message = format!("{} takes effect.", substance.name);

    // Describe effects
    let mut effect_parts = Vec::new();
    if substance.effects.strength_mod > 0 {
        effect_parts.push(format!("+{} STR", substance.effects.strength_mod));
    }
    if substance.effects.agility_mod > 0 {
        effect_parts.push(format!("+{} AGI", substance.effects.agility_mod));
    }
    if substance.effects.damage_mod > 0 {
        effect_parts.push(format!("+{} DMG", substance.effects.damage_mod));
    }
    if substance.effects.action_bonus > 0 {
        effect_parts.push(format!("+{} actions", substance.effects.action_bonus));
    }
    if substance.effects.invincible_turns > 0 {
        effect_parts.push("INVINCIBLE!".to_string());
    }
    if substance.effects.healing > 0 {
        effect_parts.push(format!("+{} HP", substance.effects.healing));
    }

    if !effect_parts.is_empty() {
        message.push_str(&format!(" ({})", effect_parts.join(", ")));
    }

    if addiction_level > 0 {
        message.push_str(&format!(" [Addiction Lv{}]", addiction_level));
    }

    if psychosis_triggered {
        message.push_str(" WARNING: YOUR MIND SHATTERS INTO MADNESS!");
    }

    SubstanceResult {
        success: true,
        message,
        psychosis_triggered,
        overdose: false,
    }
}

/// Tick down substance effects (called each turn)
pub fn tick_effects(state: &mut GameState) {
    state.active_effects.retain_mut(|effect| {
        if effect.turns_remaining > 0 {
            effect.turns_remaining -= 1;
            true
        } else {
            false
        }
    });
}

/// Check for withdrawal symptoms (called when addicted and not using)
pub fn check_withdrawal(state: &mut GameState) -> Option<String> {
    let mut withdrawal_messages = Vec::new();

    for (substance_key, level) in state.addictions.iter() {
        if *level >= 5 {
            // Check if currently under effect of this substance
            let has_active = state.active_effects.iter()
                .any(|e| e.substance_key == *substance_key);

            if !has_active {
                // Withdrawal symptoms
                let severity = match *level {
                    5..=10 => "mild",
                    11..=20 => "moderate",
                    21..=35 => "severe",
                    _ => "crippling",
                };

                if let Some(substance) = super::data::get_substance(substance_key) {
                    withdrawal_messages.push(format!(
                        "{} {} withdrawal",
                        severity, substance.name
                    ));

                    // Apply withdrawal penalties
                    match *level {
                        5..=10 => {
                            // Mild: small stat penalty
                        }
                        11..=20 => {
                            // Moderate: noticeable penalty
                            state.mental_stability -= 2;
                        }
                        21..=35 => {
                            // Severe: significant penalty
                            state.mental_stability -= 5;
                            state.hp = state.hp.saturating_sub(5);
                        }
                        _ => {
                            // Crippling: major penalty
                            state.mental_stability -= 10;
                            state.hp = state.hp.saturating_sub(10);
                        }
                    }
                }
            }
        }
    }

    if withdrawal_messages.is_empty() {
        None
    } else {
        Some(format!("Withdrawal: {}", withdrawal_messages.join(", ")))
    }
}

/// Get substance display info for shop/inventory
pub fn get_substance_display(substance: &Substance) -> String {
    let mut parts = Vec::new();

    if substance.effects.strength_mod != 0 {
        parts.push(format!("STR{:+}", substance.effects.strength_mod));
    }
    if substance.effects.agility_mod != 0 {
        parts.push(format!("AGI{:+}", substance.effects.agility_mod));
    }
    if substance.effects.vitality_mod != 0 {
        parts.push(format!("VIT{:+}", substance.effects.vitality_mod));
    }
    if substance.effects.intelligence_mod != 0 {
        parts.push(format!("INT{:+}", substance.effects.intelligence_mod));
    }
    if substance.effects.damage_mod != 0 {
        parts.push(format!("DMG{:+}", substance.effects.damage_mod));
    }
    if substance.effects.defense_mod != 0 {
        parts.push(format!("DEF{:+}", substance.effects.defense_mod));
    }
    if substance.effects.healing > 0 {
        parts.push(format!("+{}HP", substance.effects.healing));
    }
    if substance.effects.action_bonus > 0 {
        parts.push(format!("+{}act", substance.effects.action_bonus));
    }
    if substance.effects.invincible_turns > 0 {
        parts.push("INVINCIBLE".to_string());
    }

    if parts.is_empty() {
        "No effect".to_string()
    } else {
        parts.join(" ")
    }
}

/// Get all substances by category
pub fn get_substances_by_category(category: SubstanceCategory) -> Vec<&'static Substance> {
    SUBSTANCES.iter()
        .filter(|s| s.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::CharacterClass;

    #[test]
    fn test_apply_substance() {
        let mut state = GameState::new("Test".to_string(), CharacterClass::Warrior);
        let initial_mental = state.mental_stability;

        let result = apply_substance(&mut state, "basic_steroid");
        assert!(result.success);
        assert!(state.mental_stability < initial_mental);
        assert!(!state.active_effects.is_empty());
    }

    #[test]
    fn test_healing_potion() {
        let mut state = GameState::new("Test".to_string(), CharacterClass::Warrior);
        state.hp = 50;

        let result = apply_substance(&mut state, "healing_potion");
        assert!(result.success);
        assert!(state.hp > 50);
    }

    #[test]
    fn test_overdose_prevention() {
        let mut state = GameState::new("Test".to_string(), CharacterClass::Warrior);

        // Apply multiple steroids
        apply_substance(&mut state, "basic_steroid");
        apply_substance(&mut state, "power_enhancer");
        let result = apply_substance(&mut state, "rage_inducer");

        assert!(result.overdose);
    }

    #[test]
    fn test_psychosis_trigger() {
        let mut state = GameState::new("Test".to_string(), CharacterClass::Warrior);
        state.mental_stability = 5;

        let result = apply_substance(&mut state, "void_essence");
        assert!(result.psychosis_triggered);
    }

    #[test]
    fn test_substance_display() {
        let substance = super::super::data::get_substance("basic_steroid").unwrap();
        let display = get_substance_display(substance);
        assert!(display.contains("STR"));
    }
}
