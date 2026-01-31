//! Xodia Service Layer
//!
//! Handles session entry points, save/load coordination, and LLM integration.

use crate::games::xodia::{GameState, XodiaFlow, GameScreen, XodiaAction};
use crate::games::xodia::world::WorldState;
use crate::games::xodia::llm::{LlmClient, LlmConfig, LlmProvider, DM_SYSTEM_PROMPT};
use crate::games::xodia::render;
use super::db::{XodiaDb, LeaderboardEntry};

/// Sentinel for session routing
pub const SENTINEL: &str = "__xodia__";

/// Initialize or resume a game session
pub async fn start_game(
    db: &XodiaDb,
    user_id: i64,
    handle: &str,
    llm_config: Option<&LlmConfig>,
) -> Result<(XodiaFlow, String), String> {
    // Check maintenance mode
    if db.is_maintenance_mode().await.unwrap_or(false) {
        let mut flow = XodiaFlow::new(user_id);
        flow.set_maintenance_mode(true);
        let screen = render::render_maintenance_mode();
        return Ok((flow, screen));
    }

    // Check LLM availability if config provided
    let llm_available = if let Some(config) = llm_config {
        check_llm_health(config).await
    } else {
        // No config means offline mode
        false
    };

    // Check for existing save
    match db.load_character(user_id).await {
        Ok(Some(json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&json) {
                Ok(mut state) => {
                    state.handle = Some(handle.to_string());
                    state.update_timestamp();

                    // Load world state
                    let world = match db.load_world_state().await {
                        Ok(Some(world_json)) => {
                            serde_json::from_str(&world_json).unwrap_or_else(|_| WorldState::new())
                        }
                        _ => WorldState::new(),
                    };

                    let mut flow = XodiaFlow::from_state(user_id, state, world);
                    flow.set_llm_available(llm_available);

                    let screen = render_current_screen(&flow);
                    Ok((flow, screen))
                }
                Err(e) => {
                    // Corrupted save, start fresh
                    let mut flow = XodiaFlow::new(user_id);
                    flow.set_llm_available(llm_available);
                    let screen = render::render_intro();
                    eprintln!("Xodia: Corrupted save for user {}, starting fresh: {}", user_id, e);
                    Ok((flow, screen))
                }
            }
        }
        Ok(None) => {
            // New game
            let mut flow = XodiaFlow::new(user_id);
            flow.set_llm_available(llm_available);

            if !llm_available {
                let screen = render::render_offline_mode();
                return Ok((flow, screen));
            }

            let screen = render::render_intro();
            Ok((flow, screen))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(
    db: &XodiaDb,
    user_id: i64,
    handle: &str,
    flow: &XodiaFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    // Don't save during character creation
    if flow.is_new_game() {
        return Ok(());
    }

    let state_json = serde_json::to_string(state)
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_character(
        user_id,
        handle,
        &state_json,
        &state.character_name,
        state.class.name(),
        state.level,
        state.experience,
        state.health,
        state.max_health,
        state.mana,
        state.max_mana,
        &state.current_room_id,
        &state.current_region,
        state.gold,
    )
    .await
    .map_err(|e| format!("Save error: {}", e))?;

    // Save world state
    let world_json = serde_json::to_string(flow.world_state())
        .map_err(|e| format!("World serialize error: {}", e))?;

    db.save_world_state(&world_json)
        .await
        .map_err(|e| format!("World save error: {}", e))?;

    Ok(())
}

/// Delete save and start fresh
#[allow(dead_code)]
pub async fn clear_save(db: &XodiaDb, user_id: i64) -> Result<(), String> {
    db.delete_character(user_id)
        .await
        .map_err(|e| format!("Delete error: {}", e))
}

/// Record game completion
#[allow(dead_code)]
pub async fn record_game_completion(
    db: &XodiaDb,
    user_id: i64,
    handle: &str,
    flow: &XodiaFlow,
) -> Result<(), String> {
    let state = flow.game_state();

    db.record_completion(
        user_id,
        handle,
        &state.character_name,
        state.class.name(),
        state.level,
        state.discovered_rooms.len() as u32,
        0, // TODO: Track monsters slain
        state.main_quest_stage >= 10, // Arbitrary completion stage
        state.total_playtime_seconds,
    )
    .await
    .map_err(|e| format!("Record error: {}", e))?;

    // Delete save after completion
    let _ = db.delete_character(user_id).await;

    Ok(())
}

/// Get leaderboard for display
#[allow(dead_code)]
pub async fn get_game_leaderboard(db: &XodiaDb) -> Vec<LeaderboardEntry> {
    match db.get_leaderboard(10).await {
        Ok(entries) => entries,
        Err(_) => Vec::new(),
    }
}

/// Render current screen based on flow state
pub fn render_screen(flow: &XodiaFlow) -> String {
    render_current_screen(flow)
}

/// Internal render based on current screen
fn render_current_screen(flow: &XodiaFlow) -> String {
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::Intro => render::render_intro(),
        GameScreen::CharacterCreation { step, name } => {
            render::render_character_creation(*step, name.as_deref())
        }
        GameScreen::MainGame => {
            let room_desc = flow.world_state()
                .describe_room(&state.current_room_id)
                .unwrap_or_else(|| "You are in a mysterious void.".to_string());
            render::render_main_view(state, &room_desc)
        }
        GameScreen::Combat { combat, last_action } => {
            render::render_combat(state, combat, last_action)
        }
        GameScreen::Inventory => render::render_inventory(state),
        GameScreen::Stats => render::render_stats(state),
        GameScreen::Help => render::render_help(),
        GameScreen::ConfirmQuit => render::render_confirm_quit(),
        GameScreen::Offline => render::render_offline_mode(),
        GameScreen::Maintenance => render::render_maintenance_mode(),
    }
}

/// Check if LLM is available
async fn check_llm_health(config: &LlmConfig) -> bool {
    let client = LlmClient::new(config.clone());
    client.health_check().await
}

/// Generate LLM response for a prompt
#[allow(dead_code)]
pub async fn generate_llm_response(
    config: &LlmConfig,
    prompt: &str,
    system_prompt: Option<&str>,
) -> Result<String, String> {
    let client = LlmClient::new(config.clone());
    let response = client.generate(prompt, system_prompt.or(Some(DM_SYSTEM_PROMPT))).await;

    if response.success {
        Ok(response.content)
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

/// Log an event to the database
#[allow(dead_code)]
pub async fn log_event(
    db: &XodiaDb,
    room_id: &str,
    user_id: i64,
    actor_name: &str,
    action: &str,
    target: Option<&str>,
    outcome: &str,
) -> Result<(), String> {
    db.log_event(room_id, Some(user_id), actor_name, action, target, outcome)
        .await
        .map_err(|e| format!("Event log error: {}", e))?;
    Ok(())
}

/// Get LLM config from environment or defaults
pub fn get_llm_config() -> LlmConfig {
    // Check environment for configuration
    let provider = if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        LlmProvider::Anthropic {
            api_key,
            model: std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-haiku-20240307".to_string()),
        }
    } else if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        LlmProvider::OpenAI {
            api_key,
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
        }
    } else {
        // Default to Ollama
        LlmProvider::Ollama {
            base_url: std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string()),
        }
    };

    LlmConfig {
        provider,
        timeout_seconds: 30,
        max_tokens: 500,
        temperature: 0.7,
        retry_attempts: 2,
    }
}

/// Handle XodiaAction from flow
pub async fn handle_action(
    action: XodiaAction,
    db: &XodiaDb,
    user_id: i64,
    handle: &str,
    flow: &mut XodiaFlow,
    llm_config: Option<&LlmConfig>,
) -> Result<Option<String>, String> {
    match action {
        XodiaAction::Continue => Ok(None),
        XodiaAction::Render(output) => Ok(Some(output)),
        XodiaAction::Echo(chars) => Ok(Some(chars)),
        XodiaAction::SaveGame => {
            save_game_state(db, user_id, handle, flow).await?;
            Ok(None)
        }
        XodiaAction::NeedsLlm { prompt, system } => {
            if let Some(config) = llm_config {
                match generate_llm_response(config, &prompt, Some(&system)).await {
                    Ok(response) => {
                        flow.apply_llm_response(&response);
                        Ok(Some(render_screen(flow)))
                    }
                    Err(e) => {
                        eprintln!("LLM error: {}", e);
                        Ok(Some(render_screen(flow)))
                    }
                }
            } else {
                Ok(Some(render_screen(flow)))
            }
        }
        XodiaAction::Quit => {
            // Save before quitting
            save_game_state(db, user_id, handle, flow).await?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_db() -> XodiaDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_xodia.db");
        XodiaDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_sentinel_value() {
        assert_eq!(SENTINEL, "__xodia__");
    }

    #[tokio::test]
    async fn test_start_new_game() {
        let db = create_test_db().await;

        let (flow, screen) = start_game(&db, 1, "TestUser", None).await.unwrap();

        // Should show offline screen since no LLM config
        assert!(flow.is_new_game() || screen.contains("OFFLINE"));
    }

    #[tokio::test]
    async fn test_maintenance_mode() {
        let db = create_test_db().await;

        // Enable maintenance
        db.set_maintenance_mode(true).await.unwrap();

        let (_, screen) = start_game(&db, 1, "TestUser", None).await.unwrap();
        assert!(screen.contains("MAINTENANCE"));
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let db = create_test_db().await;

        // Create a flow with a character
        let mut flow = XodiaFlow::new(1);

        // Simulate character creation
        flow.handle_char('\r'); // Skip intro
        flow.input_buffer = "TestHero".to_string();
        flow.handle_char('\r'); // Enter name
        flow.input_buffer = "1".to_string();
        flow.handle_char('\r'); // Select warrior

        // Save
        save_game_state(&db, 1, "TestUser", &flow).await.unwrap();

        // Verify save exists
        assert!(db.has_character(1).await.unwrap());

        // Load
        let loaded_json = db.load_character(1).await.unwrap();
        assert!(loaded_json.is_some());

        let loaded: GameState = serde_json::from_str(&loaded_json.unwrap()).unwrap();
        assert_eq!(loaded.character_name, "TestHero");
    }

    #[tokio::test]
    async fn test_render_screens() {
        let flow = XodiaFlow::new(1);
        let screen = render_screen(&flow);
        assert!(screen.contains("XODIA"));
    }

    #[tokio::test]
    async fn test_get_llm_config_default() {
        // Clear env vars for test
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("OPENAI_API_KEY");

        let config = get_llm_config();
        assert!(matches!(config.provider, LlmProvider::Ollama { .. }));
    }
}
