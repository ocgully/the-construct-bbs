//! Economy and resource management

use super::state::ProvinceState;
use super::data::{BuildingType, UnitType, get_personality};

/// Result of an economic action
#[derive(Debug)]
#[allow(dead_code)] // Variants with fields reserved for future use/error reporting
pub enum EconomyResult {
    Success { message: String },
    InsufficientGold { needed: i64, have: i64 },
    InsufficientFood { needed: i64, have: i64 },
    InsufficientRunes { needed: i64, have: i64 },
    InsufficientLand { needed: u32, have: u32 },
    InsufficientPopulation { needed: u32, have: u32 },
    MaxCapacityReached { current: u32, max: u32 },
    InvalidAmount,
}

/// Build buildings in the province
pub fn build(state: &mut ProvinceState, building: BuildingType, count: u32) -> EconomyResult {
    if count == 0 {
        return EconomyResult::InvalidAmount;
    }

    // Calculate cost
    let unit_cost = state.building_cost(building) as i64;
    let total_cost = unit_cost * (count as i64);

    // Check gold
    if state.resources.gold < total_cost {
        return EconomyResult::InsufficientGold {
            needed: total_cost,
            have: state.resources.gold,
        };
    }

    // Check available land (each building uses 1 land)
    let used_land = total_buildings(state);
    let available = state.land.saturating_sub(used_land);
    if available < count {
        return EconomyResult::InsufficientLand {
            needed: count,
            have: available,
        };
    }

    // Deduct cost and add to construction queue
    state.resources.gold -= total_cost;
    state.stats.gold_spent += total_cost;

    *state.buildings.under_construction
        .entry(building.name().to_string())
        .or_insert(0) += count;

    EconomyResult::Success {
        message: format!(
            "Started construction of {} {}. Will complete next tick.",
            count, building.name()
        ),
    }
}

/// Train military units
pub fn train_units(state: &mut ProvinceState, unit: UnitType, count: u32) -> EconomyResult {
    if count == 0 {
        return EconomyResult::InvalidAmount;
    }

    // Calculate cost
    let unit_cost = unit.cost() as i64;
    let total_cost = unit_cost * (count as i64);

    // Check gold
    if state.resources.gold < total_cost {
        return EconomyResult::InsufficientGold {
            needed: total_cost,
            have: state.resources.gold,
        };
    }

    // Check population (peasants become soldiers)
    if state.peasants < count {
        return EconomyResult::InsufficientPopulation {
            needed: count,
            have: state.peasants,
        };
    }

    // Specialists require guildhalls
    if matches!(unit, UnitType::Thief | UnitType::Wizard | UnitType::Elite) {
        let guildhalls = state.get_building(BuildingType::Guildhall);
        let specialist_cap = guildhalls * 50;
        let current_specialists = state.military.thieves + state.military.wizards + state.military.elites;
        if current_specialists + count > specialist_cap {
            return EconomyResult::MaxCapacityReached {
                current: current_specialists,
                max: specialist_cap,
            };
        }
    }

    // Deduct cost and add to training queue
    state.resources.gold -= total_cost;
    state.stats.gold_spent += total_cost;
    state.peasants -= count;

    // Apply personality training speed bonus
    let _bonus = get_personality(&state.personality)
        .map(|p| p.train_speed)
        .unwrap_or(100);

    // Add to training queue (will be ready next tick)
    match unit {
        UnitType::Soldier => state.military.training_soldiers += count,
        UnitType::Archer => state.military.training_archers += count,
        UnitType::Knight => state.military.training_knights += count,
        UnitType::Thief => state.military.training_thieves += count,
        UnitType::Wizard => state.military.training_wizards += count,
        UnitType::Elite => state.military.training_elites += count,
    }

    EconomyResult::Success {
        message: format!(
            "Training {} {}. Will be ready next tick.",
            count, unit.name()
        ),
    }
}

/// Explore for new land
pub fn explore(state: &mut ProvinceState, explorers: u32) -> EconomyResult {
    if explorers == 0 {
        return EconomyResult::InvalidAmount;
    }

    // Cost: 50 gold per explorer
    let cost = (explorers as i64) * 50;

    if state.resources.gold < cost {
        return EconomyResult::InsufficientGold {
            needed: cost,
            have: state.resources.gold,
        };
    }

    // Need soldiers to explore
    if state.military.soldiers < explorers {
        return EconomyResult::InsufficientPopulation {
            needed: explorers,
            have: state.military.soldiers,
        };
    }

    // Deduct cost
    state.resources.gold -= cost;
    state.stats.gold_spent += cost;

    // Calculate land gained (base + personality bonus)
    let base_land = explorers / 10 + 1;
    let bonus = get_personality(&state.personality)
        .map(|p| (base_land * (p.explore_bonus - 100)) / 100)
        .unwrap_or(0);

    let land_gained = base_land + bonus;
    state.land += land_gained;

    // Some soldiers don't return
    let losses = explorers / 20;
    state.military.soldiers -= losses;

    EconomyResult::Success {
        message: format!(
            "Exploration complete! Gained {} acres. {} soldiers lost.",
            land_gained, losses
        ),
    }
}

/// Release prisoners for gold or convert to peasants
#[allow(dead_code)] // Reserved for future UI implementation
pub fn release_prisoners(state: &mut ProvinceState, count: u32, convert_to_peasants: bool) -> EconomyResult {
    if count == 0 || state.resources.prisoners < count {
        return EconomyResult::InvalidAmount;
    }

    state.resources.prisoners -= count;

    if convert_to_peasants {
        state.peasants += count;
        EconomyResult::Success {
            message: format!("{} prisoners converted to peasants.", count),
        }
    } else {
        // Ransom for gold
        let gold = (count as i64) * 100;
        state.resources.gold += gold;
        state.stats.gold_earned += gold;
        EconomyResult::Success {
            message: format!("{} prisoners ransomed for {} gold.", count, gold),
        }
    }
}

/// Start researching a science
pub fn start_research(state: &mut ProvinceState, science: &str) -> EconomyResult {
    // Cost: 1000 gold per level
    let current_level = match science {
        "alchemy" => state.sciences.alchemy,
        "tools" => state.sciences.tools,
        "housing" => state.sciences.housing,
        "food" => state.sciences.food,
        "military" => state.sciences.military,
        "crime" => state.sciences.crime,
        "channeling" => state.sciences.channeling,
        _ => return EconomyResult::InvalidAmount,
    };

    let cost = ((current_level + 1) as i64) * 1000;

    if state.resources.gold < cost {
        return EconomyResult::InsufficientGold {
            needed: cost,
            have: state.resources.gold,
        };
    }

    state.resources.gold -= cost;
    state.stats.gold_spent += cost;
    state.sciences.current_research = Some(science.to_string());
    state.sciences.research_progress = 0;

    EconomyResult::Success {
        message: format!("Started researching {}.", science),
    }
}

/// Count total buildings
fn total_buildings(state: &ProvinceState) -> u32 {
    state.buildings.counts.values().sum::<u32>()
        + state.buildings.under_construction.values().sum::<u32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_success() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        );

        let initial_gold = state.resources.gold;
        let result = build(&mut state, BuildingType::Farm, 1);

        assert!(matches!(result, EconomyResult::Success { .. }));
        assert!(state.resources.gold < initial_gold);
    }

    #[test]
    fn test_build_insufficient_gold() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        );
        state.resources.gold = 0;

        let result = build(&mut state, BuildingType::Bank, 1);

        assert!(matches!(result, EconomyResult::InsufficientGold { .. }));
    }

    #[test]
    fn test_train_units() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "warrior".to_string(),
        );

        let initial_peasants = state.peasants;
        let result = train_units(&mut state, UnitType::Soldier, 10);

        assert!(matches!(result, EconomyResult::Success { .. }));
        assert_eq!(state.peasants, initial_peasants - 10);
        assert_eq!(state.military.training_soldiers, 10);
    }

    #[test]
    fn test_explore() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "rogue".to_string(), // Explore bonus
        );

        let initial_land = state.land;
        let result = explore(&mut state, 50);

        assert!(matches!(result, EconomyResult::Success { .. }));
        assert!(state.land > initial_land);
    }

    #[test]
    fn test_research() {
        let mut state = ProvinceState::new(
            "Test".to_string(),
            "elf".to_string(),
            "sage".to_string(),
        );

        let result = start_research(&mut state, "alchemy");

        assert!(matches!(result, EconomyResult::Success { .. }));
        assert_eq!(state.sciences.current_research, Some("alchemy".to_string()));
    }
}
