use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tokio_util::sync::CancellationToken;

/// Represents a running session timer.
/// Hold onto this struct -- dropping it won't cancel the task.
/// Call cancel() to stop the timer (on clean quit).
#[allow(dead_code)]
pub struct SessionTimer {
    cancel: CancellationToken,
    handle: tokio::task::JoinHandle<TimerResult>,
    /// Set to true when the timer expires (time reaches 0).
    /// Polled by session to trigger timeout sequence.
    expired: Arc<AtomicBool>,
    /// Set to true when remaining time <= 1 minute.
    /// Polled by session to trigger time bank withdrawal prompt.
    low_time: Arc<AtomicBool>,
}

/// Result of the timer task completing.
#[derive(Debug, Clone, PartialEq)]
pub enum TimerResult {
    /// Time expired -- user should be timed out
    Expired,
    /// Timer was cancelled (user quit normally)
    Cancelled,
}

impl SessionTimer {
    /// Spawn a new session timer task.
    ///
    /// - `tx`: Channel to send timer JSON updates to the client
    /// - `remaining_minutes`: How many minutes the user has left today
    /// - `handle`: User's handle (for status bar)
    /// - `users_online`: Number of users currently online (for status bar)
    /// - `user_id`: User's ID (for checking unread mail)
    /// - `pool`: Database connection pool (for checking unread mail)
    ///
    /// The timer sends JSON messages of the form:
    /// { "type": "timer", "remaining": <minutes_or_seconds>, "unit": "min"|"sec", "warning": "normal"|"yellow"|"red", "handle": "...", "online": N, "has_mail": bool }
    ///
    /// Timer ticks per-minute normally. In the final minute, switches to per-second.
    /// When remaining reaches 0, sends { "type": "timeout" } and returns TimerResult::Expired.
    pub fn spawn(
        tx: mpsc::Sender<String>,
        remaining_minutes: i64,
        handle: String,
        users_online: usize,
        user_id: i64,
        pool: sqlx::SqlitePool,
    ) -> Self {
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let expired = Arc::new(AtomicBool::new(false));
        let expired_clone = expired.clone();
        let low_time = Arc::new(AtomicBool::new(false));
        let low_time_clone = low_time.clone();

        let handle_task = tokio::spawn(async move {
            run_timer(tx, remaining_minutes, handle, users_online, user_id, pool, cancel_clone, expired_clone, low_time_clone).await
        });

        Self {
            cancel,
            handle: handle_task,
            expired,
            low_time,
        }
    }

    /// Cancel the timer (called on clean quit).
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Wait for the timer to complete and get the result.
    #[allow(dead_code)]
    pub async fn wait(self) -> TimerResult {
        match self.handle.await {
            Ok(result) => result,
            Err(_) => TimerResult::Cancelled, // Task panicked or was aborted
        }
    }

    /// Check if the timer is still running.
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }

    /// Check if the timer has expired (time reached 0).
    /// Returns true once and stays true.
    pub fn is_expired(&self) -> bool {
        self.expired.load(Ordering::Relaxed)
    }

    /// Check if remaining time is <= 1 minute.
    /// Used by session to trigger time bank withdrawal prompt.
    pub fn is_low_time(&self) -> bool {
        self.low_time.load(Ordering::Relaxed)
    }
}

async fn run_timer(
    tx: mpsc::Sender<String>,
    remaining_minutes: i64,
    handle: String,
    users_online: usize,
    user_id: i64,
    pool: sqlx::SqlitePool,
    cancel: CancellationToken,
    expired_flag: Arc<AtomicBool>,
    low_time_flag: Arc<AtomicBool>,
) -> TimerResult {
    if remaining_minutes <= 0 {
        // Unlimited time (sysop) -- still send initial status but never expire
        let has_mail = match crate::db::messages::get_unread_count(&pool, user_id).await {
            Ok(count) => count > 0,
            Err(_) => false, // Fail silently - don't break timer for mail check
        };
        let msg = serde_json::json!({
            "type": "timer",
            "remaining": 0,
            "unit": "unlimited",
            "warning": "normal",
            "handle": handle,
            "online": users_online,
            "has_mail": has_mail
        });
        let _ = tx.send(msg.to_string()).await;

        // Just wait for cancellation
        cancel.cancelled().await;
        return TimerResult::Cancelled;
    }

    let mut remaining = remaining_minutes;
    let mut ticker = interval(Duration::from_secs(60));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // Send initial time update
    let warning = get_warning_level(remaining, false);
    let has_mail = match crate::db::messages::get_unread_count(&pool, user_id).await {
        Ok(count) => count > 0,
        Err(_) => false, // Fail silently - don't break timer for mail check
    };
    let msg = serde_json::json!({
        "type": "timer",
        "remaining": remaining,
        "unit": "min",
        "warning": warning,
        "handle": handle,
        "online": users_online,
        "has_mail": has_mail
    });
    let _ = tx.send(msg.to_string()).await;

    // Consume the first immediate tick
    ticker.tick().await;

    // Per-minute countdown
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                remaining -= 1;

                if remaining <= 1 {
                    // Signal low time for session to offer bank withdrawal
                    low_time_flag.store(true, Ordering::Relaxed);
                    // Switch to per-second countdown for final minute
                    break;
                }

                let warning = get_warning_level(remaining, false);
                let has_mail = match crate::db::messages::get_unread_count(&pool, user_id).await {
                    Ok(count) => count > 0,
                    Err(_) => false, // Fail silently - don't break timer for mail check
                };
                let msg = serde_json::json!({
                    "type": "timer",
                    "remaining": remaining,
                    "unit": "min",
                    "warning": warning,
                    "handle": handle,
                    "online": users_online,
                    "has_mail": has_mail
                });
                let _ = tx.send(msg.to_string()).await;

                // Send special warning messages at 5-min and 1-min marks
                if remaining == 5 {
                    let warn_msg = serde_json::json!({
                        "type": "timer_warning",
                        "minutes": 5
                    });
                    let _ = tx.send(warn_msg.to_string()).await;
                }
            }
            _ = cancel.cancelled() => {
                return TimerResult::Cancelled;
            }
        }
    }

    // Final minute: per-second countdown
    let mut remaining_secs: i64 = remaining * 60; // Convert remaining minutes to seconds
    let mut sec_ticker = interval(Duration::from_secs(1));
    sec_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    // Send 1-minute warning
    let warn_msg = serde_json::json!({
        "type": "timer_warning",
        "minutes": 1
    });
    let _ = tx.send(warn_msg.to_string()).await;

    // Consume first immediate tick
    sec_ticker.tick().await;

    loop {
        tokio::select! {
            _ = sec_ticker.tick() => {
                remaining_secs -= 1;

                let has_mail = match crate::db::messages::get_unread_count(&pool, user_id).await {
                    Ok(count) => count > 0,
                    Err(_) => false, // Fail silently - don't break timer for mail check
                };
                let msg = serde_json::json!({
                    "type": "timer",
                    "remaining": remaining_secs,
                    "unit": "sec",
                    "warning": "red",
                    "handle": handle,
                    "online": users_online,
                    "has_mail": has_mail
                });
                let _ = tx.send(msg.to_string()).await;

                if remaining_secs <= 0 {
                    // Time expired -- set flag and send timeout signal
                    expired_flag.store(true, Ordering::Relaxed);
                    let timeout_msg = serde_json::json!({
                        "type": "timeout"
                    });
                    let _ = tx.send(timeout_msg.to_string()).await;
                    return TimerResult::Expired;
                }
            }
            _ = cancel.cancelled() => {
                return TimerResult::Cancelled;
            }
        }
    }
}

fn get_warning_level(remaining_minutes: i64, _is_seconds: bool) -> &'static str {
    if remaining_minutes <= 1 {
        "red"
    } else if remaining_minutes <= 5 {
        "yellow"
    } else {
        "normal"
    }
}
