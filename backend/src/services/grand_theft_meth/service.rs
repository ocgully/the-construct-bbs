//! Grand Theft Meth - Door Game Service
//!
//! A commodity trading game where players buy/sell drugs across cities,
//! encounter random events, manage debt, and compete for high scores.
//!
//! Uses __game_gtm__ sentinel for session routing (similar to __chat__, __mail__).
//! Game state persisted in game's own database (grand_theft_meth.db).

use rand::Rng;
use crate::services::grand_theft_meth::db::{GtmDb, LeaderboardEntry};
use crate::game::{GameState, GtmFlow, GameScreen};
use crate::game::render::*;

/// Sentinel for session routing
pub const SENTINEL: &str = "__game_gtm__";

/// Initialize or resume a game session
pub async fn start_game(db: &GtmDb, user_id: i64, _handle: &str) -> Result<(GtmFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    // Check if it's a new real-world day - sober up if so
                    let sobered_up = state.check_new_day_sober_up();
                    if sobered_up {
                        state.last_message = Some("You slept it off. Feeling clearer now.".to_string());
                    }

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
#[allow(dead_code)]
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

/// Apply high effect - randomly block out characters based on tier
/// Tier 1: 5%, Tier 2: 10%, Tier 3: 20%
fn apply_high_effect(output: String, high_tier: u8) -> String {
    if high_tier == 0 {
        return output;
    }

    let block_chance = match high_tier {
        1 => 5,
        2 => 10,
        _ => 20,  // tier 3+
    };

    let mut rng = rand::thread_rng();
    let block_chars = ['░', '▒', '▓', '█', '?', '*', '#', '@', '!', '~'];

    output
        .chars()
        .map(|c| {
            // Don't block control characters, spaces, or newlines
            if c.is_control() || c == ' ' || c == '\n' || c == '\r' || c == '\t' {
                c
            } else if rng.gen_range(0..100) < block_chance {
                // Replace with random block character
                block_chars[rng.gen_range(0..block_chars.len())]
            } else {
                c
            }
        })
        .collect()
}

/// Render current screen based on game state
pub fn render_screen(flow: &GtmFlow) -> String {
    let state = flow.game_state();
    let prices = &flow.prices;
    let high_tier = state.high_tier;

    let output = match flow.current_screen() {
        GameScreen::Intro => render_intro(),
        GameScreen::MainMenu => render_main_menu(state, prices),
        GameScreen::Travel { selecting_city } => render_travel(state, prices, *selecting_city),
        GameScreen::Trade { mode } => render_trade(state, prices, mode),
        GameScreen::UseDrugs => render_use_drugs(state, prices),
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
    };

    // Apply high effect if player is high
    apply_high_effect(output, high_tier)
}
