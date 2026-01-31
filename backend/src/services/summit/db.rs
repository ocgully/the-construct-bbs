//! Database layer for Summit
//!
//! Stores player statistics, badges, cosmetics, and daily leaderboards.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct SummitDb {
    pool: SqlitePool,
}

impl SummitDb {
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
        // Player stats table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_stats (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                stats_json TEXT NOT NULL,
                last_updated TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Run completions (for history/leaderboards)
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                date TEXT NOT NULL,
                reached_summit INTEGER NOT NULL,
                duration_seconds INTEGER NOT NULL,
                height_reached INTEGER NOT NULL,
                party_size INTEGER NOT NULL,
                falls INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Daily leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS daily_leaderboard (
                date TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                party_size INTEGER NOT NULL,
                PRIMARY KEY (date, user_id)
            )
        "#).execute(&self.pool).await?;

        // Indexes
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_date
            ON completions(date)
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_daily_leaderboard_date
            ON daily_leaderboard(date, duration_seconds)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========================================================================
    // PLAYER STATS
    // ========================================================================

    pub async fn save_stats(&self, user_id: i64, handle: &str, stats_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO player_stats (user_id, handle, stats_json, last_updated)
             VALUES (?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(stats_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_stats(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT stats_json FROM player_stats WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    // ========================================================================
    // COMPLETIONS
    // ========================================================================

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        date: &str,
        reached_summit: bool,
        duration_seconds: i64,
        height_reached: i64,
        party_size: i64,
        falls: i64,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, date, reached_summit, duration_seconds,
             height_reached, party_size, falls, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(date)
        .bind(reached_summit as i64)
        .bind(duration_seconds)
        .bind(height_reached)
        .bind(party_size)
        .bind(falls)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    // ========================================================================
    // DAILY LEADERBOARD
    // ========================================================================

    pub async fn update_daily_leaderboard(
        &self,
        date: &str,
        user_id: i64,
        handle: &str,
        duration_seconds: i64,
        party_size: i64,
    ) -> Result<(), sqlx::Error> {
        // Only update if this is a better time
        sqlx::query(
            "INSERT INTO daily_leaderboard (date, user_id, handle, duration_seconds, party_size)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(date, user_id) DO UPDATE SET
                 duration_seconds = CASE
                     WHEN excluded.duration_seconds < daily_leaderboard.duration_seconds
                     THEN excluded.duration_seconds
                     ELSE daily_leaderboard.duration_seconds
                 END,
                 handle = excluded.handle"
        )
        .bind(date)
        .bind(user_id)
        .bind(handle)
        .bind(duration_seconds)
        .bind(party_size)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_daily_leaderboard(&self, date: &str, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64)> = sqlx::query_as(
            "SELECT user_id, handle, duration_seconds, party_size
             FROM daily_leaderboard
             WHERE date = ?
             ORDER BY duration_seconds ASC
             LIMIT ?"
        )
        .bind(date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().enumerate().map(|(i, (user_id, handle, duration, party))| {
            LeaderboardEntry {
                rank: (i + 1) as i64,
                user_id,
                handle,
                duration_seconds: duration,
                party_size: party,
            }
        }).collect())
    }

    pub async fn get_all_time_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64)> = sqlx::query_as(
            "SELECT user_id, handle, MIN(duration_seconds) as best_time, party_size
             FROM daily_leaderboard
             GROUP BY user_id
             ORDER BY best_time ASC
             LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().enumerate().map(|(i, (user_id, handle, duration, party))| {
            LeaderboardEntry {
                rank: (i + 1) as i64,
                user_id,
                handle,
                duration_seconds: duration,
                party_size: party,
            }
        }).collect())
    }

    // ========================================================================
    // PLAYER HISTORY
    // ========================================================================

    pub async fn get_player_history(&self, user_id: i64, limit: i64) -> Result<Vec<RunHistory>, sqlx::Error> {
        let rows: Vec<(String, i64, i64, i64, i64, String)> = sqlx::query_as(
            "SELECT date, reached_summit, duration_seconds, height_reached, party_size, completed_at
             FROM completions
             WHERE user_id = ?
             ORDER BY completed_at DESC
             LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(date, summit, duration, height, party, completed)| {
            RunHistory {
                date,
                reached_summit: summit != 0,
                duration_seconds: duration,
                height_reached: height,
                party_size: party,
                completed_at: completed,
            }
        }).collect())
    }

    pub async fn get_player_total_summits(&self, user_id: i64) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM completions WHERE user_id = ? AND reached_summit = 1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub user_id: i64,
    pub handle: String,
    pub duration_seconds: i64,
    pub party_size: i64,
}

#[derive(Debug, Clone)]
pub struct RunHistory {
    pub date: String,
    pub reached_summit: bool,
    pub duration_seconds: i64,
    pub height_reached: i64,
    pub party_size: i64,
    pub completed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> SummitDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_summit.db");
        SummitDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_save_and_load_stats() {
        let db = create_test_db().await;

        let stats_json = r#"{"user_id":1,"total_runs":5}"#;
        db.save_stats(1, "TestUser", stats_json).await.unwrap();

        let loaded = db.load_stats(1).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), stats_json);
    }

    #[tokio::test]
    async fn test_load_nonexistent_stats() {
        let db = create_test_db().await;
        let loaded = db.load_stats(999).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_record_completion() {
        let db = create_test_db().await;

        let id = db.record_completion(
            1, "TestUser", "2026-01-30",
            true, 600, 100, 2, 3
        ).await.unwrap();

        assert!(id > 0);
    }

    #[tokio::test]
    async fn test_daily_leaderboard() {
        let db = create_test_db().await;

        db.update_daily_leaderboard("2026-01-30", 1, "Fast", 300, 1).await.unwrap();
        db.update_daily_leaderboard("2026-01-30", 2, "Medium", 500, 2).await.unwrap();
        db.update_daily_leaderboard("2026-01-30", 3, "Slow", 800, 4).await.unwrap();

        let leaders = db.get_daily_leaderboard("2026-01-30", 10).await.unwrap();
        assert_eq!(leaders.len(), 3);
        assert_eq!(leaders[0].handle, "Fast");
        assert_eq!(leaders[0].rank, 1);
        assert_eq!(leaders[1].handle, "Medium");
        assert_eq!(leaders[2].handle, "Slow");
    }

    #[tokio::test]
    async fn test_daily_leaderboard_best_time_only() {
        let db = create_test_db().await;

        // First attempt
        db.update_daily_leaderboard("2026-01-30", 1, "Player", 500, 1).await.unwrap();

        // Better attempt
        db.update_daily_leaderboard("2026-01-30", 1, "Player", 400, 1).await.unwrap();

        // Worse attempt (should not update)
        db.update_daily_leaderboard("2026-01-30", 1, "Player", 600, 1).await.unwrap();

        let leaders = db.get_daily_leaderboard("2026-01-30", 10).await.unwrap();
        assert_eq!(leaders.len(), 1);
        assert_eq!(leaders[0].duration_seconds, 400); // Best time kept
    }

    #[tokio::test]
    async fn test_player_history() {
        let db = create_test_db().await;

        db.record_completion(1, "TestUser", "2026-01-28", true, 500, 100, 2, 1).await.unwrap();
        db.record_completion(1, "TestUser", "2026-01-29", false, 300, 50, 1, 5).await.unwrap();
        db.record_completion(1, "TestUser", "2026-01-30", true, 400, 100, 3, 2).await.unwrap();

        let history = db.get_player_history(1, 10).await.unwrap();
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_total_summits() {
        let db = create_test_db().await;

        db.record_completion(1, "TestUser", "2026-01-28", true, 500, 100, 2, 1).await.unwrap();
        db.record_completion(1, "TestUser", "2026-01-29", false, 300, 50, 1, 5).await.unwrap();
        db.record_completion(1, "TestUser", "2026-01-30", true, 400, 100, 3, 2).await.unwrap();

        let summits = db.get_player_total_summits(1).await.unwrap();
        assert_eq!(summits, 2);
    }
}
