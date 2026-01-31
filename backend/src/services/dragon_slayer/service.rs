//! Dragon Slayer - Door Game Service
//!
//! A medieval RPG inspired by Legend of the Red Dragon (LORD).
//! Players fight monsters in the forest, level up by defeating masters,
//! romance NPCs and other players, and ultimately slay the Red Dragon.
//!
//! Uses __dragon_slayer__ sentinel for session routing.

use crate::services::dragon_slayer::db::{DragonSlayerDb, LeaderboardEntry};
use crate::games::dragon_slayer::{GameState, DragonSlayerFlow};
use crate::games::dragon_slayer::render::render_screen as game_render_screen;

/// Sentinel for session routing
pub const SENTINEL: &str = "__dragon_slayer__";

/// Initialize or resume a game session
pub async fn start_game(db: &DragonSlayerDb, user_id: i64, handle: &str) -> Result<(DragonSlayerFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    // Check if it's a new day
                    state.check_new_day();
                    state.handle = Some(handle.to_string());

                    let flow = DragonSlayerFlow::from_state(state);
                    let screen = game_render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save
                    Err(format!("Save corrupted: {}. Start new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game - start character creation
            let mut flow = DragonSlayerFlow::new();
            if let Some(state) = get_mutable_state(&mut flow) {
                state.handle = Some(handle.to_string());
            }
            let screen = game_render_screen(&flow);
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Get mutable reference to game state (helper)
fn get_mutable_state(_flow: &mut DragonSlayerFlow) -> Option<&mut GameState> {
    // We need a way to get mutable state - for now return None
    // The state is accessed through flow.state directly where needed
    None
}

/// Save current game state
pub async fn save_game_state(
    db: &DragonSlayerDb,
    user_id: i64,
    handle: &str,
    flow: &DragonSlayerFlow,
) -> Result<(), String> {
    let state = flow.game_state();
    let json = serde_json::to_string(state)
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(
        user_id,
        handle,
        &state.char_name,
        state.level,
        state.experience,
        &json,
    )
    .await
    .map_err(|e| format!("Save error: {}", e))
}

/// Render current screen based on game state
pub fn render_screen(flow: &DragonSlayerFlow) -> String {
    game_render_screen(flow)
}

/// Record game completion (dragon slain)
pub async fn record_game_completion(
    db: &DragonSlayerDb,
    user_id: i64,
    handle: &str,
    flow: &DragonSlayerFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_completion(
        user_id,
        handle,
        &state.char_name,
        state.dragon_kills,
        state.level,
        state.experience,
        state.kills,
        state.deaths,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Don't delete save - player can continue playing after dragon kill
    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &DragonSlayerDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &DragonSlayerDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}
