//! Player state and submission tracking for Acromania
//!
//! Handles per-player state within a game, including scores,
//! submissions, and voting history.

use serde::{Serialize, Deserialize};
use std::time::Instant;

/// State for a player in an active game
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub user_id: i64,
    pub handle: String,
    pub score: i64,
    pub consecutive_skips: u32,
    #[allow(dead_code)]
    pub joined_at: Instant,
    pub is_connected: bool,
    /// Input buffer for current submission
    pub input_buffer: String,
    /// Current submission for this round (if any)
    pub current_submission: Option<Submission>,
    /// Vote cast this round (submission_id)
    pub current_vote: Option<u64>,
    /// Statistics for end-game display
    pub stats: PlayerStats,
}

impl PlayerState {
    pub fn new(user_id: i64, handle: String) -> Self {
        Self {
            user_id,
            handle,
            score: 0,
            consecutive_skips: 0,
            joined_at: Instant::now(),
            is_connected: true,
            input_buffer: String::new(),
            current_submission: None,
            current_vote: None,
            stats: PlayerStats::default(),
        }
    }

    /// Reset state for a new round
    pub fn reset_round(&mut self) {
        self.input_buffer.clear();
        self.current_submission = None;
        self.current_vote = None;
    }

    /// Add points with validation
    pub fn add_score(&mut self, points: i64) {
        self.score = self.score.saturating_add(points);
    }

    /// Record a skip (no submission)
    pub fn record_skip(&mut self) {
        self.consecutive_skips += 1;
        if self.consecutive_skips >= 3 {
            // Penalty for 3+ consecutive skips
            self.score = self.score.saturating_sub(25);
        }
    }

    /// Reset skip counter (player submitted)
    pub fn submitted(&mut self) {
        self.consecutive_skips = 0;
    }
}

/// A player's submission for a round
#[derive(Debug, Clone)]
pub struct Submission {
    pub id: u64,
    pub user_id: i64,
    pub text: String,
    pub submitted_at: Instant,
    #[allow(dead_code)]
    pub is_valid: bool,
    #[allow(dead_code)]
    pub votes_received: u32,
    #[allow(dead_code)]
    pub points_earned: i64,
}

impl Submission {
    pub fn new(id: u64, user_id: i64, text: String, _round_start: Instant) -> Self {
        Self {
            id,
            user_id,
            text,
            submitted_at: Instant::now(),
            is_valid: true,
            votes_received: 0,
            points_earned: 0,
        }
    }

    /// Calculate submission speed in seconds from round start
    pub fn speed_seconds(&self, round_start: Instant) -> u64 {
        self.submitted_at.duration_since(round_start).as_secs()
    }
}

/// A vote cast by a player
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Vote {
    pub voter_user_id: i64,
    pub submission_id: u64,
    pub voted_at: i64, // Unix timestamp for DB storage
}

/// Player statistics tracked during a game
#[derive(Debug, Clone, Default)]
pub struct PlayerStats {
    pub submissions_made: u32,
    pub votes_cast: u32,
    pub votes_received: u32,
    #[allow(dead_code)]
    pub speed_bonuses_earned: u32,
    pub unanimous_wins: u32,
    pub rounds_won: u32,
}

/// Persistent player data (for leaderboards)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PlayerRecord {
    pub user_id: i64,
    pub handle: String,
    pub games_played: u32,
    pub games_won: u32,
    pub total_score: i64,
    pub highest_score: i64,
    pub total_votes_received: u32,
    pub total_submissions: u32,
}

impl Default for PlayerRecord {
    fn default() -> Self {
        Self {
            user_id: 0,
            handle: String::new(),
            games_played: 0,
            games_won: 0,
            total_score: 0,
            highest_score: 0,
            total_votes_received: 0,
            total_submissions: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_state_new() {
        let player = PlayerState::new(1, "TestPlayer".to_string());
        assert_eq!(player.user_id, 1);
        assert_eq!(player.handle, "TestPlayer");
        assert_eq!(player.score, 0);
        assert_eq!(player.consecutive_skips, 0);
        assert!(player.is_connected);
    }

    #[test]
    fn test_player_add_score() {
        let mut player = PlayerState::new(1, "Test".to_string());
        player.add_score(100);
        assert_eq!(player.score, 100);
        player.add_score(50);
        assert_eq!(player.score, 150);
    }

    #[test]
    fn test_player_skip_penalty() {
        let mut player = PlayerState::new(1, "Test".to_string());
        player.add_score(100);

        player.record_skip();
        assert_eq!(player.consecutive_skips, 1);
        assert_eq!(player.score, 100); // No penalty yet

        player.record_skip();
        assert_eq!(player.consecutive_skips, 2);
        assert_eq!(player.score, 100); // No penalty yet

        player.record_skip();
        assert_eq!(player.consecutive_skips, 3);
        assert_eq!(player.score, 75); // -25 penalty

        player.record_skip();
        assert_eq!(player.consecutive_skips, 4);
        assert_eq!(player.score, 50); // Another -25 penalty
    }

    #[test]
    fn test_player_reset_round() {
        let mut player = PlayerState::new(1, "Test".to_string());
        player.input_buffer = "Some input".to_string();
        player.current_submission = Some(Submission::new(1, 1, "Test".to_string(), Instant::now()));
        player.current_vote = Some(2);

        player.reset_round();

        assert!(player.input_buffer.is_empty());
        assert!(player.current_submission.is_none());
        assert!(player.current_vote.is_none());
    }

    #[test]
    fn test_player_submitted_resets_skips() {
        let mut player = PlayerState::new(1, "Test".to_string());
        player.consecutive_skips = 5;
        player.submitted();
        assert_eq!(player.consecutive_skips, 0);
    }
}
