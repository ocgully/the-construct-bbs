//! Fortress service - Session routing and game coordination
//!
//! Uses __fortress__ sentinel for session routing.

#![allow(dead_code)]

use crate::services::fortress::db::{FortressDb, LeaderboardEntry};
use crate::games::fortress::{GameState, FortressFlow, GameScreen};
use crate::games::fortress::render::*;

/// Sentinel for session routing
pub const SENTINEL: &str = "__fortress__";

/// Initialize or resume a game session
pub async fn start_game(db: &FortressDb, user_id: i64, _handle: &str) -> Result<(FortressFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(state) => {
                    let flow = FortressFlow::from_state(state);
                    let screen = render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save
                    Err(format!("Save corrupted: {}. Starting new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game
            let flow = FortressFlow::new();
            let screen = render_intro(flow.game_state());
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &FortressDb, user_id: i64, handle: &str, flow: &FortressFlow) -> Result<(), String> {
    let json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &FortressDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion
pub async fn record_game_completion(
    db: &FortressDb,
    user_id: i64,
    handle: &str,
    flow: &FortressFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_completion(
        user_id,
        handle,
        &state.fortress_name,
        state.fortress_value() as i64,
        state.year as i64,
        state.stats.peak_population as i64,
        state.stats.invasions_repelled as i64,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete save after completion
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &FortressDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
pub fn render_screen(flow: &FortressFlow) -> String {
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(state),
        GameScreen::Naming => render_naming(state),
        GameScreen::FortressView => render_fortress_view(state),
        GameScreen::DwarfList => render_dwarf_list(state),
        GameScreen::DwarfDetail { dwarf_id } => render_dwarf_detail(state, *dwarf_id),
        GameScreen::Workshops => render_workshops(state),
        GameScreen::WorkshopDetail { workshop_id } => render_workshop_detail(state, *workshop_id),
        GameScreen::RoomDesign => render_fortress_view(state), // TODO: Dedicated render
        GameScreen::JobQueue => render_fortress_view(state), // TODO: Dedicated render
        GameScreen::WorkOrders => render_fortress_view(state), // TODO: Dedicated render
        GameScreen::Military => render_fortress_view(state), // TODO: Dedicated render
        GameScreen::Stockpiles => render_stockpiles(state),
        GameScreen::Designate => render_designate(state),
        GameScreen::BuildMenu => render_build_menu(state),
        GameScreen::Combat => render_fortress_view(state), // TODO: Dedicated render
        GameScreen::Statistics => render_statistics(state),
        GameScreen::Help => render_help(state),
        GameScreen::ConfirmQuit => render_confirm_quit(state),
        GameScreen::GameOver => render_game_over(state),
    }
}
