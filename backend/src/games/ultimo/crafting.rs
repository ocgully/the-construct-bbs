//! Crafting system for Ultimo
//!
//! Handles item creation through various professions.

use super::data::get_item;
use super::state::Character;
use serde::Serialize;

/// A crafting recipe
#[derive(Debug, Clone, Serialize)]
pub struct Recipe {
    pub key: &'static str,
    pub name: &'static str,
    pub output_item: &'static str,
    pub output_quantity: u32,
    pub required_skill: &'static str,
    pub min_skill: u32,
    /// Input materials (item_key, quantity)
    pub materials: &'static [(&'static str, u32)],
    /// Skill gain chance (0-100)
    pub skill_gain_chance: u32,
}

/// All available recipes
pub static RECIPES: &[Recipe] = &[
    // Blacksmithing recipes
    Recipe {
        key: "smelt_iron",
        name: "Smelt Iron Ore",
        output_item: "iron_ingot",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 0,
        materials: &[("iron_ore", 2)],
        skill_gain_chance: 30,
    },
    Recipe {
        key: "smelt_gold",
        name: "Smelt Gold Ore",
        output_item: "gold_ingot",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 40,
        materials: &[("gold_ore", 2)],
        skill_gain_chance: 25,
    },
    Recipe {
        key: "forge_dagger",
        name: "Forge Dagger",
        output_item: "dagger",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 10,
        materials: &[("iron_ingot", 2)],
        skill_gain_chance: 25,
    },
    Recipe {
        key: "forge_short_sword",
        name: "Forge Short Sword",
        output_item: "short_sword",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 25,
        materials: &[("iron_ingot", 4)],
        skill_gain_chance: 20,
    },
    Recipe {
        key: "forge_long_sword",
        name: "Forge Long Sword",
        output_item: "long_sword",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 45,
        materials: &[("iron_ingot", 6)],
        skill_gain_chance: 15,
    },
    Recipe {
        key: "forge_broadsword",
        name: "Forge Broadsword",
        output_item: "broadsword",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 65,
        materials: &[("iron_ingot", 8)],
        skill_gain_chance: 10,
    },
    Recipe {
        key: "forge_chain_mail",
        name: "Forge Chain Mail",
        output_item: "chain_mail",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 55,
        materials: &[("iron_ingot", 15)],
        skill_gain_chance: 12,
    },
    Recipe {
        key: "forge_plate_armor",
        name: "Forge Plate Armor",
        output_item: "plate_armor",
        output_quantity: 1,
        required_skill: "blacksmithing",
        min_skill: 80,
        materials: &[("iron_ingot", 25)],
        skill_gain_chance: 8,
    },
    // Tailoring recipes
    Recipe {
        key: "sew_cloth_tunic",
        name: "Sew Cloth Tunic",
        output_item: "cloth_tunic",
        output_quantity: 1,
        required_skill: "tailoring",
        min_skill: 0,
        materials: &[("cloth", 5)],
        skill_gain_chance: 30,
    },
    Recipe {
        key: "sew_leather_armor",
        name: "Sew Leather Armor",
        output_item: "leather_armor",
        output_quantity: 1,
        required_skill: "tailoring",
        min_skill: 30,
        materials: &[("leather", 10)],
        skill_gain_chance: 20,
    },
    Recipe {
        key: "make_bandages",
        name: "Make Bandages",
        output_item: "bandage",
        output_quantity: 5,
        required_skill: "tailoring",
        min_skill: 0,
        materials: &[("cloth", 1)],
        skill_gain_chance: 20,
    },
    // Carpentry recipes
    Recipe {
        key: "cut_boards",
        name: "Cut Boards",
        output_item: "board",
        output_quantity: 2,
        required_skill: "carpentry",
        min_skill: 0,
        materials: &[("wood", 1)],
        skill_gain_chance: 30,
    },
    Recipe {
        key: "craft_bow",
        name: "Craft Bow",
        output_item: "bow",
        output_quantity: 1,
        required_skill: "carpentry",
        min_skill: 20,
        materials: &[("board", 5)],
        skill_gain_chance: 25,
    },
    Recipe {
        key: "craft_arrows",
        name: "Craft Arrows",
        output_item: "arrow",
        output_quantity: 20,
        required_skill: "carpentry",
        min_skill: 10,
        materials: &[("board", 1)],
        skill_gain_chance: 20,
    },
    Recipe {
        key: "craft_wooden_shield",
        name: "Craft Wooden Shield",
        output_item: "wooden_shield",
        output_quantity: 1,
        required_skill: "carpentry",
        min_skill: 25,
        materials: &[("board", 8)],
        skill_gain_chance: 20,
    },
    // Alchemy recipes
    Recipe {
        key: "brew_lesser_heal",
        name: "Brew Lesser Heal Potion",
        output_item: "lesser_heal_potion",
        output_quantity: 1,
        required_skill: "alchemy",
        min_skill: 0,
        materials: &[("ginseng", 2)],
        skill_gain_chance: 30,
    },
    Recipe {
        key: "brew_heal",
        name: "Brew Heal Potion",
        output_item: "heal_potion",
        output_quantity: 1,
        required_skill: "alchemy",
        min_skill: 30,
        materials: &[("ginseng", 3), ("garlic", 1)],
        skill_gain_chance: 20,
    },
    Recipe {
        key: "brew_greater_heal",
        name: "Brew Greater Heal Potion",
        output_item: "greater_heal_potion",
        output_quantity: 1,
        required_skill: "alchemy",
        min_skill: 60,
        materials: &[("ginseng", 5), ("garlic", 2), ("mandrake_root", 1)],
        skill_gain_chance: 12,
    },
    Recipe {
        key: "brew_mana",
        name: "Brew Mana Potion",
        output_item: "mana_potion",
        output_quantity: 1,
        required_skill: "alchemy",
        min_skill: 40,
        materials: &[("nightshade", 2), ("spider_silk", 1)],
        skill_gain_chance: 18,
    },
    // Cooking recipes
    Recipe {
        key: "cook_fish",
        name: "Cook Fish",
        output_item: "cooked_meat",
        output_quantity: 1,
        required_skill: "cooking",
        min_skill: 0,
        materials: &[("raw_fish", 1)],
        skill_gain_chance: 40,
    },
    Recipe {
        key: "cook_meat",
        name: "Cook Meat",
        output_item: "cooked_meat",
        output_quantity: 1,
        required_skill: "cooking",
        min_skill: 0,
        materials: &[("raw_meat", 1)],
        skill_gain_chance: 40,
    },
];

/// Get recipe by key
pub fn get_recipe(key: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.key == key)
}

/// Get all recipes for a skill
pub fn get_recipes_for_skill(skill: &str) -> Vec<&'static Recipe> {
    RECIPES
        .iter()
        .filter(|r| r.required_skill == skill)
        .collect()
}

/// Get recipes available to a character
pub fn get_available_recipes(char: &Character) -> Vec<&'static Recipe> {
    RECIPES
        .iter()
        .filter(|r| char.get_skill(r.required_skill) >= r.min_skill)
        .collect()
}

/// Check if character can craft a recipe
pub fn can_craft(char: &Character, recipe_key: &str) -> Result<(), String> {
    let recipe = get_recipe(recipe_key).ok_or("Recipe not found")?;

    // Check skill level
    let skill_level = char.get_skill(recipe.required_skill);
    if skill_level < recipe.min_skill {
        return Err(format!(
            "Need {} {} skill (have {})",
            recipe.min_skill, recipe.required_skill, skill_level
        ));
    }

    // Check materials
    for (item_key, required) in recipe.materials {
        let have = char.get_item_count(item_key);
        if have < *required {
            let item_name = get_item(item_key)
                .map(|i| i.name)
                .unwrap_or(*item_key);
            return Err(format!(
                "Need {} {} (have {})",
                required, item_name, have
            ));
        }
    }

    // Check inventory space
    if char.inventory.len() as u32 >= char.max_inventory_slots {
        return Err("Inventory full".to_string());
    }

    Ok(())
}

/// Result of a crafting attempt
#[derive(Debug)]
pub struct CraftResult {
    pub success: bool,
    pub item_created: Option<String>,
    pub quantity: u32,
    pub skill_gained: bool,
    pub message: String,
}

/// Attempt to craft an item
pub fn craft(char: &mut Character, recipe_key: &str) -> CraftResult {
    // Check if can craft
    if let Err(msg) = can_craft(char, recipe_key) {
        return CraftResult {
            success: false,
            item_created: None,
            quantity: 0,
            skill_gained: false,
            message: msg,
        };
    }

    let recipe = get_recipe(recipe_key).unwrap();

    // Consume materials
    for (item_key, required) in recipe.materials {
        char.remove_item(item_key, *required);
    }

    // Success chance based on skill
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let skill_level = char.get_skill(recipe.required_skill);
    let success_chance = 50 + skill_level as i32 - recipe.min_skill as i32;

    if rng.gen_range(0..100) < success_chance {
        // Success - create item
        char.add_item(recipe.output_item, recipe.output_quantity);

        // Try skill gain
        let skill_gained = rng.gen_range(0..100) < recipe.skill_gain_chance
            && char.try_skill_gain(recipe.required_skill, recipe.min_skill);

        let item_name = get_item(recipe.output_item)
            .map(|i| i.name)
            .unwrap_or(recipe.output_item);

        CraftResult {
            success: true,
            item_created: Some(recipe.output_item.to_string()),
            quantity: recipe.output_quantity,
            skill_gained,
            message: format!(
                "Created {} {}!{}",
                recipe.output_quantity,
                item_name,
                if skill_gained {
                    " Skill increased!"
                } else {
                    ""
                }
            ),
        }
    } else {
        // Failure - materials lost
        // Try skill gain even on failure
        let skill_gained = rng.gen_range(0..100) < recipe.skill_gain_chance / 2
            && char.try_skill_gain(recipe.required_skill, recipe.min_skill);

        CraftResult {
            success: false,
            item_created: None,
            quantity: 0,
            skill_gained,
            message: format!(
                "Crafting failed! Materials lost.{}",
                if skill_gained {
                    " Skill increased!"
                } else {
                    ""
                }
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_recipe() {
        let recipe = get_recipe("smelt_iron");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().output_item, "iron_ingot");
    }

    #[test]
    fn test_get_recipes_for_skill() {
        let blacksmith_recipes = get_recipes_for_skill("blacksmithing");
        assert!(!blacksmith_recipes.is_empty());
        assert!(blacksmith_recipes.iter().all(|r| r.required_skill == "blacksmithing"));
    }

    #[test]
    fn test_can_craft() {
        let mut char = Character::new("Crafter", 1);
        char.skills.insert("blacksmithing".to_string(), 50);
        char.add_item("iron_ore", 10);

        // Should be able to smelt iron
        assert!(can_craft(&char, "smelt_iron").is_ok());

        // Should not be able to forge plate (need 80 skill)
        assert!(can_craft(&char, "forge_plate_armor").is_err());
    }

    #[test]
    fn test_craft_consumes_materials() {
        let mut char = Character::new("Crafter", 1);
        char.skills.insert("blacksmithing".to_string(), 50);
        char.add_item("iron_ore", 4);

        let initial_ore = char.get_item_count("iron_ore");

        let result = craft(&mut char, "smelt_iron");

        // Materials should be consumed regardless of success
        assert!(char.get_item_count("iron_ore") < initial_ore);

        // If successful, should have ingots
        if result.success {
            assert!(char.get_item_count("iron_ingot") > 0);
        }
    }

    #[test]
    fn test_available_recipes() {
        let mut char = Character::new("Novice", 1);
        char.skills.insert("blacksmithing".to_string(), 10);

        let available = get_available_recipes(&char);

        // Should have low-level recipes
        assert!(available.iter().any(|r| r.key == "smelt_iron"));

        // Should not have high-level recipes
        assert!(!available.iter().any(|r| r.key == "forge_plate_armor"));
    }
}
