//! Star Trader - Game Screen and Flow State Machine
//!
//! Handles screen transitions and input processing.

use super::state::GameState;
use super::galaxy::{Galaxy, SectorTypeData};
use super::combat::{Opponent, CombatAction, combat_round, generate_ferrengi, should_ferrengi_attack};
use super::economy;
use super::data::{Commodity, SHIP_CLASSES};

/// Current screen/state in the game
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// New game intro
    Intro,
    /// Main hub at current sector
    MainMenu,
    /// Galaxy map / navigation
    Navigation { target_sector: Option<u32> },
    /// Trading at a port
    Trading { mode: TradeMode },
    /// Ship combat
    Combat { opponent: Opponent },
    /// Port information display
    PortInfo,
    /// Planet interaction
    Planet,
    /// StarDock special area
    StarDock { area: StarDockArea },
    /// Corporation management
    Corporation,
    /// Player statistics
    Stats,
    /// Galaxy scan results
    Scanner,
    /// Game over
    GameOver,
    /// Confirm quit
    ConfirmQuit,
}

/// Trading mode
#[derive(Debug, Clone, PartialEq)]
pub enum TradeMode {
    Menu,
    Buying { commodity: Option<Commodity> },
    Selling { commodity: Option<Commodity> },
}

/// StarDock sub-areas
#[derive(Debug, Clone, PartialEq)]
pub enum StarDockArea {
    MainMenu,
    ShipDealer,
    HardwareEmporium,
    FederationHQ,
    CorporateHQ,
    Bank,
}

/// Actions returned by the flow for the session to handle
#[derive(Debug, Clone)]
pub enum StarTraderAction {
    /// Continue - no special action needed
    Continue,
    /// Render output to player
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game ended
    GameOver { final_score: i64 },
    /// Player quit to menu
    Quit,
}

/// Star Trader game flow state machine
pub struct StarTraderFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub galaxy: Galaxy,
    input_buffer: String,
}

impl StarTraderFlow {
    /// Create a new game
    pub fn new(player_id: i64, handle: String, galaxy: Galaxy) -> Self {
        Self {
            state: GameState::new(player_id, handle),
            screen: GameScreen::Intro,
            galaxy,
            input_buffer: String::new(),
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState, galaxy: Galaxy) -> Self {
        Self {
            state,
            screen: GameScreen::MainMenu,
            galaxy,
            input_buffer: String::new(),
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Get galaxy
    pub fn galaxy(&self) -> &Galaxy {
        &self.galaxy
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> StarTraderAction {
        // Handle backspace
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return StarTraderAction::Echo("\x08 \x08".to_string());
            }
            return StarTraderAction::Continue;
        }

        // Handle enter
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return StarTraderAction::Continue;
        }

        // Single-key screens process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return StarTraderAction::Echo(ch.to_string());
        }

        StarTraderAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::MainMenu
                | GameScreen::Navigation { target_sector: None }
                | GameScreen::Trading { mode: TradeMode::Menu }
                | GameScreen::Trading { mode: TradeMode::Buying { commodity: None } }
                | GameScreen::Trading { mode: TradeMode::Selling { commodity: None } }
                | GameScreen::Combat { .. }
                | GameScreen::PortInfo
                | GameScreen::Planet
                | GameScreen::StarDock { .. }
                | GameScreen::Corporation
                | GameScreen::Stats
                | GameScreen::Scanner
                | GameScreen::GameOver
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> StarTraderAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        // Check for new day
        if self.state.check_new_day() {
            self.state.last_message = Some("A new day dawns. Turns refreshed!".to_string());
        }

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::MainMenu => self.handle_main_menu(&input),
            GameScreen::Navigation { target_sector } => self.handle_navigation(&input, *target_sector),
            GameScreen::Trading { mode } => self.handle_trading(&input, mode.clone()),
            GameScreen::Combat { opponent } => self.handle_combat(&input, opponent.clone()),
            GameScreen::PortInfo => self.handle_port_info(&input),
            GameScreen::Planet => self.handle_planet(&input),
            GameScreen::StarDock { area } => self.handle_stardock(&input, area.clone()),
            GameScreen::Corporation => self.handle_corporation(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Scanner => self.handle_scanner(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> StarTraderAction {
        self.screen = GameScreen::MainMenu;
        StarTraderAction::SaveGame
    }

    fn handle_main_menu(&mut self, input: &str) -> StarTraderAction {
        self.state.last_message = None;

        match input {
            "M" | "N" => {
                // Move / Navigate
                self.screen = GameScreen::Navigation { target_sector: None };
                StarTraderAction::SaveGame
            }
            "T" => {
                // Trade (if at port)
                let sector = self.galaxy.get_sector(self.state.sector);
                if let Some(s) = sector {
                    match &s.sector_type {
                        SectorTypeData::Port(_) | SectorTypeData::StarDock => {
                            self.screen = GameScreen::Trading { mode: TradeMode::Menu };
                            StarTraderAction::SaveGame
                        }
                        _ => {
                            self.state.last_message = Some("No port in this sector.".to_string());
                            StarTraderAction::SaveGame
                        }
                    }
                } else {
                    StarTraderAction::Continue
                }
            }
            "P" => {
                // Port info
                let sector = self.galaxy.get_sector(self.state.sector);
                if let Some(s) = sector {
                    if matches!(s.sector_type, SectorTypeData::Port(_)) {
                        self.screen = GameScreen::PortInfo;
                        return StarTraderAction::SaveGame;
                    }
                }
                self.state.last_message = Some("No port in this sector.".to_string());
                StarTraderAction::SaveGame
            }
            "L" => {
                // Land on planet
                let sector = self.galaxy.get_sector(self.state.sector);
                if let Some(s) = sector {
                    if matches!(s.sector_type, SectorTypeData::Planet(_)) {
                        self.screen = GameScreen::Planet;
                        return StarTraderAction::SaveGame;
                    }
                }
                self.state.last_message = Some("No planet in this sector.".to_string());
                StarTraderAction::SaveGame
            }
            "D" => {
                // Dock at StarDock
                let sector = self.galaxy.get_sector(self.state.sector);
                if let Some(s) = sector {
                    if matches!(s.sector_type, SectorTypeData::StarDock) {
                        self.screen = GameScreen::StarDock { area: StarDockArea::MainMenu };
                        return StarTraderAction::SaveGame;
                    }
                }
                self.state.last_message = Some("StarDock is only in Sector 1.".to_string());
                StarTraderAction::SaveGame
            }
            "S" => {
                // Scanner
                self.screen = GameScreen::Scanner;
                StarTraderAction::SaveGame
            }
            "C" => {
                // Corporation
                self.screen = GameScreen::Corporation;
                StarTraderAction::SaveGame
            }
            "I" => {
                // Stats/Info
                self.screen = GameScreen::Stats;
                StarTraderAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                StarTraderAction::SaveGame
            }
            _ => StarTraderAction::Continue,
        }
    }

    fn handle_navigation(&mut self, input: &str, target: Option<u32>) -> StarTraderAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return StarTraderAction::SaveGame;
        }

        // If no target, entering sector number
        if target.is_none() {
            if let Ok(sector_num) = input.parse::<u32>() {
                // Validate sector exists
                if sector_num >= 1 && sector_num <= self.galaxy.size {
                    // Check if adjacent (direct warp)
                    let current = self.galaxy.get_sector(self.state.sector);
                    if let Some(current_sector) = current {
                        if current_sector.warps.contains(&sector_num) {
                            // Direct warp - costs 1 turn
                            return self.warp_to_sector(sector_num);
                        }
                    }

                    // Check for path
                    if let Some(path) = self.galaxy.find_path(self.state.sector, sector_num) {
                        let turns_needed = (path.len() - 1) as u32;
                        self.state.last_message = Some(format!(
                            "Path found: {} sectors. Need {} turns. Press W to warp.",
                            turns_needed,
                            turns_needed
                        ));
                        self.screen = GameScreen::Navigation { target_sector: Some(sector_num) };
                        return StarTraderAction::SaveGame;
                    } else {
                        self.state.last_message = Some("No path to that sector.".to_string());
                        return StarTraderAction::SaveGame;
                    }
                }
            }

            // Show adjacent sectors with number keys
            let current = self.galaxy.get_sector(self.state.sector);
            if let Some(current_sector) = current {
                for (i, &warp) in current_sector.warps.iter().enumerate() {
                    if input == format!("{}", i + 1) {
                        return self.warp_to_sector(warp);
                    }
                }
            }
        } else {
            // Have a target - W to confirm warp
            if input == "W" {
                if let Some(dest) = target {
                    return self.warp_to_sector(dest);
                }
            }
        }

        StarTraderAction::Continue
    }

    fn warp_to_sector(&mut self, dest: u32) -> StarTraderAction {
        // Calculate path
        if let Some(path) = self.galaxy.find_path(self.state.sector, dest) {
            let turns_needed = (path.len() - 1) as u32;

            // Check turns
            if !self.state.use_turns(turns_needed) {
                self.state.last_message = Some(format!(
                    "Not enough turns. Need {}, have {}.",
                    turns_needed,
                    self.state.turns_remaining
                ));
                return StarTraderAction::SaveGame;
            }

            // Move to destination
            self.state.sector = dest;
            self.state.explore_sector(dest);

            // Check for Ferrengi encounter in Ferrengi space
            if let Some(sector) = self.galaxy.get_sector(dest) {
                if let SectorTypeData::FerrengiSpace { strength } = sector.sector_type {
                    if should_ferrengi_attack() {
                        let ferrengi = generate_ferrengi(strength);
                        self.state.last_message = Some(format!("{} attacks!", ferrengi.name()));
                        self.screen = GameScreen::Combat { opponent: ferrengi };
                        return StarTraderAction::SaveGame;
                    }
                }
            }

            self.state.last_message = Some(format!("Arrived at Sector {}.", dest));
            self.screen = GameScreen::MainMenu;
            StarTraderAction::SaveGame
        } else {
            self.state.last_message = Some("Cannot reach that sector.".to_string());
            StarTraderAction::SaveGame
        }
    }

    fn handle_trading(&mut self, input: &str, mode: TradeMode) -> StarTraderAction {
        match mode {
            TradeMode::Menu => {
                match input {
                    "B" => {
                        self.screen = GameScreen::Trading {
                            mode: TradeMode::Buying { commodity: None }
                        };
                        StarTraderAction::SaveGame
                    }
                    "S" => {
                        self.screen = GameScreen::Trading {
                            mode: TradeMode::Selling { commodity: None }
                        };
                        StarTraderAction::SaveGame
                    }
                    "Q" | "X" => {
                        self.screen = GameScreen::MainMenu;
                        StarTraderAction::SaveGame
                    }
                    _ => StarTraderAction::Continue,
                }
            }
            TradeMode::Buying { commodity: None } => {
                // Select commodity to buy
                let comm = match input {
                    "1" | "O" => Some(Commodity::FuelOre),
                    "2" | "R" => Some(Commodity::Organics),
                    "3" | "E" => Some(Commodity::Equipment),
                    "Q" | "B" => {
                        self.screen = GameScreen::Trading { mode: TradeMode::Menu };
                        return StarTraderAction::SaveGame;
                    }
                    _ => None,
                };

                if let Some(c) = comm {
                    self.screen = GameScreen::Trading {
                        mode: TradeMode::Buying { commodity: Some(c) }
                    };
                }
                StarTraderAction::SaveGame
            }
            TradeMode::Buying { commodity: Some(comm) } => {
                // Enter quantity
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::Trading {
                        mode: TradeMode::Buying { commodity: None }
                    };
                    return StarTraderAction::SaveGame;
                }

                if let Ok(qty) = input.parse::<u32>() {
                    // Execute trade
                    if let Some(sector) = self.galaxy.get_sector_mut(self.state.sector) {
                        if let SectorTypeData::Port(ref mut port) = sector.sector_type {
                            let result = economy::buy_from_port(&mut self.state, port, comm, qty);
                            self.state.last_message = Some(result.message);
                        }
                    }
                    self.screen = GameScreen::Trading { mode: TradeMode::Menu };
                }
                StarTraderAction::SaveGame
            }
            TradeMode::Selling { commodity: None } => {
                // Select commodity to sell
                let comm = match input {
                    "1" | "O" => Some(Commodity::FuelOre),
                    "2" | "R" => Some(Commodity::Organics),
                    "3" | "E" => Some(Commodity::Equipment),
                    "Q" | "B" => {
                        self.screen = GameScreen::Trading { mode: TradeMode::Menu };
                        return StarTraderAction::SaveGame;
                    }
                    _ => None,
                };

                if let Some(c) = comm {
                    self.screen = GameScreen::Trading {
                        mode: TradeMode::Selling { commodity: Some(c) }
                    };
                }
                StarTraderAction::SaveGame
            }
            TradeMode::Selling { commodity: Some(comm) } => {
                // Enter quantity
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::Trading {
                        mode: TradeMode::Selling { commodity: None }
                    };
                    return StarTraderAction::SaveGame;
                }

                if let Ok(qty) = input.parse::<u32>() {
                    // Execute trade
                    if let Some(sector) = self.galaxy.get_sector_mut(self.state.sector) {
                        if let SectorTypeData::Port(ref mut port) = sector.sector_type {
                            let result = economy::sell_to_port(&mut self.state, port, comm, qty);
                            self.state.last_message = Some(result.message);
                        }
                    }
                    self.screen = GameScreen::Trading { mode: TradeMode::Menu };
                }
                StarTraderAction::SaveGame
            }
        }
    }

    fn handle_combat(&mut self, input: &str, mut opponent: Opponent) -> StarTraderAction {
        let action = match input {
            "A" | "F" => CombatAction::Attack,
            "R" => CombatAction::Flee,
            "S" => CombatAction::Surrender,
            _ => return StarTraderAction::Continue,
        };

        let result = combat_round(&mut self.state, &mut opponent, action);
        self.state.last_message = Some(result.message.clone());

        if result.victory || result.fled {
            self.screen = GameScreen::MainMenu;
        } else if self.state.fighters == 0 && self.state.shields == 0 {
            // Player destroyed
            self.screen = GameScreen::MainMenu;
        } else {
            // Combat continues
            self.screen = GameScreen::Combat { opponent };
        }

        StarTraderAction::SaveGame
    }

    fn handle_port_info(&mut self, input: &str) -> StarTraderAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::MainMenu;
        }
        StarTraderAction::SaveGame
    }

    fn handle_planet(&mut self, input: &str) -> StarTraderAction {
        match input {
            "C" => {
                // Colonize (if unowned)
                // TODO: Implement colonization
                self.state.last_message = Some("Colonization not yet implemented.".to_string());
                StarTraderAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::MainMenu;
                StarTraderAction::SaveGame
            }
            _ => StarTraderAction::Continue,
        }
    }

    fn handle_stardock(&mut self, input: &str, area: StarDockArea) -> StarTraderAction {
        match area {
            StarDockArea::MainMenu => {
                match input {
                    "1" | "S" => {
                        self.screen = GameScreen::StarDock { area: StarDockArea::ShipDealer };
                        StarTraderAction::SaveGame
                    }
                    "2" | "H" => {
                        self.screen = GameScreen::StarDock { area: StarDockArea::HardwareEmporium };
                        StarTraderAction::SaveGame
                    }
                    "3" | "F" => {
                        self.screen = GameScreen::StarDock { area: StarDockArea::FederationHQ };
                        StarTraderAction::SaveGame
                    }
                    "4" | "C" => {
                        self.screen = GameScreen::StarDock { area: StarDockArea::CorporateHQ };
                        StarTraderAction::SaveGame
                    }
                    "5" | "B" => {
                        self.screen = GameScreen::StarDock { area: StarDockArea::Bank };
                        StarTraderAction::SaveGame
                    }
                    "T" => {
                        self.screen = GameScreen::Trading { mode: TradeMode::Menu };
                        StarTraderAction::SaveGame
                    }
                    "Q" | "X" => {
                        self.screen = GameScreen::MainMenu;
                        StarTraderAction::SaveGame
                    }
                    _ => StarTraderAction::Continue,
                }
            }
            StarDockArea::ShipDealer => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::StarDock { area: StarDockArea::MainMenu };
                    return StarTraderAction::SaveGame;
                }

                // Buy ship by number
                if let Ok(idx) = input.parse::<usize>() {
                    if idx >= 1 && idx <= SHIP_CLASSES.len() {
                        let ship = &SHIP_CLASSES[idx - 1];

                        if ship.requires_commission && !self.state.federation_commission {
                            self.state.last_message = Some("Requires Federation Commission.".to_string());
                            return StarTraderAction::SaveGame;
                        }

                        if self.state.credits < ship.price {
                            self.state.last_message = Some(format!("Need {} credits.", ship.price));
                            return StarTraderAction::SaveGame;
                        }

                        // Buy ship
                        self.state.credits -= ship.price;
                        self.state.ship_class = ship.key.to_string();
                        // Keep fighters/shields within new limits
                        self.state.fighters = self.state.fighters.min(ship.max_fighters);
                        self.state.shields = self.state.shields.min(ship.max_shields);

                        self.state.last_message = Some(format!("Purchased {}!", ship.name));
                    }
                }
                StarTraderAction::SaveGame
            }
            StarDockArea::HardwareEmporium => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::StarDock { area: StarDockArea::MainMenu };
                    return StarTraderAction::SaveGame;
                }

                match input {
                    "F" => {
                        // Buy 10 fighters
                        match economy::buy_fighters(&mut self.state, 10) {
                            Ok(msg) => self.state.last_message = Some(msg),
                            Err(msg) => self.state.last_message = Some(msg),
                        }
                    }
                    "S" => {
                        // Buy 10 shields
                        match economy::buy_shields(&mut self.state, 10) {
                            Ok(msg) => self.state.last_message = Some(msg),
                            Err(msg) => self.state.last_message = Some(msg),
                        }
                    }
                    _ => {}
                }
                StarTraderAction::SaveGame
            }
            StarDockArea::FederationHQ => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::StarDock { area: StarDockArea::MainMenu };
                    return StarTraderAction::SaveGame;
                }

                if input == "C" && !self.state.federation_commission {
                    // Buy commission
                    let cost = 500_000i64;
                    if self.state.credits >= cost && self.state.experience >= 10_000 {
                        self.state.credits -= cost;
                        self.state.federation_commission = true;
                        self.state.last_message = Some("Federation Commission acquired!".to_string());
                    } else if self.state.experience < 10_000 {
                        self.state.last_message = Some("Need 10,000 experience for Commission.".to_string());
                    } else {
                        self.state.last_message = Some(format!("Need {} credits.", cost));
                    }
                }
                StarTraderAction::SaveGame
            }
            StarDockArea::CorporateHQ => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::StarDock { area: StarDockArea::MainMenu };
                    return StarTraderAction::SaveGame;
                }
                // Corporation management - TODO
                self.state.last_message = Some("Corporation management coming soon.".to_string());
                StarTraderAction::SaveGame
            }
            StarDockArea::Bank => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::StarDock { area: StarDockArea::MainMenu };
                    return StarTraderAction::SaveGame;
                }
                // Bank - TODO
                self.state.last_message = Some("Banking coming soon.".to_string());
                StarTraderAction::SaveGame
            }
        }
    }

    fn handle_corporation(&mut self, input: &str) -> StarTraderAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::MainMenu;
        }
        StarTraderAction::SaveGame
    }

    fn handle_stats(&mut self, input: &str) -> StarTraderAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::MainMenu;
        }
        StarTraderAction::SaveGame
    }

    fn handle_scanner(&mut self, input: &str) -> StarTraderAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::MainMenu;
        }
        StarTraderAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> StarTraderAction {
        StarTraderAction::GameOver {
            final_score: self.state.credits + self.state.experience,
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> StarTraderAction {
        match input {
            "Y" => StarTraderAction::Quit,
            _ => {
                self.screen = GameScreen::MainMenu;
                StarTraderAction::SaveGame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::star_trader::galaxy::Galaxy;

    fn create_test_flow() -> StarTraderFlow {
        let galaxy = Galaxy::generate(12345, 100);
        StarTraderFlow::new(1, "TestPlayer".to_string(), galaxy)
    }

    #[test]
    fn test_new_flow() {
        let flow = create_test_flow();
        assert!(matches!(flow.screen, GameScreen::Intro));
        assert_eq!(flow.state.sector, 1);
    }

    #[test]
    fn test_intro_to_main_menu() {
        let mut flow = create_test_flow();
        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_navigation_screen() {
        let mut flow = create_test_flow();
        flow.screen = GameScreen::MainMenu;
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::Navigation { .. }));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = create_test_flow();
        flow.screen = GameScreen::MainMenu;
        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));
    }

    #[test]
    fn test_back_from_quit() {
        let mut flow = create_test_flow();
        flow.screen = GameScreen::ConfirmQuit;
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_stardock_access() {
        let mut flow = create_test_flow();
        flow.screen = GameScreen::MainMenu;
        flow.state.sector = 1;  // StarDock location
        flow.handle_char('D');
        assert!(matches!(flow.screen, GameScreen::StarDock { .. }));
    }
}
