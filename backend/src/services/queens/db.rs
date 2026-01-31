//! Queens game database - puzzles, players, and completions

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

#[allow(dead_code)]
pub struct QueensDb {
    pool: SqlitePool,
}

#[allow(dead_code)]
impl QueensDb {
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
        // Daily puzzles - cached for performance
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS daily_puzzles (
                date TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                regions_json TEXT NOT NULL,
                solution_json TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Player stats
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_stats (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                current_streak INTEGER NOT NULL DEFAULT 0,
                longest_streak INTEGER NOT NULL DEFAULT 0,
                last_played_date TEXT,
                games_completed INTEGER NOT NULL DEFAULT 0,
                best_time_seconds INTEGER,
                avg_time_seconds INTEGER,
                pause_days_remaining INTEGER NOT NULL DEFAULT 3,
                pause_week_start TEXT,
                total_hints_used INTEGER NOT NULL DEFAULT 0,
                stats_json TEXT
            )
        "#).execute(&self.pool).await?;

        // Individual completions (for leaderboard)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                puzzle_date TEXT NOT NULL,
                time_seconds INTEGER NOT NULL,
                hints_used INTEGER NOT NULL,
                completed_at TEXT NOT NULL,
                UNIQUE(user_id, puzzle_date)
            )
        "#).execute(&self.pool).await?;

        // Current game saves (in-progress puzzles)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Index for leaderboard queries
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_stats_streak
            ON player_stats(current_streak DESC, best_time_seconds ASC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // Player Stats
    // ========================================================================

    pub async fn get_player_stats(&self, user_id: i64) -> Result<Option<PlayerStatsRow>, sqlx::Error> {
        let row: Option<PlayerStatsRow> = sqlx::query_as(
            r#"SELECT user_id, handle, current_streak, longest_streak, last_played_date,
                      games_completed, best_time_seconds, avg_time_seconds,
                      pause_days_remaining, pause_week_start, total_hints_used
               FROM player_stats WHERE user_id = ?"#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn save_player_stats(&self, user_id: i64, handle: &str, stats: &crate::games::queens::PlayerStats) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO player_stats
               (user_id, handle, current_streak, longest_streak, last_played_date,
                games_completed, best_time_seconds, avg_time_seconds,
                pause_days_remaining, pause_week_start, total_hints_used)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(user_id) DO UPDATE SET
                 handle = excluded.handle,
                 current_streak = excluded.current_streak,
                 longest_streak = excluded.longest_streak,
                 last_played_date = excluded.last_played_date,
                 games_completed = excluded.games_completed,
                 best_time_seconds = excluded.best_time_seconds,
                 avg_time_seconds = excluded.avg_time_seconds,
                 pause_days_remaining = excluded.pause_days_remaining,
                 pause_week_start = excluded.pause_week_start,
                 total_hints_used = excluded.total_hints_used"#
        )
        .bind(user_id)
        .bind(handle)
        .bind(stats.current_streak as i64)
        .bind(stats.longest_streak as i64)
        .bind(&stats.last_played_date)
        .bind(stats.games_completed as i64)
        .bind(stats.best_time_seconds.map(|t| t as i64))
        .bind(stats.avg_time_seconds.map(|t| t as i64))
        .bind(stats.pause_days_remaining as i64)
        .bind(&stats.pause_week_start)
        .bind(stats.total_hints_used as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ========================================================================
    // Game Saves
    // ========================================================================

    pub async fn save_game(&self, user_id: i64, handle: &str, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, state_json, last_saved)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(state_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Completions
    // ========================================================================

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        puzzle_date: &str,
        time_seconds: u32,
        hints_used: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO completions (user_id, handle, puzzle_date, time_seconds, hints_used, completed_at)
               VALUES (?, ?, ?, ?, ?, datetime('now', '-5 hours'))
               ON CONFLICT(user_id, puzzle_date) DO NOTHING"#
        )
        .bind(user_id)
        .bind(handle)
        .bind(puzzle_date)
        .bind(time_seconds as i64)
        .bind(hints_used as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn has_completed_today(&self, user_id: i64, date: &str) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM completions WHERE user_id = ? AND puzzle_date = ?"
        )
        .bind(user_id)
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    // ========================================================================
    // Leaderboard
    // ========================================================================

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
            r#"SELECT handle, current_streak, COALESCE(best_time_seconds, 9999), games_completed
               FROM player_stats
               WHERE games_completed > 0
               ORDER BY current_streak DESC, best_time_seconds ASC
               LIMIT ?"#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().enumerate().map(|(i, (handle, streak, time, games))| {
            LeaderboardEntry {
                rank: (i + 1) as i64,
                handle,
                current_streak: streak as u32,
                best_time_seconds: time as u32,
                games_completed: games as u32,
            }
        }).collect())
    }

    /// Get leaderboard for today's puzzle specifically
    pub async fn get_today_leaderboard(&self, date: &str, limit: i64) -> Result<Vec<TodayLeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"SELECT handle, time_seconds, hints_used
               FROM completions
               WHERE puzzle_date = ?
               ORDER BY hints_used ASC, time_seconds ASC
               LIMIT ?"#
        )
        .bind(date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().enumerate().map(|(i, (handle, time, hints))| {
            TodayLeaderboardEntry {
                rank: (i + 1) as i64,
                handle,
                time_seconds: time as u32,
                hints_used: hints as u32,
            }
        }).collect())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct PlayerStatsRow {
    pub user_id: i64,
    pub handle: String,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub last_played_date: Option<String>,
    pub games_completed: i64,
    pub best_time_seconds: Option<i64>,
    pub avg_time_seconds: Option<i64>,
    pub pause_days_remaining: i64,
    pub pause_week_start: Option<String>,
    pub total_hints_used: i64,
}

#[allow(dead_code)]
impl PlayerStatsRow {
    pub fn to_player_stats(&self) -> crate::games::queens::PlayerStats {
        crate::games::queens::PlayerStats {
            current_streak: self.current_streak as u32,
            longest_streak: self.longest_streak as u32,
            last_played_date: self.last_played_date.clone(),
            games_completed: self.games_completed as u32,
            best_time_seconds: self.best_time_seconds.map(|t| t as u32),
            avg_time_seconds: self.avg_time_seconds.map(|t| t as u32),
            pause_days_remaining: self.pause_days_remaining as u32,
            pause_week_start: self.pause_week_start.clone(),
            total_hints_used: self.total_hints_used as u32,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub current_streak: u32,
    pub best_time_seconds: u32,
    pub games_completed: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TodayLeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub time_seconds: u32,
    pub hints_used: u32,
}
