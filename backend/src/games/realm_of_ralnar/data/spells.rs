//! Spell definitions for Realm of Ralnar
//!
//! Contains all magic spells including healing, attack, buff, debuff,
//! and Guardian-granted abilities.

use super::config::CharacterClass;
use super::items::Element;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Type of spell effect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellType {
    /// Deals damage to enemies
    Damage,
    /// Restores HP
    Heal,
    /// Increases ally stats
    Buff,
    /// Decreases enemy stats
    Debuff,
    /// Inflicts status ailment
    StatusInflict,
    /// Cures status ailment
    StatusCure,
    /// Utility (detection, escape, etc.)
    Utility,
    /// Revives fallen allies
    Revive,
}

/// Who the spell can target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    /// One enemy
    SingleEnemy,
    /// All enemies
    AllEnemies,
    /// One ally
    SingleAlly,
    /// All allies
    AllAllies,
    /// The caster only
    SelfOnly,
}

/// Status effect that spells can inflict or cure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellStatus {
    Poison,
    Stone,
    Confusion,
    Sleep,
    Blind,
    Silence,
    Slow,
    Haste,
    Regen,
    Protect,
    Shell,
}

/// Full spell definition
#[derive(Debug, Clone)]
pub struct SpellDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub mp_cost: i32,
    pub spell_type: SpellType,
    pub element: Element,
    pub power: i32,
    pub accuracy: f32,
    pub target_type: TargetType,
    pub status_effect: Option<(SpellStatus, f32)>,
    pub level_required: u8,
    pub classes: &'static [CharacterClass],
    /// If true, this is a Guardian-granted ability
    pub is_guardian_power: bool,
}

// ============================================================================
// HEALING SPELLS
// ============================================================================

pub const CURE: SpellDef = SpellDef {
    id: "cure",
    name: "Cure",
    description: "Restores a small amount of HP",
    mp_cost: 4,
    spell_type: SpellType::Heal,
    element: Element::Holy,
    power: 50,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 1,
    classes: &[CharacterClass::Cleric, CharacterClass::Paladin, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const CURA: SpellDef = SpellDef {
    id: "cura",
    name: "Cura",
    description: "Restores a moderate amount of HP",
    mp_cost: 12,
    spell_type: SpellType::Heal,
    element: Element::Holy,
    power: 150,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 10,
    classes: &[CharacterClass::Cleric, CharacterClass::Paladin, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const CURAGA: SpellDef = SpellDef {
    id: "curaga",
    name: "Curaga",
    description: "Restores a large amount of HP",
    mp_cost: 28,
    spell_type: SpellType::Heal,
    element: Element::Holy,
    power: 400,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 25,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const REGEN: SpellDef = SpellDef {
    id: "regen",
    name: "Regen",
    description: "Grants regeneration over time",
    mp_cost: 15,
    spell_type: SpellType::Buff,
    element: Element::Holy,
    power: 20, // HP per turn
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: Some((SpellStatus::Regen, 1.0)),
    level_required: 15,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const CURALL: SpellDef = SpellDef {
    id: "curall",
    name: "Curall",
    description: "Heals the entire party",
    mp_cost: 35,
    spell_type: SpellType::Heal,
    element: Element::Holy,
    power: 200,
    accuracy: 1.0,
    target_type: TargetType::AllAllies,
    status_effect: None,
    level_required: 30,
    classes: &[CharacterClass::Cleric],
    is_guardian_power: false,
};

pub const RAISE: SpellDef = SpellDef {
    id: "raise",
    name: "Raise",
    description: "Revives a fallen ally with low HP",
    mp_cost: 20,
    spell_type: SpellType::Revive,
    element: Element::Holy,
    power: 25, // 25% HP
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 12,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const ARISE: SpellDef = SpellDef {
    id: "arise",
    name: "Arise",
    description: "Revives a fallen ally with full HP",
    mp_cost: 50,
    spell_type: SpellType::Revive,
    element: Element::Holy,
    power: 100, // 100% HP
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 35,
    classes: &[CharacterClass::Cleric],
    is_guardian_power: false,
};

// ============================================================================
// STATUS CURE SPELLS
// ============================================================================

pub const ESUNA: SpellDef = SpellDef {
    id: "esuna",
    name: "Esuna",
    description: "Cures all status ailments",
    mp_cost: 10,
    spell_type: SpellType::StatusCure,
    element: Element::Holy,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 8,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const POISONA: SpellDef = SpellDef {
    id: "poisona",
    name: "Poisona",
    description: "Cures poison",
    mp_cost: 3,
    spell_type: SpellType::StatusCure,
    element: Element::Holy,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 3,
    classes: &[CharacterClass::Cleric, CharacterClass::Paladin, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const STONA: SpellDef = SpellDef {
    id: "stona",
    name: "Stona",
    description: "Cures petrification",
    mp_cost: 8,
    spell_type: SpellType::StatusCure,
    element: Element::Holy,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: None,
    level_required: 12,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

// ============================================================================
// ATTACK SPELLS - FIRE
// ============================================================================

pub const FIRE: SpellDef = SpellDef {
    id: "fire",
    name: "Fire",
    description: "Deals fire damage to one enemy",
    mp_cost: 5,
    spell_type: SpellType::Damage,
    element: Element::Fire,
    power: 30,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 1,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const FIRA: SpellDef = SpellDef {
    id: "fira",
    name: "Fira",
    description: "Deals moderate fire damage",
    mp_cost: 15,
    spell_type: SpellType::Damage,
    element: Element::Fire,
    power: 80,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 12,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const FIRAGA: SpellDef = SpellDef {
    id: "firaga",
    name: "Firaga",
    description: "Deals heavy fire damage to all",
    mp_cost: 35,
    spell_type: SpellType::Damage,
    element: Element::Fire,
    power: 150,
    accuracy: 0.95,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 28,
    classes: &[CharacterClass::Wizard],
    is_guardian_power: false,
};

// ============================================================================
// ATTACK SPELLS - ICE
// ============================================================================

pub const BLIZZARD: SpellDef = SpellDef {
    id: "blizzard",
    name: "Blizzard",
    description: "Deals ice damage to one enemy",
    mp_cost: 5,
    spell_type: SpellType::Damage,
    element: Element::Ice,
    power: 30,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 1,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const BLIZZARA: SpellDef = SpellDef {
    id: "blizzara",
    name: "Blizzara",
    description: "Deals moderate ice damage",
    mp_cost: 15,
    spell_type: SpellType::Damage,
    element: Element::Ice,
    power: 80,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 12,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const BLIZZAGA: SpellDef = SpellDef {
    id: "blizzaga",
    name: "Blizzaga",
    description: "Deals heavy ice damage to all",
    mp_cost: 35,
    spell_type: SpellType::Damage,
    element: Element::Ice,
    power: 150,
    accuracy: 0.95,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 28,
    classes: &[CharacterClass::Wizard],
    is_guardian_power: false,
};

// ============================================================================
// ATTACK SPELLS - LIGHTNING
// ============================================================================

pub const THUNDER: SpellDef = SpellDef {
    id: "thunder",
    name: "Thunder",
    description: "Deals lightning damage to one enemy",
    mp_cost: 5,
    spell_type: SpellType::Damage,
    element: Element::Lightning,
    power: 30,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 1,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const THUNDARA: SpellDef = SpellDef {
    id: "thundara",
    name: "Thundara",
    description: "Deals moderate lightning damage",
    mp_cost: 15,
    spell_type: SpellType::Damage,
    element: Element::Lightning,
    power: 80,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 12,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const THUNDAGA: SpellDef = SpellDef {
    id: "thundaga",
    name: "Thundaga",
    description: "Deals heavy lightning damage to all",
    mp_cost: 35,
    spell_type: SpellType::Damage,
    element: Element::Lightning,
    power: 150,
    accuracy: 0.95,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 28,
    classes: &[CharacterClass::Wizard],
    is_guardian_power: false,
};

// ============================================================================
// ATTACK SPELLS - OTHER ELEMENTS
// ============================================================================

pub const QUAKE: SpellDef = SpellDef {
    id: "quake",
    name: "Quake",
    description: "Earth damage to all enemies",
    mp_cost: 25,
    spell_type: SpellType::Damage,
    element: Element::Earth,
    power: 100,
    accuracy: 0.90,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 20,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const FLOOD: SpellDef = SpellDef {
    id: "flood",
    name: "Flood",
    description: "Water damage to all enemies",
    mp_cost: 25,
    spell_type: SpellType::Damage,
    element: Element::Water,
    power: 100,
    accuracy: 0.90,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 20,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const AERO: SpellDef = SpellDef {
    id: "aero",
    name: "Aero",
    description: "Wind damage to one enemy",
    mp_cost: 8,
    spell_type: SpellType::Damage,
    element: Element::Wind,
    power: 40,
    accuracy: 0.95,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 5,
    classes: &[CharacterClass::Archer, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const AEROGA: SpellDef = SpellDef {
    id: "aeroga",
    name: "Aeroga",
    description: "Heavy wind damage to all enemies",
    mp_cost: 30,
    spell_type: SpellType::Damage,
    element: Element::Wind,
    power: 130,
    accuracy: 0.95,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 25,
    classes: &[CharacterClass::Archer],
    is_guardian_power: false,
};

pub const HOLY: SpellDef = SpellDef {
    id: "holy",
    name: "Holy",
    description: "Powerful holy damage",
    mp_cost: 40,
    spell_type: SpellType::Damage,
    element: Element::Holy,
    power: 180,
    accuracy: 1.0,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 35,
    classes: &[CharacterClass::Cleric, CharacterClass::Paladin],
    is_guardian_power: false,
};

pub const DARK: SpellDef = SpellDef {
    id: "dark",
    name: "Dark",
    description: "Dark damage to one enemy",
    mp_cost: 10,
    spell_type: SpellType::Damage,
    element: Element::Dark,
    power: 50,
    accuracy: 0.90,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 8,
    classes: &[CharacterClass::Wizard],
    is_guardian_power: false,
};

// ============================================================================
// BUFF SPELLS
// ============================================================================

pub const PROTECT: SpellDef = SpellDef {
    id: "protect",
    name: "Protect",
    description: "Increases physical defense",
    mp_cost: 8,
    spell_type: SpellType::Buff,
    element: Element::None,
    power: 50, // 50% defense boost
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: Some((SpellStatus::Protect, 1.0)),
    level_required: 6,
    classes: &[CharacterClass::Cleric, CharacterClass::Paladin, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const SHELL: SpellDef = SpellDef {
    id: "shell",
    name: "Shell",
    description: "Increases magic defense",
    mp_cost: 8,
    spell_type: SpellType::Buff,
    element: Element::None,
    power: 50, // 50% magic defense boost
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: Some((SpellStatus::Shell, 1.0)),
    level_required: 6,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const HASTE_SPELL: SpellDef = SpellDef {
    id: "haste_spell",
    name: "Haste",
    description: "Increases action speed",
    mp_cost: 15,
    spell_type: SpellType::Buff,
    element: Element::None,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: Some((SpellStatus::Haste, 1.0)),
    level_required: 18,
    classes: &[CharacterClass::Sage],
    is_guardian_power: false,
};

// ============================================================================
// DEBUFF SPELLS
// ============================================================================

pub const SLOW: SpellDef = SpellDef {
    id: "slow",
    name: "Slow",
    description: "Reduces enemy speed",
    mp_cost: 8,
    spell_type: SpellType::Debuff,
    element: Element::None,
    power: 0,
    accuracy: 0.75,
    target_type: TargetType::SingleEnemy,
    status_effect: Some((SpellStatus::Slow, 0.75)),
    level_required: 10,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const BLIND_SPELL: SpellDef = SpellDef {
    id: "blind_spell",
    name: "Blind",
    description: "Reduces enemy accuracy",
    mp_cost: 5,
    spell_type: SpellType::StatusInflict,
    element: Element::Dark,
    power: 0,
    accuracy: 0.70,
    target_type: TargetType::SingleEnemy,
    status_effect: Some((SpellStatus::Blind, 0.70)),
    level_required: 5,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const SILENCE_SPELL: SpellDef = SpellDef {
    id: "silence_spell",
    name: "Silence",
    description: "Prevents enemy spellcasting",
    mp_cost: 6,
    spell_type: SpellType::StatusInflict,
    element: Element::None,
    power: 0,
    accuracy: 0.65,
    target_type: TargetType::SingleEnemy,
    status_effect: Some((SpellStatus::Silence, 0.65)),
    level_required: 8,
    classes: &[CharacterClass::Cleric, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const SLEEP_SPELL: SpellDef = SpellDef {
    id: "sleep_spell",
    name: "Sleep",
    description: "Puts enemy to sleep",
    mp_cost: 10,
    spell_type: SpellType::StatusInflict,
    element: Element::None,
    power: 0,
    accuracy: 0.60,
    target_type: TargetType::SingleEnemy,
    status_effect: Some((SpellStatus::Sleep, 0.60)),
    level_required: 12,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const CONFUSE_SPELL: SpellDef = SpellDef {
    id: "confuse_spell",
    name: "Confuse",
    description: "Confuses the enemy",
    mp_cost: 12,
    spell_type: SpellType::StatusInflict,
    element: Element::None,
    power: 0,
    accuracy: 0.55,
    target_type: TargetType::SingleEnemy,
    status_effect: Some((SpellStatus::Confusion, 0.55)),
    level_required: 15,
    classes: &[CharacterClass::Wizard],
    is_guardian_power: false,
};

// ============================================================================
// UTILITY SPELLS
// ============================================================================

pub const SCAN: SpellDef = SpellDef {
    id: "scan",
    name: "Scan",
    description: "Reveals enemy weaknesses",
    mp_cost: 2,
    spell_type: SpellType::Utility,
    element: Element::None,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SingleEnemy,
    status_effect: None,
    level_required: 1,
    classes: &[CharacterClass::Wizard, CharacterClass::Sage],
    is_guardian_power: false,
};

pub const ESCAPE: SpellDef = SpellDef {
    id: "escape",
    name: "Escape",
    description: "Flee from battle instantly",
    mp_cost: 10,
    spell_type: SpellType::Utility,
    element: Element::None,
    power: 0,
    accuracy: 0.90,
    target_type: TargetType::SelfOnly,
    status_effect: None,
    level_required: 5,
    classes: &[CharacterClass::Thief, CharacterClass::Sage],
    is_guardian_power: false,
};

// ============================================================================
// GUARDIAN-GRANTED ABILITIES
// ============================================================================

/// Soul Sight - Granted by Spirit Guardian (Spirata)
/// Reveals hidden enemies, traps, and lies
pub const SOUL_SIGHT: SpellDef = SpellDef {
    id: "soul_sight",
    name: "Soul Sight",
    description: "Reveals hidden threats and weaknesses",
    mp_cost: 5,
    spell_type: SpellType::Utility,
    element: Element::Holy,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SelfOnly,
    status_effect: None,
    level_required: 1,
    classes: &[], // Granted, not learned
    is_guardian_power: true,
};

/// Stone Skin - Granted by Earth Guardian (Terreth)
/// Temporary massive defense boost
pub const STONE_SKIN: SpellDef = SpellDef {
    id: "stone_skin",
    name: "Stone Skin",
    description: "Massively increases defense temporarily",
    mp_cost: 12,
    spell_type: SpellType::Buff,
    element: Element::Earth,
    power: 100, // 100% defense boost
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: Some((SpellStatus::Protect, 1.0)),
    level_required: 1,
    classes: &[], // Granted, not learned
    is_guardian_power: true,
};

/// Healing Tide - Granted by Water Guardian (Aqualis)
/// Party-wide heal over time
pub const HEALING_TIDE: SpellDef = SpellDef {
    id: "healing_tide",
    name: "Healing Tide",
    description: "Heals all allies over time",
    mp_cost: 20,
    spell_type: SpellType::Heal,
    element: Element::Water,
    power: 40, // Per turn for several turns
    accuracy: 1.0,
    target_type: TargetType::AllAllies,
    status_effect: Some((SpellStatus::Regen, 1.0)),
    level_required: 1,
    classes: &[], // Granted, not learned
    is_guardian_power: true,
};

/// Haste - Granted by Wind Guardian (Ventus)
/// Extra actions in combat
pub const HASTE: SpellDef = SpellDef {
    id: "haste",
    name: "Windstep",
    description: "Grants an extra action this turn",
    mp_cost: 18,
    spell_type: SpellType::Buff,
    element: Element::Wind,
    power: 0,
    accuracy: 1.0,
    target_type: TargetType::SingleAlly,
    status_effect: Some((SpellStatus::Haste, 1.0)),
    level_required: 1,
    classes: &[], // Granted, not learned
    is_guardian_power: true,
};

/// Inferno - Granted by Fire Guardian (Pyreth)
/// Massive fire damage to all enemies
pub const INFERNO: SpellDef = SpellDef {
    id: "inferno",
    name: "Inferno",
    description: "Devastating fire damage to all enemies",
    mp_cost: 35,
    spell_type: SpellType::Damage,
    element: Element::Fire,
    power: 200,
    accuracy: 0.95,
    target_type: TargetType::AllEnemies,
    status_effect: None,
    level_required: 1,
    classes: &[], // Granted, not learned
    is_guardian_power: true,
};

// ============================================================================
// SPELL COLLECTION AND LOOKUP
// ============================================================================

/// All spells in the game
pub static ALL_SPELLS: &[&SpellDef] = &[
    // Healing
    &CURE, &CURA, &CURAGA, &REGEN, &CURALL, &RAISE, &ARISE,
    // Status Cure
    &ESUNA, &POISONA, &STONA,
    // Fire
    &FIRE, &FIRA, &FIRAGA,
    // Ice
    &BLIZZARD, &BLIZZARA, &BLIZZAGA,
    // Lightning
    &THUNDER, &THUNDARA, &THUNDAGA,
    // Other elements
    &QUAKE, &FLOOD, &AERO, &AEROGA, &HOLY, &DARK,
    // Buffs
    &PROTECT, &SHELL, &HASTE_SPELL,
    // Debuffs
    &SLOW, &BLIND_SPELL, &SILENCE_SPELL, &SLEEP_SPELL, &CONFUSE_SPELL,
    // Utility
    &SCAN, &ESCAPE,
    // Guardian powers
    &SOUL_SIGHT, &STONE_SKIN, &HEALING_TIDE, &HASTE, &INFERNO,
];

/// Look up a spell by its ID
pub fn get_spell(id: &str) -> Option<&'static SpellDef> {
    ALL_SPELLS.iter().find(|spell| spell.id == id).copied()
}

/// Get all spells a class can learn (excluding Guardian powers)
pub fn get_spells_for_class(class: CharacterClass) -> Vec<&'static SpellDef> {
    ALL_SPELLS.iter()
        .filter(|spell| spell.classes.contains(&class) && !spell.is_guardian_power)
        .copied()
        .collect()
}

/// Get all Guardian-granted abilities
pub fn get_guardian_powers() -> Vec<&'static SpellDef> {
    ALL_SPELLS.iter()
        .filter(|spell| spell.is_guardian_power)
        .copied()
        .collect()
}

/// Get spells by type
pub fn get_spells_by_type(spell_type: SpellType) -> Vec<&'static SpellDef> {
    ALL_SPELLS.iter()
        .filter(|spell| spell.spell_type == spell_type)
        .copied()
        .collect()
}

/// Get spells learnable at a specific level for a class
pub fn get_spells_at_level(class: CharacterClass, level: u8) -> Vec<&'static SpellDef> {
    ALL_SPELLS.iter()
        .filter(|spell| {
            spell.classes.contains(&class)
            && spell.level_required == level
            && !spell.is_guardian_power
        })
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
    fn test_all_spells_have_unique_ids() {
        let mut ids: Vec<&str> = ALL_SPELLS.iter().map(|s| s.id).collect();
        ids.sort();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Duplicate spell IDs found");
    }

    #[test]
    fn test_all_spells_lookup() {
        for spell in ALL_SPELLS.iter() {
            let found = get_spell(spell.id);
            assert!(found.is_some(), "Failed to find spell: {}", spell.id);
            assert_eq!(found.unwrap().id, spell.id);
        }
    }

    #[test]
    fn test_get_spell_not_found() {
        assert!(get_spell("nonexistent_spell").is_none());
    }

    #[test]
    fn test_spells_have_valid_mp_cost() {
        for spell in ALL_SPELLS.iter() {
            assert!(spell.mp_cost >= 0, "Spell {} should have non-negative MP cost", spell.id);
        }
    }

    #[test]
    fn test_guardian_powers_exist() {
        let guardian_powers = get_guardian_powers();
        assert_eq!(guardian_powers.len(), 5, "Should have exactly 5 Guardian powers");

        // Verify all 5 Guardian powers
        assert!(get_spell("soul_sight").is_some());
        assert!(get_spell("stone_skin").is_some());
        assert!(get_spell("healing_tide").is_some());
        assert!(get_spell("haste").is_some());
        assert!(get_spell("inferno").is_some());
    }

    #[test]
    fn test_guardian_powers_are_marked() {
        for spell in get_guardian_powers() {
            assert!(spell.is_guardian_power, "Guardian power {} should be marked", spell.id);
            assert!(spell.classes.is_empty(), "Guardian power {} should have no classes", spell.id);
        }
    }

    #[test]
    fn test_cleric_has_healing_spells() {
        let cleric_spells = get_spells_for_class(CharacterClass::Cleric);
        let has_cure = cleric_spells.iter().any(|s| s.id == "cure");
        assert!(has_cure, "Cleric should have Cure spell");
    }

    #[test]
    fn test_wizard_has_attack_spells() {
        let wizard_spells = get_spells_for_class(CharacterClass::Wizard);
        let has_fire = wizard_spells.iter().any(|s| s.id == "fire");
        let has_blizzard = wizard_spells.iter().any(|s| s.id == "blizzard");
        let has_thunder = wizard_spells.iter().any(|s| s.id == "thunder");
        assert!(has_fire, "Wizard should have Fire spell");
        assert!(has_blizzard, "Wizard should have Blizzard spell");
        assert!(has_thunder, "Wizard should have Thunder spell");
    }
}
