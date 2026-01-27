use sqlx::sqlite::SqlitePool;

/// Store a verification code for the given email and type.
///
/// Deletes any existing codes for this email+type first (only one active code
/// per email per type), then inserts the new code with an expiry.
pub async fn store_verification_code(
    pool: &SqlitePool,
    email: &str,
    code: &str,
    code_type: &str,
    expiry_hours: u32,
) -> Result<(), sqlx::Error> {
    // Remove any existing codes for this email+type
    sqlx::query("DELETE FROM verification_codes WHERE email = ? AND code_type = ?")
        .bind(email)
        .bind(code_type)
        .execute(pool)
        .await?;

    // Insert new code with expiry
    let expiry_modifier = format!("+{} hours", expiry_hours);
    sqlx::query(
        "INSERT INTO verification_codes (email, code, code_type, expires_at) \
         VALUES (?, ?, ?, datetime('now', '-5 hours', ?))",
    )
    .bind(email)
    .bind(code)
    .bind(code_type)
    .bind(&expiry_modifier)
    .execute(pool)
    .await?;

    Ok(())
}

/// Validate a verification code.
///
/// Checks that the code exists, matches the email and type, has not been used,
/// and has not expired. If valid, marks the code as used and (for registration
/// type) sets the user's email_verified flag to 1.
///
/// Returns `true` if the code was valid and consumed, `false` otherwise.
pub async fn validate_verification_code(
    pool: &SqlitePool,
    email: &str,
    code: &str,
    code_type: &str,
) -> Result<bool, sqlx::Error> {
    // Look for a matching, unused, non-expired code
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM verification_codes \
         WHERE email = ? AND code = ? AND code_type = ? \
         AND used = 0 AND datetime(expires_at) > datetime('now', '-5 hours')",
    )
    .bind(email)
    .bind(code)
    .bind(code_type)
    .fetch_optional(pool)
    .await?;

    let code_id = match row {
        Some((id,)) => id,
        None => return Ok(false),
    };

    // Mark code as used
    sqlx::query("UPDATE verification_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await?;

    // For registration type, also mark the user's email as verified
    if code_type == "registration" {
        sqlx::query("UPDATE users SET email_verified = 1 WHERE email = ?")
            .bind(email)
            .execute(pool)
            .await?;
    }

    Ok(true)
}

/// Delete all expired verification codes.
///
/// Returns the number of rows deleted.
pub async fn cleanup_expired_codes(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM verification_codes WHERE datetime(expires_at) <= datetime('now', '-5 hours')")
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
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

        // Create tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                handle TEXT UNIQUE NOT NULL,
                handle_lower TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                email_verified INTEGER NOT NULL DEFAULT 0,
                password_hash TEXT NOT NULL,
                real_name TEXT,
                location TEXT,
                signature TEXT,
                bio TEXT,
                user_level TEXT NOT NULL DEFAULT 'User',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_login TEXT,
                total_logins INTEGER NOT NULL DEFAULT 0,
                total_time_minutes INTEGER NOT NULL DEFAULT 0,
                messages_sent INTEGER NOT NULL DEFAULT 0,
                games_played INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .expect("create users table");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS verification_codes (
                id INTEGER PRIMARY KEY,
                email TEXT NOT NULL,
                code TEXT NOT NULL,
                code_type TEXT NOT NULL DEFAULT 'registration',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at TEXT NOT NULL,
                used INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .expect("create verification_codes table");

        pool
    }

    #[tokio::test]
    async fn store_and_validate_code() {
        let pool = setup_test_db().await;

        store_verification_code(&pool, "user@example.com", "123456", "registration", 24)
            .await
            .expect("store code");

        let valid =
            validate_verification_code(&pool, "user@example.com", "123456", "registration")
                .await
                .expect("validate code");
        assert!(valid, "correct code should validate");
    }

    #[tokio::test]
    async fn wrong_code_fails_validation() {
        let pool = setup_test_db().await;

        store_verification_code(&pool, "user@example.com", "123456", "registration", 24)
            .await
            .expect("store code");

        let valid =
            validate_verification_code(&pool, "user@example.com", "999999", "registration")
                .await
                .expect("validate code");
        assert!(!valid, "wrong code should not validate");
    }

    #[tokio::test]
    async fn used_code_fails_second_validation() {
        let pool = setup_test_db().await;

        store_verification_code(&pool, "user@example.com", "123456", "registration", 24)
            .await
            .expect("store code");

        // First validation succeeds
        let valid =
            validate_verification_code(&pool, "user@example.com", "123456", "registration")
                .await
                .expect("validate first");
        assert!(valid);

        // Second validation fails (code already used)
        let valid2 =
            validate_verification_code(&pool, "user@example.com", "123456", "registration")
                .await
                .expect("validate second");
        assert!(!valid2, "used code should not validate again");
    }

    #[tokio::test]
    async fn store_replaces_existing_code_for_same_email_and_type() {
        let pool = setup_test_db().await;

        store_verification_code(&pool, "user@example.com", "111111", "registration", 24)
            .await
            .expect("store first code");

        store_verification_code(&pool, "user@example.com", "222222", "registration", 24)
            .await
            .expect("store second code");

        // Old code should no longer work
        let old_valid =
            validate_verification_code(&pool, "user@example.com", "111111", "registration")
                .await
                .expect("validate old code");
        assert!(!old_valid, "old code should be deleted");

        // New code should work
        let new_valid =
            validate_verification_code(&pool, "user@example.com", "222222", "registration")
                .await
                .expect("validate new code");
        assert!(new_valid, "new code should validate");
    }

    #[tokio::test]
    async fn expired_code_fails_validation() {
        let pool = setup_test_db().await;

        // Insert a code that is already expired (well before EST comparison point)
        sqlx::query(
            "INSERT INTO verification_codes (email, code, code_type, expires_at) \
             VALUES (?, ?, ?, datetime('now', '-6 hours'))",
        )
        .bind("user@example.com")
        .bind("123456")
        .bind("registration")
        .execute(&pool)
        .await
        .expect("insert expired code");

        let valid =
            validate_verification_code(&pool, "user@example.com", "123456", "registration")
                .await
                .expect("validate expired code");
        assert!(!valid, "expired code should not validate");
    }

    #[tokio::test]
    async fn cleanup_removes_expired_codes() {
        let pool = setup_test_db().await;

        // Insert an expired code (well before EST comparison point)
        sqlx::query(
            "INSERT INTO verification_codes (email, code, code_type, expires_at) \
             VALUES (?, ?, ?, datetime('now', '-6 hours'))",
        )
        .bind("expired@example.com")
        .bind("111111")
        .bind("registration")
        .execute(&pool)
        .await
        .expect("insert expired code");

        // Insert a valid code
        store_verification_code(&pool, "valid@example.com", "222222", "registration", 24)
            .await
            .expect("store valid code");

        let deleted = cleanup_expired_codes(&pool).await.expect("cleanup");
        assert_eq!(deleted, 1, "should delete 1 expired code");

        // Valid code should still be there
        let valid =
            validate_verification_code(&pool, "valid@example.com", "222222", "registration")
                .await
                .expect("validate valid code");
        assert!(valid, "non-expired code should survive cleanup");
    }

    #[tokio::test]
    async fn registration_validation_marks_user_email_verified() {
        let pool = setup_test_db().await;

        // Create a user first
        sqlx::query(
            "INSERT INTO users (handle, handle_lower, email, password_hash) \
             VALUES (?, ?, ?, ?)",
        )
        .bind("TestUser")
        .bind("testuser")
        .bind("user@example.com")
        .bind("hash")
        .execute(&pool)
        .await
        .expect("create user");

        // Store and validate registration code
        store_verification_code(&pool, "user@example.com", "123456", "registration", 24)
            .await
            .expect("store code");

        validate_verification_code(&pool, "user@example.com", "123456", "registration")
            .await
            .expect("validate code");

        // Check that user's email_verified is now 1
        let row: (i32,) =
            sqlx::query_as("SELECT email_verified FROM users WHERE email = ?")
                .bind("user@example.com")
                .fetch_one(&pool)
                .await
                .expect("fetch user");
        assert_eq!(row.0, 1, "email_verified should be 1 after registration verification");
    }

    #[tokio::test]
    async fn different_code_type_does_not_validate() {
        let pool = setup_test_db().await;

        store_verification_code(&pool, "user@example.com", "123456", "registration", 24)
            .await
            .expect("store code");

        let valid =
            validate_verification_code(&pool, "user@example.com", "123456", "password_reset")
                .await
                .expect("validate with wrong type");
        assert!(!valid, "code should not validate for different type");
    }
}
