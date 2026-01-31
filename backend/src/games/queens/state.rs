//! Game state for Queens - player progress and streaks

use serde::{Deserialize, Serialize};

/// Player statistics and streak tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Current consecutive day streak
    pub current_streak: u32,
    /// Longest streak ever achieved
    pub longest_streak: u32,
    /// Last date the player completed a puzzle (YYYY-MM-DD)
    pub last_played_date: Option<String>,
    /// Total puzzles completed
    pub games_completed: u32,
    /// Best completion time in seconds
    pub best_time_seconds: Option<u32>,
    /// Average completion time in seconds
    pub avg_time_seconds: Option<u32>,
    /// Pause days remaining this week (0-3)
    pub pause_days_remaining: u32,
    /// Start of current pause week (YYYY-MM-DD of Sunday)
    pub pause_week_start: Option<String>,
    /// Total hints used across all games
    pub total_hints_used: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            current_streak: 0,
            longest_streak: 0,
            last_played_date: None,
            games_completed: 0,
            best_time_seconds: None,
            avg_time_seconds: None,
            pause_days_remaining: 3,
            pause_week_start: None,
            total_hints_used: 0,
        }
    }
}

impl PlayerStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update streak based on completion date
    /// Returns true if streak was preserved/extended
    pub fn record_completion(&mut self, date: &str, time_seconds: u32, hints_used: u32) -> bool {
        use chrono::NaiveDate;

        let today = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok();

        // Check if this is the same day as last played
        if self.last_played_date.as_ref() == Some(&date.to_string()) {
            // Already played today - shouldn't happen but handle gracefully
            return false;
        }

        // Check if streak should continue
        let streak_continues = if let Some(ref last_date) = self.last_played_date {
            if let (Some(last), Some(current)) = (
                NaiveDate::parse_from_str(last_date, "%Y-%m-%d").ok(),
                today,
            ) {
                let days_diff = (current - last).num_days();
                days_diff == 1 // Must be exactly consecutive
            } else {
                false
            }
        } else {
            true // First game ever, start streak
        };

        // Update streak
        if streak_continues || self.current_streak == 0 {
            self.current_streak += 1;
        } else {
            // Streak broken - start new one
            self.current_streak = 1;
        }

        // Update longest streak
        if self.current_streak > self.longest_streak {
            self.longest_streak = self.current_streak;
        }

        // Update last played date
        self.last_played_date = Some(date.to_string());

        // Update completion stats
        self.games_completed += 1;

        // Update best time
        if self.best_time_seconds.is_none() || time_seconds < self.best_time_seconds.unwrap() {
            self.best_time_seconds = Some(time_seconds);
        }

        // Update average time
        let total_games = self.games_completed as u32;
        if let Some(avg) = self.avg_time_seconds {
            let new_avg = ((avg as u64 * (total_games - 1) as u64) + time_seconds as u64) / total_games as u64;
            self.avg_time_seconds = Some(new_avg as u32);
        } else {
            self.avg_time_seconds = Some(time_seconds);
        }

        // Track hints
        self.total_hints_used += hints_used;

        streak_continues || self.current_streak == 1
    }

    /// Check if pause days can preserve streak
    /// Returns number of pause days needed, or None if streak is already broken
    pub fn check_pause_days_needed(&self, current_date: &str) -> Option<u32> {
        use chrono::NaiveDate;

        let today = NaiveDate::parse_from_str(current_date, "%Y-%m-%d").ok()?;

        if let Some(ref last_date) = self.last_played_date {
            let last = NaiveDate::parse_from_str(last_date, "%Y-%m-%d").ok()?;
            let days_diff = (today - last).num_days();

            if days_diff <= 0 {
                return Some(0); // Same day or past
            } else if days_diff == 1 {
                return Some(0); // Consecutive, no pause needed
            } else if days_diff as u32 <= self.pause_days_remaining + 1 {
                // Can use pause days (days_diff - 1 pause days needed)
                return Some((days_diff - 1) as u32);
            }
        } else {
            // No previous game, no pause needed
            return Some(0);
        }

        None // Streak is broken
    }

    /// Use pause days to preserve streak
    /// Returns true if successful
    pub fn use_pause_days(&mut self, current_date: &str, days_to_use: u32) -> bool {
        if days_to_use == 0 {
            return true;
        }

        if days_to_use > self.pause_days_remaining {
            return false;
        }

        // Update pause week if needed
        self.update_pause_week(current_date);

        // Use pause days
        self.pause_days_remaining -= days_to_use;

        true
    }

    /// Reset pause days at start of new week
    fn update_pause_week(&mut self, current_date: &str) {
        use chrono::{Datelike, NaiveDate};

        if let Some(today) = NaiveDate::parse_from_str(current_date, "%Y-%m-%d").ok() {
            // Find the Sunday of this week
            let days_since_sunday = today.weekday().num_days_from_sunday();
            let week_start = today - chrono::Duration::days(days_since_sunday as i64);
            let week_start_str = week_start.format("%Y-%m-%d").to_string();

            if self.pause_week_start.as_ref() != Some(&week_start_str) {
                // New week - reset pause days
                self.pause_days_remaining = 3;
                self.pause_week_start = Some(week_start_str);
            }
        }
    }

    /// Check if player has already played today
    pub fn has_played_today(&self, current_date: &str) -> bool {
        self.last_played_date.as_ref() == Some(&current_date.to_string())
    }
}

/// Current game state for an active puzzle attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Player's handle
    pub handle: Option<String>,
    /// The puzzle date being played
    pub puzzle_date: String,
    /// Current queen placements (row, col)
    pub placements: Vec<(usize, usize)>,
    /// Currently selected cell (row, col)
    pub cursor: (usize, usize),
    /// Start time (Unix timestamp in seconds)
    pub start_time: i64,
    /// Number of hints used this puzzle
    pub hints_used: u32,
    /// Whether puzzle is completed
    pub completed: bool,
    /// Completion time in seconds (if completed)
    pub completion_time: Option<u32>,
    /// Error message to display
    pub last_message: Option<String>,
}

impl GameState {
    pub fn new(puzzle_date: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            handle: None,
            puzzle_date: puzzle_date.to_string(),
            placements: Vec::new(),
            cursor: (0, 0),
            start_time,
            hints_used: 0,
            completed: false,
            completion_time: None,
            last_message: None,
        }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        if let Some(time) = self.completion_time {
            return time;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        (now - self.start_time).max(0) as u32
    }

    /// Toggle queen at current cursor position
    pub fn toggle_queen(&mut self) {
        let pos = self.cursor;

        // Check if queen already at this position
        if let Some(idx) = self.placements.iter().position(|&p| p == pos) {
            // Remove queen
            self.placements.remove(idx);
        } else {
            // Add queen
            self.placements.push(pos);
        }
    }

    /// Check if there's a queen at the given position
    pub fn has_queen(&self, row: usize, col: usize) -> bool {
        self.placements.iter().any(|&(r, c)| r == row && c == col)
    }

    /// Move cursor
    pub fn move_cursor(&mut self, dr: i32, dc: i32, board_size: usize) {
        let new_row = (self.cursor.0 as i32 + dr).max(0).min(board_size as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + dc).max(0).min(board_size as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }

    /// Record completion
    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.completion_time = Some(self.elapsed_seconds());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_stats_default() {
        let stats = PlayerStats::new();
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 0);
        assert_eq!(stats.games_completed, 0);
        assert_eq!(stats.pause_days_remaining, 3);
    }

    #[test]
    fn test_record_first_completion() {
        let mut stats = PlayerStats::new();
        let result = stats.record_completion("2026-01-30", 120, 0);

        assert!(result);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
        assert_eq!(stats.games_completed, 1);
        assert_eq!(stats.last_played_date, Some("2026-01-30".to_string()));
    }

    #[test]
    fn test_streak_continues_consecutive_days() {
        let mut stats = PlayerStats::new();
        stats.record_completion("2026-01-30", 120, 0);
        stats.record_completion("2026-01-31", 100, 0);

        assert_eq!(stats.current_streak, 2);
        assert_eq!(stats.longest_streak, 2);
    }

    #[test]
    fn test_streak_breaks_with_gap() {
        let mut stats = PlayerStats::new();
        stats.record_completion("2026-01-30", 120, 0);
        stats.current_streak = 5; // Simulate existing streak

        // Skip a day without using pause
        stats.record_completion("2026-02-01", 100, 0); // 2 days later

        // Streak should reset to 1
        assert_eq!(stats.current_streak, 1);
    }

    #[test]
    fn test_best_time_tracking() {
        let mut stats = PlayerStats::new();
        stats.record_completion("2026-01-30", 120, 0);
        assert_eq!(stats.best_time_seconds, Some(120));

        stats.record_completion("2026-01-31", 100, 0);
        assert_eq!(stats.best_time_seconds, Some(100));

        stats.record_completion("2026-02-01", 150, 0);
        assert_eq!(stats.best_time_seconds, Some(100)); // Still 100
    }

    #[test]
    fn test_has_played_today() {
        let mut stats = PlayerStats::new();
        assert!(!stats.has_played_today("2026-01-30"));

        stats.record_completion("2026-01-30", 120, 0);
        assert!(stats.has_played_today("2026-01-30"));
        assert!(!stats.has_played_today("2026-01-31"));
    }

    #[test]
    fn test_game_state_new() {
        let state = GameState::new("2026-01-30");
        assert_eq!(state.puzzle_date, "2026-01-30");
        assert!(state.placements.is_empty());
        assert_eq!(state.cursor, (0, 0));
        assert!(!state.completed);
    }

    #[test]
    fn test_toggle_queen() {
        let mut state = GameState::new("2026-01-30");
        state.cursor = (2, 3);

        // Add queen
        state.toggle_queen();
        assert!(state.has_queen(2, 3));

        // Remove queen
        state.toggle_queen();
        assert!(!state.has_queen(2, 3));
    }

    #[test]
    fn test_move_cursor() {
        let mut state = GameState::new("2026-01-30");
        state.cursor = (2, 2);

        state.move_cursor(1, 0, 6); // Down
        assert_eq!(state.cursor, (3, 2));

        state.move_cursor(-1, 0, 6); // Up
        assert_eq!(state.cursor, (2, 2));

        state.move_cursor(0, 1, 6); // Right
        assert_eq!(state.cursor, (2, 3));

        state.move_cursor(0, -1, 6); // Left
        assert_eq!(state.cursor, (2, 2));
    }

    #[test]
    fn test_move_cursor_bounds() {
        let mut state = GameState::new("2026-01-30");
        state.cursor = (0, 0);

        state.move_cursor(-1, -1, 6); // Try to go negative
        assert_eq!(state.cursor, (0, 0)); // Should stay at 0,0

        state.cursor = (5, 5);
        state.move_cursor(1, 1, 6); // Try to exceed bounds
        assert_eq!(state.cursor, (5, 5)); // Should stay at max
    }
}
