use sqlx::sqlite::SqlitePool;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Message {
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: i64,
    pub subject: String,
    pub body: String,
    pub sent_at: String,
    pub is_read: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct InboxEntry {
    pub id: i64,
    pub sender_id: i64,
    pub subject: String,
    pub sent_at: String,
    pub is_read: i32,
}

/// Create a new message.
/// Validates that sender_id != recipient_id.
/// Normalizes body newlines to \n only.
pub async fn create_message(
    pool: &SqlitePool,
    sender_id: i64,
    recipient_id: i64,
    subject: &str,
    body: &str,
) -> Result<i64, sqlx::Error> {
    // Validate sender != recipient
    if sender_id == recipient_id {
        return Err(sqlx::Error::Protocol(
            "Cannot send message to yourself".into(),
        ));
    }

    // Normalize newlines: replace \r\n and \r with \n
    let normalized_body = body.replace("\r\n", "\n").replace('\r', "\n");

    sqlx::query(
        "INSERT INTO messages (sender_id, recipient_id, subject, body) VALUES (?, ?, ?, ?)",
    )
    .bind(sender_id)
    .bind(recipient_id)
    .bind(subject)
    .bind(&normalized_body)
    .execute(pool)
    .await?;

    // Get the last inserted row ID
    let row: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}

/// Get a page of inbox entries for a user.
/// Returns InboxEntry (without body field).
pub async fn get_inbox_page(
    pool: &SqlitePool,
    user_id: i64,
    page: i64,
    page_size: i64,
) -> Result<Vec<InboxEntry>, sqlx::Error> {
    let offset = page * page_size;

    let entries = sqlx::query_as::<_, InboxEntry>(
        "SELECT id, sender_id, subject, sent_at, is_read
         FROM messages
         WHERE recipient_id = ?
         ORDER BY sent_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(entries)
}

/// Get total count of messages in user's inbox.
pub async fn get_inbox_count(pool: &SqlitePool, user_id: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE recipient_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}

/// Get count of unread messages in user's inbox.
pub async fn get_unread_count(pool: &SqlitePool, user_id: i64) -> Result<i64, sqlx::Error> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM messages WHERE recipient_id = ? AND is_read = 0")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    Ok(row.0)
}

/// Get a specific message by ID.
/// Ownership check built into query - returns None if user doesn't own message.
pub async fn get_message_by_id(
    pool: &SqlitePool,
    message_id: i64,
    user_id: i64,
) -> Result<Option<Message>, sqlx::Error> {
    let message = sqlx::query_as::<_, Message>(
        "SELECT id, sender_id, recipient_id, subject, body, sent_at, is_read
         FROM messages
         WHERE id = ? AND recipient_id = ?",
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(message)
}

/// Mark a message as read.
/// Ownership check built into query.
pub async fn mark_message_read(
    pool: &SqlitePool,
    message_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE messages SET is_read = 1 WHERE id = ? AND recipient_id = ?")
        .bind(message_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Delete a message.
/// Ownership check built into query.
/// Returns true if a message was deleted, false if not found or not owned by user.
pub async fn delete_message(
    pool: &SqlitePool,
    message_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM messages WHERE id = ? AND recipient_id = ?")
        .bind(message_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get handles for a list of sender IDs.
/// Returns tuples of (id, handle).
/// Deduplicates sender_ids before querying.
pub async fn get_sender_handles(
    pool: &SqlitePool,
    sender_ids: &[i64],
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    // Deduplicate sender IDs
    let mut unique_ids: Vec<i64> = sender_ids.to_vec();
    unique_ids.sort_unstable();
    unique_ids.dedup();

    let mut results = Vec::new();

    // Query each ID individually (sqlx doesn't easily support IN clauses with slices)
    for id in unique_ids {
        let row: Option<(i64, String)> =
            sqlx::query_as("SELECT id, handle FROM users WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await?;

        if let Some(result) = row {
            results.push(result);
        }
    }

    Ok(results)
}

/// Check if a user's mailbox is at or above the limit.
/// Returns true if count >= limit.
pub async fn check_mailbox_full(
    pool: &SqlitePool,
    recipient_id: i64,
    limit: i64,
) -> Result<bool, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE recipient_id = ?")
        .bind(recipient_id)
        .fetch_one(pool)
        .await?;

    Ok(row.0 >= limit)
}
