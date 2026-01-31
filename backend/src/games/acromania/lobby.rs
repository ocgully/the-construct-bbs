//! Lobby and matchmaking system for Acromania
//!
//! Handles game creation, player matching, and invite codes.

use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;

use super::game::{AcroGame, GameConfig, GamePhase};

/// Type of matchmaking
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    /// Public game - anyone can join
    Public,
    /// Private game - requires invite code
    Private { invite_code: String },
}

/// A player waiting in the lobby
#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub user_id: i64,
    pub handle: String,
    #[allow(dead_code)]
    pub joined_at: Instant,
    pub is_ready: bool,
}

/// A game lobby waiting for players
#[derive(Debug)]
pub struct GameLobby {
    #[allow(dead_code)]
    pub game_id: u64,
    pub host_user_id: i64,
    pub match_type: MatchType,
    pub players: Vec<LobbyPlayer>,
    pub config: GameConfig,
    #[allow(dead_code)]
    pub created_at: Instant,
    pub countdown_started: Option<Instant>,
}

impl GameLobby {
    pub fn new(game_id: u64, host_user_id: i64, host_handle: String, match_type: MatchType, config: GameConfig) -> Self {
        let host = LobbyPlayer {
            user_id: host_user_id,
            handle: host_handle,
            joined_at: Instant::now(),
            is_ready: true,
        };

        Self {
            game_id,
            host_user_id,
            match_type,
            players: vec![host],
            config,
            created_at: Instant::now(),
            countdown_started: None,
        }
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.config.max_players
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= self.config.min_players
            && self.players.iter().all(|p| p.is_ready)
    }

    pub fn add_player(&mut self, user_id: i64, handle: String) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("Lobby is full");
        }

        if self.players.iter().any(|p| p.user_id == user_id) {
            return Err("Already in lobby");
        }

        self.players.push(LobbyPlayer {
            user_id,
            handle,
            joined_at: Instant::now(),
            is_ready: false,
        });

        Ok(())
    }

    pub fn remove_player(&mut self, user_id: i64) -> bool {
        let original_len = self.players.len();
        self.players.retain(|p| p.user_id != user_id);
        self.players.len() != original_len
    }

    pub fn set_ready(&mut self, user_id: i64, ready: bool) {
        if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
            player.is_ready = ready;
        }
    }

    #[allow(dead_code)]
    pub fn get_invite_code(&self) -> Option<&str> {
        match &self.match_type {
            MatchType::Private { invite_code } => Some(invite_code),
            MatchType::Public => None,
        }
    }
}

/// Main lobby manager for all Acromania games
pub struct AcroLobby {
    pub lobbies: HashMap<u64, GameLobby>,
    pub active_games: HashMap<u64, AcroGame>,
    /// Map player -> game they're in (for quick lookup)
    pub player_games: HashMap<i64, u64>,
    next_game_id: u64,
}

impl AcroLobby {
    pub fn new() -> Self {
        Self {
            lobbies: HashMap::new(),
            active_games: HashMap::new(),
            player_games: HashMap::new(),
            next_game_id: 1,
        }
    }

    /// Create a new public lobby
    pub fn create_public_lobby(&mut self, host_user_id: i64, host_handle: String) -> Result<u64, &'static str> {
        self.create_lobby(host_user_id, host_handle, MatchType::Public, GameConfig::default())
    }

    /// Create a new private lobby with invite code
    pub fn create_private_lobby(&mut self, host_user_id: i64, host_handle: String) -> Result<(u64, String), &'static str> {
        let invite_code = generate_invite_code();
        let match_type = MatchType::Private { invite_code: invite_code.clone() };
        let game_id = self.create_lobby(host_user_id, host_handle, match_type, GameConfig::default())?;
        Ok((game_id, invite_code))
    }

    fn create_lobby(&mut self, host_user_id: i64, host_handle: String, match_type: MatchType, config: GameConfig) -> Result<u64, &'static str> {
        // Check if player is already in a game
        if self.player_games.contains_key(&host_user_id) {
            return Err("Already in a game");
        }

        let game_id = self.next_game_id;
        self.next_game_id += 1;

        let lobby = GameLobby::new(game_id, host_user_id, host_handle, match_type, config);
        self.lobbies.insert(game_id, lobby);
        self.player_games.insert(host_user_id, game_id);

        Ok(game_id)
    }

    /// Join a public lobby (first available or specific)
    pub fn join_public_lobby(&mut self, user_id: i64, handle: String, game_id: Option<u64>) -> Result<u64, &'static str> {
        if self.player_games.contains_key(&user_id) {
            return Err("Already in a game");
        }

        let target_id = if let Some(id) = game_id {
            // Join specific game
            if !self.lobbies.contains_key(&id) {
                return Err("Game not found");
            }
            id
        } else {
            // Find first available public lobby
            let available = self.lobbies.iter()
                .filter(|(_, lobby)| {
                    matches!(lobby.match_type, MatchType::Public)
                        && !lobby.is_full()
                        && lobby.countdown_started.is_none()
                })
                .map(|(id, _)| *id)
                .next();

            match available {
                Some(id) => id,
                None => return Err("No available games"),
            }
        };

        let lobby = self.lobbies.get_mut(&target_id).ok_or("Game not found")?;

        if !matches!(lobby.match_type, MatchType::Public) {
            return Err("Game is private");
        }

        lobby.add_player(user_id, handle)?;
        self.player_games.insert(user_id, target_id);

        Ok(target_id)
    }

    /// Join a private lobby by invite code
    pub fn join_by_invite(&mut self, user_id: i64, handle: String, invite_code: &str) -> Result<u64, &'static str> {
        if self.player_games.contains_key(&user_id) {
            return Err("Already in a game");
        }

        // Find lobby with matching invite code
        let game_id = self.lobbies.iter()
            .find(|(_, lobby)| {
                if let MatchType::Private { invite_code: code } = &lobby.match_type {
                    code.eq_ignore_ascii_case(invite_code)
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .ok_or("Invalid invite code")?;

        let lobby = self.lobbies.get_mut(&game_id).ok_or("Game not found")?;
        lobby.add_player(user_id, handle)?;
        self.player_games.insert(user_id, game_id);

        Ok(game_id)
    }

    /// Leave current game/lobby
    pub fn leave_game(&mut self, user_id: i64) -> Result<(), &'static str> {
        let game_id = self.player_games.remove(&user_id)
            .ok_or("Not in a game")?;

        // Check if in lobby
        if let Some(lobby) = self.lobbies.get_mut(&game_id) {
            lobby.remove_player(user_id);

            // If host left, close lobby
            if lobby.host_user_id == user_id || lobby.players.is_empty() {
                // Remove all players from the lobby
                for player in &lobby.players {
                    self.player_games.remove(&player.user_id);
                }
                self.lobbies.remove(&game_id);
            }

            return Ok(());
        }

        // Check if in active game
        if let Some(game) = self.active_games.get_mut(&game_id) {
            game.disconnect_player(user_id);

            // Clean up ended games
            if game.phase == GamePhase::Ended {
                self.cleanup_game(game_id);
            }

            return Ok(());
        }

        Ok(())
    }

    /// Start a lobby's game (host only)
    pub fn start_game(&mut self, user_id: i64) -> Result<u64, &'static str> {
        let game_id = *self.player_games.get(&user_id)
            .ok_or("Not in a game")?;

        let lobby = self.lobbies.get(&game_id)
            .ok_or("Lobby not found")?;

        if lobby.host_user_id != user_id {
            return Err("Only host can start");
        }

        if !lobby.can_start() {
            return Err("Not enough players or not all ready");
        }

        // Convert lobby to active game
        let lobby = self.lobbies.remove(&game_id).unwrap();
        let mut game = AcroGame::new(game_id, lobby.host_user_id, lobby.config);

        for player in lobby.players {
            let _ = game.add_player(player.user_id, player.handle);
        }

        game.start()?;
        self.active_games.insert(game_id, game);

        Ok(game_id)
    }

    /// Get a player's current game
    pub fn get_player_game(&self, user_id: i64) -> Option<&AcroGame> {
        let game_id = self.player_games.get(&user_id)?;
        self.active_games.get(game_id)
    }

    /// Get a player's current game (mutable)
    pub fn get_player_game_mut(&mut self, user_id: i64) -> Option<&mut AcroGame> {
        let game_id = *self.player_games.get(&user_id)?;
        self.active_games.get_mut(&game_id)
    }

    /// Get a player's current lobby
    #[allow(dead_code)]
    pub fn get_player_lobby(&self, user_id: i64) -> Option<&GameLobby> {
        let game_id = self.player_games.get(&user_id)?;
        self.lobbies.get(game_id)
    }

    /// Get available public lobbies
    pub fn list_public_lobbies(&self) -> Vec<(u64, usize, usize)> {
        self.lobbies.iter()
            .filter(|(_, lobby)| matches!(lobby.match_type, MatchType::Public))
            .map(|(id, lobby)| (*id, lobby.player_count(), lobby.config.max_players))
            .collect()
    }

    /// Cleanup an ended game
    pub fn cleanup_game(&mut self, game_id: u64) {
        if let Some(game) = self.active_games.remove(&game_id) {
            for user_id in game.players.keys() {
                self.player_games.remove(user_id);
            }
        }
    }

    /// Tick all active games
    pub fn tick_all(&mut self) -> Vec<(u64, GamePhase)> {
        let mut transitions = Vec::new();

        for (id, game) in self.active_games.iter_mut() {
            if let Some(new_phase) = game.tick() {
                transitions.push((*id, new_phase.clone()));

                if new_phase == GamePhase::Ended {
                    // Cleanup will happen separately
                }
            }
        }

        transitions
    }
}

impl Default for AcroLobby {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random 6-character invite code
fn generate_invite_code() -> String {
    let mut rng = rand::thread_rng();
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // No I, O, 0, 1

    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_public_lobby() {
        let mut manager = AcroLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();

        assert!(manager.lobbies.contains_key(&game_id));
        assert!(manager.player_games.contains_key(&1));
    }

    #[test]
    fn test_create_private_lobby() {
        let mut manager = AcroLobby::new();
        let (game_id, code) = manager.create_private_lobby(1, "Host".to_string()).unwrap();

        assert_eq!(code.len(), 6);
        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert!(matches!(lobby.match_type, MatchType::Private { .. }));
    }

    #[test]
    fn test_join_public_lobby() {
        let mut manager = AcroLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();

        let joined_id = manager.join_public_lobby(2, "Player2".to_string(), None).unwrap();
        assert_eq!(game_id, joined_id);

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert_eq!(lobby.player_count(), 2);
    }

    #[test]
    fn test_join_by_invite() {
        let mut manager = AcroLobby::new();
        let (game_id, code) = manager.create_private_lobby(1, "Host".to_string()).unwrap();

        let joined_id = manager.join_by_invite(2, "Player2".to_string(), &code).unwrap();
        assert_eq!(game_id, joined_id);
    }

    #[test]
    fn test_cannot_join_wrong_invite() {
        let mut manager = AcroLobby::new();
        manager.create_private_lobby(1, "Host".to_string()).unwrap();

        let result = manager.join_by_invite(2, "Player2".to_string(), "WRONG1");
        assert!(result.is_err());
    }

    #[test]
    fn test_leave_lobby() {
        let mut manager = AcroLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();
        manager.join_public_lobby(2, "Player2".to_string(), None).unwrap();

        manager.leave_game(2).unwrap();

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert_eq!(lobby.player_count(), 1);
        assert!(!manager.player_games.contains_key(&2));
    }

    #[test]
    fn test_host_leave_closes_lobby() {
        let mut manager = AcroLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();
        manager.join_public_lobby(2, "Player2".to_string(), None).unwrap();

        manager.leave_game(1).unwrap();

        assert!(!manager.lobbies.contains_key(&game_id));
        assert!(!manager.player_games.contains_key(&1));
        // Other players should also be removed
    }

    #[test]
    fn test_player_already_in_game() {
        let mut manager = AcroLobby::new();
        manager.create_public_lobby(1, "Host".to_string()).unwrap();

        // Try to create another lobby
        let result = manager.create_public_lobby(1, "Host".to_string());
        assert!(result.is_err());

        // Try to join another lobby
        manager.create_public_lobby(2, "Host2".to_string()).unwrap();
        let result = manager.join_public_lobby(1, "Host".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_public_lobbies() {
        let mut manager = AcroLobby::new();
        manager.create_public_lobby(1, "Host1".to_string()).unwrap();
        manager.create_public_lobby(2, "Host2".to_string()).unwrap();
        manager.create_private_lobby(3, "Host3".to_string()).unwrap();

        let lobbies = manager.list_public_lobbies();
        assert_eq!(lobbies.len(), 2); // Only public ones
    }

    #[test]
    fn test_generate_invite_code() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
