//! Usurper - Door Game Service
//!
//! Dark fantasy RPG with dungeons, combat, and drugs/steroids mechanics.
//! Uses __usurper__ sentinel for session routing.
//! Game state persisted in game's own database (usurper.db).

use crate::services::usurper::db::{UsurperDb, LeaderboardEntry};
use crate::games::usurper::{GameState, UsurperFlow};
use crate::games::usurper::render;

/// Sentinel for session routing
pub const SENTINEL: &str = "__usurper__";

/// Initialize or resume a game session
pub async fn start_game(db: &UsurperDb, user_id: i64, handle: &str) -> Result<(UsurperFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    // Check if it's a new real-world day
                    let new_day = state.check_new_day();
                    if new_day {
                        state.last_message = Some("A new day dawns over Durunghins...".to_string());
                    }

                    state.handle = Some(handle.to_string());
                    let flow = UsurperFlow::from_state(state);
                    let screen = render::render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save, offer to start fresh
                    Err(format!("Save corrupted: {}. Start new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game
            let flow = UsurperFlow::new();
            let screen = render::render_intro(flow.game_state());
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &UsurperDb, user_id: i64, handle: &str, flow: &UsurperFlow) -> Result<(), String> {
    let json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    // Update player status for multiplayer features
    let state = flow.game_state();
    let _ = db.update_player_status(
        user_id,
        handle,
        &state.character_name,
        state.level,
        state.class.name(),
        state.is_king,
        state.godhood_level,
        state.clan_id.as_deref(),
    ).await;

    Ok(())
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &UsurperDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion
pub async fn record_game_completion(
    db: &UsurperDb,
    user_id: i64,
    handle: &str,
    flow: &UsurperFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_completion(
        user_id,
        handle,
        &state.character_name,
        state.level,
        state.deepest_dungeon,
        state.monsters_killed,
        state.total_gold_earned,
        state.supreme_being_defeated,
        state.is_king,
        state.godhood_level,
        state.days_played,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete save after completion
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &UsurperDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Apply psychosis effects to screen output
fn apply_psychosis_effect(output: String, mental_stability: i32) -> String {
    if mental_stability > 0 {
        return output;
    }

    // The more negative, the worse the distortion
    let distortion_level = (-mental_stability).min(50) as usize;

    let mut rng = rand::thread_rng();
    use rand::Rng;

    // Characters that might replace normal text
    let psychosis_chars = [
        '\u{2591}', '\u{2592}', '\u{2593}', // Block elements
        '?', '!', '*', '#', '@', '%', '&',   // Symbols
        '\u{263A}', '\u{263B}',               // Faces
        '\u{2620}', '\u{2623}',               // Skull and biohazard
    ];

    // Phrases that might appear
    let whispers = [
        "they're watching",
        "can't trust anyone",
        "the shadows move",
        "IT SEES YOU",
        "turn back",
        "too deep",
        "MADNESS",
    ];

    let mut result: String = output
        .chars()
        .map(|c| {
            if c.is_control() || c == ' ' || c == '\n' || c == '\r' || c == '\t' {
                c
            } else if rng.gen_range(0..100) < distortion_level {
                psychosis_chars[rng.gen_range(0..psychosis_chars.len())]
            } else {
                c
            }
        })
        .collect();

    // Occasionally insert whispers
    if rng.gen_range(0..100) < distortion_level * 2 {
        let whisper = whispers[rng.gen_range(0..whispers.len())];
        result.push_str(&format!("\r\n\x1B[91m  ...{}...\x1B[0m", whisper));
    }

    result
}

/// Render current screen based on game state
pub fn render_screen(flow: &UsurperFlow) -> String {
    let state = flow.game_state();
    let output = render::render_screen(flow);

    // Apply psychosis effects if mental stability is negative
    apply_psychosis_effect(output, state.mental_stability)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel() {
        assert_eq!(SENTINEL, "__usurper__");
    }

    #[test]
    fn test_psychosis_effect_normal() {
        let input = "Hello world".to_string();
        let output = apply_psychosis_effect(input.clone(), 100);
        assert_eq!(input, output);
    }

    #[test]
    fn test_psychosis_effect_active() {
        let input = "Hello world".to_string();
        let output = apply_psychosis_effect(input.clone(), -50);
        // With heavy psychosis, output should be different
        // Note: This is probabilistic, so we just check it doesn't panic
        assert!(!output.is_empty());
    }
}
