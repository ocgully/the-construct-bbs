use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub handle: String,
    pub handle_lower: String,
    pub email: String,
    pub email_verified: i32,
    pub password_hash: String,
    pub real_name: Option<String>,
    pub location: Option<String>,
    pub signature: Option<String>,
    pub bio: Option<String>,
    pub user_level: String,
    pub created_at: String,
    pub last_login: Option<String>,
    pub total_logins: i32,
    pub total_time_minutes: i32,
    pub messages_sent: i32,
    pub games_played: i32,
}

pub async fn create_user(
    pool: &SqlitePool,
    handle: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let handle_lower = handle.to_lowercase();

    sqlx::query(
        "INSERT INTO users (handle, handle_lower, email, password_hash) VALUES (?, ?, ?, ?)",
    )
    .bind(handle)
    .bind(&handle_lower)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await?;

    // Fetch the created user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE handle_lower = ?")
        .bind(&handle_lower)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub async fn find_user_by_handle(
    pool: &SqlitePool,
    handle: &str,
) -> Result<Option<User>, sqlx::Error> {
    let handle_lower = handle.to_lowercase();
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE handle_lower = ?")
        .bind(&handle_lower)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn find_user_by_email(
    pool: &SqlitePool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn find_user_by_id(pool: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn update_last_login(pool: &SqlitePool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET last_login = datetime('now'), total_logins = total_logins + 1 WHERE id = ?",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_user_time(
    pool: &SqlitePool,
    user_id: i64,
    minutes: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET total_time_minutes = total_time_minutes + ? WHERE id = ?")
        .bind(minutes)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn handle_exists(pool: &SqlitePool, handle: &str) -> Result<bool, sqlx::Error> {
    let handle_lower = handle.to_lowercase();
    let row: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE handle_lower = ?")
            .bind(&handle_lower)
            .fetch_one(pool)
            .await?;
    Ok(row.0 > 0)
}

pub async fn email_exists(pool: &SqlitePool, email: &str) -> Result<bool, sqlx::Error> {
    let row: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(row.0 > 0)
}

pub async fn delete_user(pool: &SqlitePool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
