//! Morningmist - Door Game Service
//!
//! A multi-player text adventure RPG inspired by the classic Morningmist BBS game.
//! Players explore a fairy tale realm, learn magic, solve puzzles, and
//! compete to become the Arch-Mage of Legends.
//!
//! Uses __kyrandia__ sentinel for session routing.
//! Game state persisted in kyrandia.db.

#![allow(dead_code)]

use crate::services::kyrandia::db::{KyrandiaDb, LeaderboardEntry};
use crate::games::kyrandia::{GameState, KyrandiaFlow};
use crate::games::kyrandia::render;

/// Sentinel for session routing
pub const SENTINEL: &str = "__kyrandia__";

/// Initialize or resume a game session
pub async fn start_game(db: &KyrandiaDb, user_id: i64, handle: &str) -> Result<(KyrandiaFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    // Check for daily reset
                    let reset = state.check_daily_reset();
                    if reset {
                        state.last_message = Some("A new day dawns in Morningmist. Your turns have been restored.".to_string());
                    }

                    let flow = KyrandiaFlow::from_state(state);
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
            let flow = KyrandiaFlow::new(handle);
            let screen = render::render_intro(flow.game_state());
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(db: &KyrandiaDb, user_id: i64, handle: &str, flow: &KyrandiaFlow) -> Result<(), String> {
    let state = flow.game_state();

    let json = serde_json::to_string(state)
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(user_id, handle, &json, state.level, state.became_archmage)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    // Update player position for multiplayer visibility
    db.update_player_position(user_id, handle, &state.current_room)
        .await
        .ok();

    Ok(())
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &KyrandiaDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion
pub async fn record_game_completion(
    db: &KyrandiaDb,
    user_id: i64,
    handle: &str,
    flow: &KyrandiaFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_completion(
        user_id,
        handle,
        state.level,
        state.became_archmage,
        state.monsters_killed,
        state.total_gold_earned,
        state.known_spells.len(),
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // If became Arch-Mage, record in world state
    if state.became_archmage {
        db.set_current_archmage(handle)
            .await
            .ok();
    }

    // Delete save after completion
    let _ = db.delete_save(user_id).await;

    Ok(())
}

/// Get leaderboard for display
pub async fn get_game_leaderboard(db: &KyrandiaDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Get players currently in the same room
pub async fn get_players_in_room(db: &KyrandiaDb, room_key: &str, exclude_user_id: i64) -> Vec<(i64, String)> {
    match db.get_players_in_room(room_key, exclude_user_id).await {
        Ok(players) => players,
        Err(_) => Vec::new(),
    }
}

/// Send a message to another player
pub async fn send_player_message(db: &KyrandiaDb, from_user_id: i64, to_user_id: i64, message: &str) -> Result<(), String> {
    db.send_message(from_user_id, to_user_id, message)
        .await
        .map_err(|e| format!("Message error: {}", e))
}

/// Get unread messages for player
pub async fn get_unread_messages(db: &KyrandiaDb, user_id: i64) -> Vec<(String, String)> {
    match db.get_unread_messages(user_id).await {
        Ok(messages) => messages.into_iter().map(|(_, handle, msg)| (handle, msg)).collect(),
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on game state
pub fn render_screen(flow: &KyrandiaFlow) -> String {
    render::render_screen(flow)
}

/// Render leaderboard screen
pub fn render_leaderboard_screen(entries: &[LeaderboardEntry]) -> String {
    use crate::terminal::{AnsiWriter, Color};

    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("");
    w.writeln("  MORNINGMIST - HALL OF LEGENDS");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!(
        "    {:<4} {:<16} {:>5} {:>8} {:>10} {}",
        "Rank", "Name", "Level", "Monsters", "Gold", "Status"
    ));
    w.writeln(&format!("    {}", "\u{2500}".repeat(60)));
    w.reset_color();

    if entries.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("    No champions yet. Will you be the first?");
    } else {
        for entry in entries {
            let rank_color = match entry.rank {
                1 => Color::Yellow,
                2 => Color::White,
                3 => Color::Brown,
                _ => Color::LightGray,
            };

            w.set_fg(rank_color);
            w.write_str(&format!("    {:<4} ", entry.rank));
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("{:<16}", entry.handle));
            w.set_fg(Color::White);
            w.write_str(&format!("{:>5}", entry.level));
            w.set_fg(Color::LightGray);
            w.write_str(&format!("{:>8}", entry.monsters_killed));
            w.set_fg(Color::Yellow);
            w.write_str(&format!("{:>10}", entry.gold_earned));

            if entry.became_archmage {
                w.set_fg(Color::LightMagenta);
                w.write_str(" ARCH-MAGE");
            }
            w.writeln("");
        }
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}
