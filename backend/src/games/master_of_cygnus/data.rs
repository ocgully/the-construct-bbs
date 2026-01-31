//! Static game data for Master of Cygnus

use serde::{Deserialize, Serialize};

/// Playable races with their bonuses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub color: &'static str, // ANSI color name
    /// Research bonus percentage (e.g., 20 = +20%)
    pub research_bonus: i32,
    /// Production bonus percentage
    pub production_bonus: i32,
    /// Population growth bonus percentage
    pub growth_bonus: i32,
    /// Combat bonus percentage
    pub combat_bonus: i32,
    /// Diplomacy bonus percentage
    pub diplomacy_bonus: i32,
}

pub static RACES: &[Race] = &[
    Race {
        key: "terran",
        name: "Terran Federation",
        description: "Balanced humans with diplomatic prowess",
        color: "LightCyan",
        research_bonus: 0,
        production_bonus: 0,
        growth_bonus: 0,
        combat_bonus: 0,
        diplomacy_bonus: 25,
    },
    Race {
        key: "silicoid",
        name: "Silicoid Collective",
        description: "Silicon-based life, can colonize any planet",
        color: "Brown",
        research_bonus: -25,
        production_bonus: 25,
        growth_bonus: -50,
        combat_bonus: 0,
        diplomacy_bonus: -25,
    },
    Race {
        key: "psilon",
        name: "Psilon Technocracy",
        description: "Brilliant researchers, weak in combat",
        color: "LightMagenta",
        research_bonus: 50,
        production_bonus: 0,
        growth_bonus: 0,
        combat_bonus: -25,
        diplomacy_bonus: 0,
    },
    Race {
        key: "klackon",
        name: "Klackon Hive",
        description: "Industrious insects with production bonus",
        color: "LightGreen",
        research_bonus: 0,
        production_bonus: 50,
        growth_bonus: 0,
        combat_bonus: 0,
        diplomacy_bonus: -50,
    },
    Race {
        key: "mrrshan",
        name: "Mrrshan Pride",
        description: "Feline warriors with combat expertise",
        color: "Yellow",
        research_bonus: 0,
        production_bonus: 0,
        growth_bonus: 0,
        combat_bonus: 50,
        diplomacy_bonus: -25,
    },
    Race {
        key: "sakkra",
        name: "Sakkra Brood",
        description: "Reptilians with rapid population growth",
        color: "Green",
        research_bonus: 0,
        production_bonus: 0,
        growth_bonus: 100,
        combat_bonus: 0,
        diplomacy_bonus: -25,
    },
];

pub fn get_race(key: &str) -> Option<&'static Race> {
    RACES.iter().find(|r| r.key == key)
}

/// Star names for galaxy generation
pub static STAR_NAMES: &[&str] = &[
    "Alpha Cygni", "Deneb", "Sadr", "Gienah", "Albireo",
    "Fawaris", "Azelfafage", "Ruchba", "Eta Cygni", "Zeta Cygni",
    "Theta Cygni", "Iota Cygni", "Kappa Cygni", "Lambda Cygni", "Mu Cygni",
    "Nu Cygni", "Xi Cygni", "Omicron Cygni", "Pi Cygni", "Rho Cygni",
    "Sigma Cygni", "Tau Cygni", "Upsilon Cygni", "Phi Cygni", "Chi Cygni",
    "Psi Cygni", "Omega Cygni", "Altair Prime", "Vega Minor", "Arcturus Beta",
    "Rigel Station", "Sirius Gate", "Procyon Hub", "Aldebaran", "Antares",
    "Betelgeuse", "Capella", "Pollux", "Regulus", "Spica",
    "Fomalhaut", "Achernar", "Hadar", "Canopus", "Mimosa",
    "Acrux", "Gacrux", "Shaula", "Bellatrix", "Alnilam",
];

/// Building types for colonies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    // Basic
    ColonyBase,
    Factory,
    ResearchLab,
    Farm,
    // Advanced
    PlanetaryShield,
    Shipyard,
    DeepCoreMine,
    SpacePort,
    // Wonders
    HyperspaceComm,
    OrbitalHabitat,
    StarForge,
}

impl BuildingType {
    pub fn name(&self) -> &'static str {
        match self {
            BuildingType::ColonyBase => "Colony Base",
            BuildingType::Factory => "Factory",
            BuildingType::ResearchLab => "Research Lab",
            BuildingType::Farm => "Hydroponic Farm",
            BuildingType::PlanetaryShield => "Planetary Shield",
            BuildingType::Shipyard => "Shipyard",
            BuildingType::DeepCoreMine => "Deep Core Mine",
            BuildingType::SpacePort => "Space Port",
            BuildingType::HyperspaceComm => "Hyperspace Comm",
            BuildingType::OrbitalHabitat => "Orbital Habitat",
            BuildingType::StarForge => "Star Forge",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BuildingType::ColonyBase => "Basic colony infrastructure",
            BuildingType::Factory => "+10 production per turn",
            BuildingType::ResearchLab => "+5 research per turn",
            BuildingType::Farm => "+50 population capacity",
            BuildingType::PlanetaryShield => "Protects against orbital bombardment",
            BuildingType::Shipyard => "Enables ship construction",
            BuildingType::DeepCoreMine => "+25 production per turn",
            BuildingType::SpacePort => "+20% trade income",
            BuildingType::HyperspaceComm => "Instant communication across galaxy",
            BuildingType::OrbitalHabitat => "+100 population capacity",
            BuildingType::StarForge => "+50 production, can build capital ships",
        }
    }

    pub fn cost(&self) -> i64 {
        match self {
            BuildingType::ColonyBase => 0, // Auto-built
            BuildingType::Factory => 100,
            BuildingType::ResearchLab => 150,
            BuildingType::Farm => 80,
            BuildingType::PlanetaryShield => 500,
            BuildingType::Shipyard => 200,
            BuildingType::DeepCoreMine => 400,
            BuildingType::SpacePort => 300,
            BuildingType::HyperspaceComm => 1000,
            BuildingType::OrbitalHabitat => 800,
            BuildingType::StarForge => 2000,
        }
    }

    /// Required tech level to build (0 = always available)
    pub fn required_tech_level(&self) -> u32 {
        match self {
            BuildingType::ColonyBase => 0,
            BuildingType::Factory => 0,
            BuildingType::ResearchLab => 0,
            BuildingType::Farm => 0,
            BuildingType::PlanetaryShield => 3,
            BuildingType::Shipyard => 1,
            BuildingType::DeepCoreMine => 2,
            BuildingType::SpacePort => 2,
            BuildingType::HyperspaceComm => 4,
            BuildingType::OrbitalHabitat => 3,
            BuildingType::StarForge => 5,
        }
    }

    /// All basic buildings
    pub fn basic_buildings() -> Vec<BuildingType> {
        vec![
            BuildingType::Factory,
            BuildingType::ResearchLab,
            BuildingType::Farm,
        ]
    }

    /// All advanced buildings
    pub fn advanced_buildings() -> Vec<BuildingType> {
        vec![
            BuildingType::PlanetaryShield,
            BuildingType::Shipyard,
            BuildingType::DeepCoreMine,
            BuildingType::SpacePort,
        ]
    }

    /// All wonder buildings
    pub fn wonder_buildings() -> Vec<BuildingType> {
        vec![
            BuildingType::HyperspaceComm,
            BuildingType::OrbitalHabitat,
            BuildingType::StarForge,
        ]
    }
}

/// Game settings for different galaxy sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxySettings {
    pub star_count: usize,
    pub map_width: i32,
    pub map_height: i32,
}

impl GalaxySettings {
    pub fn small() -> Self {
        Self {
            star_count: 20,
            map_width: 100,
            map_height: 100,
        }
    }

    pub fn medium() -> Self {
        Self {
            star_count: 35,
            map_width: 150,
            map_height: 150,
        }
    }

    pub fn large() -> Self {
        Self {
            star_count: 50,
            map_width: 200,
            map_height: 200,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_race_lookup() {
        assert!(get_race("terran").is_some());
        assert!(get_race("psilon").is_some());
        assert!(get_race("invalid").is_none());
    }

    #[test]
    fn test_all_races_have_names() {
        for race in RACES {
            assert!(!race.name.is_empty());
            assert!(!race.key.is_empty());
        }
    }

    #[test]
    fn test_building_costs() {
        // Colony base should be free
        assert_eq!(BuildingType::ColonyBase.cost(), 0);
        // All other buildings should cost something
        assert!(BuildingType::Factory.cost() > 0);
        assert!(BuildingType::StarForge.cost() > 0);
    }

    #[test]
    fn test_star_names_count() {
        assert!(STAR_NAMES.len() >= 50);
    }
}
