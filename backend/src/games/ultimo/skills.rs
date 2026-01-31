//! Skill system for Ultimo
//!
//! Handles skill progression, requirements, and effects.

use super::data::{get_skill, SkillCategory, SKILLS};
use super::state::Character;

/// Get all skills in a category
pub fn get_skills_by_category(category: SkillCategory) -> Vec<&'static super::data::Skill> {
    SKILLS.iter().filter(|s| s.category == category).collect()
}

/// Calculate effective skill level (including bonuses)
pub fn effective_skill(char: &Character, skill_key: &str) -> u32 {
    let base = char.get_skill(skill_key);

    // Add stat bonuses
    let stat_bonus = if let Some(skill) = get_skill(skill_key) {
        match skill.category {
            SkillCategory::Combat => char.strength as u32 / 10,
            SkillCategory::Magic => char.intelligence as u32 / 10,
            SkillCategory::Crafting => (char.intelligence as u32 + char.dexterity as u32) / 20,
            SkillCategory::Gathering => char.strength as u32 / 10,
            SkillCategory::Miscellaneous => char.dexterity as u32 / 10,
        }
    } else {
        0
    };

    (base + stat_bonus).min(120) // Cap at 120 effective
}

/// Calculate training cost for next point
pub fn training_cost(current_level: u32) -> i64 {
    // Cost increases with skill level
    // 10 gold per point at low levels, scaling up
    let base = 10i64;
    let multiplier = 1 + (current_level as i64 / 20);
    base * (current_level as i64 + 1) * multiplier
}

/// Check if character meets skill requirements for an item
pub fn meets_skill_requirement(char: &Character, skill: &str, required_level: u32) -> bool {
    char.get_skill(skill) >= required_level
}

/// Get total skill points across all skills
pub fn total_skill_points(char: &Character) -> u32 {
    char.skills.values().sum()
}

/// Get skills above a certain level
pub fn skills_above_level(char: &Character, min_level: u32) -> Vec<(&str, u32)> {
    char.skills
        .iter()
        .filter(|(_, &level)| level >= min_level)
        .map(|(key, &level)| (key.as_str(), level))
        .collect()
}

/// Determine character's "class" based on highest skills
/// Returns a descriptive title based on skill distribution
pub fn determine_title(char: &Character) -> &'static str {
    let combat_total: u32 = ["swordsmanship", "mace_fighting", "archery", "tactics", "parrying", "wrestling"]
        .iter()
        .map(|s| char.get_skill(s))
        .sum();

    let magic_total: u32 = ["magery", "meditation", "resist_spells", "eval_int"]
        .iter()
        .map(|s| char.get_skill(s))
        .sum();

    let crafting_total: u32 = ["blacksmithing", "tailoring", "carpentry", "alchemy", "cooking"]
        .iter()
        .map(|s| char.get_skill(s))
        .sum();

    let gathering_total: u32 = ["mining", "lumberjacking", "fishing", "herbalism"]
        .iter()
        .map(|s| char.get_skill(s))
        .sum();

    let misc_total: u32 = ["healing", "animal_taming", "hiding", "stealth"]
        .iter()
        .map(|s| char.get_skill(s))
        .sum();

    // Determine primary focus
    let max = combat_total.max(magic_total).max(crafting_total).max(gathering_total).max(misc_total);

    if max < 50 {
        return "Novice";
    }

    // Primary title
    let primary = if combat_total == max {
        // Check specific combat style
        let sword = char.get_skill("swordsmanship");
        let mace = char.get_skill("mace_fighting");
        let archery = char.get_skill("archery");

        if sword >= mace && sword >= archery {
            if sword >= 80 { "Swordmaster" } else { "Swordsman" }
        } else if mace >= archery {
            if mace >= 80 { "Champion" } else { "Fighter" }
        } else {
            if archery >= 80 { "Marksman" } else { "Archer" }
        }
    } else if magic_total == max {
        if char.get_skill("magery") >= 80 {
            "Archmage"
        } else if char.get_skill("magery") >= 50 {
            "Mage"
        } else {
            "Apprentice Mage"
        }
    } else if crafting_total == max {
        // Check specific craft
        let blacksmith = char.get_skill("blacksmithing");
        let tailor = char.get_skill("tailoring");
        let carpenter = char.get_skill("carpentry");
        let alchemist = char.get_skill("alchemy");

        if blacksmith >= tailor && blacksmith >= carpenter && blacksmith >= alchemist {
            if blacksmith >= 80 { "Master Smith" } else { "Blacksmith" }
        } else if tailor >= carpenter && tailor >= alchemist {
            if tailor >= 80 { "Master Tailor" } else { "Tailor" }
        } else if carpenter >= alchemist {
            if carpenter >= 80 { "Master Carpenter" } else { "Carpenter" }
        } else {
            if alchemist >= 80 { "Master Alchemist" } else { "Alchemist" }
        }
    } else if gathering_total == max {
        if char.get_skill("mining") >= 50 {
            "Miner"
        } else if char.get_skill("lumberjacking") >= 50 {
            "Lumberjack"
        } else if char.get_skill("fishing") >= 50 {
            "Fisherman"
        } else {
            "Gatherer"
        }
    } else {
        // Misc skills
        if char.get_skill("animal_taming") >= 50 {
            "Tamer"
        } else if char.get_skill("hiding") >= 50 || char.get_skill("stealth") >= 50 {
            "Rogue"
        } else if char.get_skill("healing") >= 50 {
            "Healer"
        } else {
            "Adventurer"
        }
    };

    primary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_by_category() {
        let combat_skills = get_skills_by_category(SkillCategory::Combat);
        assert!(!combat_skills.is_empty());
        assert!(combat_skills.iter().all(|s| s.category == SkillCategory::Combat));
    }

    #[test]
    fn test_training_cost() {
        assert_eq!(training_cost(0), 10);
        assert!(training_cost(50) > training_cost(25));
        assert!(training_cost(99) > training_cost(50));
    }

    #[test]
    fn test_effective_skill() {
        let mut char = Character::new("Test", 1);
        char.skills.insert("swordsmanship".to_string(), 50);
        char.strength = 30;

        let effective = effective_skill(&char, "swordsmanship");
        assert!(effective >= 50); // Base + str bonus
    }

    #[test]
    fn test_determine_title() {
        let mut char = Character::new("Test", 1);

        // Novice
        assert_eq!(determine_title(&char), "Novice");

        // Warrior
        char.skills.insert("swordsmanship".to_string(), 80);
        char.skills.insert("tactics".to_string(), 60);
        assert_eq!(determine_title(&char), "Swordmaster");
    }

    #[test]
    fn test_total_skill_points() {
        let mut char = Character::new("Test", 1);
        char.skills.insert("swordsmanship".to_string(), 50);
        char.skills.insert("magery".to_string(), 30);

        let total = total_skill_points(&char);
        assert!(total >= 80); // At least these two + starting skills
    }
}
