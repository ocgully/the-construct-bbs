//! Crafting system for Mineteria
//!
//! Handles recipe matching and item crafting.

use super::data::{Recipe, ItemType, CraftingStation, BlockType, RECIPES};
use super::state::GameState;

/// Check if player can craft a recipe
pub fn can_craft(state: &GameState, recipe: &Recipe) -> bool {
    // Check station requirement
    if !has_crafting_station(state, recipe.station) {
        return false;
    }

    // Check all ingredients
    for (item, count) in recipe.ingredients.iter() {
        if !state.inventory.has_item(item, *count) {
            return false;
        }
    }

    true
}

/// Check if player has access to a crafting station
pub fn has_crafting_station(state: &GameState, station: CraftingStation) -> bool {
    match station {
        CraftingStation::Hand => true, // Always available
        CraftingStation::Workbench => state.is_near_block(BlockType::Workbench, 3),
        CraftingStation::Furnace => state.is_near_block(BlockType::Furnace, 3),
        CraftingStation::Anvil => false, // TODO: Implement anvil
    }
}

/// Craft an item from a recipe
pub fn craft_item(state: &mut GameState, recipe: &Recipe) -> Result<(), String> {
    if !can_craft(state, recipe) {
        return Err("Cannot craft: missing ingredients or station".to_string());
    }

    // Remove ingredients
    for (item, count) in recipe.ingredients.iter() {
        state.inventory.remove_item(item, *count);
    }

    // Add output
    let remaining = state.inventory.add_item(recipe.output, recipe.output_count);
    if remaining > 0 {
        // Inventory full, try to add back ingredients
        // For simplicity, just drop the excess (in a full game, would handle this better)
        return Err(format!("Crafted {} but {} couldn't fit in inventory",
            recipe.output.get_item().name, remaining));
    }

    state.stats.items_crafted += 1;

    Ok(())
}

/// Get all recipes the player can currently craft
pub fn available_recipes(state: &GameState) -> Vec<&'static Recipe> {
    RECIPES
        .iter()
        .filter(|r| can_craft(state, r))
        .collect()
}

/// Get all recipes for a specific station
pub fn recipes_for_station(station: CraftingStation) -> Vec<&'static Recipe> {
    RECIPES
        .iter()
        .filter(|r| r.station == station)
        .collect()
}

/// Get all recipes that produce a specific item
pub fn recipes_for_output(output: &ItemType) -> Vec<&'static Recipe> {
    RECIPES
        .iter()
        .filter(|r| r.output == *output)
        .collect()
}

/// Format a recipe for display
pub fn format_recipe(recipe: &Recipe) -> String {
    let output_item = recipe.output.get_item();
    let mut ingredients = String::new();

    for (i, (item, count)) in recipe.ingredients.iter().enumerate() {
        if i > 0 {
            ingredients.push_str(" + ");
        }
        let item_info = item.get_item();
        if *count > 1 {
            ingredients.push_str(&format!("{}x {}", count, item_info.name));
        } else {
            ingredients.push_str(item_info.name);
        }
    }

    let count_str = if recipe.output_count > 1 {
        format!(" (x{})", recipe.output_count)
    } else {
        String::new()
    };

    format!("{}{} <- {}", output_item.name, count_str, ingredients)
}

/// Get the station name for display
pub fn station_name(station: CraftingStation) -> &'static str {
    match station {
        CraftingStation::Hand => "Hand Crafting",
        CraftingStation::Workbench => "Workbench",
        CraftingStation::Furnace => "Furnace",
        CraftingStation::Anvil => "Anvil",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data::{ToolType, ToolMaterial};

    #[test]
    fn test_can_craft_hand() {
        let mut state = GameState::new(0);

        // Add wood to inventory
        state.inventory.add_item(ItemType::Block(BlockType::Wood), 4);

        // Find planks recipe
        let recipe = RECIPES.iter().find(|r| r.output == ItemType::Block(BlockType::Planks)).unwrap();

        // Should be able to craft planks from wood
        assert!(can_craft(&state, recipe));
    }

    #[test]
    fn test_cannot_craft_without_ingredients() {
        let state = GameState::new(0);

        // Find planks recipe
        let recipe = RECIPES.iter().find(|r| r.output == ItemType::Block(BlockType::Planks)).unwrap();

        // Should not be able to craft without wood
        assert!(!can_craft(&state, recipe));
    }

    #[test]
    fn test_craft_item() {
        let mut state = GameState::new(0);
        state.inventory.add_item(ItemType::Block(BlockType::Wood), 4);

        let recipe = RECIPES.iter().find(|r| r.output == ItemType::Block(BlockType::Planks)).unwrap();

        // Craft planks
        let result = craft_item(&mut state, recipe);
        assert!(result.is_ok());

        // Should have 4 planks (2 wood makes 4 planks, but we only had 1 wood)
        // Wait, recipe needs 1 wood for 4 planks
        assert_eq!(state.inventory.count_item(&ItemType::Block(BlockType::Planks)), 4);

        // Wood should be consumed
        assert_eq!(state.inventory.count_item(&ItemType::Block(BlockType::Wood)), 3);
    }

    #[test]
    fn test_workbench_requirement() {
        let mut state = GameState::new(0);

        // Add materials for wooden pickaxe
        state.inventory.add_item(ItemType::Block(BlockType::Planks), 10);
        state.inventory.add_item(ItemType::Stick, 10);

        // Find wooden pickaxe recipe
        let recipe = RECIPES.iter()
            .find(|r| r.output == ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Wood))
            .unwrap();

        // Should not be able to craft without workbench
        assert!(!can_craft(&state, recipe));

        // Place workbench near player
        state.set_modified_block(state.position.x + 1, state.position.y, BlockType::Workbench);

        // Now should be able to craft
        assert!(can_craft(&state, recipe));
    }

    #[test]
    fn test_format_recipe() {
        let recipe = RECIPES.iter().find(|r| r.output == ItemType::Block(BlockType::Planks)).unwrap();
        let formatted = format_recipe(recipe);
        assert!(formatted.contains("Planks"));
        assert!(formatted.contains("Wood"));
    }

    #[test]
    fn test_available_recipes() {
        let mut state = GameState::new(0);

        // With empty inventory, only no-ingredient recipes available
        let _available = available_recipes(&state);
        // Actually all hand recipes need at least wood or something
        // So empty inventory = no recipes

        // Add some wood
        state.inventory.add_item(ItemType::Block(BlockType::Wood), 10);

        let available = available_recipes(&state);
        assert!(!available.is_empty());
    }
}
