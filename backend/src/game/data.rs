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

/// Commodity prices scaled for game balance while respecting real-world ratios.
/// Prices are in cents. Game units represent "street-level" quantities.
/// Tiers: Junk ($5-30) -> Budget ($30-100) -> Mid ($100-300) -> Premium ($300-800) -> Luxury ($800-2000) -> Elite ($2000+)
pub static COMMODITIES: &[Commodity] = &[
    // JUNK TIER - Easy entry, low margins (starter money territory)
    Commodity { key: "tidepods", name: "Tide Pods", min_price: 500, max_price: 1500, addictive: false, action_boost: false },
    Commodity { key: "ludes", name: "Ludes", min_price: 800, max_price: 2500, addictive: false, action_boost: false },

    // BUDGET TIER - Bread and butter (early game)
    Commodity { key: "weed", name: "Weed", min_price: 1500, max_price: 4000, addictive: false, action_boost: false },       // ~$15-40/unit
    Commodity { key: "speed", name: "Speed", min_price: 2000, max_price: 5000, addictive: true, action_boost: true },       // ~$20-50/unit
    Commodity { key: "krokodil", name: "Krokodil", min_price: 2500, max_price: 6000, addictive: true, action_boost: false }, // Cheap heroin alt
    Commodity { key: "fentanyl", name: "Fentanyl", min_price: 3000, max_price: 8000, addictive: true, action_boost: false }, // Cheap per dose

    // MID TIER - Real money starts here (mid game)
    Commodity { key: "bathsalts", name: "Bath Salts", min_price: 5000, max_price: 15000, addictive: true, action_boost: true },
    Commodity { key: "meth", name: "Meth", min_price: 8000, max_price: 25000, addictive: true, action_boost: true },        // ~$80-250/unit
    Commodity { key: "acid", name: "Acid", min_price: 10000, max_price: 30000, addictive: false, action_boost: false },

    // PREMIUM TIER - High risk, high reward
    Commodity { key: "heroin", name: "Heroin", min_price: 25000, max_price: 80000, addictive: true, action_boost: false },  // ~$250-800/unit
    Commodity { key: "ketamine", name: "Ketamine", min_price: 30000, max_price: 90000, addictive: true, action_boost: false }, // Club scene ~$300-900

    // LUXURY TIER - Kingpin territory
    Commodity { key: "cocaine", name: "Cocaine", min_price: 80000, max_price: 200000, addictive: true, action_boost: false }, // ~$800-2000/unit
    Commodity { key: "mdma", name: "MDMA", min_price: 90000, max_price: 250000, addictive: true, action_boost: true },      // Pure molly, party scene

    // ELITE TIER - Wall Street, tech bros, trust fund kids
    Commodity { key: "oxy", name: "Oxy", min_price: 200000, max_price: 500000, addictive: true, action_boost: false },      // Prescription opioid ~$2000-5000
    Commodity { key: "adderall", name: "Adderall", min_price: 150000, max_price: 400000, addictive: true, action_boost: true }, // Focus drug ~$1500-4000
    Commodity { key: "dmt", name: "DMT", min_price: 300000, max_price: 800000, addictive: false, action_boost: false },     // "Spirit molecule" ~$3000-8000
];

/// Location-specific shop inventory - what's available varies by area
/// Each tuple: (commodity_key, price_modifier) - modifier is 80-120 (percentage)
/// Max 10 items per shop for clean UI
pub static SHOP_INVENTORY: &[(&str, &str, &[(&str, u8)])] = &[
    // NYC
    ("nyc", "bronx", &[
        ("tidepods", 100), ("weed", 95), ("speed", 100), ("krokodil", 90),
        ("fentanyl", 95), ("meth", 100), ("acid", 105), ("heroin", 100),
    ]),
    ("nyc", "brooklyn", &[
        ("weed", 100), ("speed", 105), ("fentanyl", 100), ("bathsalts", 95),
        ("meth", 95), ("ketamine", 100), ("heroin", 95), ("cocaine", 90),  // Mafia territory
    ]),
    ("nyc", "manhattan", &[  // Wall Street - elite drugs
        ("weed", 115), ("acid", 95), ("ketamine", 90), ("cocaine", 95),
        ("mdma", 95), ("oxy", 85), ("adderall", 80), ("dmt", 100),  // Finance bros
    ]),
    ("nyc", "queens", &[
        ("tidepods", 95), ("weed", 100), ("speed", 95), ("krokodil", 100),
        ("fentanyl", 100), ("meth", 95), ("ketamine", 105), ("heroin", 100),  // Triads
    ]),

    // MIAMI
    ("miami", "little_havana", &[
        ("weed", 90), ("speed", 95), ("fentanyl", 105), ("meth", 100),
        ("heroin", 95), ("cocaine", 80),  // Cartel - cheap cocaine
    ]),
    ("miami", "south_beach", &[  // Party scene - club drugs
        ("ludes", 90), ("weed", 110), ("bathsalts", 90), ("acid", 85),
        ("ketamine", 85), ("cocaine", 95), ("mdma", 80), ("adderall", 95),  // Club kids
    ]),
    ("miami", "downtown_miami", &[
        ("weed", 100), ("speed", 100), ("krokodil", 105), ("fentanyl", 100),
        ("meth", 100), ("acid", 100), ("heroin", 100), ("cocaine", 95),
    ]),

    // LONDON
    ("london", "east_end", &[
        ("weed", 110), ("speed", 100), ("krokodil", 95), ("fentanyl", 110),
        ("meth", 105), ("heroin", 90), ("cocaine", 95),  // Mafia
    ]),
    ("london", "soho", &[  // Trendy nightlife - designer drugs
        ("ludes", 95), ("weed", 105), ("bathsalts", 95), ("acid", 85),
        ("ketamine", 85), ("cocaine", 95), ("mdma", 90), ("dmt", 95),  // Artsy crowd
    ]),
    ("london", "brixton", &[
        ("tidepods", 100), ("weed", 85), ("speed", 95), ("krokodil", 100),
        ("fentanyl", 100), ("meth", 100), ("heroin", 100),
    ]),

    // TOKYO
    ("tokyo", "shinjuku", &[  // Yakuza territory
        ("speed", 85), ("bathsalts", 90), ("meth", 80),  // Meth capital
        ("acid", 100), ("ketamine", 95), ("heroin", 110), ("cocaine", 115),
    ]),
    ("tokyo", "shibuya", &[  // Youth fashion district - party drugs
        ("ludes", 100), ("weed", 115), ("bathsalts", 85), ("acid", 90),
        ("ketamine", 90), ("mdma", 85), ("adderall", 90),  // Young professionals
    ]),
    ("tokyo", "roppongi", &[  // Expat nightlife - everything available
        ("weed", 110), ("meth", 90), ("acid", 95), ("ketamine", 85),
        ("cocaine", 100), ("mdma", 90), ("oxy", 100), ("dmt", 90),  // International scene
    ]),

    // BOGOTA
    ("bogota", "chapinero", &[
        ("weed", 85), ("speed", 90), ("fentanyl", 115), ("meth", 95),
        ("heroin", 90), ("cocaine", 70),  // Cartel HQ - cheapest cocaine
    ]),
    ("bogota", "la_candelaria", &[
        ("tidepods", 110), ("weed", 80), ("krokodil", 90), ("fentanyl", 110),
        ("acid", 90), ("heroin", 85), ("cocaine", 75), ("dmt", 80),  // Ayahuasca country
    ]),
    ("bogota", "usaquen", &[  // Upscale diplomatic area
        ("weed", 100), ("acid", 95), ("ketamine", 95), ("cocaine", 75),
        ("mdma", 100), ("oxy", 90), ("adderall", 85),  // Rich expats
    ]),
];

/// Get available commodities and their price modifiers for a location
pub fn get_shop_inventory(city: &str, borough: &str) -> Vec<(&'static str, u8)> {
    SHOP_INVENTORY.iter()
        .find(|(c, b, _)| *c == city && *b == borough)
        .map(|(_, _, items)| items.to_vec())
        .unwrap_or_else(|| {
            // Default inventory if not found
            vec![
                ("weed", 100), ("speed", 100), ("meth", 100),
                ("heroin", 100), ("cocaine", 100),
            ]
        })
}

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
