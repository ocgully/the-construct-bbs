//! Master of Andromeda - Door Game Service
//!
//! A 4X space strategy game inspired by Master of Orion.
//! Uses __master_of_cygnus__ sentinel for session routing.

#![allow(dead_code)]

use crate::services::master_of_cygnus::db::MocDb;
use crate::games::master_of_cygnus::GameState;
use crate::games::master_of_cygnus::MocFlow;
use crate::games::master_of_cygnus::render::render_screen;
use crate::games::master_of_cygnus::state::GameStatus;

/// Sentinel for session routing
pub const SENTINEL: &str = "__master_of_cygnus__";

/// Initialize or resume a game session
pub async fn start_game(
    db: &MocDb,
    user_id: i64,
    _handle: &str,
) -> Result<(MocFlow, String), String> {
    let mut flow = MocFlow::new(user_id);

    // Check for existing games this user is in
    match db.get_user_games(user_id).await {
        Ok(games) => {
            if let Some(game_summary) = games.first() {
                // Load existing game
                match db.load_game(game_summary.id).await {
                    Ok(Some(json)) => {
                        match serde_json::from_str::<GameState>(&json) {
                            Ok(game) => {
                                flow.load_game(game);
                            }
                            Err(e) => {
                                return Err(format!("Game state corrupted: {}", e));
                            }
                        }
                    }
                    Ok(None) => {
                        // Game not found, start fresh
                    }
                    Err(e) => {
                        return Err(format!("Database error: {}", e));
                    }
                }
            }

            // Also load available games for join screen
            if let Ok(open_games) = db.get_open_games().await {
                flow.available_games = open_games.into_iter()
                    .map(|g| (g.id as u64, g.name, g.player_count, 4))
                    .collect();
            }
        }
        Err(e) => {
            return Err(format!("Database error: {}", e));
        }
    }

    let screen = render_screen(&flow);
    Ok((flow, screen))
}

/// Save current game state
pub async fn save_game_state(
    db: &MocDb,
    user_id: i64,
    flow: &MocFlow,
) -> Result<(), String> {
    let Some(game) = flow.game_state() else {
        return Ok(()); // No game to save
    };

    let json = serde_json::to_string(game)
        .map_err(|e| format!("Serialize error: {}", e))?;

    let status = match game.status {
        GameStatus::WaitingForPlayers => "waiting",
        GameStatus::InProgress => "in_progress",
        GameStatus::Completed => "completed",
    };

    db.save_game(
        game.id as i64,
        &game.name,
        &json,
        status,
        game.turn_number as i64,
        game.turn_deadline,
    )
    .await
    .map_err(|e| format!("Save error: {}", e))?;

    // Update player association if they have an empire
    if let Some(empire_id) = flow.current_empire_id {
        let _ = db.join_game(user_id, game.id as i64, empire_id).await;
        let _ = db.update_activity(user_id, game.id as i64).await;
    }

    Ok(())
}

/// Render current screen
pub fn render_current_screen(flow: &MocFlow) -> String {
    render_screen(flow)
}

/// Process turn timeout for games past their deadline
pub async fn process_timeout_games(db: &MocDb) -> Result<u32, String> {
    let games = db.get_games_past_deadline()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let mut processed = 0;

    for game_id in games {
        if let Ok(Some(json)) = db.load_game(game_id).await {
            if let Ok(mut game) = serde_json::from_str::<GameState>(&json) {
                // Check timeouts and convert players to AI
                game.check_timeouts();

                // If all orders now submitted (including AI), process turn
                if game.all_orders_submitted() {
                    game.process_turn();
                }

                // Save updated game
                let updated_json = serde_json::to_string(&game)
                    .map_err(|e| format!("Serialize error: {}", e))?;

                let status = match game.status {
                    GameStatus::WaitingForPlayers => "waiting",
                    GameStatus::InProgress => "in_progress",
                    GameStatus::Completed => "completed",
                };

                let _ = db.save_game(
                    game_id,
                    &game.name,
                    &updated_json,
                    status,
                    game.turn_number as i64,
                    game.turn_deadline,
                ).await;

                processed += 1;
            }
        }
    }

    Ok(processed)
}

/// Handle player forfeit
pub async fn forfeit_game(
    db: &MocDb,
    user_id: i64,
    game_id: i64,
) -> Result<(), String> {
    // Load game
    let json = db.load_game(game_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or("Game not found")?;

    let mut game: GameState = serde_json::from_str(&json)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Find and convert empire to AI
    if let Some(empire) = game.empires.iter_mut().find(|e| e.user_id == Some(user_id)) {
        empire.convert_to_ai("Player forfeited");
    }

    // Mark in player_games table
    let _ = db.forfeit_player(user_id, game_id).await;

    // Check if game should end (no human players left)
    let human_count = game.empires.iter()
        .filter(|e| !e.is_ai && !e.forfeited && !e.colonies.is_empty())
        .count();

    if human_count == 0 {
        game.status = GameStatus::Completed;
        game.victory_type = Some(crate::games::master_of_cygnus::state::VictoryType::LastHumanStanding);
    }

    // Save updated game
    let updated_json = serde_json::to_string(&game)
        .map_err(|e| format!("Serialize error: {}", e))?;

    let status = match game.status {
        GameStatus::WaitingForPlayers => "waiting",
        GameStatus::InProgress => "in_progress",
        GameStatus::Completed => "completed",
    };

    db.save_game(
        game_id,
        &game.name,
        &updated_json,
        status,
        game.turn_number as i64,
        game.turn_deadline,
    )
    .await
    .map_err(|e| format!("Save error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel() {
        assert_eq!(SENTINEL, "__master_of_cygnus__");
    }
}
