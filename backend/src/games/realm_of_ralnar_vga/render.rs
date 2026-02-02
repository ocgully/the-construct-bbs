//! VGA renderer for Realm of Ralnar
//!
//! Renders the game to a 320x200 framebuffer that can be sent to the client.

use super::palette::VGA_PALETTE;
use super::screen::{GameScreen, MenuScreen, RalnarFlow};
use super::{DISPLAY_HEIGHT, DISPLAY_WIDTH, TILE_SIZE, VIEW_TILES_X, VIEW_TILES_Y};

/// RGBA framebuffer (320x200 * 4 bytes)
pub type Framebuffer = Vec<u8>;

/// VGA Mode 13h renderer
pub struct RalnarVgaRenderer {
    /// 320x200 RGBA framebuffer
    framebuffer: Framebuffer,
    /// Loaded tile images (would be actual texture data in full impl)
    tiles_loaded: bool,
}

impl RalnarVgaRenderer {
    pub fn new() -> Self {
        Self {
            framebuffer: vec![0u8; (DISPLAY_WIDTH * DISPLAY_HEIGHT * 4) as usize],
            tiles_loaded: false,
        }
    }

    /// Get the framebuffer for sending to client
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Clear framebuffer to a solid color
    pub fn clear(&mut self, palette_index: u8) {
        let (r, g, b) = VGA_PALETTE[palette_index as usize];
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                self.put_pixel(x, y, r, g, b, 255);
            }
        }
    }

    /// Set a single pixel
    #[inline]
    pub fn put_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        if x < DISPLAY_WIDTH && y < DISPLAY_HEIGHT {
            let idx = ((y * DISPLAY_WIDTH + x) * 4) as usize;
            self.framebuffer[idx] = r;
            self.framebuffer[idx + 1] = g;
            self.framebuffer[idx + 2] = b;
            self.framebuffer[idx + 3] = a;
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, palette_index: u8) {
        let (r, g, b) = VGA_PALETTE[palette_index as usize];
        for py in y..y.saturating_add(h).min(DISPLAY_HEIGHT) {
            for px in x..x.saturating_add(w).min(DISPLAY_WIDTH) {
                self.put_pixel(px, py, r, g, b, 255);
            }
        }
    }

    /// Draw a border rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, palette_index: u8) {
        let (r, g, b) = VGA_PALETTE[palette_index as usize];

        // Top and bottom
        for px in x..x.saturating_add(w).min(DISPLAY_WIDTH) {
            self.put_pixel(px, y, r, g, b, 255);
            self.put_pixel(px, y + h - 1, r, g, b, 255);
        }
        // Left and right
        for py in y..y.saturating_add(h).min(DISPLAY_HEIGHT) {
            self.put_pixel(x, py, r, g, b, 255);
            self.put_pixel(x + w - 1, py, r, g, b, 255);
        }
    }

    /// Draw text using built-in 8x8 bitmap font (placeholder)
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, palette_index: u8) {
        let (r, g, b) = VGA_PALETTE[palette_index as usize];

        // Simple placeholder text rendering (8x8 characters)
        for (i, _ch) in text.chars().enumerate() {
            let cx = x + (i as u32) * 8;
            if cx + 8 > DISPLAY_WIDTH {
                break;
            }
            // Draw a simple rectangle as placeholder for each character
            for py in y..y + 7 {
                for px in cx..cx + 6 {
                    if py < DISPLAY_HEIGHT && px < DISPLAY_WIDTH {
                        self.put_pixel(px, py, r, g, b, 255);
                    }
                }
            }
        }
    }

    /// Render the current game screen
    pub fn render(&mut self, flow: &RalnarFlow) {
        match &flow.screen {
            GameScreen::Title => self.render_title(flow),
            GameScreen::NewGame => self.render_new_game(flow),
            GameScreen::Explore => self.render_explore(flow),
            GameScreen::Menu(menu) => self.render_menu(flow, menu),
            GameScreen::Dialogue { npc_id, .. } => self.render_dialogue(flow, npc_id),
            GameScreen::Battle(battle) => self.render_battle(flow, battle),
            GameScreen::Shop { shop_id } => self.render_shop(flow, shop_id),
            GameScreen::GameOver => self.render_game_over(flow),
            _ => self.clear(0),
        }
    }

    fn render_title(&mut self, flow: &RalnarFlow) {
        // Dark blue background
        self.clear(1);

        // Title text (centered)
        self.draw_text(80, 60, "REALM OF RALNAR", 15);
        self.draw_text(100, 80, "VGA EDITION", 14);

        // Blinking "Press Enter" text
        if (flow.anim_frame / 30) % 2 == 0 {
            self.draw_text(96, 150, "PRESS ENTER", 7);
        }

        // Border
        self.draw_rect(10, 10, 300, 180, 15);
    }

    fn render_new_game(&mut self, flow: &RalnarFlow) {
        self.clear(0);
        self.draw_text(80, 80, "ENTER YOUR NAME:", 15);
        self.draw_text(100, 100, &flow.state.hero().name, 14);
        self.draw_text(80, 140, "PRESS ENTER TO START", 7);
    }

    fn render_explore(&mut self, flow: &RalnarFlow) {
        // Draw sky/ground background
        self.clear(2); // Green for now (should be map tiles)

        // Draw map tiles (placeholder grid)
        for ty in 0..VIEW_TILES_Y {
            for tx in 0..VIEW_TILES_X {
                let screen_x = tx * TILE_SIZE;
                let screen_y = ty * TILE_SIZE;

                // Alternate colors for visibility
                let color = if (tx + ty) % 2 == 0 { 2 } else { 10 };
                self.fill_rect(screen_x, screen_y, TILE_SIZE, TILE_SIZE, color);
                self.draw_rect(screen_x, screen_y, TILE_SIZE, TILE_SIZE, 0);
            }
        }

        // Draw player at center
        let player_x = (VIEW_TILES_X / 2) * TILE_SIZE;
        let player_y = (VIEW_TILES_Y / 2) * TILE_SIZE;
        self.fill_rect(player_x + 4, player_y + 2, 12, 16, 4); // Red body
        self.fill_rect(player_x + 6, player_y + 2, 8, 8, 6); // Brown head

        // Draw position indicator
        let pos_text = format!(
            "X:{} Y:{}",
            flow.state.position.x, flow.state.position.y
        );
        self.fill_rect(0, 0, 80, 10, 0);
        self.draw_text(2, 2, &pos_text, 15);
    }

    fn render_menu(&mut self, flow: &RalnarFlow, menu: &MenuScreen) {
        // Semi-transparent overlay (just darken for now)
        self.render_explore(flow);

        // Menu window
        self.fill_rect(40, 30, 240, 140, 1);
        self.draw_rect(40, 30, 240, 140, 15);
        self.draw_rect(42, 32, 236, 136, 7);

        match menu {
            MenuScreen::Main => {
                self.draw_text(140, 40, "MENU", 14);

                let options = ["Items", "Equip", "Status", "Magic", "Save", "Exit"];
                for (i, opt) in options.iter().enumerate() {
                    let y = 60 + (i as u32) * 15;
                    let color = if i == flow.menu_cursor { 14 } else { 15 };
                    if i == flow.menu_cursor {
                        self.draw_text(60, y, ">", 14);
                    }
                    self.draw_text(75, y, opt, color);
                }
            }
            MenuScreen::Status => {
                let hero = flow.state.hero();
                self.draw_text(140, 40, "STATUS", 14);
                self.draw_text(60, 60, &format!("Name: {}", hero.name), 15);
                self.draw_text(60, 75, &format!("Level: {}", hero.level), 15);
                self.draw_text(60, 90, &format!("HP: {}/{}", hero.current_hp, hero.max_hp), 15);
                self.draw_text(60, 105, &format!("MP: {}/{}", hero.current_mp, hero.max_mp), 15);
                self.draw_text(60, 120, &format!("STR: {}  DEF: {}", hero.strength, hero.defense), 15);
                self.draw_text(60, 135, &format!("AGI: {}  MAG: {}", hero.agility, hero.magic), 15);
            }
            _ => {
                self.draw_text(100, 80, "NOT IMPLEMENTED", 12);
            }
        }
    }

    fn render_dialogue(&mut self, _flow: &RalnarFlow, npc_id: &str) {
        self.clear(0);
        self.fill_rect(20, 140, 280, 50, 1);
        self.draw_rect(20, 140, 280, 50, 15);
        self.draw_text(30, 150, &format!("[{}]", npc_id), 14);
        self.draw_text(30, 165, "Hello, adventurer!", 15);
    }

    fn render_battle(&mut self, flow: &RalnarFlow, battle: &super::screen::BattleState) {
        // Battle background
        self.clear(8); // Dark gray

        // Enemy area (top)
        self.fill_rect(110, 20, 100, 80, 0);
        self.draw_rect(110, 20, 100, 80, 15);
        self.draw_text(130, 50, &battle.enemy_id, 12);

        // Enemy HP bar
        let hp_width = (battle.enemy_hp as u32 * 80) / battle.enemy_max_hp.max(1) as u32;
        self.fill_rect(120, 90, 80, 8, 4);
        self.fill_rect(120, 90, hp_width, 8, 2);

        // Command window
        self.fill_rect(20, 130, 100, 60, 1);
        self.draw_rect(20, 130, 100, 60, 15);

        let commands = ["Attack", "Magic", "Item", "Run"];
        for (i, cmd) in commands.iter().enumerate() {
            let y = 140 + (i as u32) * 12;
            let color = if i == flow.menu_cursor { 14 } else { 15 };
            if i == flow.menu_cursor {
                self.draw_text(28, y, ">", 14);
            }
            self.draw_text(40, y, cmd, color);
        }

        // Player stats
        self.fill_rect(200, 130, 100, 60, 1);
        self.draw_rect(200, 130, 100, 60, 15);
        let hero = flow.state.hero();
        self.draw_text(210, 140, &hero.name, 15);
        self.draw_text(210, 155, &format!("HP:{}", hero.current_hp), 15);
        self.draw_text(210, 170, &format!("MP:{}", hero.current_mp), 15);
    }

    fn render_shop(&mut self, _flow: &RalnarFlow, shop_id: &str) {
        self.clear(6); // Brown background
        self.fill_rect(40, 40, 240, 120, 1);
        self.draw_rect(40, 40, 240, 120, 15);
        self.draw_text(130, 50, "SHOP", 14);
        self.draw_text(60, 70, &format!("Welcome to {}!", shop_id), 15);
        self.draw_text(60, 140, "Press ESC to leave", 7);
    }

    fn render_game_over(&mut self, _flow: &RalnarFlow) {
        self.clear(4); // Red
        self.draw_text(120, 90, "GAME OVER", 15);
        self.draw_text(80, 130, "Press any key to continue", 7);
    }

    /// Apply scanline effect (darken every other line by 30%)
    pub fn apply_scanlines(&mut self) {
        for y in (1..DISPLAY_HEIGHT).step_by(2) {
            for x in 0..DISPLAY_WIDTH {
                let idx = ((y * DISPLAY_WIDTH + x) * 4) as usize;
                self.framebuffer[idx] = (self.framebuffer[idx] as u32 * 70 / 100) as u8;
                self.framebuffer[idx + 1] = (self.framebuffer[idx + 1] as u32 * 70 / 100) as u8;
                self.framebuffer[idx + 2] = (self.framebuffer[idx + 2] as u32 * 70 / 100) as u8;
            }
        }
    }
}

impl Default for RalnarVgaRenderer {
    fn default() -> Self {
        Self::new()
    }
}
