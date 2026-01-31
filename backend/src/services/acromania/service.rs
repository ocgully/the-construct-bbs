//! Acromania service - session routing and game coordination
//!
//! Uses __acromania__ sentinel for session routing (per task requirements).
//! Coordinates between game logic, lobby system, and rendering.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::games::acromania::{
    AcroGame, AcroLobby, GamePhase,
    render_acro_menu, render_how_to_play, render_lobby, render_acronym_reveal,
    render_submission, render_voting, render_results, render_final_results,
    render_error, render_game_list, render_invite_prompt, render_leaderboard,
    render_submission_confirmed,
};
use crate::services::acromania::db::AcroDb;

/// Sentinel for session routing
pub const SENTINEL: &str = "__acromania__";

/// Current screen the player is viewing
#[derive(Debug, Clone, PartialEq)]
pub enum AcroScreen {
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
    /// Active game - acronym reveal
    AcronymReveal,
    /// Active game - submission phase
    Submission,
    /// Active game - voting phase
    Voting,
    /// Active game - results
    Results,
    /// Active game - final results
    FinalResults,
    /// Viewing leaderboard
    Leaderboard,
}

/// Actions returned by the service for session to handle
#[derive(Debug, Clone)]
pub enum AcroAction {
    /// No action needed
    Continue,
    /// Render output to player
    Render(String),
    /// Echo input back
    Echo(String),
    /// Player quit the game
    Quit,
}

/// Per-player service state
pub struct AcroService {
    pub screen: AcroScreen,
    pub user_id: i64,
    pub handle: String,
    input_buffer: String,
    invite_code_buffer: String,
    /// Track current submission for display
    current_submission: Option<String>,
}

impl AcroService {
    pub fn new(user_id: i64, handle: String) -> Self {
        Self {
            screen: AcroScreen::Menu,
            user_id,
            handle,
            input_buffer: String::new(),
            invite_code_buffer: String::new(),
            current_submission: None,
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, lobby: &mut AcroLobby) -> AcroAction {
        // Backspace
        if ch == '\x7f' || ch == '\x08' {
            match self.screen {
                AcroScreen::InviteEntry => {
                    if self.invite_code_buffer.pop().is_some() {
                        return AcroAction::Echo("\x08 \x08".to_string());
                    }
                }
                AcroScreen::Submission => {
                    if self.input_buffer.pop().is_some() {
                        return AcroAction::Echo("\x08 \x08".to_string());
                    }
                }
                _ => {}
            }
            return AcroAction::Continue;
        }

        // Enter
        if ch == '\r' || ch == '\n' {
            return self.process_input(lobby);
        }

        // Ignore control chars
        if ch.is_control() {
            return AcroAction::Continue;
        }

        // Buffer input for screens that need it
        match self.screen {
            AcroScreen::InviteEntry => {
                if self.invite_code_buffer.len() < 6 {
                    let upper = ch.to_ascii_uppercase();
                    self.invite_code_buffer.push(upper);
                    return AcroAction::Echo(upper.to_string());
                }
            }
            AcroScreen::Submission => {
                if self.input_buffer.len() < 200 { // Max submission length
                    self.input_buffer.push(ch);
                    return AcroAction::Echo(ch.to_string());
                }
            }
            _ => {
                // Single-key input
                self.input_buffer.clear();
                self.input_buffer.push(ch);
                return self.process_input(lobby);
            }
        }

        AcroAction::Continue
    }

    /// Process buffered input
    fn process_input(&mut self, lobby: &mut AcroLobby) -> AcroAction {
        let input = std::mem::take(&mut self.input_buffer).trim().to_uppercase();

        match self.screen {
            AcroScreen::Menu => self.handle_menu(&input, lobby),
            AcroScreen::HowToPlay => self.handle_how_to_play(&input),
            AcroScreen::GameList => self.handle_game_list(&input, lobby),
            AcroScreen::InviteEntry => self.handle_invite_entry(lobby),
            AcroScreen::Lobby => self.handle_lobby(&input, lobby),
            AcroScreen::AcronymReveal => AcroAction::Continue, // Timer-driven
            AcroScreen::Submission => self.handle_submission(lobby),
            AcroScreen::Voting => self.handle_voting(&input, lobby),
            AcroScreen::Results => AcroAction::Continue, // Timer-driven
            AcroScreen::FinalResults => self.handle_final_results(lobby),
            AcroScreen::Leaderboard => self.handle_leaderboard(&input),
        }
    }

    fn handle_menu(&mut self, input: &str, lobby: &mut AcroLobby) -> AcroAction {
        match input {
            "J" => {
                // Join public game
                self.screen = AcroScreen::GameList;
                let lobbies = lobby.list_public_lobbies();
                AcroAction::Render(render_game_list(&lobbies))
            }
            "C" => {
                // Create public game
                match lobby.create_public_lobby(self.user_id, self.handle.clone()) {
                    Ok(game_id) => {
                        self.screen = AcroScreen::Lobby;
                        if let Some(game_lobby) = lobby.lobbies.get(&game_id) {
                            AcroAction::Render(render_lobby(game_lobby))
                        } else {
                            AcroAction::Render(render_error("Failed to create lobby"))
                        }
                    }
                    Err(e) => AcroAction::Render(render_error(e)),
                }
            }
            "P" => {
                // Create private game
                match lobby.create_private_lobby(self.user_id, self.handle.clone()) {
                    Ok((game_id, _code)) => {
                        self.screen = AcroScreen::Lobby;
                        if let Some(game_lobby) = lobby.lobbies.get(&game_id) {
                            AcroAction::Render(render_lobby(game_lobby))
                        } else {
                            AcroAction::Render(render_error("Failed to create lobby"))
                        }
                    }
                    Err(e) => AcroAction::Render(render_error(e)),
                }
            }
            "I" => {
                // Enter invite code
                self.screen = AcroScreen::InviteEntry;
                self.invite_code_buffer.clear();
                AcroAction::Render(render_invite_prompt(""))
            }
            "L" => {
                // Show leaderboard
                self.screen = AcroScreen::Leaderboard;
                // Will be rendered with data from DB
                AcroAction::Continue
            }
            "H" => {
                // How to play
                self.screen = AcroScreen::HowToPlay;
                AcroAction::Render(render_how_to_play())
            }
            "Q" => {
                AcroAction::Quit
            }
            _ => AcroAction::Continue,
        }
    }

    fn handle_how_to_play(&mut self, _input: &str) -> AcroAction {
        self.screen = AcroScreen::Menu;
        AcroAction::Render(render_acro_menu())
    }

    fn handle_game_list(&mut self, input: &str, lobby: &mut AcroLobby) -> AcroAction {
        if input == "B" || input == "Q" {
            self.screen = AcroScreen::Menu;
            return AcroAction::Render(render_acro_menu());
        }

        // Try to join by number
        if let Ok(num) = input.parse::<usize>() {
            let lobbies = lobby.list_public_lobbies();
            if num > 0 && num <= lobbies.len() {
                let (game_id, _, _) = lobbies[num - 1];
                match lobby.join_public_lobby(self.user_id, self.handle.clone(), Some(game_id)) {
                    Ok(joined_id) => {
                        self.screen = AcroScreen::Lobby;
                        if let Some(game_lobby) = lobby.lobbies.get(&joined_id) {
                            return AcroAction::Render(render_lobby(game_lobby));
                        }
                    }
                    Err(e) => return AcroAction::Render(render_error(e)),
                }
            }
        }

        AcroAction::Continue
    }

    fn handle_invite_entry(&mut self, lobby: &mut AcroLobby) -> AcroAction {
        let code = std::mem::take(&mut self.invite_code_buffer);

        if code.eq_ignore_ascii_case("Q") || code.is_empty() {
            self.screen = AcroScreen::Menu;
            return AcroAction::Render(render_acro_menu());
        }

        match lobby.join_by_invite(self.user_id, self.handle.clone(), &code) {
            Ok(game_id) => {
                self.screen = AcroScreen::Lobby;
                if let Some(game_lobby) = lobby.lobbies.get(&game_id) {
                    AcroAction::Render(render_lobby(game_lobby))
                } else {
                    AcroAction::Render(render_error("Lobby not found"))
                }
            }
            Err(e) => {
                self.invite_code_buffer.clear();
                AcroAction::Render(format!("{}{}", render_error(e), render_invite_prompt("")))
            }
        }
    }

    fn handle_lobby(&mut self, input: &str, lobby: &mut AcroLobby) -> AcroAction {
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
                        return AcroAction::Render(render_lobby(game_lobby));
                    }
                }
                AcroAction::Continue
            }
            "S" => {
                // Start game (host only)
                match lobby.start_game(self.user_id) {
                    Ok(game_id) => {
                        self.screen = AcroScreen::AcronymReveal;
                        if let Some(game) = lobby.active_games.get(&game_id) {
                            AcroAction::Render(render_acronym_reveal(game))
                        } else {
                            AcroAction::Render(render_error("Game not found"))
                        }
                    }
                    Err(e) => AcroAction::Render(render_error(e)),
                }
            }
            "Q" => {
                // Leave lobby
                let _ = lobby.leave_game(self.user_id);
                self.screen = AcroScreen::Menu;
                AcroAction::Render(render_acro_menu())
            }
            _ => AcroAction::Continue,
        }
    }

    fn handle_submission(&mut self, lobby: &mut AcroLobby) -> AcroAction {
        let text = std::mem::take(&mut self.input_buffer);

        if text.is_empty() {
            return AcroAction::Continue;
        }

        if let Some(game) = lobby.get_player_game_mut(self.user_id) {
            match game.submit(self.user_id, text.clone()) {
                Ok(()) => {
                    self.current_submission = Some(text.clone());
                    AcroAction::Render(render_submission_confirmed(&text))
                }
                Err(e) => {
                    // Put text back in buffer for editing
                    self.input_buffer = text;
                    AcroAction::Render(render_error(e))
                }
            }
        } else {
            AcroAction::Continue
        }
    }

    fn handle_voting(&mut self, input: &str, lobby: &mut AcroLobby) -> AcroAction {
        if let Ok(num) = input.parse::<usize>() {
            if let Some(game) = lobby.get_player_game_mut(self.user_id) {
                let options = game.get_voting_options();
                if num > 0 && num <= options.len() {
                    let (submission_id, _) = options[num - 1];
                    match game.vote(self.user_id, submission_id) {
                        Ok(()) => {
                            let options = game.get_voting_options();
                            let current_vote = game.players.get(&self.user_id)
                                .and_then(|p| p.current_vote);
                            return AcroAction::Render(render_voting(game, &options, current_vote));
                        }
                        Err(e) => return AcroAction::Render(render_error(e)),
                    }
                }
            }
        }

        AcroAction::Continue
    }

    fn handle_final_results(&mut self, lobby: &mut AcroLobby) -> AcroAction {
        // Clean up and return to menu
        let _ = lobby.leave_game(self.user_id);
        self.screen = AcroScreen::Menu;
        self.current_submission = None;
        AcroAction::Render(render_acro_menu())
    }

    fn handle_leaderboard(&mut self, _input: &str) -> AcroAction {
        self.screen = AcroScreen::Menu;
        AcroAction::Render(render_acro_menu())
    }

    /// Update screen based on game phase
    pub fn sync_with_game(&mut self, game: &AcroGame) -> Option<String> {
        let new_screen = match game.phase {
            GamePhase::Starting | GamePhase::AcronymReveal => AcroScreen::AcronymReveal,
            GamePhase::Submission => AcroScreen::Submission,
            GamePhase::Voting => AcroScreen::Voting,
            GamePhase::Results => AcroScreen::Results,
            GamePhase::FinalResults => AcroScreen::FinalResults,
            GamePhase::Ended => {
                self.screen = AcroScreen::Menu;
                return Some(render_acro_menu());
            }
            _ => return None,
        };

        if new_screen != self.screen {
            self.screen = new_screen.clone();
            self.input_buffer.clear();

            let output = match new_screen {
                AcroScreen::AcronymReveal => render_acronym_reveal(game),
                AcroScreen::Submission => render_submission(game, ""),
                AcroScreen::Voting => {
                    let options = game.get_voting_options();
                    let current_vote = game.players.get(&self.user_id)
                        .and_then(|p| p.current_vote);
                    render_voting(game, &options, current_vote)
                }
                AcroScreen::Results => {
                    if let Some(round) = &game.current_round {
                        if let Some(results) = &round.results {
                            render_results(game, results)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                }
                AcroScreen::FinalResults => render_final_results(game),
                _ => String::new(),
            };

            return Some(output);
        }

        None
    }

    /// Render current screen
    #[allow(dead_code)]
    pub fn render(&self, lobby: &AcroLobby) -> String {
        match self.screen {
            AcroScreen::Menu => render_acro_menu(),
            AcroScreen::HowToPlay => render_how_to_play(),
            AcroScreen::GameList => {
                let lobbies = lobby.list_public_lobbies();
                render_game_list(&lobbies)
            }
            AcroScreen::InviteEntry => render_invite_prompt(&self.invite_code_buffer),
            AcroScreen::Lobby => {
                if let Some(game_lobby) = lobby.get_player_lobby(self.user_id) {
                    render_lobby(game_lobby)
                } else {
                    render_acro_menu()
                }
            }
            AcroScreen::Submission => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    render_submission(game, &self.input_buffer)
                } else {
                    render_acro_menu()
                }
            }
            AcroScreen::Voting => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    let options = game.get_voting_options();
                    let current_vote = game.players.get(&self.user_id)
                        .and_then(|p| p.current_vote);
                    render_voting(game, &options, current_vote)
                } else {
                    render_acro_menu()
                }
            }
            AcroScreen::AcronymReveal | AcroScreen::Results | AcroScreen::FinalResults => {
                if let Some(game) = lobby.get_player_game(self.user_id) {
                    match self.screen {
                        AcroScreen::AcronymReveal => render_acronym_reveal(game),
                        AcroScreen::Results => {
                            if let Some(round) = &game.current_round {
                                if let Some(results) = &round.results {
                                    render_results(game, results)
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        }
                        AcroScreen::FinalResults => render_final_results(game),
                        _ => String::new(),
                    }
                } else {
                    render_acro_menu()
                }
            }
            AcroScreen::Leaderboard => {
                // Placeholder - actual data will come from DB
                render_leaderboard(&[])
            }
        }
    }
}

/// Global lobby manager (shared across all sessions)
#[allow(dead_code)]
pub type SharedLobby = Arc<RwLock<AcroLobby>>;

/// Create shared lobby
#[allow(dead_code)]
pub fn create_shared_lobby() -> SharedLobby {
    Arc::new(RwLock::new(AcroLobby::new()))
}

/// Initialize acromania for a user
pub async fn start_acromania(user_id: i64, handle: &str) -> Result<(AcroService, String), String> {
    let service = AcroService::new(user_id, handle.to_string());
    let screen = render_acro_menu();
    Ok((service, screen))
}

/// Render current screen
#[allow(dead_code)]
pub fn render_screen(service: &AcroService, lobby: &AcroLobby) -> String {
    service.render(lobby)
}

/// Render leaderboard with DB data
pub async fn render_leaderboard_screen(db: &AcroDb) -> String {
    match db.get_leaderboard(20).await {
        Ok(entries) => {
            let data: Vec<_> = entries.into_iter()
                .map(|e| (e.handle, e.highest_score, e.games_played, e.games_won))
                .collect();
            render_leaderboard(&data)
        }
        Err(_) => render_leaderboard(&[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = AcroService::new(1, "TestPlayer".to_string());
        assert_eq!(service.screen, AcroScreen::Menu);
        assert_eq!(service.user_id, 1);
    }

    #[test]
    fn test_menu_navigation() {
        let mut service = AcroService::new(1, "Test".to_string());
        let mut lobby = AcroLobby::new();

        // Press H for how to play
        service.input_buffer = "H".to_string();
        let action = service.process_input(&mut lobby);

        assert!(matches!(action, AcroAction::Render(_)));
        assert_eq!(service.screen, AcroScreen::HowToPlay);

        // Press any key to return
        service.input_buffer = " ".to_string();
        let action = service.process_input(&mut lobby);

        assert!(matches!(action, AcroAction::Render(_)));
        assert_eq!(service.screen, AcroScreen::Menu);
    }

    #[test]
    fn test_create_public_lobby() {
        let mut service = AcroService::new(1, "Host".to_string());
        let mut lobby = AcroLobby::new();

        service.input_buffer = "C".to_string();
        let action = service.process_input(&mut lobby);

        assert!(matches!(action, AcroAction::Render(_)));
        assert_eq!(service.screen, AcroScreen::Lobby);
        assert_eq!(lobby.lobbies.len(), 1);
    }

    #[test]
    fn test_quit_action() {
        let mut service = AcroService::new(1, "Test".to_string());
        let mut lobby = AcroLobby::new();

        service.input_buffer = "Q".to_string();
        let action = service.process_input(&mut lobby);

        assert!(matches!(action, AcroAction::Quit));
    }

    #[tokio::test]
    async fn test_start_acromania() {
        let (service, screen) = start_acromania(1, "Test").await.unwrap();
        assert_eq!(service.screen, AcroScreen::Menu);
        assert!(!screen.is_empty());
        assert!(screen.contains("Welcome to Acromania"));
    }
}
