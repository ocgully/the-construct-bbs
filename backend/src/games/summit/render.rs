//! ANSI rendering for Summit
//!
//! Creates the visual presentation with mountain/alpine theme.
//! Uses natural colors: greens, browns, white for snow, orange for lava.

use crate::terminal::{AnsiWriter, Color};

use super::data::{BiomeType, BADGES};
use super::state::{ClimberState, RunState, PlayerStats, ClimberStatus, RunStatus};
use super::screen::{GameScreen, LobbyScreen, SummitFlow};
use super::mountain::{Mountain, TileType};
use super::lobby::{SummitLobby, GameLobby, ActiveGame};

// ============================================================================
// CONSTANTS
// ============================================================================

const VIEWPORT_WIDTH: usize = 60;
const VIEWPORT_HEIGHT: usize = 15;

// ============================================================================
// COLOR PALETTE (Alpine/Mountain theme)
// ============================================================================

fn biome_color(biome: BiomeType) -> Color {
    match biome {
        BiomeType::Beach => Color::Yellow,
        BiomeType::Jungle => Color::Green,
        BiomeType::Alpine => Color::White,
        BiomeType::Volcanic => Color::LightRed,
    }
}

fn tile_color(tile_type: TileType, biome: BiomeType) -> Color {
    match tile_type {
        TileType::Air => Color::DarkGray,
        TileType::Rock => Color::LightGray,
        TileType::Climbable => biome_color(biome),
        TileType::Ledge => Color::Brown,
        TileType::Campfire => Color::LightRed,
        TileType::Luggage => Color::LightCyan,
        TileType::SecretArea => Color::LightMagenta,
        TileType::Sand => Color::Yellow,
        TileType::Palm => Color::Green,
        TileType::Vine => Color::LightGreen,
        TileType::Waterfall => Color::LightCyan,
        TileType::Snow => Color::White,
        TileType::Ice => Color::LightCyan,
        TileType::Lava => Color::LightRed,
        TileType::AshCloud => Color::DarkGray,
    }
}

// ============================================================================
// HEADER
// ============================================================================

fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::White);
    w.bold();
    w.writeln("");
    w.writeln("   ███████╗██╗   ██╗███╗   ███╗███╗   ███╗██╗████████╗");
    w.writeln("   ██╔════╝██║   ██║████╗ ████║████╗ ████║██║╚══██╔══╝");
    w.writeln("   ███████╗██║   ██║██╔████╔██║██╔████╔██║██║   ██║   ");
    w.writeln("   ╚════██║██║   ██║██║╚██╔╝██║██║╚██╔╝██║██║   ██║   ");
    w.writeln("   ███████║╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║   ██║   ");
    w.writeln("   ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝   ╚═╝   ");
    w.reset_color();
    w.set_fg(Color::LightCyan);
    w.writeln("              Cooperative Mountain Climbing");
    w.reset_color();
}

fn render_mini_header(w: &mut AnsiWriter, biome: BiomeType, elapsed: &str) {
    w.clear_screen();
    w.set_fg(Color::White);
    w.bold();
    w.write_str("  SUMMIT");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(biome_color(biome));
    w.write_str(biome.name());
    w.write_str(" Biome");
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightCyan);
    w.write_str("Time: ");
    w.write_str(elapsed);
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(78));
    w.reset_color();
}

// ============================================================================
// LOBBY SCREENS
// ============================================================================

pub fn render_lobby_main_menu(stats: &PlayerStats) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Scale the mountain. Help your friends. Reach the Summit.");
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  Summits: {}  |  Best Time: {}  |  Badges: {}/{}",
        stats.total_summits,
        stats.fastest_summit_seconds
            .map(|s| format!("{}:{:02}", s / 60, s % 60))
            .unwrap_or_else(|| "--:--".to_string()),
        stats.badges.len(),
        BADGES.len()
    ));
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Today's Mountain awaits...");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [1] ");
    w.set_fg(Color::White);
    w.writeln("Create Game");

    w.set_fg(Color::LightCyan);
    w.write_str("    [2] ");
    w.set_fg(Color::White);
    w.writeln("Join Game");

    w.set_fg(Color::LightCyan);
    w.write_str("    [3] ");
    w.set_fg(Color::White);
    w.writeln("Stats & Badges");

    w.set_fg(Color::LightCyan);
    w.write_str("    [4] ");
    w.set_fg(Color::White);
    w.writeln("Customize Scout");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("    [Q] ");
    w.set_fg(Color::LightGray);
    w.writeln("Quit");
    w.reset_color();

    w.flush()
}

pub fn render_create_game() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Create New Expedition");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [1] ");
    w.set_fg(Color::White);
    w.writeln("Public Game - Anyone can join");

    w.set_fg(Color::LightCyan);
    w.write_str("    [2] ");
    w.set_fg(Color::White);
    w.writeln("Friends Only - Share invite code");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("    [B] ");
    w.set_fg(Color::LightGray);
    w.writeln("Back");
    w.reset_color();

    w.flush()
}

pub fn render_join_game(lobbies: &[(u64, usize, usize)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Join Expedition");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [1] ");
    w.set_fg(Color::White);
    w.writeln("Quick Join - Join first available");

    w.set_fg(Color::LightCyan);
    w.write_str("    [2] ");
    w.set_fg(Color::White);
    w.writeln("Enter Invite Code");

    w.writeln("");

    if lobbies.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No public games available.");
    } else {
        w.set_fg(Color::LightGray);
        w.writeln("  Available Games:");
        for (id, players, max) in lobbies.iter().take(5) {
            w.set_fg(Color::DarkGray);
            w.write_str("    ");
            w.set_fg(Color::White);
            w.writeln(&format!("Game #{} - {}/{} scouts", id, players, max));
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("    [B] ");
    w.set_fg(Color::LightGray);
    w.writeln("Back");
    w.reset_color();

    w.flush()
}

pub fn render_enter_invite_code(buffer: &str) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Enter Invite Code");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.write_str("  Code: ");
    w.set_fg(Color::LightCyan);
    w.write_str(buffer);
    w.set_fg(Color::DarkGray);
    w.writeln(&"_".repeat(6 - buffer.len().min(6)));

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Enter the 6-character code shared by your friend.");
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.write_str("    [B] ");
    w.set_fg(Color::LightGray);
    w.writeln("Back");
    w.reset_color();

    w.flush()
}

pub fn render_waiting_room(lobby: &GameLobby, user_id: i64) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Expedition Staging Area");
    w.reset_color();

    // Show invite code for private games
    if let Some(code) = lobby.get_invite_code() {
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  Invite Code: {}", code));
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Scouts:");

    for player in &lobby.players {
        let is_host = player.user_id == lobby.host_user_id;
        let is_you = player.user_id == user_id;

        w.write_str("    ");

        if player.is_ready {
            w.set_fg(Color::LightGreen);
            w.write_str("[READY] ");
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str("[....] ");
        }

        w.set_fg(Color::White);
        w.write_str(&player.handle);

        if is_host {
            w.set_fg(Color::Yellow);
            w.write_str(" (Host)");
        }
        if is_you {
            w.set_fg(Color::LightCyan);
            w.write_str(" <-- You");
        }

        w.writeln("");
    }

    // Empty slots
    for _ in lobby.players.len()..4 {
        w.set_fg(Color::DarkGray);
        w.writeln("    [....] Waiting for scout...");
    }

    w.writeln("");

    let is_host = user_id == lobby.host_user_id;
    let my_ready = lobby.players.iter()
        .find(|p| p.user_id == user_id)
        .map(|p| p.is_ready)
        .unwrap_or(false);

    w.set_fg(Color::LightCyan);
    w.write_str("    [R] ");
    w.set_fg(Color::White);
    if my_ready {
        w.writeln("Not Ready");
    } else {
        w.writeln("Ready Up!");
    }

    if is_host && lobby.can_start() {
        w.set_fg(Color::LightGreen);
        w.write_str("    [S] ");
        w.set_fg(Color::White);
        w.writeln("Start Expedition!");
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("    [L] ");
    w.set_fg(Color::LightGray);
    w.writeln("Leave");
    w.reset_color();

    w.flush()
}

pub fn render_countdown() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("              Expedition starting...");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("              Prepare for crash landing!");
    w.reset_color();

    w.flush()
}

// ============================================================================
// GAME SCREENS
// ============================================================================

pub fn render_crash_landing() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::LightRed);
    w.writeln("");
    w.writeln("  *CRASH*");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  The plane goes down on the beach of a mysterious island...");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  As the smoke clears, you see a massive mountain rising");
    w.writeln("  from the center of the island. Its peak disappears into");
    w.writeln("  the clouds.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Your only hope of rescue is to reach the Summit!");
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Scavenge supplies from the wreckage and begin your climb.");
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Press any key to begin...");
    w.reset_color();

    w.flush()
}

pub fn render_climbing(game: &ActiveGame, user_id: i64) -> String {
    let mut w = AnsiWriter::new();

    let climber = match game.run.get_climber(user_id) {
        Some(c) => c,
        None => return render_game_over(&game.run),
    };

    let biome = climber.current_biome();
    render_mini_header(&mut w, biome, &game.run.elapsed_display());

    // Render viewport
    render_viewport(&mut w, &game.mountain, &game.run, climber);

    // Status bars
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(78));

    // Your status
    render_climber_status(&mut w, climber, true);

    // Teammates status (compact)
    for (id, teammate) in game.run.climbers.iter() {
        if *id != user_id {
            w.set_fg(Color::DarkGray);
            w.write_str("  | ");
            render_climber_status_compact(&mut w, teammate);
        }
    }
    w.writeln("");

    // Controls
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(78));
    w.set_fg(Color::LightGray);
    w.writeln("  [WASD] Move  [SPACE] Grab/Rest  [R] Rope  [P] Piton  [I] Inventory  [E] Eat");
    w.set_fg(Color::DarkGray);
    w.writeln("  [H] Help teammate  [Q] Quit");
    w.reset_color();

    w.flush()
}

fn render_viewport(w: &mut AnsiWriter, mountain: &Mountain, run: &RunState, focus: &ClimberState) {
    let viewport = mountain.get_viewport(focus.x, focus.y, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
    let biome = focus.current_biome();

    let half_w = VIEWPORT_WIDTH as i32 / 2;
    let half_h = VIEWPORT_HEIGHT as i32 / 2;
    let start_x = focus.x - half_w;
    let start_y = focus.y - half_h;

    // Render from top to bottom (higher y = lower on screen for mountain feel)
    for (dy, row) in viewport.iter().rev().enumerate() {
        w.write_str("  ");
        for (dx, tile) in row.iter().enumerate() {
            let world_x = start_x + dx as i32;
            let world_y = start_y + (VIEWPORT_HEIGHT - 1 - dy) as i32;

            // Check for climbers at this position
            let climber_here = run.climbers.values()
                .find(|c| c.x == world_x && c.y == world_y);

            if let Some(c) = climber_here {
                w.set_fg(if c.is_active() { Color::LightGreen } else { Color::LightRed });
                w.write_str(if c.user_id == focus.user_id { "@" } else { "O" });
            } else {
                // Check for placed items
                let placed_items = run.get_items_at(world_x, world_y);
                if let Some(item) = placed_items.first() {
                    w.set_fg(Color::Brown);
                    match item.item_type {
                        super::data::ItemType::Rope => w.write_str("|"),
                        super::data::ItemType::Piton => w.write_str("+"),
                        _ => w.write_str("*"),
                    }
                } else {
                    // Render tile
                    w.set_fg(tile_color(tile.tile_type, biome));
                    let ch = tile.tile_type.character(biome);
                    w.write_str(&ch.to_string());
                }
            }
        }
        w.reset_color();
        w.writeln("");
    }
}

fn render_climber_status(w: &mut AnsiWriter, climber: &ClimberState, is_you: bool) {
    w.set_fg(Color::White);
    w.write_str(&format!("  {} ", if is_you { "YOU:" } else { &climber.handle }));

    // Stamina bar
    w.set_fg(Color::DarkGray);
    w.write_str("[");
    let stamina_bars = (climber.stamina_current as usize * 15) / 100;
    let max_bars = (climber.stamina_max as usize * 15) / 100;

    for i in 0..15 {
        if i < stamina_bars {
            w.set_fg(if climber.stamina_current > 30 { Color::LightGreen } else { Color::LightRed });
            w.write_str("=");
        } else if i < max_bars {
            w.set_fg(Color::DarkGray);
            w.write_str("-");
        } else {
            w.set_fg(Color::Red);
            w.write_str("x");
        }
    }
    w.set_fg(Color::DarkGray);
    w.write_str("] ");

    // Status effects
    if !climber.status_effects.is_empty() {
        w.set_fg(Color::Yellow);
        for effect in &climber.status_effects {
            use super::state::StatusEffectType::*;
            let icon = match effect.effect_type {
                Cold => "C",
                Poisoned => "P",
                Hungry => "H",
                Exhausted => "E",
                Invulnerable => "*",
                UnlimitedStamina => "U",
                Jitters => "J",
                Heavy => "W",
                Hallucinating => "?",
            };
            w.write_str(icon);
        }
        w.write_str(" ");
    }

    // Height
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("Height: {}m", climber.y));
}

fn render_climber_status_compact(w: &mut AnsiWriter, climber: &ClimberState) {
    w.set_fg(match climber.status {
        ClimberStatus::Active => Color::LightGreen,
        ClimberStatus::Downed => Color::LightRed,
        ClimberStatus::Dead => Color::DarkGray,
        ClimberStatus::Disconnected => Color::DarkGray,
    });

    let status_char = match climber.status {
        ClimberStatus::Active => "O",
        ClimberStatus::Downed => "X",
        ClimberStatus::Dead => "D",
        ClimberStatus::Disconnected => "-",
    };

    w.write_str(&format!("{} [{}] {}%",
        &climber.handle[..climber.handle.len().min(8)],
        status_char,
        climber.stamina_current
    ));
}

pub fn render_inventory(climber: &ClimberState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  INVENTORY");
    w.reset_color();

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(40));

    w.set_fg(Color::White);
    w.writeln("  Items:");

    let items: Vec<_> = climber.items.iter()
        .filter(|(_, count)| **count > 0)
        .collect();

    if items.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("    No items");
    } else {
        for (i, (item_type, count)) in items.iter().enumerate() {
            if let Some(item) = super::data::get_item(**item_type) {
                w.set_fg(Color::LightCyan);
                w.write_str(&format!("    [{}] ", i + 1));
                w.set_fg(Color::White);
                w.write_str(item.name);
                w.set_fg(Color::DarkGray);
                w.writeln(&format!(" x{}", count));
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Food:");

    if climber.foods.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("    No food");
    } else {
        for food_id in &climber.foods {
            if let Some(food) = super::data::get_food(*food_id) {
                w.set_fg(Color::LightGreen);
                w.write_str("    - ");
                w.set_fg(Color::White);
                w.writeln(food.name);
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  [B] Back");
    w.reset_color();

    w.flush()
}

pub fn render_campfire(climber: &ClimberState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::LightRed);
    w.writeln("");
    w.writeln("      (");
    w.writeln("       )");
    w.writeln("      (");
    w.writeln("     .");
    w.set_fg(Color::Yellow);
    w.writeln("    /|\\");
    w.writeln("   /_|_\\");
    w.set_fg(Color::Brown);
    w.writeln("  =======");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CAMPFIRE - Safe Zone");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Stamina: {}/{}",
        climber.stamina_current, climber.stamina_max));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [R] ");
    w.set_fg(Color::White);
    w.writeln("Rest (restore stamina)");

    if climber.has_item(super::data::ItemType::Marshmallow) {
        w.set_fg(Color::LightCyan);
        w.write_str("    [M] ");
        w.set_fg(Color::White);
        w.writeln("Roast Marshmallow");
    }

    if !climber.foods.is_empty() {
        w.set_fg(Color::LightCyan);
        w.write_str("    [C] ");
        w.set_fg(Color::White);
        w.writeln("Cook/Eat Food");
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("    [L] Leave campfire");
    w.reset_color();

    w.flush()
}

pub fn render_roast_marshmallow(heat_level: u32) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  ROASTING MARSHMALLOW");
    w.reset_color();
    w.writeln("");

    // Visual representation
    let marshmallow_color = if heat_level < 40 {
        Color::White
    } else if heat_level < 70 {
        Color::Yellow
    } else if heat_level < 90 {
        Color::Brown
    } else {
        Color::LightRed
    };

    w.set_fg(marshmallow_color);
    w.writeln("           ___");
    w.writeln("          /   \\");
    w.writeln("         |     |");
    w.writeln("          \\___/");
    w.set_fg(Color::Brown);
    w.writeln("            |");
    w.writeln("            |");
    w.writeln("            |");
    w.reset_color();

    // Heat meter
    w.writeln("");
    w.set_fg(Color::White);
    w.write_str("  Heat: [");
    for i in 0..10 {
        let threshold = i * 10;
        if heat_level > threshold {
            if heat_level > 90 {
                w.set_fg(Color::LightRed);
            } else if heat_level > 60 {
                w.set_fg(Color::Yellow);
            } else {
                w.set_fg(Color::Green);
            }
            w.write_str("=");
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str("-");
        }
    }
    w.set_fg(Color::White);
    w.writeln("]");

    w.writeln("");
    if heat_level < 40 {
        w.set_fg(Color::LightGray);
        w.writeln("  Too cold... keep holding!");
    } else if heat_level < 70 {
        w.set_fg(Color::LightGreen);
        w.writeln("  Getting golden... almost perfect!");
    } else if heat_level < 90 {
        w.set_fg(Color::Yellow);
        w.writeln("  PERFECT! Remove it now!");
    } else {
        w.set_fg(Color::LightRed);
        w.writeln("  IT'S BURNING!");
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [SPACE] ");
    w.set_fg(Color::White);
    w.writeln("Hold over fire");

    w.set_fg(Color::LightCyan);
    w.write_str("  [E] ");
    w.set_fg(Color::White);
    w.writeln("Remove from fire");
    w.reset_color();

    w.flush()
}

pub fn render_summit() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.writeln("");
    w.writeln("");
    w.writeln("              *    *");
    w.writeln("           *    *    *");
    w.writeln("        *    *    *    *");
    w.set_fg(Color::White);
    w.bold();
    w.writeln("");
    w.writeln("    ╔═════════════════════════════════╗");
    w.writeln("    ║                                 ║");
    w.writeln("    ║        Y O U   M A D E   I T    ║");
    w.writeln("    ║                                 ║");
    w.writeln("    ║      S U M M I T   R E A C H E D  ║");
    w.writeln("    ║                                 ║");
    w.writeln("    ╚═════════════════════════════════╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("    The rescue helicopter spots your signal!");
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("    Press any key to see results...");
    w.reset_color();

    w.flush()
}

pub fn render_game_over(run: &RunState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::LightRed);
    w.writeln("");
    w.writeln("");
    w.writeln("    ╔═════════════════════════════════╗");
    w.writeln("    ║                                 ║");
    w.writeln("    ║      E X P E D I T I O N       ║");
    w.writeln("    ║           F A I L E D           ║");
    w.writeln("    ║                                 ║");
    w.writeln("    ╚═════════════════════════════════╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("    Highest reached: {}m", run.highest_reached));
    w.writeln(&format!("    Time: {}", run.elapsed_display()));
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("    Press any key to continue...");
    w.reset_color();

    w.flush()
}

pub fn render_results(run: &RunState, climber: &ClimberState, stats: &PlayerStats) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    let success = run.status == RunStatus::Summit;

    w.set_fg(if success { Color::Yellow } else { Color::LightRed });
    w.bold();
    w.writeln(&format!("  EXPEDITION {}",
        if success { "COMPLETE" } else { "FAILED" }
    ));
    w.reset_color();

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(40));

    w.set_fg(Color::White);
    w.writeln(&format!("  Time: {}", run.elapsed_display()));
    w.writeln(&format!("  Height Reached: {}m", climber.y.max(0) as u32));
    w.writeln(&format!("  Biomes Visited: {}", run.biomes_reached.len()));
    w.writeln("");
    w.writeln(&format!("  Falls: {}", climber.falls));
    w.writeln(&format!("  Items Used: {}", climber.items_used));
    w.writeln(&format!("  Foods Eaten: {}", climber.foods_eaten.len()));
    w.writeln(&format!("  Revives Given: {}", climber.revives_given));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  Career Stats:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Total Summits: {}", stats.total_summits));
    w.writeln(&format!("    Badges Earned: {}/{}", stats.badges.len(), BADGES.len()));

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

pub fn render_stats(stats: &PlayerStats) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Your Scout Career");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(50));

    // Stats
    w.set_fg(Color::White);
    w.writeln(&format!("  Total Runs: {}", stats.total_runs));
    w.writeln(&format!("  Summits: {}", stats.total_summits));
    w.writeln(&format!("  Highest Reached: {}m", stats.highest_reached));
    w.writeln(&format!("  Fastest Summit: {}",
        stats.fastest_summit_seconds
            .map(|s| format!("{}:{:02}", s / 60, s % 60))
            .unwrap_or_else(|| "--:--".to_string())
    ));

    w.writeln("");
    w.writeln(&format!("  Total Falls: {}", stats.total_falls));
    w.writeln(&format!("  Revives Given: {}", stats.total_revives_given));
    w.writeln(&format!("  Ropes Placed: {}", stats.total_ropes_placed));
    w.writeln(&format!("  Foods Tried: {}/30", stats.foods_tried.len()));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Badges: {}/{}", stats.badges.len(), BADGES.len()));
    w.set_fg(Color::White);

    // Show earned badges
    for badge in BADGES.iter() {
        if stats.has_badge(badge.id) {
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("    [{}] ", badge.icon));
            w.set_fg(Color::White);
            w.writeln(badge.name);
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  [B] Back");
    w.reset_color();

    w.flush()
}

pub fn render_cosmetics(stats: &PlayerStats) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Customize Your Scout");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  [1] Uniform: {}", stats.equipped_cosmetics.uniform));
    w.writeln(&format!("  [2] Hat: {}",
        stats.equipped_cosmetics.hat.as_deref().unwrap_or("None")));
    w.writeln(&format!("  [3] Backpack: {}", stats.equipped_cosmetics.backpack));
    w.writeln(&format!("  [4] Accessory: {}",
        stats.equipped_cosmetics.accessory.as_deref().unwrap_or("None")));
    w.writeln(&format!("  [5] Rope Color: {}", stats.equipped_cosmetics.rope_color));

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Press number to cycle options");
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  [B] Back");
    w.reset_color();

    w.flush()
}

pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.writeln("");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Are you sure you want to quit?");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Your progress will be lost.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Y] ");
    w.set_fg(Color::White);
    w.writeln("Yes, quit");
    w.set_fg(Color::LightCyan);
    w.write_str("    [N] ");
    w.set_fg(Color::White);
    w.writeln("No, go back");
    w.reset_color();

    w.flush()
}

// ============================================================================
// MAIN RENDER FUNCTION
// ============================================================================

pub fn render_screen(flow: &SummitFlow, lobby_manager: &SummitLobby) -> String {
    match &flow.screen {
        GameScreen::Lobby(lobby_screen) => {
            match lobby_screen {
                LobbyScreen::MainMenu => render_lobby_main_menu(&flow.stats),
                LobbyScreen::CreateGame => render_create_game(),
                LobbyScreen::JoinGame => {
                    let lobbies = lobby_manager.list_public_lobbies();
                    render_join_game(&lobbies)
                }
                LobbyScreen::EnterInviteCode => {
                    // Buffer is in flow, we need to expose it
                    render_enter_invite_code("")
                }
                LobbyScreen::WaitingRoom => {
                    if let Some(lobby) = lobby_manager.get_player_lobby(flow.user_id) {
                        render_waiting_room(lobby, flow.user_id)
                    } else {
                        render_lobby_main_menu(&flow.stats)
                    }
                }
                LobbyScreen::Countdown => render_countdown(),
            }
        }
        GameScreen::CrashLanding => render_crash_landing(),
        GameScreen::Climbing => {
            if let Some(game) = lobby_manager.get_player_game(flow.user_id) {
                render_climbing(game, flow.user_id)
            } else {
                render_lobby_main_menu(&flow.stats)
            }
        }
        GameScreen::Inventory => {
            if let Some(game) = lobby_manager.get_player_game(flow.user_id) {
                if let Some(climber) = game.run.get_climber(flow.user_id) {
                    return render_inventory(climber);
                }
            }
            render_lobby_main_menu(&flow.stats)
        }
        GameScreen::EatFood => {
            if let Some(game) = lobby_manager.get_player_game(flow.user_id) {
                if let Some(climber) = game.run.get_climber(flow.user_id) {
                    return render_inventory(climber);  // Same as inventory for now
                }
            }
            render_lobby_main_menu(&flow.stats)
        }
        GameScreen::Campfire => {
            if let Some(game) = lobby_manager.get_player_game(flow.user_id) {
                if let Some(climber) = game.run.get_climber(flow.user_id) {
                    return render_campfire(climber);
                }
            }
            render_lobby_main_menu(&flow.stats)
        }
        GameScreen::RoastMarshmallow { heat_level } => {
            render_roast_marshmallow(*heat_level)
        }
        GameScreen::RevivePartner { .. } => {
            let mut w = AnsiWriter::new();
            w.clear_screen();
            w.set_fg(Color::Yellow);
            w.writeln("  Hold [H] to revive your teammate!");
            w.reset_color();
            w.flush()
        }
        GameScreen::Summit => render_summit(),
        GameScreen::GameOver => {
            if let Some(game) = lobby_manager.get_player_game(flow.user_id) {
                render_game_over(&game.run)
            } else {
                render_game_over(&RunState::new(0, String::new(), 0))
            }
        }
        GameScreen::Results => {
            if let Some(game) = lobby_manager.get_player_game(flow.user_id) {
                if let Some(climber) = game.run.get_climber(flow.user_id) {
                    return render_results(&game.run, climber, &flow.stats);
                }
            }
            render_lobby_main_menu(&flow.stats)
        }
        GameScreen::Stats => render_stats(&flow.stats),
        GameScreen::Cosmetics => render_cosmetics(&flow.stats),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_lobby_main_menu() {
        let stats = PlayerStats::new(1);
        let output = render_lobby_main_menu(&stats);
        assert!(output.contains("Summit")); // In "Reach the Summit"
        assert!(output.contains("Create Game"));
        assert!(output.contains("Join Game"));
    }

    #[test]
    fn test_render_create_game() {
        let output = render_create_game();
        assert!(output.contains("Public Game"));
        assert!(output.contains("Friends Only"));
    }

    #[test]
    fn test_render_crash_landing() {
        let output = render_crash_landing();
        assert!(output.contains("CRASH"));
        assert!(output.contains("Summit"));
    }

    #[test]
    fn test_render_summit() {
        let output = render_summit();
        assert!(output.contains("S U M M I T"));
        assert!(output.contains("R E A C H E D"));
    }

    #[test]
    fn test_render_confirm_quit() {
        let output = render_confirm_quit();
        assert!(output.contains("quit"));
        assert!(output.contains("[Y]"));
        assert!(output.contains("[N]"));
    }

    #[test]
    fn test_biome_colors() {
        assert_eq!(biome_color(BiomeType::Beach), Color::Yellow);
        assert_eq!(biome_color(BiomeType::Jungle), Color::Green);
        assert_eq!(biome_color(BiomeType::Alpine), Color::White);
        assert_eq!(biome_color(BiomeType::Volcanic), Color::LightRed);
    }
}
