//! Lobby and matchmaking system for Depths of Diablo
//!
//! Handles game creation, player matching, invite codes, and co-op sessions.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use super::dungeon::daily_seed;
use super::state::GameState;

/// Type of matchmaking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
    /// Public game - anyone can join
    Public,
    /// Private game - requires invite code
    Private { invite_code: String },
    /// Solo game - no matchmaking
    Solo,
}

/// State of a lobby
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyState {
    WaitingForPlayers,
    Countdown,
    InProgress,
    Completed,
}

/// A player in the lobby
#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub user_id: i64,
    pub handle: String,
    pub joined_at: Instant,
    pub is_ready: bool,
    pub is_host: bool,
    pub selected_class: Option<String>,
}

/// A game lobby
#[derive(Debug)]
pub struct GameLobby {
    pub game_id: u64,
    pub match_type: MatchType,
    pub players: Vec<LobbyPlayer>,
    pub state: LobbyState,
    pub seed: u64,
    pub created_at: Instant,
    pub countdown_started: Option<Instant>,
    pub max_players: usize,
    pub min_players: usize,
}

impl GameLobby {
    pub fn new(game_id: u64, host_user_id: i64, host_handle: String, match_type: MatchType) -> Self {
        let host = LobbyPlayer {
            user_id: host_user_id,
            handle: host_handle,
            joined_at: Instant::now(),
            is_ready: true,
            is_host: true,
            selected_class: None,
        };

        GameLobby {
            game_id,
            match_type,
            players: vec![host],
            state: LobbyState::WaitingForPlayers,
            seed: daily_seed(),
            created_at: Instant::now(),
            countdown_started: None,
            max_players: 4,
            min_players: 1,
        }
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= self.min_players
            && self.players.iter().all(|p| p.is_ready && p.selected_class.is_some())
    }

    pub fn host_id(&self) -> Option<i64> {
        self.players.iter().find(|p| p.is_host).map(|p| p.user_id)
    }

    pub fn add_player(&mut self, user_id: i64, handle: String) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("Lobby is full");
        }

        if self.state != LobbyState::WaitingForPlayers {
            return Err("Game already in progress");
        }

        if self.players.iter().any(|p| p.user_id == user_id) {
            return Err("Already in lobby");
        }

        self.players.push(LobbyPlayer {
            user_id,
            handle,
            joined_at: Instant::now(),
            is_ready: false,
            is_host: false,
            selected_class: None,
        });

        Ok(())
    }

    pub fn remove_player(&mut self, user_id: i64) -> bool {
        let was_host = self.players.iter().find(|p| p.user_id == user_id)
            .map(|p| p.is_host)
            .unwrap_or(false);

        self.players.retain(|p| p.user_id != user_id);

        // If host left, promote next player
        if was_host && !self.players.is_empty() {
            self.players[0].is_host = true;
        }

        was_host
    }

    pub fn set_ready(&mut self, user_id: i64, ready: bool) -> bool {
        if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
            player.is_ready = ready;
            true
        } else {
            false
        }
    }

    pub fn select_class(&mut self, user_id: i64, class: &str) -> bool {
        if let Some(player) = self.players.iter_mut().find(|p| p.user_id == user_id) {
            player.selected_class = Some(class.to_string());
            true
        } else {
            false
        }
    }

    pub fn get_invite_code(&self) -> Option<&str> {
        match &self.match_type {
            MatchType::Private { invite_code } => Some(invite_code),
            _ => None,
        }
    }

    pub fn start_game(&mut self) -> Result<(), &'static str> {
        if !self.can_start() {
            return Err("Not all players ready or class selected");
        }

        self.state = LobbyState::InProgress;
        Ok(())
    }

    pub fn get_player(&self, user_id: i64) -> Option<&LobbyPlayer> {
        self.players.iter().find(|p| p.user_id == user_id)
    }
}

/// Active co-op game session
#[derive(Debug)]
pub struct CoopSession {
    pub game_id: u64,
    pub seed: u64,
    pub current_floor: u32,
    pub players: HashMap<i64, GameState>,
    pub started_at: Instant,
    pub last_update: Instant,
}

impl CoopSession {
    pub fn new(game_id: u64, seed: u64) -> Self {
        CoopSession {
            game_id,
            seed,
            current_floor: 1,
            players: HashMap::new(),
            started_at: Instant::now(),
            last_update: Instant::now(),
        }
    }

    pub fn add_player(&mut self, state: GameState) {
        self.players.insert(state.user_id, state);
    }

    pub fn remove_player(&mut self, user_id: i64) -> Option<GameState> {
        self.players.remove(&user_id)
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn get_player(&self, user_id: i64) -> Option<&GameState> {
        self.players.get(&user_id)
    }

    pub fn get_player_mut(&mut self, user_id: i64) -> Option<&mut GameState> {
        self.players.get_mut(&user_id)
    }

    pub fn all_players_dead(&self) -> bool {
        self.players.values().all(|state| {
            state.character.as_ref().map(|c| !c.is_alive()).unwrap_or(true)
        })
    }

    pub fn alive_player_count(&self) -> usize {
        self.players.values().filter(|state| {
            state.character.as_ref().map(|c| c.is_alive()).unwrap_or(false)
        }).count()
    }
}

/// Main lobby manager for all Depths of Diablo games
pub struct DiabloLobby {
    pub lobbies: HashMap<u64, GameLobby>,
    pub active_sessions: HashMap<u64, CoopSession>,
    pub player_games: HashMap<i64, u64>,
    next_game_id: u64,
}

impl DiabloLobby {
    pub fn new() -> Self {
        DiabloLobby {
            lobbies: HashMap::new(),
            active_sessions: HashMap::new(),
            player_games: HashMap::new(),
            next_game_id: 1,
        }
    }

    /// Create a new public lobby
    pub fn create_public_lobby(&mut self, host_user_id: i64, host_handle: String) -> Result<u64, &'static str> {
        self.create_lobby(host_user_id, host_handle, MatchType::Public)
    }

    /// Create a new private lobby with invite code
    pub fn create_private_lobby(&mut self, host_user_id: i64, host_handle: String) -> Result<(u64, String), &'static str> {
        let invite_code = generate_invite_code();
        let match_type = MatchType::Private { invite_code: invite_code.clone() };
        let game_id = self.create_lobby(host_user_id, host_handle, match_type)?;
        Ok((game_id, invite_code))
    }

    /// Create a solo game
    pub fn create_solo_game(&mut self, user_id: i64, handle: String) -> Result<u64, &'static str> {
        self.create_lobby(user_id, handle, MatchType::Solo)
    }

    fn create_lobby(&mut self, host_user_id: i64, host_handle: String, match_type: MatchType) -> Result<u64, &'static str> {
        if self.player_games.contains_key(&host_user_id) {
            return Err("Already in a game");
        }

        let game_id = self.next_game_id;
        self.next_game_id += 1;

        let lobby = GameLobby::new(game_id, host_user_id, host_handle, match_type);
        self.lobbies.insert(game_id, lobby);
        self.player_games.insert(host_user_id, game_id);

        Ok(game_id)
    }

    /// Join a public lobby
    pub fn join_public_lobby(&mut self, user_id: i64, handle: String, game_id: Option<u64>) -> Result<u64, &'static str> {
        if self.player_games.contains_key(&user_id) {
            return Err("Already in a game");
        }

        let target_id = if let Some(id) = game_id {
            id
        } else {
            // Find first available public lobby
            self.lobbies.iter()
                .filter(|(_, lobby)| {
                    matches!(lobby.match_type, MatchType::Public)
                        && !lobby.is_full()
                        && lobby.state == LobbyState::WaitingForPlayers
                })
                .map(|(id, _)| *id)
                .next()
                .ok_or("No available games")?
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
            let was_host = lobby.remove_player(user_id);

            // If host left or lobby empty, close it
            if was_host || lobby.players.is_empty() {
                for player in &lobby.players {
                    self.player_games.remove(&player.user_id);
                }
                self.lobbies.remove(&game_id);
            }

            return Ok(());
        }

        // Check if in active session
        if let Some(session) = self.active_sessions.get_mut(&game_id) {
            session.remove_player(user_id);

            // If no players left, close session
            if session.player_count() == 0 {
                self.active_sessions.remove(&game_id);
            }

            return Ok(());
        }

        Ok(())
    }

    /// Start a lobby's game
    pub fn start_game(&mut self, user_id: i64) -> Result<u64, &'static str> {
        let game_id = *self.player_games.get(&user_id)
            .ok_or("Not in a game")?;

        let lobby = self.lobbies.get(&game_id)
            .ok_or("Lobby not found")?;

        if lobby.host_id() != Some(user_id) {
            return Err("Only host can start");
        }

        if !lobby.can_start() {
            return Err("Not all players ready");
        }

        // Convert lobby to active session
        let lobby = self.lobbies.remove(&game_id).unwrap();
        let mut session = CoopSession::new(game_id, lobby.seed);

        // Create game states for each player
        for player in &lobby.players {
            let state = GameState::new(player.user_id, &player.handle);
            // Start run will be called when player enters
            session.add_player(state);
        }

        self.active_sessions.insert(game_id, session);

        Ok(game_id)
    }

    /// Get player's current lobby
    pub fn get_player_lobby(&self, user_id: i64) -> Option<&GameLobby> {
        let game_id = self.player_games.get(&user_id)?;
        self.lobbies.get(game_id)
    }

    /// Get player's current lobby (mutable)
    pub fn get_player_lobby_mut(&mut self, user_id: i64) -> Option<&mut GameLobby> {
        let game_id = *self.player_games.get(&user_id)?;
        self.lobbies.get_mut(&game_id)
    }

    /// Get player's active session
    pub fn get_player_session(&self, user_id: i64) -> Option<&CoopSession> {
        let game_id = self.player_games.get(&user_id)?;
        self.active_sessions.get(game_id)
    }

    /// Get player's active session (mutable)
    pub fn get_player_session_mut(&mut self, user_id: i64) -> Option<&mut CoopSession> {
        let game_id = *self.player_games.get(&user_id)?;
        self.active_sessions.get_mut(&game_id)
    }

    /// Get available public lobbies
    pub fn list_public_lobbies(&self) -> Vec<(u64, usize, usize)> {
        self.lobbies.iter()
            .filter(|(_, lobby)| matches!(lobby.match_type, MatchType::Public))
            .map(|(id, lobby)| (*id, lobby.player_count(), lobby.max_players))
            .collect()
    }

    /// Check if player is in a game
    pub fn is_player_in_game(&self, user_id: i64) -> bool {
        self.player_games.contains_key(&user_id)
    }

    /// Get player's game ID
    pub fn get_player_game_id(&self, user_id: i64) -> Option<u64> {
        self.player_games.get(&user_id).copied()
    }
}

impl Default for DiabloLobby {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random 6-character invite code
fn generate_invite_code() -> String {
    let mut rng = rand::thread_rng();
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_public_lobby() {
        let mut manager = DiabloLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();

        assert!(manager.lobbies.contains_key(&game_id));
        assert!(manager.player_games.contains_key(&1));
    }

    #[test]
    fn test_create_private_lobby() {
        let mut manager = DiabloLobby::new();
        let (game_id, code) = manager.create_private_lobby(1, "Host".to_string()).unwrap();

        assert_eq!(code.len(), 6);
        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert!(matches!(lobby.match_type, MatchType::Private { .. }));
    }

    #[test]
    fn test_join_public_lobby() {
        let mut manager = DiabloLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();

        let joined_id = manager.join_public_lobby(2, "Player2".to_string(), None).unwrap();
        assert_eq!(game_id, joined_id);

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert_eq!(lobby.player_count(), 2);
    }

    #[test]
    fn test_join_by_invite() {
        let mut manager = DiabloLobby::new();
        let (game_id, code) = manager.create_private_lobby(1, "Host".to_string()).unwrap();

        let joined_id = manager.join_by_invite(2, "Player2".to_string(), &code).unwrap();
        assert_eq!(game_id, joined_id);
    }

    #[test]
    fn test_leave_lobby() {
        let mut manager = DiabloLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();
        manager.join_public_lobby(2, "Player2".to_string(), None).unwrap();

        manager.leave_game(2).unwrap();

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert_eq!(lobby.player_count(), 1);
    }

    #[test]
    fn test_host_leave_closes_lobby() {
        let mut manager = DiabloLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string()).unwrap();

        manager.leave_game(1).unwrap();

        assert!(!manager.lobbies.contains_key(&game_id));
    }

    #[test]
    fn test_solo_game() {
        let mut manager = DiabloLobby::new();
        let game_id = manager.create_solo_game(1, "Solo".to_string()).unwrap();

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert!(matches!(lobby.match_type, MatchType::Solo));
    }

    #[test]
    fn test_lobby_can_start() {
        let mut manager = DiabloLobby::new();
        let game_id = manager.create_solo_game(1, "Solo".to_string()).unwrap();

        let lobby = manager.lobbies.get_mut(&game_id).unwrap();

        // Not ready until class selected
        assert!(!lobby.can_start());

        lobby.select_class(1, "Warrior");
        assert!(lobby.can_start());
    }
}
