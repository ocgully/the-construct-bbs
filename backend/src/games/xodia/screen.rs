//! Screen state machine for Xodia
//!
//! Manages game flow, screen transitions, and input handling.

use super::state::{GameState, CharacterClass};
use super::world::WorldState;
use super::parser::{parse_command, CommandType};
use super::combat::{CombatState, CombatAction, resolve_combat_round};
use super::render;

/// Game screens
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Initial loading/intro
    Intro,
    /// Character creation - name entry
    CharacterCreation { step: u32, name: Option<String> },
    /// Main game view
    MainGame,
    /// In combat
    Combat { combat: Box<CombatState>, last_action: String },
    /// Viewing inventory
    Inventory,
    /// Viewing stats
    Stats,
    /// Help screen
    Help,
    /// Quit confirmation
    ConfirmQuit,
    /// Game is offline (LLM unavailable)
    Offline,
    /// Maintenance mode
    Maintenance,
}

/// Actions returned by the flow for the service layer
#[derive(Debug, Clone)]
pub enum XodiaAction {
    /// Continue - no output needed
    Continue,
    /// Render screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game requires LLM generation
    NeedsLlm { prompt: String, system: String },
    /// Player quit to BBS menu
    Quit,
}

/// Main game flow state machine
pub struct XodiaFlow {
    pub state: GameState,
    pub world: WorldState,
    pub screen: GameScreen,
    pub user_id: i64,
    /// Input buffer for typed commands (public for testing)
    pub input_buffer: String,
    llm_available: bool,
    maintenance_mode: bool,
}

impl XodiaFlow {
    /// Create a new game (no existing save)
    pub fn new(user_id: i64) -> Self {
        Self {
            state: GameState::new("", CharacterClass::Warrior), // Placeholder until creation
            world: WorldState::new(),
            screen: GameScreen::Intro,
            user_id,
            input_buffer: String::new(),
            llm_available: true,
            maintenance_mode: false,
        }
    }

    /// Resume from saved state
    pub fn from_state(user_id: i64, state: GameState, world: WorldState) -> Self {
        Self {
            state,
            world,
            screen: GameScreen::MainGame,
            user_id,
            input_buffer: String::new(),
            llm_available: true,
            maintenance_mode: false,
        }
    }

    /// Set LLM availability
    pub fn set_llm_available(&mut self, available: bool) {
        self.llm_available = available;
        if !available && !matches!(self.screen, GameScreen::CharacterCreation { .. }) {
            self.screen = GameScreen::Offline;
        }
    }

    /// Set maintenance mode
    pub fn set_maintenance_mode(&mut self, enabled: bool) {
        self.maintenance_mode = enabled;
        if enabled {
            self.screen = GameScreen::Maintenance;
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Get world state
    pub fn world_state(&self) -> &WorldState {
        &self.world
    }

    /// Check if in character creation
    pub fn is_new_game(&self) -> bool {
        matches!(self.screen, GameScreen::Intro | GameScreen::CharacterCreation { .. })
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> XodiaAction {
        // Handle backspace
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return XodiaAction::Echo("\x08 \x08".to_string());
            }
            return XodiaAction::Continue;
        }

        // Handle enter
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control characters
        if ch.is_control() {
            return XodiaAction::Continue;
        }

        // Single-key screens
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 200 {
            self.input_buffer.push(ch);
            return XodiaAction::Echo(ch.to_string());
        }

        XodiaAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::Stats
                | GameScreen::Help
                | GameScreen::ConfirmQuit
                | GameScreen::Offline
                | GameScreen::Maintenance
        ) || matches!(self.screen, GameScreen::CharacterCreation { step: 1, .. })
    }

    /// Process buffered input
    fn process_input(&mut self) -> XodiaAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(),
            GameScreen::CharacterCreation { step, name } => {
                self.handle_character_creation(input, *step, name.clone())
            }
            GameScreen::MainGame => self.handle_main_game(input),
            GameScreen::Combat { combat, .. } => {
                let combat_clone = combat.clone();
                self.handle_combat(input, *combat_clone)
            }
            GameScreen::Inventory => self.handle_inventory(input),
            GameScreen::Stats => self.handle_stats(),
            GameScreen::Help => self.handle_help(),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(input),
            GameScreen::Offline => self.handle_offline(),
            GameScreen::Maintenance => self.handle_maintenance(),
        }
    }

    fn handle_intro(&mut self) -> XodiaAction {
        // Check maintenance mode first
        if self.maintenance_mode {
            self.screen = GameScreen::Maintenance;
            return XodiaAction::Render(render::render_maintenance_mode());
        }

        // Check LLM availability
        if !self.llm_available {
            self.screen = GameScreen::Offline;
            return XodiaAction::Render(render::render_offline_mode());
        }

        // Start character creation
        self.screen = GameScreen::CharacterCreation { step: 0, name: None };
        XodiaAction::Render(render::render_character_creation(0, None))
    }

    fn handle_character_creation(
        &mut self,
        input: &str,
        step: u32,
        name: Option<String>,
    ) -> XodiaAction {
        match step {
            0 => {
                // Name entry
                let trimmed = input.trim();
                if trimmed.len() >= 3 && trimmed.len() <= 20 && trimmed.chars().all(|c| c.is_alphanumeric() || c == ' ') {
                    self.screen = GameScreen::CharacterCreation {
                        step: 1,
                        name: Some(trimmed.to_string()),
                    };
                    XodiaAction::Render(render::render_character_creation(1, Some(trimmed)))
                } else {
                    // Invalid name, stay on screen
                    self.state.last_message = Some("Please enter a valid name (3-20 alphanumeric characters).".to_string());
                    XodiaAction::Render(render::render_character_creation(0, None))
                }
            }
            1 => {
                // Class selection
                let class = match input.to_uppercase().as_str() {
                    "1" | "W" => Some(CharacterClass::Warrior),
                    "2" | "M" => Some(CharacterClass::Mage),
                    "3" | "R" => Some(CharacterClass::Rogue),
                    "4" | "C" => Some(CharacterClass::Cleric),
                    _ => None,
                };

                if let Some(class) = class {
                    let char_name = name.unwrap_or_else(|| "Unknown".to_string());
                    self.state = GameState::new(&char_name, class);
                    self.world.mark_room_discovered(&self.state.current_room_id, self.user_id);

                    self.screen = GameScreen::MainGame;
                    let room_desc = self.get_current_room_description();
                    XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
                } else {
                    // Invalid selection
                    XodiaAction::Continue
                }
            }
            _ => XodiaAction::Continue,
        }
    }

    fn handle_main_game(&mut self, input: &str) -> XodiaAction {
        if input.is_empty() {
            // Just pressing enter shows the room again
            let room_desc = self.get_current_room_description();
            return XodiaAction::Render(render::render_main_view(&self.state, &room_desc));
        }

        let parsed = parse_command(input);
        self.state.last_message = None;
        self.state.last_llm_response = None;

        match parsed.command_type {
            CommandType::Go { direction } => self.handle_movement(&direction),
            CommandType::Look { target } => self.handle_look(target.as_deref()),
            CommandType::Examine { target } => self.handle_examine(&target),
            CommandType::Take { target, quantity } => self.handle_take(&target, quantity),
            CommandType::Drop { target, quantity } => self.handle_drop(&target, quantity),
            CommandType::Attack { target } => self.handle_attack(&target),
            CommandType::Talk { target } => self.handle_talk(&target),
            CommandType::Use { target } => self.handle_use(&target),
            CommandType::Inventory => {
                self.screen = GameScreen::Inventory;
                XodiaAction::Render(render::render_inventory(&self.state))
            }
            CommandType::Stats => {
                self.screen = GameScreen::Stats;
                XodiaAction::Render(render::render_stats(&self.state))
            }
            CommandType::Equip { target } => self.handle_equip(&target),
            CommandType::Help => {
                self.screen = GameScreen::Help;
                XodiaAction::Render(render::render_help())
            }
            CommandType::Quit => {
                self.screen = GameScreen::ConfirmQuit;
                XodiaAction::Render(render::render_confirm_quit())
            }
            CommandType::Save => {
                self.state.last_message = Some("Game saved.".to_string());
                XodiaAction::SaveGame
            }
            CommandType::Flee => {
                self.state.last_message = Some("You're not in combat.".to_string());
                let room_desc = self.get_current_room_description();
                XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
            }
            CommandType::Unknown { raw_input } => {
                // Send to LLM for interpretation
                self.generate_llm_response(&raw_input)
            }
            _ => {
                self.state.last_message = Some("I don't understand that command.".to_string());
                let room_desc = self.get_current_room_description();
                XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
            }
        }
    }

    fn handle_movement(&mut self, direction: &str) -> XodiaAction {
        let current_room = self.state.current_room_id.clone();

        if let Some(next_room_id) = self.world.get_exit(&current_room, direction) {
            let next_room_id = next_room_id.to_string();

            // Move to the new room
            self.state.current_room_id = next_room_id.clone();

            // Mark as discovered
            let is_new = self.state.discover_room(&next_room_id);
            self.world.mark_room_discovered(&next_room_id, self.user_id);

            // Update region if changed
            if let Some(room) = self.world.get_room(&next_room_id) {
                self.state.current_region = room.region.clone();
            }

            // Check for hostile NPCs
            let hostiles = self.world.get_hostile_npcs_in_room(&next_room_id);
            if !hostiles.is_empty() {
                // Enter combat with first hostile
                let npc = hostiles[0];
                let combat = CombatState::new(npc);
                self.state.in_combat = true;
                self.state.combat_target = Some(npc.instance_id.clone());
                self.screen = GameScreen::Combat {
                    combat: Box::new(combat.clone()),
                    last_action: format!("A {} attacks!", combat.enemy_name),
                };
                return XodiaAction::Render(render::render_combat(
                    &self.state,
                    &combat,
                    &format!("A {} attacks!", combat.enemy_name),
                ));
            }

            if is_new {
                self.state.last_message = Some("You discover a new area!".to_string());
            }

            let room_desc = self.get_current_room_description();
            XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
        } else {
            self.state.last_message = Some("You can't go that way.".to_string());
            let room_desc = self.get_current_room_description();
            XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
        }
    }

    fn handle_look(&mut self, target: Option<&str>) -> XodiaAction {
        if target.is_none() {
            let room_desc = self.get_current_room_description();
            XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
        } else {
            self.handle_examine(target.unwrap())
        }
    }

    fn handle_examine(&mut self, target: &str) -> XodiaAction {
        let target_lower = target.to_lowercase();

        // Check NPCs in room
        let npcs = self.world.get_npcs_in_room(&self.state.current_room_id);
        for npc in &npcs {
            if npc.name.to_lowercase().contains(&target_lower) {
                if let Some(template) = super::data::get_npc_template(&npc.template_key) {
                    self.state.last_llm_response = Some(format!(
                        "{}\n\n\"{}\"",
                        template.description, template.dialogue_intro
                    ));
                }
                break;
            }
        }

        // Check items in room
        if let Some(room) = self.world.get_room(&self.state.current_room_id) {
            for item in &room.items {
                if item.name.to_lowercase().contains(&target_lower) {
                    if let Some(template) = super::data::get_item_template(&item.template_key) {
                        self.state.last_llm_response = Some(template.description.to_string());
                    }
                    break;
                }
            }
        }

        if self.state.last_llm_response.is_none() {
            self.state.last_message = Some(format!("You don't see '{}' here.", target));
        }

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_take(&mut self, target: &str, quantity: Option<u32>) -> XodiaAction {
        let target_lower = target.to_lowercase();
        let qty = quantity.unwrap_or(1);

        // Find item in room
        let room_id = self.state.current_room_id.clone();
        if let Some(room) = self.world.get_room_mut(&room_id) {
            if let Some(idx) = room.items.iter().position(|i| i.name.to_lowercase().contains(&target_lower)) {
                let item = room.items.remove(idx);

                // Get item template for weight
                let weight = super::data::get_item_template(&item.template_key)
                    .map(|t| t.weight)
                    .unwrap_or(0.5);

                if self.state.add_item(&item.template_key, &item.name, qty.min(item.quantity), weight) {
                    self.state.last_message = Some(format!("You take the {}.", item.name));
                } else {
                    self.state.last_message = Some("You can't carry any more.".to_string());
                    // Put item back
                    room.items.push(item);
                }
            } else {
                self.state.last_message = Some(format!("You don't see '{}' here.", target));
            }
        }

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_drop(&mut self, target: &str, quantity: Option<u32>) -> XodiaAction {
        let target_lower = target.to_lowercase();
        let qty = quantity.unwrap_or(1);

        // Find item in inventory
        if let Some(idx) = self.state.inventory.iter().position(|i| i.name.to_lowercase().contains(&target_lower)) {
            let item = self.state.inventory[idx].clone();

            if self.state.remove_item(&item.key, qty) {
                // Add to room
                let room_item = super::world::RoomItem {
                    instance_id: format!("dropped_{}_{}", item.key, rand::random::<u32>()),
                    template_key: item.key.clone(),
                    name: item.name.clone(),
                    quantity: qty,
                };

                self.world.add_room_item(&self.state.current_room_id, room_item);
                self.state.last_message = Some(format!("You drop the {}.", item.name));
            }
        } else {
            self.state.last_message = Some(format!("You don't have '{}'.", target));
        }

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_attack(&mut self, target: &str) -> XodiaAction {
        let target_lower = target.to_lowercase();

        // Find target in room
        let npcs = self.world.get_npcs_in_room(&self.state.current_room_id);
        for npc in npcs {
            if npc.name.to_lowercase().contains(&target_lower) {
                // Start combat
                let combat = CombatState::new(npc);
                self.state.in_combat = true;
                self.state.combat_target = Some(npc.instance_id.clone());
                self.screen = GameScreen::Combat {
                    combat: Box::new(combat.clone()),
                    last_action: format!("You attack the {}!", combat.enemy_name),
                };
                return XodiaAction::Render(render::render_combat(
                    &self.state,
                    &combat,
                    &format!("You attack the {}!", combat.enemy_name),
                ));
            }
        }

        self.state.last_message = Some(format!("You don't see '{}' here to attack.", target));
        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_talk(&mut self, target: &str) -> XodiaAction {
        let target_lower = target.to_lowercase();

        let npcs = self.world.get_npcs_in_room(&self.state.current_room_id);
        for npc in &npcs {
            if npc.name.to_lowercase().contains(&target_lower) {
                if npc.is_hostile {
                    self.state.last_message = Some(format!("{} doesn't want to talk!", npc.name));
                } else if let Some(template) = super::data::get_npc_template(&npc.template_key) {
                    self.state.last_llm_response = Some(format!(
                        "{}: \"{}\"",
                        npc.name, template.dialogue_intro
                    ));
                }
                break;
            }
        }

        if self.state.last_llm_response.is_none() && self.state.last_message.is_none() {
            self.state.last_message = Some(format!("You don't see '{}' here.", target));
        }

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_use(&mut self, target: &str) -> XodiaAction {
        let target_lower = target.to_lowercase();

        // Find item in inventory
        if let Some(idx) = self.state.inventory.iter().position(|i| i.name.to_lowercase().contains(&target_lower)) {
            let item = self.state.inventory[idx].clone();

            // Check item type
            if let Some(template) = super::data::get_item_template(&item.key) {
                match template.item_type {
                    super::data::ItemType::Potion => {
                        if item.key.contains("health") {
                            self.state.remove_item(&item.key, 1);
                            self.state.heal(template.stat_bonus);
                            self.state.last_message = Some(format!(
                                "You drink the {}. Restored {} HP!",
                                item.name, template.stat_bonus
                            ));
                        } else if item.key.contains("mana") {
                            self.state.remove_item(&item.key, 1);
                            self.state.restore_mana(template.stat_bonus);
                            self.state.last_message = Some(format!(
                                "You drink the {}. Restored {} MP!",
                                item.name, template.stat_bonus
                            ));
                        }
                    }
                    _ => {
                        self.state.last_message = Some(format!("You can't use {} like that.", item.name));
                    }
                }
            }
        } else {
            self.state.last_message = Some(format!("You don't have '{}'.", target));
        }

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_equip(&mut self, target: &str) -> XodiaAction {
        let target_lower = target.to_lowercase();

        // Find item in inventory
        if let Some(item) = self.state.inventory.iter().find(|i| i.name.to_lowercase().contains(&target_lower)) {
            let key = item.key.clone();
            match self.state.equip_item(&key) {
                Ok(msg) => {
                    self.state.last_message = Some(msg);
                }
                Err(msg) => {
                    self.state.last_message = Some(msg);
                }
            }
        } else {
            self.state.last_message = Some(format!("You don't have '{}'.", target));
        }

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_combat(&mut self, input: &str, mut combat: CombatState) -> XodiaAction {
        let action = match input.to_uppercase().as_str() {
            "A" | "ATTACK" => CombatAction::Attack,
            "D" | "DEFEND" => CombatAction::Defend,
            "F" | "FLEE" | "RUN" => CombatAction::Flee,
            "U" | "USE" => CombatAction::UseItem { item: "health_potion".to_string() },
            "C" | "CAST" => CombatAction::Cast { spell: "fireball".to_string() },
            _ => return XodiaAction::Continue,
        };

        let result = resolve_combat_round(&mut self.state, &mut combat, action);

        if result.combat_ended {
            self.state.in_combat = false;
            self.state.combat_target = None;

            if result.player_victory {
                // Grant XP and loot
                if let Some(new_level) = self.state.add_experience(result.xp_gained) {
                    self.state.last_message = Some(format!(
                        "Victory! Gained {} XP. LEVEL UP! You are now level {}!",
                        result.xp_gained, new_level
                    ));
                } else {
                    self.state.last_message = Some(format!(
                        "Victory! Gained {} XP.",
                        result.xp_gained
                    ));
                }

                // Add loot
                for loot in &result.loot {
                    if loot.gold > 0 {
                        self.state.gold += loot.gold;
                    } else {
                        let weight = super::data::get_item_template(&loot.item_key)
                            .map(|t| t.weight)
                            .unwrap_or(0.5);
                        self.state.add_item(&loot.item_key, &loot.item_name, loot.quantity, weight);
                    }
                }

                // Remove defeated NPC from world
                if let Some(ref target) = self.state.combat_target {
                    if let Some(npc) = self.world.get_npc_mut(target) {
                        npc.is_alive = false;
                    }
                }

                self.screen = GameScreen::MainGame;
                let room_desc = self.get_current_room_description();
                return XodiaAction::Render(render::render_main_view(&self.state, &room_desc));
            } else if result.player_defeated {
                // Player died - could implement resurrection here
                self.state.last_message = Some("You have been defeated...".to_string());
                self.screen = GameScreen::MainGame;
                // Respawn with half HP at starting location
                self.state.health = self.state.max_health / 2;
                self.state.current_room_id = "misthollow_square".to_string();
                self.state.current_region = "misthollow".to_string();
                let room_desc = self.get_current_room_description();
                return XodiaAction::Render(render::render_main_view(&self.state, &room_desc));
            } else if result.fled {
                self.screen = GameScreen::MainGame;
                let room_desc = self.get_current_room_description();
                return XodiaAction::Render(render::render_main_view(&self.state, &room_desc));
            }
        }

        // Continue combat
        self.screen = GameScreen::Combat {
            combat: Box::new(combat.clone()),
            last_action: result.narrative_context.clone(),
        };
        XodiaAction::Render(render::render_combat(&self.state, &combat, &result.narrative_context))
    }

    fn handle_inventory(&mut self, input: &str) -> XodiaAction {
        match input.to_uppercase().as_str() {
            "Q" | "X" => {
                self.screen = GameScreen::MainGame;
                let room_desc = self.get_current_room_description();
                XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
            }
            _ => XodiaAction::Continue,
        }
    }

    fn handle_stats(&mut self) -> XodiaAction {
        self.screen = GameScreen::MainGame;
        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_help(&mut self) -> XodiaAction {
        self.screen = GameScreen::MainGame;
        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    fn handle_confirm_quit(&mut self, input: &str) -> XodiaAction {
        match input.to_uppercase().as_str() {
            "Y" | "YES" => XodiaAction::Quit,
            _ => {
                self.screen = GameScreen::MainGame;
                let room_desc = self.get_current_room_description();
                XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
            }
        }
    }

    fn handle_offline(&mut self) -> XodiaAction {
        XodiaAction::Quit
    }

    fn handle_maintenance(&mut self) -> XodiaAction {
        XodiaAction::Quit
    }

    /// Generate LLM response for unknown command
    fn generate_llm_response(&mut self, input: &str) -> XodiaAction {
        // For now, just indicate we don't understand
        // In the full implementation, this would call the LLM
        self.state.last_message = Some(format!(
            "The DM ponders your request: \"{}\"... (LLM integration pending)",
            input
        ));

        let room_desc = self.get_current_room_description();
        XodiaAction::Render(render::render_main_view(&self.state, &room_desc))
    }

    /// Get current room description
    fn get_current_room_description(&self) -> String {
        self.world.describe_room(&self.state.current_room_id)
            .unwrap_or_else(|| "You are in a mysterious void.".to_string())
    }

    /// Apply LLM response (called by service after LLM generation)
    pub fn apply_llm_response(&mut self, response: &str) {
        self.state.last_llm_response = Some(response.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = XodiaFlow::new(1);
        assert!(matches!(flow.screen, GameScreen::Intro));
        assert!(flow.is_new_game());
    }

    #[test]
    fn test_intro_to_character_creation() {
        let mut flow = XodiaFlow::new(1);
        let action = flow.handle_char('\r');

        assert!(matches!(flow.screen, GameScreen::CharacterCreation { step: 0, .. }));
        assert!(matches!(action, XodiaAction::Render(_)));
    }

    #[test]
    fn test_character_name_validation() {
        let mut flow = XodiaFlow::new(1);
        flow.screen = GameScreen::CharacterCreation { step: 0, name: None };

        // Too short
        flow.input_buffer = "ab".to_string();
        let _ = flow.process_input();
        assert!(matches!(flow.screen, GameScreen::CharacterCreation { step: 0, .. }));

        // Valid name
        flow.input_buffer = "TestHero".to_string();
        let _ = flow.process_input();
        assert!(matches!(flow.screen, GameScreen::CharacterCreation { step: 1, .. }));
    }

    #[test]
    fn test_class_selection() {
        let mut flow = XodiaFlow::new(1);
        flow.screen = GameScreen::CharacterCreation {
            step: 1,
            name: Some("TestHero".to_string()),
        };

        flow.input_buffer = "1".to_string();
        let _ = flow.process_input();

        assert!(matches!(flow.screen, GameScreen::MainGame));
        assert_eq!(flow.state.character_name, "TestHero");
        assert_eq!(flow.state.class, CharacterClass::Warrior);
    }

    #[test]
    fn test_movement() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;
        flow.world = WorldState::new();

        // Move north from starting square
        flow.input_buffer = "north".to_string();
        let _ = flow.process_input();

        assert_eq!(flow.state.current_room_id, "misthollow_elder");
    }

    #[test]
    fn test_invalid_movement() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;
        flow.world = WorldState::new();

        let original_room = flow.state.current_room_id.clone();

        // Try invalid direction
        flow.input_buffer = "northeast".to_string();
        let _ = flow.process_input();

        assert_eq!(flow.state.current_room_id, original_room);
        assert!(flow.state.last_message.as_ref().unwrap().contains("can't go"));
    }

    #[test]
    fn test_look_command() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;
        flow.world = WorldState::new();

        flow.input_buffer = "look".to_string();
        let action = flow.process_input();

        assert!(matches!(action, XodiaAction::Render(_)));
    }

    #[test]
    fn test_inventory_screen() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;

        flow.input_buffer = "inventory".to_string();
        let _ = flow.process_input();

        assert!(matches!(flow.screen, GameScreen::Inventory));

        // Exit inventory
        flow.input_buffer = "q".to_string();
        let _ = flow.process_input();

        assert!(matches!(flow.screen, GameScreen::MainGame));
    }

    #[test]
    fn test_help_screen() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;

        flow.input_buffer = "help".to_string();
        let _ = flow.process_input();

        assert!(matches!(flow.screen, GameScreen::Help));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;

        flow.input_buffer = "quit".to_string();
        let _ = flow.process_input();

        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        // Cancel quit
        flow.input_buffer = "n".to_string();
        let _ = flow.process_input();

        assert!(matches!(flow.screen, GameScreen::MainGame));
    }

    #[test]
    fn test_llm_availability() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;

        flow.set_llm_available(false);
        assert!(matches!(flow.screen, GameScreen::Offline));
    }

    #[test]
    fn test_maintenance_mode() {
        let mut flow = XodiaFlow::new(1);
        flow.state = GameState::new("TestHero", CharacterClass::Warrior);
        flow.screen = GameScreen::MainGame;

        flow.set_maintenance_mode(true);
        assert!(matches!(flow.screen, GameScreen::Maintenance));
    }

    #[test]
    fn test_backspace_handling() {
        let mut flow = XodiaFlow::new(1);
        flow.screen = GameScreen::CharacterCreation { step: 0, name: None };

        // Type and then backspace
        flow.handle_char('H');
        flow.handle_char('e');
        flow.handle_char('l');
        let action = flow.handle_char('\x7f'); // Backspace

        assert!(matches!(action, XodiaAction::Echo(_)));
        assert_eq!(flow.input_buffer, "He");
    }
}
