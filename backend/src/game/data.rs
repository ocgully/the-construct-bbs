/// City with boroughs
#[derive(Debug, Clone)]
pub struct City {
    pub key: &'static str,
    pub name: &'static str,
    pub boroughs: &'static [Borough],
    pub has_casino: bool,
}

#[derive(Debug, Clone)]
pub struct Borough {
    pub key: &'static str,
    pub name: &'static str,
    pub gang_territory: Option<&'static str>,
    pub has_hospital: bool,
    pub has_gun_shop: bool,
    pub has_mob_doctor: bool,
}

/// Commodity (drug) with price ranges
#[derive(Debug, Clone)]
pub struct Commodity {
    pub key: &'static str,
    pub name: &'static str,
    pub min_price: i64,      // In cents
    pub max_price: i64,
    pub addictive: bool,     // Can cause addiction
    pub action_boost: bool,  // Grants extra actions
}

/// Weapon definition
#[derive(Debug, Clone)]
pub struct Weapon {
    pub key: &'static str,
    pub name: &'static str,
    pub damage: u32,
    pub price: i64,          // In cents
    pub is_gun: bool,        // false = melee
}

/// Gang definition
#[derive(Debug, Clone)]
pub struct Gang {
    pub key: &'static str,
    pub name: &'static str,
    pub tribute_cost: i64,   // Cost to buy favor
}

// ============================================================================
// STATIC DATA
// ============================================================================

pub static CITIES: &[City] = &[
    City {
        key: "nyc",
        name: "New York City",
        boroughs: &[
            Borough { key: "bronx", name: "The Bronx", gang_territory: None, has_hospital: true, has_gun_shop: true, has_mob_doctor: false },
            Borough { key: "brooklyn", name: "Brooklyn", gang_territory: Some("mafia"), has_hospital: false, has_gun_shop: false, has_mob_doctor: true },
            Borough { key: "manhattan", name: "Manhattan", gang_territory: None, has_hospital: true, has_gun_shop: true, has_mob_doctor: false },
            Borough { key: "queens", name: "Queens", gang_territory: Some("triads"), has_hospital: false, has_gun_shop: false, has_mob_doctor: false },
        ],
        has_casino: true,
    },
    City {
        key: "miami",
        name: "Miami",
        boroughs: &[
            Borough { key: "little_havana", name: "Little Havana", gang_territory: Some("cartel"), has_hospital: false, has_gun_shop: true, has_mob_doctor: true },
            Borough { key: "south_beach", name: "South Beach", gang_territory: None, has_hospital: true, has_gun_shop: false, has_mob_doctor: false },
            Borough { key: "downtown_miami", name: "Downtown", gang_territory: None, has_hospital: true, has_gun_shop: true, has_mob_doctor: false },
        ],
        has_casino: true,
    },
    City {
        key: "london",
        name: "London",
        boroughs: &[
            Borough { key: "east_end", name: "East End", gang_territory: Some("mafia"), has_hospital: false, has_gun_shop: false, has_mob_doctor: true },
            Borough { key: "soho", name: "Soho", gang_territory: None, has_hospital: true, has_gun_shop: false, has_mob_doctor: false },
            Borough { key: "brixton", name: "Brixton", gang_territory: None, has_hospital: false, has_gun_shop: true, has_mob_doctor: false },
        ],
        has_casino: false,
    },
    City {
        key: "tokyo",
        name: "Tokyo",
        boroughs: &[
            Borough { key: "shinjuku", name: "Shinjuku", gang_territory: Some("triads"), has_hospital: false, has_gun_shop: true, has_mob_doctor: true },
            Borough { key: "shibuya", name: "Shibuya", gang_territory: None, has_hospital: true, has_gun_shop: false, has_mob_doctor: false },
            Borough { key: "roppongi", name: "Roppongi", gang_territory: None, has_hospital: false, has_gun_shop: false, has_mob_doctor: false },
        ],
        has_casino: true,
    },
    City {
        key: "bogota",
        name: "Bogota",
        boroughs: &[
            Borough { key: "chapinero", name: "Chapinero", gang_territory: Some("cartel"), has_hospital: true, has_gun_shop: true, has_mob_doctor: true },
            Borough { key: "la_candelaria", name: "La Candelaria", gang_territory: Some("cartel"), has_hospital: false, has_gun_shop: true, has_mob_doctor: false },
            Borough { key: "usaquen", name: "Usaquen", gang_territory: None, has_hospital: true, has_gun_shop: false, has_mob_doctor: false },
        ],
        has_casino: false,
    },
];

pub static COMMODITIES: &[Commodity] = &[
    // Classic drugs
    Commodity { key: "cocaine", name: "Cocaine", min_price: 1500000, max_price: 3000000, addictive: true, action_boost: false },
    Commodity { key: "heroin", name: "Heroin", min_price: 500000, max_price: 1400000, addictive: true, action_boost: false },
    Commodity { key: "acid", name: "Acid", min_price: 100000, max_price: 450000, addictive: false, action_boost: false },
    Commodity { key: "weed", name: "Weed", min_price: 30000, max_price: 90000, addictive: false, action_boost: false },
    Commodity { key: "meth", name: "Meth", min_price: 200000, max_price: 550000, addictive: true, action_boost: true },
    Commodity { key: "speed", name: "Speed", min_price: 9000, max_price: 25000, addictive: true, action_boost: true },
    Commodity { key: "ludes", name: "Ludes", min_price: 1100, max_price: 6000, addictive: false, action_boost: false },
    // Modern additions
    Commodity { key: "fentanyl", name: "Fentanyl", min_price: 3000000, max_price: 8000000, addictive: true, action_boost: false },
    Commodity { key: "bathsalts", name: "Bath Salts", min_price: 50000, max_price: 150000, addictive: true, action_boost: true },
    Commodity { key: "krokodil", name: "Krokodil", min_price: 10000, max_price: 40000, addictive: true, action_boost: false },
    Commodity { key: "tidepods", name: "Tide Pods", min_price: 500, max_price: 2000, addictive: false, action_boost: false },
];

pub static WEAPONS: &[Weapon] = &[
    // Melee weapons
    Weapon { key: "knuckles", name: "Brass Knuckles", damage: 5, price: 5000, is_gun: false },
    Weapon { key: "knife", name: "Switchblade", damage: 10, price: 15000, is_gun: false },
    Weapon { key: "pipe", name: "Lead Pipe", damage: 15, price: 8000, is_gun: false },
    Weapon { key: "machete", name: "Machete", damage: 25, price: 35000, is_gun: false },
    // Guns
    Weapon { key: "glock", name: "Glock 19", damage: 20, price: 50000, is_gun: true },
    Weapon { key: "revolver", name: "Six Shooter", damage: 25, price: 40000, is_gun: true },
    Weapon { key: "deagle", name: "Desert Eagle", damage: 35, price: 100000, is_gun: true },
    Weapon { key: "shotgun", name: "Shotgun", damage: 45, price: 80000, is_gun: true },
    Weapon { key: "uzi", name: "Uzi", damage: 40, price: 150000, is_gun: true },
    Weapon { key: "ak47", name: "AK-47", damage: 55, price: 250000, is_gun: true },
    Weapon { key: "m16", name: "M16", damage: 60, price: 300000, is_gun: true },
    Weapon { key: "gatling", name: "Gatling Gun", damage: 100, price: 1000000, is_gun: true },
];

pub static GANGS: &[Gang] = &[
    Gang { key: "triads", name: "The Triads", tribute_cost: 500000 },
    Gang { key: "cartel", name: "The Cartel", tribute_cost: 750000 },
    Gang { key: "mafia", name: "The Mafia", tribute_cost: 600000 },
];

// ============================================================================
// LOOKUP FUNCTIONS
// ============================================================================

pub fn get_city(key: &str) -> Option<&'static City> {
    CITIES.iter().find(|c| c.key == key)
}

pub fn get_borough(city_key: &str, borough_key: &str) -> Option<&'static Borough> {
    get_city(city_key)?.boroughs.iter().find(|b| b.key == borough_key)
}

pub fn get_commodity(key: &str) -> Option<&'static Commodity> {
    COMMODITIES.iter().find(|c| c.key == key)
}

pub fn get_weapon(key: &str) -> Option<&'static Weapon> {
    WEAPONS.iter().find(|w| w.key == key)
}

pub fn get_gang(key: &str) -> Option<&'static Gang> {
    GANGS.iter().find(|g| g.key == key)
}

/// Get travel cost between cities (in cents)
pub fn get_travel_cost(from_city: &str, to_city: &str, mode: TravelMode) -> (i64, u32) {
    if from_city == to_city {
        // Intra-city taxi fare, instant
        return (2000, 0); // $20, 0 days
    }

    // Inter-city travel costs money and time
    match mode {
        TravelMode::Bus => (10000, 1),      // $100, 1 day
        TravelMode::Plane => (50000, 0),    // $500, instant
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TravelMode {
    Bus,
    Plane,
}

/// Get coat tier upgrade cost and what trenchcoat guy might want
pub fn get_coat_upgrade_cost(current_tier: u32) -> Option<i64> {
    match current_tier {
        0 => Some(50000),   // $500 for tier 1
        1 => Some(100000),  // $1000 for tier 2
        2 => Some(250000),  // $2500 for tier 3
        _ => None,          // Already max tier
    }
}
