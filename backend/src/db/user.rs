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
    pub daily_time_used: i32,
    pub banked_time: i32,
    pub last_daily_reset: Option<String>,
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
        "UPDATE users SET last_login = datetime('now', '-5 hours'), total_logins = total_logins + 1 WHERE id = ?",
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

/// Update a specific profile field for a user.
///
/// Supported fields: real_name, location, signature, bio.
/// Returns an error if the field name is not recognized.
pub async fn update_user_field(
    pool: &SqlitePool,
    user_id: i64,
    field: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    let query = match field {
        "real_name" => "UPDATE users SET real_name = ? WHERE id = ?",
        "location" => "UPDATE users SET location = ? WHERE id = ?",
        "signature" => "UPDATE users SET signature = ? WHERE id = ?",
        "bio" => "UPDATE users SET bio = ? WHERE id = ?",
        _ => {
            return Err(sqlx::Error::Protocol(format!(
                "Unknown profile field: {}",
                field
            )));
        }
    };
    sqlx::query(query)
        .bind(value)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_user(pool: &SqlitePool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn check_daily_reset(pool: &SqlitePool, user_id: i64) -> Result<bool, sqlx::Error> {
    let row: (i32,) = sqlx::query_as(
        "SELECT CASE
            WHEN last_daily_reset IS NULL THEN 1
            WHEN date(last_daily_reset) < date('now', '-5 hours') THEN 1
            ELSE 0
        END FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 == 1)
}

pub async fn reset_daily_time(
    pool: &SqlitePool,
    user_id: i64,
    daily_limit: i64,
    bank_cap: i64,
) -> Result<i64, sqlx::Error> {
    let row: (i32, i32) = sqlx::query_as(
        "SELECT daily_time_used, banked_time FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let used = row.0 as i64;
    let current_bank = row.1 as i64;
    let unused = (daily_limit - used).max(0);
    let new_bank = (current_bank + unused).min(bank_cap);
    let banked_amount = new_bank - current_bank;

    sqlx::query(
        "UPDATE users SET daily_time_used = 0, banked_time = ?, last_daily_reset = datetime('now', '-5 hours') WHERE id = ?"
    )
    .bind(new_bank)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(banked_amount)
}

pub async fn update_daily_time_used(
    pool: &SqlitePool,
    user_id: i64,
    minutes: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET daily_time_used = daily_time_used + ? WHERE id = ?")
        .bind(minutes)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_user_time_info(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<(i64, i64, i64), sqlx::Error> {
    let row: (i32, i32) = sqlx::query_as(
        "SELECT daily_time_used, banked_time FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok((row.0 as i64, row.1 as i64, 0)) // daily_limit filled by caller from config
}

pub async fn withdraw_banked_time(
    pool: &SqlitePool,
    user_id: i64,
    minutes: i64,
) -> Result<i64, sqlx::Error> {
    let row: (i32,) = sqlx::query_as(
        "SELECT banked_time FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let available = row.0 as i64;
    let withdraw = minutes.min(available);

    sqlx::query(
        "UPDATE users SET banked_time = banked_time - ?, daily_time_used = CASE WHEN daily_time_used >= ? THEN daily_time_used - ? ELSE 0 END WHERE id = ?"
    )
    .bind(withdraw)
    .bind(withdraw)
    .bind(withdraw)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(withdraw)
}
