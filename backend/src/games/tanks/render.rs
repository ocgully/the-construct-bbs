//! ANSI rendering functions for Tanks
//!
//! Military-themed visual identity with olive/khaki/steel color palette.

use crate::terminal::{AnsiWriter, Color};
use super::data::{colors, tank_color, FIELD_WIDTH, FIELD_HEIGHT};
use super::state::{TankGame, TankState};
use super::terrain::{Terrain, TerrainType};
use super::lobby::{GameLobby, MatchType};
use super::physics::calculate_aim_preview;

/// Render the Tanks title header
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(colors::OLIVE);
    w.bold();
    w.writeln("");
    w.writeln("  ████████╗ █████╗ ███╗   ██╗██╗  ██╗███████╗");
    w.writeln("  ╚══██╔══╝██╔══██╗████╗  ██║██║ ██╔╝██╔════╝");
    w.writeln("     ██║   ███████║██╔██╗ ██║█████╔╝ ███████╗");
    w.writeln("     ██║   ██╔══██║██║╚██╗██║██╔═██╗ ╚════██║");
    w.writeln("     ██║   ██║  ██║██║ ╚████║██║  ██╗███████║");
    w.writeln("     ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝");
    w.set_fg(colors::KHAKI);
    w.writeln("                    TANKS");
    w.writeln("           B L I T Z K R I E G");
    w.reset_color();
}

/// Render main menu
pub fn render_tanks_menu() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(colors::HEADER);
    w.bold();
    w.writeln("  COMMAND CENTER");
    w.reset_color();
    w.writeln("");

    w.set_fg(colors::RADAR);
    w.write_str("    [J] ");
    w.set_fg(Color::White);
    w.writeln("Join Public Battle");

    w.set_fg(colors::RADAR);
    w.write_str("    [C] ");
    w.set_fg(Color::White);
    w.writeln("Create Public Battle");

    w.set_fg(colors::RADAR);
    w.write_str("    [P] ");
    w.set_fg(Color::White);
    w.writeln("Create Private Battle");

    w.set_fg(colors::RADAR);
    w.write_str("    [I] ");
    w.set_fg(Color::White);
    w.writeln("Join by Invite Code");

    w.writeln("");

    w.set_fg(colors::RADAR);
    w.write_str("    [L] ");
    w.set_fg(Color::White);
    w.writeln("Leaderboard");

    w.set_fg(colors::RADAR);
    w.write_str("    [H] ");
    w.set_fg(Color::White);
    w.writeln("How to Play");

    w.set_fg(colors::RADAR);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Return to BBS");

    w.writeln("");
    w.set_fg(colors::KHAKI);
    w.write_str("  > ");
    w.reset_color();

    w.flush()
}

/// Render how to play screen
pub fn render_how_to_play() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(colors::HEADER);
    w.bold();
    w.writeln("  FIELD MANUAL");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  Tanks is a turn-based artillery game. Destroy all enemy tanks to win!");
    w.writeln("");

    w.set_fg(colors::AMMO);
    w.writeln("  CONTROLS:");
    w.set_fg(Color::White);
    w.writeln("    Left/Right Arrow  - Adjust angle");
    w.writeln("    Up/Down Arrow     - Adjust power");
    w.writeln("    W/S               - Fine angle control (+/-1)");
    w.writeln("    A/D               - Fine power control (+/-1)");
    w.writeln("    Tab               - Cycle weapon");
    w.writeln("    Space/Enter       - FIRE!");
    w.writeln("    V                 - View other tanks (Picture-in-Picture)");
    w.writeln("");

    w.set_fg(colors::AMMO);
    w.writeln("  PHYSICS:");
    w.set_fg(Color::White);
    w.writeln("    - Projectiles arc due to gravity");
    w.writeln("    - Wind affects trajectory (shown in HUD)");
    w.writeln("    - Terrain is destructible");
    w.writeln("");

    w.set_fg(colors::RADAR);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render game list
pub fn render_game_list(lobbies: &[(u64, usize, usize)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(colors::HEADER);
    w.bold();
    w.writeln("  AVAILABLE BATTLES");
    w.reset_color();
    w.writeln("");

    if lobbies.is_empty() {
        w.set_fg(colors::KHAKI);
        w.writeln("    No battles in progress. Create one!");
    } else {
        w.set_fg(colors::SMOKE);
        w.writeln("     #   Players    Status");
        w.writeln(&format!("    {}", "\u{2500}".repeat(30)));
        w.reset_color();

        for (i, (id, players, max)) in lobbies.iter().enumerate() {
            w.set_fg(colors::RADAR);
            w.write_str(&format!("    [{:1}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("Battle #{:<4}", id));
            w.set_fg(if *players < *max { colors::RADAR } else { colors::ALERT });
            w.writeln(&format!("  {}/{}", players, max));
        }
    }

    w.writeln("");
    w.set_fg(colors::RADAR);
    w.write_str("    [B] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.writeln("");
    w.set_fg(colors::KHAKI);
    w.write_str("  > ");
    w.reset_color();

    w.flush()
}

/// Render invite code prompt
pub fn render_invite_prompt(current: &str) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(colors::HEADER);
    w.bold();
    w.writeln("  ENTER INVITE CODE");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  Enter the 6-character code to join a private battle.");
    w.writeln("");

    w.set_fg(colors::AMMO);
    w.write_str("  Code: ");
    w.set_fg(Color::White);
    w.bold();
    w.write_str(current);
    w.write_str("_");
    w.reset_color();

    w.writeln("");
    w.writeln("");
    w.set_fg(colors::RADAR);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Cancel");

    w.flush()
}

/// Render lobby waiting screen
pub fn render_lobby(lobby: &GameLobby) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");

    match &lobby.match_type {
        MatchType::Private { invite_code } => {
            w.set_fg(colors::ALERT);
            w.write_str("  PRIVATE BATTLE  ");
            w.set_fg(colors::AMMO);
            w.bold();
            w.writeln(&format!("Invite Code: {}", invite_code));
        }
        MatchType::Public => {
            w.set_fg(colors::HEADER);
            w.bold();
            w.writeln("  PUBLIC BATTLE");
        }
    }
    w.reset_color();
    w.writeln("");

    w.set_fg(colors::SMOKE);
    w.writeln("  COMBATANTS:");
    w.writeln(&format!("    {}", "\u{2500}".repeat(40)));
    w.reset_color();

    for (i, player) in lobby.players.iter().enumerate() {
        let is_host = player.user_id == lobby.host_user_id;

        w.set_fg(tank_color(i));
        w.write_str(&format!("    {}. ", i + 1));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<16}", player.handle));

        if is_host {
            w.set_fg(colors::AMMO);
            w.write_str(" [COMMANDER]");
        }

        w.set_fg(if player.is_ready { colors::RADAR } else { colors::SMOKE });
        w.writeln(if player.is_ready { " READY" } else { " ---" });
    }

    w.writeln("");
    w.set_fg(colors::SMOKE);
    w.writeln(&format!("    {}", "\u{2500}".repeat(40)));
    w.writeln("");

    w.set_fg(colors::RADAR);
    w.write_str("    [R] ");
    w.set_fg(Color::White);
    w.writeln("Toggle Ready");

    if lobby.host_user_id == lobby.players[0].user_id && lobby.can_start() {
        w.set_fg(colors::AMMO);
        w.bold();
        w.write_str("    [S] ");
        w.set_fg(Color::White);
        w.writeln("START BATTLE!");
        w.reset_color();
    } else if lobby.player_count() < 2 {
        w.set_fg(colors::SMOKE);
        w.writeln("    Waiting for more players...");
    } else if !lobby.can_start() {
        w.set_fg(colors::SMOKE);
        w.writeln("    Waiting for all players to ready up...");
    }

    w.set_fg(colors::RADAR);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave Lobby");

    w.flush()
}

/// Render the main game battlefield
pub fn render_battlefield(game: &TankGame, viewer_user_id: i64, show_aim: bool) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    // Find viewer's tank (for PIP and aiming)
    let viewer_tank = game.get_tank_by_user(viewer_user_id).map(|(_, t)| t);
    let current_tank = game.get_current_tank();

    // Render status bar at top
    render_hud(&mut w, game, viewer_tank);

    // Render the terrain and tanks
    render_terrain_with_tanks(&mut w, &game.terrain, &game.tanks.values().collect::<Vec<_>>(), game.wind, current_tank, show_aim);

    // Render controls at bottom
    render_controls(&mut w, game, viewer_user_id);

    w.flush()
}

fn render_hud(w: &mut AnsiWriter, game: &TankGame, viewer_tank: Option<&TankState>) {
    // Top status line
    w.set_fg(colors::SMOKE);
    w.writeln(&"\u{2500}".repeat(FIELD_WIDTH));

    w.set_fg(colors::HEADER);
    w.write_str(&format!(" Round {:2}", game.round));

    w.set_fg(colors::SMOKE);
    w.write_str(" | ");

    // Wind indicator
    w.set_fg(colors::AMMO);
    w.write_str("Wind: ");
    if game.wind > 0 {
        w.set_fg(colors::RADAR);
        w.write_str(&format!("{:>2} -->", game.wind));
    } else if game.wind < 0 {
        w.set_fg(colors::ALERT);
        w.write_str(&format!("<-- {:<2}", -game.wind));
    } else {
        w.set_fg(colors::SMOKE);
        w.write_str("Calm  ");
    }

    w.set_fg(colors::SMOKE);
    w.write_str(" | ");

    // Current turn
    if let Some(tank) = game.get_current_tank() {
        w.set_fg(colors::AMMO);
        w.write_str(&format!("{}'s turn", tank.handle));
    }

    // Viewer's stats
    if let Some(tank) = viewer_tank {
        w.set_fg(colors::SMOKE);
        w.write_str(" | ");

        w.set_fg(if tank.health > 50 { colors::RADAR } else { colors::ALERT });
        w.write_str(&format!("HP:{:3}", tank.health));

        if let Some(weapon) = tank.get_current_weapon_def() {
            w.set_fg(colors::SMOKE);
            w.write_str(" | ");
            w.set_fg(colors::AMMO);
            w.write_str(&format!("{}", weapon.name));
            if let Some(ammo) = tank.get_current_weapon().and_then(|w| w.ammo) {
                w.write_str(&format!("({})", ammo));
            }
        }
    }

    w.writeln("");
    w.set_fg(colors::SMOKE);
    w.writeln(&"\u{2500}".repeat(FIELD_WIDTH));
    w.reset_color();
}

fn render_terrain_with_tanks(
    w: &mut AnsiWriter,
    terrain: &Terrain,
    tanks: &[&TankState],
    wind: i32,
    current_tank: Option<&TankState>,
    show_aim: bool,
) {
    // Build a buffer for the field
    let mut field: Vec<Vec<(char, Color)>> = vec![vec![(' ', colors::SKY); FIELD_WIDTH]; FIELD_HEIGHT];

    // Draw terrain
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            let terrain_type = terrain.get(x, y);
            let (ch, color) = match terrain_type {
                TerrainType::Air => (' ', colors::SKY),
                TerrainType::Dirt => ('#', colors::KHAKI),
                TerrainType::Rock => ('%', colors::SMOKE),
                TerrainType::Bedrock => ('=', Color::DarkGray),
            };
            if y < FIELD_HEIGHT && x < FIELD_WIDTH {
                field[y][x] = (ch, color);
            }
        }
    }

    // Draw aim preview if active
    if show_aim {
        if let Some(tank) = current_tank {
            let preview = calculate_aim_preview(
                tank.x as f64,
                (tank.y as f64) - 1.0,
                tank.angle,
                tank.power,
                wind,
                20,
            );
            for (i, (px, py)) in preview.iter().enumerate() {
                if *px >= 0 && *py >= 0 && (*px as usize) < FIELD_WIDTH && (*py as usize) < FIELD_HEIGHT {
                    let ch = if i % 3 == 0 { '*' } else { '.' };
                    field[*py as usize][*px as usize] = (ch, colors::SMOKE);
                }
            }
        }
    }

    // Draw tanks
    for (i, tank) in tanks.iter().enumerate() {
        if !tank.is_alive {
            continue;
        }

        let color = tank_color(i);
        let x = tank.x;
        let y = tank.y;

        // Simple tank representation: [^] or [<] or [>]
        let turret = if tank.angle <= 60 {
            '>'
        } else if tank.angle >= 120 {
            '<'
        } else {
            '^'
        };

        // Draw tank body
        if y > 0 && y < FIELD_HEIGHT && x > 0 && x < FIELD_WIDTH - 1 {
            // Turret
            if y > 0 {
                field[y - 1][x] = (turret, color);
            }
            // Body
            if x > 0 {
                field[y][x - 1] = ('[', color);
            }
            field[y][x] = ('O', color);
            if x + 1 < FIELD_WIDTH {
                field[y][x + 1] = (']', color);
            }
        }
    }

    // Render the field
    for y in 0..FIELD_HEIGHT {
        for x in 0..FIELD_WIDTH {
            let (ch, color) = field[y][x];
            w.set_fg(color);
            w.write_str(&ch.to_string());
        }
        w.writeln("");
    }
    w.reset_color();
}

fn render_controls(w: &mut AnsiWriter, game: &TankGame, viewer_user_id: i64) {
    w.set_fg(colors::SMOKE);
    w.writeln(&"\u{2500}".repeat(FIELD_WIDTH));

    let is_my_turn = game.get_current_tank()
        .map(|t| t.user_id == viewer_user_id)
        .unwrap_or(false);

    if is_my_turn {
        if let Some(tank) = game.get_tank_by_user(viewer_user_id).map(|(_, t)| t) {
            w.set_fg(colors::AMMO);
            w.write_str(&format!(" Angle: {:3}  ", tank.angle));
            w.set_fg(colors::RADAR);
            w.write_str(&format!("Power: {:3}%  ", tank.power));

            w.set_fg(Color::White);
            w.writeln("[Arrow Keys] Aim  [Space] FIRE!  [Tab] Weapon  [V] View");
        }
    } else {
        w.set_fg(colors::SMOKE);
        w.writeln(" Waiting for opponent's move...  [V] View other tanks  [Q] Leave");
    }

    w.reset_color();
}

/// Render turn result (explosion, damage dealt)
pub fn render_turn_result(game: &TankGame, message: &str) -> String {
    let mut w = AnsiWriter::new();

    // Show the battlefield state
    w.write_str(&render_battlefield(game, 0, false));

    // Show result message
    w.set_fg(colors::EXPLOSION);
    w.bold();
    w.writeln("");
    w.writeln(&format!("  {}", message));
    w.reset_color();

    if let Some(result) = &game.last_result {
        if !result.hits.is_empty() {
            w.set_fg(colors::ALERT);
            w.writeln("  HITS:");
            for (tank_id, damage) in &result.hits {
                if let Some(tank) = game.tanks.get(tank_id) {
                    w.writeln(&format!("    {} took {} damage!", tank.handle, damage));
                    if !tank.is_alive {
                        w.set_fg(colors::EXPLOSION);
                        w.bold();
                        w.writeln(&format!("    {} DESTROYED!", tank.handle));
                        w.reset_color();
                    }
                }
            }
        }
    }

    w.writeln("");
    w.set_fg(colors::SMOKE);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render game over screen
pub fn render_game_over(game: &TankGame) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(colors::EXPLOSION);
    w.bold();
    w.writeln("");
    w.writeln("  ██████╗  █████╗ ███╗   ███╗███████╗     ██████╗ ██╗   ██╗███████╗██████╗ ");
    w.writeln("  ██╔════╝ ██╔══██╗████╗ ████║██╔════╝    ██╔═══██╗██║   ██║██╔════╝██╔══██╗");
    w.writeln("  ██║  ███╗███████║██╔████╔██║█████╗      ██║   ██║██║   ██║█████╗  ██████╔╝");
    w.writeln("  ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝      ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗");
    w.writeln("  ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗    ╚██████╔╝ ╚████╔╝ ███████╗██║  ██║");
    w.writeln("   ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝     ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝");
    w.reset_color();

    w.writeln("");

    if let Some(winner) = game.get_winner() {
        w.set_fg(colors::AMMO);
        w.bold();
        w.writeln(&format!("  VICTORY: {} conquers the battlefield!", winner.handle));
        w.reset_color();
    } else {
        w.set_fg(colors::SMOKE);
        w.writeln("  The battle ends in mutual destruction...");
    }

    w.writeln("");
    w.set_fg(colors::HEADER);
    w.bold();
    w.writeln("  BATTLE STATISTICS");
    w.reset_color();
    w.writeln("");

    w.set_fg(colors::SMOKE);
    w.writeln(&format!("  {:<16} {:>6} {:>8} {:>8}", "Combatant", "Kills", "Damage", "Status"));
    w.writeln(&format!("  {}", "\u{2500}".repeat(44)));

    let standings = game.get_standings();
    for (handle, kills, damage, alive) in standings {
        w.set_fg(if alive { colors::RADAR } else { colors::SMOKE });
        w.write_str(&format!("  {:<16}", handle));
        w.set_fg(colors::AMMO);
        w.write_str(&format!("{:>6}", kills));
        w.set_fg(Color::White);
        w.write_str(&format!("{:>8}", damage));
        w.set_fg(if alive { colors::RADAR } else { colors::ALERT });
        w.writeln(&format!("{:>8}", if alive { "ALIVE" } else { "KIA" }));
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(colors::RADAR);
    w.writeln("  Press any key to return to menu...");
    w.reset_color();

    w.flush()
}

/// Render leaderboard
pub fn render_leaderboard(entries: &[(String, u32, u32, u32)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(colors::HEADER);
    w.bold();
    w.writeln("  HALL OF HEROES");
    w.reset_color();
    w.writeln("");

    w.set_fg(colors::SMOKE);
    w.writeln(&format!("  {:<4} {:<16} {:>8} {:>10} {:>8}", "Rank", "Commander", "Wins", "Kills", "Accuracy"));
    w.writeln(&format!("  {}", "\u{2500}".repeat(50)));

    if entries.is_empty() {
        w.set_fg(colors::KHAKI);
        w.writeln("    No battles recorded yet.");
    } else {
        for (i, (handle, wins, kills, accuracy)) in entries.iter().enumerate() {
            let rank_color = match i {
                0 => colors::AMMO,
                1 => colors::STEEL,
                2 => colors::KHAKI,
                _ => colors::SMOKE,
            };
            w.set_fg(rank_color);
            w.write_str(&format!("  {:<4}", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<16}", handle));
            w.set_fg(colors::RADAR);
            w.write_str(&format!("{:>8}", wins));
            w.set_fg(colors::ALERT);
            w.write_str(&format!("{:>10}", kills));
            w.set_fg(colors::AMMO);
            w.writeln(&format!("{:>7}%", accuracy));
        }
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(colors::RADAR);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render error message
pub fn render_error(message: &str) -> String {
    let mut w = AnsiWriter::new();
    w.set_fg(colors::ALERT);
    w.writeln("");
    w.writeln(&format!("  ERROR: {}", message));
    w.reset_color();
    w.flush()
}

/// Render PIP (Picture-in-Picture) view of another tank
pub fn render_pip_view(tank: &TankState, terrain: &Terrain) -> String {
    let mut w = AnsiWriter::new();

    // Small 20x8 view centered on the tank
    let view_width = 20;
    let view_height = 8;
    let start_x = tank.x.saturating_sub(view_width / 2);
    let start_y = tank.y.saturating_sub(view_height / 2);

    w.set_fg(colors::SMOKE);
    w.writeln(&format!("  [{} HP:{}/{}]", tank.handle, tank.health, tank.max_health));
    w.writeln(&format!("  {}", "\u{250c}".to_string() + &"\u{2500}".repeat(view_width) + "\u{2510}"));

    for vy in 0..view_height {
        w.write_str("  \u{2502}");
        for vx in 0..view_width {
            let x = start_x + vx;
            let y = start_y + vy;

            // Check if tank is at this position
            if x == tank.x && y == tank.y {
                w.set_fg(colors::RADAR);
                w.write_str("O");
            } else if x < terrain.width && y < terrain.height {
                let t = terrain.get(x, y);
                match t {
                    TerrainType::Air => {
                        w.set_fg(colors::SKY);
                        w.write_str(" ");
                    }
                    TerrainType::Dirt => {
                        w.set_fg(colors::KHAKI);
                        w.write_str("#");
                    }
                    TerrainType::Rock => {
                        w.set_fg(colors::SMOKE);
                        w.write_str("%");
                    }
                    TerrainType::Bedrock => {
                        w.set_fg(Color::DarkGray);
                        w.write_str("=");
                    }
                }
            } else {
                w.write_str(" ");
            }
        }
        w.set_fg(colors::SMOKE);
        w.writeln("\u{2502}");
    }

    w.writeln(&format!("  {}", "\u{2514}".to_string() + &"\u{2500}".repeat(view_width) + "\u{2518}"));
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_menu() {
        let output = render_tanks_menu();
        assert!(!output.is_empty());
        assert!(output.contains("TANKS"));
        assert!(output.contains("Join"));
        assert!(output.contains("Create"));
    }

    #[test]
    fn test_render_how_to_play() {
        let output = render_how_to_play();
        assert!(output.contains("FIELD MANUAL"));
        assert!(output.contains("Arrow"));
        assert!(output.contains("FIRE"));
    }

    #[test]
    fn test_render_game_list_empty() {
        let output = render_game_list(&[]);
        assert!(output.contains("No battles"));
    }

    #[test]
    fn test_render_game_list_with_games() {
        let lobbies = vec![(1, 2, 8), (2, 4, 8)];
        let output = render_game_list(&lobbies);
        assert!(output.contains("Battle #1"));
        assert!(output.contains("2/8"));
    }

    #[test]
    fn test_render_leaderboard() {
        let entries = vec![
            ("Champion".to_string(), 10, 25, 65),
            ("Runner".to_string(), 5, 12, 50),
        ];
        let output = render_leaderboard(&entries);
        assert!(output.contains("Champion"));
        assert!(output.contains("10"));
    }
}
