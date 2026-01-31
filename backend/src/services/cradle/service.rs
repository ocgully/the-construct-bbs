//! Cradle - Door Game Service
//!
//! An infinite progression RPG where players cultivate through tiers
//! from Unsouled to Transcendent, combining Sacred Arts paths and
//! techniques while managing prestige/ascension mechanics.
//!
//! Uses __cradle__ sentinel for session routing.
//! Game state persisted in game's own database (cradle.db).

use crate::services::cradle::db::{CradleDb, LeaderboardEntry};
use crate::games::cradle::{GameState, CradleFlow};
use crate::games::cradle::render;

/// Sentinel for session routing
pub const SENTINEL: &str = "__cradle__";

/// Initialize or resume a game session
pub async fn start_game(db: &CradleDb, user_id: i64, _handle: &str) -> Result<(CradleFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(state) => {
                    let flow = CradleFlow::from_state(state);
                    let screen = render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save
                    Err(format!("Save corrupted: {}. Start new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game
            let flow = CradleFlow::new();
            let screen = render::render_intro(&GameState::new(String::new()));
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &CradleDb, user_id: i64, handle: &str, flow: &CradleFlow) -> Result<(), String> {
    let json = serde_json::to_string(flow.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    // Also update prestige progress
    let state = flow.game_state();
    let _ = db.update_prestige_progress(
        user_id,
        state.prestige.ascension_points as i64,
        state.prestige.total_prestiges as i64,
        state.prestige.highest_tier_ever as u8 as i64,
        state.prestige.madra_multiplier,
        state.prestige.insight_multiplier,
        state.prestige.spirit_stone_multiplier,
        state.prestige.unlock_speed_bonus,
        state.prestige.starting_tier_bonus as i64,
    ).await;

    Ok(())
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &CradleDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion (reaching Transcendent or voluntary ascension)
pub async fn record_game_completion(
    db: &CradleDb,
    user_id: i64,
    handle: &str,
    flow: &CradleFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_completion(
        user_id,
        handle,
        state.tier as u8,
        state.prestige.ascension_points as i64,
        state.total_ticks as i64,
        state.prestige.total_prestiges as i64,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &CradleDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
pub fn render_screen(flow: &CradleFlow) -> String {
    render::render_screen(flow)
}

/// Render leaderboard with fetched entries
#[allow(dead_code)]
pub fn render_leaderboard_screen(entries: &[LeaderboardEntry]) -> String {
    use crate::terminal::{AnsiWriter, Color};
    use crate::games::cradle::data::TierLevel;
    use crate::games::cradle::format_power;

    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("");
    w.writeln("  HALL OF TRANSCENDENCE");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("    {:<4} {:<15} {:>12} {:>12} {:>10}",
        "Rank", "Cultivator", "Tier", "Points", "Ascensions"));
    w.writeln(&format!("    {}", "─".repeat(60)));
    w.reset_color();

    if entries.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("    No transcendent beings yet. Will you be the first?");
    } else {
        for entry in entries {
            let tier_name = TierLevel::from_u8(entry.final_tier)
                .map(|t| t.name())
                .unwrap_or("???");

            let rank_color = match entry.rank {
                1 => Color::Yellow,
                2 => Color::White,
                3 => Color::Brown,
                _ => Color::LightGray,
            };

            w.set_fg(rank_color);
            w.write_str(&format!("    {:<4} ", entry.rank));
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("{:<15} ", entry.handle));
            w.set_fg(Color::LightMagenta);
            w.write_str(&format!("{:>12} ", tier_name));
            w.set_fg(Color::Yellow);
            w.write_str(&format!("{:>12} ", format_power(entry.ascension_points)));
            w.set_fg(Color::LightGray);
            w.writeln(&format!("{:>10}", entry.prestige_count));
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}
