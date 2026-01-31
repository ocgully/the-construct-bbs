//! Character definitions for Realm of Ralnar
//!
//! Contains all party member definitions including the brothers,
//! permanent recruits, and temporary party members.

use super::config::CharacterClass;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Character stats
#[derive(Debug, Clone, Copy)]
pub struct CharacterStats {
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub vitality: i32,
    pub luck: i32,
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            strength: 10,
            agility: 10,
            intelligence: 10,
            vitality: 10,
            luck: 10,
        }
    }
}

/// Equipment slots for a character
#[derive(Debug, Clone, Default)]
pub struct Equipment {
    pub weapon: Option<String>,
    pub armor: Option<String>,
    pub accessory: Option<String>,
}

/// Full party member definition
#[derive(Debug, Clone)]
pub struct PartyMember {
    pub id: String,
    pub name: String,
    pub class: CharacterClass,
    pub level: u8,
    pub exp: u32,
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub stats: CharacterStats,
    pub equipment: Equipment,
    /// True for Herbert and Valeran - cannot leave party
    pub is_brother: bool,
    /// Backstory text
    pub backstory: String,
    /// True if this is a permanent party member
    pub is_permanent: bool,
}

/// Character recruitment status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitmentType {
    /// Always in party (brothers)
    Core,
    /// Joins permanently
    Permanent,
    /// Joins temporarily, leaves, may return
    Temporary,
}

/// Character definition for creating party members
#[derive(Debug, Clone)]
pub struct CharacterDef {
    pub id: &'static str,
    pub name: &'static str,
    pub class: CharacterClass,
    pub starting_level: u8,
    pub base_hp: i32,
    pub base_mp: i32,
    pub base_stats: CharacterStats,
    pub starting_weapon: Option<&'static str>,
    pub starting_armor: Option<&'static str>,
    pub is_brother: bool,
    pub recruitment_type: RecruitmentType,
    pub backstory: &'static str,
    pub join_region: u8,
}

// ============================================================================
// CHARACTER DEFINITIONS
// ============================================================================

/// Herbert - The Hero / Older Brother (Warrior)
pub const HERBERT_DEF: CharacterDef = CharacterDef {
    id: "herbert",
    name: "Herbert",
    class: CharacterClass::Warrior,
    starting_level: 1,
    base_hp: 100,
    base_mp: 20,
    base_stats: CharacterStats {
        strength: 15,
        agility: 10,
        intelligence: 8,
        vitality: 12,
        luck: 8,
    },
    starting_weapon: Some("iron_sword"),
    starting_armor: Some("leather_armor"),
    is_brother: true,
    recruitment_type: RecruitmentType::Core,
    backstory: "Herbert always wanted a simple life - a forge, honest work, maybe a family someday. \
                He apprenticed under the village blacksmith before the troubles began. When their \
                parents disappeared during The Dimming, Herbert became father, mother, and brother \
                to young Valeran. He's not seeking adventure - he's seeking answers.",
    join_region: 1,
};

/// Valeran - The Younger Brother (Paladin)
pub const VALERAN_DEF: CharacterDef = CharacterDef {
    id: "valeran",
    name: "Valeran",
    class: CharacterClass::Paladin,
    starting_level: 1,
    base_hp: 80,
    base_mp: 30,
    base_stats: CharacterStats {
        strength: 12,
        agility: 12,
        intelligence: 10,
        vitality: 10,
        luck: 10,
    },
    starting_weapon: None,
    starting_armor: None,
    is_brother: true,
    recruitment_type: RecruitmentType::Core,
    backstory: "Valeran barely remembers their parents, but he remembers the stories Herbert told \
                him - of noble knights who protected the realm. He decided young that he would \
                become one. He trains constantly, reads every book on chivalry, and practices \
                his heroic poses when he thinks no one is looking. Unlike Herbert, Valeran WANTS adventure.",
    join_region: 1,
};

/// Sera - The Wayward Cleric
pub const SERA_DEF: CharacterDef = CharacterDef {
    id: "sera",
    name: "Sera",
    class: CharacterClass::Cleric,
    starting_level: 3,
    base_hp: 65,
    base_mp: 50,
    base_stats: CharacterStats {
        strength: 8,
        agility: 10,
        intelligence: 14,
        vitality: 9,
        luck: 12,
    },
    starting_weapon: Some("oak_staff"),
    starting_armor: Some("robes"),
    is_brother: false,
    recruitment_type: RecruitmentType::Permanent,
    backstory: "Sera was a priestess of the old faith - the worship of the Five Guardians. When \
                monsters began appearing, her temple fell. She alone survived, questioning why \
                the Guardians seemed silent. She joins the brothers hoping to find answers at \
                the shrines - not knowing she'll find the most terrible answer of all.",
    join_region: 2,
};

/// Korrath - The Disgraced Knight
pub const KORRATH_DEF: CharacterDef = CharacterDef {
    id: "korrath",
    name: "Korrath",
    class: CharacterClass::Knight,
    starting_level: 8,
    base_hp: 120,
    base_mp: 25,
    base_stats: CharacterStats {
        strength: 14,
        agility: 8,
        intelligence: 10,
        vitality: 16,
        luck: 6,
    },
    starting_weapon: Some("steel_sword"),
    starting_armor: Some("chain_mail"),
    is_brother: false,
    recruitment_type: RecruitmentType::Permanent,
    backstory: "Once the captain of Castle Herbert's guard, Korrath was imprisoned when he \
                accused the King's new advisor of dark magic. The advisor? An old man who \
                appeared months ago... Korrath knows Dorl from the castle, and his warnings \
                were ignored.",
    join_region: 4,
};

/// Zanth - The Wandering Mystic
pub const ZANTH_DEF: CharacterDef = CharacterDef {
    id: "zanth",
    name: "Zanth",
    class: CharacterClass::Wizard,
    starting_level: 5,
    base_hp: 55,
    base_mp: 70,
    base_stats: CharacterStats {
        strength: 6,
        agility: 9,
        intelligence: 18,
        vitality: 8,
        luck: 14,
    },
    starting_weapon: Some("mage_staff"),
    starting_armor: Some("mage_robes"),
    is_brother: false,
    recruitment_type: RecruitmentType::Temporary,
    backstory: "Zanth has spent thirty years wandering the land, tending to small shrines, \
                blessing travelers, and listening to what she calls 'the whispers between worlds.' \
                She's known something was wrong for months - the spirits have been agitated, \
                the omens dark. She doesn't see warriors. She sees two lost boys who need \
                someone to believe in them.",
    join_region: 2,
};

/// Captain John - The Seafaring Dreamer
pub const CAPTAIN_JOHN_DEF: CharacterDef = CharacterDef {
    id: "captain_john",
    name: "Captain John",
    class: CharacterClass::Swashbuckler,
    starting_level: 4,
    base_hp: 75,
    base_mp: 30,
    base_stats: CharacterStats {
        strength: 11,
        agility: 14,
        intelligence: 12,
        vitality: 10,
        luck: 11,
    },
    starting_weapon: Some("cutlass"),
    starting_armor: Some("brigandine"),
    is_brother: false,
    recruitment_type: RecruitmentType::Temporary,
    backstory: "Captain John dreamed of being a doctor, not a sailor. But his father's father's \
                father was a captain, and his father before him. The sea be in his blood, whether \
                he likes it or not. But he still reads medical texts when the crew isn't looking.",
    join_region: 1,
};

/// Nomodest - The Unrepentant Rogue
pub const NOMODEST_DEF: CharacterDef = CharacterDef {
    id: "nomodest",
    name: "Nomodest",
    class: CharacterClass::Thief,
    starting_level: 4,
    base_hp: 60,
    base_mp: 25,
    base_stats: CharacterStats {
        strength: 9,
        agility: 16,
        intelligence: 11,
        vitality: 8,
        luck: 15,
    },
    starting_weapon: Some("dagger"),
    starting_armor: Some("brigandine"),
    is_brother: false,
    recruitment_type: RecruitmentType::Temporary,
    backstory: "Nomodest is a thief with a buried conscience. Found robbing ancient ruins, he \
                decided the party was a better mark... then actually started helping. Sarcastic \
                and self-serving, but with unexpected moments of heroism. He'll leave with some \
                loot - and an apologetic note.",
    join_region: 2,
};

/// Elder Morath - The Guardian's Voice
pub const ELDER_MORATH_DEF: CharacterDef = CharacterDef {
    id: "elder_morath",
    name: "Elder Morath",
    class: CharacterClass::Sage,
    starting_level: 12,
    base_hp: 45,
    base_mp: 90,
    base_stats: CharacterStats {
        strength: 5,
        agility: 6,
        intelligence: 20,
        vitality: 7,
        luck: 10,
    },
    starting_weapon: Some("wizard_staff"),
    starting_armor: Some("archmage_robes"),
    is_brother: false,
    recruitment_type: RecruitmentType::Temporary,
    backstory: "The last priest who remembers the old sealing. Elder Morath is 78 years old and \
                fragile, but his magic is powerful. He provides exposition about the original \
                sealing and is the first to realize what Dorl is doing - tragically, too late.",
    join_region: 3,
};

/// Lyra - The Sky Nomad
pub const LYRA_DEF: CharacterDef = CharacterDef {
    id: "lyra",
    name: "Lyra",
    class: CharacterClass::Archer,
    starting_level: 7,
    base_hp: 70,
    base_mp: 40,
    base_stats: CharacterStats {
        strength: 10,
        agility: 15,
        intelligence: 12,
        vitality: 9,
        luck: 12,
    },
    starting_weapon: Some("long_bow"),
    starting_armor: Some("wind_cloak"),
    is_brother: false,
    recruitment_type: RecruitmentType::Temporary,
    backstory: "The Sky Nomads have guarded the Wind Shrine for generations - without knowing why. \
                'We were told to watch, never to enter. Now I understand.' Lyra is 20 years old \
                and will stay to help her people evacuate when the island falls. She brings the \
                Airship to the party as her people's last gift.",
    join_region: 4,
};

// ============================================================================
// CHARACTER CREATION FUNCTIONS
// ============================================================================

/// Create Herbert (the hero/older brother)
pub fn create_herbert() -> PartyMember {
    create_party_member(&HERBERT_DEF)
}

/// Create Valeran (younger brother)
pub fn create_valeran() -> PartyMember {
    create_party_member(&VALERAN_DEF)
}

/// Create Sera (cleric)
pub fn create_sera() -> PartyMember {
    create_party_member(&SERA_DEF)
}

/// Create Korrath (knight)
pub fn create_korrath() -> PartyMember {
    create_party_member(&KORRATH_DEF)
}

/// Create Zanth (wizard)
pub fn create_zanth() -> PartyMember {
    create_party_member(&ZANTH_DEF)
}

/// Create Captain John (swashbuckler)
pub fn create_captain_john() -> PartyMember {
    create_party_member(&CAPTAIN_JOHN_DEF)
}

/// Create Nomodest (thief)
pub fn create_nomodest() -> PartyMember {
    create_party_member(&NOMODEST_DEF)
}

/// Create Elder Morath (sage)
pub fn create_elder_morath() -> PartyMember {
    create_party_member(&ELDER_MORATH_DEF)
}

/// Create Lyra (archer)
pub fn create_lyra() -> PartyMember {
    create_party_member(&LYRA_DEF)
}

/// Create a party member from a character definition
pub fn create_party_member(def: &CharacterDef) -> PartyMember {
    PartyMember {
        id: def.id.to_string(),
        name: def.name.to_string(),
        class: def.class,
        level: def.starting_level,
        exp: 0,
        hp: def.base_hp,
        hp_max: def.base_hp,
        mp: def.base_mp,
        mp_max: def.base_mp,
        stats: def.base_stats,
        equipment: Equipment {
            weapon: def.starting_weapon.map(|s| s.to_string()),
            armor: def.starting_armor.map(|s| s.to_string()),
            accessory: None,
        },
        is_brother: def.is_brother,
        backstory: def.backstory.to_string(),
        is_permanent: matches!(def.recruitment_type, RecruitmentType::Core | RecruitmentType::Permanent),
    }
}

/// Create a party member at a specific level (for level scaling)
pub fn create_party_member_at_level(def: &CharacterDef, level: u8) -> PartyMember {
    use super::config::{stat_growth_for_class, xp_for_level};

    let growth = stat_growth_for_class(def.class);
    let levels_gained = level.saturating_sub(def.starting_level) as i32;

    let hp = def.base_hp + (growth.hp * levels_gained);
    let mp = def.base_mp + (growth.mp * levels_gained);

    PartyMember {
        id: def.id.to_string(),
        name: def.name.to_string(),
        class: def.class,
        level,
        exp: xp_for_level(level),
        hp,
        hp_max: hp,
        mp,
        mp_max: mp,
        stats: CharacterStats {
            strength: def.base_stats.strength + (growth.strength * levels_gained),
            agility: def.base_stats.agility + (growth.agility * levels_gained),
            intelligence: def.base_stats.intelligence + (growth.intelligence * levels_gained),
            vitality: def.base_stats.vitality + (growth.vitality * levels_gained),
            luck: def.base_stats.luck + (growth.luck * levels_gained),
        },
        equipment: Equipment {
            weapon: def.starting_weapon.map(|s| s.to_string()),
            armor: def.starting_armor.map(|s| s.to_string()),
            accessory: None,
        },
        is_brother: def.is_brother,
        backstory: def.backstory.to_string(),
        is_permanent: matches!(def.recruitment_type, RecruitmentType::Core | RecruitmentType::Permanent),
    }
}

// ============================================================================
// CHARACTER COLLECTION AND LOOKUP
// ============================================================================

/// All character definitions
pub static ALL_CHARACTERS: &[&CharacterDef] = &[
    &HERBERT_DEF,
    &VALERAN_DEF,
    &SERA_DEF,
    &KORRATH_DEF,
    &ZANTH_DEF,
    &CAPTAIN_JOHN_DEF,
    &NOMODEST_DEF,
    &ELDER_MORATH_DEF,
    &LYRA_DEF,
];

/// Look up a character definition by ID
pub fn get_character_def(id: &str) -> Option<&'static CharacterDef> {
    ALL_CHARACTERS.iter().find(|c| c.id == id).copied()
}

/// Get all permanent party members
pub fn get_permanent_characters() -> Vec<&'static CharacterDef> {
    ALL_CHARACTERS.iter()
        .filter(|c| matches!(c.recruitment_type, RecruitmentType::Core | RecruitmentType::Permanent))
        .copied()
        .collect()
}

/// Get all temporary party members
pub fn get_temporary_characters() -> Vec<&'static CharacterDef> {
    ALL_CHARACTERS.iter()
        .filter(|c| matches!(c.recruitment_type, RecruitmentType::Temporary))
        .copied()
        .collect()
}

/// Get the brothers (core characters)
pub fn get_brothers() -> Vec<&'static CharacterDef> {
    ALL_CHARACTERS.iter()
        .filter(|c| c.is_brother)
        .copied()
        .collect()
}

/// Get characters that join in a specific region
pub fn get_characters_in_region(region: u8) -> Vec<&'static CharacterDef> {
    ALL_CHARACTERS.iter()
        .filter(|c| c.join_region == region)
        .copied()
        .collect()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_characters_have_unique_ids() {
        let mut ids: Vec<&str> = ALL_CHARACTERS.iter().map(|c| c.id).collect();
        ids.sort();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate character IDs found");
    }

    #[test]
    fn test_create_herbert() {
        let herbert = create_herbert();
        assert_eq!(herbert.id, "herbert");
        assert_eq!(herbert.name, "Herbert");
        assert_eq!(herbert.class, CharacterClass::Warrior);
        assert!(herbert.is_brother);
        assert!(herbert.hp > 0);
        assert!(herbert.hp_max > 0);
        assert_eq!(herbert.hp, herbert.hp_max);
    }

    #[test]
    fn test_create_valeran() {
        let valeran = create_valeran();
        assert_eq!(valeran.id, "valeran");
        assert_eq!(valeran.name, "Valeran");
        assert_eq!(valeran.class, CharacterClass::Paladin);
        assert!(valeran.is_brother);
    }

    #[test]
    fn test_brothers_both_exist() {
        let brothers = get_brothers();
        assert_eq!(brothers.len(), 2, "Should have exactly 2 brothers");

        let ids: Vec<&str> = brothers.iter().map(|b| b.id).collect();
        assert!(ids.contains(&"herbert"));
        assert!(ids.contains(&"valeran"));
    }

    #[test]
    fn test_brothers_are_core() {
        for brother in get_brothers() {
            assert!(brother.is_brother);
            assert_eq!(brother.recruitment_type, RecruitmentType::Core);
        }
    }

    #[test]
    fn test_character_creation_functions() {
        // Test all character creation functions produce valid members
        let characters = vec![
            create_herbert(),
            create_valeran(),
            create_sera(),
            create_korrath(),
            create_zanth(),
            create_captain_john(),
            create_nomodest(),
            create_elder_morath(),
            create_lyra(),
        ];

        for character in &characters {
            assert!(!character.id.is_empty(), "Character should have an ID");
            assert!(!character.name.is_empty(), "Character should have a name");
            assert!(character.hp > 0, "Character {} should have positive HP", character.id);
            assert!(character.hp_max > 0, "Character {} should have positive max HP", character.id);
            assert!(character.mp >= 0, "Character {} should have non-negative MP", character.id);
            assert!(character.level >= 1, "Character {} should be at least level 1", character.id);
        }
    }

    #[test]
    fn test_character_level_scaling() {
        let herbert_lv1 = create_herbert();
        let herbert_lv10 = create_party_member_at_level(&HERBERT_DEF, 10);

        assert!(herbert_lv10.hp_max > herbert_lv1.hp_max, "Level 10 should have more HP");
        assert!(herbert_lv10.stats.strength > herbert_lv1.stats.strength, "Level 10 should have more strength");
        assert_eq!(herbert_lv10.level, 10);
    }

    #[test]
    fn test_permanent_vs_temporary() {
        let permanent = get_permanent_characters();
        let temporary = get_temporary_characters();

        // Brothers + Sera + Korrath = 4 permanent
        assert_eq!(permanent.len(), 4, "Should have 4 permanent characters");

        // Zanth, Captain John, Nomodest, Elder Morath, Lyra = 5 temporary
        assert_eq!(temporary.len(), 5, "Should have 5 temporary characters");
    }

    #[test]
    fn test_sera_is_permanent() {
        let sera_def = get_character_def("sera");
        assert!(sera_def.is_some());
        assert_eq!(sera_def.unwrap().recruitment_type, RecruitmentType::Permanent);
    }

    #[test]
    fn test_equipment_setup() {
        let herbert = create_herbert();
        assert!(herbert.equipment.weapon.is_some(), "Herbert should have a starting weapon");
        assert!(herbert.equipment.armor.is_some(), "Herbert should have starting armor");

        let valeran = create_valeran();
        assert!(valeran.equipment.weapon.is_none(), "Valeran starts without a weapon");
    }
}
