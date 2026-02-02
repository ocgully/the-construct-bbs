//! Realm of Ralnar VGA - Service Entry Points
//!
//! Handles session initialization, save/load, and framebuffer rendering.
//!
//! This version outputs raw RGBA framebuffer data (320x200x4 = 256KB)
//! that can be displayed on an HTML5 Canvas element.

use crate::games::realm_of_ralnar_vga::{
    screen::RalnarFlow,
    state::GameState,
    render::RalnarVgaRenderer,
    DISPLAY_WIDTH, DISPLAY_HEIGHT,
};
use super::db::RalnarVgaDb;

/// Sentinel for session routing
pub const SENTINEL: &str = "__realm_of_ralnar_vga__";

/// Framebuffer size in bytes (320 * 200 * 4 RGBA)
pub const FRAMEBUFFER_SIZE: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT * 4) as usize;

/// VGA game session
pub struct VgaSession {
    pub flow: RalnarFlow,
    pub renderer: RalnarVgaRenderer,
    pub user_id: i64,
    pub handle: String,
    pub dirty: bool,  // Needs re-render
}

impl VgaSession {
    /// Create new game session
    pub fn new(user_id: i64, handle: String) -> Self {
        let mut flow = RalnarFlow::new();
        let mut renderer = RalnarVgaRenderer::new();

        // Initial render
        renderer.render(&flow);

        Self {
            flow,
            renderer,
            user_id,
            handle,
            dirty: false,
        }
    }

    /// Resume from saved state
    pub fn from_state(user_id: i64, handle: String, state: GameState) -> Self {
        let mut flow = RalnarFlow::with_state(state);
        let mut renderer = RalnarVgaRenderer::new();

        renderer.render(&flow);

        Self {
            flow,
            renderer,
            user_id,
            handle,
            dirty: false,
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: &str) -> bool {
        let action = self.flow.handle_input(key);

        match action {
            crate::games::realm_of_ralnar_vga::screen::GameAction::Render |
            crate::games::realm_of_ralnar_vga::screen::GameAction::ChangeScreen(_) => {
                self.dirty = true;
                true
            }
            crate::games::realm_of_ralnar_vga::screen::GameAction::Exit => {
                // Game is exiting
                true
            }
            _ => false,
        }
    }

    /// Update and render if dirty
    pub fn update(&mut self) {
        self.flow.tick();

        if self.dirty {
            self.renderer.render(&self.flow);
            self.dirty = false;
        }
    }

    /// Get current framebuffer (320x200 RGBA)
    pub fn framebuffer(&self) -> &[u8] {
        self.renderer.framebuffer()
    }

    /// Get current game state for saving
    pub fn game_state(&self) -> &GameState {
        &self.flow.state
    }

    /// Check if on title screen (no save needed)
    pub fn is_title_screen(&self) -> bool {
        matches!(self.flow.screen, crate::games::realm_of_ralnar_vga::screen::GameScreen::Title)
    }
}

/// Initialize or resume a game session
pub async fn start_game(
    db: &RalnarVgaDb,
    user_id: i64,
    handle: &str,
) -> Result<VgaSession, String> {
    // Check for existing save
    match db.load_game(user_id).await {
        Ok(Some(state_json)) => {
            // Resume existing game
            match serde_json::from_str::<GameState>(&state_json) {
                Ok(state) => {
                    Ok(VgaSession::from_state(user_id, handle.to_string(), state))
                }
                Err(_) => {
                    // Corrupted save - start fresh
                    let _ = db.delete_save(user_id).await;
                    Ok(VgaSession::new(user_id, handle.to_string()))
                }
            }
        }
        Ok(None) => {
            // New game
            Ok(VgaSession::new(user_id, handle.to_string()))
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

/// Save current game state
pub async fn save_game_state(
    db: &RalnarVgaDb,
    session: &VgaSession,
) -> Result<(), String> {
    // Don't save if on title screen
    if session.is_title_screen() {
        return Ok(());
    }

    let state_json = serde_json::to_string(session.game_state())
        .map_err(|e| format!("Serialize error: {}", e))?;

    db.save_game(session.user_id, &session.handle, &state_json)
        .await
        .map_err(|e| format!("Save error: {}", e))?;

    Ok(())
}

/// Encode framebuffer for WebSocket transport
///
/// Options:
/// 1. Raw RGBA (256KB per frame - too large)
/// 2. PNG compressed (varies, ~10-50KB)
/// 3. Delta encoding (only changed pixels)
/// 4. Palette-indexed (64KB + palette)
pub fn encode_framebuffer_png(framebuffer: &[u8]) -> Vec<u8> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        framebuffer.to_vec(),
    ).expect("Invalid framebuffer size");

    let mut png_data = Vec::new();
    {
        use image::codecs::png::PngEncoder;
        use std::io::Cursor;
        let cursor = Cursor::new(&mut png_data);
        let encoder = PngEncoder::new(cursor);
        encoder.encode(
            img.as_raw(),
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            image::ExtendedColorType::Rgba8,
        ).expect("PNG encoding failed");
    }

    png_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = VgaSession::new(1, "TestPlayer".to_string());
        assert!(session.is_title_screen());
        assert_eq!(session.framebuffer().len(), FRAMEBUFFER_SIZE);
    }

    #[test]
    fn test_framebuffer_size() {
        assert_eq!(FRAMEBUFFER_SIZE, 320 * 200 * 4);
    }
}
