//! Sudoku - Door Game Service
//!
//! Daily puzzle game where all players get the same puzzle each day.
//! Tracks streaks, completion times, and provides leaderboards.
//!
//! Uses __sudoku__ sentinel for session routing.
//! Game state persisted in sudoku.db.

use crate::services::sudoku::db::SudokuDb;
use crate::games::sudoku::{GameState, SudokuFlow};
use crate::games::sudoku::render;
use crate::games::sudoku::generator::get_eastern_date;

/// Sentinel for session routing
pub const SENTINEL: &str = "__sudoku__";

/// Initialize or resume a game session
pub async fn start_game(db: &SudokuDb, user_id: i64, handle: &str) -> Result<(SudokuFlow, String), String> {
    let today = get_eastern_date();

    // Get player stats
    let stats = db.get_or_create_stats(user_id, handle)
        .await
        .map_err(|e| format!("Stats error: {}", e))?;

    // Check if already completed today
    let completion_time = db.has_completed_today(user_id, &today)
        .await
        .map_err(|e| format!("Completion check error: {}", e))?;

    if let Some(time) = completion_time {
        // Already completed - show completion screen
        let flow = SudokuFlow::with_stats(
            true,
            Some(time),
            stats.current_streak,
            stats.longest_streak,
        );
        let screen = render::render_screen(&flow);
        return Ok((flow, screen));
    }

    // Check for existing save
    let saved = db.load_game(user_id, &today)
        .await
        .map_err(|e| format!("Load error: {}", e))?;

    if let Some(json) = saved {
        // Resume existing game
        match serde_json::from_str::<GameState>(&json) {
            Ok(state) => {
                let mut flow = SudokuFlow::with_stats(
                    false,
                    None,
                    stats.current_streak,
                    stats.longest_streak,
                );
                flow.resume_puzzle(state);
                let screen = render::render_screen(&flow);
                return Ok((flow, screen));
            }
            Err(e) => {
                // Corrupted save - delete and start fresh
                let _ = db.delete_save(user_id, &today).await;
                return Err(format!("Save corrupted: {}. Please try again.", e));
            }
        }
    }

    // New game
    let flow = SudokuFlow::with_stats(
        false,
        None,
        stats.current_streak,
        stats.longest_streak,
    );
    let screen = render::render_intro(&flow);
    Ok((flow, screen))
}

/// Save current game state
pub async fn save_game_state(db: &SudokuDb, user_id: i64, flow: &SudokuFlow) -> Result<(), String> {
    if let Some(state) = flow.game_state() {
        let json = serde_json::to_string(state)
            .map_err(|e| format!("Serialize error: {}", e))?;

        db.save_game(user_id, &state.puzzle_date, &json)
            .await
            .map_err(|e| format!("Save error: {}", e))?;
    }
    Ok(())
}

/// Record puzzle completion
pub async fn record_completion(
    db: &SudokuDb,
    user_id: i64,
    handle: &str,
    flow: &SudokuFlow,
) -> Result<(u32, u32), String> {
    if let Some(state) = flow.game_state() {
        let (current_streak, longest_streak) = db.record_completion(
            user_id,
            handle,
            &state.puzzle_date,
            state.elapsed_seconds,
            state.error_count,
        )
        .await
        .map_err(|e| format!("Record error: {}", e))?;

        return Ok((current_streak, longest_streak));
    }
    Err("No game state to record".to_string())
}

/// Get leaderboard for display
pub async fn get_leaderboard(db: &SudokuDb) -> Vec<(String, u32, u32)> {
    match db.get_leaderboard_tuples(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
pub fn render_screen(flow: &SudokuFlow) -> String {
    render::render_screen(flow)
}

/// Render stats screen with leaderboard data
pub fn render_stats_screen(
    flow: &SudokuFlow,
    games_completed: u32,
    best_time: Option<u32>,
    leaderboard: &[(String, u32, u32)],
) -> String {
    render::render_stats(
        flow.current_streak,
        flow.longest_streak,
        games_completed,
        best_time,
        leaderboard,
    )
}

/// Render completion screen with updated streaks
pub fn render_completion_screen(flow: &SudokuFlow, current_streak: u32, longest_streak: u32) -> String {
    if let Some(state) = flow.game_state() {
        render::render_completed(state, current_streak, longest_streak)
    } else {
        render::render_screen(flow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel() {
        assert_eq!(SENTINEL, "__sudoku__");
    }
}
