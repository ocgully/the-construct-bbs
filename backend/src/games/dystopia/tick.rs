//! Tick system - processes resource generation and game updates
//!
//! Implements hybrid tick system:
//! - Background jobs for active games
//! - Catchup calculation for inactive players

use super::state::ProvinceState;
use super::data::{BuildingType, get_race, get_personality};

/// Result of processing ticks
#[derive(Debug, Clone)]
pub struct TickResult {
    pub ticks_processed: u32,
    pub gold_earned: i64,
    pub food_produced: i64,
    pub food_consumed: i64,
    pub runes_produced: i64,
    pub military_upkeep: i64,
    pub peasants_born: u32,
    pub peasants_starved: u32,
    pub buildings_completed: u32,
    pub units_trained: u32,
    pub research_completed: Option<String>,
    pub protection_expired: bool,
}

impl Default for TickResult {
    fn default() -> Self {
        Self {
            ticks_processed: 0,
            gold_earned: 0,
            food_produced: 0,
            food_consumed: 0,
            runes_produced: 0,
            military_upkeep: 0,
            peasants_born: 0,
            peasants_starved: 0,
            buildings_completed: 0,
            units_trained: 0,
            research_completed: None,
            protection_expired: false,
        }
    }
}

/// Process a single tick for a province
pub fn process_tick(state: &mut ProvinceState) -> TickResult {
    let mut result = TickResult::default();
    result.ticks_processed = 1;

    // 1. Complete buildings under construction
    result.buildings_completed = complete_construction(state);

    // 2. Complete military training
    result.units_trained = complete_training(state);

    // 3. Resource production
    let gold_income = state.gold_income();
    let food_production = state.food_production();
    let food_consumption = state.food_consumption();
    let rune_production = state.rune_production();
    let military_upkeep = state.military_upkeep();

    // Apply prosperity effect if active
    let prosperity_bonus = get_effect_magnitude(state, "prosperity");
    let adjusted_gold = gold_income + (gold_income * prosperity_bonus as i64) / 100;

    // Apply income
    state.resources.gold += adjusted_gold - military_upkeep;
    state.resources.food += food_production - food_consumption;
    state.resources.runes += rune_production;

    result.gold_earned = adjusted_gold;
    result.food_produced = food_production;
    result.food_consumed = food_consumption;
    result.runes_produced = rune_production;
    result.military_upkeep = military_upkeep;

    // Track statistics
    state.stats.gold_earned += adjusted_gold;

    // 4. Population growth/starvation
    if state.resources.food < 0 {
        // Starvation!
        let starved = ((-state.resources.food) / 10).min(state.peasants as i64) as u32;
        state.peasants = state.peasants.saturating_sub(starved);
        result.peasants_starved = starved;
        state.resources.food = 0;
    } else {
        // Population growth
        let growth_rate = get_race(&state.race)
            .map(|r| r.pop_growth as i32)
            .unwrap_or(100);

        // Base growth: 1% per tick
        let base_growth = state.peasants as i32 / 100;
        let bonus = (base_growth * (growth_rate - 100)) / 100;
        let growth = (base_growth + bonus).max(0) as u32;

        // Cap at max peasants
        let new_total = state.peasants + growth;
        if new_total <= state.max_peasants {
            state.peasants = new_total;
            result.peasants_born = growth;
        } else if state.peasants < state.max_peasants {
            let actual_growth = state.max_peasants - state.peasants;
            state.peasants = state.max_peasants;
            result.peasants_born = actual_growth;
        }
    }

    // 5. Process research
    if let Some(ref research) = state.sciences.current_research.clone() {
        // Research progress based on universities
        let universities = state.get_building(BuildingType::University);
        let base_progress = 5 + universities;

        // Apply sage personality bonus
        let bonus = get_personality(&state.personality)
            .map(|p| (base_progress * (p.science_speed - 100)) / 100)
            .unwrap_or(0);

        state.sciences.research_progress += base_progress + bonus;

        // Complete at 100 progress
        if state.sciences.research_progress >= 100 {
            match research.as_str() {
                "alchemy" => state.sciences.alchemy += 1,
                "tools" => state.sciences.tools += 1,
                "housing" => {
                    state.sciences.housing += 1;
                    state.recalculate_max_peasants();
                }
                "food" => state.sciences.food += 1,
                "military" => state.sciences.military += 1,
                "crime" => state.sciences.crime += 1,
                "channeling" => state.sciences.channeling += 1,
                _ => {}
            }
            result.research_completed = Some(research.clone());
            state.sciences.current_research = None;
            state.sciences.research_progress = 0;
        }
    }

    // 6. Process protection timer
    if state.protection_ticks > 0 {
        state.protection_ticks -= 1;
        if state.protection_ticks == 0 {
            result.protection_expired = true;
        }
    }

    // 7. Process active effects (decay duration)
    state.effects.retain_mut(|effect| {
        if effect.remaining_ticks > 0 {
            effect.remaining_ticks -= 1;
            effect.remaining_ticks > 0
        } else {
            false
        }
    });

    // 8. Return deployed troops (simplified - instant return)
    state.military.soldiers += state.military.deployed_soldiers;
    state.military.archers += state.military.deployed_archers;
    state.military.knights += state.military.deployed_knights;
    state.military.deployed_soldiers = 0;
    state.military.deployed_archers = 0;
    state.military.deployed_knights = 0;

    // 9. Update networth tracking
    let current_nw = state.networth();
    if current_nw > state.stats.peak_networth {
        state.stats.peak_networth = current_nw;
    }

    // 10. Update tick counter
    state.last_tick += 1;

    result
}

/// Process multiple ticks (catchup for inactive players)
pub fn process_catchup_ticks(state: &mut ProvinceState, ticks: u32) -> TickResult {
    let mut combined = TickResult::default();

    for _ in 0..ticks {
        let tick_result = process_tick(state);

        combined.ticks_processed += 1;
        combined.gold_earned += tick_result.gold_earned;
        combined.food_produced += tick_result.food_produced;
        combined.food_consumed += tick_result.food_consumed;
        combined.runes_produced += tick_result.runes_produced;
        combined.military_upkeep += tick_result.military_upkeep;
        combined.peasants_born += tick_result.peasants_born;
        combined.peasants_starved += tick_result.peasants_starved;
        combined.buildings_completed += tick_result.buildings_completed;
        combined.units_trained += tick_result.units_trained;

        if tick_result.research_completed.is_some() {
            combined.research_completed = tick_result.research_completed;
        }

        if tick_result.protection_expired {
            combined.protection_expired = true;
        }
    }

    combined
}

/// Complete buildings under construction
fn complete_construction(state: &mut ProvinceState) -> u32 {
    let mut completed = 0u32;

    // Move from under_construction to counts
    let constructions: Vec<(String, u32)> = state.buildings.under_construction
        .drain()
        .collect();

    for (building, count) in constructions {
        *state.buildings.counts.entry(building).or_insert(0) += count;
        completed += count;
    }

    // Recalculate max peasants if homes were built
    if completed > 0 {
        state.recalculate_max_peasants();
    }

    completed
}

/// Complete military training
fn complete_training(state: &mut ProvinceState) -> u32 {
    let mut trained = 0u32;

    // Apply haste effect
    let haste_bonus = get_effect_magnitude(state, "haste");
    let multiplier = 100 + haste_bonus as u32;

    // Move from training to active
    let soldiers = (state.military.training_soldiers * multiplier) / 100;
    let archers = (state.military.training_archers * multiplier) / 100;
    let knights = (state.military.training_knights * multiplier) / 100;
    let thieves = (state.military.training_thieves * multiplier) / 100;
    let wizards = (state.military.training_wizards * multiplier) / 100;
    let elites = (state.military.training_elites * multiplier) / 100;

    state.military.soldiers += soldiers;
    state.military.archers += archers;
    state.military.knights += knights;
    state.military.thieves += thieves;
    state.military.wizards += wizards;
    state.military.elites += elites;

    trained += soldiers + archers + knights + thieves + wizards + elites;

    // Clear training queues
    state.military.training_soldiers = 0;
    state.military.training_archers = 0;
    state.military.training_knights = 0;
    state.military.training_thieves = 0;
    state.military.training_wizards = 0;
    state.military.training_elites = 0;

    trained
}

/// Get the magnitude of an active effect
fn get_effect_magnitude(state: &ProvinceState, effect_type: &str) -> i32 {
    state.effects
        .iter()
        .find(|e| e.effect_type == effect_type)
        .map(|e| e.magnitude)
        .unwrap_or(0)
}

/// Calculate ticks since last update
pub fn ticks_since_last_update(state: &ProvinceState, current_tick: i64) -> u32 {
    if current_tick > state.last_tick {
        (current_tick - state.last_tick) as u32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tick() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        );

        let initial_gold = state.resources.gold;
        let result = process_tick(&mut state);

        assert_eq!(result.ticks_processed, 1);
        assert!(result.gold_earned > 0);
        // Gold should increase (income > upkeep for starting province)
        assert!(state.resources.gold > initial_gold);
    }

    #[test]
    fn test_catchup_ticks() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "dwarf".to_string(),
            "merchant".to_string(),
        );

        let initial_tick = state.last_tick;
        let result = process_catchup_ticks(&mut state, 10);

        assert_eq!(result.ticks_processed, 10);
        assert_eq!(state.last_tick, initial_tick + 10);
    }

    #[test]
    fn test_building_completion() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        );

        // Add buildings under construction
        state.buildings.under_construction.insert("Farm".to_string(), 5);

        let initial_farms = state.get_building(BuildingType::Farm);
        let result = process_tick(&mut state);

        assert_eq!(result.buildings_completed, 5);
        assert_eq!(state.get_building(BuildingType::Farm), initial_farms + 5);
    }

    #[test]
    fn test_training_completion() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "orc".to_string(),
            "warrior".to_string(),
        );

        // Add units in training
        state.military.training_soldiers = 20;
        let initial_soldiers = state.military.soldiers;

        let result = process_tick(&mut state);

        assert!(result.units_trained > 0);
        assert!(state.military.soldiers > initial_soldiers);
        assert_eq!(state.military.training_soldiers, 0);
    }

    #[test]
    fn test_population_growth() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "halfling".to_string(), // Best population growth
            "merchant".to_string(),
        );

        let initial_peasants = state.peasants;
        state.resources.food = 10000; // Plenty of food

        process_tick(&mut state);

        // Should have grown
        assert!(state.peasants >= initial_peasants);
    }

    #[test]
    fn test_starvation() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "warrior".to_string(),
        );

        // Set up starvation condition
        state.resources.food = -1000;
        let initial_peasants = state.peasants;

        let result = process_tick(&mut state);

        assert!(result.peasants_starved > 0);
        assert!(state.peasants < initial_peasants);
        assert_eq!(state.resources.food, 0);
    }

    #[test]
    fn test_protection_timer() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        );

        state.protection_ticks = 1;
        let result = process_tick(&mut state);

        assert!(result.protection_expired);
        assert_eq!(state.protection_ticks, 0);
    }

    #[test]
    fn test_undead_no_food_consumption() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "undead".to_string(),
            "mystic".to_string(),
        );

        let initial_food = state.resources.food;
        let result = process_tick(&mut state);

        // Undead don't consume food, only produce
        assert_eq!(result.food_consumed, 0);
        assert!(state.resources.food >= initial_food);
    }
}
