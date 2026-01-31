//! Dystopia - Door Game Service
//!
//! A kingdom management strategy game where players manage provinces,
//! build armies, and coordinate with kingdoms.
//!
//! Uses __dystopia__ sentinel for session routing.
//! Game state persisted in game's own database (dystopia.db).

use crate::services::dystopia::db::{DystopiaDb, LeaderboardEntry};
use crate::games::dystopia::{ProvinceState, DystopiaFlow};
use crate::games::dystopia::render::render_screen as game_render_screen;
use crate::games::dystopia::tick::{process_catchup_ticks, ticks_since_last_update};

/// Sentinel for session routing
pub const SENTINEL: &str = "__dystopia__";

/// Calculate current global tick (hours since epoch, for example)
fn current_tick() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    // One tick per hour
    (duration.as_secs() / 3600) as i64
}

/// Initialize or resume a game session
pub async fn start_game(db: &DystopiaDb, user_id: i64, _handle: &str) -> Result<(DystopiaFlow, String), String> {
    let age_id = db.get_active_age_id().await
        .map_err(|e| format!("Database error: {}", e))?;

    // Check for existing province in this age
    match db.load_province(user_id, age_id).await {
        Ok(Some((_province_id, json, _last_tick))) => {
            // Resume existing game
            match serde_json::from_str::<ProvinceState>(&json) {
                Ok(mut state) => {
                    // Calculate and process catchup ticks
                    let current = current_tick();
                    let missed = ticks_since_last_update(&state, current);

                    if missed > 0 {
                        // Cap at 168 ticks (1 week) to prevent excessive catchup
                        let ticks_to_process = missed.min(168);
                        let result = process_catchup_ticks(&mut state, ticks_to_process);

                        // Set message about catchup
                        if result.ticks_processed > 0 {
                            let mut msg = format!("Welcome back! {} ticks processed.", result.ticks_processed);
                            if result.peasants_starved > 0 {
                                msg.push_str(&format!(" {} peasants starved.", result.peasants_starved));
                            }
                            if result.research_completed.is_some() {
                                msg.push_str(" Research completed!");
                            }
                            if result.protection_expired {
                                msg.push_str(" Protection has expired!");
                            }
                            state.last_message = Some(msg);
                        }

                        state.last_tick = current;
                    }

                    let flow = DystopiaFlow::from_state(state);
                    let screen = game_render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    Err(format!("Save corrupted: {}. Start new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game - start character creation
            let flow = DystopiaFlow::new();
            let screen = game_render_screen(&flow);
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &DystopiaDb, user_id: i64, handle: &str, flow: &DystopiaFlow) -> Result<(), String> {
    // Only save if we have a valid province (past character creation)
    if flow.game_state().name.is_empty() {
        return Ok(());
    }

    let age_id = db.get_active_age_id().await
        .map_err(|e| format!("Database error: {}", e))?;

    let json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    let current = current_tick();

    db.save_province(user_id, handle, age_id, &json, current)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    Ok(())
}

/// Render current screen
pub fn render_screen(flow: &DystopiaFlow) -> String {
    game_render_screen(flow)
}

/// Record game completion (end of age)
pub async fn record_game_completion(
    db: &DystopiaDb,
    user_id: i64,
    handle: &str,
    flow: &DystopiaFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    let age_id = db.get_active_age_id().await
        .map_err(|e| format!("Database error: {}", e))?;

    db.record_completion(
        user_id,
        handle,
        age_id,
        state.networth(),
        state.land,
        state.stats.attacks_won,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &DystopiaDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_tick() {
        let tick = current_tick();
        // Should be a reasonable number (hours since 1970)
        assert!(tick > 400000); // After year 2015
    }

    #[test]
    fn test_render_new_game() {
        let flow = DystopiaFlow::new();
        let screen = render_screen(&flow);
        assert!(screen.contains("RACE")); // Race selection screen
    }
}
