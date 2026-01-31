//! Screen state machine for Mineteria
//!
//! Manages game screens, input handling, and transitions.

use super::data::{BlockType, ItemType, ToolType, RECIPES, CraftingStation};
use super::state::{GameState, Position};
use super::world::World;
use super::crafting::{can_craft, craft_item, has_crafting_station};
use super::combat::{ActiveMonster, combat_round, attempt_flee, should_spawn_monster};

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// New game intro
    Intro,
    /// Main gameplay view
    Playing,
    /// Inventory management
    Inventory,
    /// Crafting menu
    Crafting { station: CraftingStation },
    /// Combat encounter
    Combat { monster: ActiveMonster },
    /// Chest contents
    ChestView { chest_pos: Position },
    /// Player stats
    Stats,
    /// Help screen
    Help,
    /// Game over
    GameOver,
    /// Confirm quit
    ConfirmQuit,
}

/// Actions returned by MineteriaFlow for session to handle
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Continue - no output needed
    Continue,
    /// Show screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game is over
    GameOver { score: i64 },
    /// Player quit to BBS menu
    Quit,
}

/// Mineteria game flow state machine
pub struct MineteriaFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub world: World,
    input_buffer: String,
    /// Currently visible area cache
    visible_cache: Option<Vec<Vec<BlockType>>>,
}

impl MineteriaFlow {
    /// Create new game from fresh state
    pub fn new(seed: u64) -> Self {
        let world = World::new(seed);
        let spawn = world.find_spawn_point();

        let mut state = GameState::new(seed);
        state.position = spawn;
        state.spawn_point = spawn;

        // Give starting items
        state.inventory.add_item(ItemType::Tool(ToolType::Pickaxe, super::data::ToolMaterial::Wood), 1);
        state.inventory.add_item(ItemType::Tool(ToolType::Axe, super::data::ToolMaterial::Wood), 1);
        state.inventory.add_item(ItemType::Block(BlockType::Torch), 16);

        Self {
            state,
            screen: GameScreen::Intro,
            world,
            input_buffer: String::new(),
            visible_cache: None,
        }
    }

    /// Resume game from loaded state
    pub fn from_state(state: GameState) -> Self {
        let world = World::new(state.world_seed);
        Self {
            state,
            screen: GameScreen::Intro,
            world,
            input_buffer: String::new(),
            visible_cache: None,
        }
    }

    /// Get current screen for rendering
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> GameAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return GameAction::Echo("\x08 \x08".to_string());
            }
            return GameAction::Continue;
        }

        // Single-key screens process immediately (including Enter key)
        if self.is_single_key_screen() {
            // For single-key screens, Enter triggers action even with empty buffer
            if ch == '\r' || ch == '\n' {
                self.input_buffer.clear();
                return self.process_input();
            }
            // Ignore other control chars
            if ch.is_control() {
                return GameAction::Continue;
            }
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Enter processing (only for buffered input screens)
        if ch == '\r' || ch == '\n' {
            if !self.input_buffer.is_empty() {
                return self.process_input();
            }
            return GameAction::Continue;
        }

        // Ignore control chars
        if ch.is_control() {
            return GameAction::Continue;
        }

        // Buffer input
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return GameAction::Echo(ch.to_string());
        }

        GameAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::Playing
                | GameScreen::Inventory
                | GameScreen::Crafting { .. }
                | GameScreen::Combat { .. }
                | GameScreen::ChestView { .. }
                | GameScreen::Stats
                | GameScreen::Help
                | GameScreen::GameOver
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> GameAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::Playing => self.handle_playing(&input),
            GameScreen::Inventory => self.handle_inventory(&input),
            GameScreen::Crafting { station } => self.handle_crafting(&input, *station),
            GameScreen::Combat { monster } => {
                let monster = monster.clone();
                self.handle_combat(&input, monster)
            }
            GameScreen::ChestView { chest_pos } => {
                let pos = *chest_pos;
                self.handle_chest(&input, pos)
            }
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Help => self.handle_help(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> GameAction {
        self.screen = GameScreen::Playing;
        GameAction::SaveGame
    }

    fn handle_playing(&mut self, input: &str) -> GameAction {
        self.state.last_message = None;
        self.visible_cache = None;

        match input {
            // Movement
            "W" | "K" => self.try_move(0, 1),   // Up
            "S" | "J" => self.try_move(0, -1),  // Down
            "A" | "H" => self.try_move(-1, 0),  // Left
            "D" | "L" => self.try_move(1, 0),   // Right

            // Cursor movement in build mode
            "8" if self.state.build_mode => {
                self.state.move_cursor(0, 1);
                GameAction::SaveGame
            }
            "2" if self.state.build_mode => {
                self.state.move_cursor(0, -1);
                GameAction::SaveGame
            }
            "4" if self.state.build_mode => {
                self.state.move_cursor(-1, 0);
                GameAction::SaveGame
            }
            "6" if self.state.build_mode => {
                self.state.move_cursor(1, 0);
                GameAction::SaveGame
            }

            // Toggle build mode
            "B" => {
                self.state.build_mode = !self.state.build_mode;
                self.state.reset_cursor();
                self.state.last_message = Some(if self.state.build_mode {
                    "Build mode ON. Use numpad to move cursor.".to_string()
                } else {
                    "Build mode OFF.".to_string()
                });
                GameAction::SaveGame
            }

            // Mine block at cursor
            "M" | " " => self.mine_block(),

            // Place block at cursor
            "P" => self.place_block(),

            // Hotbar selection (1-9)
            c if c.len() == 1 && c.chars().next().unwrap().is_ascii_digit() => {
                let digit = c.chars().next().unwrap().to_digit(10).unwrap() as u8;
                if digit >= 1 && digit <= 9 {
                    self.state.inventory.select_slot(digit - 1);
                }
                GameAction::SaveGame
            }

            // Open screens
            "I" => {
                self.screen = GameScreen::Inventory;
                GameAction::SaveGame
            }
            "C" => {
                // Open crafting - check for nearby stations
                let station = if self.state.is_near_block(BlockType::Furnace, 3) {
                    CraftingStation::Furnace
                } else if self.state.is_near_block(BlockType::Workbench, 3) {
                    CraftingStation::Workbench
                } else {
                    CraftingStation::Hand
                };
                self.screen = GameScreen::Crafting { station };
                GameAction::SaveGame
            }
            "Y" => {
                self.screen = GameScreen::Stats;
                GameAction::SaveGame
            }
            "?" | "F1" => {
                self.screen = GameScreen::Help;
                GameAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                GameAction::SaveGame
            }

            // Interact with chest
            "E" => self.interact(),

            // Eat food
            "F" => self.eat_food(),

            _ => GameAction::Continue,
        }
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> GameAction {
        let new_pos = self.state.position.offset(dx, dy);

        // Check if destination is solid
        if self.world.is_solid(&self.state, new_pos.x, new_pos.y) {
            self.state.last_message = Some("Blocked!".to_string());
            return GameAction::SaveGame;
        }

        // Check for gravity - if moving up without ladder/support
        if dy > 0 {
            let current_block = self.world.get_block(&self.state, self.state.position.x, self.state.position.y);
            if current_block != BlockType::Ladder && current_block != BlockType::Water {
                // Check if standing on solid ground
                let below = self.world.get_block(&self.state, self.state.position.x, self.state.position.y - 1);
                if !below.get_block().solid {
                    self.state.last_message = Some("Can't jump without support!".to_string());
                    return GameAction::SaveGame;
                }
            }
        }

        // Move player
        self.state.position = new_pos;
        self.state.stats.distance_walked += 1;

        // Track depth
        if new_pos.y < self.state.stats.depth_reached {
            self.state.stats.depth_reached = new_pos.y;
        }

        // Advance time slightly
        self.state.advance_time(10);

        // Check for lava damage
        let current_block = self.world.get_block(&self.state, new_pos.x, new_pos.y);
        if current_block == BlockType::Lava {
            if self.state.take_damage(4) {
                self.state.respawn();
            } else {
                self.state.last_message = Some("Burning in lava! -4 HP".to_string());
            }
        }

        // Check for monster spawn
        if let Some(monster_type) = should_spawn_monster(&self.state, new_pos.x + 10, new_pos.y) {
            let monster = super::combat::ActiveMonster::new(
                monster_type,
                new_pos.x + 10,
                new_pos.y,
            );
            self.state.last_message = Some(format!("A {} appears!", monster.get_stats().name));
            self.screen = GameScreen::Combat { monster };
        }

        GameAction::SaveGame
    }

    fn mine_block(&mut self) -> GameAction {
        let target = if self.state.build_mode {
            self.state.cursor_position()
        } else {
            // Mine block player is standing in or facing
            self.state.position.offset(0, 0)
        };

        let block = self.world.get_block(&self.state, target.x, target.y);
        let block_info = block.get_block();

        if block == BlockType::Air || block == BlockType::Bedrock {
            self.state.last_message = Some("Nothing to mine here.".to_string());
            return GameAction::SaveGame;
        }

        // Check tool requirement
        if let Some(required_tool) = block_info.tool_required {
            if let Some((material, _, _)) = self.state.inventory.get_best_tool(required_tool) {
                if material.tier() < block_info.min_tool_tier {
                    self.state.last_message = Some(format!(
                        "Need a better {} to mine {}!",
                        format!("{:?}", required_tool).to_lowercase(),
                        block_info.name
                    ));
                    return GameAction::SaveGame;
                }
            } else {
                self.state.last_message = Some(format!(
                    "Need a {} to mine {}!",
                    format!("{:?}", required_tool).to_lowercase(),
                    block_info.name
                ));
                return GameAction::SaveGame;
            }
        }

        // Use tool durability
        self.state.inventory.use_selected_tool();

        // Get drop (either the block itself or its designated drop)
        let drop_block = block_info.drops.unwrap_or(block);
        let drop_item = ItemType::Block(drop_block);

        // Add to inventory
        let remaining = self.state.inventory.add_item(drop_item, 1);
        if remaining > 0 {
            self.state.last_message = Some("Inventory full!".to_string());
        } else {
            self.state.last_message = Some(format!("Mined {}.", block_info.name));
        }

        // Remove block from world
        self.state.set_modified_block(target.x, target.y, BlockType::Air);
        self.state.stats.blocks_mined += 1;

        // Advance time
        self.state.advance_time(50);

        GameAction::SaveGame
    }

    fn place_block(&mut self) -> GameAction {
        let target = if self.state.build_mode {
            self.state.cursor_position()
        } else {
            // Can't place where standing
            self.state.last_message = Some("Toggle build mode (B) to place blocks.".to_string());
            return GameAction::SaveGame;
        };

        // Check if target is empty
        let current = self.world.get_block(&self.state, target.x, target.y);
        if current != BlockType::Air && current != BlockType::Water {
            self.state.last_message = Some("Can't place here - not empty!".to_string());
            return GameAction::SaveGame;
        }

        // Check selected item is a placeable block
        let block_to_place = self.state.inventory.selected_item()
            .and_then(|slot| {
                if let ItemType::Block(block_type) = slot.item {
                    Some((block_type, slot.item.clone()))
                } else {
                    None
                }
            });

        if let Some((block_type, item_type)) = block_to_place {
            // Place the block
            self.state.set_modified_block(target.x, target.y, block_type);
            self.state.inventory.remove_item(&item_type, 1);
            self.state.stats.blocks_placed += 1;

            let block_info = block_type.get_block();
            self.state.last_message = Some(format!("Placed {}.", block_info.name));

            // Advance time
            self.state.advance_time(20);

            return GameAction::SaveGame;
        }

        self.state.last_message = Some("Select a block to place.".to_string());
        GameAction::SaveGame
    }

    fn interact(&mut self) -> GameAction {
        // Check for chest at cursor or adjacent
        let positions = [
            self.state.position,
            self.state.position.offset(1, 0),
            self.state.position.offset(-1, 0),
            self.state.position.offset(0, 1),
            self.state.position.offset(0, -1),
        ];

        for pos in positions {
            let block = self.world.get_block(&self.state, pos.x, pos.y);
            if block == BlockType::Chest {
                self.screen = GameScreen::ChestView { chest_pos: pos };
                return GameAction::SaveGame;
            }
        }

        self.state.last_message = Some("Nothing to interact with.".to_string());
        GameAction::SaveGame
    }

    fn eat_food(&mut self) -> GameAction {
        // Try to eat from hotbar
        for i in 0..9 {
            if let Some(slot) = &self.state.inventory.hotbar[i] {
                let item = slot.item;
                if matches!(item, ItemType::Apple | ItemType::Bread | ItemType::CookedMeat | ItemType::RawMeat) {
                    if self.state.eat(&item) {
                        self.state.last_message = Some(format!("Ate {}. Hunger: {}/{}",
                            item.get_item().name, self.state.hunger, self.state.max_hunger));
                        return GameAction::SaveGame;
                    }
                }
            }
        }

        self.state.last_message = Some("No food to eat!".to_string());
        GameAction::SaveGame
    }

    fn handle_inventory(&mut self, input: &str) -> GameAction {
        match input {
            "Q" | "I" => {
                self.screen = GameScreen::Playing;
                GameAction::SaveGame
            }
            // Could add inventory management here
            _ => GameAction::Continue,
        }
    }

    fn handle_crafting(&mut self, input: &str, station: CraftingStation) -> GameAction {
        if input == "Q" || input == "C" {
            self.screen = GameScreen::Playing;
            return GameAction::SaveGame;
        }

        // Switch station
        if input == "W" && has_crafting_station(&self.state, CraftingStation::Workbench) {
            self.screen = GameScreen::Crafting { station: CraftingStation::Workbench };
            return GameAction::SaveGame;
        }
        if input == "F" && has_crafting_station(&self.state, CraftingStation::Furnace) {
            self.screen = GameScreen::Crafting { station: CraftingStation::Furnace };
            return GameAction::SaveGame;
        }
        if input == "H" {
            self.screen = GameScreen::Crafting { station: CraftingStation::Hand };
            return GameAction::SaveGame;
        }

        // Craft by number
        if let Ok(num) = input.parse::<usize>() {
            let recipes: Vec<_> = RECIPES
                .iter()
                .filter(|r| r.station == station)
                .collect();

            if num >= 1 && num <= recipes.len() {
                let recipe = recipes[num - 1];
                if can_craft(&self.state, recipe) {
                    match craft_item(&mut self.state, recipe) {
                        Ok(()) => {
                            self.state.last_message = Some(format!(
                                "Crafted {} {}!",
                                recipe.output_count,
                                recipe.output.get_item().name
                            ));
                        }
                        Err(e) => {
                            self.state.last_message = Some(e);
                        }
                    }
                } else {
                    self.state.last_message = Some("Missing ingredients!".to_string());
                }
            }
        }

        GameAction::SaveGame
    }

    fn handle_combat(&mut self, input: &str, mut monster: ActiveMonster) -> GameAction {
        match input {
            "A" | "F" => {
                // Attack
                let result = combat_round(&mut self.state, &mut monster);
                self.state.last_message = Some(result.message);

                if result.player_killed {
                    self.state.respawn();
                    self.screen = GameScreen::Playing;
                } else if result.monster_killed {
                    self.screen = GameScreen::Playing;
                } else {
                    self.screen = GameScreen::Combat { monster };
                }
            }
            "R" => {
                // Run
                let monster_stats = monster.get_stats();
                let (success, damage) = attempt_flee(&self.state, &monster_stats);

                if success {
                    self.state.last_message = Some("You fled successfully!".to_string());
                    self.screen = GameScreen::Playing;
                } else {
                    if self.state.take_damage(damage) {
                        self.state.respawn();
                        self.screen = GameScreen::Playing;
                    } else {
                        self.state.last_message = Some(format!(
                            "Failed to flee! The {} hit you for {} damage.",
                            monster_stats.name, damage
                        ));
                        self.screen = GameScreen::Combat { monster };
                    }
                }
            }
            _ => return GameAction::Continue,
        }

        GameAction::SaveGame
    }

    fn handle_chest(&mut self, input: &str, _chest_pos: Position) -> GameAction {
        if input == "Q" || input == "E" {
            self.screen = GameScreen::Playing;
            return GameAction::SaveGame;
        }

        // TODO: Implement chest transfer
        GameAction::Continue
    }

    fn handle_stats(&mut self, _input: &str) -> GameAction {
        self.screen = GameScreen::Playing;
        GameAction::SaveGame
    }

    fn handle_help(&mut self, _input: &str) -> GameAction {
        self.screen = GameScreen::Playing;
        GameAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> GameAction {
        let score = (self.state.stats.blocks_mined as i64 * 10)
            + (self.state.stats.monsters_killed as i64 * 50)
            + (self.state.day as i64 * 100);

        GameAction::GameOver { score }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> GameAction {
        match input {
            "Y" => GameAction::Quit,
            _ => {
                self.screen = GameScreen::Playing;
                GameAction::SaveGame
            }
        }
    }

    /// Get visible area for rendering
    pub fn get_visible_area(&self, width: i32, height: i32) -> Vec<Vec<BlockType>> {
        self.world.get_visible_area(&self.state, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let flow = MineteriaFlow::new(12345);
        assert!(matches!(flow.screen, GameScreen::Intro));
        assert_eq!(flow.state.health, 20);
    }

    #[test]
    fn test_intro_to_playing() {
        let mut flow = MineteriaFlow::new(12345);
        let action = flow.handle_char('\r');
        assert!(matches!(action, GameAction::SaveGame));
        assert!(matches!(flow.screen, GameScreen::Playing));
    }

    #[test]
    fn test_movement() {
        let mut flow = MineteriaFlow::new(12345);
        flow.screen = GameScreen::Playing;

        let initial_x = flow.state.position.x;

        // Try moving right
        flow.handle_char('D');

        // Position may or may not change depending on terrain
        // Just verify no crash
        assert!(flow.state.position.x >= initial_x - 1);
    }

    #[test]
    fn test_build_mode_toggle() {
        let mut flow = MineteriaFlow::new(12345);
        flow.screen = GameScreen::Playing;

        assert!(!flow.state.build_mode);
        flow.handle_char('B');
        assert!(flow.state.build_mode);
        flow.handle_char('B');
        assert!(!flow.state.build_mode);
    }

    #[test]
    fn test_hotbar_selection() {
        let mut flow = MineteriaFlow::new(12345);
        flow.screen = GameScreen::Playing;

        flow.handle_char('5');
        assert_eq!(flow.state.inventory.selected_slot, 4);
    }

    #[test]
    fn test_open_inventory() {
        let mut flow = MineteriaFlow::new(12345);
        flow.screen = GameScreen::Playing;

        flow.handle_char('I');
        assert!(matches!(flow.screen, GameScreen::Inventory));

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::Playing));
    }

    #[test]
    fn test_open_crafting() {
        let mut flow = MineteriaFlow::new(12345);
        flow.screen = GameScreen::Playing;

        flow.handle_char('C');
        assert!(matches!(flow.screen, GameScreen::Crafting { .. }));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = MineteriaFlow::new(12345);
        flow.screen = GameScreen::Playing;

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        // Cancel quit
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::Playing));

        // Confirm quit
        flow.handle_char('Q');
        let action = flow.handle_char('Y');
        assert!(matches!(action, GameAction::Quit));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let flow = MineteriaFlow::new(42);
        let json = serde_json::to_string(flow.game_state()).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.world_seed, 42);
    }
}
