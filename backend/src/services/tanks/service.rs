//! Tanks service - session routing and game coordination
//!
//! Uses __tanks__ sentinel for session routing.
//! Coordinates between game logic, lobby system, and rendering.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::games::tanks::{
    TanksLobby, TanksFlow, TanksAction, TanksScreen,
    render_tanks_menu, render_leaderboard,
};
use crate::services::tanks::db::TanksDb;

/// Sentinel for session routing
pub const SENTINEL: &str = "__tanks__";

/// Per-player service state
pub struct TanksService {
    pub flow: TanksFlow,
}

impl TanksService {
    pub fn new(user_id: i64, handle: String) -> Self {
        Self {
            flow: TanksFlow::new(user_id, handle),
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, lobby: &mut TanksLobby) -> TanksAction {
        self.flow.handle_char(ch, lobby)
    }

    /// Handle escape sequence (arrow keys)
    pub fn handle_escape_sequence(&mut self, seq: &str, lobby: &mut TanksLobby) -> TanksAction {
        self.flow.handle_escape_sequence(seq, lobby)
    }

    /// Get current screen
    pub fn current_screen(&self) -> &TanksScreen {
        &self.flow.screen
    }

    /// Sync with game state changes
    pub fn sync_with_game(&mut self, lobby: &TanksLobby) -> Option<String> {
        if let Some(game) = lobby.get_player_game(self.flow.user_id) {
            self.flow.sync_with_game(game)
        } else {
            None
        }
    }

    /// Render current screen
    pub fn render(&self, lobby: &TanksLobby) -> String {
        self.flow.render(lobby)
    }
}

/// Global lobby manager (shared across all sessions)
pub type SharedLobby = Arc<RwLock<TanksLobby>>;

/// Create shared lobby
pub fn create_shared_lobby() -> SharedLobby {
    Arc::new(RwLock::new(TanksLobby::new()))
}

/// Initialize tanks for a user
pub async fn start_tanks(user_id: i64, handle: &str) -> Result<(TanksService, String), String> {
    let service = TanksService::new(user_id, handle.to_string());
    let screen = render_tanks_menu();
    Ok((service, screen))
}

/// Render current screen
pub fn render_screen(service: &TanksService, lobby: &TanksLobby) -> String {
    service.render(lobby)
}

/// Render leaderboard with DB data
pub async fn render_leaderboard_screen(db: &TanksDb) -> String {
    match db.get_leaderboard(20).await {
        Ok(entries) => {
            let data: Vec<_> = entries.into_iter()
                .map(|e| (e.handle, e.wins, e.kills, e.accuracy))
                .collect();
            render_leaderboard(&data)
        }
        Err(_) => render_leaderboard(&[]),
    }
}

/// Record completed game to database
pub async fn record_game_completion(
    db: &TanksDb,
    lobby: &TanksLobby,
    game_id: u64,
) -> Result<(), String> {
    let game = lobby.active_games.get(&game_id)
        .ok_or("Game not found")?;

    let winner = game.get_winner();
    let winner_user_id = winner.map(|w| w.user_id).unwrap_or(0);
    let winner_handle = winner.map(|w| w.handle.as_str()).unwrap_or("None");

    // Record the game
    let db_game_id = db.record_game(
        game.tanks.len() as u32,
        game.round,
        winner_user_id,
        winner_handle,
        0, // Would need to store terrain seed
    ).await.map_err(|e| e.to_string())?;

    // Record each player
    let mut rank = 1;
    let standings = game.get_standings();
    for (handle, kills, damage, alive) in standings {
        let tank = game.tanks.values()
            .find(|t| t.handle == handle)
            .unwrap();

        db.record_game_player(
            db_game_id,
            tank.user_id,
            &handle,
            rank,
            kills,
            damage,
            tank.shots_fired,
            tank.shots_hit,
            alive,
        ).await.map_err(|e| e.to_string())?;

        db.update_player_stats(
            tank.user_id,
            &handle,
            alive && winner_user_id == tank.user_id,
            kills,
            damage,
            tank.shots_fired,
            tank.shots_hit,
        ).await.map_err(|e| e.to_string())?;

        rank += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = TanksService::new(1, "TestPlayer".to_string());
        assert_eq!(*service.current_screen(), TanksScreen::Menu);
    }

    #[test]
    fn test_menu_navigation() {
        let mut service = TanksService::new(1, "Test".to_string());
        let mut lobby = TanksLobby::new();

        // Press H for how to play
        let action = service.handle_char('H', &mut lobby);
        assert!(matches!(action, TanksAction::Render(_)));
        assert_eq!(*service.current_screen(), TanksScreen::HowToPlay);

        // Press any key to return
        let action = service.handle_char(' ', &mut lobby);
        assert!(matches!(action, TanksAction::Render(_)));
        assert_eq!(*service.current_screen(), TanksScreen::Menu);
    }

    #[test]
    fn test_create_lobby() {
        let mut service = TanksService::new(1, "Host".to_string());
        let mut lobby = TanksLobby::new();

        let action = service.handle_char('C', &mut lobby);
        assert!(matches!(action, TanksAction::Render(_)));
        assert_eq!(*service.current_screen(), TanksScreen::Lobby);
        assert_eq!(lobby.lobbies.len(), 1);
    }

    #[test]
    fn test_quit_action() {
        let mut service = TanksService::new(1, "Test".to_string());
        let mut lobby = TanksLobby::new();

        let action = service.handle_char('Q', &mut lobby);
        assert!(matches!(action, TanksAction::Quit));
    }

    #[tokio::test]
    async fn test_start_tanks() {
        let (service, screen) = start_tanks(1, "Test").await.unwrap();
        assert_eq!(*service.current_screen(), TanksScreen::Menu);
        assert!(!screen.is_empty());
        assert!(screen.contains("TANKS"));
    }

    #[test]
    fn test_shared_lobby() {
        let lobby = create_shared_lobby();
        assert!(lobby.try_read().is_ok());
    }
}
