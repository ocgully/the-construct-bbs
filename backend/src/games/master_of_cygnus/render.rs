//! ANSI rendering for Master of Andromeda
//!
//! Uses a distinct sci-fi visual identity: deep space blues, star yellows,
//! with cyan accents for interface elements.

use crate::terminal::{AnsiWriter, Color};
use super::state::{GameState, EmpireState, ColonyState};
use super::screen::{GameScreen, MocFlow};
use super::galaxy::{Star, PlanetType};
use super::tech::TechField;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Render the game header with title art
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightBlue);
    w.bold();
    w.writeln("");
    w.writeln("  ███╗   ███╗ █████╗ ███████╗████████╗███████╗██████╗ ");
    w.writeln("  ████╗ ████║██╔══██╗██╔════╝╚══██╔══╝██╔════╝██╔══██╗");
    w.writeln("  ██╔████╔██║███████║███████╗   ██║   █████╗  ██████╔╝");
    w.writeln("  ██║╚██╔╝██║██╔══██║╚════██║   ██║   ██╔══╝  ██╔══██╗");
    w.writeln("  ██║ ╚═╝ ██║██║  ██║███████║   ██║   ███████╗██║  ██║");
    w.writeln("  ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝");
    w.set_fg(Color::Yellow);
    w.writeln("  ╔═╗╔╗╔╔╦╗╦═╗╔═╗╔╦╗╔═╗╔╦╗╔═╗");
    w.writeln("  ╠═╣║║║ ║║╠╦╝║ ║║║║║╣  ║║╠═╣");
    w.writeln("  ╩ ╩╝╚╝═╩╝╩╚═╚═╝╩ ╩╚═╝═╩╝╩ ╩");
    w.writeln("       MASTER OF ANDROMEDA");
    w.reset_color();
}

/// Render status bar
fn render_status_bar(w: &mut AnsiWriter, game: &GameState, empire: Option<&EmpireState>) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    if let Some(emp) = empire {
        w.set_fg(Color::Yellow);
        w.write_str(&format!(" {} ", emp.name));
        w.set_fg(Color::DarkGray);
        w.write_str("| ");

        w.set_fg(Color::LightCyan);
        w.write_str(&format!("Turn {} ", game.turn_number));
        w.set_fg(Color::DarkGray);
        w.write_str("| ");

        w.set_fg(Color::LightGreen);
        w.write_str(&format!("Pop: {} ", emp.total_population()));
        w.set_fg(Color::DarkGray);
        w.write_str("| ");

        w.set_fg(Color::LightBlue);
        w.write_str(&format!("Colonies: {} ", emp.colonies.len()));
        w.set_fg(Color::DarkGray);
        w.write_str("| ");

        w.set_fg(Color::LightMagenta);
        w.write_str(&format!("Fleets: {}", emp.fleets.len()));
        w.writeln("");
    } else {
        w.set_fg(Color::LightCyan);
        w.writeln(&format!(" Turn {} | Waiting for players...", game.turn_number));
    }

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));
    w.reset_color();
}

/// Get color for a planet type
fn planet_color(planet_type: PlanetType) -> Color {
    match planet_type {
        PlanetType::Terran => Color::LightGreen,
        PlanetType::Ocean => Color::LightBlue,
        PlanetType::Arid => Color::Yellow,
        PlanetType::Tundra => Color::LightCyan,
        PlanetType::Barren => Color::Brown,
        PlanetType::Toxic => Color::LightMagenta,
        PlanetType::GasGiant => Color::LightRed,
    }
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Main render function - dispatches to screen-specific renderers
pub fn render_screen(flow: &MocFlow) -> String {
    let mut w = AnsiWriter::new();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(&mut w),
        GameScreen::GalaxyMap => render_galaxy_map(&mut w, flow),
        GameScreen::StarSystem { star_id } => render_star_system(&mut w, flow, *star_id),
        GameScreen::ColonyManagement { star_id } => render_colony(&mut w, flow, *star_id),
        GameScreen::FleetManagement { fleet_id } => render_fleet(&mut w, flow, *fleet_id),
        GameScreen::Research => render_research(&mut w, flow),
        GameScreen::ShipDesigner => render_ship_designer(&mut w, flow),
        GameScreen::TurnSummary => render_turn_summary(&mut w, flow),
        GameScreen::GameOver => render_game_over(&mut w, flow),
        GameScreen::Lobby => render_lobby(&mut w, flow),
        GameScreen::NewGame { step } => render_new_game(&mut w, *step),
        GameScreen::JoinGame => render_join_game(&mut w, flow),
        GameScreen::Settings => render_settings(&mut w),
        GameScreen::ConfirmQuit => render_confirm_quit(&mut w),
    }

    w.flush()
}

/// Render intro screen
fn render_intro(w: &mut AnsiWriter) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  The Andromeda Galaxy. A thousand stars await conquest.");
    w.writeln("");
    w.writeln("  Lead your civilization across the void. Build colonies on");
    w.writeln("  distant worlds. Research technologies lost to time. Design");
    w.writeln("  mighty warships. Forge alliances or crush your enemies.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  The stars are waiting. Will you answer their call?");
    w.writeln("");
    w.reset_color();

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

/// Render galaxy map
fn render_galaxy_map(w: &mut AnsiWriter, flow: &MocFlow) {
    render_header(w);

    let Some(game) = flow.game_state() else {
        w.set_fg(Color::LightRed);
        w.writeln("  No game loaded.");
        return;
    };

    let empire = flow.current_empire_id
        .and_then(|id| game.get_empire(id));

    render_status_bar(w, game, empire);

    // Display last message if any
    if let Some(ref msg) = game.last_message {
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
        w.writeln("");
    }

    // Simplified galaxy view - list of stars
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  GALAXY VIEW");
    w.reset_color();
    w.writeln("");

    // Header
    w.set_fg(Color::DarkGray);
    w.writeln("    ID   Star Name            Type       Owner");
    w.writeln(&format!("    {}", "\u{2500}".repeat(55)));
    w.reset_color();

    // Show stars (limited view)
    let known_stars: Vec<u32> = empire
        .map(|e| e.known_stars.clone())
        .unwrap_or_else(|| (0..10).collect());

    for star_id in known_stars.iter().take(15) {
        if let Some(star) = game.galaxy.get_star(*star_id) {
            render_star_row(w, star, game);
        }
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  COMMANDS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [#] ");
    w.set_fg(Color::White);
    w.writeln("View star by ID");

    w.set_fg(Color::LightCyan);
    w.write_str("    [C] ");
    w.set_fg(Color::White);
    w.writeln("Colony management");

    w.set_fg(Color::LightCyan);
    w.write_str("    [F] ");
    w.set_fg(Color::White);
    w.writeln("Fleet management");

    w.set_fg(Color::LightCyan);
    w.write_str("    [R] ");
    w.set_fg(Color::White);
    w.writeln("Research allocation");

    w.set_fg(Color::LightCyan);
    w.write_str("    [D] ");
    w.set_fg(Color::White);
    w.writeln("Ship designer");

    w.set_fg(Color::LightCyan);
    w.write_str("    [T] ");
    w.set_fg(Color::White);
    w.writeln("End turn / Submit orders");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Save & Quit");

    w.writeln("");
    w.write_str("  > ");
}

fn render_star_row(w: &mut AnsiWriter, star: &Star, game: &GameState) {
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("    {:2}   ", star.id));

    w.set_fg(Color::White);
    w.write_str(&format!("{:<20} ", star.name));

    w.set_fg(planet_color(star.planet.planet_type));
    w.write_str(&format!("{:<10} ", star.planet.planet_type.name()));

    match star.owner {
        Some(empire_id) => {
            if let Some(empire) = game.get_empire(empire_id) {
                w.set_fg(Color::Yellow);
                w.writeln(&empire.name);
            } else {
                w.set_fg(Color::DarkGray);
                w.writeln("Unknown");
            }
        }
        None => {
            w.set_fg(Color::DarkGray);
            w.writeln("-");
        }
    }
}

/// Render star system view
fn render_star_system(w: &mut AnsiWriter, flow: &MocFlow, star_id: u32) {
    render_header(w);

    let Some(game) = flow.game_state() else {
        w.set_fg(Color::LightRed);
        w.writeln("  No game loaded.");
        return;
    };

    let empire = flow.current_empire_id
        .and_then(|id| game.get_empire(id));

    render_status_bar(w, game, empire);

    let Some(star) = game.galaxy.get_star(star_id) else {
        w.set_fg(Color::LightRed);
        w.writeln("  Star not found.");
        return;
    };

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  STAR SYSTEM: {}", star.name));
    w.reset_color();
    w.writeln("");

    // Star ASCII art (simple)
    w.set_fg(Color::Yellow);
    w.writeln("            *");
    w.writeln("           ***");
    w.writeln("          *****");
    w.writeln("           ***");
    w.writeln("            *");
    w.reset_color();

    w.writeln("");

    // Planet info
    w.set_fg(Color::White);
    w.write_str("  Planet Type: ");
    w.set_fg(planet_color(star.planet.planet_type));
    w.writeln(star.planet.planet_type.name());

    w.set_fg(Color::White);
    w.write_str("  Max Population: ");
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("{}", star.planet.max_population));

    w.set_fg(Color::White);
    w.write_str("  Base Production: ");
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("{}", star.planet.base_production));

    if let Some(special) = &star.planet.special {
        w.set_fg(Color::White);
        w.write_str("  Special: ");
        w.set_fg(Color::LightMagenta);
        w.writeln(special.name());
    }

    // Owner info
    w.writeln("");
    w.set_fg(Color::White);
    w.write_str("  Owner: ");
    if let Some(owner_id) = star.owner {
        if let Some(owner) = game.get_empire(owner_id) {
            w.set_fg(Color::Yellow);
            w.writeln(&owner.name);

            // If we own this, show colony info
            if Some(owner_id) == flow.current_empire_id {
                if let Some(colony) = owner.colonies.iter().find(|c| c.star_id == star_id) {
                    w.writeln("");
                    render_colony_info(w, colony);
                }
            }
        }
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("Unclaimed");
    }

    // Fleets at this location
    let fleets_here: Vec<_> = game.empires.iter()
        .flat_map(|e| e.fleets.iter())
        .filter(|f| f.location_star_id == star_id && !f.is_in_transit())
        .collect();

    if !fleets_here.is_empty() {
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln("  Fleets present:");
        for fleet in fleets_here {
            if let Some(owner) = game.get_empire(fleet.empire_id) {
                w.set_fg(Color::LightCyan);
                w.writeln(&format!("    - {} ({} ships) - {}", fleet.name, fleet.total_ships(), owner.name));
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [B] ");
    w.set_fg(Color::White);
    w.writeln("Build (if owned)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [C] ");
    w.set_fg(Color::White);
    w.writeln("Colonize (if unowned)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to galaxy map");

    w.writeln("");
    w.write_str("  > ");
}

fn render_colony_info(w: &mut AnsiWriter, colony: &ColonyState) {
    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("  COLONY STATUS");
    w.reset_color();

    w.set_fg(Color::White);
    w.writeln(&format!("    Population: {}/{}", colony.population, colony.max_population));
    w.writeln(&format!("    Production: {} per turn", colony.production_output()));
    w.writeln(&format!("    Research: {} per turn", colony.research_output()));

    w.writeln("");
    w.writeln("    Buildings:");
    for building in &colony.buildings {
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("      - {}", building));
    }

    if !colony.production_queue.is_empty() {
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln("    Production Queue:");
        for item in &colony.production_queue {
            w.set_fg(Color::Yellow);
            w.writeln(&format!("      > {}", item));
        }
    }
}

/// Render colony management
fn render_colony(w: &mut AnsiWriter, flow: &MocFlow, star_id: u32) {
    render_header(w);

    let Some(game) = flow.game_state() else { return; };
    let empire = flow.current_empire_id.and_then(|id| game.get_empire(id));
    render_status_bar(w, game, empire);

    let star = game.galaxy.get_star(star_id);
    let colony = empire.and_then(|e| e.colonies.iter().find(|c| c.star_id == star_id));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  COLONY: {}", star.map(|s| s.name.as_str()).unwrap_or("Unknown")));
    w.reset_color();

    if let Some(col) = colony {
        render_colony_info(w, col);
    } else {
        w.set_fg(Color::LightRed);
        w.writeln("  No colony at this location.");
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  BUILD OPTIONS");
    w.reset_color();

    w.set_fg(Color::LightCyan);
    w.write_str("    [1] ");
    w.set_fg(Color::White);
    w.writeln("Factory (100 prod)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [2] ");
    w.set_fg(Color::White);
    w.writeln("Research Lab (150 prod)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [3] ");
    w.set_fg(Color::White);
    w.writeln("Farm (80 prod)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [4] ");
    w.set_fg(Color::White);
    w.writeln("Shipyard (200 prod)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [5] ");
    w.set_fg(Color::White);
    w.writeln("Scout Ship (20 prod)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [6] ");
    w.set_fg(Color::White);
    w.writeln("Fighter (70 prod)");

    w.set_fg(Color::LightCyan);
    w.write_str("    [7] ");
    w.set_fg(Color::White);
    w.writeln("Colony Ship (100 prod)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to galaxy map");

    w.writeln("");
    w.write_str("  > ");
}

/// Render fleet management
fn render_fleet(w: &mut AnsiWriter, flow: &MocFlow, fleet_id: u32) {
    render_header(w);

    let Some(game) = flow.game_state() else { return; };
    let empire = flow.current_empire_id.and_then(|id| game.get_empire(id));
    render_status_bar(w, game, empire);

    let fleet = empire.and_then(|e| e.fleets.iter().find(|f| f.id == fleet_id));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    if let Some(f) = fleet {
        w.writeln(&format!("  FLEET: {}", f.name));
        w.reset_color();
        w.writeln("");

        w.set_fg(Color::White);
        w.write_str("  Location: ");
        if let Some(star) = game.galaxy.get_star(f.location_star_id) {
            w.set_fg(Color::LightCyan);
            w.writeln(&star.name);
        }

        if f.is_in_transit() {
            w.set_fg(Color::White);
            w.write_str("  Destination: ");
            if let Some(dest_id) = f.destination_star_id {
                if let Some(star) = game.galaxy.get_star(dest_id) {
                    w.set_fg(Color::Yellow);
                    w.writeln(&format!("{} (ETA: {} turns)", star.name, f.eta_turns));
                }
            }
        }

        w.writeln("");
        w.set_fg(Color::White);
        w.writeln("  Ships:");
        if let Some(emp) = empire {
            for (design_id, count) in &f.ships {
                if let Some(design) = emp.ship_designs.iter().find(|d| d.id == *design_id) {
                    w.set_fg(Color::LightCyan);
                    w.writeln(&format!("    {} x {}", count, design.name));
                }
            }
        }
    } else {
        w.writeln("  FLEET NOT FOUND");
    }
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Enter destination star ID to move fleet.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to galaxy map");

    w.writeln("");
    w.write_str("  > ");
}

/// Render research screen
fn render_research(w: &mut AnsiWriter, flow: &MocFlow) {
    render_header(w);

    let Some(game) = flow.game_state() else { return; };
    let empire = flow.current_empire_id.and_then(|id| game.get_empire(id));
    render_status_bar(w, game, empire);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  RESEARCH ALLOCATION");
    w.reset_color();
    w.writeln("");

    if let Some(emp) = empire {
        w.set_fg(Color::White);
        w.writeln(&format!("  Total Research Output: {} points/turn", emp.research_output()));
        w.writeln("");

        for (i, field) in TechField::all().iter().enumerate() {
            let level = emp.research.level(*field);
            let alloc = emp.research.allocation_for(*field);
            let points = emp.research.points_in_field(*field);

            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<14} ", field.name()));
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("Lv{} ", level));
            w.set_fg(Color::Yellow);
            w.write_str(&format!("({}%) ", alloc));
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("[{} pts]", points));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [B] ");
    w.set_fg(Color::White);
    w.writeln("Balanced allocation");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to galaxy map");

    w.writeln("");
    w.write_str("  > ");
}

/// Render ship designer
fn render_ship_designer(w: &mut AnsiWriter, flow: &MocFlow) {
    render_header(w);

    let Some(game) = flow.game_state() else { return; };
    let empire = flow.current_empire_id.and_then(|id| game.get_empire(id));
    render_status_bar(w, game, empire);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SHIP DESIGNER");
    w.reset_color();
    w.writeln("");

    if let Some(emp) = empire {
        w.set_fg(Color::White);
        w.writeln("  Current Designs:");
        w.writeln("");

        for design in &emp.ship_designs {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    {} ", design.name));
            w.set_fg(Color::DarkGray);
            w.write_str(&format!("[{}] ", design.hull.name()));
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("ATK:{} ", design.attack_power));
            w.set_fg(Color::LightBlue);
            w.write_str(&format!("DEF:{} ", design.defense));
            w.set_fg(Color::Yellow);
            w.writeln(&format!("SPD:{}", design.speed));
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  (Full ship designer coming soon)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to galaxy map");

    w.writeln("");
    w.write_str("  > ");
}

/// Render turn summary
fn render_turn_summary(w: &mut AnsiWriter, flow: &MocFlow) {
    render_header(w);

    let Some(game) = flow.game_state() else { return; };
    let empire = flow.current_empire_id.and_then(|id| game.get_empire(id));
    render_status_bar(w, game, empire);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  END OF TURN {} SUMMARY", game.turn_number));
    w.reset_color();
    w.writeln("");

    if let Some(emp) = empire {
        w.set_fg(Color::White);
        w.writeln(&format!("  Colonies: {}", emp.colonies.len()));
        w.writeln(&format!("  Total Population: {}", emp.total_population()));
        w.writeln(&format!("  Production: {} per turn", emp.production_output()));
        w.writeln(&format!("  Research: {} per turn", emp.research_output()));
        w.writeln(&format!("  Fleets: {}", emp.fleets.len()));
    }

    if let Some(orders) = &flow.draft_orders {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln("  Pending Orders:");
        w.writeln(&format!("    Colony orders: {}", orders.colony_orders.len()));
        w.writeln(&format!("    Fleet orders: {}", orders.fleet_orders.len()));
    }

    w.writeln("");
    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("  Submit turn orders?");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [S/Y] ");
    w.set_fg(Color::White);
    w.writeln("Submit and end turn");

    w.set_fg(Color::LightCyan);
    w.write_str("    [N/Q] ");
    w.set_fg(Color::White);
    w.writeln("Return to game");

    w.writeln("");
    w.write_str("  > ");
}

/// Render game over screen
fn render_game_over(w: &mut AnsiWriter, flow: &MocFlow) {
    w.clear_screen();

    let Some(game) = flow.game_state() else { return; };

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  ╔═══════════════════════════════════════════════════════════════╗");
    w.writeln("  ║                                                               ║");
    w.writeln("  ║                    G A M E   O V E R                          ║");
    w.writeln("  ║                                                               ║");
    w.writeln("  ╚═══════════════════════════════════════════════════════════════╝");
    w.reset_color();

    w.writeln("");

    if let Some(winner_id) = game.winner_empire_id {
        if let Some(winner) = game.get_empire(winner_id) {
            w.set_fg(Color::LightGreen);
            w.bold();
            w.writeln(&format!("  VICTORY: {}", winner.name));
            w.reset_color();

            w.writeln("");
            w.set_fg(Color::White);
            if let Some(victory_type) = &game.victory_type {
                w.writeln(&format!("  Victory Type: {:?}", victory_type));
            }
        }
    } else {
        w.set_fg(Color::LightRed);
        w.writeln("  No winner - all human players left the game.");
    }

    w.writeln("");
    w.writeln(&format!("  Total Turns: {}", game.turn_number));
    w.writeln(&format!("  Empires: {}", game.empires.len()));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to exit...");
    w.reset_color();
}

/// Render lobby screen
fn render_lobby(w: &mut AnsiWriter, flow: &MocFlow) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  GAME LOBBY");
    w.reset_color();
    w.writeln("");

    if let Some(game) = flow.game_state() {
        w.set_fg(Color::White);
        w.writeln(&format!("  Game: {}", game.name));
        w.writeln(&format!("  Players: {}/{}", game.empires.len(), game.settings.max_players));
        w.writeln("");

        w.set_fg(Color::LightCyan);
        w.writeln("  Joined Empires:");
        for empire in &game.empires {
            w.set_fg(Color::White);
            w.write_str(&format!("    - {} ", empire.name));
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("({})", empire.race_key));
        }

        if game.empires.len() >= 2 {
            w.writeln("");
            w.set_fg(Color::LightGreen);
            w.writeln("  Ready to start!");
        }
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("  No game in progress.");
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [N] ");
    w.set_fg(Color::White);
    w.writeln("New Game");

    w.set_fg(Color::LightCyan);
    w.write_str("    [J] ");
    w.set_fg(Color::White);
    w.writeln("Join Existing Game");

    if flow.game_state().map(|g| g.empires.len() >= 2).unwrap_or(false) {
        w.set_fg(Color::LightCyan);
        w.write_str("    [S] ");
        w.set_fg(Color::White);
        w.writeln("Start Game");
    }

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit to BBS");

    w.writeln("");
    w.write_str("  > ");
}

/// Render new game creation
fn render_new_game(w: &mut AnsiWriter, step: u32) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CREATE NEW GAME");
    w.reset_color();
    w.writeln("");

    match step {
        0 => {
            w.set_fg(Color::White);
            w.writeln("  Enter game name:");
            w.writeln("");
            w.write_str("  > ");
        }
        1 => {
            w.set_fg(Color::White);
            w.writeln("  Enter your empire name:");
            w.writeln("");
            w.write_str("  > ");
        }
        _ => {
            w.set_fg(Color::LightGreen);
            w.writeln("  Game created! Returning to lobby...");
        }
    }
}

/// Render join game screen
fn render_join_game(w: &mut AnsiWriter, flow: &MocFlow) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  JOIN EXISTING GAME");
    w.reset_color();
    w.writeln("");

    if flow.available_games.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No games available to join.");
    } else {
        for (i, (id, name, players, max)) in flow.available_games.iter().enumerate() {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(name);
            w.set_fg(Color::DarkGray);
            w.writeln(&format!(" ({}/{} players) [ID: {}]", players, max, id));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to lobby");

    w.writeln("");
    w.write_str("  > ");
}

/// Render settings screen
fn render_settings(w: &mut AnsiWriter) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SETTINGS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("  (Settings coming soon)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.writeln("");
    w.write_str("  > ");
}

/// Render quit confirmation
fn render_confirm_quit(w: &mut AnsiWriter) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Your game will be saved and you can resume later.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_intro() {
        let flow = MocFlow::new(1);
        let output = render_screen(&flow);
        assert!(output.contains("MASTER"));
        assert!(output.contains("ANDROMEDA"));
    }

    #[test]
    fn test_render_lobby() {
        let mut flow = MocFlow::new(1);
        flow.screen = GameScreen::Lobby;
        let output = render_screen(&flow);
        assert!(output.contains("LOBBY"));
    }
}
