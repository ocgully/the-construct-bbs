//! Core game logic for Acromania
//!
//! Manages the game state machine, round progression, and player interactions.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::data::{acronym_length_for_round, generate_acronym, random_category, Category};
use super::profanity::ProfanityFilter;
use super::scoring::{calculate_round_results, RoundResult};
use super::state::{PlayerState, Submission};

/// Validate that submission matches acronym
pub fn validate_submission(text: &str, acronym: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    let letters: Vec<char> = acronym.chars().collect();

    // Must have same number of words as letters
    if words.len() != letters.len() {
        return false;
    }

    // Each word must start with the correct letter
    for (word, letter) in words.iter().zip(letters.iter()) {
        if let Some(first_char) = word.chars().next() {
            if first_char.to_ascii_uppercase() != letter.to_ascii_uppercase() {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

/// Game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub total_rounds: u32,
    pub submission_time_secs: u64,
    pub voting_time_secs: u64,
    pub results_time_secs: u64,
    #[allow(dead_code)]
    pub lobby_time_secs: u64,
    pub min_players: usize,
    pub max_players: usize,
    pub profanity_filter: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            total_rounds: 10,
            submission_time_secs: 60,
            voting_time_secs: 30,
            results_time_secs: 10,
            lobby_time_secs: 30,
            min_players: 3,
            max_players: 16,
            profanity_filter: true, // Default ON per GAME_DECISIONS.md
        }
    }
}

/// Current phase of the game
#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    /// Waiting for players to join
    Lobby,
    /// Game starting countdown
    Starting,
    /// Showing the acronym
    AcronymReveal,
    /// Accepting submissions
    Submission,
    /// Voting phase
    Voting,
    /// Showing round results
    Results,
    /// Final scoreboard
    FinalResults,
    /// Game ended
    Ended,
}

/// Sub-phase within certain game phases
#[derive(Debug, Clone, PartialEq)]
pub enum RoundPhase {
    /// Normal round
    Normal,
    /// Face-off round (final round, top 2 players)
    FaceOff,
}

/// Round state
#[derive(Debug, Clone)]
pub struct Round {
    pub number: u32,
    pub acronym: String,
    pub category: Option<&'static Category>,
    pub phase: RoundPhase,
    pub submissions: HashMap<i64, Submission>,
    pub started_at: Instant,
    pub results: Option<Vec<RoundResult>>,
}

impl Round {
    pub fn new(number: u32, total_rounds: u32) -> Self {
        let length = acronym_length_for_round(number);
        let acronym = generate_acronym(length);
        let category = random_category();
        let phase = if number == total_rounds {
            RoundPhase::FaceOff
        } else {
            RoundPhase::Normal
        };

        Self {
            number,
            acronym,
            category,
            phase,
            submissions: HashMap::new(),
            started_at: Instant::now(),
            results: None,
        }
    }
}

/// Main game state
pub struct AcroGame {
    #[allow(dead_code)]
    pub id: u64,
    pub config: GameConfig,
    pub phase: GamePhase,
    pub players: HashMap<i64, PlayerState>,
    pub current_round: Option<Round>,
    pub phase_started_at: Instant,
    pub profanity_filter: ProfanityFilter,
    #[allow(dead_code)]
    pub host_user_id: i64,
    #[allow(dead_code)]
    pub created_at: Instant,
    next_submission_id: u64,
}

impl AcroGame {
    pub fn new(id: u64, host_user_id: i64, config: GameConfig) -> Self {
        let filter_enabled = config.profanity_filter;
        Self {
            id,
            config,
            phase: GamePhase::Lobby,
            players: HashMap::new(),
            current_round: None,
            phase_started_at: Instant::now(),
            profanity_filter: ProfanityFilter::new(filter_enabled),
            host_user_id,
            created_at: Instant::now(),
            next_submission_id: 1,
        }
    }

    /// Add a player to the game
    pub fn add_player(&mut self, user_id: i64, handle: String) -> Result<(), &'static str> {
        if self.players.len() >= self.config.max_players {
            return Err("Game is full");
        }

        if self.phase != GamePhase::Lobby && self.phase != GamePhase::Starting {
            return Err("Game already in progress");
        }

        if self.players.contains_key(&user_id) {
            return Err("Already in game");
        }

        self.players.insert(user_id, PlayerState::new(user_id, handle));
        Ok(())
    }

    /// Remove a player from the game
    #[allow(dead_code)]
    pub fn remove_player(&mut self, user_id: i64) -> bool {
        if self.players.remove(&user_id).is_some() {
            // Check if game should end (below minimum players)
            if self.players.len() < 2 && self.phase != GamePhase::Lobby {
                self.phase = GamePhase::Ended;
            }
            true
        } else {
            false
        }
    }

    /// Mark player as disconnected (can rejoin)
    pub fn disconnect_player(&mut self, user_id: i64) {
        if let Some(player) = self.players.get_mut(&user_id) {
            player.is_connected = false;
        }

        // Count connected players
        let connected = self.players.values().filter(|p| p.is_connected).count();

        // End game if below minimum
        if connected < 2 && self.phase != GamePhase::Lobby {
            self.phase = GamePhase::Ended;
        }
    }

    /// Reconnect a player
    #[allow(dead_code)]
    pub fn reconnect_player(&mut self, user_id: i64) -> bool {
        if let Some(player) = self.players.get_mut(&user_id) {
            player.is_connected = true;
            true
        } else {
            false
        }
    }

    /// Get number of active (connected) players
    pub fn active_player_count(&self) -> usize {
        self.players.values().filter(|p| p.is_connected).count()
    }

    /// Check if game can start
    pub fn can_start(&self) -> bool {
        self.phase == GamePhase::Lobby
            && self.active_player_count() >= self.config.min_players
    }

    /// Start the game
    pub fn start(&mut self) -> Result<(), &'static str> {
        if !self.can_start() {
            return Err("Cannot start game");
        }

        self.phase = GamePhase::Starting;
        self.phase_started_at = Instant::now();
        Ok(())
    }

    /// Advance to next phase based on timers
    pub fn tick(&mut self) -> Option<GamePhase> {
        let elapsed = self.phase_started_at.elapsed();

        let (should_advance, next_phase) = match self.phase {
            GamePhase::Starting => {
                (elapsed >= Duration::from_secs(5), Some(GamePhase::AcronymReveal))
            }
            GamePhase::AcronymReveal => {
                (elapsed >= Duration::from_secs(5), Some(GamePhase::Submission))
            }
            GamePhase::Submission => {
                (elapsed >= Duration::from_secs(self.config.submission_time_secs), Some(GamePhase::Voting))
            }
            GamePhase::Voting => {
                (elapsed >= Duration::from_secs(self.config.voting_time_secs), Some(GamePhase::Results))
            }
            GamePhase::Results => {
                let next = if let Some(round) = &self.current_round {
                    if round.number >= self.config.total_rounds {
                        GamePhase::FinalResults
                    } else {
                        GamePhase::AcronymReveal
                    }
                } else {
                    GamePhase::FinalResults
                };
                (elapsed >= Duration::from_secs(self.config.results_time_secs), Some(next))
            }
            GamePhase::FinalResults => {
                (elapsed >= Duration::from_secs(15), Some(GamePhase::Ended))
            }
            _ => (false, None),
        };

        if should_advance {
            if let Some(next) = next_phase {
                self.transition_to(next.clone());
                return Some(next);
            }
        }

        None
    }

    /// Transition to a new game phase
    fn transition_to(&mut self, phase: GamePhase) {
        self.phase_started_at = Instant::now();

        match phase {
            GamePhase::AcronymReveal => {
                // Start new round
                let round_num = self.current_round.as_ref().map(|r| r.number + 1).unwrap_or(1);
                self.current_round = Some(Round::new(round_num, self.config.total_rounds));

                // Reset player round state
                for player in self.players.values_mut() {
                    player.reset_round();
                }
            }
            GamePhase::Voting => {
                // Mark players who didn't submit as skipping
                for player in self.players.values_mut() {
                    if player.current_submission.is_none() && player.is_connected {
                        player.record_skip();
                    }
                }
            }
            GamePhase::Results => {
                // Calculate round results
                self.calculate_results();
            }
            _ => {}
        }

        self.phase = phase;
    }

    /// Submit an answer for the current round
    pub fn submit(&mut self, user_id: i64, text: String) -> Result<(), &'static str> {
        if self.phase != GamePhase::Submission {
            return Err("Not in submission phase");
        }

        // Check player exists and is connected
        {
            let player = self.players.get(&user_id)
                .ok_or("Player not found")?;
            if !player.is_connected {
                return Err("Player disconnected");
            }
        }

        // Validate submission and get round info
        let (acronym, round_started_at) = {
            let round = self.current_round.as_ref().ok_or("No active round")?;
            (round.acronym.clone(), round.started_at)
        };

        if !validate_submission(&text, &acronym) {
            return Err("Invalid submission - must match acronym letters");
        }

        // Check profanity filter
        if !self.profanity_filter.is_clean(&text) {
            return Err("Submission contains inappropriate content");
        }

        // Create submission
        let submission_id = self.next_submission_id;
        self.next_submission_id += 1;

        let submission = Submission::new(
            submission_id,
            user_id,
            text.clone(),
            round_started_at,
        );

        // Store in player state
        if let Some(player) = self.players.get_mut(&user_id) {
            player.current_submission = Some(submission.clone());
            player.submitted();
            player.stats.submissions_made += 1;
        }

        // Store in round
        if let Some(round) = &mut self.current_round {
            round.submissions.insert(user_id, submission);
        }

        Ok(())
    }


    /// Cast a vote for a submission
    pub fn vote(&mut self, user_id: i64, submission_id: u64) -> Result<(), &'static str> {
        if self.phase != GamePhase::Voting {
            return Err("Not in voting phase");
        }

        let player = self.players.get_mut(&user_id)
            .ok_or("Player not found")?;

        if !player.is_connected {
            return Err("Player disconnected");
        }

        // Find the submission
        let round = self.current_round.as_ref().ok_or("No active round")?;

        let submission = round.submissions.values()
            .find(|s| s.id == submission_id)
            .ok_or("Submission not found")?;

        // Can't vote for your own submission
        if submission.user_id == user_id {
            return Err("Cannot vote for your own submission");
        }

        // Record vote
        player.current_vote = Some(submission_id);
        player.stats.votes_cast += 1;

        Ok(())
    }

    /// Calculate round results after voting ends
    fn calculate_results(&mut self) {
        let round = match &mut self.current_round {
            Some(r) => r,
            None => return,
        };

        // Count votes for each submission
        let mut vote_counts: HashMap<u64, u32> = HashMap::new();
        for player in self.players.values() {
            if let Some(vote) = player.current_vote {
                *vote_counts.entry(vote).or_insert(0) += 1;
            }
        }

        // Build submission data for scoring
        let total_voters = self.players.values()
            .filter(|p| p.is_connected && p.current_vote.is_some())
            .count() as u32;

        let submissions: Vec<_> = round.submissions.values()
            .map(|s| {
                let votes = vote_counts.get(&s.id).copied().unwrap_or(0);
                let handle = self.players.get(&s.user_id)
                    .map(|p| p.handle.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let seconds = s.speed_seconds(round.started_at);
                (s.user_id, handle, s.text.clone(), votes, seconds)
            })
            .collect();

        let is_faceoff = round.phase == RoundPhase::FaceOff;
        let results = calculate_round_results(
            &submissions,
            self.config.submission_time_secs,
            total_voters,
            is_faceoff,
        );

        // Apply scores to players
        for result in &results {
            if let Some(player) = self.players.get_mut(&result.user_id) {
                player.add_score(result.total_points);
                player.stats.votes_received += result.votes_received;
                if result.unanimous_bonus > 0 {
                    player.stats.unanimous_wins += 1;
                }
                if result.is_winner {
                    player.stats.rounds_won += 1;
                }
            }
        }

        round.results = Some(results);
    }

    /// Get current standings sorted by score
    pub fn get_standings(&self) -> Vec<(i64, String, i64)> {
        let mut standings: Vec<_> = self.players.values()
            .map(|p| (p.user_id, p.handle.clone(), p.score))
            .collect();
        standings.sort_by(|a, b| b.2.cmp(&a.2));
        standings
    }

    /// Get time remaining in current phase (in seconds)
    pub fn time_remaining(&self) -> u64 {
        let duration = match self.phase {
            GamePhase::Starting => Duration::from_secs(5),
            GamePhase::AcronymReveal => Duration::from_secs(5),
            GamePhase::Submission => Duration::from_secs(self.config.submission_time_secs),
            GamePhase::Voting => Duration::from_secs(self.config.voting_time_secs),
            GamePhase::Results => Duration::from_secs(self.config.results_time_secs),
            GamePhase::FinalResults => Duration::from_secs(15),
            _ => Duration::from_secs(0),
        };

        let elapsed = self.phase_started_at.elapsed();
        if elapsed >= duration {
            0
        } else {
            (duration - elapsed).as_secs()
        }
    }

    /// Get submissions for voting display (randomized order, anonymous)
    pub fn get_voting_options(&self) -> Vec<(u64, String)> {
        if let Some(round) = &self.current_round {
            let mut options: Vec<_> = round.submissions.values()
                .map(|s| (s.id, s.text.clone()))
                .collect();

            // Shuffle for anonymous voting
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            options.shuffle(&mut rng);

            options
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_game() -> AcroGame {
        let config = GameConfig {
            min_players: 2, // Lower for testing
            ..Default::default()
        };
        AcroGame::new(1, 100, config)
    }

    #[test]
    fn test_add_player() {
        let mut game = create_test_game();
        assert!(game.add_player(1, "Alice".to_string()).is_ok());
        assert!(game.add_player(2, "Bob".to_string()).is_ok());
        assert_eq!(game.players.len(), 2);
    }

    #[test]
    fn test_add_duplicate_player() {
        let mut game = create_test_game();
        assert!(game.add_player(1, "Alice".to_string()).is_ok());
        assert!(game.add_player(1, "Alice2".to_string()).is_err());
    }

    #[test]
    fn test_remove_player() {
        let mut game = create_test_game();
        game.add_player(1, "Alice".to_string()).unwrap();
        game.add_player(2, "Bob".to_string()).unwrap();

        assert!(game.remove_player(1));
        assert_eq!(game.players.len(), 1);
        assert!(!game.remove_player(1)); // Already removed
    }

    #[test]
    fn test_can_start() {
        let mut game = create_test_game();
        assert!(!game.can_start()); // No players

        game.add_player(1, "Alice".to_string()).unwrap();
        assert!(!game.can_start()); // Need 2 players

        game.add_player(2, "Bob".to_string()).unwrap();
        assert!(game.can_start()); // Now can start
    }

    #[test]
    fn test_start_game() {
        let mut game = create_test_game();
        game.add_player(1, "Alice".to_string()).unwrap();
        game.add_player(2, "Bob".to_string()).unwrap();

        assert!(game.start().is_ok());
        assert_eq!(game.phase, GamePhase::Starting);
    }

    #[test]
    fn test_validate_submission() {
        assert!(validate_submission("Apple Banana Cherry", "ABC"));
        assert!(validate_submission("apple banana cherry", "ABC")); // Case insensitive
        assert!(!validate_submission("Apple Banana", "ABC")); // Wrong count
        assert!(!validate_submission("Apple Xray Cherry", "ABC")); // Wrong letter
        assert!(!validate_submission("", "ABC")); // Empty
    }

    #[test]
    fn test_disconnect_reconnect() {
        let mut game = create_test_game();
        game.add_player(1, "Alice".to_string()).unwrap();
        game.add_player(2, "Bob".to_string()).unwrap();

        assert_eq!(game.active_player_count(), 2);

        game.disconnect_player(1);
        assert_eq!(game.active_player_count(), 1);
        assert!(!game.players.get(&1).unwrap().is_connected);

        game.reconnect_player(1);
        assert_eq!(game.active_player_count(), 2);
        assert!(game.players.get(&1).unwrap().is_connected);
    }

    #[test]
    fn test_game_ends_below_min_players() {
        let mut game = create_test_game();
        game.add_player(1, "Alice".to_string()).unwrap();
        game.add_player(2, "Bob".to_string()).unwrap();
        game.add_player(3, "Carol".to_string()).unwrap(); // Need 3 players to test
        game.start().unwrap();

        // First disconnect - still 2 connected, game continues
        game.disconnect_player(1);
        assert_eq!(game.active_player_count(), 2);
        assert_eq!(game.phase, GamePhase::Starting); // Still running

        // Second disconnect - only 1 connected, game ends
        game.disconnect_player(2);
        assert_eq!(game.active_player_count(), 1);
        assert_eq!(game.phase, GamePhase::Ended); // Now ended
    }

    #[test]
    fn test_standings() {
        let mut game = create_test_game();
        game.add_player(1, "Alice".to_string()).unwrap();
        game.add_player(2, "Bob".to_string()).unwrap();

        game.players.get_mut(&1).unwrap().add_score(200);
        game.players.get_mut(&2).unwrap().add_score(300);

        let standings = game.get_standings();
        assert_eq!(standings[0].0, 2); // Bob first
        assert_eq!(standings[0].2, 300);
        assert_eq!(standings[1].0, 1); // Alice second
        assert_eq!(standings[1].2, 200);
    }
}
