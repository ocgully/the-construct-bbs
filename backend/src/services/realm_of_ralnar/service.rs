//! Realm of Ralnar - Service Entry Points
//!
//! Handles session initialization, save/load, and screen rendering.

use crate::games::realm_of_ralnar::{GameScreen, GameState, RalnarFlow};
use crate::games::realm_of_ralnar::render;
use super::db::{RalnarDb, LeaderboardEntry};

/// Sentinel for session routing
pub const SENTINEL: &str = "__realm_of_ralnar__";

/// Initialize or resume a game session
pub async fn start_game(
    db: &RalnarDb,
    user_id: i64,
    handle: &str,
) -> Result<(RalnarFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(state_json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&state_json) {
                Ok(state) => {
                    // Resume from saved state - screen is determined by flow
                    let flow = RalnarFlow::from_state(state);
                    let rendered = render_screen(&flow);
                    Ok((flow, rendered))
                }
                Err(_) => {
                    // Corrupted save - start fresh
                    let _ = db.delete_save(user_id).await;
                    let flow = RalnarFlow::new(user_id, handle.to_string());
                    let screen = render::render_screen(&flow);
                    Ok((flow, screen))
                }
            }
        }
        Ok(None) => {
            // New game with Herbert as the main character
            let flow = RalnarFlow::new(user_id, handle.to_string());
            let screen = render::render_screen(&flow);
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(
    db: &RalnarDb,
    user_id: i64,
    handle: &str,
    flow: &RalnarFlow,
) -> Result<(), String> {
    // Serialize state
    let state_json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    // Save
    db.save_game(user_id, handle, &state_json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    Ok(())
}

/// Delete save and start fresh
pub async fn clear_save(db: &RalnarDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Render current screen based on game state
pub fn render_screen(flow: &RalnarFlow) -> String {
    render::render_screen(flow)
}

/// Record game completion (for leaderboard and history)
pub async fn record_game_completion(
    db: &RalnarDb,
    user_id: i64,
    handle: &str,
    flow: &RalnarFlow,
    ending_type: &str,
) -> Result<(), String> {
    let state = flow.game_state();

    // Calculate total party experience
    let total_exp: i32 = state.party.members.iter()
        .map(|m| m.exp as i32)
        .sum();

    // Record in completions
    db.record_completion(
        user_id,
        handle,
        ending_type,
        state.play_time_seconds as i64,
        state.shrines_destroyed_count() as i32,
        total_exp,
        state.gold as i32,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Also add to leaderboard
    db.record_leaderboard(
        user_id,
        handle,
        ending_type,
        state.play_time_seconds as i64,
        state.shrines_destroyed_count() as i32,
    )
    .await
    .map_err(|e| format!("Leaderboard error: {}", e))?;

    // Delete save after recording completion
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get game leaderboard
pub async fn get_game_leaderboard(db: &RalnarDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Get user's best time
pub async fn get_user_best(db: &RalnarDb, user_id: i64) -> Option<LeaderboardEntry> {
    match db.get_user_best_time(user_id).await {
        Ok(entry) => entry,
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> RalnarDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_ralnar.db");
        RalnarDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_new_game() {
        let db = create_test_db().await;
        let (flow, screen) = start_game(&db, 1, "TestPlayer").await.unwrap();

        assert!(matches!(flow.current_screen(), GameScreen::Intro));
        assert!(screen.contains("Ralnar"));
        // Should have Herbert as the party leader
        assert!(!flow.game_state().party.members.is_empty());
        let leader = flow.game_state().party.leader().unwrap();
        assert_eq!(leader.name, "TestPlayer");
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let db = create_test_db().await;

        // Start new game
        let (mut flow, _) = start_game(&db, 1, "TestPlayer").await.unwrap();

        // Modify state
        flow.game_state_mut().gold = 500;
        flow.game_state_mut().current_map = "northern_path".to_string();

        // Save
        save_game_state(&db, 1, "TestPlayer", &flow).await.unwrap();

        // Load again
        let (flow2, _) = start_game(&db, 1, "TestPlayer").await.unwrap();
        assert_eq!(flow2.game_state().gold, 500);
        assert_eq!(flow2.game_state().current_map, "northern_path");
    }

    #[tokio::test]
    async fn test_render_screen() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let screen = render_screen(&flow);
        assert!(screen.contains("Ralnar"));
    }

    #[tokio::test]
    async fn test_record_completion() {
        let db = create_test_db().await;

        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.game_state_mut().gold = 1000;
        flow.game_state_mut().shrines_destroyed[0] = true;

        record_game_completion(&db, 1, "TestPlayer", &flow, "victory")
            .await
            .unwrap();

        // Should be on leaderboard
        let leaders = get_game_leaderboard(&db).await;
        assert_eq!(leaders.len(), 1);
        assert_eq!(leaders[0].handle, "TestPlayer");
        assert_eq!(leaders[0].ending_type, "victory");
    }

    #[tokio::test]
    async fn test_clear_save() {
        let db = create_test_db().await;

        // Start and save
        let (flow, _) = start_game(&db, 1, "TestPlayer").await.unwrap();
        save_game_state(&db, 1, "TestPlayer", &flow).await.unwrap();

        // Verify save exists
        assert!(db.has_save(1).await.unwrap());

        // Clear
        clear_save(&db, 1).await.unwrap();

        // Verify save gone
        assert!(!db.has_save(1).await.unwrap());
    }

    #[tokio::test]
    async fn test_leaderboard_ordering() {
        let db = create_test_db().await;

        // Create completions with different times
        let mut flow1 = RalnarFlow::new(1, "Slow".to_string());
        flow1.game_state_mut().play_time_seconds = 7200;

        let mut flow2 = RalnarFlow::new(2, "Fast".to_string());
        flow2.game_state_mut().play_time_seconds = 1800;

        let mut flow3 = RalnarFlow::new(3, "Medium".to_string());
        flow3.game_state_mut().play_time_seconds = 3600;

        record_game_completion(&db, 1, "Slow", &flow1, "victory").await.unwrap();
        record_game_completion(&db, 2, "Fast", &flow2, "victory").await.unwrap();
        record_game_completion(&db, 3, "Medium", &flow3, "victory").await.unwrap();

        let leaders = get_game_leaderboard(&db).await;
        assert_eq!(leaders.len(), 3);
        assert_eq!(leaders[0].handle, "Fast");
        assert_eq!(leaders[1].handle, "Medium");
        assert_eq!(leaders[2].handle, "Slow");
    }

    #[tokio::test]
    async fn test_user_best() {
        let db = create_test_db().await;

        let mut flow1 = RalnarFlow::new(1, "Player".to_string());
        flow1.game_state_mut().play_time_seconds = 7200;
        record_game_completion(&db, 1, "Player", &flow1, "victory").await.unwrap();

        let mut flow2 = RalnarFlow::new(1, "Player".to_string());
        flow2.game_state_mut().play_time_seconds = 3600;
        record_game_completion(&db, 1, "Player", &flow2, "victory").await.unwrap();

        let best = get_user_best(&db, 1).await;
        assert!(best.is_some());
        assert_eq!(best.unwrap().playtime_seconds, 3600);
    }

    #[tokio::test]
    async fn test_party_initialized() {
        let db = create_test_db().await;
        let (flow, _) = start_game(&db, 1, "TestPlayer").await.unwrap();

        // Should have at least one party member (Herbert)
        let state = flow.game_state();
        assert!(!state.party.members.is_empty());

        // First member should be a brother (cannot be removed)
        let leader = state.party.leader().unwrap();
        assert!(leader.is_brother);
        assert!(leader.hp > 0);
    }

    #[tokio::test]
    async fn test_shrine_tracking() {
        let db = create_test_db().await;

        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());

        // No shrines initially
        assert_eq!(flow.game_state().shrines_destroyed_count(), 0);

        // Destroy some shrines
        flow.game_state_mut().shrines_destroyed[0] = true;
        flow.game_state_mut().shrines_destroyed[2] = true;
        assert_eq!(flow.game_state().shrines_destroyed_count(), 2);

        // Save and reload
        save_game_state(&db, 1, "TestPlayer", &flow).await.unwrap();
        let (flow2, _) = start_game(&db, 1, "TestPlayer").await.unwrap();
        assert_eq!(flow2.game_state().shrines_destroyed_count(), 2);
    }
}
