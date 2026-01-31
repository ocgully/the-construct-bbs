//! Screen flow and input handling for Master of Cygnus

use super::state::{GameState, GameStatus, TurnOrders, ColonyOrders, FleetOrders};
use super::tech::TechField;

/// Which screen the player is viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Game intro/title
    Intro,
    /// Main game view - galaxy map
    GalaxyMap,
    /// Viewing a specific star system
    StarSystem { star_id: u32 },
    /// Colony management
    ColonyManagement { star_id: u32 },
    /// Fleet management
    FleetManagement { fleet_id: u32 },
    /// Research allocation
    Research,
    /// Ship designer
    ShipDesigner,
    /// Turn summary
    TurnSummary,
    /// Victory/defeat screen
    GameOver,
    /// Game lobby (waiting for players)
    Lobby,
    /// Creating a new game
    NewGame { step: u32 },
    /// Joining an existing game
    JoinGame,
    /// Settings menu
    Settings,
    /// Confirm quit
    ConfirmQuit,
}

/// Actions returned by input handler
#[derive(Debug, Clone)]
pub enum MocAction {
    /// Continue - no output needed
    Continue,
    /// Render the current screen
    Render(String),
    /// Echo characters back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Submit turn orders
    SubmitTurn,
    /// Game is over
    GameOver { winner_name: Option<String> },
    /// Return to BBS menu
    Quit,
}

/// Master of Cygnus game flow state machine
pub struct MocFlow {
    pub game_state: Option<GameState>,
    pub screen: GameScreen,
    pub current_empire_id: Option<u32>,
    /// Orders being built for current turn
    pub draft_orders: Option<TurnOrders>,
    /// Input buffer for text entry
    input_buffer: String,
    /// Current user ID
    pub user_id: i64,
    /// Available games for joining
    pub available_games: Vec<(u64, String, u32, u32)>, // (id, name, players, max)
}

impl MocFlow {
    /// Create a new game flow
    pub fn new(user_id: i64) -> Self {
        MocFlow {
            game_state: None,
            screen: GameScreen::Intro,
            current_empire_id: None,
            draft_orders: None,
            input_buffer: String::new(),
            user_id,
            available_games: Vec::new(),
        }
    }

    /// Load an existing game
    pub fn load_game(&mut self, game: GameState) {
        self.current_empire_id = game.get_empire_by_user(self.user_id).map(|e| e.id);
        self.game_state = Some(game);

        // Initialize draft orders for current empire
        if let Some(empire_id) = self.current_empire_id {
            self.init_draft_orders(empire_id);
        }

        if self.game_state.as_ref().map(|g| g.status) == Some(GameStatus::WaitingForPlayers) {
            self.screen = GameScreen::Lobby;
        } else {
            self.screen = GameScreen::GalaxyMap;
        }
    }

    /// Initialize draft orders for an empire
    fn init_draft_orders(&mut self, empire_id: u32) {
        if let Some(game) = &self.game_state {
            if let Some(empire) = game.get_empire(empire_id) {
                self.draft_orders = Some(TurnOrders {
                    empire_id,
                    colony_orders: empire.colonies.iter().map(|c| ColonyOrders {
                        star_id: c.star_id,
                        build_queue: c.production_queue.clone(),
                        population_transfer: None,
                    }).collect(),
                    fleet_orders: empire.fleets.iter().map(|f| FleetOrders {
                        fleet_id: f.id,
                        destination: f.destination_star_id,
                        colonize: false,
                    }).collect(),
                    research_allocation: empire.research.allocation.clone(),
                    submitted: false,
                    ai_generated: false,
                });
            }
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get game state reference
    pub fn game_state(&self) -> Option<&GameState> {
        self.game_state.as_ref()
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> MocAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return MocAction::Echo("\x08 \x08".to_string());
            }
            return MocAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return MocAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 30 {
            self.input_buffer.push(ch);
            return MocAction::Echo(ch.to_string());
        }

        MocAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::GalaxyMap
                | GameScreen::StarSystem { .. }
                | GameScreen::ColonyManagement { .. }
                | GameScreen::FleetManagement { .. }
                | GameScreen::Research
                | GameScreen::TurnSummary
                | GameScreen::GameOver
                | GameScreen::Lobby
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> MocAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::GalaxyMap => self.handle_galaxy_map(&input),
            GameScreen::StarSystem { star_id } => self.handle_star_system(&input, *star_id),
            GameScreen::ColonyManagement { star_id } => self.handle_colony(&input, *star_id),
            GameScreen::FleetManagement { fleet_id } => self.handle_fleet(&input, *fleet_id),
            GameScreen::Research => self.handle_research(&input),
            GameScreen::ShipDesigner => self.handle_ship_designer(&input),
            GameScreen::TurnSummary => self.handle_turn_summary(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::Lobby => self.handle_lobby(&input),
            GameScreen::NewGame { step } => self.handle_new_game(&input, *step),
            GameScreen::JoinGame => self.handle_join_game(&input),
            GameScreen::Settings => self.handle_settings(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> MocAction {
        // Any key advances to lobby or game
        if self.game_state.is_some() {
            if self.game_state.as_ref().map(|g| g.status) == Some(GameStatus::WaitingForPlayers) {
                self.screen = GameScreen::Lobby;
            } else {
                self.screen = GameScreen::GalaxyMap;
            }
        } else {
            self.screen = GameScreen::Lobby;
        }
        MocAction::SaveGame
    }

    fn handle_galaxy_map(&mut self, input: &str) -> MocAction {
        match input {
            "S" | "1" => {
                // View star list
                if let Some(game) = &self.game_state {
                    if let Some(empire_id) = self.current_empire_id {
                        if let Some(empire) = game.get_empire(empire_id) {
                            if let Some(colony) = empire.colonies.first() {
                                self.screen = GameScreen::StarSystem { star_id: colony.star_id };
                                return MocAction::SaveGame;
                            }
                        }
                    }
                }
                MocAction::Continue
            }
            "C" | "2" => {
                // Colony management
                if let Some(game) = &self.game_state {
                    if let Some(empire_id) = self.current_empire_id {
                        if let Some(empire) = game.get_empire(empire_id) {
                            if let Some(colony) = empire.colonies.first() {
                                self.screen = GameScreen::ColonyManagement { star_id: colony.star_id };
                                return MocAction::SaveGame;
                            }
                        }
                    }
                }
                MocAction::Continue
            }
            "F" | "3" => {
                // Fleet management
                if let Some(game) = &self.game_state {
                    if let Some(empire_id) = self.current_empire_id {
                        if let Some(empire) = game.get_empire(empire_id) {
                            if let Some(fleet) = empire.fleets.first() {
                                self.screen = GameScreen::FleetManagement { fleet_id: fleet.id };
                                return MocAction::SaveGame;
                            }
                        }
                    }
                }
                MocAction::Continue
            }
            "R" | "4" => {
                self.screen = GameScreen::Research;
                MocAction::SaveGame
            }
            "D" | "5" => {
                self.screen = GameScreen::ShipDesigner;
                MocAction::SaveGame
            }
            "T" | "E" => {
                // End turn / submit
                self.screen = GameScreen::TurnSummary;
                MocAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                MocAction::SaveGame
            }
            _ => {
                // Try to parse as star number
                if let Ok(star_num) = input.parse::<u32>() {
                    self.screen = GameScreen::StarSystem { star_id: star_num };
                    return MocAction::SaveGame;
                }
                MocAction::Continue
            }
        }
    }

    fn handle_star_system(&mut self, input: &str, star_id: u32) -> MocAction {
        match input {
            "C" => {
                // Colonize if we have a colony ship here
                // TODO: Implement colonization
                MocAction::Continue
            }
            "B" => {
                // Build (if we own this star)
                self.screen = GameScreen::ColonyManagement { star_id };
                MocAction::SaveGame
            }
            "Q" | "M" => {
                self.screen = GameScreen::GalaxyMap;
                MocAction::SaveGame
            }
            _ => MocAction::Continue,
        }
    }

    fn handle_colony(&mut self, input: &str, star_id: u32) -> MocAction {
        match input {
            "1" => self.queue_building(star_id, "Factory"),
            "2" => self.queue_building(star_id, "ResearchLab"),
            "3" => self.queue_building(star_id, "Farm"),
            "4" => self.queue_building(star_id, "Shipyard"),
            "5" => self.queue_building(star_id, "Scout"),
            "6" => self.queue_building(star_id, "Fighter"),
            "7" => self.queue_building(star_id, "Colony Ship"),
            "Q" | "M" => {
                self.screen = GameScreen::GalaxyMap;
                MocAction::SaveGame
            }
            _ => MocAction::Continue,
        }
    }

    fn queue_building(&mut self, star_id: u32, item: &str) -> MocAction {
        if let Some(orders) = &mut self.draft_orders {
            if let Some(col_orders) = orders.colony_orders.iter_mut().find(|c| c.star_id == star_id) {
                col_orders.build_queue.push(item.to_string());
                return MocAction::SaveGame;
            }
        }
        MocAction::Continue
    }

    fn handle_fleet(&mut self, input: &str, fleet_id: u32) -> MocAction {
        match input {
            "Q" | "M" => {
                self.screen = GameScreen::GalaxyMap;
                MocAction::SaveGame
            }
            _ => {
                // Try to parse as destination star
                if let Ok(dest) = input.parse::<u32>() {
                    if let Some(orders) = &mut self.draft_orders {
                        if let Some(fleet_orders) = orders.fleet_orders.iter_mut().find(|f| f.fleet_id == fleet_id) {
                            fleet_orders.destination = Some(dest);
                            return MocAction::SaveGame;
                        }
                    }
                }
                MocAction::Continue
            }
        }
    }

    fn handle_research(&mut self, input: &str) -> MocAction {
        if let Some(orders) = &mut self.draft_orders {
            match input {
                "1" => { orders.research_allocation.insert(TechField::Propulsion, 50); }
                "2" => { orders.research_allocation.insert(TechField::Weapons, 50); }
                "3" => { orders.research_allocation.insert(TechField::Shields, 50); }
                "4" => { orders.research_allocation.insert(TechField::Planetology, 50); }
                "5" => { orders.research_allocation.insert(TechField::Construction, 50); }
                "6" => { orders.research_allocation.insert(TechField::Computers, 50); }
                "B" => {
                    // Balanced allocation
                    for field in TechField::all() {
                        orders.research_allocation.insert(field, 16);
                    }
                }
                "Q" | "M" => {
                    self.screen = GameScreen::GalaxyMap;
                    return MocAction::SaveGame;
                }
                _ => {}
            }
        }
        MocAction::SaveGame
    }

    fn handle_ship_designer(&mut self, input: &str) -> MocAction {
        match input {
            "Q" | "M" => {
                self.screen = GameScreen::GalaxyMap;
                MocAction::SaveGame
            }
            _ => MocAction::Continue,
        }
    }

    fn handle_turn_summary(&mut self, input: &str) -> MocAction {
        match input {
            "S" | "Y" => {
                // Submit turn
                if let Some(orders) = self.draft_orders.take() {
                    if let Some(game) = &mut self.game_state {
                        game.submit_orders(orders.empire_id, orders);

                        // If all orders submitted, process turn
                        if game.all_orders_submitted() {
                            game.process_turn();

                            // Check for game over
                            if game.status == GameStatus::Completed {
                                self.screen = GameScreen::GameOver;
                                return MocAction::SaveGame;
                            }
                        }
                    }
                    // Re-init orders for next turn
                    if let Some(empire_id) = self.current_empire_id {
                        self.init_draft_orders(empire_id);
                    }
                }
                self.screen = GameScreen::GalaxyMap;
                MocAction::SubmitTurn
            }
            "N" | "Q" | "M" => {
                self.screen = GameScreen::GalaxyMap;
                MocAction::SaveGame
            }
            _ => MocAction::Continue,
        }
    }

    fn handle_game_over(&mut self, _input: &str) -> MocAction {
        let winner_name = self.game_state.as_ref()
            .and_then(|g| g.winner_empire_id)
            .and_then(|id| self.game_state.as_ref()?.get_empire(id))
            .map(|e| e.name.clone());

        MocAction::GameOver { winner_name }
    }

    fn handle_lobby(&mut self, input: &str) -> MocAction {
        match input {
            "N" | "1" => {
                self.screen = GameScreen::NewGame { step: 0 };
                MocAction::SaveGame
            }
            "J" | "2" => {
                self.screen = GameScreen::JoinGame;
                MocAction::SaveGame
            }
            "S" | "3" => {
                // Start game if we have enough players
                if let Some(game) = &mut self.game_state {
                    if game.empires.len() >= 2 {
                        game.start_game();
                        self.screen = GameScreen::GalaxyMap;
                    }
                }
                MocAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                MocAction::SaveGame
            }
            _ => MocAction::Continue,
        }
    }

    fn handle_new_game(&mut self, input: &str, step: u32) -> MocAction {
        match step {
            0 => {
                // Game name
                if !input.is_empty() {
                    let game = GameState::new(
                        rand::random(),
                        input.to_string(),
                        Default::default(),
                    );
                    self.game_state = Some(game);
                    self.screen = GameScreen::NewGame { step: 1 };
                }
                MocAction::SaveGame
            }
            1 => {
                // Empire name
                if !input.is_empty() {
                    if let Some(game) = &mut self.game_state {
                        let empire_id = game.add_player(
                            self.user_id,
                            input.to_string(),
                            "terran".to_string(),
                            "LightCyan".to_string(),
                        ).ok();
                        self.current_empire_id = empire_id;
                        if let Some(id) = empire_id {
                            self.init_draft_orders(id);
                        }
                    }
                    self.screen = GameScreen::Lobby;
                }
                MocAction::SaveGame
            }
            _ => {
                self.screen = GameScreen::Lobby;
                MocAction::SaveGame
            }
        }
    }

    fn handle_join_game(&mut self, input: &str) -> MocAction {
        if input == "Q" || input == "M" {
            self.screen = GameScreen::Lobby;
            return MocAction::SaveGame;
        }

        // Try to parse as game number
        if let Ok(game_idx) = input.parse::<usize>() {
            if game_idx > 0 && game_idx <= self.available_games.len() {
                // TODO: Load the selected game
            }
        }

        MocAction::Continue
    }

    fn handle_settings(&mut self, input: &str) -> MocAction {
        match input {
            "Q" | "M" => {
                self.screen = GameScreen::GalaxyMap;
                MocAction::SaveGame
            }
            _ => MocAction::Continue,
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> MocAction {
        match input {
            "Y" => MocAction::Quit,
            _ => {
                self.screen = if self.game_state.is_some() {
                    GameScreen::GalaxyMap
                } else {
                    GameScreen::Lobby
                };
                MocAction::SaveGame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_creation() {
        let flow = MocFlow::new(1);
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_intro_advances() {
        let mut flow = MocFlow::new(1);
        let action = flow.handle_char('\r');
        assert!(matches!(action, MocAction::SaveGame));
        // Without a game loaded, should go to lobby
        assert!(matches!(flow.screen, GameScreen::Lobby));
    }

    #[test]
    fn test_single_key_detection() {
        let flow = MocFlow::new(1);
        assert!(flow.is_single_key_screen()); // Intro is single-key
    }
}
