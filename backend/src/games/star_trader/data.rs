//! Star Trader - Static Game Data
//!
//! Defines ships, commodities, port types, and other game constants.

/// Ship class with stats
#[derive(Debug, Clone, PartialEq)]
pub struct ShipClass {
    pub key: &'static str,
    pub name: &'static str,
    pub cargo_holds: u32,
    pub max_fighters: u32,
    pub max_shields: u32,
    pub warp_speed: u32,        // Sectors per turn
    pub scanner_range: u32,     // Sectors visible
    pub price: i64,             // Credits
    pub requires_commission: bool,
}

/// Trading commodity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Commodity {
    FuelOre,
    Organics,
    Equipment,
}

impl Commodity {
    pub fn name(&self) -> &'static str {
        match self {
            Commodity::FuelOre => "Fuel Ore",
            Commodity::Organics => "Organics",
            Commodity::Equipment => "Equipment",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Commodity::FuelOre => "Ore",
            Commodity::Organics => "Org",
            Commodity::Equipment => "Equ",
        }
    }

    pub fn all() -> [Commodity; 3] {
        [Commodity::FuelOre, Commodity::Organics, Commodity::Equipment]
    }
}

/// Port trading type - what they buy/sell
/// B = Buy from traders, S = Sell to traders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortType {
    pub fuel_ore: TradeDirection,
    pub organics: TradeDirection,
    pub equipment: TradeDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeDirection {
    Buying,   // Port buys FROM you (you sell)
    Selling,  // Port sells TO you (you buy)
}

impl PortType {
    /// BBB - Buys all (rare, profitable dump location)
    pub const BBB: PortType = PortType {
        fuel_ore: TradeDirection::Buying,
        organics: TradeDirection::Buying,
        equipment: TradeDirection::Buying,
    };

    /// BBS - Buys Ore/Org, Sells Equipment
    pub const BBS: PortType = PortType {
        fuel_ore: TradeDirection::Buying,
        organics: TradeDirection::Buying,
        equipment: TradeDirection::Selling,
    };

    /// BSB - Buys Ore/Equip, Sells Organics
    pub const BSB: PortType = PortType {
        fuel_ore: TradeDirection::Buying,
        organics: TradeDirection::Selling,
        equipment: TradeDirection::Buying,
    };

    /// SBB - Sells Ore, Buys Org/Equip
    pub const SBB: PortType = PortType {
        fuel_ore: TradeDirection::Selling,
        organics: TradeDirection::Buying,
        equipment: TradeDirection::Buying,
    };

    /// SSB - Sells Ore/Org, Buys Equipment
    pub const SSB: PortType = PortType {
        fuel_ore: TradeDirection::Selling,
        organics: TradeDirection::Selling,
        equipment: TradeDirection::Buying,
    };

    /// SBS - Sells Ore/Equip, Buys Organics
    pub const SBS: PortType = PortType {
        fuel_ore: TradeDirection::Selling,
        organics: TradeDirection::Buying,
        equipment: TradeDirection::Selling,
    };

    /// BSS - Buys Ore, Sells Org/Equip
    pub const BSS: PortType = PortType {
        fuel_ore: TradeDirection::Buying,
        organics: TradeDirection::Selling,
        equipment: TradeDirection::Selling,
    };

    /// SSS - Sells all (rare, good buying location)
    pub const SSS: PortType = PortType {
        fuel_ore: TradeDirection::Selling,
        organics: TradeDirection::Selling,
        equipment: TradeDirection::Selling,
    };

    pub fn code(&self) -> String {
        let ore = if self.fuel_ore == TradeDirection::Buying { 'B' } else { 'S' };
        let org = if self.organics == TradeDirection::Buying { 'B' } else { 'S' };
        let equ = if self.equipment == TradeDirection::Buying { 'B' } else { 'S' };
        format!("{}{}{}", ore, org, equ)
    }

    pub fn direction_for(&self, commodity: Commodity) -> TradeDirection {
        match commodity {
            Commodity::FuelOre => self.fuel_ore,
            Commodity::Organics => self.organics,
            Commodity::Equipment => self.equipment,
        }
    }

    /// Get all port types
    pub fn all() -> [PortType; 8] {
        [
            PortType::BBB, PortType::BBS, PortType::BSB, PortType::SBB,
            PortType::SSB, PortType::SBS, PortType::BSS, PortType::SSS,
        ]
    }

    /// Get random port type weighted by rarity
    pub fn random(rng: &mut impl rand::Rng) -> PortType {
        // BBB and SSS are rare (5% each), others share remaining 90%
        let roll = rng.gen_range(0..100);
        match roll {
            0..=4 => PortType::BBB,
            5..=9 => PortType::SSS,
            10..=24 => PortType::BBS,
            25..=39 => PortType::BSB,
            40..=54 => PortType::SBB,
            55..=69 => PortType::SSB,
            70..=84 => PortType::SBS,
            _ => PortType::BSS,
        }
    }
}

/// Planet class (colonizable worlds)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetClass {
    ClassM,  // Earth-like (best)
    ClassL,  // Marginal
    ClassK,  // Adaptable
    ClassO,  // Oceanic
    ClassH,  // Desert
}

impl PlanetClass {
    pub fn name(&self) -> &'static str {
        match self {
            PlanetClass::ClassM => "Class M (Earth-like)",
            PlanetClass::ClassL => "Class L (Marginal)",
            PlanetClass::ClassK => "Class K (Adaptable)",
            PlanetClass::ClassO => "Class O (Oceanic)",
            PlanetClass::ClassH => "Class H (Desert)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            PlanetClass::ClassM => "M",
            PlanetClass::ClassL => "L",
            PlanetClass::ClassK => "K",
            PlanetClass::ClassO => "O",
            PlanetClass::ClassH => "H",
        }
    }

    /// Production multiplier (Class M = 1.0, others lower)
    pub fn production_multiplier(&self) -> f64 {
        match self {
            PlanetClass::ClassM => 1.0,
            PlanetClass::ClassL => 0.7,
            PlanetClass::ClassK => 0.8,
            PlanetClass::ClassO => 0.6,
            PlanetClass::ClassH => 0.5,
        }
    }

    pub fn random(rng: &mut impl rand::Rng) -> PlanetClass {
        match rng.gen_range(0..100) {
            0..=10 => PlanetClass::ClassM,   // 11% - rare
            11..=30 => PlanetClass::ClassL,  // 20%
            31..=55 => PlanetClass::ClassK,  // 25%
            56..=75 => PlanetClass::ClassO,  // 20%
            _ => PlanetClass::ClassH,        // 24%
        }
    }
}

/// Sector type (what's in a sector)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectorType {
    Empty,           // Just warps
    Port,            // Trading post
    Planet,          // Colonizable world
    StarDock,        // Central hub (sector 1)
    FerrengiSpace,   // Dangerous territory
    Nebula,          // Sensor interference
    Asteroid,        // Mining opportunity
}

/// Ship classes available for purchase
pub static SHIP_CLASSES: &[ShipClass] = &[
    ShipClass {
        key: "merchant_cruiser",
        name: "Merchant Cruiser",
        cargo_holds: 20,
        max_fighters: 30,
        max_shields: 100,
        warp_speed: 3,
        scanner_range: 2,
        price: 0,  // Starting ship
        requires_commission: false,
    },
    ShipClass {
        key: "scout_marauder",
        name: "Scout Marauder",
        cargo_holds: 10,
        max_fighters: 50,
        max_shields: 80,
        warp_speed: 5,
        scanner_range: 4,
        price: 50_000,
        requires_commission: false,
    },
    ShipClass {
        key: "cargo_freighter",
        name: "Cargo Freighter",
        cargo_holds: 100,
        max_fighters: 20,
        max_shields: 150,
        warp_speed: 2,
        scanner_range: 1,
        price: 150_000,
        requires_commission: false,
    },
    ShipClass {
        key: "colonial_frigate",
        name: "Colonial Frigate",
        cargo_holds: 50,
        max_fighters: 75,
        max_shields: 200,
        warp_speed: 3,
        scanner_range: 3,
        price: 300_000,
        requires_commission: false,
    },
    ShipClass {
        key: "battle_cruiser",
        name: "Battle Cruiser",
        cargo_holds: 30,
        max_fighters: 150,
        max_shields: 400,
        warp_speed: 4,
        scanner_range: 3,
        price: 500_000,
        requires_commission: false,
    },
    ShipClass {
        key: "imperial_starship",
        name: "Imperial StarShip",
        cargo_holds: 75,
        max_fighters: 300,
        max_shields: 600,
        warp_speed: 5,
        scanner_range: 5,
        price: 1_000_000,
        requires_commission: true,  // Requires Federation commission
    },
];

/// Get ship class by key
pub fn get_ship_class(key: &str) -> Option<&'static ShipClass> {
    SHIP_CLASSES.iter().find(|s| s.key == key)
}

/// Game configuration constants
pub mod config {
    /// Daily turns per player
    pub const DAILY_TURNS: u32 = 100;

    /// Maximum corporation size
    pub const MAX_CORP_SIZE: usize = 10;

    /// Starting credits
    pub const STARTING_CREDITS: i64 = 10_000;

    /// Starting turns
    pub const STARTING_TURNS: u32 = 100;

    /// Base fighter cost
    pub const FIGHTER_COST: i64 = 100;

    /// Base shield cost
    pub const SHIELD_COST: i64 = 50;

    /// Commodity base prices (credits per unit)
    pub const FUEL_ORE_BASE_PRICE: i64 = 20;
    pub const ORGANICS_BASE_PRICE: i64 = 25;
    pub const EQUIPMENT_BASE_PRICE: i64 = 50;

    /// Price variance percentage
    pub const PRICE_VARIANCE: f64 = 0.30;

    /// Experience per kill
    pub const XP_PER_KILL: i64 = 100;

    /// Experience per successful trade
    pub const XP_PER_TRADE: i64 = 10;

    /// Ferrengi encounter chance (percentage)
    pub const FERRENGI_CHANCE: u32 = 15;

    /// Galaxy sizes
    pub const GALAXY_SMALL: u32 = 1000;
    pub const GALAXY_MEDIUM: u32 = 5000;
    pub const GALAXY_LARGE: u32 = 10000;

    /// Average warps per sector
    pub const AVG_WARPS_PER_SECTOR: u32 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_class_lookup() {
        let ship = get_ship_class("merchant_cruiser");
        assert!(ship.is_some());
        assert_eq!(ship.unwrap().cargo_holds, 20);
    }

    #[test]
    fn test_port_type_code() {
        assert_eq!(PortType::BBB.code(), "BBB");
        assert_eq!(PortType::SSS.code(), "SSS");
        assert_eq!(PortType::BBS.code(), "BBS");
    }

    #[test]
    fn test_commodity_names() {
        assert_eq!(Commodity::FuelOre.name(), "Fuel Ore");
        assert_eq!(Commodity::FuelOre.short_name(), "Ore");
    }

    #[test]
    fn test_planet_class_multipliers() {
        assert_eq!(PlanetClass::ClassM.production_multiplier(), 1.0);
        assert!(PlanetClass::ClassH.production_multiplier() < 1.0);
    }
}
