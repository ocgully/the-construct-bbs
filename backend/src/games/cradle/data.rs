//! Static game data for Cradle
//!
//! Defines tiers, paths, aspects, techniques, and mentors.

use serde::{Deserialize, Serialize};

/// Progression tiers - each represents a power level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TierLevel {
    Unsouled = 0,   // Unawakened
    Copper = 1,     // Village level
    Iron = 2,       // Town level
    Jade = 3,       // City level
    Gold = 4,       // Regional level
    Lord = 5,       // National level (formerly Lowgold/Highgold/Truegold)
    Overlord = 6,   // Continental level
    Sage = 7,       // World level
    Herald = 8,     // Pocket dimension level
    Monarch = 9,    // Solar system level
    Dreadgod = 10,  // Galactic quadrant level
    Abidan = 11,    // Galaxy level
    Judge = 12,     // Universal level
    God = 13,       // Multiversal level
    Void = 14,      // Beyond existence
    Transcendent = 15, // Beyond comprehension
}

impl TierLevel {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Unsouled),
            1 => Some(Self::Copper),
            2 => Some(Self::Iron),
            3 => Some(Self::Jade),
            4 => Some(Self::Gold),
            5 => Some(Self::Lord),
            6 => Some(Self::Overlord),
            7 => Some(Self::Sage),
            8 => Some(Self::Herald),
            9 => Some(Self::Monarch),
            10 => Some(Self::Dreadgod),
            11 => Some(Self::Abidan),
            12 => Some(Self::Judge),
            13 => Some(Self::God),
            14 => Some(Self::Void),
            15 => Some(Self::Transcendent),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Unsouled => "Unsouled",
            Self::Copper => "Copper",
            Self::Iron => "Iron",
            Self::Jade => "Jade",
            Self::Gold => "Gold",
            Self::Lord => "Lord",
            Self::Overlord => "Overlord",
            Self::Sage => "Sage",
            Self::Herald => "Herald",
            Self::Monarch => "Monarch",
            Self::Dreadgod => "Dreadgod",
            Self::Abidan => "Abidan",
            Self::Judge => "Judge",
            Self::God => "God",
            Self::Void => "Void",
            Self::Transcendent => "???",
        }
    }

    pub fn next(&self) -> Option<Self> {
        TierLevel::from_u8(*self as u8 + 1)
    }
}

/// Tier definition with requirements and bonuses
#[derive(Debug, Clone)]
pub struct Tier {
    pub level: TierLevel,
    pub name: &'static str,
    pub description: &'static str,
    pub world_scope: &'static str,
    pub madra_requirement: u64,      // Base madra to advance
    pub insight_requirement: u32,    // Spiritual insight needed
    pub trial_required: bool,        // Must complete advancement trial
    pub power_multiplier: f64,       // Combat power multiplier
    pub madra_regen_bonus: f64,      // Per-tick regen multiplier
    pub unlocks: &'static [&'static str], // Features unlocked at this tier
}

/// Core aspects that can be combined into paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aspect {
    Force,
    Life,
    Shadow,
    Light,
    Fire,
    Water,
    Earth,
    Wind,
    Time,
    Space,
    Dream,
    Destruction,
    Blood,
    Sword,
}

impl Aspect {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Force => "Force",
            Self::Life => "Life",
            Self::Shadow => "Shadow",
            Self::Light => "Light",
            Self::Fire => "Fire",
            Self::Water => "Water",
            Self::Earth => "Earth",
            Self::Wind => "Wind",
            Self::Time => "Time",
            Self::Space => "Space",
            Self::Dream => "Dream",
            Self::Destruction => "Destruction",
            Self::Blood => "Blood",
            Self::Sword => "Sword",
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            Self::Force => '*',
            Self::Life => '+',
            Self::Shadow => '~',
            Self::Light => '!',
            Self::Fire => '^',
            Self::Water => '~',
            Self::Earth => '#',
            Self::Wind => '=',
            Self::Time => '@',
            Self::Space => '%',
            Self::Dream => '?',
            Self::Destruction => 'X',
            Self::Blood => '&',
            Self::Sword => '/',
        }
    }
}

/// Cultivation path combining aspects
#[derive(Debug, Clone)]
pub struct Path {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub aspects: &'static [Aspect],
    pub max_tier_natural: TierLevel,  // Highest tier without special items
    pub power_focus: f64,             // Attack multiplier
    pub defense_focus: f64,           // Defense multiplier
    pub speed_focus: f64,             // Speed multiplier
    pub regen_focus: f64,             // Regen multiplier
    pub compatible_with: &'static [&'static str], // Paths that combine well
    pub incompatible_with: &'static [&'static str], // Paths that cause plateau
}

/// Technique types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechniqueType {
    Cycling,    // Passive advancement
    Enforcer,   // Body enhancement
    Striker,    // Ranged attacks
    Ruler,      // Environment manipulation
    Forger,     // Construct creation
}

impl TechniqueType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cycling => "Cycling",
            Self::Enforcer => "Enforcer",
            Self::Striker => "Striker",
            Self::Ruler => "Ruler",
            Self::Forger => "Forger",
        }
    }
}

/// Individual technique definition
#[derive(Debug, Clone)]
pub struct Technique {
    pub key: &'static str,
    pub name: &'static str,
    pub technique_type: TechniqueType,
    pub path_key: &'static str,
    pub tier_requirement: TierLevel,
    pub description: &'static str,
    pub power_base: u64,
    pub madra_cost: u64,
    pub cooldown_ticks: u32,
}

/// Mentor definition
#[derive(Debug, Clone)]
pub struct Mentor {
    pub key: &'static str,
    pub name: &'static str,
    pub tier_range: (TierLevel, TierLevel),
    pub specialization: &'static [Aspect],
    pub personality: &'static str,
    pub hints: &'static [&'static str],
}

// ============================================================================
// STATIC DATA
// ============================================================================

pub static TIERS: &[Tier] = &[
    Tier {
        level: TierLevel::Unsouled,
        name: "Unsouled",
        description: "You have not yet awakened to the sacred arts.",
        world_scope: "Your village",
        madra_requirement: 0,
        insight_requirement: 0,
        trial_required: false,
        power_multiplier: 0.1,
        madra_regen_bonus: 0.1,
        unlocks: &[],
    },
    Tier {
        level: TierLevel::Copper,
        name: "Copper",
        description: "The first steps on the path. You can sense aura.",
        world_scope: "Your valley",
        madra_requirement: 100,
        insight_requirement: 1,
        trial_required: false,
        power_multiplier: 1.0,
        madra_regen_bonus: 1.0,
        unlocks: &["cycling", "basic_techniques"],
    },
    Tier {
        level: TierLevel::Iron,
        name: "Iron",
        description: "Your body is tempered. Physical limits transcended.",
        world_scope: "Your region",
        madra_requirement: 1_000,
        insight_requirement: 5,
        trial_required: false,
        power_multiplier: 2.0,
        madra_regen_bonus: 1.5,
        unlocks: &["iron_body", "enforcer_techniques"],
    },
    Tier {
        level: TierLevel::Jade,
        name: "Jade",
        description: "Your spirit awakens. You can perceive souls.",
        world_scope: "Your nation",
        madra_requirement: 10_000,
        insight_requirement: 15,
        trial_required: true,
        power_multiplier: 5.0,
        madra_regen_bonus: 2.0,
        unlocks: &["jade_senses", "striker_techniques", "ruler_techniques"],
    },
    Tier {
        level: TierLevel::Gold,
        name: "Gold",
        description: "True power manifests. Lesser beings bow before you.",
        world_scope: "Your continent",
        madra_requirement: 100_000,
        insight_requirement: 50,
        trial_required: true,
        power_multiplier: 15.0,
        madra_regen_bonus: 3.0,
        unlocks: &["goldsign", "forger_techniques", "madra_channels"],
    },
    Tier {
        level: TierLevel::Lord,
        name: "Lord",
        description: "You touch the Way. Reality bends to your will.",
        world_scope: "The world",
        madra_requirement: 1_000_000,
        insight_requirement: 100,
        trial_required: true,
        power_multiplier: 50.0,
        madra_regen_bonus: 5.0,
        unlocks: &["soulfire", "revelation", "domain"],
    },
    Tier {
        level: TierLevel::Overlord,
        name: "Overlord",
        description: "Your revelation reshapes your very existence.",
        world_scope: "Other worlds",
        madra_requirement: 10_000_000,
        insight_requirement: 200,
        trial_required: true,
        power_multiplier: 200.0,
        madra_regen_bonus: 10.0,
        unlocks: &["overlord_revelation", "pocket_worlds"],
    },
    Tier {
        level: TierLevel::Sage,
        name: "Sage",
        description: "You touch the origin. Icons manifest at your will.",
        world_scope: "The Willverse",
        madra_requirement: 100_000_000,
        insight_requirement: 500,
        trial_required: true,
        power_multiplier: 1000.0,
        madra_regen_bonus: 25.0,
        unlocks: &["icon", "authority", "sage_commands"],
    },
    Tier {
        level: TierLevel::Herald,
        name: "Herald",
        description: "Spirit and body become one. You exist in both worlds.",
        world_scope: "The spiritual realm",
        madra_requirement: 1_000_000_000,
        insight_requirement: 1000,
        trial_required: true,
        power_multiplier: 5000.0,
        madra_regen_bonus: 50.0,
        unlocks: &["spirit_merge", "herald_form", "dimension_travel"],
    },
    Tier {
        level: TierLevel::Monarch,
        name: "Monarch",
        description: "Sage and Herald combined. You are a force of nature.",
        world_scope: "Solar systems",
        madra_requirement: 100_000_000_000,
        insight_requirement: 2500,
        trial_required: true,
        power_multiplier: 50000.0,
        madra_regen_bonus: 200.0,
        unlocks: &["monarch_authority", "world_creation", "abidan_notice"],
    },
    Tier {
        level: TierLevel::Dreadgod,
        name: "Dreadgod",
        description: "You have consumed the power of a Dreadgod.",
        world_scope: "Galactic quadrants",
        madra_requirement: 10_000_000_000_000,
        insight_requirement: 5000,
        trial_required: true,
        power_multiplier: 500000.0,
        madra_regen_bonus: 1000.0,
        unlocks: &["dreadgod_power", "corruption_resistance"],
    },
    Tier {
        level: TierLevel::Abidan,
        name: "Abidan",
        description: "You walk between worlds as a guardian of reality.",
        world_scope: "Galaxies",
        madra_requirement: 1_000_000_000_000_000,
        insight_requirement: 10000,
        trial_required: true,
        power_multiplier: 10_000_000.0,
        madra_regen_bonus: 10000.0,
        unlocks: &["world_walking", "abidan_mantle", "iteration_access"],
    },
    Tier {
        level: TierLevel::Judge,
        name: "Judge",
        description: "You are among the supreme guardians of existence.",
        world_scope: "Universes",
        madra_requirement: 1_000_000_000_000_000_000,
        insight_requirement: 25000,
        trial_required: true,
        power_multiplier: 1_000_000_000.0,
        madra_regen_bonus: 100000.0,
        unlocks: &["judge_authority", "fate_manipulation", "origin_access"],
    },
    Tier {
        level: TierLevel::God,
        name: "God",
        description: "You transcend the very concept of power.",
        world_scope: "Multiverses",
        madra_requirement: u64::MAX / 1000,
        insight_requirement: 50000,
        trial_required: true,
        power_multiplier: 1_000_000_000_000.0,
        madra_regen_bonus: 1_000_000.0,
        unlocks: &["multiverse_access", "reality_creation"],
    },
    Tier {
        level: TierLevel::Void,
        name: "Void",
        description: "You exist beyond existence itself.",
        world_scope: "Beyond existence",
        madra_requirement: u64::MAX / 10,
        insight_requirement: 100000,
        trial_required: true,
        power_multiplier: 1_000_000_000_000_000.0,
        madra_regen_bonus: 100_000_000.0,
        unlocks: &["void_touch", "non_existence"],
    },
    Tier {
        level: TierLevel::Transcendent,
        name: "???",
        description: "Beyond comprehension. Beyond description.",
        world_scope: "???",
        madra_requirement: u64::MAX,
        insight_requirement: u32::MAX,
        trial_required: true,
        power_multiplier: f64::MAX,
        madra_regen_bonus: f64::MAX,
        unlocks: &["???"],
    },
];

pub static ASPECTS: &[Aspect] = &[
    Aspect::Force,
    Aspect::Life,
    Aspect::Shadow,
    Aspect::Light,
    Aspect::Fire,
    Aspect::Water,
    Aspect::Earth,
    Aspect::Wind,
    Aspect::Time,
    Aspect::Space,
    Aspect::Dream,
    Aspect::Destruction,
    Aspect::Blood,
    Aspect::Sword,
];

pub static PATHS: &[Path] = &[
    Path {
        key: "pure_force",
        name: "Path of Pure Force",
        description: "Raw power, direct and overwhelming.",
        aspects: &[Aspect::Force],
        max_tier_natural: TierLevel::Lord,
        power_focus: 1.5,
        defense_focus: 0.8,
        speed_focus: 1.0,
        regen_focus: 1.0,
        compatible_with: &["sword_king"],
        incompatible_with: &["white_fox", "shadow_walker"],
    },
    Path {
        key: "blackflame",
        name: "Path of Blackflame",
        description: "Fire and destruction. Burns the user as well as enemies.",
        aspects: &[Aspect::Fire, Aspect::Destruction],
        max_tier_natural: TierLevel::Overlord,
        power_focus: 2.0,
        defense_focus: 0.5,
        speed_focus: 1.2,
        regen_focus: 0.7,
        compatible_with: &["void_walker"],
        incompatible_with: &["life_weaver", "white_fox"],
    },
    Path {
        key: "white_fox",
        name: "Path of the White Fox",
        description: "Light and dreams. Illusions and mind manipulation.",
        aspects: &[Aspect::Light, Aspect::Dream],
        max_tier_natural: TierLevel::Sage,
        power_focus: 0.8,
        defense_focus: 1.2,
        speed_focus: 1.5,
        regen_focus: 1.0,
        compatible_with: &["shadow_walker"],
        incompatible_with: &["blackflame", "pure_force"],
    },
    Path {
        key: "hollow_king",
        name: "Path of the Hollow King",
        description: "Shadow and force. Void attacks and impenetrable defense.",
        aspects: &[Aspect::Shadow, Aspect::Force],
        max_tier_natural: TierLevel::Monarch,
        power_focus: 1.3,
        defense_focus: 1.5,
        speed_focus: 0.9,
        regen_focus: 0.8,
        compatible_with: &["void_walker", "shadow_walker"],
        incompatible_with: &["life_weaver", "light_eternal"],
    },
    Path {
        key: "life_weaver",
        name: "Path of the Life Weaver",
        description: "Life madra. Healing and restoration.",
        aspects: &[Aspect::Life],
        max_tier_natural: TierLevel::Sage,
        power_focus: 0.6,
        defense_focus: 1.0,
        speed_focus: 0.8,
        regen_focus: 2.5,
        compatible_with: &["blood_eternal"],
        incompatible_with: &["blackflame", "hollow_king"],
    },
    Path {
        key: "shadow_walker",
        name: "Path of the Shadow Walker",
        description: "Shadow and space. Stealth and teleportation.",
        aspects: &[Aspect::Shadow, Aspect::Space],
        max_tier_natural: TierLevel::Herald,
        power_focus: 1.0,
        defense_focus: 0.9,
        speed_focus: 2.0,
        regen_focus: 0.9,
        compatible_with: &["white_fox", "hollow_king"],
        incompatible_with: &["pure_force", "light_eternal"],
    },
    Path {
        key: "sword_king",
        name: "Path of the Sword King",
        description: "Sword and force. Unmatched blade mastery.",
        aspects: &[Aspect::Sword, Aspect::Force],
        max_tier_natural: TierLevel::Monarch,
        power_focus: 1.8,
        defense_focus: 0.7,
        speed_focus: 1.3,
        regen_focus: 0.8,
        compatible_with: &["pure_force"],
        incompatible_with: &["life_weaver"],
    },
    Path {
        key: "void_walker",
        name: "Path of the Void Walker",
        description: "Space and destruction. Reality tears at your touch.",
        aspects: &[Aspect::Space, Aspect::Destruction],
        max_tier_natural: TierLevel::Dreadgod,
        power_focus: 1.6,
        defense_focus: 0.6,
        speed_focus: 1.4,
        regen_focus: 0.6,
        compatible_with: &["blackflame", "hollow_king"],
        incompatible_with: &["life_weaver", "earth_guardian"],
    },
    Path {
        key: "earth_guardian",
        name: "Path of the Earth Guardian",
        description: "Earth madra. Immovable defense.",
        aspects: &[Aspect::Earth],
        max_tier_natural: TierLevel::Lord,
        power_focus: 0.9,
        defense_focus: 2.0,
        speed_focus: 0.5,
        regen_focus: 1.2,
        compatible_with: &["life_weaver"],
        incompatible_with: &["void_walker", "wind_dancer"],
    },
    Path {
        key: "wind_dancer",
        name: "Path of the Wind Dancer",
        description: "Wind madra. Speed beyond perception.",
        aspects: &[Aspect::Wind],
        max_tier_natural: TierLevel::Overlord,
        power_focus: 0.8,
        defense_focus: 0.7,
        speed_focus: 2.5,
        regen_focus: 1.0,
        compatible_with: &["shadow_walker"],
        incompatible_with: &["earth_guardian"],
    },
    Path {
        key: "blood_eternal",
        name: "Path of Blood Eternal",
        description: "Blood madra. Life force manipulation.",
        aspects: &[Aspect::Blood, Aspect::Life],
        max_tier_natural: TierLevel::Herald,
        power_focus: 1.2,
        defense_focus: 1.0,
        speed_focus: 1.0,
        regen_focus: 2.0,
        compatible_with: &["life_weaver"],
        incompatible_with: &["light_eternal"],
    },
    Path {
        key: "light_eternal",
        name: "Path of Eternal Light",
        description: "Pure light madra. Blinding power.",
        aspects: &[Aspect::Light],
        max_tier_natural: TierLevel::Sage,
        power_focus: 1.3,
        defense_focus: 1.1,
        speed_focus: 1.2,
        regen_focus: 1.0,
        compatible_with: &["white_fox"],
        incompatible_with: &["shadow_walker", "hollow_king", "blood_eternal"],
    },
    Path {
        key: "time_keeper",
        name: "Path of the Time Keeper",
        description: "Time madra. Manipulation of the flow of time.",
        aspects: &[Aspect::Time],
        max_tier_natural: TierLevel::Judge,
        power_focus: 1.0,
        defense_focus: 1.0,
        speed_focus: 1.8,
        regen_focus: 1.5,
        compatible_with: &["void_walker"],
        incompatible_with: &[],
    },
];

pub static TECHNIQUES: &[Technique] = &[
    // Pure Force techniques
    Technique {
        key: "force_cycling",
        name: "Force Cycling",
        technique_type: TechniqueType::Cycling,
        path_key: "pure_force",
        tier_requirement: TierLevel::Copper,
        description: "Basic force madra cycling technique.",
        power_base: 10,
        madra_cost: 0,
        cooldown_ticks: 0,
    },
    Technique {
        key: "force_push",
        name: "Force Push",
        technique_type: TechniqueType::Striker,
        path_key: "pure_force",
        tier_requirement: TierLevel::Jade,
        description: "Project a wave of force at enemies.",
        power_base: 100,
        madra_cost: 50,
        cooldown_ticks: 3,
    },
    Technique {
        key: "force_armor",
        name: "Force Armor",
        technique_type: TechniqueType::Enforcer,
        path_key: "pure_force",
        tier_requirement: TierLevel::Iron,
        description: "Reinforce your body with force madra.",
        power_base: 50,
        madra_cost: 25,
        cooldown_ticks: 5,
    },
    // Blackflame techniques
    Technique {
        key: "blackflame_cycling",
        name: "Blackflame Cycling",
        technique_type: TechniqueType::Cycling,
        path_key: "blackflame",
        tier_requirement: TierLevel::Copper,
        description: "Dangerous cycling that burns as it empowers.",
        power_base: 15,
        madra_cost: 0,
        cooldown_ticks: 0,
    },
    Technique {
        key: "dragon_breath",
        name: "Dragon's Breath",
        technique_type: TechniqueType::Striker,
        path_key: "blackflame",
        tier_requirement: TierLevel::Jade,
        description: "Breathe black fire that consumes all.",
        power_base: 200,
        madra_cost: 100,
        cooldown_ticks: 5,
    },
    Technique {
        key: "burning_cloak",
        name: "Burning Cloak",
        technique_type: TechniqueType::Enforcer,
        path_key: "blackflame",
        tier_requirement: TierLevel::Gold,
        description: "Cloak yourself in destructive flames.",
        power_base: 150,
        madra_cost: 75,
        cooldown_ticks: 8,
    },
    // White Fox techniques
    Technique {
        key: "fox_cycling",
        name: "Fox Dream Cycling",
        technique_type: TechniqueType::Cycling,
        path_key: "white_fox",
        tier_requirement: TierLevel::Copper,
        description: "Cycle light and dream madra in harmony.",
        power_base: 8,
        madra_cost: 0,
        cooldown_ticks: 0,
    },
    Technique {
        key: "phantom_light",
        name: "Phantom Light",
        technique_type: TechniqueType::Forger,
        path_key: "white_fox",
        tier_requirement: TierLevel::Jade,
        description: "Create illusions indistinguishable from reality.",
        power_base: 75,
        madra_cost: 60,
        cooldown_ticks: 4,
    },
    // Hollow King techniques
    Technique {
        key: "hollow_cycling",
        name: "Hollow King Cycling",
        technique_type: TechniqueType::Cycling,
        path_key: "hollow_king",
        tier_requirement: TierLevel::Copper,
        description: "Draw power from the void between shadows.",
        power_base: 12,
        madra_cost: 0,
        cooldown_ticks: 0,
    },
    Technique {
        key: "void_strike",
        name: "Void Strike",
        technique_type: TechniqueType::Striker,
        path_key: "hollow_king",
        tier_requirement: TierLevel::Jade,
        description: "Strike with the nothingness between worlds.",
        power_base: 180,
        madra_cost: 90,
        cooldown_ticks: 6,
    },
];

pub static MENTORS: &[Mentor] = &[
    Mentor {
        key: "elder_wei",
        name: "Elder Wei Shi",
        tier_range: (TierLevel::Unsouled, TierLevel::Iron),
        specialization: &[Aspect::Force],
        personality: "Patient village elder who speaks in proverbs.",
        hints: &[
            "The path of a thousand miles begins with a single step.",
            "Do not scatter your focus. Master one technique before learning another.",
            "Cycling during sleep accelerates advancement.",
            "Your foundation determines your ceiling. Build it strong.",
        ],
    },
    Mentor {
        key: "master_chen",
        name: "Master Chen Blackheart",
        tier_range: (TierLevel::Jade, TierLevel::Gold),
        specialization: &[Aspect::Fire, Aspect::Destruction],
        personality: "Harsh but effective trainer who demands excellence.",
        hints: &[
            "Power without control is nothing but self-destruction.",
            "The Blackflame path burns those who fear commitment.",
            "You must embrace the flames, not fight them.",
            "Combining incompatible paths leads only to stagnation.",
        ],
    },
    Mentor {
        key: "sage_akura",
        name: "Sage Akura Mercy",
        tier_range: (TierLevel::Lord, TierLevel::Overlord),
        specialization: &[Aspect::Shadow, Aspect::Dream],
        personality: "Mysterious figure who teaches through riddles.",
        hints: &[
            "The shadow and the light are two faces of the same coin.",
            "Your revelation must be true to your nature.",
            "Soulfire comes to those who know themselves completely.",
            "The path to Overlord requires accepting what you are.",
        ],
    },
    Mentor {
        key: "makiel_guide",
        name: "Makiel's Echo",
        tier_range: (TierLevel::Sage, TierLevel::Monarch),
        specialization: &[Aspect::Time, Aspect::Space],
        personality: "Fragment of an Abidan guide, speaks of cosmic truths.",
        hints: &[
            "The Way is the foundation of all existence.",
            "Icons are not claimed - they recognize you.",
            "A Monarch combines the enlightenment of a Sage with the power of a Herald.",
            "The Dreadgods offer power at a terrible cost.",
        ],
    },
    Mentor {
        key: "void_whisper",
        name: "The Void Whispers",
        tier_range: (TierLevel::Dreadgod, TierLevel::Transcendent),
        specialization: &[],
        personality: "Cryptic, dangerous wisdom from beyond existence.",
        hints: &[
            "...existence is a prison you chose to enter...",
            "...the end is merely another beginning...",
            "...you are already more than you know...",
            "...when you see the truth, you will unmake yourself...",
        ],
    },
];

// ============================================================================
// LOOKUP FUNCTIONS
// ============================================================================

pub fn get_tier(level: TierLevel) -> Option<&'static Tier> {
    TIERS.iter().find(|t| t.level == level)
}

pub fn get_path(key: &str) -> Option<&'static Path> {
    PATHS.iter().find(|p| p.key == key)
}

pub fn get_technique(key: &str) -> Option<&'static Technique> {
    TECHNIQUES.iter().find(|t| t.key == key)
}

pub fn get_mentor(key: &str) -> Option<&'static Mentor> {
    MENTORS.iter().find(|m| m.key == key)
}

pub fn get_aspect(name: &str) -> Option<Aspect> {
    ASPECTS.iter().find(|a| a.name().to_lowercase() == name.to_lowercase()).copied()
}

pub fn get_mentors_for_tier(tier: TierLevel) -> Vec<&'static Mentor> {
    MENTORS.iter().filter(|m| {
        tier >= m.tier_range.0 && tier <= m.tier_range.1
    }).collect()
}

pub fn get_techniques_for_path(path_key: &str) -> Vec<&'static Technique> {
    TECHNIQUES.iter().filter(|t| t.path_key == path_key).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_ordering() {
        assert!(TierLevel::Copper > TierLevel::Unsouled);
        assert!(TierLevel::Monarch > TierLevel::Sage);
        assert!(TierLevel::Transcendent > TierLevel::Void);
    }

    #[test]
    fn test_tier_lookup() {
        let tier = get_tier(TierLevel::Gold);
        assert!(tier.is_some());
        assert_eq!(tier.unwrap().name, "Gold");
    }

    #[test]
    fn test_path_lookup() {
        let path = get_path("blackflame");
        assert!(path.is_some());
        assert_eq!(path.unwrap().name, "Path of Blackflame");
        assert!(path.unwrap().aspects.contains(&Aspect::Fire));
    }

    #[test]
    fn test_mentor_for_tier() {
        let mentors = get_mentors_for_tier(TierLevel::Iron);
        assert!(!mentors.is_empty());
        assert!(mentors.iter().any(|m| m.key == "elder_wei"));
    }

    #[test]
    fn test_techniques_for_path() {
        let techniques = get_techniques_for_path("blackflame");
        assert!(!techniques.is_empty());
        assert!(techniques.iter().any(|t| t.key == "dragon_breath"));
    }

    #[test]
    fn test_all_tiers_have_data() {
        for i in 0..=15 {
            let tier = TierLevel::from_u8(i).unwrap();
            assert!(get_tier(tier).is_some(), "Missing data for tier {}", i);
        }
    }

    #[test]
    fn test_path_compatibility() {
        let blackflame = get_path("blackflame").unwrap();
        assert!(blackflame.incompatible_with.contains(&"life_weaver"));
        assert!(blackflame.compatible_with.contains(&"void_walker"));
    }
}
