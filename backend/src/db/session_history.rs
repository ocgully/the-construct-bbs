use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct SessionHistoryEntry {
    pub id: i64,
    pub user_id: i64,
    pub handle: String,
    pub login_time: String,
    pub logout_time: Option<String>,
    pub duration_minutes: i32,
}

pub async fn insert_session_history(
    pool: &SqlitePool,
    user_id: i64,
    handle: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO session_history (user_id, handle) VALUES (?, ?)"
    )
    .bind(user_id)
    .bind(handle)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn update_session_history_logout(
    pool: &SqlitePool,
    session_history_id: i64,
    duration_minutes: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE session_history SET logout_time = datetime('now', '-5 hours'), duration_minutes = ? WHERE id = ?"
    )
    .bind(duration_minutes)
    .bind(session_history_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_last_callers(
    pool: &SqlitePool,
    limit: i32,
) -> Result<Vec<SessionHistoryEntry>, sqlx::Error> {
    sqlx::query_as::<_, SessionHistoryEntry>(
        "SELECT id, user_id, handle, login_time, logout_time, duration_minutes
         FROM session_history
         ORDER BY login_time DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}
