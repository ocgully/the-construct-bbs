//! Mineteria - Door Game Service
//!
//! A 2D sandbox mining/crafting game inspired by Terraria/Minecraft.
//! Features procedural world generation, crafting, and survival mechanics.
//!
//! Uses __mineteria__ sentinel for session routing.

use rand::Rng;
use crate::services::mineteria::db::{MineteriaDb, LeaderboardEntry};
use crate::games::mineteria::{GameState, MineteriaFlow};
use crate::games::mineteria::render::*;

/// Sentinel for session routing
pub const SENTINEL: &str = "__mineteria__";

/// Initialize or resume a game session
pub async fn start_game(db: &MineteriaDb, user_id: i64, _handle: &str) -> Result<(MineteriaFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(state) => {
                    let flow = MineteriaFlow::from_state(state);
                    let screen = render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save, offer to start fresh
                    Err(format!("Save corrupted: {}. Starting new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game with random seed
            let seed = rand::thread_rng().gen::<u64>();
            let flow = MineteriaFlow::new(seed);
            let screen = render_intro();
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(
    db: &MineteriaDb,
    user_id: i64,
    handle: &str,
    flow: &MineteriaFlow,
) -> Result<(), String> {
    let state = flow.game_state();
    let json = serde_json::to_string(state)
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, state.world_seed, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &MineteriaDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion
pub async fn record_game_completion(
    db: &MineteriaDb,
    user_id: i64,
    handle: &str,
    flow: &MineteriaFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    let score = (state.stats.blocks_mined as i64 * 10)
        + (state.stats.monsters_killed as i64 * 50)
        + (state.day as i64 * 100);

    db.record_completion(
        user_id,
        handle,
        score,
        state.day as i64,
        state.level as i64,
        state.stats.blocks_mined as i64,
        state.stats.monsters_killed as i64,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete save after completion
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &MineteriaDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen
pub fn render_current_screen(flow: &MineteriaFlow) -> String {
    render_screen(flow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel_format() {
        assert!(SENTINEL.starts_with("__"));
        assert!(SENTINEL.ends_with("__"));
        assert!(SENTINEL.contains("mineteria"));
    }
}
