//! Economy and resource management for Cradle
//!
//! Handles resource generation, upgrades, and purchases.

use super::state::GameState;
use super::data::{TierLevel, get_tier, get_path, get_technique};

/// Resource gain result from a tick
#[derive(Debug, Clone, Default)]
pub struct ResourceGain {
    pub madra: u64,
    pub spirit_stones: u64,
    pub insight: u32,
    pub technique_xp: u64,
}

/// Calculate resources gained per tick
pub fn calculate_tick_resources(state: &GameState) -> ResourceGain {
    let mut gain = ResourceGain::default();

    // Base madra regen
    let base_madra = state.madra_per_tick();
    gain.madra = (base_madra as f64 * state.prestige.madra_multiplier) as u64;

    // Spirit stones based on tier and cycling
    let tier_level = state.tier as u64 + 1;
    let stone_rate = tier_level * 5;
    gain.spirit_stones = (stone_rate as f64 * state.prestige.spirit_stone_multiplier) as u64;

    // Insight based on cycling and meditation
    if state.primary_path.is_some() {
        let base_insight = (state.stats.cycling_speed * 10.0) as u32;
        gain.insight = ((base_insight as f64) * state.prestige.insight_multiplier) as u32;
    }

    // Technique XP from cycling techniques
    if !state.active_techniques.is_empty() {
        let cycle_power: u64 = state.active_techniques.iter()
            .filter_map(|t| get_technique(t))
            .filter(|t| t.technique_type == super::data::TechniqueType::Cycling)
            .map(|t| t.power_base)
            .sum();
        gain.technique_xp = cycle_power;
    }

    gain
}

/// Apply resource gains to state
pub fn apply_resource_gains(state: &mut GameState, gains: &ResourceGain) {
    // Add madra (cap at max)
    state.madra = (state.madra + gains.madra).min(state.max_madra);

    // Add spirit stones
    state.spirit_stones += gains.spirit_stones;

    // Add insight
    state.insight += gains.insight;

    // Apply technique XP to path mastery
    if let Some(ref path_key) = state.primary_path {
        let progress = state.path_progress.entry(path_key.clone()).or_default();
        progress.mastery += gains.technique_xp;
    }
}

/// Calculate tier advancement progress from current madra
pub fn calculate_tier_progress(state: &GameState) -> f64 {
    if let Some(next_tier) = state.tier.next() {
        if let Some(tier_data) = get_tier(next_tier) {
            let requirement = tier_data.madra_requirement;
            if requirement == 0 {
                return 1.0;
            }
            return (state.madra as f64 / requirement as f64).min(1.0);
        }
    }
    1.0  // At max tier
}

/// Attempt to advance to the next tier
pub fn try_advance_tier(state: &mut GameState) -> Result<TierLevel, &'static str> {
    if !state.can_advance_tier() {
        if let Some(next_tier) = state.tier.next() {
            if let Some(tier_data) = get_tier(next_tier) {
                if state.madra < tier_data.madra_requirement {
                    return Err("Insufficient madra for advancement");
                }
                if state.insight < tier_data.insight_requirement {
                    return Err("Insufficient insight for advancement");
                }
                if tier_data.trial_required && !state.trials_completed.contains(&next_tier) {
                    return Err("Trial required for advancement");
                }
            }
        }
        return Err("Cannot advance tier");
    }

    // Consume requirements
    if let Some(next_tier) = state.tier.next() {
        if let Some(tier_data) = get_tier(next_tier) {
            state.madra = state.madra.saturating_sub(tier_data.madra_requirement / 2);
            state.insight = state.insight.saturating_sub(tier_data.insight_requirement / 2);
        }

        // Advance tier
        let _old_tier = state.tier;
        state.tier = next_tier;
        state.tier_progress = 0.0;

        // Increase max madra
        if let Some(tier_data) = get_tier(next_tier) {
            state.max_madra = (state.max_madra as f64 * tier_data.madra_regen_bonus) as u64;
        }

        // Update stats
        if next_tier > state.stats.highest_tier_reached {
            state.stats.highest_tier_reached = next_tier;
        }
        if next_tier > state.prestige.highest_tier_ever {
            state.prestige.highest_tier_ever = next_tier;
        }

        // Unlock new features
        if let Some(tier_data) = get_tier(next_tier) {
            for unlock in tier_data.unlocks {
                if !state.unlocked_features.contains(&unlock.to_string()) {
                    state.unlocked_features.push(unlock.to_string());
                }
            }
        }

        // Update stats based on tier
        update_stats_for_tier(state);

        Ok(next_tier)
    } else {
        Err("Already at maximum tier")
    }
}

/// Update character stats after tier advancement
fn update_stats_for_tier(state: &mut GameState) {
    let tier_bonus = state.tier as u64 + 1;

    // Base stat increases per tier
    state.stats.power += 10 * tier_bonus;
    state.stats.defense += 8 * tier_bonus;
    state.stats.speed += 5 * tier_bonus;
    state.stats.madra_regen += tier_bonus;

    // Update peak power tracking
    let total_power = state.total_power();
    if total_power > state.stats.peak_power {
        state.stats.peak_power = total_power;
    }
}

/// Select a primary cultivation path
pub fn select_path(state: &mut GameState, path_key: &str) -> Result<(), &'static str> {
    let path = get_path(path_key).ok_or("Unknown path")?;

    // Check if we already have a primary path
    if state.primary_path.is_some() && state.tier >= TierLevel::Iron {
        return Err("Cannot change primary path after Iron tier without respec");
    }

    // Check compatibility with secondary path
    if let Some(ref secondary) = state.secondary_path {
        if path.incompatible_with.contains(&secondary.as_str()) {
            return Err("Incompatible with current secondary path");
        }
    }

    state.primary_path = Some(path_key.to_string());

    // Initialize path progress
    let progress = state.path_progress.entry(path_key.to_string()).or_default();
    progress.affinity = 0.5;  // Starting affinity
    progress.resonance = 1.0;

    // Unlock starting technique
    let techniques = super::data::get_techniques_for_path(path_key);
    if let Some(cycling) = techniques.iter().find(|t| t.technique_type == super::data::TechniqueType::Cycling) {
        if !progress.techniques_unlocked.contains(&cycling.key.to_string()) {
            progress.techniques_unlocked.push(cycling.key.to_string());
            state.active_techniques.push(cycling.key.to_string());
        }
    }

    Ok(())
}

/// Select a secondary path
pub fn select_secondary_path(state: &mut GameState, path_key: &str) -> Result<(), &'static str> {
    // Must have a primary path first
    if state.primary_path.is_none() {
        return Err("Must select primary path first");
    }

    // Must be at least Gold tier
    if state.tier < TierLevel::Gold {
        return Err("Must be Gold tier or higher for secondary path");
    }

    let _path = get_path(path_key).ok_or("Unknown path")?;
    let primary_key = state.primary_path.as_ref().unwrap();
    let primary = get_path(primary_key).ok_or("Invalid primary path")?;

    // Check compatibility
    if primary.incompatible_with.contains(&path_key) {
        return Err("Incompatible with primary path - will cause plateau!");
    }

    // Warn if not in compatible list (reduced effectiveness)
    let is_compatible = primary.compatible_with.contains(&path_key);

    state.secondary_path = Some(path_key.to_string());

    // Initialize with reduced affinity if not fully compatible
    let progress = state.path_progress.entry(path_key.to_string()).or_default();
    progress.affinity = if is_compatible { 0.4 } else { 0.2 };
    progress.resonance = if is_compatible { 0.8 } else { 0.5 };

    Ok(())
}

/// Purchase a technique with spirit stones
pub fn purchase_technique(state: &mut GameState, technique_key: &str) -> Result<(), &'static str> {
    let technique = get_technique(technique_key).ok_or("Unknown technique")?;

    // Check if technique is for player's path
    let owns_path = state.primary_path.as_ref().map(|p| p == technique.path_key).unwrap_or(false)
        || state.secondary_path.as_ref().map(|p| p == technique.path_key).unwrap_or(false);

    if !owns_path {
        return Err("Technique not available for your path");
    }

    // Check tier requirement
    if state.tier < technique.tier_requirement {
        return Err("Tier too low for this technique");
    }

    // Calculate cost
    let base_cost = 100u64 * (technique.tier_requirement as u64 + 1);
    let power_cost = technique.power_base / 10;
    let total_cost = base_cost + power_cost;

    if state.spirit_stones < total_cost {
        return Err("Insufficient spirit stones");
    }

    // Purchase
    state.spirit_stones -= total_cost;

    // Add to path progress
    if let Some(progress) = state.path_progress.get_mut(technique.path_key) {
        if !progress.techniques_unlocked.contains(&technique_key.to_string()) {
            progress.techniques_unlocked.push(technique_key.to_string());
        }
    }

    // Set initial level
    state.technique_levels.insert(technique_key.to_string(), 1);

    Ok(())
}

/// Upgrade a technique level
pub fn upgrade_technique(state: &mut GameState, technique_key: &str) -> Result<u32, &'static str> {
    let current_level = state.technique_levels.get(technique_key).copied().unwrap_or(0);
    if current_level == 0 {
        return Err("Technique not learned");
    }

    // Max level is tier + 5
    let max_level = (state.tier as u32) + 5;
    if current_level >= max_level {
        return Err("Technique at maximum level for your tier");
    }

    // Calculate upgrade cost
    let cost = (current_level as u64 + 1) * 50;
    if state.spirit_stones < cost {
        return Err("Insufficient spirit stones");
    }

    state.spirit_stones -= cost;
    let new_level = current_level + 1;
    state.technique_levels.insert(technique_key.to_string(), new_level);

    Ok(new_level)
}

/// Consume an elixir for temporary boost
pub fn use_elixir(state: &mut GameState) -> Result<&'static str, &'static str> {
    if state.elixirs == 0 {
        return Err("No elixirs available");
    }

    state.elixirs -= 1;

    // Boost madra to max
    state.madra = state.max_madra;

    // Temporary stat boost (not persistent, just flavor)
    state.stats.madra_regen += 10;

    Ok("Elixir consumed! Madra fully restored, regeneration boosted.")
}

/// Purchase elixirs with spirit stones
pub fn purchase_elixirs(state: &mut GameState, count: u32) -> Result<(), &'static str> {
    let cost_per = 500u64;
    let total_cost = cost_per * (count as u64);

    if state.spirit_stones < total_cost {
        return Err("Insufficient spirit stones");
    }

    state.spirit_stones -= total_cost;
    state.elixirs += count;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_calculation() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Copper;
        state.stats.madra_regen = 10;

        let gains = calculate_tick_resources(&state);
        assert!(gains.madra > 0);
        assert!(gains.spirit_stones > 0);
    }

    #[test]
    fn test_tier_advancement() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Unsouled;
        state.madra = 200;
        state.insight = 10;

        let result = try_advance_tier(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.tier, TierLevel::Copper);
    }

    #[test]
    fn test_path_selection() {
        let mut state = GameState::new("Test".to_string());

        let result = select_path(&mut state, "blackflame");
        assert!(result.is_ok());
        assert_eq!(state.primary_path, Some("blackflame".to_string()));
        assert!(!state.active_techniques.is_empty());
    }

    #[test]
    fn test_secondary_path_requires_gold() {
        let mut state = GameState::new("Test".to_string());
        state.primary_path = Some("blackflame".to_string());
        state.tier = TierLevel::Jade;

        let result = select_secondary_path(&mut state, "void_walker");
        assert!(result.is_err());  // Must be Gold tier
    }

    #[test]
    fn test_incompatible_paths() {
        let mut state = GameState::new("Test".to_string());
        state.primary_path = Some("blackflame".to_string());
        state.tier = TierLevel::Gold;

        let result = select_secondary_path(&mut state, "life_weaver");
        assert!(result.is_err());  // Blackflame incompatible with life_weaver
    }

    #[test]
    fn test_elixir_purchase_and_use() {
        let mut state = GameState::new("Test".to_string());
        state.spirit_stones = 1000;

        purchase_elixirs(&mut state, 2).unwrap();
        assert_eq!(state.elixirs, 2);

        state.madra = 0;
        use_elixir(&mut state).unwrap();
        assert_eq!(state.madra, state.max_madra);
        assert_eq!(state.elixirs, 1);
    }

    #[test]
    fn test_tier_progress_calculation() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Copper;
        state.madra = 500;  // 50% of Iron requirement (1000)

        let progress = calculate_tier_progress(&state);
        assert!(progress > 0.4 && progress < 0.6);
    }
}
