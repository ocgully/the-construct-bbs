//! AI player logic for Master of Cygnus
//!
//! Handles AI-controlled empires, including takeover of timed-out players.

use serde::{Deserialize, Serialize};
use rand::Rng;
use super::state::{EmpireState, TurnOrders, ColonyOrders, FleetOrders};
use super::galaxy::Galaxy;
use super::tech::TechField;

/// AI difficulty/personality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiPersonality {
    /// Focuses on expansion
    Expansionist,
    /// Focuses on technology
    Technologist,
    /// Focuses on military
    Militarist,
    /// Balanced approach
    Balanced,
}

impl AiPersonality {
    /// Choose a random personality
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => AiPersonality::Expansionist,
            1 => AiPersonality::Technologist,
            2 => AiPersonality::Militarist,
            _ => AiPersonality::Balanced,
        }
    }
}

/// AI controller for an empire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiController {
    pub empire_id: u32,
    pub personality: AiPersonality,
    /// Target star for expansion
    pub expansion_target: Option<u32>,
    /// Star to attack
    pub attack_target: Option<u32>,
}

impl AiController {
    pub fn new(empire_id: u32) -> Self {
        AiController {
            empire_id,
            personality: AiPersonality::random(),
            expansion_target: None,
            attack_target: None,
        }
    }

    /// Generate orders for this AI empire
    pub fn generate_orders(
        &mut self,
        empire: &EmpireState,
        galaxy: &Galaxy,
        _all_empires: &[EmpireState],
    ) -> TurnOrders {
        let mut orders = TurnOrders {
            empire_id: self.empire_id,
            colony_orders: Vec::new(),
            fleet_orders: Vec::new(),
            research_allocation: empire.research.allocation.clone(),
            submitted: true,
            ai_generated: true,
        };

        // Generate colony orders
        for colony in &empire.colonies {
            let colony_orders = self.generate_colony_orders(colony.star_id, empire);
            orders.colony_orders.push(colony_orders);
        }

        // Generate fleet orders
        for fleet in &empire.fleets {
            if let Some(fleet_orders) = self.generate_fleet_orders(fleet.id, empire, galaxy) {
                orders.fleet_orders.push(fleet_orders);
            }
        }

        // Adjust research allocation based on personality
        self.adjust_research_allocation(&mut orders);

        orders
    }

    /// Generate orders for a colony
    fn generate_colony_orders(&self, star_id: u32, empire: &EmpireState) -> ColonyOrders {
        let mut rng = rand::thread_rng();
        let mut build_queue = Vec::new();

        // Find the colony
        let colony = empire.colonies.iter().find(|c| c.star_id == star_id);

        if let Some(col) = colony {
            // Priority based on personality and current state
            let needs_shipyard = !col.buildings.iter().any(|b| b == "Shipyard");
            let needs_factory = col.buildings.iter().filter(|b| *b == "Factory").count() < 3;
            let needs_lab = col.buildings.iter().filter(|b| *b == "ResearchLab").count() < 2;

            match self.personality {
                AiPersonality::Expansionist => {
                    if needs_shipyard {
                        build_queue.push("Shipyard".to_string());
                    } else if rng.gen_bool(0.5) {
                        build_queue.push("Colony Ship".to_string());
                    }
                    if needs_factory {
                        build_queue.push("Factory".to_string());
                    }
                }
                AiPersonality::Technologist => {
                    if needs_lab {
                        build_queue.push("ResearchLab".to_string());
                    }
                    if needs_factory {
                        build_queue.push("Factory".to_string());
                    }
                }
                AiPersonality::Militarist => {
                    if needs_shipyard {
                        build_queue.push("Shipyard".to_string());
                    } else {
                        build_queue.push("Fighter".to_string());
                    }
                }
                AiPersonality::Balanced => {
                    if needs_factory {
                        build_queue.push("Factory".to_string());
                    } else if needs_shipyard {
                        build_queue.push("Shipyard".to_string());
                    } else if needs_lab {
                        build_queue.push("ResearchLab".to_string());
                    }
                }
            }
        }

        ColonyOrders {
            star_id,
            build_queue,
            population_transfer: None,
        }
    }

    /// Generate orders for a fleet
    fn generate_fleet_orders(
        &mut self,
        fleet_id: u32,
        empire: &EmpireState,
        galaxy: &Galaxy,
    ) -> Option<FleetOrders> {
        let fleet = empire.fleets.iter().find(|f| f.id == fleet_id)?;

        // Don't give new orders to fleets in transit
        if fleet.is_in_transit() {
            return None;
        }

        let mut rng = rand::thread_rng();

        // Check for nearby uncolonized stars
        let uncolonized_nearby: Vec<_> = galaxy.stars_in_range(fleet.location_star_id, 10.0)
            .into_iter()
            .filter(|s| s.owner.is_none())
            .collect();

        // Check for nearby enemy stars
        let enemy_nearby: Vec<_> = galaxy.stars_in_range(fleet.location_star_id, 10.0)
            .into_iter()
            .filter(|s| s.owner.is_some() && s.owner != Some(self.empire_id))
            .collect();

        let destination = match self.personality {
            AiPersonality::Expansionist | AiPersonality::Balanced => {
                // Prioritize expansion
                if !uncolonized_nearby.is_empty() && rng.gen_bool(0.7) {
                    Some(uncolonized_nearby[rng.gen_range(0..uncolonized_nearby.len())].id)
                } else if !enemy_nearby.is_empty() && rng.gen_bool(0.3) {
                    Some(enemy_nearby[rng.gen_range(0..enemy_nearby.len())].id)
                } else {
                    None
                }
            }
            AiPersonality::Militarist => {
                // Prioritize combat
                if !enemy_nearby.is_empty() && rng.gen_bool(0.7) {
                    Some(enemy_nearby[rng.gen_range(0..enemy_nearby.len())].id)
                } else if !uncolonized_nearby.is_empty() {
                    Some(uncolonized_nearby[rng.gen_range(0..uncolonized_nearby.len())].id)
                } else {
                    None
                }
            }
            AiPersonality::Technologist => {
                // Stay close to home, only expand occasionally
                if !uncolonized_nearby.is_empty() && rng.gen_bool(0.4) {
                    Some(uncolonized_nearby[rng.gen_range(0..uncolonized_nearby.len())].id)
                } else {
                    None
                }
            }
        };

        destination.map(|dest| FleetOrders {
            fleet_id,
            destination: Some(dest),
            colonize: true, // Will only work if colony ship present
        })
    }

    /// Adjust research allocation based on personality
    fn adjust_research_allocation(&self, orders: &mut TurnOrders) {
        match self.personality {
            AiPersonality::Expansionist => {
                orders.research_allocation.insert(TechField::Propulsion, 30);
                orders.research_allocation.insert(TechField::Planetology, 25);
                orders.research_allocation.insert(TechField::Construction, 20);
                orders.research_allocation.insert(TechField::Weapons, 10);
                orders.research_allocation.insert(TechField::Shields, 10);
                orders.research_allocation.insert(TechField::Computers, 5);
            }
            AiPersonality::Technologist => {
                orders.research_allocation.insert(TechField::Computers, 30);
                orders.research_allocation.insert(TechField::Planetology, 20);
                orders.research_allocation.insert(TechField::Construction, 20);
                orders.research_allocation.insert(TechField::Propulsion, 15);
                orders.research_allocation.insert(TechField::Weapons, 10);
                orders.research_allocation.insert(TechField::Shields, 5);
            }
            AiPersonality::Militarist => {
                orders.research_allocation.insert(TechField::Weapons, 30);
                orders.research_allocation.insert(TechField::Shields, 25);
                orders.research_allocation.insert(TechField::Computers, 20);
                orders.research_allocation.insert(TechField::Propulsion, 15);
                orders.research_allocation.insert(TechField::Construction, 5);
                orders.research_allocation.insert(TechField::Planetology, 5);
            }
            AiPersonality::Balanced => {
                // Keep default even distribution
                for field in TechField::all() {
                    orders.research_allocation.insert(field, 16);
                }
            }
        }
    }
}

/// Check if an empire has timed out and should be taken over by AI
pub fn check_timeout(
    last_activity_turn: u32,
    current_turn: u32,
    timeout_hours: u32,
    hours_per_turn: u32,
) -> bool {
    let turns_since_activity = current_turn.saturating_sub(last_activity_turn);
    let hours_since_activity = turns_since_activity * hours_per_turn;
    hours_since_activity >= timeout_hours
}

/// Create AI takeover notification message
pub fn ai_takeover_message(empire_name: &str, reason: &str) -> String {
    format!(
        "Empire '{}' has been taken over by AI control. Reason: {}",
        empire_name, reason
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_personality_random() {
        // Just make sure it doesn't panic
        for _ in 0..10 {
            let _ = AiPersonality::random();
        }
    }

    #[test]
    fn test_timeout_check() {
        // 72 hour timeout with 1 hour per turn
        assert!(!check_timeout(0, 10, 72, 1)); // Only 10 hours
        assert!(!check_timeout(0, 71, 72, 1)); // 71 hours, still safe
        assert!(check_timeout(0, 72, 72, 1));  // Exactly 72 hours
        assert!(check_timeout(0, 100, 72, 1)); // Way over
    }

    #[test]
    fn test_ai_controller_creation() {
        let ai = AiController::new(0);
        assert_eq!(ai.empire_id, 0);
        assert!(ai.expansion_target.is_none());
    }

    #[test]
    fn test_ai_takeover_message() {
        let msg = ai_takeover_message("Terran Federation", "Player timeout after 72 hours");
        assert!(msg.contains("Terran Federation"));
        assert!(msg.contains("72 hours"));
    }
}
