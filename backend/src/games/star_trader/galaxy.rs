//! Star Trader - Galaxy Generation and Navigation
//!
//! Handles procedural galaxy generation, sector connections (warps),
//! and navigation between sectors.

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet, VecDeque};
use super::data::{PortType, PlanetClass, config};

/// A sector in the galaxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub id: u32,
    pub sector_type: SectorTypeData,
    pub warps: Vec<u32>,          // Connected sectors (one-way possible)
    pub fighters: u32,            // Deployed fighters (defense)
    pub mines: u32,               // Deployed mines
    pub owner_id: Option<i64>,    // Player who controls this sector
}

/// Sector type with associated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectorTypeData {
    Empty,
    Port(PortData),
    Planet(PlanetData),
    StarDock,
    FerrengiSpace { strength: u32 },
    Nebula,
    Asteroid { ore_remaining: u32 },
}

/// Port trading data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortData {
    pub name: String,
    pub port_type: PortTypeCode,
    pub fuel_ore: PortStock,
    pub organics: PortStock,
    pub equipment: PortStock,
}

/// Port type as serializable code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PortTypeCode {
    BBB, BBS, BSB, SBB, SSB, SBS, BSS, SSS,
}

impl PortTypeCode {
    pub fn to_port_type(&self) -> PortType {
        match self {
            PortTypeCode::BBB => PortType::BBB,
            PortTypeCode::BBS => PortType::BBS,
            PortTypeCode::BSB => PortType::BSB,
            PortTypeCode::SBB => PortType::SBB,
            PortTypeCode::SSB => PortType::SSB,
            PortTypeCode::SBS => PortType::SBS,
            PortTypeCode::BSS => PortType::BSS,
            PortTypeCode::SSS => PortType::SSS,
        }
    }

    pub fn from_port_type(pt: PortType) -> Self {
        if pt.fuel_ore == super::data::TradeDirection::Buying {
            if pt.organics == super::data::TradeDirection::Buying {
                if pt.equipment == super::data::TradeDirection::Buying {
                    PortTypeCode::BBB
                } else {
                    PortTypeCode::BBS
                }
            } else if pt.equipment == super::data::TradeDirection::Buying {
                PortTypeCode::BSB
            } else {
                PortTypeCode::BSS
            }
        } else if pt.organics == super::data::TradeDirection::Buying {
            if pt.equipment == super::data::TradeDirection::Buying {
                PortTypeCode::SBB
            } else {
                PortTypeCode::SBS
            }
        } else if pt.equipment == super::data::TradeDirection::Buying {
            PortTypeCode::SSB
        } else {
            PortTypeCode::SSS
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            PortTypeCode::BBB => "BBB",
            PortTypeCode::BBS => "BBS",
            PortTypeCode::BSB => "BSB",
            PortTypeCode::SBB => "SBB",
            PortTypeCode::SSB => "SSB",
            PortTypeCode::SBS => "SBS",
            PortTypeCode::BSS => "BSS",
            PortTypeCode::SSS => "SSS",
        }
    }
}

/// Stock level for a commodity at a port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortStock {
    pub quantity: u32,
    pub max_quantity: u32,
    pub price: i64,            // Current price
    pub base_price: i64,       // Base price for this port
}

impl PortStock {
    pub fn new(max: u32, base_price: i64) -> Self {
        Self {
            quantity: max / 2,  // Start half full
            max_quantity: max,
            price: base_price,
            base_price,
        }
    }

    /// Update price based on supply/demand
    pub fn update_price(&mut self) {
        let ratio = self.quantity as f64 / self.max_quantity as f64;
        // Low supply = high price, high supply = low price
        let multiplier = 1.5 - ratio;  // 0.5 to 1.5
        self.price = ((self.base_price as f64) * multiplier) as i64;
        self.price = self.price.max(1);  // Minimum 1 credit
    }
}

/// Planet colonization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetData {
    pub name: String,
    pub class: PlanetClassCode,
    pub owner_id: Option<i64>,
    pub colonists: u32,
    pub citadel_level: u32,        // Defense level 0-5
    pub ore_production: u32,       // Daily production
    pub organics_production: u32,
    pub equipment_production: u32,
    pub fighter_production: u32,
    pub ore_stored: u32,
    pub organics_stored: u32,
    pub equipment_stored: u32,
    pub fighters_stored: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanetClassCode {
    M, L, K, O, H,
}

impl PlanetClassCode {
    pub fn to_planet_class(&self) -> PlanetClass {
        match self {
            PlanetClassCode::M => PlanetClass::ClassM,
            PlanetClassCode::L => PlanetClass::ClassL,
            PlanetClassCode::K => PlanetClass::ClassK,
            PlanetClassCode::O => PlanetClass::ClassO,
            PlanetClassCode::H => PlanetClass::ClassH,
        }
    }

    pub fn from_planet_class(pc: PlanetClass) -> Self {
        match pc {
            PlanetClass::ClassM => PlanetClassCode::M,
            PlanetClass::ClassL => PlanetClassCode::L,
            PlanetClass::ClassK => PlanetClassCode::K,
            PlanetClass::ClassO => PlanetClassCode::O,
            PlanetClass::ClassH => PlanetClassCode::H,
        }
    }
}

impl PlanetData {
    pub fn new(name: String, class: PlanetClass) -> Self {
        Self {
            name,
            class: PlanetClassCode::from_planet_class(class),
            owner_id: None,
            colonists: 0,
            citadel_level: 0,
            ore_production: 0,
            organics_production: 0,
            equipment_production: 0,
            fighter_production: 0,
            ore_stored: 0,
            organics_stored: 0,
            equipment_stored: 0,
            fighters_stored: 0,
        }
    }
}

/// Galaxy structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Galaxy {
    pub seed: u64,
    pub size: u32,
    pub sectors: HashMap<u32, Sector>,
}

impl Galaxy {
    /// Generate a new galaxy with the given seed and size
    pub fn generate(seed: u64, size: u32) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut sectors = HashMap::new();

        // Generate StarDock at sector 1
        sectors.insert(1, Sector {
            id: 1,
            sector_type: SectorTypeData::StarDock,
            warps: Vec::new(),
            fighters: 0,
            mines: 0,
            owner_id: None,
        });

        // Generate remaining sectors
        for id in 2..=size {
            let sector = generate_sector(&mut rng, id);
            sectors.insert(id, sector);
        }

        // Generate warp connections
        generate_warps(&mut rng, &mut sectors, size);

        // Ensure StarDock is well-connected
        let stardock_warps: Vec<u32> = (2..=6.min(size)).collect();
        if let Some(sector) = sectors.get_mut(&1) {
            sector.warps = stardock_warps.clone();
        }
        // Add return warps
        for warp_to in stardock_warps {
            if let Some(sector) = sectors.get_mut(&warp_to) {
                if !sector.warps.contains(&1) {
                    sector.warps.push(1);
                }
            }
        }

        Galaxy { seed, size, sectors }
    }

    /// Get a sector by ID
    pub fn get_sector(&self, id: u32) -> Option<&Sector> {
        self.sectors.get(&id)
    }

    /// Get a mutable sector by ID
    pub fn get_sector_mut(&mut self, id: u32) -> Option<&mut Sector> {
        self.sectors.get_mut(&id)
    }

    /// Find path between two sectors (BFS)
    pub fn find_path(&self, from: u32, to: u32) -> Option<Vec<u32>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parents: HashMap<u32, u32> = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            if let Some(sector) = self.sectors.get(&current) {
                for &next in &sector.warps {
                    if !visited.contains(&next) {
                        visited.insert(next);
                        parents.insert(next, current);
                        queue.push_back(next);

                        if next == to {
                            // Reconstruct path
                            let mut path = vec![to];
                            let mut curr = to;
                            while let Some(&parent) = parents.get(&curr) {
                                path.push(parent);
                                curr = parent;
                            }
                            path.reverse();
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    /// Get sectors within scanner range
    pub fn scan_sectors(&self, from: u32, range: u32) -> Vec<u32> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((from, 0u32));
        visited.insert(from);

        while let Some((current, depth)) = queue.pop_front() {
            if depth > 0 {
                result.push(current);
            }

            if depth < range {
                if let Some(sector) = self.sectors.get(&current) {
                    for &next in &sector.warps {
                        if !visited.contains(&next) {
                            visited.insert(next);
                            queue.push_back((next, depth + 1));
                        }
                    }
                }
            }
        }

        result
    }
}

/// Generate a single sector
fn generate_sector(rng: &mut StdRng, id: u32) -> Sector {
    let sector_type = generate_sector_type(rng, id);

    Sector {
        id,
        sector_type,
        warps: Vec::new(),
        fighters: 0,
        mines: 0,
        owner_id: None,
    }
}

/// Generate sector type
fn generate_sector_type(rng: &mut StdRng, id: u32) -> SectorTypeData {
    let roll = rng.gen_range(0..100);

    match roll {
        0..=39 => SectorTypeData::Empty,                      // 40% empty
        40..=64 => {                                          // 25% port
            let port_type = PortType::random(rng);
            let name = generate_port_name(rng, id);
            SectorTypeData::Port(PortData {
                name,
                port_type: PortTypeCode::from_port_type(port_type),
                fuel_ore: PortStock::new(
                    rng.gen_range(500..2000),
                    config::FUEL_ORE_BASE_PRICE,
                ),
                organics: PortStock::new(
                    rng.gen_range(500..2000),
                    config::ORGANICS_BASE_PRICE,
                ),
                equipment: PortStock::new(
                    rng.gen_range(300..1000),
                    config::EQUIPMENT_BASE_PRICE,
                ),
            })
        }
        65..=79 => {                                          // 15% planet
            let class = PlanetClass::random(rng);
            let name = generate_planet_name(rng, id);
            SectorTypeData::Planet(PlanetData::new(name, class))
        }
        80..=89 => {                                          // 10% ferrengi
            SectorTypeData::FerrengiSpace {
                strength: rng.gen_range(10..100),
            }
        }
        90..=94 => SectorTypeData::Nebula,                    // 5% nebula
        _ => {                                                // 5% asteroid
            SectorTypeData::Asteroid {
                ore_remaining: rng.gen_range(100..1000),
            }
        }
    }
}

/// Generate port name
fn generate_port_name(rng: &mut StdRng, sector_id: u32) -> String {
    let prefixes = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Omega", "Nova", "Star", "Sol"];
    let suffixes = ["Station", "Port", "Hub", "Depot", "Terminal", "Base", "Outpost", "Exchange"];

    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];

    format!("{} {} {}", prefix, suffix, sector_id)
}

/// Generate planet name
fn generate_planet_name(rng: &mut StdRng, sector_id: u32) -> String {
    let names = [
        "Terra", "Gaia", "Kepler", "Proxima", "Eden", "Atlas", "Titan",
        "Rigel", "Vega", "Sirius", "Orion", "Lyra", "Draco", "Cygnus",
        "Aquila", "Phoenix", "Perseus", "Andromeda", "Cassiopeia",
    ];

    let name = names[rng.gen_range(0..names.len())];
    let suffix = ['b', 'c', 'd', 'e', 'f'][rng.gen_range(0..5)];

    format!("{}-{}{}", name, sector_id, suffix)
}

/// Generate warp connections between sectors
fn generate_warps(rng: &mut StdRng, sectors: &mut HashMap<u32, Sector>, size: u32) {
    // Create a connected graph using Erdos-Renyi-like approach
    // but ensuring connectivity

    // First, create a spanning tree to ensure connectivity
    let mut connected: HashSet<u32> = HashSet::new();
    let mut unconnected: Vec<u32> = (1..=size).collect();

    // Shuffle unconnected
    for i in (1..unconnected.len()).rev() {
        let j = rng.gen_range(0..=i);
        unconnected.swap(i, j);
    }

    // Start with first sector
    let first = unconnected.pop().unwrap();
    connected.insert(first);

    // Connect remaining sectors to existing ones
    while let Some(new_sector) = unconnected.pop() {
        // Pick a random connected sector
        let connected_vec: Vec<u32> = connected.iter().copied().collect();
        let connect_to = connected_vec[rng.gen_range(0..connected_vec.len())];

        // Add bidirectional warp
        if let Some(sector) = sectors.get_mut(&new_sector) {
            sector.warps.push(connect_to);
        }
        if let Some(sector) = sectors.get_mut(&connect_to) {
            sector.warps.push(new_sector);
        }

        connected.insert(new_sector);
    }

    // Add additional random warps
    let extra_warps = (size as f64 * (config::AVG_WARPS_PER_SECTOR as f64 - 1.0)) as u32;
    for _ in 0..extra_warps {
        let from = rng.gen_range(1..=size);
        let to = rng.gen_range(1..=size);

        if from != to {
            if let Some(sector) = sectors.get_mut(&from) {
                if !sector.warps.contains(&to) && sector.warps.len() < 6 {
                    sector.warps.push(to);

                    // 80% chance of bidirectional warp
                    if rng.gen_range(0..100) < 80 {
                        if let Some(to_sector) = sectors.get_mut(&to) {
                            if !to_sector.warps.contains(&from) && to_sector.warps.len() < 6 {
                                to_sector.warps.push(from);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort all warp lists
    for sector in sectors.values_mut() {
        sector.warps.sort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galaxy_generation() {
        let galaxy = Galaxy::generate(12345, 100);
        assert_eq!(galaxy.size, 100);
        assert_eq!(galaxy.sectors.len(), 100);
    }

    #[test]
    fn test_stardock_exists() {
        let galaxy = Galaxy::generate(12345, 100);
        let stardock = galaxy.get_sector(1).unwrap();
        assert!(matches!(stardock.sector_type, SectorTypeData::StarDock));
    }

    #[test]
    fn test_connectivity() {
        let galaxy = Galaxy::generate(12345, 100);
        // Should be able to reach any sector from sector 1
        for id in 2..=100 {
            let path = galaxy.find_path(1, id);
            assert!(path.is_some(), "No path from 1 to {}", id);
        }
    }

    #[test]
    fn test_scan_sectors() {
        let galaxy = Galaxy::generate(12345, 100);
        let scanned = galaxy.scan_sectors(1, 2);
        assert!(!scanned.is_empty());
    }

    #[test]
    fn test_port_stock() {
        let mut stock = PortStock::new(1000, 100);
        assert_eq!(stock.quantity, 500);  // Starts half full

        stock.quantity = 100;  // Low supply
        stock.update_price();
        assert!(stock.price > stock.base_price);  // Higher price

        stock.quantity = 900;  // High supply
        stock.update_price();
        assert!(stock.price < stock.base_price);  // Lower price
    }

    #[test]
    fn test_port_type_codes() {
        for pt in PortType::all() {
            let code = PortTypeCode::from_port_type(pt);
            let back = code.to_port_type();
            assert_eq!(pt.code(), back.code());
        }
    }
}
