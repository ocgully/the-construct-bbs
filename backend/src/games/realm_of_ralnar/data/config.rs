//! Game configuration constants for Realm of Ralnar
//!
//! Contains display settings, party limits, combat formulas, XP tables, and stat growth.

// ============================================================================
// DISPLAY SETTINGS
// ============================================================================

/// Screen width in characters (BBS terminal width)
pub const SCREEN_WIDTH: u32 = 80;

/// Screen height in characters (BBS terminal height)
pub const SCREEN_HEIGHT: u32 = 24;

/// Native game resolution width (Mode 13h)
pub const NATIVE_WIDTH: u32 = 320;

/// Native game resolution height (Mode 13h)
pub const NATIVE_HEIGHT: u32 = 200;

/// Tile width in pixels
pub const TILE_WIDTH: u32 = 20;

/// Tile height in pixels
pub const TILE_HEIGHT: u32 = 20;

/// Visible area in tiles (horizontal)
pub const VIEW_WIDTH: u32 = 16;

/// Visible area in tiles (vertical)
pub const VIEW_HEIGHT: u32 = 10;

// ============================================================================
// PARTY SETTINGS
// ============================================================================

/// Maximum number of active party members
pub const MAX_PARTY_SIZE: usize = 4;

/// Starting gold for a new game
pub const STARTING_GOLD: u32 = 100;

/// Maximum inventory slots
pub const MAX_INVENTORY_SLOTS: usize = 20;

/// Brother stat bonus when both are in party (percentage)
pub const BROTHER_TOGETHER_BONUS: f32 = 0.10;

/// Protective fury bonus when brother is low HP (percentage)
pub const PROTECTIVE_FURY_BONUS: f32 = 0.25;

/// HP threshold for protective fury activation
pub const PROTECTIVE_FURY_THRESHOLD: f32 = 0.25;

/// Separation penalty when brothers are split (percentage)
pub const SEPARATION_PENALTY: f32 = 0.10;

/// Reunion boost when brothers reunite (percentage)
pub const REUNION_BOOST: f32 = 0.20;

// ============================================================================
// COMBAT SETTINGS
// ============================================================================

/// Base chance to hit (before modifiers)
pub const BASE_HIT_CHANCE: f32 = 0.85;

/// Base critical hit chance
pub const CRITICAL_CHANCE: f32 = 0.05;

/// Critical hit damage multiplier
pub const CRITICAL_MULTIPLIER: f32 = 2.0;

/// Base chance to flee from combat
pub const FLEE_BASE_CHANCE: f32 = 0.50;

/// Minimum hit chance (can't go below this)
pub const MIN_HIT_CHANCE: f32 = 0.10;

/// Maximum hit chance (can't go above this)
pub const MAX_HIT_CHANCE: f32 = 0.95;

/// Damage variance percentage (plus or minus)
pub const DAMAGE_VARIANCE: f32 = 0.25;

/// Minimum damage dealt (always at least 1)
pub const MIN_DAMAGE: i32 = 1;

/// Elemental weakness damage multiplier
pub const ELEMENTAL_WEAKNESS_MULTIPLIER: f32 = 1.5;

/// Elemental resistance damage multiplier
pub const ELEMENTAL_RESISTANCE_MULTIPLIER: f32 = 0.5;

// ============================================================================
// LEVEL AND XP SETTINGS
// ============================================================================

/// Maximum character level
pub const MAX_LEVEL: u8 = 99;

/// XP table - total XP needed to reach each level (index = level - 1)
/// Follows a progressive curve common in JRPGs
pub const XP_TABLE: [u32; 100] = [
    0,       // Level 1 (starting level)
    100,     // Level 2
    250,     // Level 3
    450,     // Level 4
    700,     // Level 5
    1000,    // Level 6
    1400,    // Level 7
    1900,    // Level 8
    2500,    // Level 9
    3200,    // Level 10
    4000,    // Level 11
    4900,    // Level 12
    5900,    // Level 13
    7000,    // Level 14
    8200,    // Level 15
    9500,    // Level 16
    10900,   // Level 17
    12400,   // Level 18
    14000,   // Level 19
    15700,   // Level 20
    17500,   // Level 21
    19400,   // Level 22
    21400,   // Level 23
    23500,   // Level 24
    25700,   // Level 25
    28000,   // Level 26
    30400,   // Level 27
    32900,   // Level 28
    35500,   // Level 29
    38200,   // Level 30
    41000,   // Level 31
    43900,   // Level 32
    46900,   // Level 33
    50000,   // Level 34
    53200,   // Level 35
    56500,   // Level 36
    59900,   // Level 37
    63400,   // Level 38
    67000,   // Level 39
    70700,   // Level 40
    74500,   // Level 41
    78400,   // Level 42
    82400,   // Level 43
    86500,   // Level 44
    90700,   // Level 45
    95000,   // Level 46
    99400,   // Level 47
    103900,  // Level 48
    108500,  // Level 49
    113200,  // Level 50
    118000,  // Level 51
    122900,  // Level 52
    127900,  // Level 53
    133000,  // Level 54
    138200,  // Level 55
    143500,  // Level 56
    148900,  // Level 57
    154400,  // Level 58
    160000,  // Level 59
    165700,  // Level 60
    171500,  // Level 61
    177400,  // Level 62
    183400,  // Level 63
    189500,  // Level 64
    195700,  // Level 65
    202000,  // Level 66
    208400,  // Level 67
    214900,  // Level 68
    221500,  // Level 69
    228200,  // Level 70
    235000,  // Level 71
    241900,  // Level 72
    248900,  // Level 73
    256000,  // Level 74
    263200,  // Level 75
    270500,  // Level 76
    277900,  // Level 77
    285400,  // Level 78
    293000,  // Level 79
    300700,  // Level 80
    308500,  // Level 81
    316400,  // Level 82
    324400,  // Level 83
    332500,  // Level 84
    340700,  // Level 85
    349000,  // Level 86
    357400,  // Level 87
    365900,  // Level 88
    374500,  // Level 89
    383200,  // Level 90
    392000,  // Level 91
    400900,  // Level 92
    409900,  // Level 93
    419000,  // Level 94
    428200,  // Level 95
    437500,  // Level 96
    446900,  // Level 97
    456400,  // Level 98
    466000,  // Level 99
    475700,  // Level 100 (unused, but prevents overflow)
];

// ============================================================================
// STAT GROWTH DEFINITIONS
// ============================================================================

/// Stat growth per level for each class
#[derive(Debug, Clone, Copy)]
pub struct StatGrowth {
    pub hp: i32,
    pub mp: i32,
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub vitality: i32,
    pub luck: i32,
}

/// Warrior: High HP, high strength, low magic
pub const WARRIOR_GROWTH: StatGrowth = StatGrowth {
    hp: 25,
    mp: 5,
    strength: 3,
    agility: 1,
    intelligence: 1,
    vitality: 2,
    luck: 1,
};

/// Paladin: Balanced fighter with healing magic
pub const PALADIN_GROWTH: StatGrowth = StatGrowth {
    hp: 20,
    mp: 12,
    strength: 2,
    agility: 2,
    intelligence: 2,
    vitality: 2,
    luck: 1,
};

/// Cleric: High MP, strong healing, moderate defense
pub const CLERIC_GROWTH: StatGrowth = StatGrowth {
    hp: 15,
    mp: 20,
    strength: 1,
    agility: 1,
    intelligence: 3,
    vitality: 2,
    luck: 1,
};

/// Wizard: Highest MP, powerful offensive magic, fragile
pub const WIZARD_GROWTH: StatGrowth = StatGrowth {
    hp: 12,
    mp: 25,
    strength: 1,
    agility: 1,
    intelligence: 4,
    vitality: 1,
    luck: 1,
};

/// Knight: Very high defense, good HP, moderate offense
pub const KNIGHT_GROWTH: StatGrowth = StatGrowth {
    hp: 22,
    mp: 8,
    strength: 2,
    agility: 1,
    intelligence: 1,
    vitality: 3,
    luck: 1,
};

/// Swashbuckler: High agility, fast attacks, evasion
pub const SWASHBUCKLER_GROWTH: StatGrowth = StatGrowth {
    hp: 16,
    mp: 10,
    strength: 2,
    agility: 4,
    intelligence: 1,
    vitality: 1,
    luck: 2,
};

/// Thief: High luck, agility, stealth abilities
pub const THIEF_GROWTH: StatGrowth = StatGrowth {
    hp: 14,
    mp: 8,
    strength: 1,
    agility: 3,
    intelligence: 2,
    vitality: 1,
    luck: 3,
};

/// Sage: Balanced magic user with utility spells
pub const SAGE_GROWTH: StatGrowth = StatGrowth {
    hp: 10,
    mp: 22,
    strength: 1,
    agility: 1,
    intelligence: 3,
    vitality: 1,
    luck: 2,
};

/// Archer: High agility, ranged attacks
pub const ARCHER_GROWTH: StatGrowth = StatGrowth {
    hp: 15,
    mp: 12,
    strength: 2,
    agility: 3,
    intelligence: 2,
    vitality: 1,
    luck: 1,
};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get the XP required to reach a specific level
pub fn xp_for_level(level: u8) -> u32 {
    if level == 0 || level > MAX_LEVEL {
        return 0;
    }
    XP_TABLE[(level - 1) as usize]
}

/// Calculate experience required for a given level (alias for xp_for_level)
pub fn exp_for_level(level: u8) -> u32 {
    xp_for_level(level)
}

/// Get the stat growth for a character class
pub fn stat_growth_for_class(class: CharacterClass) -> &'static StatGrowth {
    match class {
        CharacterClass::Warrior => &WARRIOR_GROWTH,
        CharacterClass::Paladin => &PALADIN_GROWTH,
        CharacterClass::Cleric => &CLERIC_GROWTH,
        CharacterClass::Wizard => &WIZARD_GROWTH,
        CharacterClass::Knight => &KNIGHT_GROWTH,
        CharacterClass::Swashbuckler => &SWASHBUCKLER_GROWTH,
        CharacterClass::Thief => &THIEF_GROWTH,
        CharacterClass::Sage => &SAGE_GROWTH,
        CharacterClass::Archer => &ARCHER_GROWTH,
    }
}

/// Character classes available in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Warrior,
    Paladin,
    Cleric,
    Wizard,
    Knight,
    Swashbuckler,
    Thief,
    Sage,
    Archer,
}

impl CharacterClass {
    /// Get the display name for this class
    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Warrior => "Warrior",
            CharacterClass::Paladin => "Paladin",
            CharacterClass::Cleric => "Cleric",
            CharacterClass::Wizard => "Wizard",
            CharacterClass::Knight => "Knight",
            CharacterClass::Swashbuckler => "Swashbuckler",
            CharacterClass::Thief => "Thief",
            CharacterClass::Sage => "Sage",
            CharacterClass::Archer => "Archer",
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_table_monotonically_increasing() {
        for i in 1..XP_TABLE.len() {
            assert!(
                XP_TABLE[i] > XP_TABLE[i - 1],
                "XP table not monotonically increasing at index {}: {} <= {}",
                i,
                XP_TABLE[i],
                XP_TABLE[i - 1]
            );
        }
    }

    #[test]
    fn test_xp_for_level_boundaries() {
        assert_eq!(xp_for_level(0), 0);
        assert_eq!(xp_for_level(1), 0);
        assert_eq!(xp_for_level(2), 100);
        assert_eq!(xp_for_level(MAX_LEVEL), XP_TABLE[(MAX_LEVEL - 1) as usize]);
        assert_eq!(xp_for_level(MAX_LEVEL + 1), 0);
    }

    #[test]
    fn test_stat_growth_for_all_classes() {
        // Ensure all classes have valid growth
        let classes = [
            CharacterClass::Warrior,
            CharacterClass::Paladin,
            CharacterClass::Cleric,
            CharacterClass::Wizard,
            CharacterClass::Knight,
            CharacterClass::Swashbuckler,
            CharacterClass::Thief,
            CharacterClass::Sage,
            CharacterClass::Archer,
        ];

        for class in &classes {
            let growth = stat_growth_for_class(*class);
            assert!(growth.hp > 0, "{:?} should have positive HP growth", class);
            assert!(growth.mp >= 0, "{:?} should have non-negative MP growth", class);
        }
    }

    #[test]
    fn test_combat_constants_valid() {
        assert!(BASE_HIT_CHANCE > 0.0 && BASE_HIT_CHANCE <= 1.0);
        assert!(CRITICAL_CHANCE > 0.0 && CRITICAL_CHANCE < 1.0);
        assert!(CRITICAL_MULTIPLIER > 1.0);
        assert!(FLEE_BASE_CHANCE > 0.0 && FLEE_BASE_CHANCE < 1.0);
        assert!(MIN_HIT_CHANCE < MAX_HIT_CHANCE);
    }

    #[test]
    fn test_party_constants_valid() {
        assert!(MAX_PARTY_SIZE >= 2); // At least the brothers
        assert!(STARTING_GOLD > 0);
        assert!(MAX_INVENTORY_SLOTS > 0);
    }
}
