//! Ship design and fleet management for Master of Andromeda

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ship hull sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HullSize {
    Scout,
    Destroyer,
    Cruiser,
    Battleship,
    Dreadnought,
}

impl HullSize {
    pub fn name(&self) -> &'static str {
        match self {
            HullSize::Scout => "Scout",
            HullSize::Destroyer => "Destroyer",
            HullSize::Cruiser => "Cruiser",
            HullSize::Battleship => "Battleship",
            HullSize::Dreadnought => "Dreadnought",
        }
    }

    /// Available component space
    pub fn space(&self) -> u32 {
        match self {
            HullSize::Scout => 10,
            HullSize::Destroyer => 25,
            HullSize::Cruiser => 50,
            HullSize::Battleship => 100,
            HullSize::Dreadnought => 200,
        }
    }

    /// Base production cost
    pub fn cost(&self) -> u32 {
        match self {
            HullSize::Scout => 20,
            HullSize::Destroyer => 50,
            HullSize::Cruiser => 120,
            HullSize::Battleship => 300,
            HullSize::Dreadnought => 600,
        }
    }

    /// Base hull hit points
    pub fn base_hp(&self) -> u32 {
        match self {
            HullSize::Scout => 10,
            HullSize::Destroyer => 25,
            HullSize::Cruiser => 60,
            HullSize::Battleship => 150,
            HullSize::Dreadnought => 350,
        }
    }

    /// Base speed
    pub fn base_speed(&self) -> u32 {
        match self {
            HullSize::Scout => 5,
            HullSize::Destroyer => 4,
            HullSize::Cruiser => 3,
            HullSize::Battleship => 2,
            HullSize::Dreadnought => 1,
        }
    }

    /// Required tech level to build
    pub fn required_tech_level(&self) -> u32 {
        match self {
            HullSize::Scout => 0,
            HullSize::Destroyer => 0,
            HullSize::Cruiser => 3,
            HullSize::Battleship => 4,
            HullSize::Dreadnought => 5,
        }
    }

    /// All hull sizes
    pub fn all() -> Vec<HullSize> {
        vec![
            HullSize::Scout,
            HullSize::Destroyer,
            HullSize::Cruiser,
            HullSize::Battleship,
            HullSize::Dreadnought,
        ]
    }
}

/// Ship component types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Engine,
    Weapon,
    Shield,
    Computer,
    Special,
}

/// A component that can be installed on ships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipComponent {
    pub name: String,
    pub component_type: ComponentType,
    /// Space required to install
    pub space: u32,
    /// Cost to build
    pub cost: u32,
    /// Required tech level
    pub tech_level: u32,
    /// Component-specific stats
    pub stats: ComponentStats,
}

/// Component-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStats {
    Engine {
        speed_bonus: i32,
        range_bonus: i32,
    },
    Weapon {
        damage: u32,
        range: u32,
        accuracy: i32,
    },
    Shield {
        strength: u32,
        regeneration: u32,
    },
    Computer {
        accuracy_bonus: i32,
        ecm: i32,
    },
    Special {
        effect: String,
    },
}

impl ShipComponent {
    /// Get all available components
    pub fn all_components() -> Vec<ShipComponent> {
        vec![
            // Engines
            ShipComponent {
                name: "Nuclear Drive".to_string(),
                component_type: ComponentType::Engine,
                space: 5,
                cost: 10,
                tech_level: 0,
                stats: ComponentStats::Engine { speed_bonus: 0, range_bonus: 3 },
            },
            ShipComponent {
                name: "Ion Drive".to_string(),
                component_type: ComponentType::Engine,
                space: 5,
                cost: 20,
                tech_level: 2,
                stats: ComponentStats::Engine { speed_bonus: 1, range_bonus: 5 },
            },
            ShipComponent {
                name: "Warp Engine".to_string(),
                component_type: ComponentType::Engine,
                space: 8,
                cost: 40,
                tech_level: 3,
                stats: ComponentStats::Engine { speed_bonus: 2, range_bonus: 8 },
            },
            ShipComponent {
                name: "Hyperdrive".to_string(),
                component_type: ComponentType::Engine,
                space: 10,
                cost: 80,
                tech_level: 4,
                stats: ComponentStats::Engine { speed_bonus: 3, range_bonus: 12 },
            },

            // Weapons
            ShipComponent {
                name: "Laser Cannon".to_string(),
                component_type: ComponentType::Weapon,
                space: 3,
                cost: 5,
                tech_level: 0,
                stats: ComponentStats::Weapon { damage: 5, range: 1, accuracy: 0 },
            },
            ShipComponent {
                name: "Particle Beam".to_string(),
                component_type: ComponentType::Weapon,
                space: 5,
                cost: 15,
                tech_level: 2,
                stats: ComponentStats::Weapon { damage: 10, range: 2, accuracy: 5 },
            },
            ShipComponent {
                name: "Fusion Missile".to_string(),
                component_type: ComponentType::Weapon,
                space: 8,
                cost: 30,
                tech_level: 3,
                stats: ComponentStats::Weapon { damage: 20, range: 3, accuracy: -5 },
            },
            ShipComponent {
                name: "Plasma Torpedo".to_string(),
                component_type: ComponentType::Weapon,
                space: 12,
                cost: 60,
                tech_level: 4,
                stats: ComponentStats::Weapon { damage: 35, range: 3, accuracy: 0 },
            },
            ShipComponent {
                name: "Antimatter Cannon".to_string(),
                component_type: ComponentType::Weapon,
                space: 20,
                cost: 120,
                tech_level: 5,
                stats: ComponentStats::Weapon { damage: 60, range: 4, accuracy: 10 },
            },

            // Shields
            ShipComponent {
                name: "Deflector Screen".to_string(),
                component_type: ComponentType::Shield,
                space: 3,
                cost: 8,
                tech_level: 1,
                stats: ComponentStats::Shield { strength: 10, regeneration: 1 },
            },
            ShipComponent {
                name: "Class II Shield".to_string(),
                component_type: ComponentType::Shield,
                space: 5,
                cost: 20,
                tech_level: 2,
                stats: ComponentStats::Shield { strength: 25, regeneration: 2 },
            },
            ShipComponent {
                name: "Class III Shield".to_string(),
                component_type: ComponentType::Shield,
                space: 8,
                cost: 50,
                tech_level: 3,
                stats: ComponentStats::Shield { strength: 50, regeneration: 5 },
            },
            ShipComponent {
                name: "Class V Shield".to_string(),
                component_type: ComponentType::Shield,
                space: 15,
                cost: 100,
                tech_level: 5,
                stats: ComponentStats::Shield { strength: 100, regeneration: 10 },
            },

            // Computers
            ShipComponent {
                name: "Targeting Computer".to_string(),
                component_type: ComponentType::Computer,
                space: 2,
                cost: 10,
                tech_level: 1,
                stats: ComponentStats::Computer { accuracy_bonus: 10, ecm: 0 },
            },
            ShipComponent {
                name: "Battle Computer".to_string(),
                component_type: ComponentType::Computer,
                space: 4,
                cost: 30,
                tech_level: 3,
                stats: ComponentStats::Computer { accuracy_bonus: 25, ecm: 10 },
            },
            ShipComponent {
                name: "Oracle System".to_string(),
                component_type: ComponentType::Computer,
                space: 8,
                cost: 80,
                tech_level: 5,
                stats: ComponentStats::Computer { accuracy_bonus: 50, ecm: 30 },
            },

            // Specials
            ShipComponent {
                name: "Colony Module".to_string(),
                component_type: ComponentType::Special,
                space: 15,
                cost: 50,
                tech_level: 0,
                stats: ComponentStats::Special { effect: "colony".to_string() },
            },
            ShipComponent {
                name: "Extended Fuel Tanks".to_string(),
                component_type: ComponentType::Special,
                space: 5,
                cost: 15,
                tech_level: 1,
                stats: ComponentStats::Special { effect: "fuel_range_+50%".to_string() },
            },
            ShipComponent {
                name: "Cloaking Device".to_string(),
                component_type: ComponentType::Special,
                space: 20,
                cost: 200,
                tech_level: 5,
                stats: ComponentStats::Special { effect: "cloak".to_string() },
            },
        ]
    }

    /// Get component by name
    pub fn get(name: &str) -> Option<ShipComponent> {
        Self::all_components().into_iter().find(|c| c.name == name)
    }

    /// Get components available at a tech level
    pub fn available_at_level(level: u32) -> Vec<ShipComponent> {
        Self::all_components().into_iter()
            .filter(|c| c.tech_level <= level)
            .collect()
    }
}

/// A custom ship design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipDesign {
    pub id: u32,
    pub name: String,
    pub hull: HullSize,
    pub components: Vec<String>, // Component names
    /// Cached stats
    pub total_hp: u32,
    pub attack_power: u32,
    pub defense: u32,
    pub speed: u32,
    pub range: u32,
    pub cost: u32,
    /// Can this ship establish colonies?
    pub is_colony_ship: bool,
}

impl ShipDesign {
    /// Create a new ship design
    pub fn new(id: u32, name: String, hull: HullSize, components: Vec<String>) -> Result<Self, String> {
        let all_components = ShipComponent::all_components();
        let mut total_space = 0;
        let mut total_cost = hull.cost();
        let mut speed_bonus = 0i32;
        let mut range_bonus = 0i32;
        let mut attack = 0u32;
        let mut defense = 0u32;
        let mut is_colony_ship = false;

        for comp_name in &components {
            let Some(comp) = all_components.iter().find(|c| &c.name == comp_name) else {
                return Err(format!("Unknown component: {}", comp_name));
            };
            total_space += comp.space;
            total_cost += comp.cost;

            match &comp.stats {
                ComponentStats::Engine { speed_bonus: s, range_bonus: r } => {
                    speed_bonus += s;
                    range_bonus += r;
                }
                ComponentStats::Weapon { damage, .. } => {
                    attack += damage;
                }
                ComponentStats::Shield { strength, .. } => {
                    defense += strength;
                }
                ComponentStats::Computer { accuracy_bonus, .. } => {
                    attack += (*accuracy_bonus as u32) / 5; // Minor attack bonus
                }
                ComponentStats::Special { effect } => {
                    if effect == "colony" {
                        is_colony_ship = true;
                    }
                }
            }
        }

        if total_space > hull.space() {
            return Err(format!("Design exceeds hull space: {} > {}", total_space, hull.space()));
        }

        let speed = ((hull.base_speed() as i32) + speed_bonus).max(1) as u32;
        let range = (3 + range_bonus).max(1) as u32;

        Ok(ShipDesign {
            id,
            name,
            hull,
            components,
            total_hp: hull.base_hp(),
            attack_power: attack,
            defense,
            speed,
            range,
            cost: total_cost,
            is_colony_ship,
        })
    }

    /// Create default designs for starting a game
    pub fn default_designs() -> Vec<ShipDesign> {
        vec![
            ShipDesign::new(
                0,
                "Scout".to_string(),
                HullSize::Scout,
                vec!["Nuclear Drive".to_string()],
            ).unwrap(),
            ShipDesign::new(
                1,
                "Colony Ship".to_string(),
                HullSize::Destroyer,
                vec!["Nuclear Drive".to_string(), "Colony Module".to_string()],
            ).unwrap(),
            ShipDesign::new(
                2,
                "Fighter".to_string(),
                HullSize::Destroyer,
                vec![
                    "Nuclear Drive".to_string(),
                    "Laser Cannon".to_string(),
                    "Laser Cannon".to_string(),
                ],
            ).unwrap(),
        ]
    }
}

/// A fleet of ships at a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fleet {
    pub id: u32,
    pub empire_id: u32,
    pub name: String,
    /// Star ID where fleet is located (or travelling from)
    pub location_star_id: u32,
    /// Star ID fleet is travelling to (None if stationary)
    pub destination_star_id: Option<u32>,
    /// Turns until arrival (0 = arrived or stationary)
    pub eta_turns: u32,
    /// Ships in this fleet: design_id -> count
    pub ships: HashMap<u32, u32>,
}

impl Fleet {
    pub fn new(id: u32, empire_id: u32, name: String, location: u32) -> Self {
        Fleet {
            id,
            empire_id,
            name,
            location_star_id: location,
            destination_star_id: None,
            eta_turns: 0,
            ships: HashMap::new(),
        }
    }

    /// Add ships to the fleet
    pub fn add_ships(&mut self, design_id: u32, count: u32) {
        *self.ships.entry(design_id).or_insert(0) += count;
    }

    /// Remove ships from the fleet
    pub fn remove_ships(&mut self, design_id: u32, count: u32) -> bool {
        if let Some(current) = self.ships.get_mut(&design_id) {
            if *current >= count {
                *current -= count;
                if *current == 0 {
                    self.ships.remove(&design_id);
                }
                return true;
            }
        }
        false
    }

    /// Total ship count
    pub fn total_ships(&self) -> u32 {
        self.ships.values().sum()
    }

    /// Check if fleet is in transit
    pub fn is_in_transit(&self) -> bool {
        self.destination_star_id.is_some() && self.eta_turns > 0
    }

    /// Start moving to a destination
    pub fn set_destination(&mut self, star_id: u32, eta: u32) {
        self.destination_star_id = Some(star_id);
        self.eta_turns = eta;
    }

    /// Process one turn of movement
    pub fn advance_turn(&mut self) -> bool {
        if self.eta_turns > 0 {
            self.eta_turns -= 1;
            if self.eta_turns == 0 {
                if let Some(dest) = self.destination_star_id.take() {
                    self.location_star_id = dest;
                    return true; // Arrived
                }
            }
        }
        false
    }

    /// Calculate fleet combat power
    pub fn combat_power(&self, designs: &[ShipDesign]) -> u32 {
        let mut power = 0;
        for (design_id, count) in &self.ships {
            if let Some(design) = designs.iter().find(|d| d.id == *design_id) {
                power += (design.attack_power + design.defense) * count;
            }
        }
        power
    }

    /// Calculate fleet speed (slowest ship)
    pub fn fleet_speed(&self, designs: &[ShipDesign]) -> u32 {
        let mut min_speed = u32::MAX;
        for (design_id, _) in &self.ships {
            if let Some(design) = designs.iter().find(|d| d.id == *design_id) {
                min_speed = min_speed.min(design.speed);
            }
        }
        if min_speed == u32::MAX { 1 } else { min_speed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hull_sizes() {
        assert!(HullSize::Scout.space() < HullSize::Destroyer.space());
        assert!(HullSize::Destroyer.space() < HullSize::Cruiser.space());
        assert!(HullSize::Dreadnought.cost() > HullSize::Scout.cost());
    }

    #[test]
    fn test_ship_design_creation() {
        let design = ShipDesign::new(
            0,
            "Test Ship".to_string(),
            HullSize::Destroyer,
            vec!["Nuclear Drive".to_string(), "Laser Cannon".to_string()],
        );
        assert!(design.is_ok());
        let design = design.unwrap();
        assert_eq!(design.name, "Test Ship");
        assert!(design.cost > 0);
    }

    #[test]
    fn test_ship_design_over_capacity() {
        // Try to put too much on a scout
        let result = ShipDesign::new(
            0,
            "Overloaded".to_string(),
            HullSize::Scout,
            vec![
                "Hyperdrive".to_string(), // 10 space
                "Plasma Torpedo".to_string(), // 12 space
            ],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_colony_ship() {
        let design = ShipDesign::new(
            0,
            "Colony Ship".to_string(),
            HullSize::Destroyer,
            vec!["Nuclear Drive".to_string(), "Colony Module".to_string()],
        ).unwrap();
        assert!(design.is_colony_ship);
    }

    #[test]
    fn test_fleet_operations() {
        let mut fleet = Fleet::new(0, 0, "1st Fleet".to_string(), 0);
        fleet.add_ships(0, 5);
        assert_eq!(fleet.total_ships(), 5);

        fleet.add_ships(0, 3);
        assert_eq!(fleet.total_ships(), 8);

        assert!(fleet.remove_ships(0, 3));
        assert_eq!(fleet.total_ships(), 5);

        assert!(!fleet.remove_ships(0, 10)); // Can't remove more than we have
    }

    #[test]
    fn test_fleet_movement() {
        let mut fleet = Fleet::new(0, 0, "1st Fleet".to_string(), 0);
        assert!(!fleet.is_in_transit());

        fleet.set_destination(5, 3);
        assert!(fleet.is_in_transit());

        fleet.advance_turn();
        assert_eq!(fleet.eta_turns, 2);
        assert!(fleet.is_in_transit());

        fleet.advance_turn();
        fleet.advance_turn();
        assert!(!fleet.is_in_transit());
        assert_eq!(fleet.location_star_id, 5);
    }
}
