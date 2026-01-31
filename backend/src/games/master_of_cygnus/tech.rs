//! Technology research system for Master of Andromeda

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Technology research fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechField {
    /// Ship speed and range
    Propulsion,
    /// Beam, missile, bomb damage
    Weapons,
    /// Defense technology
    Shields,
    /// Terraforming, farming, population
    Planetology,
    /// Building speed, ship hulls
    Construction,
    /// Targeting, scanning
    Computers,
}

impl TechField {
    pub fn name(&self) -> &'static str {
        match self {
            TechField::Propulsion => "Propulsion",
            TechField::Weapons => "Weapons",
            TechField::Shields => "Shields",
            TechField::Planetology => "Planetology",
            TechField::Construction => "Construction",
            TechField::Computers => "Computers",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TechField::Propulsion => "Ship speed, fuel range, and warp technology",
            TechField::Weapons => "Beam weapons, missiles, and planetary bombs",
            TechField::Shields => "Deflector shields and armor plating",
            TechField::Planetology => "Terraforming, farming, and population growth",
            TechField::Construction => "Factory output, ship hulls, and build speed",
            TechField::Computers => "Targeting systems, scanners, and ECM",
        }
    }

    pub fn all() -> Vec<TechField> {
        vec![
            TechField::Propulsion,
            TechField::Weapons,
            TechField::Shields,
            TechField::Planetology,
            TechField::Construction,
            TechField::Computers,
        ]
    }
}

/// A specific technology that can be researched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technology {
    pub id: u32,
    pub name: String,
    pub field: TechField,
    /// Tech level (1-10)
    pub level: u32,
    /// Research points required
    pub cost: u32,
    pub description: String,
    /// Effects granted by this tech
    pub effects: Vec<TechEffect>,
}

/// Effects that technologies can provide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechEffect {
    /// Unlock ability to colonize a planet type
    ColonizePlanet(String), // planet type name
    /// Increase ship speed
    ShipSpeed(i32),
    /// Increase ship range
    ShipRange(i32),
    /// New weapon damage
    WeaponDamage { weapon_name: String, damage: i32 },
    /// Shield strength
    ShieldStrength(i32),
    /// Production bonus percentage
    ProductionBonus(i32),
    /// Population growth bonus
    GrowthBonus(i32),
    /// Research bonus
    ResearchBonus(i32),
    /// Unlock a building type
    UnlockBuilding(String),
    /// Unlock a ship hull
    UnlockHull(String),
    /// Scanner range
    ScannerRange(i32),
    /// Combat accuracy bonus
    AccuracyBonus(i32),
}

/// Tech tree containing all technologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechTree {
    pub technologies: Vec<Technology>,
}

impl TechTree {
    /// Create the default tech tree
    pub fn new() -> Self {
        let mut techs = Vec::new();
        let mut id = 0;

        // Propulsion technologies
        for level in 1..=6 {
            let (name, desc, effects) = match level {
                1 => ("Hydrogen Fuel Cells", "Basic interstellar propulsion",
                      vec![TechEffect::ShipSpeed(1), TechEffect::ShipRange(3)]),
                2 => ("Ion Drives", "Efficient sub-light engines",
                      vec![TechEffect::ShipSpeed(2), TechEffect::ShipRange(5)]),
                3 => ("Warp Drive", "Faster-than-light travel",
                      vec![TechEffect::ShipSpeed(3), TechEffect::ShipRange(8)]),
                4 => ("Hyperdrive", "Advanced FTL propulsion",
                      vec![TechEffect::ShipSpeed(4), TechEffect::ShipRange(12)]),
                5 => ("Antimatter Engines", "Near-light speed capability",
                      vec![TechEffect::ShipSpeed(5), TechEffect::ShipRange(18)]),
                6 => ("Quantum Slipstream", "Instantaneous galaxy travel",
                      vec![TechEffect::ShipSpeed(6), TechEffect::ShipRange(99)]),
                _ => continue,
            };
            techs.push(Technology {
                id, name: name.to_string(), field: TechField::Propulsion,
                level, cost: level * 100, description: desc.to_string(), effects,
            });
            id += 1;
        }

        // Weapons technologies
        for level in 1..=6 {
            let (name, desc, effects) = match level {
                1 => ("Laser Cannon", "Basic directed energy weapon",
                      vec![TechEffect::WeaponDamage { weapon_name: "Laser".to_string(), damage: 5 }]),
                2 => ("Particle Beam", "High-energy particle accelerator",
                      vec![TechEffect::WeaponDamage { weapon_name: "Particle Beam".to_string(), damage: 10 }]),
                3 => ("Fusion Missiles", "Tactical nuclear warheads",
                      vec![TechEffect::WeaponDamage { weapon_name: "Fusion Missile".to_string(), damage: 15 }]),
                4 => ("Plasma Torpedo", "Superheated plasma projectile",
                      vec![TechEffect::WeaponDamage { weapon_name: "Plasma Torpedo".to_string(), damage: 25 }]),
                5 => ("Antimatter Bomb", "Planetary bombardment weapon",
                      vec![TechEffect::WeaponDamage { weapon_name: "AM Bomb".to_string(), damage: 50 }]),
                6 => ("Stellar Converter", "Star-destroying superweapon",
                      vec![TechEffect::WeaponDamage { weapon_name: "Stellar Converter".to_string(), damage: 100 }]),
                _ => continue,
            };
            techs.push(Technology {
                id, name: name.to_string(), field: TechField::Weapons,
                level, cost: level * 150, description: desc.to_string(), effects,
            });
            id += 1;
        }

        // Shields technologies
        for level in 1..=6 {
            let (name, desc, effects) = match level {
                1 => ("Deflector Screen", "Basic energy barrier",
                      vec![TechEffect::ShieldStrength(5)]),
                2 => ("Class II Shields", "Improved deflectors",
                      vec![TechEffect::ShieldStrength(10)]),
                3 => ("Class III Shields", "Heavy deflector grid",
                      vec![TechEffect::ShieldStrength(20)]),
                4 => ("Planetary Shields", "Full planet protection",
                      vec![TechEffect::ShieldStrength(35), TechEffect::UnlockBuilding("PlanetaryShield".to_string())]),
                5 => ("Class V Shields", "Nearly impenetrable barrier",
                      vec![TechEffect::ShieldStrength(50)]),
                6 => ("Hardened Shields", "Maximum protection technology",
                      vec![TechEffect::ShieldStrength(75)]),
                _ => continue,
            };
            techs.push(Technology {
                id, name: name.to_string(), field: TechField::Shields,
                level, cost: level * 120, description: desc.to_string(), effects,
            });
            id += 1;
        }

        // Planetology technologies
        for level in 1..=6 {
            let (name, desc, effects) = match level {
                1 => ("Soil Enrichment", "Improved agricultural yield",
                      vec![TechEffect::GrowthBonus(10), TechEffect::UnlockBuilding("Farm".to_string())]),
                2 => ("Controlled Environment", "Colonize harsh worlds",
                      vec![TechEffect::ColonizePlanet("Tundra".to_string())]),
                3 => ("Terraforming", "Transform barren worlds",
                      vec![TechEffect::ColonizePlanet("Barren".to_string()), TechEffect::GrowthBonus(20)]),
                4 => ("Advanced Terraforming", "Toxic atmosphere conversion",
                      vec![TechEffect::ColonizePlanet("Toxic".to_string())]),
                5 => ("Gaia Transformation", "Create paradise worlds",
                      vec![TechEffect::GrowthBonus(50)]),
                6 => ("Atmospheric Control", "Perfect any environment",
                      vec![TechEffect::GrowthBonus(100), TechEffect::UnlockBuilding("OrbitalHabitat".to_string())]),
                _ => continue,
            };
            techs.push(Technology {
                id, name: name.to_string(), field: TechField::Planetology,
                level, cost: level * 80, description: desc.to_string(), effects,
            });
            id += 1;
        }

        // Construction technologies
        for level in 1..=6 {
            let (name, desc, effects) = match level {
                1 => ("Robotic Workers", "Automated factory labor",
                      vec![TechEffect::ProductionBonus(10), TechEffect::UnlockBuilding("Factory".to_string())]),
                2 => ("Deep Mining", "Access subterranean resources",
                      vec![TechEffect::ProductionBonus(20), TechEffect::UnlockBuilding("DeepCoreMine".to_string())]),
                3 => ("Cruiser Hull", "Medium warship design",
                      vec![TechEffect::UnlockHull("Cruiser".to_string())]),
                4 => ("Battleship Hull", "Heavy warship design",
                      vec![TechEffect::UnlockHull("Battleship".to_string())]),
                5 => ("Dreadnought Hull", "Capital ship design",
                      vec![TechEffect::UnlockHull("Dreadnought".to_string())]),
                6 => ("Star Forge", "Ultimate production facility",
                      vec![TechEffect::ProductionBonus(50), TechEffect::UnlockBuilding("StarForge".to_string())]),
                _ => continue,
            };
            techs.push(Technology {
                id, name: name.to_string(), field: TechField::Construction,
                level, cost: level * 100, description: desc.to_string(), effects,
            });
            id += 1;
        }

        // Computer technologies
        for level in 1..=6 {
            let (name, desc, effects) = match level {
                1 => ("Battle Computer Mk I", "Basic targeting assistance",
                      vec![TechEffect::AccuracyBonus(5)]),
                2 => ("Deep Space Scanner", "Extended detection range",
                      vec![TechEffect::ScannerRange(3), TechEffect::UnlockBuilding("ResearchLab".to_string())]),
                3 => ("Battle Computer Mk II", "Advanced fire control",
                      vec![TechEffect::AccuracyBonus(15)]),
                4 => ("Hyperspace Scanner", "Galaxy-wide detection",
                      vec![TechEffect::ScannerRange(10), TechEffect::ResearchBonus(20)]),
                5 => ("Battle Computer Mk III", "Superior targeting",
                      vec![TechEffect::AccuracyBonus(30)]),
                6 => ("Oracle Network", "Perfect information warfare",
                      vec![TechEffect::ScannerRange(99), TechEffect::AccuracyBonus(50),
                           TechEffect::UnlockBuilding("HyperspaceComm".to_string())]),
                _ => continue,
            };
            techs.push(Technology {
                id, name: name.to_string(), field: TechField::Computers,
                level, cost: level * 110, description: desc.to_string(), effects,
            });
            id += 1;
        }

        TechTree { technologies: techs }
    }

    /// Get all technologies in a field
    pub fn get_field_techs(&self, field: TechField) -> Vec<&Technology> {
        self.technologies.iter()
            .filter(|t| t.field == field)
            .collect()
    }

    /// Get a technology by ID
    pub fn get_tech(&self, id: u32) -> Option<&Technology> {
        self.technologies.iter().find(|t| t.id == id)
    }

    /// Get the next researchable technology in a field
    pub fn next_tech_in_field(&self, field: TechField, current_level: u32) -> Option<&Technology> {
        self.technologies.iter()
            .filter(|t| t.field == field && t.level == current_level + 1)
            .next()
    }
}

impl Default for TechTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Research progress for an empire
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchProgress {
    /// Current research points accumulated per field
    pub points: HashMap<TechField, u32>,
    /// Current tech level achieved per field
    pub levels: HashMap<TechField, u32>,
    /// Research allocation percentage per field (should sum to 100)
    pub allocation: HashMap<TechField, u32>,
}

impl ResearchProgress {
    pub fn new() -> Self {
        let mut allocation = HashMap::new();
        // Default: equal distribution
        for field in TechField::all() {
            allocation.insert(field, 16); // 16 * 6 = 96, close to 100
        }
        allocation.insert(TechField::Propulsion, 20); // Slightly boost propulsion

        Self {
            points: HashMap::new(),
            levels: HashMap::new(),
            allocation,
        }
    }

    /// Get current level in a field (0 if not researched)
    pub fn level(&self, field: TechField) -> u32 {
        self.levels.get(&field).copied().unwrap_or(0)
    }

    /// Get current points in a field
    pub fn points_in_field(&self, field: TechField) -> u32 {
        self.points.get(&field).copied().unwrap_or(0)
    }

    /// Get allocation for a field
    pub fn allocation_for(&self, field: TechField) -> u32 {
        self.allocation.get(&field).copied().unwrap_or(0)
    }

    /// Add research points and check for breakthroughs
    pub fn add_research(&mut self, total_points: u32, tech_tree: &TechTree) -> Vec<Technology> {
        let mut breakthroughs = Vec::new();

        for field in TechField::all() {
            let alloc = self.allocation_for(field);
            let field_points = (total_points * alloc) / 100;

            if field_points > 0 {
                // Add points
                let new_points = self.points.get(&field).copied().unwrap_or(0) + field_points;
                self.points.insert(field, new_points);

                // Check if we've completed the next tech
                let current_level = self.level(field);
                if let Some(next_tech) = tech_tree.next_tech_in_field(field, current_level) {
                    if new_points >= next_tech.cost {
                        self.points.insert(field, new_points - next_tech.cost);
                        self.levels.insert(field, current_level + 1);
                        breakthroughs.push(next_tech.clone());
                    }
                }
            }
        }

        breakthroughs
    }

    /// Set allocation for a field (caller must ensure total = 100)
    pub fn set_allocation(&mut self, field: TechField, percent: u32) {
        self.allocation.insert(field, percent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tech_tree_creation() {
        let tree = TechTree::new();
        assert!(!tree.technologies.is_empty());

        // Should have techs in all fields
        for field in TechField::all() {
            let field_techs = tree.get_field_techs(field);
            assert!(!field_techs.is_empty(), "Field {:?} has no techs", field);
        }
    }

    #[test]
    fn test_tech_levels() {
        let tree = TechTree::new();

        // Each field should have 6 levels
        for field in TechField::all() {
            let field_techs = tree.get_field_techs(field);
            assert_eq!(field_techs.len(), 6, "Field {:?} should have 6 techs", field);

            // Check levels are 1-6
            for level in 1..=6 {
                assert!(
                    field_techs.iter().any(|t| t.level == level),
                    "Field {:?} missing level {}", field, level
                );
            }
        }
    }

    #[test]
    fn test_research_progress() {
        let tree = TechTree::new();
        let mut progress = ResearchProgress::new();

        // Initial level should be 0
        assert_eq!(progress.level(TechField::Weapons), 0);

        // Add enough research to complete first weapon tech (150 points)
        progress.allocation.insert(TechField::Weapons, 100); // All to weapons
        let breakthroughs = progress.add_research(200, &tree);

        assert_eq!(progress.level(TechField::Weapons), 1);
        assert_eq!(breakthroughs.len(), 1);
    }

    #[test]
    fn test_next_tech() {
        let tree = TechTree::new();

        // Should get level 1 tech when at level 0
        let next = tree.next_tech_in_field(TechField::Propulsion, 0);
        assert!(next.is_some());
        assert_eq!(next.unwrap().level, 1);

        // Should get level 2 tech when at level 1
        let next = tree.next_tech_in_field(TechField::Propulsion, 1);
        assert!(next.is_some());
        assert_eq!(next.unwrap().level, 2);
    }
}
