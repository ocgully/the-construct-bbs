//! Queens - Door Game Service
//!
//! A daily N-Queens puzzle game with colored regions.
//! Same puzzle worldwide each day (date-seeded).
//!
//! Uses __queens__ sentinel for session routing.
//! Game state persisted in game's own database (queens.db).

use crate::services::queens::db::{QueensDb, LeaderboardEntry};
use crate::games::queens::{GameState, PlayerStats, QueensFlow, GameScreen};
use crate::games::queens::render::*;

/// Sentinel for session routing
#[allow(dead_code)]
pub const SENTINEL: &str = "__queens__";

/// Get today's date in Eastern timezone (for puzzle consistency)
#[allow(dead_code)]
pub fn get_today_date() -> String {
    use chrono::{Utc, Duration};

    // Eastern time is UTC-5 (or UTC-4 during DST, but we use -5 for consistency)
    let eastern = Utc::now() - Duration::hours(5);
    eastern.format("%Y-%m-%d").to_string()
}

/// Initialize or resume a game session
#[allow(dead_code)]
pub async fn start_game(
    db: &QueensDb,
    user_id: i64,
    _handle: &str,
) -> Result<(QueensFlow, String), String> {
    let today = get_today_date();

    // Load player stats
    let stats = match db.get_player_stats(user_id).await {
        Ok(Some(row)) => row.to_player_stats(),
        Ok(None) => PlayerStats::new(),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Try to resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(state) => {
                    // Check if save is for today's puzzle
                    if state.puzzle_date == today && !state.completed {
                        let flow = QueensFlow::from_state(state, stats);
                        let screen = render_screen(&flow);
                        return Ok((flow, screen));
                    } else {
                        // Old save - delete it and start fresh
                        let _ = db.delete_save(user_id).await;
                    }
                }
                Err(_) => {
                    // Corrupted save - delete it
                    let _ = db.delete_save(user_id).await;
                }
            }
        }
        Ok(None) => {}
        Err(e) => return Err(format!("Database error: {}", e)),
    }

    // Start new game for today
    let flow = QueensFlow::new(&today, stats);
    let screen = render_screen(&flow);
    Ok((flow, screen))
}

/// Save current game state
#[allow(dead_code)]
pub async fn save_game_state(
    db: &QueensDb,
    user_id: i64,
    handle: &str,
    flow: &QueensFlow,
) -> Result<(), String> {
    let json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    // Also save player stats
    db.save_player_stats(user_id, handle, flow.player_stats())
        .await
        .map_err(|e| format!("Stats save error: {}", e))
}

/// Record game completion
#[allow(dead_code)]
pub async fn record_game_completion(
    db: &QueensDb,
    user_id: i64,
    handle: &str,
    flow: &mut QueensFlow,
    time_seconds: u32,
    hints_used: u32,
) -> Result<(), String> {
    let puzzle_date = flow.game_state().puzzle_date.clone();

    // Update player stats
    flow.stats.record_completion(&puzzle_date, time_seconds, hints_used);

    // Save completion to DB
    db.record_completion(user_id, handle, &puzzle_date, time_seconds, hints_used)
        .await
        .map_err(|e| format!("Record error: {}", e))?;

    // Save updated stats
    db.save_player_stats(user_id, handle, &flow.stats)
        .await
        .map_err(|e| format!("Stats error: {}", e))?;

    // Delete in-progress save
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard entries
#[allow(dead_code)]
pub async fn get_game_leaderboard(db: &QueensDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
#[allow(dead_code)]
pub fn render_screen(flow: &QueensFlow) -> String {
    let state = flow.game_state();
    let stats = flow.player_stats();
    let puzzle = &flow.puzzle;

    match flow.current_screen() {
        GameScreen::Intro => render_intro(puzzle, stats),
        GameScreen::Playing => render_playing(state, stats, puzzle),
        GameScreen::Victory => render_victory(state, stats, puzzle),
        GameScreen::AlreadyPlayed => render_already_played(stats, puzzle),
        GameScreen::Stats => render_stats(stats),
        GameScreen::Leaderboard => {
            // For now, render with empty entries - actual data would come from DB
            render_leaderboard(&[])
        }
        GameScreen::ConfirmQuit => render_confirm_quit(),
        GameScreen::Help => render_help(),
    }
}
