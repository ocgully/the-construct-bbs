//! Summit game service
//!
//! Entry points for session management and game flow.
//! Uses __summit__ sentinel for session routing.

use crate::games::summit::{SummitFlow, PlayerStats};
use crate::games::summit::lobby::SummitLobby;
use crate::games::summit::render;
use crate::services::summit::db::SummitDb;

/// Sentinel for session routing
pub const SENTINEL: &str = "__summit__";

/// Initialize or resume a game session
pub async fn start_game(
    db: &SummitDb,
    user_id: i64,
    handle: &str,
) -> Result<(SummitFlow, String), String> {
    // Load player stats or create new
    let stats = match db.load_stats(user_id).await {
        Ok(Some(json)) => {
            serde_json::from_str::<PlayerStats>(&json)
                .unwrap_or_else(|_| PlayerStats::new(user_id))
        }
        _ => PlayerStats::new(user_id),
    };

    let flow = SummitFlow::new(user_id, handle.to_string(), stats);

    // Create temporary lobby for rendering
    let lobby_manager = SummitLobby::new();
    let screen = render::render_screen(&flow, &lobby_manager);

    Ok((flow, screen))
}

/// Save player statistics
pub async fn save_player_stats(
    db: &SummitDb,
    user_id: i64,
    handle: &str,
    stats: &PlayerStats,
) -> Result<(), String> {
    let json = serde_json::to_string(stats)
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_stats(user_id, handle, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))
}

/// Render current screen
pub fn render_screen(flow: &SummitFlow, lobby_manager: &SummitLobby) -> String {
    render::render_screen(flow, lobby_manager)
}

/// Record a completed run
pub async fn record_run_completion(
    db: &SummitDb,
    user_id: i64,
    handle: &str,
    date: &str,
    reached_summit: bool,
    duration_seconds: i64,
    height_reached: i64,
    party_size: i64,
    falls: i64,
) -> Result<(), String> {
    // Record in completions table
    db.record_completion(
        user_id, handle, date,
        reached_summit, duration_seconds,
        height_reached, party_size, falls
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Update leaderboard if summit reached
    if reached_summit {
        db.update_daily_leaderboard(date, user_id, handle, duration_seconds, party_size)
            .await
            .map_err(|e| format!("Leaderboard error: {}", e))?;
    }

    Ok(())
}

/// Get daily leaderboard
pub async fn get_daily_leaderboard(db: &SummitDb, date: &str) -> Vec<crate::services::summit::db::LeaderboardEntry> {
    match db.get_daily_leaderboard(date, 10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Get all-time leaderboard
pub async fn get_all_time_leaderboard(db: &SummitDb) -> Vec<crate::services::summit::db::LeaderboardEntry> {
    match db.get_all_time_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::summit::screen::{GameScreen, LobbyScreen};
    use tempfile::tempdir;

    async fn create_test_db() -> SummitDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_summit.db");
        SummitDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_start_game_new_player() {
        let db = create_test_db().await;

        let (flow, screen) = start_game(&db, 1, "NewPlayer").await.unwrap();

        assert_eq!(flow.user_id, 1);
        assert_eq!(flow.handle, "NewPlayer");
        assert!(matches!(flow.screen, GameScreen::Lobby(LobbyScreen::MainMenu)));
        assert!(screen.contains("Summit")); // In "Reach the Summit"
    }

    #[tokio::test]
    async fn test_save_and_load_stats() {
        let db = create_test_db().await;

        let mut stats = PlayerStats::new(1);
        stats.total_summits = 5;
        stats.award_badge("first_summit");

        save_player_stats(&db, 1, "TestUser", &stats).await.unwrap();

        // Start game again - should load saved stats
        let (flow, _) = start_game(&db, 1, "TestUser").await.unwrap();

        assert_eq!(flow.get_stats().total_summits, 5);
        assert!(flow.get_stats().has_badge("first_summit"));
    }

    #[tokio::test]
    async fn test_record_completion() {
        let db = create_test_db().await;

        record_run_completion(
            &db, 1, "TestUser", "2026-01-30",
            true, 600, 100, 2, 3
        ).await.unwrap();

        let leaders = get_daily_leaderboard(&db, "2026-01-30").await;
        assert_eq!(leaders.len(), 1);
        assert_eq!(leaders[0].handle, "TestUser");
    }

    #[tokio::test]
    async fn test_leaderboard_ordering() {
        let db = create_test_db().await;

        record_run_completion(&db, 1, "Slow", "2026-01-30", true, 800, 100, 1, 0).await.unwrap();
        record_run_completion(&db, 2, "Fast", "2026-01-30", true, 400, 100, 1, 0).await.unwrap();
        record_run_completion(&db, 3, "Medium", "2026-01-30", true, 600, 100, 1, 0).await.unwrap();

        let leaders = get_daily_leaderboard(&db, "2026-01-30").await;
        assert_eq!(leaders.len(), 3);
        assert_eq!(leaders[0].handle, "Fast");
        assert_eq!(leaders[1].handle, "Medium");
        assert_eq!(leaders[2].handle, "Slow");
    }
}
