//! Screen state machine for Tanks
//!
//! Manages screen transitions and input handling.

use super::state::{TankGame, GamePhase};
use super::lobby::TanksLobby;
use super::render::*;

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum TanksScreen {
    /// Main menu
    Menu,
    /// How to play
    HowToPlay,
    /// Listing available games
    GameList,
    /// Entering invite code
    InviteEntry,
    /// In a lobby waiting for game
    Lobby,
    /// Active game - battlefield view
    Battlefield { show_aim: bool },
    /// Active game - turn result
    TurnResult,
    /// Viewing another tank (PIP)
    ViewingTank { target_index: usize },
    /// Game over screen
    GameOver,
    /// Viewing leaderboard
    Leaderboard,
}

/// Actions returned by the flow for session to handle
#[derive(Debug, Clone)]
pub enum TanksAction {
    /// No action needed
    Continue,
    /// Render output to player
    Render(String),
    /// Echo input back
    Echo(String),
    /// Player quit the game
    Quit,
    /// Notify other players of update
    Broadcast(String),
}

/// Per-player flow state
pub struct TanksFlow {
    pub screen: TanksScreen,
    pub user_id: i64,
    pub handle: String,
    input_buffer: String,
    invite_code_buffer: String,
}

impl TanksFlow {
    pub fn new(user_id: i64, handle: String) -> Self {
        Self {
            screen: TanksScreen::Menu,
            user_id,
            handle,
            input_buffer: String::new(),
            invite_code_buffer: String::new(),
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, lobby: &mut TanksLobby) -> TanksAction {
        // Backspace
        if ch == '\x7f' || ch == '\x08' {
            match self.screen {
                TanksScreen::InviteEntry => {
                    if self.invite_code_buffer.pop().is_some() {
                        return TanksAction::Echo("\x08 \x08".to_string());
                    }
                }
                _ => {}
            }
            return TanksAction::Continue;
        }

        // Enter
        if ch == '\r' || ch == '\n' {
            return self.process_input(lobby);
        }

        // Ignore control chars (except arrow keys which come as escape sequences)
        if ch.is_control() && ch != '\x1b' {
            return TanksAction::Continue;
        }

        // Handle based on screen
        match self.screen {
            TanksScreen::InviteEntry => {
                if self.invite_code_buffer.len() < 6 {
                    let upper = ch.to_ascii_uppercase();
                    self.invite_code_buffer.push(upper);
                    return TanksAction::Echo(upper.to_string());
                }
                TanksAction::Continue
            }
            TanksScreen::Battlefield { .. } => {
                // Immediate input for game controls
                self.input_buffer.clear();
                self.input_buffer.push(ch);
                self.process_game_input(lobby)
            }
            _ => {
                // Single-key input for menus
                self.input_buffer.clear();
                self.input_buffer.push(ch);
                self.process_input(lobby)
            }
        }
    }

    /// Handle arrow key escape sequences
    pub fn handle_escape_sequence(&mut self, seq: &str, lobby: &mut TanksLobby) -> TanksAction {
        if !matches!(self.screen, TanksScreen::Battlefield { .. }) {
            return TanksAction::Continue;
        }

        // Arrow key sequences: [A=up, B=down, C=right, D=left]
        match seq {
            "[A" | "OA" => {
                // Up arrow - increase power
                self.adjust_power(lobby, 5)
            }
            "[B" | "OB" => {
                // Down arrow - decrease power
                self.adjust_power(lobby, -5)
            }
            "[C" | "OC" => {
                // Right arrow - increase angle (if facing right) or decrease (if left)
                self.adjust_angle(lobby, 5)
            }
            "[D" | "OD" => {
                // Left arrow - decrease angle
                self.adjust_angle(lobby, -5)
            }
            _ => TanksAction::Continue,
        }
    }

    fn adjust_angle(&mut self, lobby: &mut TanksLobby, delta: i32) -> TanksAction {
        if let Some(game) = lobby.get_player_game_mut(self.user_id) {
            // Check if it's our turn first
            let is_our_turn = game.get_current_tank().map(|t| t.user_id) == Some(self.user_id);
            if is_our_turn {
                if let Some((_, tank)) = game.get_tank_by_user_mut(self.user_id) {
                    tank.adjust_angle(delta);
                }
                return TanksAction::Render(render_battlefield(game, self.user_id, true));
            }
        }
        TanksAction::Continue
    }

    fn adjust_power(&mut self, lobby: &mut TanksLobby, delta: i32) -> TanksAction {
        if let Some(game) = lobby.get_player_game_mut(self.user_id) {
            // Check if it's our turn first
            let is_our_turn = game.get_current_tank().map(|t| t.user_id) == Some(self.user_id);
            if is_our_turn {
                if let Some((_, tank)) = game.get_tank_by_user_mut(self.user_id) {
                    tank.adjust_power(delta);
                }
                return TanksAction::Render(render_battlefield(game, self.user_id, true));
            }
        }
        TanksAction::Continue
    }

    /// Process buffered input
    fn process_input(&mut self, lobby: &mut TanksLobby) -> TanksAction {
        let input = std::mem::take(&mut self.input_buffer).trim().to_uppercase();

        match self.screen {
            TanksScreen::Menu => self.handle_menu(&input, lobby),
            TanksScreen::HowToPlay => self.handle_how_to_play(&input),
            TanksScreen::GameList => self.handle_game_list(&input, lobby),
            TanksScreen::InviteEntry => self.handle_invite_entry(lobby),
            TanksScreen::Lobby => self.handle_lobby(&input, lobby),
            TanksScreen::Battlefield { .. } => self.process_game_input(lobby),
            TanksScreen::TurnResult => self.handle_turn_result(lobby),
            TanksScreen::ViewingTank { target_index } => self.handle_viewing_tank(&input, lobby, target_index),
            TanksScreen::GameOver => self.handle_game_over(lobby),
            TanksScreen::Leaderboard => self.handle_leaderboard(&input),
        }
    }

    fn handle_menu(&mut self, input: &str, lobby: &mut TanksLobby) -> TanksAction {
        match input {
            "J" => {
                self.screen = TanksScreen::GameList;
                let lobbies = lobby.list_public_lobbies();
                TanksAction::Render(render_game_list(&lobbies))
            }
            "C" => {
                match lobby.create_public_lobby(self.user_id, self.handle.clone()) {
                    Ok(game_id) => {
                        self.screen = TanksScreen::Lobby;
                        if let Some(game_lobby) = lobby.lobbies.get(&game_id) {
                            TanksAction::Render(render_lobby(game_lobby))
                        } else {
                            TanksAction::Render(render_error("Failed to create lobby"))
                        }
                    }
                    Err(e) => TanksAction::Render(render_error(e)),
                }
            }
            "P" => {
                match lobby.create_private_lobby(self.user_id, self.handle.clone()) {
                    Ok((game_id, _code)) => {
                        self.screen = TanksScreen::Lobby;
                        if let Some(game_lobby) = lobby.lobbies.get(&game_id) {
                            TanksAction::Render(render_lobby(game_lobby))
                        } else {
                            TanksAction::Render(render_error("Failed to create lobby"))
                        }
                    }
                    Err(e) => TanksAction::Render(render_error(e)),
                }
            }
            "I" => {
                self.screen = TanksScreen::InviteEntry;
                self.invite_code_buffer.clear();
                TanksAction::Render(render_invite_prompt(""))
            }
            "L" => {
                self.screen = TanksScreen::Leaderboard;
                // Would be populated from DB
                TanksAction::Render(render_leaderboard(&[]))
            }
            "H" => {
                self.screen = TanksScreen::HowToPlay;
                TanksAction::Render(render_how_to_play())
            }
            "Q" => TanksAction::Quit,
            _ => TanksAction::Continue,
        }
    }

    fn handle_how_to_play(&mut self, _input: &str) -> TanksAction {
        self.screen = TanksScreen::Menu;
        TanksAction::Render(render_tanks_menu())
    }

    fn handle_game_list(&mut self, input: &str, lobby: &mut TanksLobby) -> TanksAction {
        if input == "B" || input == "Q" {
            self.screen = TanksScreen::Menu;
            return TanksAction::Render(render_tanks_menu());
        }

        // Try to join by number
        if let Ok(num) = input.parse::<usize>() {
            let lobbies = lobby.list_public_lobbies();
            if num > 0 && num <= lobbies.len() {
                let (game_id, _, _) = lobbies[num - 1];
                match lobby.join_public_lobby(self.user_id, self.handle.clone(), Some(game_id)) {
                    Ok(joined_id) => {
                        self.screen = TanksScreen::Lobby;
                        if let Some(game_lobby) = lobby.lobbies.get(&joined_id) {
                            return TanksAction::Render(render_lobby(game_lobby));
                        }
                    }
                    Err(e) => return TanksAction::Render(render_error(e)),
                }
            }
        }

        TanksAction::Continue
    }

    fn handle_invite_entry(&mut self, lobby: &mut TanksLobby) -> TanksAction {
        let code = std::mem::take(&mut self.invite_code_buffer);

        if code.eq_ignore_ascii_case("Q") || code.is_empty() {
            self.screen = TanksScreen::Menu;
            return TanksAction::Render(render_tanks_menu());
        }

        match lobby.join_by_invite(self.user_id, self.handle.clone(), &code) {
            Ok(game_id) => {
                self.screen = TanksScreen::Lobby;
                if let Some(game_lobby) = lobby.lobbies.get(&game_id) {
                    TanksAction::Render(render_lobby(game_lobby))
                } else {
                    TanksAction::Render(render_error("Lobby not found"))
                }
            }
            Err(e) => {
                self.invite_code_buffer.clear();
                TanksAction::Render(format!("{}{}", render_error(e), render_invite_prompt("")))
            }
        }
    }

    fn handle_lobby(&mut self, input: &str, lobby: &mut TanksLobby) -> TanksAction {
        match input {
            "R" => {
                // Toggle ready
                if let Some(game_id) = lobby.player_games.get(&self.user_id).copied() {
                    if let Some(game_lobby) = lobby.lobbies.get_mut(&game_id) {
                        let current_ready = game_lobby.players.iter()
                            .find(|p| p.user_id == self.user_id)
                            .map(|p| p.is_ready)
                            .unwrap_or(false);
                        game_lobby.set_ready(self.user_id, !current_ready);
                        return TanksAction::Render(render_lobby(game_lobby));
                    }
                }
                TanksAction::Continue
            }
            "S" => {
                // Start game (host only)
                match lobby.start_game(self.user_id) {
                    Ok(game_id) => {
                        self.screen = TanksScreen::Battlefield { show_aim: true };
                        if let Some(game) = lobby.active_games.get(&game_id) {
                            TanksAction::Render(render_battlefield(game, self.user_id, true))
                        } else {
                            TanksAction::Render(render_error("Game not found"))
                        }
                    }
                    Err(e) => TanksAction::Render(render_error(e)),
                }
            }
            "Q" => {
                // Leave lobby
                let _ = lobby.leave_game(self.user_id);
                self.screen = TanksScreen::Menu;
                TanksAction::Render(render_tanks_menu())
            }
            _ => TanksAction::Continue,
        }
    }

    fn process_game_input(&mut self, lobby: &mut TanksLobby) -> TanksAction {
        let input = self.input_buffer.trim().to_uppercase();
        self.input_buffer.clear();

        // Check if it's our turn
        let is_my_turn = lobby.get_player_game(self.user_id)
            .and_then(|g| g.get_current_tank())
            .map(|t| t.user_id == self.user_id)
            .unwrap_or(false);

        match input.as_str() {
            " " | "\r" | "\n" => {
                // Fire!
                if !is_my_turn {
                    return TanksAction::Continue;
                }

                if let Some(game) = lobby.get_player_game_mut(self.user_id) {
                    if let Some((tank_id, _)) = game.get_tank_by_user(self.user_id) {
                        if game.fire(tank_id).is_some() {
                            self.screen = TanksScreen::TurnResult;
                            return TanksAction::Render(render_turn_result(game, "BOOM!"));
                        }
                    }
                }
                TanksAction::Continue
            }
            "W" => {
                // Fine angle up
                if is_my_turn {
                    self.adjust_angle(lobby, 1)
                } else {
                    TanksAction::Continue
                }
            }
            "S" => {
                // Fine angle down
                if is_my_turn {
                    self.adjust_angle(lobby, -1)
                } else {
                    TanksAction::Continue
                }
            }
            "A" => {
                // Fine power down
                if is_my_turn {
                    self.adjust_power(lobby, -1)
                } else {
                    TanksAction::Continue
                }
            }
            "D" => {
                // Fine power up
                if is_my_turn {
                    self.adjust_power(lobby, 1)
                } else {
                    TanksAction::Continue
                }
            }
            "\t" => {
                // Cycle weapon
                if is_my_turn {
                    if let Some(game) = lobby.get_player_game_mut(self.user_id) {
                        if let Some((_, tank)) = game.get_tank_by_user_mut(self.user_id) {
                            tank.cycle_weapon(true);
                            return TanksAction::Render(render_battlefield(game, self.user_id, true));
                        }
                    }
                }
                TanksAction::Continue
            }
            "V" => {
                // View other tanks
                self.screen = TanksScreen::ViewingTank { target_index: 0 };
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    let tanks: Vec<_> = game.tanks.values()
                        .filter(|t| t.user_id != self.user_id && t.is_alive)
                        .collect();
                    if let Some(tank) = tanks.first() {
                        return TanksAction::Render(render_pip_view(tank, &game.terrain));
                    }
                }
                TanksAction::Continue
            }
            "Q" => {
                // Leave game
                let _ = lobby.leave_game(self.user_id);
                self.screen = TanksScreen::Menu;
                TanksAction::Render(render_tanks_menu())
            }
            _ => TanksAction::Continue,
        }
    }

    fn handle_turn_result(&mut self, lobby: &mut TanksLobby) -> TanksAction {
        // Any key advances to next turn
        if let Some(game) = lobby.get_player_game_mut(self.user_id) {
            // Check if game is over
            if game.phase == GamePhase::GameOver {
                self.screen = TanksScreen::GameOver;
                return TanksAction::Render(render_game_over(game));
            }

            // Advance to next turn
            game.next_turn();
            self.screen = TanksScreen::Battlefield { show_aim: true };
            return TanksAction::Render(render_battlefield(game, self.user_id, true));
        }

        self.screen = TanksScreen::Menu;
        TanksAction::Render(render_tanks_menu())
    }

    fn handle_viewing_tank(&mut self, input: &str, lobby: &TanksLobby, current_index: usize) -> TanksAction {
        match input {
            "N" | " " | "\r" | "\n" => {
                // Next tank
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    let tanks: Vec<_> = game.tanks.values()
                        .filter(|t| t.user_id != self.user_id && t.is_alive)
                        .collect();
                    if !tanks.is_empty() {
                        let next_index = (current_index + 1) % tanks.len();
                        self.screen = TanksScreen::ViewingTank { target_index: next_index };
                        return TanksAction::Render(render_pip_view(tanks[next_index], &game.terrain));
                    }
                }
                TanksAction::Continue
            }
            "Q" | "V" | "B" => {
                // Return to battlefield
                self.screen = TanksScreen::Battlefield { show_aim: true };
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    TanksAction::Render(render_battlefield(game, self.user_id, true))
                } else {
                    self.screen = TanksScreen::Menu;
                    TanksAction::Render(render_tanks_menu())
                }
            }
            _ => TanksAction::Continue,
        }
    }

    fn handle_game_over(&mut self, lobby: &mut TanksLobby) -> TanksAction {
        // Clean up and return to menu
        let _ = lobby.leave_game(self.user_id);
        self.screen = TanksScreen::Menu;
        TanksAction::Render(render_tanks_menu())
    }

    fn handle_leaderboard(&mut self, _input: &str) -> TanksAction {
        self.screen = TanksScreen::Menu;
        TanksAction::Render(render_tanks_menu())
    }

    /// Sync screen with game phase changes
    pub fn sync_with_game(&mut self, game: &TankGame) -> Option<String> {
        match &game.phase {
            GamePhase::GameOver => {
                if self.screen != TanksScreen::GameOver {
                    self.screen = TanksScreen::GameOver;
                    return Some(render_game_over(game));
                }
            }
            GamePhase::TurnResult { .. } => {
                if self.screen != TanksScreen::TurnResult {
                    self.screen = TanksScreen::TurnResult;
                    return Some(render_turn_result(game, ""));
                }
            }
            GamePhase::TurnActive { .. } => {
                if !matches!(self.screen, TanksScreen::Battlefield { .. }) {
                    self.screen = TanksScreen::Battlefield { show_aim: true };
                    return Some(render_battlefield(game, self.user_id, true));
                }
            }
            _ => {}
        }
        None
    }

    /// Render current screen
    pub fn render(&self, lobby: &TanksLobby) -> String {
        match &self.screen {
            TanksScreen::Menu => render_tanks_menu(),
            TanksScreen::HowToPlay => render_how_to_play(),
            TanksScreen::GameList => {
                let lobbies = lobby.list_public_lobbies();
                render_game_list(&lobbies)
            }
            TanksScreen::InviteEntry => render_invite_prompt(&self.invite_code_buffer),
            TanksScreen::Lobby => {
                if let Some(game_lobby) = lobby.get_player_lobby(self.user_id) {
                    render_lobby(game_lobby)
                } else {
                    render_tanks_menu()
                }
            }
            TanksScreen::Battlefield { show_aim } => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    render_battlefield(game, self.user_id, *show_aim)
                } else {
                    render_tanks_menu()
                }
            }
            TanksScreen::TurnResult => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    render_turn_result(game, "")
                } else {
                    render_tanks_menu()
                }
            }
            TanksScreen::ViewingTank { target_index } => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    let tanks: Vec<_> = game.tanks.values()
                        .filter(|t| t.user_id != self.user_id && t.is_alive)
                        .collect();
                    if let Some(tank) = tanks.get(*target_index) {
                        render_pip_view(tank, &game.terrain)
                    } else {
                        render_tanks_menu()
                    }
                } else {
                    render_tanks_menu()
                }
            }
            TanksScreen::GameOver => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    render_game_over(game)
                } else {
                    render_tanks_menu()
                }
            }
            TanksScreen::Leaderboard => render_leaderboard(&[]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_creation() {
        let flow = TanksFlow::new(1, "TestPlayer".to_string());
        assert_eq!(flow.screen, TanksScreen::Menu);
        assert_eq!(flow.user_id, 1);
    }

    #[test]
    fn test_menu_navigation() {
        let mut flow = TanksFlow::new(1, "Test".to_string());
        let mut lobby = TanksLobby::new();

        // Press H for how to play
        flow.input_buffer = "H".to_string();
        let action = flow.process_input(&mut lobby);

        assert!(matches!(action, TanksAction::Render(_)));
        assert_eq!(flow.screen, TanksScreen::HowToPlay);

        // Press any key to return
        flow.input_buffer = " ".to_string();
        let action = flow.process_input(&mut lobby);

        assert!(matches!(action, TanksAction::Render(_)));
        assert_eq!(flow.screen, TanksScreen::Menu);
    }

    #[test]
    fn test_create_public_lobby() {
        let mut flow = TanksFlow::new(1, "Host".to_string());
        let mut lobby = TanksLobby::new();

        flow.input_buffer = "C".to_string();
        let action = flow.process_input(&mut lobby);

        assert!(matches!(action, TanksAction::Render(_)));
        assert_eq!(flow.screen, TanksScreen::Lobby);
        assert_eq!(lobby.lobbies.len(), 1);
    }

    #[test]
    fn test_quit_action() {
        let mut flow = TanksFlow::new(1, "Test".to_string());
        let mut lobby = TanksLobby::new();

        flow.input_buffer = "Q".to_string();
        let action = flow.process_input(&mut lobby);

        assert!(matches!(action, TanksAction::Quit));
    }
}
