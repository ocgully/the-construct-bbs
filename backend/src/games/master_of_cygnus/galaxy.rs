//! Galaxy generation and star system management

use serde::{Deserialize, Serialize};
use rand::Rng;
use rand::seq::SliceRandom;
use super::data::{STAR_NAMES, GalaxySettings};

/// A star in the galaxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Star {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub planet: Planet,
    /// Empire ID that owns this system (None = uncolonized)
    pub owner: Option<u32>,
}

/// A planet orbiting a star (one per star, simplified like MOO1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Planet {
    pub planet_type: PlanetType,
    /// Maximum population this planet can support
    pub max_population: u32,
    /// Current population
    pub population: u32,
    /// Base production output
    pub base_production: u32,
    /// Special resources or features
    pub special: Option<PlanetSpecial>,
}

/// Planet classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanetType {
    /// Ideal conditions, high pop capacity
    Terran,
    /// Good conditions, needs basic tech
    Ocean,
    /// Moderate capacity
    Arid,
    /// Poor conditions
    Tundra,
    /// Requires colonization tech
    Barren,
    /// Advanced tech only
    Toxic,
    /// Cannot be colonized, fuel harvesting only
    GasGiant,
}

impl PlanetType {
    pub fn name(&self) -> &'static str {
        match self {
            PlanetType::Terran => "Terran",
            PlanetType::Ocean => "Ocean",
            PlanetType::Arid => "Arid",
            PlanetType::Tundra => "Tundra",
            PlanetType::Barren => "Barren",
            PlanetType::Toxic => "Toxic",
            PlanetType::GasGiant => "Gas Giant",
        }
    }

    /// Base max population for this planet type
    pub fn base_max_pop(&self) -> u32 {
        match self {
            PlanetType::Terran => 200,
            PlanetType::Ocean => 150,
            PlanetType::Arid => 100,
            PlanetType::Tundra => 80,
            PlanetType::Barren => 50,
            PlanetType::Toxic => 30,
            PlanetType::GasGiant => 0,
        }
    }

    /// Base production value
    pub fn base_production(&self) -> u32 {
        match self {
            PlanetType::Terran => 20,
            PlanetType::Ocean => 15,
            PlanetType::Arid => 15,
            PlanetType::Tundra => 10,
            PlanetType::Barren => 25, // Good for mining
            PlanetType::Toxic => 30, // Rare minerals
            PlanetType::GasGiant => 5, // Fuel only
        }
    }

    /// Required tech level to colonize (0 = always colonizable)
    pub fn required_tech_level(&self) -> u32 {
        match self {
            PlanetType::Terran => 0,
            PlanetType::Ocean => 0,
            PlanetType::Arid => 0,
            PlanetType::Tundra => 1,
            PlanetType::Barren => 2,
            PlanetType::Toxic => 3,
            PlanetType::GasGiant => 99, // Never colonizable
        }
    }

    /// Color for display
    pub fn color(&self) -> &'static str {
        match self {
            PlanetType::Terran => "LightGreen",
            PlanetType::Ocean => "LightBlue",
            PlanetType::Arid => "Yellow",
            PlanetType::Tundra => "LightCyan",
            PlanetType::Barren => "Brown",
            PlanetType::Toxic => "LightMagenta",
            PlanetType::GasGiant => "LightRed",
        }
    }
}

/// Special planetary features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanetSpecial {
    /// Rich in minerals (+50% production)
    MineralRich,
    /// Poor in minerals (-25% production)
    MineralPoor,
    /// Ultra rich (+100% production)
    UltraRich,
    /// Ancient artifacts (+research)
    Artifacts,
    /// Hostile natives (combat on colonization)
    HostileNatives,
    /// Fertile soil (+population growth)
    Fertile,
}

impl PlanetSpecial {
    pub fn name(&self) -> &'static str {
        match self {
            PlanetSpecial::MineralRich => "Mineral Rich",
            PlanetSpecial::MineralPoor => "Mineral Poor",
            PlanetSpecial::UltraRich => "Ultra Rich",
            PlanetSpecial::Artifacts => "Ancient Artifacts",
            PlanetSpecial::HostileNatives => "Hostile Natives",
            PlanetSpecial::Fertile => "Fertile",
        }
    }
}

/// The galaxy containing all stars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Galaxy {
    pub stars: Vec<Star>,
    pub width: i32,
    pub height: i32,
    pub seed: u64,
}

impl Galaxy {
    /// Generate a new galaxy with the given settings and seed
    pub fn generate(settings: &GalaxySettings, seed: u64) -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut stars = Vec::with_capacity(settings.star_count);
        let mut available_names: Vec<&str> = STAR_NAMES.to_vec();
        available_names.shuffle(&mut rng);

        // Generate stars with minimum spacing
        let min_spacing = (settings.map_width / 10).max(10);

        for i in 0..settings.star_count {
            let name = if i < available_names.len() {
                available_names[i].to_string()
            } else {
                format!("Star-{}", i + 1)
            };

            // Try to find a position that's not too close to existing stars
            let (x, y) = loop {
                let x = rng.gen_range(5..settings.map_width - 5);
                let y = rng.gen_range(5..settings.map_height - 5);

                let too_close = stars.iter().any(|s: &Star| {
                    let dx = (s.x - x).abs();
                    let dy = (s.y - y).abs();
                    dx < min_spacing && dy < min_spacing
                });

                if !too_close || stars.len() < 2 {
                    break (x, y);
                }
            };

            let planet = generate_planet(&mut rng);

            stars.push(Star {
                id: i as u32,
                name,
                x,
                y,
                planet,
                owner: None,
            });
        }

        Galaxy {
            stars,
            width: settings.map_width,
            height: settings.map_height,
            seed,
        }
    }

    /// Get a star by ID
    pub fn get_star(&self, id: u32) -> Option<&Star> {
        self.stars.iter().find(|s| s.id == id)
    }

    /// Get a mutable star by ID
    pub fn get_star_mut(&mut self, id: u32) -> Option<&mut Star> {
        self.stars.iter_mut().find(|s| s.id == id)
    }

    /// Get all stars owned by an empire
    pub fn get_empire_stars(&self, empire_id: u32) -> Vec<&Star> {
        self.stars.iter().filter(|s| s.owner == Some(empire_id)).collect()
    }

    /// Calculate distance between two stars
    pub fn distance(&self, star1_id: u32, star2_id: u32) -> Option<f64> {
        let s1 = self.get_star(star1_id)?;
        let s2 = self.get_star(star2_id)?;
        let dx = (s1.x - s2.x) as f64;
        let dy = (s1.y - s2.y) as f64;
        Some((dx * dx + dy * dy).sqrt())
    }

    /// Get stars within range of a given star
    pub fn stars_in_range(&self, star_id: u32, range: f64) -> Vec<&Star> {
        let Some(origin) = self.get_star(star_id) else {
            return Vec::new();
        };

        self.stars.iter()
            .filter(|s| {
                if s.id == star_id {
                    return false;
                }
                let dx = (s.x - origin.x) as f64;
                let dy = (s.y - origin.y) as f64;
                (dx * dx + dy * dy).sqrt() <= range
            })
            .collect()
    }

    /// Find the nearest uncolonized star to a given position
    pub fn nearest_uncolonized(&self, x: i32, y: i32) -> Option<&Star> {
        self.stars.iter()
            .filter(|s| s.owner.is_none() && s.planet.planet_type != PlanetType::GasGiant)
            .min_by_key(|s| {
                let dx = (s.x - x).abs();
                let dy = (s.y - y).abs();
                dx * dx + dy * dy
            })
    }
}

/// Generate a random planet
fn generate_planet<R: Rng>(rng: &mut R) -> Planet {
    // Planet type distribution
    let planet_type = match rng.gen_range(0..100) {
        0..=15 => PlanetType::Terran,   // 15%
        16..=30 => PlanetType::Ocean,   // 15%
        31..=50 => PlanetType::Arid,    // 20%
        51..=65 => PlanetType::Tundra,  // 15%
        66..=80 => PlanetType::Barren,  // 15%
        81..=90 => PlanetType::Toxic,   // 10%
        _ => PlanetType::GasGiant,       // 10%
    };

    // Size variation (affects max pop)
    let size_mod = rng.gen_range(80..=120) as u32;
    let max_population = (planet_type.base_max_pop() * size_mod) / 100;

    // Production variation
    let prod_mod = rng.gen_range(80..=120) as u32;
    let base_production = (planet_type.base_production() * prod_mod) / 100;

    // Special features (20% chance)
    let special = if rng.gen_range(0..100) < 20 {
        Some(match rng.gen_range(0..6) {
            0 => PlanetSpecial::MineralRich,
            1 => PlanetSpecial::MineralPoor,
            2 => PlanetSpecial::UltraRich,
            3 => PlanetSpecial::Artifacts,
            4 => PlanetSpecial::HostileNatives,
            _ => PlanetSpecial::Fertile,
        })
    } else {
        None
    };

    Planet {
        planet_type,
        max_population,
        population: 0,
        base_production,
        special,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galaxy_generation() {
        let settings = GalaxySettings::small();
        let galaxy = Galaxy::generate(&settings, 12345);

        assert_eq!(galaxy.stars.len(), settings.star_count);
        assert!(galaxy.stars.iter().all(|s| s.id < settings.star_count as u32));
    }

    #[test]
    fn test_galaxy_deterministic() {
        let settings = GalaxySettings::small();
        let galaxy1 = Galaxy::generate(&settings, 12345);
        let galaxy2 = Galaxy::generate(&settings, 12345);

        // Same seed should produce same galaxy
        assert_eq!(galaxy1.stars.len(), galaxy2.stars.len());
        for (s1, s2) in galaxy1.stars.iter().zip(galaxy2.stars.iter()) {
            assert_eq!(s1.name, s2.name);
            assert_eq!(s1.x, s2.x);
            assert_eq!(s1.y, s2.y);
        }
    }

    #[test]
    fn test_star_lookup() {
        let settings = GalaxySettings::small();
        let galaxy = Galaxy::generate(&settings, 12345);

        let star = galaxy.get_star(0);
        assert!(star.is_some());
        assert_eq!(star.unwrap().id, 0);

        let invalid = galaxy.get_star(999);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_distance_calculation() {
        let settings = GalaxySettings::small();
        let galaxy = Galaxy::generate(&settings, 12345);

        let dist = galaxy.distance(0, 1);
        assert!(dist.is_some());
        assert!(dist.unwrap() > 0.0);
    }

    #[test]
    fn test_planet_types() {
        assert_eq!(PlanetType::Terran.base_max_pop(), 200);
        assert_eq!(PlanetType::GasGiant.base_max_pop(), 0);
        assert!(PlanetType::Toxic.required_tech_level() > 0);
    }
}
