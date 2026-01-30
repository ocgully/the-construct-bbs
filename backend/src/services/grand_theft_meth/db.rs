use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub struct GtmDb {
    pool: SqlitePool,
}

impl GtmDb {
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
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS saves (
                user_id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                state_json TEXT NOT NULL,
                last_saved TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                handle TEXT NOT NULL,
                final_score INTEGER NOT NULL,
                days_played INTEGER NOT NULL,
                story_completed INTEGER NOT NULL,
                completed_at TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_completions_score
            ON completions(final_score DESC)
        "#).execute(&self.pool).await?;

        Ok(())
    }

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

    #[allow(dead_code)]
    pub async fn has_save(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT 1 FROM saves WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }

    pub async fn record_completion(
        &self,
        user_id: i64,
        handle: &str,
        final_score: i64,
        days_played: i64,
        story_completed: bool,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO completions (user_id, handle, final_score, days_played, story_completed, completed_at)
             VALUES (?, ?, ?, ?, ?, datetime('now', '-5 hours'))"
        )
        .bind(user_id)
        .bind(handle)
        .bind(final_score)
        .bind(days_played)
        .bind(story_completed as i64)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_leaderboard(&self, limit: i64) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
        let rows: Vec<(i64, String, i64, i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT
                RANK() OVER (ORDER BY final_score DESC) as rank,
                handle,
                final_score,
                days_played,
                completed_at,
                story_completed
            FROM completions
            ORDER BY final_score DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(rank, handle, score, days, completed, story)| {
            LeaderboardEntry {
                rank,
                handle,
                final_score: score,
                days_played: days,
                completed_at: completed,
                story_completed: story != 0,
            }
        }).collect())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub handle: String,
    pub final_score: i64,
    pub days_played: i64,
    pub completed_at: String,
    pub story_completed: bool,
}
