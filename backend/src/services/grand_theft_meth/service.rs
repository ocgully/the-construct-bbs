//! Grand Theft Meth - Door Game Service
//!
//! A commodity trading game where players buy/sell drugs across cities,
//! encounter random events, manage debt, and compete for high scores.
//!
//! Uses __game_gtm__ sentinel for session routing (similar to __chat__, __mail__).
//! Game state persisted in game's own database (grand_theft_meth.db).

use crate::services::grand_theft_meth::db::{GtmDb, LeaderboardEntry};
use crate::game::{GameState, GtmFlow, GtmAction, GameScreen};
use crate::game::render::*;

/// Sentinel for session routing
pub const SENTINEL: &str = "__game_gtm__";

/// Initialize or resume a game session
pub async fn start_game(db: &GtmDb, user_id: i64, handle: &str) -> Result<(GtmFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(state) => {
                    let flow = GtmFlow::from_state(state);
                    let screen = render_screen(&flow);
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
            let flow = GtmFlow::new();
            let screen = render_intro();
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &GtmDb, user_id: i64, handle: &str, flow: &GtmFlow) -> Result<(), String> {
    let json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))
}

/// Delete save and start fresh
pub async fn clear_save(db: &GtmDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion
pub async fn record_game_completion(
    db: &GtmDb,
    user_id: i64,
    handle: &str,
    flow: &GtmFlow,
) -> Result<(), String> {
    let state = flow.game_state();
    let prices = &flow.prices;
    let final_score = state.net_worth(prices);
    let days_played = state.day as i64;
    let story_completed = state.quest_state.story_step >= 15;

    db.record_completion(
        user_id,
        handle,
        final_score,
        days_played,
        story_completed,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete save after completion
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &GtmDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
pub fn render_screen(flow: &GtmFlow) -> String {
    let state = flow.game_state();
    let prices = &flow.prices;

    match flow.current_screen() {
        GameScreen::Intro => render_intro(),
        GameScreen::MainMenu => render_main_menu(state, prices),
        GameScreen::Travel { selecting_city } => render_travel(state, prices, *selecting_city),
        GameScreen::Trade { mode } => render_trade(state, prices, mode),
        GameScreen::Combat { enemy_type, enemy_hp } => render_combat(state, prices, enemy_type, *enemy_hp),
        GameScreen::Event { event } => render_event(state, prices, event),
        GameScreen::LoanShark => render_loan_shark(state, prices),
        GameScreen::Bank => render_bank(state, prices),
        GameScreen::Hospital { is_mob_doctor } => render_hospital(state, prices, *is_mob_doctor),
        GameScreen::GunShop => render_gun_shop(state, prices),
        GameScreen::Quest => render_quest(state, prices),
        GameScreen::Casino { .. } => {
            // Casino rendering not yet implemented
            render_main_menu(state, prices)
        }
        GameScreen::GameOver => render_game_over(state, prices),
        GameScreen::Leaderboard => "Loading leaderboard...".to_string(),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}
