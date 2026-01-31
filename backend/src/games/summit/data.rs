//! Static game data for Summit
//!
//! Contains all the items, foods, hazards, badges, and cosmetics.

use serde::{Deserialize, Serialize};

// ============================================================================
// BIOMES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Beach,
    Jungle,
    Alpine,
    Volcanic,
}

impl BiomeType {
    pub fn name(&self) -> &'static str {
        match self {
            BiomeType::Beach => "Beach",
            BiomeType::Jungle => "Jungle",
            BiomeType::Alpine => "Alpine",
            BiomeType::Volcanic => "Volcanic",
        }
    }

    pub fn difficulty(&self) -> &'static str {
        match self {
            BiomeType::Beach => "Easy",
            BiomeType::Jungle => "Medium",
            BiomeType::Alpine => "Hard",
            BiomeType::Volcanic => "Extreme",
        }
    }

    pub fn stamina_drain_rate(&self) -> u32 {
        match self {
            BiomeType::Beach => 1,
            BiomeType::Jungle => 2,
            BiomeType::Alpine => 3,
            BiomeType::Volcanic => 4,
        }
    }

    pub fn fall_damage_multiplier(&self) -> f32 {
        match self {
            BiomeType::Beach => 0.5,
            BiomeType::Jungle => 1.0,
            BiomeType::Alpine => 1.5,
            BiomeType::Volcanic => 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Biome {
    pub biome_type: BiomeType,
    pub height_start: u32,
    pub height_end: u32,
    pub description: &'static str,
}

pub const BIOMES: [Biome; 4] = [
    Biome {
        biome_type: BiomeType::Beach,
        height_start: 0,
        height_end: 25,
        description: "Sandy cliffs, palm trees, gentle slopes. Tutorial area.",
    },
    Biome {
        biome_type: BiomeType::Jungle,
        height_start: 25,
        height_end: 50,
        description: "Dense vegetation, vines, waterfalls. Slippery surfaces.",
    },
    Biome {
        biome_type: BiomeType::Alpine,
        height_start: 50,
        height_end: 75,
        description: "Snow, ice, exposed rock faces. Cold status if not moving.",
    },
    Biome {
        biome_type: BiomeType::Volcanic,
        height_start: 75,
        height_end: 100,
        description: "Unstable terrain, lava vents, ash clouds. Final push!",
    },
];

// ============================================================================
// CLIMBING ITEMS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Rope,
    Piton,
    RopeCannon,
    GrapplingHook,
    ClimbingGloves,
    SafetyHarness,
    AntiRope,
    Marshmallow,
    Chocolate,
    GrahamCracker,
}

#[derive(Debug, Clone)]
pub struct ClimbingItem {
    pub item_type: ItemType,
    pub name: &'static str,
    pub description: &'static str,
    pub uses: Option<u32>,
    pub rarity: u32, // 1-100, lower = rarer
}

pub const CLIMBING_ITEMS: [ClimbingItem; 10] = [
    ClimbingItem {
        item_type: ItemType::Rope,
        name: "Rope",
        description: "Deploy to create a climbable path down from attachment point.",
        uses: Some(1),
        rarity: 80,
    },
    ClimbingItem {
        item_type: ItemType::Piton,
        name: "Piton",
        description: "Place on any surface to create a rest point with fast stamina regen.",
        uses: Some(1),
        rarity: 60,
    },
    ClimbingItem {
        item_type: ItemType::RopeCannon,
        name: "Rope Cannon",
        description: "Fire rope at distant anchor point. Creates long-distance path.",
        uses: Some(3),
        rarity: 15,
    },
    ClimbingItem {
        item_type: ItemType::GrapplingHook,
        name: "Grappling Hook",
        description: "Pull yourself to anchor points. Can grab distant items.",
        uses: Some(5),
        rarity: 20,
    },
    ClimbingItem {
        item_type: ItemType::ClimbingGloves,
        name: "Climbing Gloves",
        description: "Reduce stamina cost on rough surfaces. Prevent slipping on ice.",
        uses: Some(50),
        rarity: 40,
    },
    ClimbingItem {
        item_type: ItemType::SafetyHarness,
        name: "Safety Harness",
        description: "Reduces fall damage by 50%. Won't prevent falls.",
        uses: None,
        rarity: 30,
    },
    ClimbingItem {
        item_type: ItemType::AntiRope,
        name: "Anti-Rope",
        description: "???",
        uses: Some(1),
        rarity: 5,
    },
    ClimbingItem {
        item_type: ItemType::Marshmallow,
        name: "Marshmallow",
        description: "Roast at campfire for stamina boost.",
        uses: Some(1),
        rarity: 70,
    },
    ClimbingItem {
        item_type: ItemType::Chocolate,
        name: "Chocolate Bar",
        description: "Combine with marshmallow and graham cracker for s'more.",
        uses: Some(1),
        rarity: 50,
    },
    ClimbingItem {
        item_type: ItemType::GrahamCracker,
        name: "Graham Cracker",
        description: "Essential s'more ingredient.",
        uses: Some(1),
        rarity: 50,
    },
];

pub fn get_item(item_type: ItemType) -> Option<&'static ClimbingItem> {
    CLIMBING_ITEMS.iter().find(|i| i.item_type == item_type)
}

// ============================================================================
// FOODS (30 Questionable Foods)
// ============================================================================

#[derive(Debug, Clone)]
pub struct FoodEffect {
    pub stamina_current: i32,      // Instant stamina change
    pub stamina_max: i32,          // Permanent max stamina change
    pub stamina_regen: i32,        // Regen rate modifier (duration in ticks)
    pub hunger_relief: i32,        // 0-100
    pub cure_cold: bool,
    pub cure_poison: bool,
    pub side_effect: Option<&'static str>,
    pub poison_chance: u32,        // 0-100
    pub special: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct Food {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub effect: FoodEffect,
    pub rarity: u32,               // 1-100, lower = rarer
    pub requires_campfire: bool,
}

pub const FOODS: [Food; 30] = [
    // ENERGY & STAMINA (1-5)
    Food {
        id: 1,
        name: "Energy Drink",
        description: "Instant 50% stamina, crash after 60s.",
        effect: FoodEffect {
            stamina_current: 50, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 0, cure_cold: false, cure_poison: false,
            side_effect: Some("Crash: -30% stamina after 60s"),
            poison_chance: 0, special: None,
        },
        rarity: 60,
        requires_campfire: false,
    },
    Food {
        id: 2,
        name: "Sports Gel",
        description: "Steady stamina regen for 30s.",
        effect: FoodEffect {
            stamina_current: 10, stamina_max: 0, stamina_regen: 30,
            hunger_relief: 0, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 65,
        requires_campfire: false,
    },
    Food {
        id: 3,
        name: "Coffee Thermos",
        description: "Stamina regen boost, but jitters.",
        effect: FoodEffect {
            stamina_current: 20, stamina_max: 0, stamina_regen: 45,
            hunger_relief: 10, cure_cold: true, cure_poison: false,
            side_effect: Some("Jitters: shaky movement for 30s"),
            poison_chance: 0, special: None,
        },
        rarity: 50,
        requires_campfire: false,
    },
    Food {
        id: 4,
        name: "Protein Bar",
        description: "25% stamina, satisfies hunger.",
        effect: FoodEffect {
            stamina_current: 25, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 60, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 70,
        requires_campfire: false,
    },
    Food {
        id: 5,
        name: "Trail Mix",
        description: "20% stamina, light hunger relief.",
        effect: FoodEffect {
            stamina_current: 20, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 40, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 80,
        requires_campfire: false,
    },
    // HUNGER & HEALING (6-10)
    Food {
        id: 6,
        name: "Mystery Meat",
        description: "30% stamina, hunger relief, 20% food poisoning risk.",
        effect: FoodEffect {
            stamina_current: 30, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 80, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 20, special: None,
        },
        rarity: 55,
        requires_campfire: false,
    },
    Food {
        id: 7,
        name: "Canned Beans",
        description: "Full hunger relief, makes noise.",
        effect: FoodEffect {
            stamina_current: 15, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 100, cure_cold: false, cure_poison: false,
            side_effect: Some("Gas: alerts nearby hazards"),
            poison_chance: 0, special: None,
        },
        rarity: 60,
        requires_campfire: false,
    },
    Food {
        id: 8,
        name: "Dried Fruit",
        description: "Cures hunger, safe choice.",
        effect: FoodEffect {
            stamina_current: 10, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 70, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 75,
        requires_campfire: false,
    },
    Food {
        id: 9,
        name: "Beef Jerky",
        description: "Hunger relief plus 15% stamina.",
        effect: FoodEffect {
            stamina_current: 15, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 65, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 65,
        requires_campfire: false,
    },
    Food {
        id: 10,
        name: "MRE Pack",
        description: "Full hunger, 40% stamina, but heavy.",
        effect: FoodEffect {
            stamina_current: 40, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 100, cure_cold: false, cure_poison: false,
            side_effect: Some("Heavy: slows movement for 60s"),
            poison_chance: 0, special: None,
        },
        rarity: 35,
        requires_campfire: false,
    },
    // SPECIAL EFFECTS (11-14)
    Food {
        id: 11,
        name: "Lollipop",
        description: "Unlimited stamina 15s, severe exhaustion after.",
        effect: FoodEffect {
            stamina_current: 100, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 10, cure_cold: false, cure_poison: false,
            side_effect: Some("Exhaustion: no stamina regen for 30s"),
            poison_chance: 0, special: Some("unlimited_stamina_15s"),
        },
        rarity: 25,
        requires_campfire: false,
    },
    Food {
        id: 12,
        name: "Milk",
        description: "10s invulnerability. Extremely rare.",
        effect: FoodEffect {
            stamina_current: 20, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 30, cure_cold: false, cure_poison: true,
            side_effect: None, poison_chance: 0,
            special: Some("invulnerable_10s"),
        },
        rarity: 5,
        requires_campfire: false,
    },
    Food {
        id: 13,
        name: "Hot Cocoa",
        description: "Cures cold, 25% stamina, cozy feeling.",
        effect: FoodEffect {
            stamina_current: 25, stamina_max: 0, stamina_regen: 15,
            hunger_relief: 20, cure_cold: true, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 40,
        requires_campfire: true,
    },
    Food {
        id: 14,
        name: "Warm Soup",
        description: "Cures cold, hunger, and 30% stamina.",
        effect: FoodEffect {
            stamina_current: 30, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 80, cure_cold: true, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 35,
        requires_campfire: true,
    },
    // QUESTIONABLE CHOICES (15-20)
    Food {
        id: 15,
        name: "Strange Mushroom",
        description: "May restore 20% MAX stamina, or hallucinate.",
        effect: FoodEffect {
            stamina_current: 10, stamina_max: 20, stamina_regen: 0,
            hunger_relief: 10, cure_cold: false, cure_poison: false,
            side_effect: Some("Hallucination: vision distortion 30s"),
            poison_chance: 40, special: Some("restore_max_stamina"),
        },
        rarity: 15,
        requires_campfire: false,
    },
    Food {
        id: 16,
        name: "Suspicious Berry",
        description: "Random: healing OR poison OR energy boost.",
        effect: FoodEffect {
            stamina_current: 0, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 20, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 33,
            special: Some("random_effect"),
        },
        rarity: 50,
        requires_campfire: false,
    },
    Food {
        id: 17,
        name: "Mystery Can",
        description: "Could be anything.",
        effect: FoodEffect {
            stamina_current: 0, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 0, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0,
            special: Some("mystery_can"),
        },
        rarity: 45,
        requires_campfire: false,
    },
    Food {
        id: 18,
        name: "Gas Station Sushi",
        description: "50% stamina if it doesn't kill you (50/50).",
        effect: FoodEffect {
            stamina_current: 50, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 70, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 50,
            special: Some("high_risk_high_reward"),
        },
        rarity: 30,
        requires_campfire: false,
    },
    Food {
        id: 19,
        name: "Week-Old Sandwich",
        description: "Desperate times. 30% poison chance.",
        effect: FoodEffect {
            stamina_current: 20, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 60, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 30, special: None,
        },
        rarity: 55,
        requires_campfire: false,
    },
    Food {
        id: 20,
        name: "Found Candy",
        description: "Small stamina boost, probably fine.",
        effect: FoodEffect {
            stamina_current: 15, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 15, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 5, special: None,
        },
        rarity: 70,
        requires_campfire: false,
    },
    // CAMPFIRE SPECIALS (21-24)
    Food {
        id: 21,
        name: "Roasted Marshmallow",
        description: "15% stamina, morale boost.",
        effect: FoodEffect {
            stamina_current: 15, stamina_max: 0, stamina_regen: 10,
            hunger_relief: 10, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 0, // Created from marshmallow item
        requires_campfire: true,
    },
    Food {
        id: 22,
        name: "S'more",
        description: "25% stamina, requires chocolate + graham + marshmallow.",
        effect: FoodEffect {
            stamina_current: 25, stamina_max: 0, stamina_regen: 20,
            hunger_relief: 30, cure_cold: true, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 0, // Crafted
        requires_campfire: true,
    },
    Food {
        id: 23,
        name: "Hot Dog",
        description: "Hunger relief, 20% stamina.",
        effect: FoodEffect {
            stamina_current: 20, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 75, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 40,
        requires_campfire: true,
    },
    Food {
        id: 24,
        name: "Campfire Coffee",
        description: "Major stamina regen, requires rest.",
        effect: FoodEffect {
            stamina_current: 10, stamina_max: 0, stamina_regen: 60,
            hunger_relief: 5, cure_cold: true, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 35,
        requires_campfire: true,
    },
    // RARE & EXOTIC (25-30)
    Food {
        id: 25,
        name: "Golden Apple",
        description: "Restores 30% MAX stamina. Legendary.",
        effect: FoodEffect {
            stamina_current: 50, stamina_max: 30, stamina_regen: 0,
            hunger_relief: 50, cure_cold: true, cure_poison: true,
            side_effect: None, poison_chance: 0,
            special: Some("restore_max_stamina"),
        },
        rarity: 3,
        requires_campfire: false,
    },
    Food {
        id: 26,
        name: "Ancient Ration",
        description: "Full restore but tastes terrible.",
        effect: FoodEffect {
            stamina_current: 100, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 100, cure_cold: true, cure_poison: true,
            side_effect: Some("Disgust: movement penalty 10s"),
            poison_chance: 0, special: None,
        },
        rarity: 10,
        requires_campfire: false,
    },
    Food {
        id: 27,
        name: "Glowing Fruit",
        description: "??? Found only in volcanic biome.",
        effect: FoodEffect {
            stamina_current: 0, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 0, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0,
            special: Some("glowing_mystery"),
        },
        rarity: 8,
        requires_campfire: false,
    },
    Food {
        id: 28,
        name: "Scout's Emergency Chocolate",
        description: "Clutch 40% stamina restore.",
        effect: FoodEffect {
            stamina_current: 40, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 30, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0, special: None,
        },
        rarity: 20,
        requires_campfire: false,
    },
    Food {
        id: 29,
        name: "Mystery Pill",
        description: "Massive gamble - miracle or disaster.",
        effect: FoodEffect {
            stamina_current: 0, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 0, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0,
            special: Some("mystery_pill"),
        },
        rarity: 12,
        requires_campfire: false,
    },
    Food {
        id: 30,
        name: "The Forbidden Snack",
        description: "Found at Summit, grants badge. Effects unknown.",
        effect: FoodEffect {
            stamina_current: 0, stamina_max: 0, stamina_regen: 0,
            hunger_relief: 0, cure_cold: false, cure_poison: false,
            side_effect: None, poison_chance: 0,
            special: Some("forbidden_snack"),
        },
        rarity: 1,
        requires_campfire: false,
    },
];

pub fn get_food(id: u32) -> Option<&'static Food> {
    FOODS.iter().find(|f| f.id == id)
}

// ============================================================================
// HAZARDS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HazardType {
    // Beach
    Crab,
    LooseSand,
    TidePool,
    Seagull,
    // Jungle
    PoisonPlant,
    Snake,
    SuddenDrop,
    SlipperyVine,
    // Alpine
    WindGust,
    IceSlide,
    BrittleRock,
    Avalanche,
    ExtremeCold,
    // Volcanic
    SteamVent,
    CollapsingPlatform,
    LavaFlow,
    AshCloud,
    Eruption,
}

#[derive(Debug, Clone)]
pub struct Hazard {
    pub hazard_type: HazardType,
    pub name: &'static str,
    pub damage: u32,           // Stamina damage
    pub max_damage: u32,       // Max stamina damage
    pub duration: u32,         // Effect duration in ticks
    pub biome: BiomeType,
}

pub const HAZARDS: [Hazard; 18] = [
    // Beach
    Hazard { hazard_type: HazardType::Crab, name: "Crab", damage: 5, max_damage: 2, duration: 0, biome: BiomeType::Beach },
    Hazard { hazard_type: HazardType::LooseSand, name: "Loose Sand", damage: 0, max_damage: 0, duration: 30, biome: BiomeType::Beach },
    Hazard { hazard_type: HazardType::TidePool, name: "Tide Pool", damage: 3, max_damage: 0, duration: 0, biome: BiomeType::Beach },
    Hazard { hazard_type: HazardType::Seagull, name: "Seagull", damage: 0, max_damage: 0, duration: 0, biome: BiomeType::Beach },
    // Jungle
    Hazard { hazard_type: HazardType::PoisonPlant, name: "Poison Plant", damage: 5, max_damage: 0, duration: 60, biome: BiomeType::Jungle },
    Hazard { hazard_type: HazardType::Snake, name: "Snake", damage: 10, max_damage: 5, duration: 45, biome: BiomeType::Jungle },
    Hazard { hazard_type: HazardType::SuddenDrop, name: "Hidden Cliff", damage: 20, max_damage: 10, duration: 0, biome: BiomeType::Jungle },
    Hazard { hazard_type: HazardType::SlipperyVine, name: "Slippery Vine", damage: 10, max_damage: 5, duration: 0, biome: BiomeType::Jungle },
    // Alpine
    Hazard { hazard_type: HazardType::WindGust, name: "Wind Gust", damage: 15, max_damage: 5, duration: 0, biome: BiomeType::Alpine },
    Hazard { hazard_type: HazardType::IceSlide, name: "Ice Slide", damage: 25, max_damage: 10, duration: 0, biome: BiomeType::Alpine },
    Hazard { hazard_type: HazardType::BrittleRock, name: "Brittle Rock", damage: 20, max_damage: 8, duration: 0, biome: BiomeType::Alpine },
    Hazard { hazard_type: HazardType::Avalanche, name: "Avalanche", damage: 50, max_damage: 25, duration: 0, biome: BiomeType::Alpine },
    Hazard { hazard_type: HazardType::ExtremeCold, name: "Extreme Cold", damage: 1, max_damage: 1, duration: 20, biome: BiomeType::Alpine },
    // Volcanic
    Hazard { hazard_type: HazardType::SteamVent, name: "Steam Vent", damage: 15, max_damage: 5, duration: 0, biome: BiomeType::Volcanic },
    Hazard { hazard_type: HazardType::CollapsingPlatform, name: "Collapsing Platform", damage: 30, max_damage: 15, duration: 0, biome: BiomeType::Volcanic },
    Hazard { hazard_type: HazardType::LavaFlow, name: "Lava Flow", damage: 100, max_damage: 50, duration: 0, biome: BiomeType::Volcanic },
    Hazard { hazard_type: HazardType::AshCloud, name: "Ash Cloud", damage: 5, max_damage: 2, duration: 30, biome: BiomeType::Volcanic },
    Hazard { hazard_type: HazardType::Eruption, name: "Eruption", damage: 40, max_damage: 20, duration: 0, biome: BiomeType::Volcanic },
];

// ============================================================================
// BADGES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeCategory {
    Progression,
    Skill,
    Food,
    Discovery,
    Challenge,
}

#[derive(Debug, Clone)]
pub struct Badge {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: BadgeCategory,
    pub icon: char,
}

pub const BADGES: [Badge; 20] = [
    // Progression
    Badge { id: "first_summit", name: "First Summit", description: "Complete any mountain", category: BadgeCategory::Progression, icon: 'M' },
    Badge { id: "veteran", name: "Veteran Scout", description: "Complete 10 mountains", category: BadgeCategory::Progression, icon: 'V' },
    Badge { id: "master", name: "Mountain Master", description: "Complete 50 mountains", category: BadgeCategory::Progression, icon: '*' },
    Badge { id: "all_biomes", name: "All Biomes", description: "Reach all 4 biomes in one run", category: BadgeCategory::Progression, icon: '4' },
    Badge { id: "true_scout", name: "True Scout", description: "Complete with no deaths", category: BadgeCategory::Progression, icon: 'T' },
    // Skill
    Badge { id: "speed_climber", name: "Speed Climber", description: "Summit in under 15 minutes", category: BadgeCategory::Skill, icon: 'S' },
    Badge { id: "featherfoot", name: "Featherfoot", description: "Summit without falling", category: BadgeCategory::Skill, icon: 'F' },
    Badge { id: "solo_summit", name: "Solo Summit", description: "Complete alone", category: BadgeCategory::Skill, icon: '1' },
    Badge { id: "team_player", name: "Team Player", description: "Revive 10 teammates total", category: BadgeCategory::Skill, icon: '+' },
    Badge { id: "trailblazer", name: "Trailblazer", description: "Place 100 ropes total", category: BadgeCategory::Skill, icon: 'R' },
    // Food
    Badge { id: "adventurous", name: "Adventurous Eater", description: "Try 15 different foods", category: BadgeCategory::Food, icon: 'E' },
    Badge { id: "iron_stomach", name: "Iron Stomach", description: "Try all 30 foods", category: BadgeCategory::Food, icon: 'I' },
    Badge { id: "survivor", name: "Survivor", description: "Recover from food poisoning 5 times", category: BadgeCategory::Food, icon: 'P' },
    Badge { id: "marshmallow_master", name: "Marshmallow Master", description: "Perfect roast 20 times", category: BadgeCategory::Food, icon: 'm' },
    Badge { id: "smore_connoisseur", name: "S'more Connoisseur", description: "Make 10 s'mores", category: BadgeCategory::Food, icon: 's' },
    // Discovery
    Badge { id: "explorer", name: "Explorer", description: "Find a secret area", category: BadgeCategory::Discovery, icon: '?' },
    Badge { id: "scavenger", name: "Scavenger", description: "Open 100 luggage containers", category: BadgeCategory::Discovery, icon: 'L' },
    Badge { id: "anti_rope_user", name: "Anti-Rope User", description: "Discover what Anti-Rope does", category: BadgeCategory::Discovery, icon: 'A' },
    Badge { id: "lava_surfer", name: "Lava Surfer", description: "Survive volcanic biome hazard", category: BadgeCategory::Discovery, icon: '~' },
    Badge { id: "forbidden_snack", name: "The Forbidden Snack", description: "Eat it at the Summit", category: BadgeCategory::Discovery, icon: '!' },
];

pub fn get_badge(id: &str) -> Option<&'static Badge> {
    BADGES.iter().find(|b| b.id == id)
}

// ============================================================================
// COSMETICS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CosmeticType {
    Uniform,
    Hat,
    Backpack,
    Accessory,
    RopeColor,
}

#[derive(Debug, Clone)]
pub struct Cosmetic {
    pub id: &'static str,
    pub name: &'static str,
    pub cosmetic_type: CosmeticType,
    pub unlock_requirement: Option<&'static str>, // Badge ID or condition
    pub character: char,                           // ASCII representation
}

pub const COSMETICS: [Cosmetic; 20] = [
    // Uniforms
    Cosmetic { id: "uniform_green", name: "Classic Green", cosmetic_type: CosmeticType::Uniform, unlock_requirement: None, character: 'G' },
    Cosmetic { id: "uniform_brown", name: "Wilderness Brown", cosmetic_type: CosmeticType::Uniform, unlock_requirement: Some("first_summit"), character: 'B' },
    Cosmetic { id: "uniform_white", name: "Arctic White", cosmetic_type: CosmeticType::Uniform, unlock_requirement: Some("all_biomes"), character: 'W' },
    Cosmetic { id: "uniform_red", name: "Volcanic Red", cosmetic_type: CosmeticType::Uniform, unlock_requirement: Some("lava_surfer"), character: 'R' },
    Cosmetic { id: "uniform_gold", name: "Golden Scout", cosmetic_type: CosmeticType::Uniform, unlock_requirement: Some("master"), character: '*' },
    // Hats
    Cosmetic { id: "hat_cap", name: "Scout Cap", cosmetic_type: CosmeticType::Hat, unlock_requirement: None, character: '^' },
    Cosmetic { id: "hat_lamp", name: "Headlamp", cosmetic_type: CosmeticType::Hat, unlock_requirement: Some("veteran"), character: 'o' },
    Cosmetic { id: "hat_helmet", name: "Climbing Helmet", cosmetic_type: CosmeticType::Hat, unlock_requirement: Some("featherfoot"), character: 'O' },
    Cosmetic { id: "hat_bandana", name: "Bandana", cosmetic_type: CosmeticType::Hat, unlock_requirement: Some("speed_climber"), character: '~' },
    Cosmetic { id: "hat_hood", name: "Mysterious Hood", cosmetic_type: CosmeticType::Hat, unlock_requirement: Some("anti_rope_user"), character: 'n' },
    // Backpacks
    Cosmetic { id: "pack_standard", name: "Standard Issue", cosmetic_type: CosmeticType::Backpack, unlock_requirement: None, character: 'D' },
    Cosmetic { id: "pack_large", name: "Oversized", cosmetic_type: CosmeticType::Backpack, unlock_requirement: Some("scavenger"), character: 'Q' },
    Cosmetic { id: "pack_minimal", name: "Minimalist", cosmetic_type: CosmeticType::Backpack, unlock_requirement: Some("solo_summit"), character: 'd' },
    Cosmetic { id: "pack_badges", name: "Badge Display", cosmetic_type: CosmeticType::Backpack, unlock_requirement: Some("master"), character: '#' },
    // Accessories
    Cosmetic { id: "acc_sunglasses", name: "Sunglasses", cosmetic_type: CosmeticType::Accessory, unlock_requirement: Some("first_summit"), character: '8' },
    Cosmetic { id: "acc_goggles", name: "Goggles", cosmetic_type: CosmeticType::Accessory, unlock_requirement: Some("all_biomes"), character: '%' },
    Cosmetic { id: "acc_scarf", name: "Scarf", cosmetic_type: CosmeticType::Accessory, unlock_requirement: Some("true_scout"), character: '~' },
    // Rope Colors
    Cosmetic { id: "rope_tan", name: "Standard Tan", cosmetic_type: CosmeticType::RopeColor, unlock_requirement: None, character: '|' },
    Cosmetic { id: "rope_orange", name: "Safety Orange", cosmetic_type: CosmeticType::RopeColor, unlock_requirement: Some("trailblazer"), character: '!' },
    Cosmetic { id: "rope_rainbow", name: "Rainbow", cosmetic_type: CosmeticType::RopeColor, unlock_requirement: Some("iron_stomach"), character: '$' },
];

pub fn get_cosmetic(id: &str) -> Option<&'static Cosmetic> {
    COSMETICS.iter().find(|c| c.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_foods_defined() {
        assert_eq!(FOODS.len(), 30);
        for (i, food) in FOODS.iter().enumerate() {
            assert_eq!(food.id, (i + 1) as u32, "Food {} has wrong id", food.name);
        }
    }

    #[test]
    fn test_biome_ordering() {
        assert!(BiomeType::Beach.stamina_drain_rate() < BiomeType::Jungle.stamina_drain_rate());
        assert!(BiomeType::Jungle.stamina_drain_rate() < BiomeType::Alpine.stamina_drain_rate());
        assert!(BiomeType::Alpine.stamina_drain_rate() < BiomeType::Volcanic.stamina_drain_rate());
    }

    #[test]
    fn test_get_item() {
        assert!(get_item(ItemType::Rope).is_some());
        assert_eq!(get_item(ItemType::Rope).unwrap().name, "Rope");
    }

    #[test]
    fn test_get_food() {
        assert!(get_food(1).is_some());
        assert_eq!(get_food(1).unwrap().name, "Energy Drink");
        assert!(get_food(30).is_some());
        assert_eq!(get_food(30).unwrap().name, "The Forbidden Snack");
    }

    #[test]
    fn test_get_badge() {
        assert!(get_badge("first_summit").is_some());
        assert_eq!(get_badge("first_summit").unwrap().name, "First Summit");
    }

    #[test]
    fn test_hazards_per_biome() {
        let beach_hazards: Vec<_> = HAZARDS.iter().filter(|h| h.biome == BiomeType::Beach).collect();
        let jungle_hazards: Vec<_> = HAZARDS.iter().filter(|h| h.biome == BiomeType::Jungle).collect();
        let alpine_hazards: Vec<_> = HAZARDS.iter().filter(|h| h.biome == BiomeType::Alpine).collect();
        let volcanic_hazards: Vec<_> = HAZARDS.iter().filter(|h| h.biome == BiomeType::Volcanic).collect();

        assert!(!beach_hazards.is_empty());
        assert!(!jungle_hazards.is_empty());
        assert!(!alpine_hazards.is_empty());
        assert!(!volcanic_hazards.is_empty());
    }
}
