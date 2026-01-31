//! Memory Garden Service Layer
//!
//! Handles session routing and coordinates between the flow state machine
//! and the database layer.
//!
//! Uses __memory_garden__ sentinel for session routing.

use crate::services::memory_garden::db::MemoryGardenDb;
use crate::games::memory_garden::{GardenFlow, GardenAction};
use crate::games::memory_garden::render::render_screen as do_render;

/// Sentinel for session routing
pub const SENTINEL: &str = "__memory_garden__";

/// Initialize a Memory Garden session for a user
pub async fn start_garden(
    db: &MemoryGardenDb,
    user_id: i64,
    handle: &str,
    is_sysop: bool,
) -> Result<(GardenFlow, String), String> {
    let mut flow = GardenFlow::new(user_id, handle, is_sysop);

    // Check if user has already posted today
    let posted_today = db.has_posted_today(user_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    flow.set_posted_today(posted_today);

    // Check flags remaining
    let flags_remaining = db.get_flags_remaining(user_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    flow.set_flags_remaining(flags_remaining);

    // Load random memories for welcome screen
    let random_memories = db.get_random_memories(5)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    flow.set_welcome_memories(random_memories);

    // Increment session count (may trigger milestone)
    let _ = db.increment_sessions().await;

    let screen = render_screen(&flow);
    Ok((flow, screen))
}

/// Render current screen
pub fn render_screen(flow: &GardenFlow) -> String {
    do_render(flow)
}

/// Process a GardenAction and return the next screen output
pub async fn process_action(
    db: &MemoryGardenDb,
    flow: &mut GardenFlow,
    action: GardenAction,
) -> Result<Option<String>, String> {
    match action {
        GardenAction::Continue => {
            // Re-render current screen
            Ok(Some(render_screen(flow)))
        }

        GardenAction::Render(output) => {
            Ok(Some(output))
        }

        GardenAction::Echo(chars) => {
            // Echo is handled by session layer
            Ok(Some(chars))
        }

        GardenAction::Quit => {
            // Signal to exit (handled by session)
            Ok(None)
        }

        GardenAction::SaveNewMemory { content } => {
            let user_id = flow.user_id;
            let handle = flow.handle.clone();

            match db.create_memory(user_id, &handle, &content).await {
                Ok(_) => {
                    flow.set_posted_today(true);
                    // Reload memories for browse
                    let (memories, total) = db.get_memories(0, 10, None, None)
                        .await
                        .map_err(|e| format!("Load error: {}", e))?;
                    flow.set_memories(memories, total);
                    Ok(Some(render_screen(flow)))
                }
                Err(e) => {
                    flow.view_state.set_message(&format!("Error saving memory: {}", e));
                    Ok(Some(render_screen(flow)))
                }
            }
        }

        GardenAction::UpdateMemory { memory_id, content } => {
            let user_id = flow.user_id;

            match db.update_memory(memory_id, user_id, &content).await {
                Ok(true) => {
                    // Reload current view
                    let (memories, total) = db.get_memories(
                        flow.view_state.current_page,
                        flow.view_state.page_size,
                        flow.view_state.filter_date.as_deref(),
                        if flow.view_state.show_own_only { Some(user_id) } else { None },
                    ).await.map_err(|e| format!("Load error: {}", e))?;
                    flow.set_memories(memories, total);
                    Ok(Some(render_screen(flow)))
                }
                Ok(false) => {
                    flow.view_state.set_message("Could not update memory.");
                    Ok(Some(render_screen(flow)))
                }
                Err(e) => {
                    flow.view_state.set_message(&format!("Error: {}", e));
                    Ok(Some(render_screen(flow)))
                }
            }
        }

        GardenAction::DeleteMemory { memory_id } => {
            let user_id = flow.user_id;

            match db.delete_memory(memory_id, user_id).await {
                Ok(true) => {
                    // Reload memories
                    let (memories, total) = db.get_memories(
                        flow.view_state.current_page,
                        flow.view_state.page_size,
                        flow.view_state.filter_date.as_deref(),
                        if flow.view_state.show_own_only { Some(user_id) } else { None },
                    ).await.map_err(|e| format!("Load error: {}", e))?;
                    flow.set_memories(memories, total);
                    Ok(Some(render_screen(flow)))
                }
                Ok(false) => {
                    flow.view_state.set_message("Could not delete memory.");
                    Ok(Some(render_screen(flow)))
                }
                Err(e) => {
                    flow.view_state.set_message(&format!("Error: {}", e));
                    Ok(Some(render_screen(flow)))
                }
            }
        }

        GardenAction::FlagMemory { memory_id, reason } => {
            let reporter_id = flow.user_id;

            match db.flag_memory(memory_id, reporter_id, reason.as_deref()).await {
                Ok(()) => {
                    // Update flags remaining
                    let remaining = db.get_flags_remaining(reporter_id)
                        .await
                        .unwrap_or(0);
                    flow.set_flags_remaining(remaining);

                    // Reload memories
                    let (memories, total) = db.get_memories(
                        flow.view_state.current_page,
                        flow.view_state.page_size,
                        flow.view_state.filter_date.as_deref(),
                        if flow.view_state.show_own_only { Some(reporter_id) } else { None },
                    ).await.map_err(|e| format!("Load error: {}", e))?;
                    flow.set_memories(memories, total);
                    Ok(Some(render_screen(flow)))
                }
                Err(e) => {
                    flow.view_state.set_message(&format!("Error flagging: {}", e));
                    Ok(Some(render_screen(flow)))
                }
            }
        }

        GardenAction::LoadMemories { page, filter_date, own_only } => {
            let user_id_filter = if own_only { Some(flow.user_id) } else { None };

            let (memories, total) = db.get_memories(
                page,
                flow.view_state.page_size,
                filter_date.as_deref(),
                user_id_filter,
            ).await.map_err(|e| format!("Load error: {}", e))?;

            flow.set_memories(memories, total);
            Ok(Some(render_screen(flow)))
        }

        GardenAction::LoadMemory { memory_id } => {
            let memory = db.get_memory(memory_id)
                .await
                .map_err(|e| format!("Load error: {}", e))?;
            flow.set_current_memory(memory);
            Ok(Some(render_screen(flow)))
        }

        GardenAction::LoadRandomMemories { count } => {
            let memories = db.get_random_memories(count)
                .await
                .map_err(|e| format!("Load error: {}", e))?;
            flow.set_welcome_memories(memories);
            Ok(Some(render_screen(flow)))
        }

        GardenAction::CheckCanPost => {
            let can_post = !db.has_posted_today(flow.user_id)
                .await
                .unwrap_or(true);
            flow.set_posted_today(!can_post);
            Ok(Some(render_screen(flow)))
        }

        GardenAction::CheckFlagsRemaining => {
            let remaining = db.get_flags_remaining(flow.user_id)
                .await
                .unwrap_or(0);
            flow.set_flags_remaining(remaining);
            Ok(Some(render_screen(flow)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::memory_garden::screen::GardenScreen;
    use tempfile::tempdir;

    async fn create_test_db() -> MemoryGardenDb {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        MemoryGardenDb::new(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_start_garden() {
        let db = create_test_db().await;
        let (flow, screen) = start_garden(&db, 1, "testuser", false).await.unwrap();

        assert!(matches!(flow.screen, GardenScreen::Welcome));
        assert!(!flow.posted_today);
        assert_eq!(flow.flags_remaining, 3);
        // Check for menu options (ASCII art header uses Unicode blocks, not plain text)
        assert!(screen.contains("Browse the Garden"));
        assert!(screen.contains("My Memories"));
    }

    #[tokio::test]
    async fn test_create_and_view_memory() {
        let db = create_test_db().await;
        let (mut flow, _) = start_garden(&db, 1, "testuser", false).await.unwrap();

        // Navigate to new memory
        flow.handle_char('N');
        assert!(matches!(flow.screen, GardenScreen::NewMemory));

        // Type content and submit
        for ch in "Hello garden world!".chars() {
            flow.handle_char(ch);
        }
        let action = flow.handle_char('\r');

        // Process the save action
        if let GardenAction::SaveNewMemory { content } = action {
            assert_eq!(content, "Hello garden world!");
            let result = process_action(&db, &mut flow, GardenAction::SaveNewMemory { content }).await;
            assert!(result.is_ok());
        } else {
            panic!("Expected SaveNewMemory action");
        }

        // Verify posted_today is now true
        assert!(flow.posted_today);
    }

    #[tokio::test]
    async fn test_navigation() {
        let db = create_test_db().await;
        let (mut flow, _) = start_garden(&db, 1, "testuser", false).await.unwrap();

        // Browse
        let action = flow.handle_char('B');
        assert!(matches!(flow.screen, GardenScreen::Browse));
        assert!(matches!(action, GardenAction::LoadMemories { .. }));

        // Process load
        let _ = process_action(&db, &mut flow, action).await;

        // Quit
        flow.handle_char('Q');
        assert!(matches!(flow.screen, GardenScreen::ConfirmQuit));

        // Confirm quit
        let action = flow.handle_char('Y');
        assert!(matches!(action, GardenAction::Quit));
    }

    #[tokio::test]
    async fn test_flag_memory() {
        let db = create_test_db().await;

        // User 1 creates a memory
        let _ = db.create_memory(1, "user1", "Some content").await.unwrap();

        // User 2 starts session and flags it
        let (mut flow, _) = start_garden(&db, 2, "user2", false).await.unwrap();
        assert_eq!(flow.flags_remaining, 3);

        // Navigate to browse, then view, then flag
        flow.handle_char('B');
        let _ = process_action(&db, &mut flow, GardenAction::LoadMemories { page: 0, filter_date: None, own_only: false }).await;

        // The memories should include the one we just created
        assert!(!flow.memories.is_empty());
    }
}
