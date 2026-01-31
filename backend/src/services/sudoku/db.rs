//! Sudoku database operations
//!
//! Manages:
//! - Daily puzzles (cached generation)
//! - Player attempts (one per day)
//! - Player stats (streaks, completion times)

use chrono::Datelike;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct SudokuDb {
    pool: SqlitePool,
}

/// Leaderboard entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub longest_streak: u32,
    pub current_streak: u32,
    pub games_completed: u32,
    pub best_time_seconds: Option<u32>,
}

/// Player stats from database
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerStats {
    pub user_id: i64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub last_played_date: Option<String>,
    pub games_completed: u32,
    pub best_time_seconds: Option<u32>,
    pub total_time_seconds: u64,
    pub pause_days_used: u32,
    pub pause_days_week_start: Option<String>,
}

/// Completion record for a specific puzzle
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompletionRecord {
    pub user_id: i64,
    pub puzzle_date: String,
    pub time_seconds: u32,
    pub errors: u32,
    pub completed_at: String,
}

impl SudokuDb {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Enable WAL mode for concurrency
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&pool)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // Daily puzzles (cached)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS daily_puzzles (
                puzzle_date TEXT PRIMARY KEY,
                puzzle TEXT NOT NULL,
                solution TEXT NOT NULL,
                clue_count INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Player stats (one per user)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_stats (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                current_streak INTEGER NOT NULL DEFAULT 0,
                longest_streak INTEGER NOT NULL DEFAULT 0,
                last_played_date TEXT,
                games_completed INTEGER NOT NULL DEFAULT 0,
                best_time_seconds INTEGER,
                total_time_seconds INTEGER NOT NULL DEFAULT 0,
                pause_days_used INTEGER NOT NULL DEFAULT 0,
                pause_days_week_start TEXT
            )
        "#).execute(&self.pool).await?;

        // Completions (one per user per day)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                puzzle_date TEXT NOT NULL,
                time_seconds INTEGER NOT NULL,
                errors INTEGER NOT NULL DEFAULT 0,
                completed_at TEXT NOT NULL,
                UNIQUE(user_id, puzzle_date)
            )
        "#).execute(&self.pool).await?;

        // Active saves (in-progress puzzles)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER NOT NULL,
                puzzle_date TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL,
                PRIMARY KEY (user_id, puzzle_date)
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_user
            ON completions(user_id, puzzle_date)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_stats_streak
            ON player_stats(longest_streak DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // Puzzle Management
    // ========================================================================

    /// Get or create daily puzzle
    #[allow(dead_code)]
    pub async fn get_or_create_puzzle(&self, date: &str) -> Result<(String, String, u32), sqlx::Error> {
        // Try to get existing puzzle
        let existing: Option<(String, String, i32)> = sqlx::query_as(
            "SELECT puzzle, solution, clue_count FROM daily_puzzles WHERE puzzle_date = ?"
        )
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((puzzle, solution, clue_count)) = existing {
            return Ok((puzzle, solution, clue_count as u32));
        }

        // Generate new puzzle
        use crate::games::sudoku::generator::generate_puzzle;
        let daily = generate_puzzle(date);

        // Serialize to string (81 chars each)
        let puzzle_str: String = daily.puzzle.iter()
            .flat_map(|row| row.iter())
            .map(|&n| char::from_digit(n as u32, 10).unwrap_or('0'))
            .collect();

        let solution_str: String = daily.solution.iter()
            .flat_map(|row| row.iter())
            .map(|&n| char::from_digit(n as u32, 10).unwrap_or('0'))
            .collect();

        // Save to database
        sqlx::query(
            "INSERT INTO daily_puzzles (puzzle_date, puzzle, solution, clue_count, created_at)
             VALUES (?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(date)
        .bind(&puzzle_str)
        .bind(&solution_str)
        .bind(daily.clue_count as i32)
        .execute(&self.pool)
        .await?;

        Ok((puzzle_str, solution_str, daily.clue_count))
    }

    // ========================================================================
    // Save/Load Game State
    // ========================================================================

    /// Save in-progress game
    pub async fn save_game(&self, user_id: i64, puzzle_date: &str, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, puzzle_date, state_json, last_saved)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(puzzle_date)
        .bind(state_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load in-progress game for today
    pub async fn load_game(&self, user_id: i64, puzzle_date: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ? AND puzzle_date = ?"
        )
        .bind(user_id)
        .bind(puzzle_date)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Delete save (after completion)
    pub async fn delete_save(&self, user_id: i64, puzzle_date: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ? AND puzzle_date = ?")
            .bind(user_id)
            .bind(puzzle_date)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Player Stats
    // ========================================================================

    /// Get or create player stats
    pub async fn get_or_create_stats(&self, user_id: i64, handle: &str) -> Result<PlayerStats, sqlx::Error> {
        // Try to get existing
        let existing: Option<(i64, i32, i32, Option<String>, i32, Option<i32>, i64, i32, Option<String>)> = sqlx::query_as(
            "SELECT user_id, current_streak, longest_streak, last_played_date, games_completed,
                    best_time_seconds, total_time_seconds, pause_days_used, pause_days_week_start
             FROM player_stats WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((uid, cur, longest, last_date, games, best, total, pause, pause_week)) = existing {
            return Ok(PlayerStats {
                user_id: uid,
                current_streak: cur as u32,
                longest_streak: longest as u32,
                last_played_date: last_date,
                games_completed: games as u32,
                best_time_seconds: best.map(|t| t as u32),
                total_time_seconds: total as u64,
                pause_days_used: pause as u32,
                pause_days_week_start: pause_week,
            });
        }

        // Create new stats
        sqlx::query(
            "INSERT INTO player_stats (user_id, handle) VALUES (?, ?)"
        )
        .bind(user_id)
        .bind(handle)
        .execute(&self.pool)
        .await?;

        Ok(PlayerStats {
            user_id,
            current_streak: 0,
            longest_streak: 0,
            last_played_date: None,
            games_completed: 0,
            best_time_seconds: None,
            total_time_seconds: 0,
            pause_days_used: 0,
            pause_days_week_start: None,
        })
    }

    /// Check if player already completed today's puzzle
    pub async fn has_completed_today(&self, user_id: i64, puzzle_date: &str) -> Result<Option<u32>, sqlx::Error> {
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT time_seconds FROM completions WHERE user_id = ? AND puzzle_date = ?"
        )
        .bind(user_id)
        .bind(puzzle_date)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(t,)| t as u32))
    }

    /// Record puzzle completion and update stats
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        puzzle_date: &str,
        time_seconds: u32,
        errors: u32,
    ) -> Result<(u32, u32), sqlx::Error> {
        // Insert completion record
        sqlx::query(
            "INSERT OR REPLACE INTO completions (user_id, puzzle_date, time_seconds, errors, completed_at)
             VALUES (?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(puzzle_date)
        .bind(time_seconds as i32)
        .bind(errors as i32)
        .execute(&self.pool)
        .await?;

        // Get current stats
        let stats = self.get_or_create_stats(user_id, handle).await?;

        // Calculate new streak
        let (new_current_streak, used_pause_day) = self.calculate_new_streak(&stats, puzzle_date);
        let new_longest_streak = stats.longest_streak.max(new_current_streak);

        // Update best time
        let new_best_time = match stats.best_time_seconds {
            Some(best) if best <= time_seconds => Some(best),
            _ => Some(time_seconds),
        };

        // Reset pause days if new week
        let (new_pause_used, new_pause_week) = self.calculate_pause_days(&stats, puzzle_date, used_pause_day);

        // Update stats
        sqlx::query(
            "UPDATE player_stats SET
                current_streak = ?,
                longest_streak = ?,
                last_played_date = ?,
                games_completed = games_completed + 1,
                best_time_seconds = ?,
                total_time_seconds = total_time_seconds + ?,
                pause_days_used = ?,
                pause_days_week_start = ?
             WHERE user_id = ?"
        )
        .bind(new_current_streak as i32)
        .bind(new_longest_streak as i32)
        .bind(puzzle_date)
        .bind(new_best_time.map(|t| t as i32))
        .bind(time_seconds as i64)
        .bind(new_pause_used as i32)
        .bind(&new_pause_week)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Delete save
        self.delete_save(user_id, puzzle_date).await?;

        Ok((new_current_streak, new_longest_streak))
    }

    /// Calculate new streak value based on last played date
    fn calculate_new_streak(&self, stats: &PlayerStats, today: &str) -> (u32, bool) {
        let yesterday = self.date_subtract_one(today);

        match &stats.last_played_date {
            None => (1, false), // First game
            Some(last) if last == today => (stats.current_streak, false), // Already played today
            Some(last) if last == &yesterday => (stats.current_streak + 1, false), // Consecutive day
            Some(last) => {
                // Check if we can use a pause day
                let days_missed = self.days_between(last, today);
                if days_missed <= 3 && stats.pause_days_used < 3 {
                    // Use pause day(s)
                    (stats.current_streak + 1, true)
                } else {
                    // Streak broken
                    (1, false)
                }
            }
        }
    }

    /// Calculate pause day tracking
    fn calculate_pause_days(&self, stats: &PlayerStats, today: &str, used_pause: bool) -> (u32, Option<String>) {
        let week_start = self.get_week_start(today);

        // Check if it's a new week
        match &stats.pause_days_week_start {
            None => {
                // First time
                (if used_pause { 1 } else { 0 }, Some(week_start))
            }
            Some(start) if start != &week_start => {
                // New week - reset
                (if used_pause { 1 } else { 0 }, Some(week_start))
            }
            Some(start) => {
                // Same week
                (if used_pause { stats.pause_days_used + 1 } else { stats.pause_days_used }, Some(start.clone()))
            }
        }
    }

    /// Get Monday of the week for a date
    fn get_week_start(&self, date: &str) -> String {
        use chrono::NaiveDate;

        if let Ok(d) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            let weekday = d.weekday().num_days_from_monday();
            let monday = d - chrono::Duration::days(weekday as i64);
            return monday.format("%Y-%m-%d").to_string();
        }
        date.to_string()
    }

    /// Subtract one day from a date string
    fn date_subtract_one(&self, date: &str) -> String {
        use chrono::NaiveDate;

        if let Ok(d) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            let yesterday = d - chrono::Duration::days(1);
            return yesterday.format("%Y-%m-%d").to_string();
        }
        date.to_string()
    }

    /// Calculate days between two date strings
    fn days_between(&self, from: &str, to: &str) -> i32 {
        use chrono::NaiveDate;

        if let (Ok(d1), Ok(d2)) = (
            NaiveDate::parse_from_str(from, "%Y-%m-%d"),
            NaiveDate::parse_from_str(to, "%Y-%m-%d"),
        ) {
            return (d2 - d1).num_days() as i32;
        }
        999 // Large number if parsing fails
    }

    // ========================================================================
    // Leaderboard
    // ========================================================================

    /// Get leaderboard by longest streak
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i32, i32, i32, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY longest_streak DESC, games_completed DESC) as rank,
                handle,
                longest_streak,
                current_streak,
                games_completed,
                best_time_seconds
            FROM player_stats
            WHERE games_completed > 0
            ORDER BY longest_streak DESC, games_completed DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, longest, current, games, best)| {
            LeaderboardEntry {
                rank,
                handle,
                longest_streak: longest as u32,
                current_streak: current as u32,
                games_completed: games as u32,
                best_time_seconds: best.map(|t| t as u32),
            }
        }).collect())
    }

    /// Get leaderboard as tuples for rendering
    pub async fn get_leaderboard_tuples(&self, limit: i64) -> Result<Vec<(String, u32, u32)>, sqlx::Error> {
        let entries = self.get_leaderboard(limit).await?;
        Ok(entries.into_iter().map(|e| (e.handle, e.longest_streak, e.games_completed)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> SudokuDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_sudoku.db");
        SudokuDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_db() {
        let _db = create_test_db().await;
        // If we get here, DB was created successfully
    }

    #[tokio::test]
    async fn test_get_or_create_stats() {
        let db = create_test_db().await;

        let stats = db.get_or_create_stats(1, "TestUser").await.unwrap();
        assert_eq!(stats.user_id, 1);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.games_completed, 0);

        // Get again should return same
        let stats2 = db.get_or_create_stats(1, "TestUser").await.unwrap();
        assert_eq!(stats2.user_id, 1);
    }

    #[tokio::test]
    async fn test_save_load_game() {
        let db = create_test_db().await;

        let state_json = r#"{"puzzle_date":"2026-01-30","cursor":[0,0]}"#;
        db.save_game(1, "2026-01-30", state_json).await.unwrap();

        let loaded = db.load_game(1, "2026-01-30").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), state_json);
    }

    #[tokio::test]
    async fn test_has_completed_today() {
        let db = create_test_db().await;

        // Not completed yet
        let result = db.has_completed_today(1, "2026-01-30").await.unwrap();
        assert!(result.is_none());

        // Record completion
        db.get_or_create_stats(1, "TestUser").await.unwrap();
        db.record_completion(1, "TestUser", "2026-01-30", 300, 0).await.unwrap();

        // Now completed
        let result = db.has_completed_today(1, "2026-01-30").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 300);
    }

    #[tokio::test]
    async fn test_streak_calculation() {
        let db = create_test_db().await;
        db.get_or_create_stats(1, "TestUser").await.unwrap();

        // First completion
        let (streak, _) = db.record_completion(1, "TestUser", "2026-01-30", 300, 0).await.unwrap();
        assert_eq!(streak, 1);

        // Next day
        let (streak, _) = db.record_completion(1, "TestUser", "2026-01-31", 350, 0).await.unwrap();
        assert_eq!(streak, 2);
    }

    #[tokio::test]
    async fn test_get_puzzle() {
        let db = create_test_db().await;

        let (puzzle, solution, clue_count) = db.get_or_create_puzzle("2026-01-30").await.unwrap();
        assert_eq!(puzzle.len(), 81);
        assert_eq!(solution.len(), 81);
        assert!(clue_count >= 25 && clue_count <= 35);

        // Get again should return same
        let (puzzle2, solution2, _) = db.get_or_create_puzzle("2026-01-30").await.unwrap();
        assert_eq!(puzzle, puzzle2);
        assert_eq!(solution, solution2);
    }
}
