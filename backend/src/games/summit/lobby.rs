//! Lobby and matchmaking system for Summit
//!
//! Handles game creation, player matching, and invite codes.
//! Supports both public matchmaking and private (friends-only) games.

use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;

use super::mountain::{Mountain, daily_seed, today_date};
use super::state::{RunState, EquippedCosmetics, PlayerStats};

// ============================================================================
// MATCH TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    /// Public game - anyone can join
    Public,
    /// Private game - requires invite code
    Private { invite_code: String },
}

// ============================================================================
// LOBBY PLAYER
// ============================================================================

#[derive(Debug, Clone)]
pub struct LobbyPlayer {
    pub user_id: i64,
    pub handle: String,
    pub joined_at: Instant,
    pub is_ready: bool,
    pub cosmetics: EquippedCosmetics,
    pub stats: PlayerStats,
}

// ============================================================================
// GAME LOBBY
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameLobbyState {
    Waiting,
    Countdown,
    InGame,
    Completed,
}

#[derive(Debug)]
pub struct GameLobby {
    pub game_id: u64,
    pub host_user_id: i64,
    pub match_type: MatchType,
    pub players: Vec<LobbyPlayer>,
    pub state: GameLobbyState,
    pub created_at: Instant,
    pub countdown_started: Option<Instant>,
    pub date: String,
}

impl GameLobby {
    pub fn new(game_id: u64, host_user_id: i64, host_handle: String, match_type: MatchType, stats: PlayerStats) -> Self {
        let host = LobbyPlayer {
            user_id: host_user_id,
            handle: host_handle,
            joined_at: Instant::now(),
            is_ready: true,
            cosmetics: stats.equipped_cosmetics.clone(),
            stats,
        };

        Self {
            game_id,
            host_user_id,
            match_type,
            players: vec![host],
            state: GameLobbyState::Waiting,
            created_at: Instant::now(),
            countdown_started: None,
            date: today_date(),
        }
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= 4
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= 1 && self.players.iter().all(|p| p.is_ready)
    }

    pub fn add_player(&mut self, user_id: i64, handle: String, stats: PlayerStats) -> Result<(), &'static str> {
        if self.is_full() {
            return Err("Lobby is full");
        }

        if self.state != GameLobbyState::Waiting {
            return Err("Game already started");
        }

        if self.players.iter().any(|p| p.user_id == user_id) {
            return Err("Already in lobby");
        }

        self.players.push(LobbyPlayer {
            user_id,
            handle,
            joined_at: Instant::now(),
            is_ready: false,
            cosmetics: stats.equipped_cosmetics.clone(),
            stats,
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

    pub fn get_invite_code(&self) -> Option<&str> {
        match &self.match_type {
            MatchType::Private { invite_code } => Some(invite_code),
            MatchType::Public => None,
        }
    }

    pub fn start_countdown(&mut self) {
        if self.state == GameLobbyState::Waiting && self.can_start() {
            self.state = GameLobbyState::Countdown;
            self.countdown_started = Some(Instant::now());
        }
    }

    pub fn is_countdown_complete(&self) -> bool {
        if let Some(started) = self.countdown_started {
            started.elapsed().as_secs() >= 3
        } else {
            false
        }
    }
}

// ============================================================================
// ACTIVE GAME
// ============================================================================

#[derive(Debug)]
pub struct ActiveGame {
    pub game_id: u64,
    pub run: RunState,
    pub mountain: Mountain,
    pub last_tick: Instant,
}

impl ActiveGame {
    pub fn from_lobby(lobby: &GameLobby) -> Self {
        let date = lobby.date.clone();
        let seed = daily_seed(&date);
        let mountain = Mountain::generate(seed, date.clone());
        let mut run = RunState::new(lobby.game_id, date, seed);

        for player in &lobby.players {
            run.add_climber(player.user_id, player.handle.clone(), player.cosmetics.clone());
        }

        Self {
            game_id: lobby.game_id,
            run,
            mountain,
            last_tick: Instant::now(),
        }
    }

    pub fn should_tick(&self) -> bool {
        self.last_tick.elapsed().as_millis() >= 100  // 10 ticks per second
    }

    pub fn tick(&mut self) {
        self.run.tick();
        self.last_tick = Instant::now();
    }
}

// ============================================================================
// LOBBY MANAGER
// ============================================================================

pub struct SummitLobby {
    pub lobbies: HashMap<u64, GameLobby>,
    pub active_games: HashMap<u64, ActiveGame>,
    /// Map player -> game they're in (lobby or active)
    pub player_games: HashMap<i64, u64>,
    next_game_id: u64,
}

impl SummitLobby {
    pub fn new() -> Self {
        Self {
            lobbies: HashMap::new(),
            active_games: HashMap::new(),
            player_games: HashMap::new(),
            next_game_id: 1,
        }
    }

    /// Create a new public lobby
    pub fn create_public_lobby(&mut self, user_id: i64, handle: String, stats: PlayerStats) -> Result<u64, &'static str> {
        self.create_lobby(user_id, handle, MatchType::Public, stats)
    }

    /// Create a new private lobby with invite code
    pub fn create_private_lobby(&mut self, user_id: i64, handle: String, stats: PlayerStats) -> Result<(u64, String), &'static str> {
        let invite_code = generate_invite_code();
        let match_type = MatchType::Private { invite_code: invite_code.clone() };
        let game_id = self.create_lobby(user_id, handle, match_type, stats)?;
        Ok((game_id, invite_code))
    }

    fn create_lobby(&mut self, user_id: i64, handle: String, match_type: MatchType, stats: PlayerStats) -> Result<u64, &'static str> {
        // Check if player is already in a game
        if self.player_games.contains_key(&user_id) {
            return Err("Already in a game");
        }

        let game_id = self.next_game_id;
        self.next_game_id += 1;

        let lobby = GameLobby::new(game_id, user_id, handle, match_type, stats);
        self.lobbies.insert(game_id, lobby);
        self.player_games.insert(user_id, game_id);

        Ok(game_id)
    }

    /// Join a public lobby (first available or specific)
    pub fn join_public_lobby(&mut self, user_id: i64, handle: String, stats: PlayerStats, game_id: Option<u64>) -> Result<u64, &'static str> {
        if self.player_games.contains_key(&user_id) {
            return Err("Already in a game");
        }

        let target_id = if let Some(id) = game_id {
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
                        && lobby.state == GameLobbyState::Waiting
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

        lobby.add_player(user_id, handle, stats)?;
        self.player_games.insert(user_id, target_id);

        Ok(target_id)
    }

    /// Join a private lobby by invite code
    pub fn join_by_invite(&mut self, user_id: i64, handle: String, stats: PlayerStats, invite_code: &str) -> Result<u64, &'static str> {
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
        lobby.add_player(user_id, handle, stats)?;
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

            // If host left or empty, close lobby
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
            if let Some(climber) = game.run.get_climber_mut(user_id) {
                climber.disconnect();
            }

            // Clean up if all disconnected
            let all_disconnected = game.run.climbers.values()
                .all(|c| c.status == super::state::ClimberStatus::Disconnected);

            if all_disconnected {
                self.cleanup_game(game_id);
            }

            return Ok(());
        }

        Ok(())
    }

    /// Rejoin an active game after disconnect
    pub fn rejoin_game(&mut self, user_id: i64) -> Result<u64, &'static str> {
        let game_id = *self.player_games.get(&user_id)
            .ok_or("Not in a game")?;

        if let Some(game) = self.active_games.get_mut(&game_id) {
            if let Some(climber) = game.run.get_climber_mut(user_id) {
                climber.reconnect();
                return Ok(game_id);
            }
        }

        Err("Cannot rejoin")
    }

    /// Set player ready status
    pub fn set_ready(&mut self, user_id: i64, ready: bool) -> Result<(), &'static str> {
        let game_id = *self.player_games.get(&user_id)
            .ok_or("Not in a game")?;

        if let Some(lobby) = self.lobbies.get_mut(&game_id) {
            lobby.set_ready(user_id, ready);
            return Ok(());
        }

        Err("Not in lobby")
    }

    /// Start the game (host only)
    pub fn start_game(&mut self, user_id: i64) -> Result<u64, &'static str> {
        let game_id = *self.player_games.get(&user_id)
            .ok_or("Not in a game")?;

        let lobby = self.lobbies.get_mut(&game_id)
            .ok_or("Lobby not found")?;

        if lobby.host_user_id != user_id {
            return Err("Only host can start");
        }

        if !lobby.can_start() {
            return Err("Not all players ready");
        }

        // Start countdown
        lobby.start_countdown();

        Ok(game_id)
    }

    /// Transition lobby to active game when countdown completes
    pub fn transition_to_game(&mut self, game_id: u64) -> Result<(), &'static str> {
        let lobby = self.lobbies.remove(&game_id)
            .ok_or("Lobby not found")?;

        if !lobby.is_countdown_complete() {
            // Put it back
            self.lobbies.insert(game_id, lobby);
            return Err("Countdown not complete");
        }

        let game = ActiveGame::from_lobby(&lobby);
        self.active_games.insert(game_id, game);

        Ok(())
    }

    /// Get player's current lobby
    pub fn get_player_lobby(&self, user_id: i64) -> Option<&GameLobby> {
        let game_id = self.player_games.get(&user_id)?;
        self.lobbies.get(game_id)
    }

    /// Get player's current active game
    pub fn get_player_game(&self, user_id: i64) -> Option<&ActiveGame> {
        let game_id = self.player_games.get(&user_id)?;
        self.active_games.get(game_id)
    }

    /// Get player's current active game (mutable)
    pub fn get_player_game_mut(&mut self, user_id: i64) -> Option<&mut ActiveGame> {
        let game_id = *self.player_games.get(&user_id)?;
        self.active_games.get_mut(&game_id)
    }

    /// List available public lobbies
    pub fn list_public_lobbies(&self) -> Vec<(u64, usize, usize)> {
        self.lobbies.iter()
            .filter(|(_, lobby)| {
                matches!(lobby.match_type, MatchType::Public)
                    && lobby.state == GameLobbyState::Waiting
            })
            .map(|(id, lobby)| (*id, lobby.player_count(), 4)) // Max 4 players
            .collect()
    }

    /// Cleanup completed or abandoned game
    pub fn cleanup_game(&mut self, game_id: u64) {
        if let Some(game) = self.active_games.remove(&game_id) {
            for user_id in game.run.climbers.keys() {
                self.player_games.remove(user_id);
            }
        }

        if let Some(lobby) = self.lobbies.remove(&game_id) {
            for player in &lobby.players {
                self.player_games.remove(&player.user_id);
            }
        }
    }

    /// Tick all active games
    pub fn tick_all(&mut self) {
        let mut completed = Vec::new();

        for (game_id, game) in self.active_games.iter_mut() {
            if game.should_tick() {
                game.tick();

                // Check for completion
                if game.run.status != super::state::RunStatus::Active {
                    completed.push(*game_id);
                }
            }
        }

        // Don't immediately cleanup - let players see results
        // Cleanup happens when all players leave
    }

    /// Check if player is in a game
    pub fn is_player_in_game(&self, user_id: i64) -> bool {
        self.player_games.contains_key(&user_id)
    }
}

impl Default for SummitLobby {
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

    fn test_stats() -> PlayerStats {
        PlayerStats::new(1)
    }

    #[test]
    fn test_create_public_lobby() {
        let mut manager = SummitLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string(), test_stats()).unwrap();

        assert!(manager.lobbies.contains_key(&game_id));
        assert!(manager.player_games.contains_key(&1));
    }

    #[test]
    fn test_create_private_lobby() {
        let mut manager = SummitLobby::new();
        let (game_id, code) = manager.create_private_lobby(1, "Host".to_string(), test_stats()).unwrap();

        assert_eq!(code.len(), 6);
        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert!(matches!(lobby.match_type, MatchType::Private { .. }));
    }

    #[test]
    fn test_join_public_lobby() {
        let mut manager = SummitLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string(), test_stats()).unwrap();

        let mut stats2 = test_stats();
        stats2.user_id = 2;
        let joined_id = manager.join_public_lobby(2, "Player2".to_string(), stats2, None).unwrap();
        assert_eq!(game_id, joined_id);

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert_eq!(lobby.player_count(), 2);
    }

    #[test]
    fn test_join_by_invite() {
        let mut manager = SummitLobby::new();
        let (game_id, code) = manager.create_private_lobby(1, "Host".to_string(), test_stats()).unwrap();

        let mut stats2 = test_stats();
        stats2.user_id = 2;
        let joined_id = manager.join_by_invite(2, "Player2".to_string(), stats2, &code).unwrap();
        assert_eq!(game_id, joined_id);
    }

    #[test]
    fn test_cannot_join_wrong_invite() {
        let mut manager = SummitLobby::new();
        manager.create_private_lobby(1, "Host".to_string(), test_stats()).unwrap();

        let mut stats2 = test_stats();
        stats2.user_id = 2;
        let result = manager.join_by_invite(2, "Player2".to_string(), stats2, "WRONG1");
        assert!(result.is_err());
    }

    #[test]
    fn test_leave_lobby() {
        let mut manager = SummitLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string(), test_stats()).unwrap();

        let mut stats2 = test_stats();
        stats2.user_id = 2;
        manager.join_public_lobby(2, "Player2".to_string(), stats2, None).unwrap();

        manager.leave_game(2).unwrap();

        let lobby = manager.lobbies.get(&game_id).unwrap();
        assert_eq!(lobby.player_count(), 1);
        assert!(!manager.player_games.contains_key(&2));
    }

    #[test]
    fn test_host_leave_closes_lobby() {
        let mut manager = SummitLobby::new();
        let game_id = manager.create_public_lobby(1, "Host".to_string(), test_stats()).unwrap();

        let mut stats2 = test_stats();
        stats2.user_id = 2;
        manager.join_public_lobby(2, "Player2".to_string(), stats2, None).unwrap();

        manager.leave_game(1).unwrap();

        assert!(!manager.lobbies.contains_key(&game_id));
        assert!(!manager.player_games.contains_key(&1));
    }

    #[test]
    fn test_lobby_max_4_players() {
        let mut manager = SummitLobby::new();
        manager.create_public_lobby(1, "Host".to_string(), test_stats()).unwrap();

        for i in 2..=4 {
            let mut stats = test_stats();
            stats.user_id = i;
            manager.join_public_lobby(i, format!("Player{}", i), stats, None).unwrap();
        }

        // 5th player should fail
        let mut stats5 = test_stats();
        stats5.user_id = 5;
        let result = manager.join_public_lobby(5, "Player5".to_string(), stats5, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_public_lobbies() {
        let mut manager = SummitLobby::new();
        let mut stats1 = test_stats();
        stats1.user_id = 1;
        let mut stats2 = test_stats();
        stats2.user_id = 2;
        let mut stats3 = test_stats();
        stats3.user_id = 3;

        manager.create_public_lobby(1, "Host1".to_string(), stats1).unwrap();
        manager.create_public_lobby(2, "Host2".to_string(), stats2).unwrap();
        manager.create_private_lobby(3, "Host3".to_string(), stats3).unwrap();

        let lobbies = manager.list_public_lobbies();
        assert_eq!(lobbies.len(), 2); // Only public ones
    }

    #[test]
    fn test_generate_invite_code() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_ready_up() {
        let mut manager = SummitLobby::new();
        manager.create_public_lobby(1, "Host".to_string(), test_stats()).unwrap();

        let mut stats2 = test_stats();
        stats2.user_id = 2;
        manager.join_public_lobby(2, "Player2".to_string(), stats2, None).unwrap();

        // Player 2 starts not ready
        let lobby = manager.get_player_lobby(2).unwrap();
        assert!(!lobby.players[1].is_ready);

        // Set ready
        manager.set_ready(2, true).unwrap();
        let lobby = manager.get_player_lobby(2).unwrap();
        assert!(lobby.players[1].is_ready);
    }

    #[test]
    fn test_active_game_from_lobby() {
        let lobby = GameLobby::new(
            1,
            1,
            "Host".to_string(),
            MatchType::Public,
            test_stats(),
        );

        let game = ActiveGame::from_lobby(&lobby);

        assert_eq!(game.game_id, 1);
        assert_eq!(game.run.climbers.len(), 1);
        assert!(game.run.get_climber(1).is_some());
    }
}
