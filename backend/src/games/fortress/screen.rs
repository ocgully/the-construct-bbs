//! Screen state machine for Fortress
//!
//! Manages screens and input handling.

use super::state::GameState;
use super::jobs::JobType;
use super::tick::{process_tick, process_catchup};

/// Which screen the player is viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Welcome/intro screen
    Intro,
    /// Name the fortress
    Naming,
    /// Main fortress view
    FortressView,
    /// Dwarf list
    DwarfList,
    /// Individual dwarf details
    DwarfDetail { dwarf_id: u32 },
    /// Workshop list and crafting
    Workshops,
    /// Workshop detail with recipes
    WorkshopDetail { workshop_id: u32 },
    /// Room designation
    RoomDesign,
    /// Job queue view
    JobQueue,
    /// Work orders
    WorkOrders,
    /// Military screen
    Military,
    /// Resource stockpile view
    Stockpiles,
    /// Designate digging
    Designate,
    /// Build menu
    BuildMenu,
    /// Combat/invasion screen
    Combat,
    /// Statistics
    Statistics,
    /// Help screen
    Help,
    /// Confirm quit
    ConfirmQuit,
    /// Game over (fortress destroyed)
    GameOver,
}

/// Actions returned by input handler
#[derive(Debug, Clone)]
pub enum FortressAction {
    /// No action needed
    Continue,
    /// Render screen
    Render(String),
    /// Echo input
    Echo(String),
    /// Save game
    SaveGame,
    /// Game over
    GameOver { final_score: u64 },
    /// Quit to main menu
    Quit,
}

/// Main flow controller
pub struct FortressFlow {
    pub state: GameState,
    pub screen: GameScreen,
    input_buffer: String,
    /// Selected index for lists
    selected_index: usize,
    /// Name input for new fortress
    name_input: String,
    /// Cursor position for designations
    cursor: (u32, u32),
}

impl FortressFlow {
    /// Create new game
    pub fn new() -> Self {
        Self {
            state: GameState::new("New Fortress".to_string(), 0),
            screen: GameScreen::Intro,
            input_buffer: String::new(),
            selected_index: 0,
            name_input: String::new(),
            cursor: (0, 0),
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState) -> Self {
        Self {
            state,
            screen: GameScreen::Intro,
            input_buffer: String::new(),
            selected_index: 0,
            name_input: String::new(),
            cursor: (0, 0),
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

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> FortressAction {
        // Backspace
        if ch == '\x7f' || ch == '\x08' {
            if self.screen == GameScreen::Naming {
                if self.name_input.pop().is_some() {
                    return FortressAction::Echo("\x08 \x08".to_string());
                }
            } else if self.input_buffer.pop().is_some() {
                return FortressAction::Echo("\x08 \x08".to_string());
            }
            return FortressAction::Continue;
        }

        // Enter
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Control chars
        if ch.is_control() {
            return FortressAction::Continue;
        }

        // For naming screen, buffer input
        if self.screen == GameScreen::Naming {
            if self.name_input.len() < 20 {
                self.name_input.push(ch);
                return FortressAction::Echo(ch.to_string());
            }
            return FortressAction::Continue;
        }

        // Single-key input for most screens
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return FortressAction::Echo(ch.to_string());
        }

        FortressAction::Continue
    }

    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::FortressView
                | GameScreen::DwarfList
                | GameScreen::DwarfDetail { .. }
                | GameScreen::Workshops
                | GameScreen::WorkshopDetail { .. }
                | GameScreen::RoomDesign
                | GameScreen::JobQueue
                | GameScreen::WorkOrders
                | GameScreen::Military
                | GameScreen::Stockpiles
                | GameScreen::Designate
                | GameScreen::BuildMenu
                | GameScreen::Combat
                | GameScreen::Statistics
                | GameScreen::Help
                | GameScreen::ConfirmQuit
                | GameScreen::GameOver
        )
    }

    fn process_input(&mut self) -> FortressAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::Naming => self.handle_naming(),
            GameScreen::FortressView => self.handle_fortress_view(&input),
            GameScreen::DwarfList => self.handle_dwarf_list(&input),
            GameScreen::DwarfDetail { dwarf_id } => self.handle_dwarf_detail(&input, *dwarf_id),
            GameScreen::Workshops => self.handle_workshops(&input),
            GameScreen::WorkshopDetail { workshop_id } => self.handle_workshop_detail(&input, *workshop_id),
            GameScreen::RoomDesign => self.handle_room_design(&input),
            GameScreen::JobQueue => self.handle_job_queue(&input),
            GameScreen::WorkOrders => self.handle_work_orders(&input),
            GameScreen::Military => self.handle_military(&input),
            GameScreen::Stockpiles => self.handle_stockpiles(&input),
            GameScreen::Designate => self.handle_designate(&input),
            GameScreen::BuildMenu => self.handle_build_menu(&input),
            GameScreen::Combat => self.handle_combat(&input),
            GameScreen::Statistics => self.handle_statistics(&input),
            GameScreen::Help => self.handle_help(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> FortressAction {
        // Check if this is a new game or resume
        if self.state.tick == 0 {
            self.screen = GameScreen::Naming;
        } else {
            // Process catchup ticks
            let elapsed = 10; // Would be calculated from real time
            if elapsed > 0 {
                let result = process_catchup(&mut self.state, elapsed.min(100));
                if !result.resources_gathered.is_empty() || result.migrants_arrived > 0 {
                    self.state.notify(format!("While you were away: {} ticks processed", result.tick + 1));
                }
            }
            self.screen = GameScreen::FortressView;
        }
        FortressAction::SaveGame
    }

    fn handle_naming(&mut self) -> FortressAction {
        let name = std::mem::take(&mut self.name_input);

        if name.len() >= 3 {
            // Create new game with name
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(12345);

            self.state = GameState::new(name, seed);
            self.screen = GameScreen::FortressView;
            FortressAction::SaveGame
        } else {
            // Name too short
            self.state.last_message = Some("Name must be at least 3 characters.".to_string());
            FortressAction::SaveGame
        }
    }

    fn handle_fortress_view(&mut self, input: &str) -> FortressAction {
        self.state.last_message = None;

        // Process a tick while viewing
        process_tick(&mut self.state);

        match input {
            // Navigation
            "W" | "8" => {
                if self.state.view_y > 0 {
                    self.state.view_y -= 1;
                }
            }
            "S" | "2" => {
                if self.state.view_y < self.state.terrain.height - 1 {
                    self.state.view_y += 1;
                }
            }
            "A" | "4" => {
                if self.state.view_x > 0 {
                    self.state.view_x -= 1;
                }
            }
            "D" | "6" => {
                if self.state.view_x < self.state.terrain.width - 1 {
                    self.state.view_x += 1;
                }
            }
            "<" | "," => {
                // Go up z-level
                if self.state.view_z > 0 {
                    self.state.view_z -= 1;
                }
            }
            ">" | "." => {
                // Go down z-level
                if self.state.view_z < self.state.terrain.depth - 1 {
                    self.state.view_z += 1;
                }
            }

            // Menus
            "U" => {
                self.screen = GameScreen::DwarfList;
            }
            "B" => {
                self.screen = GameScreen::BuildMenu;
            }
            "P" => {
                self.screen = GameScreen::Workshops;
            }
            "R" => {
                self.screen = GameScreen::RoomDesign;
            }
            "J" => {
                self.screen = GameScreen::JobQueue;
            }
            "O" => {
                self.screen = GameScreen::WorkOrders;
            }
            "M" => {
                self.screen = GameScreen::Military;
            }
            "I" => {
                self.screen = GameScreen::Stockpiles;
            }
            "Z" => {
                self.screen = GameScreen::Designate;
            }
            "C" => {
                if self.state.under_siege() {
                    self.screen = GameScreen::Combat;
                }
            }
            "T" => {
                self.screen = GameScreen::Statistics;
            }
            "?" | "H" => {
                self.screen = GameScreen::Help;
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
            }

            // Quick actions
            "F" => {
                // Farm: set farm at cursor
                let (x, y) = (self.state.view_x, self.state.view_y);
                let z = self.state.view_z;
                if self.state.terrain.set_farm(x, y, z) {
                    self.state.last_message = Some("Farm plot created.".to_string());
                }
            }
            "K" => {
                // Stockpile at cursor
                let (x, y) = (self.state.view_x, self.state.view_y);
                let z = self.state.view_z;
                if self.state.terrain.set_stockpile(x, y, z) {
                    self.state.last_message = Some("Stockpile designated.".to_string());
                }
            }

            _ => {}
        }

        FortressAction::SaveGame
    }

    fn handle_dwarf_list(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            _ => {
                // Select dwarf by number
                if let Ok(idx) = input.parse::<usize>() {
                    if idx > 0 && idx <= self.state.dwarves.len() {
                        let dwarf_id = self.state.dwarves[idx - 1].id;
                        self.screen = GameScreen::DwarfDetail { dwarf_id };
                    }
                }
            }
        }
        FortressAction::SaveGame
    }

    fn handle_dwarf_detail(&mut self, input: &str, dwarf_id: u32) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::DwarfList;
            }
            "E" => {
                // Equip weapon if available
                let has_weapon = self.state.resources.weapon > 0;
                let needs_weapon = self.state.get_dwarf(dwarf_id)
                    .map(|d| d.equipped_weapon.is_none())
                    .unwrap_or(false);

                if has_weapon && needs_weapon {
                    self.state.resources.weapon -= 1;
                    if let Some(dwarf) = self.state.get_dwarf_mut(dwarf_id) {
                        dwarf.equipped_weapon = Some("iron_sword".to_string());
                    }
                    self.state.last_message = Some("Weapon equipped.".to_string());
                }
            }
            "A" => {
                // Equip armor
                let has_armor = self.state.resources.armor > 0;
                let needs_armor = self.state.get_dwarf(dwarf_id)
                    .map(|d| d.equipped_armor.is_none())
                    .unwrap_or(false);

                if has_armor && needs_armor {
                    self.state.resources.armor -= 1;
                    if let Some(dwarf) = self.state.get_dwarf_mut(dwarf_id) {
                        dwarf.equipped_armor = Some("iron_mail".to_string());
                    }
                    self.state.last_message = Some("Armor equipped.".to_string());
                }
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_workshops(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            "N" => {
                // New workshop (would need position selection)
                self.screen = GameScreen::BuildMenu;
            }
            _ => {
                if let Ok(idx) = input.parse::<usize>() {
                    if idx > 0 && idx <= self.state.workshops.len() {
                        let workshop_id = self.state.workshops[idx - 1].id;
                        self.screen = GameScreen::WorkshopDetail { workshop_id };
                    }
                }
            }
        }
        FortressAction::SaveGame
    }

    fn handle_workshop_detail(&mut self, input: &str, workshop_id: u32) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::Workshops;
            }
            _ => {
                // Add work order by recipe number
                if let Ok(idx) = input.parse::<usize>() {
                    if let Some(workshop) = self.state.get_workshop(workshop_id) {
                        let recipes = super::data::get_workshop_recipes(&workshop.workshop_type);
                        if idx > 0 && idx <= recipes.len() {
                            let recipe = recipes[idx - 1];
                            self.state.job_queue.add_work_order(
                                recipe.name.to_string(),
                                recipe.key.to_string(),
                                workshop_id,
                                5, // Default quantity
                                false,
                                self.state.tick,
                            );
                            self.state.last_message = Some(format!("Work order added: {}", recipe.name));
                        }
                    }
                }
            }
        }
        FortressAction::SaveGame
    }

    fn handle_room_design(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_job_queue(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            "C" => {
                // Cancel selected job
                let pending = self.state.job_queue.pending_jobs();
                if !pending.is_empty() && self.selected_index < pending.len() {
                    let job_id = pending[self.selected_index].id;
                    self.state.job_queue.cancel_job(job_id);
                    self.state.last_message = Some("Job cancelled.".to_string());
                }
            }
            "J" | "2" => {
                // Move selection down
                let count = self.state.job_queue.pending_jobs().len();
                if count > 0 {
                    self.selected_index = (self.selected_index + 1) % count;
                }
            }
            "K" | "8" => {
                // Move selection up
                let count = self.state.job_queue.pending_jobs().len();
                if count > 0 && self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_work_orders(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            "P" => {
                // Pause/unpause selected order
                let orders = self.state.job_queue.active_work_orders();
                if !orders.is_empty() && self.selected_index < orders.len() {
                    let order_id = orders[self.selected_index].id;
                    self.state.job_queue.toggle_work_order_pause(order_id);
                }
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_military(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            "D" => {
                // Draft dwarf
                for dwarf in &mut self.state.dwarves {
                    if dwarf.skills.combat >= 2 && dwarf.status == super::dwarves::DwarfStatus::Idle {
                        dwarf.status = super::dwarves::DwarfStatus::Fighting;
                        self.state.last_message = Some(format!("{} drafted.", dwarf.name));
                        break;
                    }
                }
            }
            "S" => {
                // Stand down
                for dwarf in &mut self.state.dwarves {
                    if dwarf.status == super::dwarves::DwarfStatus::Fighting {
                        dwarf.status = super::dwarves::DwarfStatus::Idle;
                    }
                }
                self.state.last_message = Some("Military stood down.".to_string());
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_stockpiles(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_designate(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            "D" | " " => {
                // Designate tile for digging
                let (x, y) = (self.state.view_x, self.state.view_y);
                let z = self.state.view_z;

                if self.state.terrain.designate(x, y, z) {
                    // Create mining job
                    self.state.job_queue.add_job(
                        JobType::Mine { x, y, z },
                        self.state.tick,
                    );
                    self.state.last_message = Some(format!("Designated ({},{},{}) for digging.", x, y, z));
                }
            }
            // Movement same as fortress view
            "W" | "8" => { if self.state.view_y > 0 { self.state.view_y -= 1; } }
            "S" | "2" => { if self.state.view_y < self.state.terrain.height - 1 { self.state.view_y += 1; } }
            "A" | "4" => { if self.state.view_x > 0 { self.state.view_x -= 1; } }
            "6" | "L" => { if self.state.view_x < self.state.terrain.width - 1 { self.state.view_x += 1; } }
            "<" | "," => { if self.state.view_z > 0 { self.state.view_z -= 1; } }
            ">" | "." => { if self.state.view_z < self.state.terrain.depth - 1 { self.state.view_z += 1; } }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_build_menu(&mut self, input: &str) -> FortressAction {
        use super::data::WORKSHOPS;

        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            _ => {
                if let Ok(idx) = input.parse::<usize>() {
                    if idx > 0 && idx <= WORKSHOPS.len() {
                        let workshop_type = WORKSHOPS[idx - 1].key;
                        let (x, y) = (self.state.view_x, self.state.view_y);
                        let z = self.state.view_z;

                        if let Some(_id) = self.state.build_workshop(workshop_type, x, y, z) {
                            self.state.last_message = Some(format!("{} built!", WORKSHOPS[idx - 1].name));
                        } else {
                            self.state.last_message = Some("Cannot build here or not enough resources.".to_string());
                        }
                    }
                }
            }
        }
        FortressAction::SaveGame
    }

    fn handle_combat(&mut self, input: &str) -> FortressAction {
        match input {
            "Q" | "X" => {
                self.screen = GameScreen::FortressView;
            }
            _ => {}
        }
        FortressAction::SaveGame
    }

    fn handle_statistics(&mut self, input: &str) -> FortressAction {
        match input {
            _ => {
                self.screen = GameScreen::FortressView;
            }
        }
        FortressAction::SaveGame
    }

    fn handle_help(&mut self, input: &str) -> FortressAction {
        match input {
            _ => {
                self.screen = GameScreen::FortressView;
            }
        }
        FortressAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> FortressAction {
        match input {
            "Y" => FortressAction::Quit,
            _ => {
                self.screen = GameScreen::FortressView;
                FortressAction::SaveGame
            }
        }
    }

    fn handle_game_over(&mut self, _input: &str) -> FortressAction {
        FortressAction::GameOver {
            final_score: self.state.fortress_value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = FortressFlow::new();
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_intro_to_naming() {
        let mut flow = FortressFlow::new();
        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::Naming));
    }

    #[test]
    fn test_naming_flow() {
        let mut flow = FortressFlow::new();
        flow.handle_char('\r'); // Skip intro

        // Type name
        for c in "TestFort".chars() {
            flow.handle_char(c);
        }
        flow.handle_char('\r');

        assert!(matches!(flow.screen, GameScreen::FortressView));
        assert_eq!(flow.state.fortress_name, "TestFort");
    }

    #[test]
    fn test_navigation() {
        let mut flow = FortressFlow::new();
        flow.screen = GameScreen::FortressView;
        flow.state = GameState::new("Test".to_string(), 42);

        let initial_y = flow.state.view_y;
        flow.handle_char('S');
        assert_eq!(flow.state.view_y, initial_y + 1);
    }

    #[test]
    fn test_screen_transitions() {
        let mut flow = FortressFlow::new();
        flow.screen = GameScreen::FortressView;
        flow.state = GameState::new("Test".to_string(), 42);

        flow.handle_char('U');
        assert!(matches!(flow.screen, GameScreen::DwarfList));

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::FortressView));
    }
}
