//! Realm of Ralnar - Database Layer
//!
//! SQLite database for game saves, completions, and leaderboards.

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct RalnarDb {
    pool: SqlitePool,
}

impl RalnarDb {
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
        // Active player saves
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Game completions history
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                ending_type TEXT NOT NULL,
                playtime_seconds INTEGER NOT NULL,
                shrines_found INTEGER NOT NULL DEFAULT 0,
                experience INTEGER NOT NULL DEFAULT 0,
                gold INTEGER NOT NULL DEFAULT 0,
                completed_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Leaderboard
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS leaderboard (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                ending_type TEXT NOT NULL,
                playtime_seconds INTEGER NOT NULL,
                shrines_found INTEGER NOT NULL DEFAULT 0,
                recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Index for leaderboard by playtime
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_leaderboard_playtime
            ON leaderboard(playtime_seconds ASC)
        "#).execute(&self.pool).await?;

        // Index for completions by user
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_user
            ON completions(user_id)
        "#).execute(&self.pool).await?;

        Ok(())
    }

    // ========== Save Operations ==========

    /// Save game state for a user
    pub async fn save_game(&self, user_id: i64, handle: &str, state_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO saves (user_id, handle, state_json, created_at, updated_at)
             VALUES (?, ?, ?,
                COALESCE((SELECT created_at FROM saves WHERE user_id = ?), datetime('now')),
                datetime('now'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(state_json)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load game state for a user
    pub async fn load_game(&self, user_id: i64) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT state_json FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// Delete a saved game
    pub async fn delete_save(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM saves WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Check if user has a saved game
    pub async fn has_save(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    // ========== Completion Operations ==========

    /// Record a game completion
    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        ending_type: &str,
        playtime_seconds: i64,
        shrines_found: i32,
        experience: i32,
        gold: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, ending_type, playtime_seconds, shrines_found, experience, gold)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(handle)
        .bind(ending_type)
        .bind(playtime_seconds)
        .bind(shrines_found)
        .bind(experience)
        .bind(gold)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get user's completion history
    pub async fn get_user_completions(&self, user_id: i64, limit: i64) -> Result<Vec<CompletionEntry>, sqlx::Error> {
        let rows: Vec<(String, i64, i64, i64, i64, String)> = sqlx::query_as(
            "SELECT ending_type, playtime_seconds, shrines_found, experience, gold, completed_at
             FROM completions
             WHERE user_id = ?
             ORDER BY completed_at DESC
             LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(ending, playtime, shrines, exp, gold, completed)| {
            CompletionEntry {
                ending_type: ending,
                playtime_seconds: playtime,
                shrines_found: shrines as i32,
                experience: exp as i32,
                gold: gold as i32,
                completed_at: completed,
            }
        }).collect())
    }

    // ========== Leaderboard Operations ==========

    /// Record a leaderboard entry (best completion times)
    pub async fn record_leaderboard(
        &self,
        user_id: i64,
        handle: &str,
        ending_type: &str,
        playtime_seconds: i64,
        shrines_found: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO leaderboard (user_id, handle, ending_type, playtime_seconds, shrines_found)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(handle)
        .bind(ending_type)
        .bind(playtime_seconds)
        .bind(shrines_found)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// Get the leaderboard (fastest completions)
    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY playtime_seconds ASC) as rank,
                handle,
                ending_type,
                playtime_seconds,
                shrines_found,
                recorded_at
            FROM leaderboard
            WHERE ending_type != 'death'
            ORDER BY playtime_seconds ASC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, ending, playtime, shrines, recorded)| {
            LeaderboardEntry {
                rank,
                handle,
                ending_type: ending,
                playtime_seconds: playtime,
                shrines_found: shrines as i32,
                completed_at: recorded,
            }
        }).collect())
    }

    /// Get leaderboard for a specific ending type
    pub async fn get_leaderboard_by_ending(&self, ending_type: &str, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, String, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY playtime_seconds ASC) as rank,
                handle,
                ending_type,
                playtime_seconds,
                shrines_found,
                recorded_at
            FROM leaderboard
            WHERE ending_type = ?
            ORDER BY playtime_seconds ASC
            LIMIT ?
            "#
        )
        .bind(ending_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, ending, playtime, shrines, recorded)| {
            LeaderboardEntry {
                rank,
                handle,
                ending_type: ending,
                playtime_seconds: playtime,
                shrines_found: shrines as i32,
                completed_at: recorded,
            }
        }).collect())
    }

    /// Get user's best time
    pub async fn get_user_best_time(&self, user_id: i64) -> Result<Option<LeaderboardEntry>, sqlx::Error> {
        let row: Option<(String, String, i64, i64, String)> = sqlx::query_as(
            r#"
            SELECT handle, ending_type, playtime_seconds, shrines_found, recorded_at
            FROM leaderboard
            WHERE user_id = ? AND ending_type != 'death'
            ORDER BY playtime_seconds ASC
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(handle, ending, playtime, shrines, recorded)| {
            LeaderboardEntry {
                rank: 0, // Rank not computed for single entry
                handle,
                ending_type: ending,
                playtime_seconds: playtime,
                shrines_found: shrines as i32,
                completed_at: recorded,
            }
        }))
    }
}

/// Leaderboard entry for display
#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub ending_type: String,
    pub playtime_seconds: i64,
    pub shrines_found: i32,
    pub completed_at: String,
}

/// Completion history entry
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    pub ending_type: String,
    pub playtime_seconds: i64,
    pub shrines_found: i32,
    pub experience: i32,
    pub gold: i32,
    pub completed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> RalnarDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_ralnar.db");
        RalnarDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_db() {
        let _db = create_test_db().await;
    }

    #[tokio::test]
    async fn test_save_and_load_game() {
        let db = create_test_db().await;

        let state_json = r#"{"user_id":1,"handle":"TestPlayer","location":"village"}"#;
        db.save_game(1, "TestPlayer", state_json).await.unwrap();

        let loaded = db.load_game(1).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), state_json);
    }

    #[tokio::test]
    async fn test_load_nonexistent_save() {
        let db = create_test_db().await;
        let loaded = db.load_game(999).await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_has_save() {
        let db = create_test_db().await;

        assert!(!db.has_save(1).await.unwrap());

        db.save_game(1, "TestPlayer", "{}").await.unwrap();

        assert!(db.has_save(1).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_save() {
        let db = create_test_db().await;

        db.save_game(1, "TestPlayer", "{}").await.unwrap();
        assert!(db.has_save(1).await.unwrap());

        db.delete_save(1).await.unwrap();
        assert!(!db.has_save(1).await.unwrap());
    }

    #[tokio::test]
    async fn test_record_completion() {
        let db = create_test_db().await;

        let id = db.record_completion(1, "TestPlayer", "victory", 3600, 5, 100, 500).await.unwrap();
        assert!(id > 0);

        let completions = db.get_user_completions(1, 10).await.unwrap();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].ending_type, "victory");
        assert_eq!(completions[0].playtime_seconds, 3600);
        assert_eq!(completions[0].shrines_found, 5);
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let db = create_test_db().await;

        // Add entries with different times
        db.record_leaderboard(1, "Fast", "victory", 1800, 5).await.unwrap();
        db.record_leaderboard(2, "Medium", "victory", 3600, 4).await.unwrap();
        db.record_leaderboard(3, "Slow", "victory", 7200, 3).await.unwrap();

        let leaders = db.get_leaderboard(10).await.unwrap();
        assert_eq!(leaders.len(), 3);
        assert_eq!(leaders[0].handle, "Fast");
        assert_eq!(leaders[0].rank, 1);
        assert_eq!(leaders[1].handle, "Medium");
        assert_eq!(leaders[2].handle, "Slow");
    }

    #[tokio::test]
    async fn test_leaderboard_excludes_deaths() {
        let db = create_test_db().await;

        db.record_leaderboard(1, "Victor", "victory", 3600, 5).await.unwrap();
        db.record_leaderboard(2, "Deceased", "death", 600, 1).await.unwrap();

        let leaders = db.get_leaderboard(10).await.unwrap();
        assert_eq!(leaders.len(), 1);
        assert_eq!(leaders[0].handle, "Victor");
    }

    #[tokio::test]
    async fn test_user_best_time() {
        let db = create_test_db().await;

        db.record_leaderboard(1, "Player", "victory", 7200, 3).await.unwrap();
        db.record_leaderboard(1, "Player", "victory", 3600, 5).await.unwrap();
        db.record_leaderboard(1, "Player", "victory", 5400, 4).await.unwrap();

        let best = db.get_user_best_time(1).await.unwrap();
        assert!(best.is_some());
        assert_eq!(best.unwrap().playtime_seconds, 3600);
    }

    #[tokio::test]
    async fn test_leaderboard_by_ending() {
        let db = create_test_db().await;

        db.record_leaderboard(1, "Good", "good_ending", 3600, 5).await.unwrap();
        db.record_leaderboard(2, "Evil", "evil_ending", 2400, 2).await.unwrap();
        db.record_leaderboard(3, "True", "true_ending", 4800, 7).await.unwrap();

        let good_leaders = db.get_leaderboard_by_ending("good_ending", 10).await.unwrap();
        assert_eq!(good_leaders.len(), 1);
        assert_eq!(good_leaders[0].handle, "Good");

        let evil_leaders = db.get_leaderboard_by_ending("evil_ending", 10).await.unwrap();
        assert_eq!(evil_leaders.len(), 1);
        assert_eq!(evil_leaders[0].handle, "Evil");
    }
}
