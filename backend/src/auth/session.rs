use sqlx::sqlite::SqlitePool;

/// Generate a cryptographically secure session token (UUID v4).
pub fn generate_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Create a new session for a user, returning the session token.
pub async fn create_session(
    pool: &SqlitePool,
    user_id: i64,
    node_id: Option<i32>,
    duration_hours: u32,
) -> Result<String, sqlx::Error> {
    let token = generate_token();
    let expires_at = format!("+{} hours", duration_hours);

    sqlx::query(
        "INSERT INTO sessions (token, user_id, node_id, expires_at) \
         VALUES (?, ?, ?, datetime('now', '-5 hours', ?))",
    )
    .bind(&token)
    .bind(user_id)
    .bind(node_id)
    .bind(&expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

/// Validate a session token. Returns `Some(user_id)` if the session is valid
/// and not expired, `None` otherwise. Also updates `last_activity` on success.
pub async fn validate_session(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT user_id FROM sessions \
         WHERE token = ? AND datetime(expires_at) > datetime('now', '-5 hours')",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    if let Some((user_id,)) = row {
        // Update last activity timestamp
        sqlx::query("UPDATE sessions SET last_activity = datetime('now', '-5 hours') WHERE token = ?")
            .bind(token)
            .execute(pool)
            .await?;
        Ok(Some(user_id))
    } else {
        Ok(None)
    }
}

/// Delete a specific session by token (logout).
pub async fn delete_session(pool: &SqlitePool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete all sessions for a user (force logout everywhere).
#[allow(dead_code)]
pub async fn delete_user_sessions(pool: &SqlitePool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Remove all expired sessions. Returns number of rows deleted.
#[allow(dead_code)]
pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE datetime(expires_at) < datetime('now', '-5 hours')")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Check if a user already has an active (non-expired) session.
/// Returns the token if one exists.
pub async fn get_active_session_for_user(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT token FROM sessions \
         WHERE user_id = ? AND datetime(expires_at) > datetime('now', '-5 hours') \
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(token,)| token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_is_valid_uuid_v4() {
        let token = generate_token();
        // UUID v4 format: xxxxxxxx-xxxx-4xxx-[89ab]xxx-xxxxxxxxxxxx
        let parsed = uuid::Uuid::parse_str(&token);
        assert!(parsed.is_ok(), "token should be valid UUID");
        let uuid = parsed.unwrap();
        assert_eq!(
            uuid.get_version(),
            Some(uuid::Version::Random),
            "token should be UUID v4"
        );
    }

    #[test]
    fn tokens_are_unique() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2, "each token should be unique");
    }
}
