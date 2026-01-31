//! Last Dream - Door Game Service
//!
//! A classic JRPG in the style of Final Fantasy 1-2.
//! Features party-based combat, overworld exploration, and a hidden
//! simulation twist revealed only at the very end.
//!
//! Uses __last_dream__ sentinel for session routing.

#![allow(dead_code)]

use crate::services::last_dream::db::{LastDreamDb, LeaderboardEntry};
use crate::games::last_dream::{GameState, LastDreamFlow};
use crate::games::last_dream::render::render_screen as game_render_screen;

/// Sentinel for session routing
pub const SENTINEL: &str = "__last_dream__";

/// Initialize or resume a game session
pub async fn start_game(
    db: &LastDreamDb,
    user_id: i64,
    handle: &str,
) -> Result<(LastDreamFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    state.handle = Some(handle.to_string());
                    let flow = LastDreamFlow::from_state(state);
                    let screen = game_render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save - start new
                    Err(format!("Save corrupted: {}. Starting new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game - start party creation
            let mut flow = LastDreamFlow::new();
            flow.state.handle = Some(handle.to_string());
            let screen = game_render_screen(&flow);
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(
    db: &LastDreamDb,
    user_id: i64,
    handle: &str,
    flow: &LastDreamFlow,
) -> Result<(), String> {
    let state = flow.game_state();
    let json = serde_json::to_string(state)
        .map_err(|e| format!("Serialize error: {}", e))?;

    // Calculate crystals lit
    let crystals_lit = [
        state.has_flag("earth_crystal_lit"),
        state.has_flag("fire_crystal_lit"),
        state.has_flag("water_crystal_lit"),
        state.has_flag("wind_crystal_lit"),
    ].iter().filter(|&&b| b).count() as u8;

    // Get average party level
    let party_level = state.party.average_level();

    db.save_game(
        user_id,
        handle,
        &json,
        state.play_time,
        party_level,
        crystals_lit,
    )
    .await
    .map_err(|e| format!("Save error: {}", e))?;

    // Save party member info for display
    for (i, member) in state.party.members.iter().enumerate() {
        let _ = db.save_party_member(
            user_id,
            i as u8,
            &member.name,
            member.class.name(),
            member.level,
        ).await;
    }

    Ok(())
}

/// Render current screen based on game state
pub fn render_screen(flow: &LastDreamFlow) -> String {
    game_render_screen(flow)
}

/// Record game completion
pub async fn record_game_completion(
    db: &LastDreamDb,
    user_id: i64,
    handle: &str,
    flow: &LastDreamFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    // Build party info string
    let party_info: String = state.party.members.iter()
        .map(|m| format!("{} L{}", m.name, m.level))
        .collect::<Vec<_>>()
        .join(" / ");

    let crystals_lit = [
        state.has_flag("earth_crystal_lit"),
        state.has_flag("fire_crystal_lit"),
        state.has_flag("water_crystal_lit"),
        state.has_flag("wind_crystal_lit"),
    ].iter().filter(|&&b| b).count() as u8;

    db.record_completion(
        user_id,
        handle,
        &party_info,
        state.play_time,
        crystals_lit,
        state.monsters_defeated,
        state.battles_fought,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete save after completion (game is done)
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &LastDreamDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &LastDreamDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Format play time for leaderboard display
pub fn format_play_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_play_time() {
        assert_eq!(format_play_time(0), "00:00:00");
        assert_eq!(format_play_time(61), "00:01:01");
        assert_eq!(format_play_time(3661), "01:01:01");
        assert_eq!(format_play_time(7200), "02:00:00");
    }

    #[test]
    fn test_sentinel() {
        assert_eq!(SENTINEL, "__last_dream__");
    }
}
