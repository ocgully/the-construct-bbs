//! Ultimo - Door Game Service
//!
//! An Ultima Online-inspired persistent world MMO where players share a world,
//! develop skills through use, engage in PvE and PvP combat, craft items,
//! own housing, and trade with other players.
//!
//! Uses __ultimo__ sentinel for session routing.

use crate::games::ultimo::render::render_screen as game_render_screen;
use crate::games::ultimo::{GameState, UltimoFlow};
use crate::services::ultimo::db::{LeaderboardEntry, OnlinePlayer, UltimoDb, VisiblePlayer};

/// Sentinel for session routing
#[allow(dead_code)]
pub const SENTINEL: &str = "__ultimo__";

/// Initialize or resume a game session
#[allow(dead_code)]
pub async fn start_game(
    db: &UltimoDb,
    user_id: i64,
    _handle: &str,
) -> Result<(UltimoFlow, String), String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    // Load visible players in zone
                    if let Some(ref char) = state.character {
                        match db.get_players_in_zone(&char.position.zone, user_id).await {
                            Ok(players) => {
                                state.visible_players = players
                                    .into_iter()
                                    .map(|p| crate::games::ultimo::state::VisiblePlayer {
                                        name: p.name,
                                        level: p.level,
                                        x: p.x,
                                        y: p.y,
                                        guild: p.guild,
                                    })
                                    .collect();
                            }
                            Err(_) => {}
                        }
                    }

                    let flow = UltimoFlow::from_state(state);
                    let screen = game_render_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save
                    Err(format!("Save corrupted: {}. Start new game.", e))
                }
            }
        }
        Ok(None) => {
            // New game - start character creation
            let flow = UltimoFlow::new();
            let screen = game_render_screen(&flow);
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
#[allow(dead_code)]
pub async fn save_game_state(
    db: &UltimoDb,
    user_id: i64,
    handle: &str,
    flow: &UltimoFlow,
) -> Result<(), String> {
    let state = flow.game_state();
    let json =
        serde_json::to_string(state).map_err(|e| format!("Serialize error: {}", e))?;

    if let Some(ref char) = state.character {
        // Save game state
        db.save_game(user_id, handle, &char.name, char.level(), char.total_xp, &json)
            .await
            .map_err(|e| format!("Save error: {}", e))?;

        // Sync character to multiplayer tables
        let kills_total: u32 = char.kills.values().sum();
        db.sync_character(
            user_id,
            handle,
            &char.name,
            char.level(),
            char.total_xp,
            char.hp,
            char.max_hp,
            char.mana,
            char.max_mana,
            char.stamina,
            char.max_stamina,
            char.strength,
            char.dexterity,
            char.intelligence,
            char.gold,
            char.bank_gold,
            &char.position.zone,
            char.position.x,
            char.position.y,
            char.equipped_weapon.as_deref(),
            char.equipped_armor.as_deref(),
            char.equipped_shield.as_deref(),
            kills_total,
            char.deaths,
            char.pvp_kills,
            char.pvp_deaths,
            char.title.as_deref(),
            char.guild_id,
            char.partner_user_id,
            char.partner_name.as_deref(),
            char.is_dead,
        )
        .await
        .map_err(|e| format!("Sync error: {}", e))?;

        // Sync skills
        let skills: Vec<(String, u32)> = char
            .skills
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        db.sync_skills(user_id, &skills)
            .await
            .map_err(|e| format!("Skills sync error: {}", e))?;
    } else {
        // No character yet, just save state
        db.save_game(user_id, handle, "New Character", 1, 0, &json)
            .await
            .map_err(|e| format!("Save error: {}", e))?;
    }

    Ok(())
}

/// Mark player as offline when they disconnect
#[allow(dead_code)]
pub async fn player_disconnect(db: &UltimoDb, user_id: i64) -> Result<(), String> {
    db.set_offline(user_id)
        .await
        .map_err(|e| format!("Disconnect error: {}", e))
}

/// Render current screen based on game state
#[allow(dead_code)]
pub fn render_screen(flow: &UltimoFlow) -> String {
    game_render_screen(flow)
}

/// Get level leaderboard for display
#[allow(dead_code)]
pub async fn get_game_leaderboard(db: &UltimoDb) -> Vec<LeaderboardEntry> {
    match db.get_level_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Get PvP leaderboard
#[allow(dead_code)]
pub async fn get_pvp_leaderboard(db: &UltimoDb) -> Vec<LeaderboardEntry> {
    match db.get_pvp_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Get online players
#[allow(dead_code)]
pub async fn get_online_players(db: &UltimoDb) -> Vec<OnlinePlayer> {
    match db.get_online_players().await {
        Ok(players) => players,
        Err(_) => Vec::new(),
    }
}

/// Get visible players in a zone
#[allow(dead_code)]
pub async fn get_visible_players(
    db: &UltimoDb,
    zone: &str,
    exclude_user_id: i64,
) -> Vec<VisiblePlayer> {
    match db.get_players_in_zone(zone, exclude_user_id).await {
        Ok(players) => players,
        Err(_) => Vec::new(),
    }
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &UltimoDb, user_id: i64) -> Result<(), String> {
    db.delete_save(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel() {
        assert_eq!(SENTINEL, "__ultimo__");
    }
}
