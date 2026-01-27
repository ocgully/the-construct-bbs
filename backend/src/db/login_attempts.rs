use sqlx::sqlite::SqlitePool;

/// Record a login attempt (success or failure) for a handle.
pub async fn record_login_attempt(
    pool: &SqlitePool,
    handle: &str,
    success: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO login_attempts (handle, success) VALUES (?, ?)",
    )
    .bind(handle)
    .bind(success as i32)
    .execute(pool)
    .await?;
    Ok(())
}

/// Count recent failed login attempts for a handle within the given time window.
pub async fn get_recent_failures(
    pool: &SqlitePool,
    handle: &str,
    window_minutes: u32,
) -> Result<i64, sqlx::Error> {
    let modifier = format!("-{} minutes", window_minutes);
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM login_attempts \
         WHERE handle = ? AND success = 0 \
         AND datetime(attempted_at) > datetime('now', ?)",
    )
    .bind(handle)
    .bind(&modifier)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Check if a handle is currently locked out due to too many failed attempts.
///
/// Returns true if recent failures >= max_attempts within the lockout window.
pub async fn is_locked_out(
    pool: &SqlitePool,
    handle: &str,
    max_attempts: u32,
    lockout_minutes: u32,
) -> Result<bool, sqlx::Error> {
    let failures = get_recent_failures(pool, handle, lockout_minutes).await?;
    Ok(failures >= max_attempts as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("connect to in-memory db");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS login_attempts (
                id INTEGER PRIMARY KEY,
                handle TEXT NOT NULL,
                ip_address TEXT,
                attempted_at TEXT NOT NULL DEFAULT (datetime('now')),
                success INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .expect("create login_attempts table");

        pool
    }

    #[tokio::test]
    async fn record_and_count_failures() {
        let pool = setup_test_db().await;

        record_login_attempt(&pool, "testuser", false).await.unwrap();
        record_login_attempt(&pool, "testuser", false).await.unwrap();
        record_login_attempt(&pool, "testuser", true).await.unwrap();

        let failures = get_recent_failures(&pool, "testuser", 15).await.unwrap();
        assert_eq!(failures, 2, "should count 2 failures (not the success)");
    }

    #[tokio::test]
    async fn is_locked_out_after_max_attempts() {
        let pool = setup_test_db().await;

        for _ in 0..5 {
            record_login_attempt(&pool, "lockme", false).await.unwrap();
        }

        let locked = is_locked_out(&pool, "lockme", 5, 15).await.unwrap();
        assert!(locked, "should be locked out after 5 failures");

        let not_locked = is_locked_out(&pool, "lockme", 10, 15).await.unwrap();
        assert!(!not_locked, "should not be locked out with higher threshold");
    }

    #[tokio::test]
    async fn different_handles_are_independent() {
        let pool = setup_test_db().await;

        for _ in 0..5 {
            record_login_attempt(&pool, "userA", false).await.unwrap();
        }

        let failures_b = get_recent_failures(&pool, "userB", 15).await.unwrap();
        assert_eq!(failures_b, 0, "userB should have no failures");
    }
}
