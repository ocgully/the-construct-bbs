//! Star Trader - Service Entry Points
//!
//! Handles session initialization, save/load, and screen rendering.

use crate::services::star_trader::db::{StarTraderDb, LeaderboardEntry};
use crate::games::star_trader::{GameState, StarTraderFlow};
use crate::games::star_trader::galaxy::Galaxy;
use crate::games::star_trader::render;
use crate::games::star_trader::data::config;

/// Sentinel for session routing
pub const SENTINEL: &str = "__star_trader__";

/// Default galaxy seed (perpetual galaxy)
const DEFAULT_GALAXY_SEED: u64 = 2002_0125;  // Trade Wars tribute

/// Default galaxy size
const DEFAULT_GALAXY_SIZE: u32 = config::GALAXY_MEDIUM;

/// Initialize or resume a game session
pub async fn start_game(db: &StarTraderDb, user_id: i64, handle: &str) -> Result<(StarTraderFlow, String), String> {
    // Get or create the galaxy
    let (_galaxy_id, galaxy_json) = db.get_or_create_galaxy(DEFAULT_GALAXY_SEED, DEFAULT_GALAXY_SIZE)
        .await
        .map_err(|e| format!("Galaxy error: {}", e))?;

    let galaxy: Galaxy = serde_json::from_str(&galaxy_json)
        .map_err(|e| format!("Galaxy parse error: {}", e))?;

    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some((_saved_galaxy_id, state_json))) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&state_json) {
                Ok(mut state) => {
                    // Check for new day
                    let new_day = state.check_new_day();
                    if new_day {
                        state.last_message = Some("A new day dawns. Turns refreshed!".to_string());
                    }

                    let flow = StarTraderFlow::from_state(state, galaxy);
                    let screen = render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(_) => {
                    // Corrupted save - start fresh
                    let _ = db.delete_save(user_id).await;
                    let flow = StarTraderFlow::new(user_id, handle.to_string(), galaxy);
                    let screen = render::render_intro();
                    Ok((flow, screen))
                }
            }
        }
        Ok(None) => {
            // New game
            let flow = StarTraderFlow::new(user_id, handle.to_string(), galaxy);
            let screen = render::render_intro();
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(
    db: &StarTraderDb,
    user_id: i64,
    handle: &str,
    flow: &StarTraderFlow
) -> Result<(), String> {
    // Get galaxy ID
    let (galaxy_id, _) = db.get_or_create_galaxy(DEFAULT_GALAXY_SEED, DEFAULT_GALAXY_SIZE)
        .await
        .map_err(|e| format!("Galaxy error: {}", e))?;

    // Serialize state
    let state_json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    // Save
    db.save_game(user_id, handle, galaxy_id, &state_json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    // Also save galaxy state (trades affect ports)
    let galaxy_json = serde_json::to_string(flow.galaxy())
        .map_err(|e| format!("Galaxy serialize error: {}", e))?;

    db.save_galaxy(galaxy_id, &galaxy_json)
        .await
        .map_err(|e| format!("Galaxy save error: {}", e))?;

    Ok(())
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &StarTraderDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion (for leaderboard)
pub async fn record_game_completion(
    db: &StarTraderDb,
    user_id: i64,
    handle: &str,
    flow: &StarTraderFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_score(
        user_id,
        handle,
        state.credits,
        state.experience,
        state.kills,
        state.stats.sectors_explored,
        state.federation_rank.name(),
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Optionally delete save after recording (or keep for "continue playing")
    // let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard entries
pub async fn get_game_leaderboard(db: &StarTraderDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
pub fn render_screen(flow: &StarTraderFlow) -> String {
    render::render_screen(flow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::star_trader::screen::GameScreen;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_new_game() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_star_trader.db");
        let db = StarTraderDb::new(&db_path).await.unwrap();

        let result = start_game(&db, 1, "TestPlayer").await;
        assert!(result.is_ok());

        let (flow, screen) = result.unwrap();
        assert!(matches!(flow.current_screen(), GameScreen::Intro));
        // The header is ASCII art that spells STAR TRADER, not literal text
        // Check for intro text elements instead
        assert!(screen.contains("trader"));
        assert!(screen.contains("Press any key"));
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_star_trader.db");
        let db = StarTraderDb::new(&db_path).await.unwrap();

        // Start new game
        let (flow, _) = start_game(&db, 1, "TestPlayer").await.unwrap();

        // Save
        save_game_state(&db, 1, "TestPlayer", &flow).await.unwrap();

        // Load again
        let (flow2, _) = start_game(&db, 1, "TestPlayer").await.unwrap();
        assert_eq!(flow2.game_state().handle, "TestPlayer");
    }
}
