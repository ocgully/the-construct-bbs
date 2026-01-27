# Phase 4: Time Limits & User Lists - Research

**Researched:** 2026-01-27
**Domain:** Session time management, timer enforcement, ANSI terminal status bars, user activity tracking
**Confidence:** MEDIUM-HIGH

## Summary

Phase 4 implements daily time limits with visible countdown timers, graceful timeout handling, time banking, and user list displays. The research covered five technical domains: Tokio-based timer management, client-server timer synchronization, ANSI terminal status bar implementation, SQLite datetime queries for user history, and graceful timeout/cleanup patterns.

**Core pattern:** Server spawns per-session timer task using `tokio::time::interval` that broadcasts remaining time via WebSocket JSON messages. Client maintains countdown display using JavaScript `setInterval` with server-provided time. Status bar uses ANSI cursor positioning (ESC[24;1H) to write to row 24, with color changes for warnings. Timeout triggers graceful shutdown using `tokio::select!` with cleanup before disconnect.

Time banking stores unused daily minutes in user table. Daily reset uses SQLite datetime functions querying `datetime('now', 'localtime')` compared to `date(last_login)` to detect day boundary crossing. User lists query active nodes from NodeManager and session history from SQLite using `datetime()` functions with timezone modifiers.

**Primary recommendation:** Use tokio::time::interval for server-side timer ticks (per-minute normally, per-second in last minute), client-side countdown display, ANSI escape sequences for status bar positioning, and tokio::select! for timeout handling with CancellationToken for cleanup signaling.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio::time | 1.x (latest) | Timer intervals, timeouts, sleep | Async runtime standard, hashed timing wheel, well-tested |
| tokio::select! | macro | Racing async operations with timeouts | Idiomatic Tokio pattern for cancellation-safe multi-branch |
| tokio_util::sync::CancellationToken | 0.7+ | Graceful shutdown signaling | Official Tokio utility, clone-based broadcast pattern |
| serde_json | 1.x | WebSocket JSON message serialization | De-facto Rust JSON standard |
| chrono | 0.4+ | Timestamp storage and manipulation | Already used in NodeManager, datetime handling |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::sync::broadcast | built-in | Multi-consumer message passing | If broadcasting timer updates to multiple tasks |
| futures::StreamExt | 0.3+ | WebSocket split() for concurrent read/write | Needed for simultaneous send/receive on socket |
| sqlx datetime functions | built-in | SQLite date/time queries | Session history, last callers, daily reset detection |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tokio::time::interval | std::thread::sleep + mpsc | interval is async, non-blocking, integrates with select! |
| Client-side countdown | Server pushes every second | Client countdown reduces WebSocket traffic 60x |
| ANSI escape codes | Custom xterm.js addon | ANSI is standard, portable, no extra dependencies |

**Installation:**
```bash
# Already in dependencies
cargo add tokio --features full
cargo add tokio-util --features sync
cargo add serde_json
cargo add chrono
cargo add futures
```

## Architecture Patterns

### Recommended Project Structure
```
backend/src/
├── connection/
│   ├── node_manager.rs    # Extend with idle_time tracking
│   └── timer.rs           # NEW: Session timer task spawner
├── db/
│   ├── user.rs            # Add time banking fields/queries
│   └── session_history.rs # NEW: Last callers tracking
├── models/
│   └── user.rs            # Add time limit config per level
├── websocket/
│   └── session.rs         # Integrate timer task, timeout handling
└── services/
    ├── who.rs             # NEW: Who's Online service
    ├── last_callers.rs    # NEW: Last Callers service
    └── user_profile.rs    # NEW: View other users' profiles

frontend/src/
├── status-bar.ts          # NEW: Status bar renderer with ANSI positioning
└── timer.ts               # NEW: Client-side countdown logic
```

### Pattern 1: Session Timer Task with Timeout
**What:** Spawn a separate tokio task per session that ticks at intervals and enforces timeout
**When to use:** Session-scoped background work with graceful cleanup
**Example:**
```rust
// Source: https://tokio.rs/tokio/tutorial/select + https://tokio.rs/tokio/topics/shutdown
use tokio::time::{interval, Duration};
use tokio::select;
use tokio_util::sync::CancellationToken;

async fn session_timer_task(
    tx: mpsc::Sender<String>,
    remaining_minutes: i64,
    cancel: CancellationToken,
) {
    let mut remaining = remaining_minutes;
    let mut ticker = interval(Duration::from_secs(60)); // 1-minute ticks
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        select! {
            _ = ticker.tick() => {
                remaining -= 1;

                // Switch to per-second in last minute
                if remaining == 1 && ticker.period().as_secs() == 60 {
                    ticker = interval(Duration::from_secs(1));
                    // Convert to seconds for final minute
                    remaining = 60;
                }

                // Send time update to client
                let msg = serde_json::json!({
                    "type": "timer",
                    "remaining": remaining,
                    "warning": if remaining <= 60 { "red" } else if remaining <= 5 { "yellow" } else { "normal" }
                });
                let _ = tx.send(msg.to_string()).await;

                if remaining <= 0 {
                    // Trigger timeout sequence
                    let _ = tx.send(serde_json::json!({"type": "timeout"}).to_string()).await;
                    break;
                }
            }
            _ = cancel.cancelled() => {
                // Graceful shutdown requested (user quit)
                break;
            }
        }
    }
}
```

### Pattern 2: ANSI Status Bar Positioning
**What:** Use ANSI escape codes to write persistent status bar at row 24
**When to use:** Need persistent UI element that doesn't scroll with terminal content
**Example:**
```rust
// Source: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
fn render_status_bar(
    handle: &str,
    users_online: usize,
    time_remaining: i64,
    warning_level: &str,
) -> String {
    let (bg_color, fg_color) = match warning_level {
        "red" => ("\x1b[41m", "\x1b[97m"),    // Red bg, bright white fg
        "yellow" => ("\x1b[43m", "\x1b[30m"), // Yellow bg, black fg
        _ => ("\x1b[40m", "\x1b[37m"),        // Black bg, white fg
    };

    // Save cursor position, move to row 24, render bar, restore cursor
    format!(
        "\x1b7\x1b[24;1H{}{}  {} | Online: {} | Time: {}m \x1b[0m\x1b8",
        bg_color,
        fg_color,
        handle,
        users_online,
        time_remaining
    )
}
```
**Note:** ESC 7 saves cursor, ESC[24;1H moves to row 24 col 1, ESC 8 restores cursor, ESC[0m resets colors

### Pattern 3: Daily Time Reset Detection
**What:** Detect midnight boundary crossing using SQLite datetime comparison
**When to use:** Need to reset daily quotas without external cron
**Example:**
```rust
// Source: https://sqlite.org/lang_datefunc.html
async fn check_daily_reset(pool: &SqlitePool, user_id: i64) -> Result<bool, sqlx::Error> {
    // Query checks if last_login is from a different date than today
    let row: (i32,) = sqlx::query_as(
        "SELECT CASE
            WHEN date(last_login) < date('now', 'localtime') THEN 1
            ELSE 0
        END FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0 == 1)
}

async fn reset_daily_time(pool: &SqlitePool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET daily_time_used = 0 WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
```

### Pattern 4: Last Callers Query
**What:** Query recent session history with datetime formatting
**When to use:** Need to display user activity log with timestamps
**Example:**
```rust
// Source: https://sqlite.org/lang_datefunc.html
#[derive(Debug, FromRow)]
struct LastCaller {
    handle: String,
    login_time: String,  // Formatted datetime
    duration_minutes: i32,
}

async fn get_last_callers(pool: &SqlitePool, limit: i32) -> Result<Vec<LastCaller>, sqlx::Error> {
    sqlx::query_as::<_, LastCaller>(
        "SELECT handle,
                datetime(login_time, 'localtime') as login_time,
                duration_minutes
         FROM session_history
         ORDER BY login_time DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}
```

### Pattern 5: Graceful Timeout with Cleanup
**What:** Use select! to handle timeout alongside normal operations, with cleanup before disconnect
**When to use:** Need to gracefully shut down with resource cleanup on timeout
**Example:**
```rust
// Source: https://tokio.rs/tokio/topics/shutdown
async fn handle_authenticated_with_timeout(
    &mut self,
    timeout_cancel: CancellationToken,
) {
    loop {
        select! {
            input = self.receive_input() => {
                self.handle_input(&input).await;
            }
            _ = timeout_cancel.cancelled() => {
                // Timeout occurred - run cleanup
                self.handle_timeout().await;
                break;
            }
        }
    }
}

async fn handle_timeout(&mut self) {
    // 1. Save game state if in service
    if let Some(service_name) = &self.current_service {
        let service = self.state.registry.get(service_name).cloned();
        if let Some(svc) = service {
            svc.on_timeout(self);  // Game-specific timeout handling
        }
    }

    // 2. Save session time
    let session_minutes = self.login_time.elapsed().as_secs() / 60;
    let _ = update_user_time(&self.state.db_pool, self.user_id, session_minutes).await;

    // 3. Show timeout-specific goodbye
    let goodbye = render_timeout_goodbye(&self.handle, session_minutes);
    let _ = self.tx.send(goodbye).await;

    // 4. Wait for user to read, then disconnect
    tokio::time::sleep(Duration::from_secs(3)).await;
    self.disconnecting = true;
}
```

### Pattern 6: Client-Side Countdown with Server Sync
**What:** Client maintains local countdown, server sends periodic updates for drift correction
**When to use:** Minimize WebSocket traffic while maintaining accuracy
**Example:**
```typescript
// Source: https://medium.com/@flowersayo/syncing-countdown-timers-across-multiple-clients-a-subtle-but-critical-challenge-384ba5fbef9a
class SessionTimer {
    private remainingSeconds: number = 0;
    private intervalId: number | null = null;
    private tickRate: number = 60000; // 1 minute in ms

    updateFromServer(remainingMinutes: number, warning: string) {
        this.remainingSeconds = remainingMinutes * 60;

        // Switch to per-second ticking in last minute
        if (remainingMinutes <= 1) {
            this.remainingSeconds = remainingMinutes; // Already in seconds
            this.tickRate = 1000;
            this.restart();
        }

        this.updateDisplay(warning);
    }

    start() {
        this.intervalId = setInterval(() => {
            if (this.tickRate === 60000) {
                this.remainingSeconds -= 60;
            } else {
                this.remainingSeconds -= 1;
            }
            this.updateDisplay();
        }, this.tickRate);
    }

    updateDisplay(warning?: string) {
        const minutes = Math.floor(this.remainingSeconds / 60);
        const color = warning || (this.remainingSeconds <= 60 ? 'red' :
                                  this.remainingSeconds <= 300 ? 'yellow' : 'normal');
        // Update status bar display
        this.renderStatusBar(minutes, color);
    }

    private restart() {
        if (this.intervalId) clearInterval(this.intervalId);
        this.start();
    }
}
```

### Anti-Patterns to Avoid
- **Server push every second:** Wastes bandwidth. Client-side countdown with periodic server sync (per-minute) is sufficient.
- **Blocking timeout check:** Don't use sleep loops. Use tokio::select! with interval/timeout futures.
- **Hard-coded time limits:** Make limits configurable per user level in config.toml.
- **Instant::now() for daily reset:** Use SQLite datetime functions with 'localtime' modifier for consistent timezone handling.
- **Forgetting cursor restore:** Always save cursor position before writing status bar and restore after, or content will be corrupted.
- **Time drift accumulation:** Use interval.tick() not sleep() in loops to prevent drift from task execution time.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Task cancellation/cleanup | Manual Arc<AtomicBool> flags | tokio_util::sync::CancellationToken | Clone-based broadcast, integrates with select!, Drop cleanup |
| Timer intervals with drift correction | sleep() in loop | tokio::time::interval with MissedTickBehavior | Prevents accumulating delays from task execution time |
| Midnight detection | Custom timezone math | SQLite date('now', 'localtime') comparison | Handles DST, leap seconds, timezone changes |
| WebSocket concurrent read/write | Manual mpsc channels | futures::StreamExt::split() | Standard pattern, prevents borrow conflicts |
| Time synchronization | Custom NTP client | Client countdown + periodic server updates | Simple, sufficient for session timers (not wall-clock sync) |

**Key insight:** Timer management, graceful cancellation, and datetime handling have subtle edge cases (drift, DST, cleanup ordering). Use battle-tested Tokio/SQLite primitives rather than custom implementations.

## Common Pitfalls

### Pitfall 1: Timer Drift from Sleep Loops
**What goes wrong:** Using `tokio::time::sleep(Duration::from_secs(60))` in a loop causes timer to drift as each iteration includes task execution time
**Why it happens:** sleep() measures from call time, not absolute intervals
**How to avoid:** Use `tokio::time::interval()` which measures elapsed time between ticks and compensates for drift
**Warning signs:** Timer gradually falls behind real time, warnings appear late

### Pitfall 2: Timezone Inconsistency Between Server and SQLite
**What goes wrong:** Server uses chrono::Utc::now() while SQLite queries use datetime('now', 'localtime'), causing daily reset logic to trigger at wrong time
**Why it happens:** SQLite's 'now' defaults to UTC, 'localtime' modifier converts to local time, but server code may use different timezone
**How to avoid:** Consistently use 'localtime' modifier in all SQLite datetime queries, or store Unix timestamps and convert in application layer
**Warning signs:** Daily reset happens at wrong hour, time bank resets unexpectedly

### Pitfall 3: Forgetting Cursor Position Save/Restore
**What goes wrong:** Writing status bar with ESC[24;1H moves cursor to row 24, but not restoring cursor position leaves user typing at bottom of screen
**Why it happens:** ANSI cursor positioning is stateful, must explicitly save and restore
**How to avoid:** Always wrap status bar writes with ESC 7 (save) and ESC 8 (restore), or use ESC[s/ESC[u (SCO variant)
**Warning signs:** User input appears on status bar line, terminal content corrupted

### Pitfall 4: Not Handling Mid-Session Timeout During Service
**What goes wrong:** Timeout occurs while user is in a game/service, but game state isn't saved because only session cleanup runs
**Why it happens:** Timeout handler doesn't call service's on_timeout lifecycle method
**How to avoid:** Define Service::on_timeout() method, call it before general cleanup in timeout handler
**Warning signs:** Users complain about lost game progress after timeouts

### Pitfall 5: Time Bank Withdrawal Prompt Blocking
**What goes wrong:** Showing time bank withdrawal prompt at 1-minute warning blocks user input, preventing them from responding before timeout
**Why it happens:** Synchronous prompt or not handling timeout during prompt display
**How to avoid:** Send withdrawal prompt as non-blocking message, use select! to race user response against timeout continuation
**Warning signs:** Users never able to use banked time, always timeout before responding

### Pitfall 6: Race Condition Between Quit and Timeout
**What goes wrong:** User quits manually at same moment timeout fires, causing double cleanup (session saved twice, token deleted twice, goodbye screen shown twice)
**Why it happens:** Quit handler and timeout handler both trigger cleanup independently
**How to avoid:** Use atomic flag or state transition to ensure cleanup runs exactly once, or make cleanup idempotent
**Warning signs:** Database errors about missing session tokens, users see corrupted goodbye screens

### Pitfall 7: Last Minute Per-Second Switch Causing Duplicate Intervals
**What goes wrong:** Switching from per-minute to per-second interval at 1-minute remaining creates two active intervals, causing double-tick
**Why it happens:** Creating new interval without dropping old one
**How to avoid:** Replace interval by assigning new one to same variable (old dropped), or use Duration::from_secs(60) initially and call reset() method
**Warning signs:** Timer counts down twice per second in last minute, timeout triggers at 30 seconds

## Code Examples

Verified patterns from official sources:

### Setting Interval Missed Tick Behavior
```rust
// Source: https://docs.rs/tokio/latest/tokio/time/fn.interval.html
use tokio::time::{interval, MissedTickBehavior, Duration};

let mut ticker = interval(Duration::from_secs(60));
ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

// Prevents backlog of ticks if system is under load
loop {
    ticker.tick().await;
    // Do work
}
```

### Using tokio::select! with Timeout
```rust
// Source: https://tokio.rs/tokio/tutorial/select
use tokio::select;
use tokio::time::{sleep, Duration};

select! {
    result = async_operation() => {
        // Operation completed normally
    }
    _ = sleep(Duration::from_secs(5)) => {
        // Timeout occurred
    }
}
// All non-selected branches are dropped (cancelled)
```

### SQLite Date Comparison for Daily Reset
```sql
-- Source: https://sqlite.org/lang_datefunc.html
-- Check if last_login is from a different day than today (local time)
SELECT CASE
    WHEN date(last_login) < date('now', 'localtime') THEN 1
    ELSE 0
END
FROM users
WHERE id = ?;

-- Query session history from last 24 hours
SELECT *
FROM session_history
WHERE login_time > datetime('now', '-24 hours', 'localtime')
ORDER BY login_time DESC;
```

### WebSocket Split for Concurrent Read/Write
```rust
// Source: https://docs.rs/axum/latest/axum/extract/ws/index.html
use futures::StreamExt;

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn task for sending
    tokio::spawn(async move {
        loop {
            sender.send(Message::Text("ping".into())).await.ok();
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    // Main task for receiving
    while let Some(msg) = receiver.next().await {
        // Handle incoming messages
    }
}
```

### ANSI Escape Code Reference
```rust
// Source: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
const ESC: &str = "\x1b";

// Cursor positioning
format!("{ESC}[{row};{col}H")        // Move to row, col (1-based)
format!("{ESC}7")                     // Save cursor position (DEC)
format!("{ESC}8")                     // Restore cursor position (DEC)
format!("{ESC}[s")                    // Save cursor position (SCO)
format!("{ESC}[u")                    // Restore cursor position (SCO)

// Colors
format!("{ESC}[40m")                  // Black background
format!("{ESC}[41m")                  // Red background
format!("{ESC}[43m")                  // Yellow background
format!("{ESC}[37m")                  // White foreground
format!("{ESC}[97m")                  // Bright white foreground
format!("{ESC}[0m")                   // Reset all attributes
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| std::thread + sleep | tokio::time::interval | Tokio 0.2+ (2019) | Async, non-blocking, integrates with runtime |
| Manual AtomicBool shutdown | CancellationToken | tokio-util 0.6+ (2021) | Clone-based broadcast, Drop cleanup |
| setInterval(..., 1000) | setInterval with drift correction | Modern JS (2020+) | Accuracy for long-running timers |
| Server push every tick | Client countdown + periodic sync | WebSocket best practice | Reduces bandwidth 60x |
| DECSTBM scroll regions | Simple cursor positioning | Terminal compatibility | Wider support, simpler |

**Deprecated/outdated:**
- **tokio::timer module:** Removed in Tokio 1.0, use tokio::time instead
- **chrono 0.3:** Deprecated, use chrono 0.4+ which has improved timezone handling
- **SCO cursor save/restore (ESC[s/ESC[u):** Less widely supported than DEC (ESC 7/ESC 8)

## Open Questions

Things that couldn't be fully resolved:

1. **Status bar scrolling region approach**
   - What we know: Can use DECSTBM (ESC[<top>;<bottom>r) to reserve row 24, but xterm.js support is unclear
   - What's unclear: Whether xterm.js properly handles DECSTBM, or if simple cursor positioning is more reliable
   - Recommendation: Start with simple cursor save/position/restore (ESC 7, ESC[24;1H, ESC 8). Test DECSTBM if content scrolling over status bar is an issue.

2. **Time bank cap enforcement location**
   - What we know: Config should have per-level time bank caps (e.g., User max 120 minutes banked)
   - What's unclear: Whether to enforce cap on deposit (at logout) or withdrawal (at login)
   - Recommendation: Enforce on deposit - simpler logic, prevents over-banking. Cap in config as `time_bank_cap_minutes` per level.

3. **Idle time tracking granularity**
   - What we know: NodeManager tracks connected_at, need to add last_activity timestamp
   - What's unclear: Whether to update last_activity on every input (noisy) or only on significant actions (menu selection, service launch)
   - Recommendation: Update on every non-empty input for accuracy. Performance impact negligible with async writes.

4. **Profile view from Who's Online - modal or service**
   - What we know: Need to view other users' profiles from Who's Online list
   - What's unclear: Whether to show profile as temporary overlay (returns to Who's Online after) or launch as separate service (requires navigation back)
   - Recommendation: Use service pattern (like current profile view) but add "return to previous service" state tracking for better UX.

5. **Timer synchronization precision**
   - What we know: Server sends time updates per-minute (normally) or per-second (last minute)
   - What's unclear: Acceptable drift tolerance before requiring sub-minute server updates
   - Recommendation: Accept up to 5 seconds drift (common in BBS nostalgia). If precision critical, send server time with each update and client corrects drift.

## Sources

### Primary (HIGH confidence)
- Tokio time documentation: https://docs.rs/tokio/latest/tokio/time/index.html - Timer utilities, interval, timeout
- Tokio select tutorial: https://tokio.rs/tokio/tutorial/select - Using select! macro, cancellation safety
- Tokio graceful shutdown: https://tokio.rs/tokio/topics/shutdown - CancellationToken, cleanup patterns
- SQLite datetime functions: https://sqlite.org/lang_datefunc.html - Date/time queries, timezone handling
- Axum WebSocket docs: https://docs.rs/axum/latest/axum/extract/ws/index.html - WebSocket handling, split pattern
- ANSI escape codes reference: https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797 - Cursor positioning, colors

### Secondary (MEDIUM confidence)
- Syncing countdown timers article (Medium, Sep 2025): https://medium.com/@flowersayo/syncing-countdown-timers-across-multiple-clients-a-subtle-but-critical-challenge-384ba5fbef9a - Client-side countdown with server sync patterns
- Rust WebSocket with Axum tutorial (Medium, 2024): https://medium.com/@itsuki.enjoy/rust-websocket-with-axum-for-realtime-communications-49a93468268f - Axum WebSocket patterns
- Building async countdown timer in Rust (Medium, 2024): https://medium.com/@onur-karaduman/building-an-async-countdown-timer-in-rust-with-tokio-f765a24b739e - Tokio interval usage
- Configuration management in Rust (LogRocket): https://blog.logrocket.com/configuration-management-in-rust-web-services/ - TOML per-level config patterns

### Tertiary (LOW confidence - general references)
- ANSI escape code Wikipedia: https://en.wikipedia.org/wiki/ANSI_escape_code - General ANSI reference
- Build Command Line with ANSI: https://www.lihaoyi.com/post/BuildyourownCommandLinewithANSIescapecodes.html - ANSI patterns
- Various Rust scheduler libraries (GitHub): Multiple libraries for cron-like scheduling in Rust

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Tokio time, select, CancellationToken are official and well-documented
- Architecture: MEDIUM-HIGH - Patterns verified from official docs, but integration with existing BBS architecture untested
- Pitfalls: MEDIUM - Based on common timer/WebSocket issues and Tokio gotchas, not BBS-specific testing
- ANSI status bar: MEDIUM - Standard escape codes verified, but xterm.js specific behavior not tested
- Client-side timer: MEDIUM - JavaScript patterns verified, but drift correction for BBS use case not validated

**Research date:** 2026-01-27
**Valid until:** 2026-02-27 (30 days - stable domain, Tokio patterns well-established)
