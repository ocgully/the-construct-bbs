# Phase 5: Email System - Research

**Researched:** 2026-01-27
**Domain:** Inter-user private messaging with SQLite database backend
**Confidence:** HIGH

## Summary

Phase 5 implements a classic BBS-style private mail system where users can send messages to other users by handle, read their inbox with unread indicators, reply with quoted text, and delete messages. This research focused on database schema design for messaging systems, SQLite foreign key constraints for data integrity, pagination patterns for inbox display, and authentic BBS mail UX conventions.

The standard approach uses a messages table with sender/recipient foreign keys to the users table, CASCADE delete for orphan prevention, and a boolean or enum for read/unread tracking. Pagination follows LIMIT/OFFSET patterns with page size calculated from terminal rows. BBS mail traditionally used line-by-line editors with slash commands (/s to send, /a to abort) and quoted replies with > prefix on each line.

Key findings: SQLite foreign keys must be explicitly enabled with PRAGMA foreign_keys = ON; CASCADE constraints prevent orphaned messages; pagination is already implemented in the codebase via Pager; the existing sentinel service pattern (__whos_online__, etc.) provides a proven model for state-based mail views; and the project already uses character-by-character input with handle_char for masked password entry, which extends naturally to line-by-line mail composition.

**Primary recommendation:** Extend existing patterns - use sentinel services for mail views (__mail_read__, __mail_compose__), add messages table with ON DELETE CASCADE, leverage existing Pager for inbox pagination, and implement line-by-line editor state machine similar to LoginFlow/RegistrationFlow character handling.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| sqlx | 0.8+ | Async SQLite queries | Already in project; string-based queries match Phase 2 decision |
| tokio::sync::mpsc | 1.x | WebSocket output channel | Already in project; proven pattern for session output |
| chrono | N/A | Date/time parsing | Already used implicitly for datetime() functions; SQLite datetime('now', '-5 hours') pattern established |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.x | Config serialization | Already in project; for mailbox_size_limit in config.toml |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Boolean is_read | Enum MessageStatus | Boolean sufficient for BBS mail (only 2 states: new/read); enum adds complexity without benefit for this phase |
| Separate inbox/sent tables | Single messages table | Single table with sender_id/recipient_id simpler; queries just filter by user_id |
| External pagination library | Manual LIMIT/OFFSET | Existing Pager already handles terminal-based pagination; no need for external crate |

**Installation:**
No new dependencies required. All functionality uses existing crates (sqlx, tokio, serde).

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── db/
│   └── messages.rs         # Message CRUD operations
├── services/
│   ├── mail.rs            # Mail service (inbox list, compose, read)
│   └── mod.rs             # Register mail service
└── websocket/
    └── session.rs         # Add mail sentinel services and state
```

### Pattern 1: Messages Table with Foreign Keys
**What:** Single messages table with sender_id and recipient_id foreign keys to users table, using ON DELETE CASCADE to prevent orphaned messages.

**When to use:** All private messaging systems where messages have clear sender/recipient relationships.

**Example:**
```sql
-- Source: SQLite Foreign Keys documentation + project's existing schema.sql patterns
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY,
    sender_id INTEGER NOT NULL,
    recipient_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    sent_at TEXT NOT NULL DEFAULT (datetime('now', '-5 hours')),
    is_read INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipient_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_id, sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id, sent_at DESC);
```

**Critical detail:** SQLite foreign keys are disabled by default. Must call `PRAGMA foreign_keys = ON;` at pool initialization (likely already done in pool.rs).

### Pattern 2: Sentinel Service Views
**What:** Use current_service sentinel strings like `__mail_inbox__`, `__mail_read__`, `__mail_compose__` to route input without creating new AuthState variants or Service trait impls.

**When to use:** For views that need async database access but aren't traditional "services" (like profile view, Who's Online, User Lookup).

**Example:**
```rust
// Source: Existing pattern from websocket/session.rs Phase 4 implementation
// In handle_authenticated_input:
if service_name == "__mail_inbox__" {
    // Handle inbox navigation (number to read, N/P for pagination, Q to quit)
    return Ok(());
}

if service_name == "__mail_read__" {
    // Handle message reading actions (R=reply, D=delete, N=next, Q=quit)
    return Ok(());
}

if service_name == "__mail_compose__" {
    // Handle line-by-line composition (/s, /a, /h, /l)
    return Ok(());
}
```

**Why:** Proven pattern from Phase 4; keeps Session struct lightweight; allows async DB access without Service trait constraints.

### Pattern 3: Line-by-Line Editor State Machine
**What:** Track composition state (To, Subject, Body, BodyBuffer) similar to LoginFlow/RegistrationFlow character handling. Buffer lines until /s (send) or /a (abort).

**When to use:** BBS-style text editors where users type line-by-line with Enter to confirm each line.

**Example:**
```rust
// Source: Existing registration.rs and login.rs patterns
enum ComposeState {
    PromptTo,
    InputTo { buffer: String },
    PromptSubject { recipient_id: i64, recipient_handle: String },
    InputSubject { recipient_id: i64, recipient_handle: String, buffer: String },
    PromptBody { recipient_id: i64, recipient_handle: String, subject: String },
    InputBody { recipient_id: i64, recipient_handle: String, subject: String, lines: Vec<String>, current_line: String },
}

// handle_char processes each character (including backspace)
// handle_input processes completed lines (after Enter key)
```

**Key insight:** Registration and Login already demonstrate character-by-character echo with input_buffer. Same pattern applies to mail composition.

### Pattern 4: Inbox Pagination with LIMIT/OFFSET
**What:** Query messages with LIMIT and OFFSET based on page size (15 messages per page per context). Use Pager for consistent page calculations.

**When to use:** Any list view that exceeds terminal height (25 rows - 2 reserved = 23 lines per page).

**Example:**
```rust
// Source: Existing paging.rs implementation + SQLite LIMIT/OFFSET
let page_size = 15; // From context: 15 messages per page
let offset = page_number * page_size;

let messages: Vec<Message> = sqlx::query_as(
    "SELECT id, sender_id, recipient_id, subject, sent_at, is_read
     FROM messages
     WHERE recipient_id = ?
     ORDER BY sent_at DESC
     LIMIT ? OFFSET ?"
)
.bind(user_id)
.bind(page_size as i64)
.bind(offset as i64)
.fetch_all(pool)
.await?;

// Count total for pagination
let total: (i32,) = sqlx::query_as(
    "SELECT COUNT(*) FROM messages WHERE recipient_id = ?"
)
.bind(user_id)
.fetch_one(pool)
.await?;
```

**Warning:** OFFSET performance degrades with large offsets (rare for BBS inbox sizes). Consider cursor-based pagination if mailboxes exceed 1000+ messages.

### Pattern 5: Quoted Reply with > Prefix
**What:** When replying, fetch original message body, prefix each line with "> ", auto-populate subject with "Re: [original]", and place cursor at bottom for user's reply.

**When to use:** All reply functionality (universal email/message convention).

**Example:**
```rust
// Source: Email quoting convention (Wikipedia Posting style + project's ANSI patterns)
fn quote_message(original_body: &str) -> String {
    original_body
        .lines()
        .map(|line| format!("> {}", line))
        .collect::<Vec<_>>()
        .join("\r\n")
}

// In reply flow:
let quoted = quote_message(&original_message.body);
writer.writeln(&quoted);
writer.writeln(""); // Blank line separator
writer.writeln("Type your reply below. /s to send, /a to abort:");
```

### Anti-Patterns to Avoid

- **Self-mail blocking only at UI level:** Must validate recipient != sender at BOTH input validation AND database insert to prevent bypasses.
- **Reading messages without transaction:** Mark-as-read and fetch-body should be atomic to prevent race conditions with concurrent sessions.
- **Unbounded message body length:** Even though context says "no message length limit", database TEXT columns in SQLite have 1GB max; consider practical limit (e.g., 64KB) to prevent abuse.
- **Deleting messages without CASCADE:** If CASCADE not set, deleting a user leaves orphaned messages; must be in schema, not application logic.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terminal pagination | Custom line-counting logic | Existing Pager struct | Already handles terminal rows, page splitting, [More] prompts, and page state |
| Character-by-character input | Raw WebSocket char parsing | Existing handle_char pattern | Registration/Login already implement backspace handling, buffer management, and echo |
| ANSI box-drawing | Manual escape sequences | Existing AnsiWriter + profile.rs patterns | Profile cards already demonstrate CP437 double-line boxes, color palettes, 80-column layout |
| User validation | Manual handle lookups | Existing find_user_by_handle | db/user.rs already provides case-insensitive handle_lower lookups |
| Date/time formatting | chrono parsing | Existing format_date/format_datetime in profile.rs | Already handles EST timezone ("datetime('now', '-5 hours')") and display formatting |

**Key insight:** Phase 5 extends existing patterns, not new infrastructure. 90% of needed functionality already exists in Phases 1-4 (Pager, ANSI rendering, character input, database queries, sentinel services).

## Common Pitfalls

### Pitfall 1: Foreign Keys Disabled by Default
**What goes wrong:** Messages table has `FOREIGN KEY ... ON DELETE CASCADE` in schema, but CASCADE deletes don't happen. Deleting a user leaves orphaned messages.

**Why it happens:** SQLite disables foreign key enforcement by default for historical compatibility. Schema alone isn't enough.

**How to avoid:** Call `PRAGMA foreign_keys = ON;` when initializing the connection pool. Check if pool.rs already does this (likely yes from Phase 2).

**Warning signs:** Integration tests delete users but messages table still has records; foreign key violations aren't raised when expected.

### Pitfall 2: Mailbox Quota Not Enforced at Database Level
**What goes wrong:** User bypasses mailbox size limit by opening multiple sessions or exploiting race conditions between count check and insert.

**Why it happens:** Application-level checks (SELECT COUNT then INSERT) aren't atomic without transactions.

**How to avoid:** Two approaches:
1. Use transaction with SERIALIZABLE isolation + recheck count inside transaction
2. Add CHECK constraint or trigger at database level (more complex in SQLite)

**Warning signs:** Users have more messages than mailbox_size_limit; concurrent sends bypass quota.

### Pitfall 3: Mark-as-Read Race Condition
**What goes wrong:** User reads message, is_read flag updated, but another session or refresh shows message as unread again.

**Why it happens:** Two queries (SELECT to fetch, UPDATE to mark read) aren't atomic. If session dies between queries, message stays unread.

**How to avoid:**
- Acceptable approach: Mark read immediately on message open (UPDATE before SELECT body)
- Better approach: Use RETURNING clause: `UPDATE messages SET is_read = 1 WHERE id = ? RETURNING *`

**Warning signs:** Messages marked read reappear as unread after page refresh.

### Pitfall 4: OFFSET Performance with Large Mailboxes
**What goes wrong:** Inbox page 20 (OFFSET 300) takes seconds to load even though mailbox only has 350 messages.

**Why it happens:** SQLite must scan all rows up to OFFSET before returning results. Performance degrades linearly with offset size.

**How to avoid:** For Phase 5, not critical (BBS mailboxes rarely exceed 100 messages). If needed later, use cursor-based pagination: `WHERE id < last_seen_id ORDER BY id DESC LIMIT 15`.

**Warning signs:** Pagination slows down on later pages; EXPLAIN QUERY PLAN shows full table scan.

### Pitfall 5: Subject Line "Re: Re: Re: Re:" Accumulation
**What goes wrong:** Replying to a reply to a reply creates subjects like "Re: Re: Re: Original Subject".

**Why it happens:** Naively prepending "Re: " to existing subject on every reply.

**How to avoid:** Check if subject already starts with "Re: " before adding:
```rust
let reply_subject = if original_subject.starts_with("Re: ") {
    original_subject.to_string()
} else {
    format!("Re: {}", original_subject)
};
```

**Warning signs:** Subject lines grow unbounded with reply depth.

### Pitfall 6: Newline Normalization in Message Body
**What goes wrong:** Message body stored with `\n` but terminal expects `\r\n`, causing line ending issues on display.

**Why it happens:** User input may contain mixed line endings; database stores as-is; terminal rendering requires CRLF.

**How to avoid:** Normalize on storage (replace all `\r\n` and `\r` with `\n`, store as LF) and convert on display (join lines with `\r\n`). Existing pattern from Phase 1 decision: "CRLF (\r\n) line endings for all terminal output."

**Warning signs:** Message display has incorrect line spacing; quoted text rendering breaks.

## Code Examples

### Message CRUD Operations
```rust
// Source: Project's db/user.rs patterns + SQLite foreign keys best practices

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

pub async fn create_message(
    pool: &SqlitePool,
    sender_id: i64,
    recipient_id: i64,
    subject: &str,
    body: &str,
) -> Result<Message, sqlx::Error> {
    // Prevent self-mail at database level
    if sender_id == recipient_id {
        return Err(sqlx::Error::Protocol("Cannot send message to self".into()));
    }

    sqlx::query(
        "INSERT INTO messages (sender_id, recipient_id, subject, body)
         VALUES (?, ?, ?, ?)"
    )
    .bind(sender_id)
    .bind(recipient_id)
    .bind(subject)
    .bind(body)
    .execute(pool)
    .await?;

    // Fetch the created message
    let message = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages
         WHERE sender_id = ? AND recipient_id = ?
         ORDER BY id DESC LIMIT 1"
    )
    .bind(sender_id)
    .bind(recipient_id)
    .fetch_one(pool)
    .await?;

    Ok(message)
}

pub async fn get_inbox_page(
    pool: &SqlitePool,
    user_id: i64,
    page: i64,
    page_size: i64,
) -> Result<Vec<Message>, sqlx::Error> {
    let offset = page * page_size;

    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages
         WHERE recipient_id = ?
         ORDER BY sent_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn get_inbox_count(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let row: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE recipient_id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 as i64)
}

pub async fn get_unread_count(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let row: (i32,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE recipient_id = ? AND is_read = 0"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 as i64)
}

pub async fn mark_message_read(
    pool: &SqlitePool,
    message_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    // Verify ownership before marking read
    sqlx::query(
        "UPDATE messages SET is_read = 1
         WHERE id = ? AND recipient_id = ?"
    )
    .bind(message_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_message(
    pool: &SqlitePool,
    message_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    // Verify ownership before deleting
    sqlx::query(
        "DELETE FROM messages WHERE id = ? AND recipient_id = ?"
    )
    .bind(message_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}
```

### Inbox Rendering with Pagination
```rust
// Source: Project's profile.rs ANSI patterns + who.rs table rendering

use crate::terminal::{AnsiWriter, Color};
use crate::db::messages::Message;
use crate::services::profile::format_date;

pub fn render_inbox(
    messages: &[Message],
    page: usize,
    total: usize,
    page_size: usize,
    handles: &[(i64, String)], // sender_id -> handle lookup
) -> String {
    let mut writer = AnsiWriter::new();

    // Header with box-drawing (double-line for BBS aesthetic)
    writer.set_color(Color::LightCyan, Color::Black);
    writer.writeln("╔════════════════════════════════════════════════════════════════════════════╗");
    writer.write_str("║ ");
    writer.set_color(Color::Yellow, Color::Black);
    writer.bold();
    writer.write_str("MAIL");
    writer.reset_color();
    writer.set_color(Color::LightCyan, Color::Black);
    writer.write_str(" ".repeat(70).as_str());
    writer.writeln("║");
    writer.writeln("╠════╦════════════════════╦═══════════════════════════════════╦══════════════╣");

    // Column headers
    writer.write_str("║ ");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("#  ");
    writer.reset_color();
    writer.set_color(Color::LightCyan, Color::Black);
    writer.write_str("║ ");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("From               ");
    writer.reset_color();
    writer.set_color(Color::LightCyan, Color::Black);
    writer.write_str("║ ");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("Subject                           ");
    writer.reset_color();
    writer.set_color(Color::LightCyan, Color::Black);
    writer.write_str("║ ");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("Date         ");
    writer.reset_color();
    writer.set_color(Color::LightCyan, Color::Black);
    writer.writeln("║");
    writer.writeln("╠════╬════════════════════╬═══════════════════════════════════╬══════════════╣");

    // Message rows
    for (idx, msg) in messages.iter().enumerate() {
        let msg_num = page * page_size + idx + 1;
        let sender = handles.iter()
            .find(|(id, _)| *id == msg.sender_id)
            .map(|(_, h)| h.as_str())
            .unwrap_or("Unknown");

        writer.write_str("║ ");

        // Unread indicator
        if msg.is_read == 0 {
            writer.set_color(Color::Yellow, Color::Black);
            writer.bold();
            writer.write_str(&format!("{:<2}", msg_num));
            writer.write_str("N");
            writer.reset_color();
        } else {
            writer.set_color(Color::LightGray, Color::Black);
            writer.write_str(&format!("{:<3}", msg_num));
            writer.reset_color();
        }

        writer.set_color(Color::LightCyan, Color::Black);
        writer.write_str("║ ");

        // From (truncate to 18 chars)
        let from_display = if sender.len() > 18 {
            format!("{}...", &sender[..15])
        } else {
            format!("{:<18}", sender)
        };
        writer.set_color(Color::White, Color::Black);
        writer.write_str(&from_display);
        writer.reset_color();

        writer.set_color(Color::LightCyan, Color::Black);
        writer.write_str("║ ");

        // Subject (truncate to 33 chars)
        let subject_display = if msg.subject.len() > 33 {
            format!("{}...", &msg.subject[..30])
        } else {
            format!("{:<33}", msg.subject)
        };
        writer.set_color(Color::White, Color::Black);
        writer.write_str(&subject_display);
        writer.reset_color();

        writer.set_color(Color::LightCyan, Color::Black);
        writer.write_str("║ ");

        // Date
        let date_str = format_date(&msg.sent_at);
        writer.set_color(Color::LightGray, Color::Black);
        writer.write_str(&format!("{:<12}", date_str));
        writer.reset_color();

        writer.set_color(Color::LightCyan, Color::Black);
        writer.writeln("║");
    }

    // Footer with pagination
    writer.writeln("╚════╩════════════════════╩═══════════════════════════════════╩══════════════╝");

    // Pagination info
    let total_pages = (total + page_size - 1) / page_size;
    writer.set_color(Color::LightGray, Color::Black);
    writer.writeln(&format!("\r\nPage {} of {} ({} messages)", page + 1, total_pages, total));

    // Commands
    writer.set_color(Color::Yellow, Color::Black);
    writer.write_str("\r\n[");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("#");
    writer.reset_color();
    writer.set_color(Color::Yellow, Color::Black);
    writer.write_str("] Read message  [");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("C");
    writer.reset_color();
    writer.set_color(Color::Yellow, Color::Black);
    writer.write_str("] Compose  [");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("N");
    writer.reset_color();
    writer.set_color(Color::Yellow, Color::Black);
    writer.write_str("] Next page  [");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("P");
    writer.reset_color();
    writer.set_color(Color::Yellow, Color::Black);
    writer.write_str("] Prev  [");
    writer.set_color(Color::White, Color::Black);
    writer.bold();
    writer.write_str("Q");
    writer.reset_color();
    writer.set_color(Color::Yellow, Color::Black);
    writer.writeln("] Quit");
    writer.reset_color();

    writer.flush()
}
```

### Login Notification Check
```rust
// Source: Project's existing login flow + unread count query

// In session.rs after authentication succeeds:
async fn show_login_notification(&mut self, user_id: i64) {
    use crate::db::messages::get_unread_count;

    if let Ok(unread) = get_unread_count(&self.state.pool, user_id).await {
        if unread > 0 {
            let mut writer = AnsiWriter::new();
            writer.writeln("");
            writer.set_color(Color::Yellow, Color::Black);
            writer.bold();
            writer.write_str("*** You have ");
            writer.set_color(Color::White, Color::Black);
            writer.write_str(&unread.to_string());
            writer.set_color(Color::Yellow, Color::Black);
            writer.write_str(" new message");
            if unread > 1 {
                writer.write_str("s");
            }
            writer.write_str(". ***");
            writer.reset_color();
            writer.writeln("");

            self.output_buffer.write_str(&writer.flush());
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Compile-time checked sqlx queries | String-based sqlx queries | Phase 2 (02-01) | No DATABASE_URL at build time; simpler CI/CD; applies to messages table |
| Service trait for all features | Sentinel services for views | Phase 4 (04-06) | Lightweight state-based routing; mail views use __mail_*__ pattern |
| Threaded message display | Flat chronological inbox | Classic BBS UX | Simpler queries; no parent_id foreign key; matches user context decision |
| read_receipt tracking | Simple is_read boolean | Phase 5 decision | BBS mail doesn't need "seen by sender" receipts; binary read/unread sufficient |
| Separate inbox/sent tables | Single messages table | Modern best practice | Filter by sender_id or recipient_id; simpler schema; no data duplication |

**Deprecated/outdated:**
- **Separate inbox/outbox tables:** Modern messaging systems use single messages table with sender/recipient columns. Simpler queries, no duplication, easier to implement reply threading if needed later.
- **Read receipts:** BBS mail systems don't notify senders when messages are read. Only track is_read for recipient's inbox display.
- **Message threading:** User context specifies flat chronological inbox. Don't add parent_message_id or threading logic.

## Open Questions

1. **Mailbox size limit enforcement precision**
   - What we know: Context specifies "configurable mailbox size limit" with "Mailbox full" error when exceeded
   - What's unclear: Should limit count total messages (sent + received) or only received messages?
   - Recommendation: Limit only received messages (recipient_id = user_id). Rationale: User controls what they receive (can delete), but not what others send them. Sent messages don't consume recipient's quota.

2. **Status bar MAIL indicator update frequency**
   - What we know: Context specifies "Real-time notification: Status bar indicator (e.g., 'MAIL' flag on row 24) when unread mail exists during active session"
   - What's unclear: How often to check for new mail? On every timer tick (per minute)? On specific actions?
   - Recommendation: Check on timer ticks (already happening every minute, switching to per-second in final minute). Low overhead query (SELECT COUNT(*) WHERE recipient_id = ? AND is_read = 0), and matches existing timer integration pattern.

3. **Message deletion for sent messages**
   - What we know: Context specifies recipients can delete messages from inbox (immediate and permanent)
   - What's unclear: Can senders delete messages they sent (before recipient reads)?
   - Recommendation: No sender deletion. BBS convention: once sent, message belongs to recipient. Simplifies implementation and matches authentic BBS behavior.

## Sources

### Primary (HIGH confidence)
- [SQLite Foreign Key Support](https://sqlite.org/foreignkeys.html) - CASCADE constraints, PRAGMA foreign_keys
- [SQLite Foreign Keys with Cascade Delete](https://www.techonthenet.com/sqlite/foreign_keys/foreign_delete.php) - Syntax examples
- [SQLite Foreign Key Tutorial](https://www.sqlitetutorial.net/sqlite-foreign-key/) - Best practices
- Project codebase:
  - backend/src/db/schema.sql - Existing table patterns
  - backend/src/db/user.rs - CRUD operation patterns
  - backend/src/services/profile.rs - ANSI rendering patterns
  - backend/src/terminal/paging.rs - Pagination implementation
  - backend/src/websocket/session.rs - Sentinel service pattern (__whos_online__, __profile__)
- [Posting style - Wikipedia](https://en.wikipedia.org/wiki/Posting_style) - Message quoting conventions

### Secondary (MEDIUM confidence)
- [How to Design a Database for Messaging Systems - GeeksforGeeks](https://www.geeksforgeeks.org/dbms/how-to-design-a-database-for-messaging-systems/) - Schema design principles
- [Database Model for a Messaging System - Redgate](https://www.red-gate.com/blog/database-model-for-a-messaging-system) - Normalization patterns
- [Efficient tracking of unread messages per user - DEV Community](https://dev.to/anoopfranc/how-would-you-make-it-efficient-and-optimized-way-of-tracking-unread-message-per-user-3o00) - Read/unread patterns
- [System Design: Newly Unread Message Indicator - Medium](https://medium.com/@krutilin.sergey.ks/system-design-newly-unread-message-indicator-bb118492af92) - Unread tracking architectures
- [Read Receipts - PubNub Design Patterns](https://scalabl3.github.io/pubnub-design-patterns/2017/04/19/Read-Receipts.html) - Read receipt patterns
- [Proposed database structure for read/unread messages - GitHub Issue](https://github.com/dwyl/bestevidence/issues/354) - Timestamp-based tracking discussion

### Tertiary (LOW confidence)
- [WWIV BBS Internal Editor](http://docs.wwivbbs.org/en/wwiv55/editors/internal/) - BBS line editor commands (historical reference)
- [BBS Archives - Off-Line Mail Readers](http://www.bahaistudies.net/asma/qwk_mail.pdf) - QWK mail format (historical context)
- [LineBuffer in rustyline - Rust docs](https://docs.rs/rustyline/latest/rustyline/line_buffer/struct.LineBuffer.html) - Line editor implementation reference
- [Managing Exchange Server Mailbox Quotas - Practical365](https://practical365.com/configuring-exchange-server-mailbox-quotas/) - Quota management concepts (enterprise context, not directly applicable)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in project; patterns proven in Phases 1-4
- Architecture: HIGH - Sentinel services proven in Phase 4; pagination proven in Phase 1; CRUD patterns proven in Phase 2
- Pitfalls: HIGH - Foreign key enforcement verified in SQLite docs; race conditions well-documented in messaging systems; existing codebase demonstrates mitigation strategies

**Research date:** 2026-01-27
**Valid until:** 2026-02-27 (30 days - stable domain, established patterns)
