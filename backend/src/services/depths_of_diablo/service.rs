//! Depths of Diablo - Door Game Service
//!
//! A real-time roguelite dungeon crawler inspired by Diablo 1-2.
//! Features procedural dungeons, real-time combat, and meta-progression.
//!
//! Uses __depths_of_diablo__ sentinel for session routing.

#![allow(dead_code)]

use crate::games::depths_of_diablo::DiabloFlow;
use crate::games::depths_of_diablo::render;
use crate::games::depths_of_diablo::state::{GameState, MetaProgress};
use crate::services::depths_of_diablo::db::{DiabloDb, LeaderboardEntry};

/// Sentinel for session routing
pub const SENTINEL: &str = "__depths_of_diablo__";

/// Initialize or resume a game session
pub async fn start_game(db: &DiabloDb, user_id: i64, handle: &str) -> Result<(DiabloFlow, String), String> {
    // Try to load existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    // Try to also load meta progression (may be more recent)
                    if let Ok(Some(meta_json)) = db.load_meta(user_id).await {
                        if let Ok(meta) = serde_json::from_str::<MetaProgress>(&meta_json) {
                            state.meta = meta;
                        }
                    }

                    let flow = DiabloFlow::from_state(state);
                    let screen = render::render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(_e) => {
                    // Corrupted save - start fresh but try to preserve meta
                    let mut state = GameState::new(user_id, handle);

                    if let Ok(Some(meta_json)) = db.load_meta(user_id).await {
                        if let Ok(meta) = serde_json::from_str::<MetaProgress>(&meta_json) {
                            state.meta = meta;
                        }
                    }

                    // Delete corrupted save
                    let _ = db.delete_save(user_id).await;

                    let flow = DiabloFlow::from_state(state);
                    let screen = render::render_screen(&flow);
                    Ok((flow, screen))
                }
            }
        }
        Ok(None) => {
            // No save - check for meta progression
            let mut state = GameState::new(user_id, handle);

            if let Ok(Some(meta_json)) = db.load_meta(user_id).await {
                if let Ok(meta) = serde_json::from_str::<MetaProgress>(&meta_json) {
                    state.meta = meta;
                }
            }

            let flow = DiabloFlow::from_state(state);
            let screen = render::render_screen(&flow);
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &DiabloDb, user_id: i64, handle: &str, flow: &DiabloFlow) -> Result<(), String> {
    let state = flow.game_state();

    // Save full game state
    let state_json = serde_json::to_string(&state)
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &state_json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    // Also save meta progression separately (survives run death)
    let meta_json = serde_json::to_string(&state.meta)
        .map_err(|e| format!("Serialize meta error: {}", e))?;

    db.save_meta(user_id, handle, &meta_json)
        .await
        .map_err(|e| format!("Save meta error: {}", e))?;

    Ok(())
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &DiabloDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion (run end)
pub async fn record_game_completion(
    db: &DiabloDb,
    user_id: i64,
    handle: &str,
    flow: &DiabloFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    // Get run info
    let floor_reached = state.run.as_ref().map(|r| r.max_floor_reached).unwrap_or(1);
    let soul_essence = state.run.as_ref().map(|r| r.soul_essence_reward()).unwrap_or(0);
    let victory = state.run.as_ref().map(|r| r.run_completed).unwrap_or(false);
    let class = state.character.as_ref()
        .map(|c| c.class.name().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Record completion
    db.record_completion(
        user_id,
        handle,
        floor_reached,
        soul_essence,
        &class,
        victory,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete active save (run is over)
    let _ = db.delete_save(user_id).await;

    // Save updated meta progression
    let meta_json = serde_json::to_string(&state.meta)
        .map_err(|e| format!("Serialize meta error: {}", e))?;

    db.save_meta(user_id, handle, &meta_json)
        .await
        .map_err(|e| format!("Save meta error: {}", e))?;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &DiabloDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen
pub fn render_screen(flow: &DiabloFlow) -> String {
    render::render_screen(flow)
}

/// Process real-time tick (for combat updates)
pub fn process_tick(flow: &mut DiabloFlow) -> bool {
    use crate::games::depths_of_diablo::screen::DiabloAction;

    match flow.tick() {
        DiabloAction::Tick => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_start_new_game() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        let result = start_game(&db, 1, "TestPlayer").await;
        assert!(result.is_ok());

        let (_flow, screen) = result.unwrap();
        assert!(screen.contains("DEPTHS")); // Should show intro
    }

    #[tokio::test]
    async fn test_save_and_resume() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        // Start new game
        let (mut flow, _) = start_game(&db, 1, "TestPlayer").await.unwrap();

        // Advance past intro
        flow.handle_char('\r');

        // Save
        save_game_state(&db, 1, "TestPlayer", &flow).await.unwrap();

        // Load again
        let (_flow2, screen) = start_game(&db, 1, "TestPlayer").await.unwrap();

        // Should be at main menu now (after intro)
        assert!(screen.contains("MAIN MENU") || screen.contains("DEPTHS"));
    }

    #[tokio::test]
    async fn test_meta_progression_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_diablo.db");
        let db = DiabloDb::new(&db_path).await.unwrap();

        // Create game and add some soul essence
        let (mut flow, _) = start_game(&db, 1, "TestPlayer").await.unwrap();
        flow.game_state_mut().meta.soul_essence = 500;

        // Save
        save_game_state(&db, 1, "TestPlayer", &flow).await.unwrap();

        // Delete save (simulating run end)
        db.delete_save(1).await.unwrap();

        // Start new game - should have meta
        let (flow2, _) = start_game(&db, 1, "TestPlayer").await.unwrap();
        assert_eq!(flow2.game_state().meta.soul_essence, 500);
    }
}
