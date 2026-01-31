//! Star Trader - ANSI Rendering
//!
//! Space-themed visual identity with cool sci-fi palette (cyans, blues, greens).

use crate::terminal::{AnsiWriter, Color};
use super::state::GameState;
use super::screen::{GameScreen, TradeMode, StarDockArea, StarTraderFlow};
use super::galaxy::{Galaxy, SectorTypeData, PortData};
use super::data::{Commodity, TradeDirection, SHIP_CLASSES, config};
use super::combat::Opponent;

/// Format credits with thousands separators
pub fn format_credits(amount: i64) -> String {
    let sign = if amount < 0 { "-" } else { "" };
    let abs = amount.abs();

    let s = format!("{}", abs);
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }

    format!("{}{}cr", sign, result)
}

/// Render header with Star Trader logo
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("");
    w.writeln("   _____ _______       _____    _______ _____            _____  ______ _____  ");
    w.writeln("  / ____|__   __|/\\   |  __ \\  |__   __|  __ \\     /\\   |  __ \\|  ____|  __ \\ ");
    w.set_fg(Color::Cyan);
    w.writeln(" | (___    | |  /  \\  | |__) |    | |  | |__) |   /  \\  | |  | | |__  | |__) |");
    w.writeln("  \\___ \\   | | / /\\ \\ |  _  /     | |  |  _  /   / /\\ \\ | |  | |  __| |  _  / ");
    w.set_fg(Color::Blue);
    w.writeln("  ____) |  | |/ ____ \\| | \\ \\     | |  | | \\ \\  / ____ \\| |__| | |____| | \\ \\ ");
    w.writeln(" |_____/   |_/_/    \\_\\_|  \\_\\    |_|  |_|  \\_\\/_/    \\_\\_____/|______|_|  \\_\\");
    w.reset_color();
    w.writeln("");
}

/// Render status bar
pub fn render_status_bar(state: &GameState, galaxy: &Galaxy) -> String {
    let mut w = AnsiWriter::new();

    // Separator
    w.set_fg(Color::DarkGray);
    w.writeln(&"═".repeat(80));

    // Line 1: Location and turns
    let sector = galaxy.get_sector(state.sector);
    let sector_type = sector.map(|s| describe_sector_type(&s.sector_type)).unwrap_or("Unknown");

    w.set_fg(Color::LightCyan);
    w.write_str(&format!(" Sector {:>5}", state.sector));
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::White);
    w.write_str(sector_type);
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::LightGreen);
    w.write_str(&format!("Credits: {}", format_credits(state.credits)));
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("Turns: {}", state.turns_remaining));

    // Line 2: Ship status
    let ship = state.ship();
    let ship_name = ship.map(|s| s.name).unwrap_or("Unknown Ship");

    w.set_fg(Color::LightBlue);
    w.write_str(&format!(" Ship: {}", ship_name));
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::LightRed);
    w.write_str(&format!("Fighters: {}", state.fighters));
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::LightMagenta);
    w.write_str(&format!("Shields: {}", state.shields));
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("Cargo: {}/{}", state.cargo.total(), state.cargo_capacity()));

    // Separator
    w.set_fg(Color::DarkGray);
    w.writeln(&"═".repeat(80));
    w.reset_color();

    w.flush()
}

/// Describe sector type briefly
fn describe_sector_type(sector_type: &SectorTypeData) -> &'static str {
    match sector_type {
        SectorTypeData::Empty => "Empty Space",
        SectorTypeData::Port(_) => "Trading Port",
        SectorTypeData::Planet(_) => "Planet",
        SectorTypeData::StarDock => "StarDock",
        SectorTypeData::FerrengiSpace { .. } => "Ferrengi Space!",
        SectorTypeData::Nebula => "Nebula",
        SectorTypeData::Asteroid { .. } => "Asteroid Field",
    }
}

/// Render intro screen
pub fn render_intro() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.set_fg(Color::LightGray);
    w.writeln("  In the vastness of space, fortunes are made and lost among the stars.");
    w.writeln("");
    w.writeln("  You are a trader, starting with nothing but a worn-out Merchant Cruiser");
    w.writeln("  and a dream of galactic domination. The galaxy awaits - thousands of");
    w.writeln("  sectors filled with trading ports, colonizable planets, and danger.");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  Trade commodities between ports. Build your fleet. Form corporations.");
    w.writeln("  Battle the Ferrengi. Conquer the universe.");
    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln("  But beware - the Ferrengi control vast regions of space, and other");
    w.writeln("  traders compete for the same riches.");
    w.writeln("");
    w.reset_color();
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to begin your journey...");
    w.reset_color();

    w.flush()
}

/// Render main menu
pub fn render_main_menu(state: &GameState, galaxy: &Galaxy) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Command Console:");
    w.reset_color();
    w.writeln("");

    // Show warps from current sector
    if let Some(sector) = galaxy.get_sector(state.sector) {
        w.set_fg(Color::DarkGray);
        w.write_str("  Warps: ");
        w.set_fg(Color::LightCyan);
        let warps: Vec<String> = sector.warps.iter().map(|w| w.to_string()).collect();
        w.writeln(&warps.join(", "));
        w.reset_color();
        w.writeln("");
    }

    // Navigation
    w.set_fg(Color::LightCyan);
    w.write_str("    [M] ");
    w.set_fg(Color::White);
    w.writeln("Move/Navigate");

    // Trade (if at port or stardock)
    if let Some(sector) = galaxy.get_sector(state.sector) {
        match &sector.sector_type {
            SectorTypeData::Port(_) | SectorTypeData::StarDock => {
                w.set_fg(Color::LightCyan);
                w.write_str("    [T] ");
                w.set_fg(Color::White);
                w.writeln("Trade at Port");

                w.set_fg(Color::LightCyan);
                w.write_str("    [P] ");
                w.set_fg(Color::White);
                w.writeln("Port Information");
            }
            SectorTypeData::Planet(planet) => {
                w.set_fg(Color::LightCyan);
                w.write_str("    [L] ");
                w.set_fg(Color::White);
                w.writeln(&format!("Land on {}", planet.name));
            }
            _ => {}
        }

        if matches!(sector.sector_type, SectorTypeData::StarDock) {
            w.set_fg(Color::LightCyan);
            w.write_str("    [D] ");
            w.set_fg(Color::White);
            w.writeln("Dock at StarDock");
        }
    }

    w.set_fg(Color::LightCyan);
    w.write_str("    [S] ");
    w.set_fg(Color::White);
    w.writeln("Scanner");

    w.set_fg(Color::LightCyan);
    w.write_str("    [C] ");
    w.set_fg(Color::White);
    w.writeln("Corporation");

    w.set_fg(Color::LightCyan);
    w.write_str("    [I] ");
    w.set_fg(Color::White);
    w.writeln("Ship Info & Stats");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Save & Quit");

    w.reset_color();
    w.writeln("");
    w.write_str("  Command> ");

    w.flush()
}

/// Render navigation screen
pub fn render_navigation(state: &GameState, galaxy: &Galaxy, target: Option<u32>) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Navigation Computer");
    w.reset_color();
    w.writeln("");

    // Current sector info
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("  Current Sector: {}", state.sector));
    w.reset_color();

    // Show adjacent sectors
    if let Some(sector) = galaxy.get_sector(state.sector) {
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln("  Adjacent Sectors (direct warp):");
        w.writeln("");

        for (i, &warp) in sector.warps.iter().enumerate() {
            let dest_sector = galaxy.get_sector(warp);
            let dest_type = dest_sector.map(|s| describe_sector_type(&s.sector_type)).unwrap_or("Unknown");

            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("Sector {} - ", warp));
            w.set_fg(Color::LightGray);
            w.writeln(dest_type);
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Enter sector number for long-range navigation, or press [Q] to return.");
    w.reset_color();

    if let Some(dest) = target {
        w.writeln("");
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  Target: Sector {}. Press [W] to warp.", dest));
        w.reset_color();
    }

    w.writeln("");
    w.write_str("  Destination> ");

    w.flush()
}

/// Render trading screen
pub fn render_trading(state: &GameState, galaxy: &Galaxy, mode: &TradeMode) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Trading Terminal");
    w.reset_color();
    w.writeln("");

    // Get port data
    let port = if let Some(sector) = galaxy.get_sector(state.sector) {
        match &sector.sector_type {
            SectorTypeData::Port(p) => Some(p.clone()),
            _ => None,
        }
    } else {
        None
    };

    if let Some(ref port) = port {
        render_port_prices(&mut w, port);
    }

    // Show cargo
    w.writeln("");
    w.set_fg(Color::LightBlue);
    w.writeln("  Your Cargo:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Fuel Ore:  {}", state.cargo.fuel_ore));
    w.writeln(&format!("    Organics:  {}", state.cargo.organics));
    w.writeln(&format!("    Equipment: {}", state.cargo.equipment));
    w.writeln(&format!("    Space: {}/{}", state.cargo.total(), state.cargo_capacity()));
    w.reset_color();

    w.writeln("");

    match mode {
        TradeMode::Menu => {
            w.set_fg(Color::LightCyan);
            w.write_str("    [B] ");
            w.set_fg(Color::White);
            w.writeln("Buy from Port");

            w.set_fg(Color::LightCyan);
            w.write_str("    [S] ");
            w.set_fg(Color::White);
            w.writeln("Sell to Port");

            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Exit Trading");

            w.writeln("");
            w.write_str("  Action> ");
        }
        TradeMode::Buying { commodity: None } => {
            w.set_fg(Color::LightGreen);
            w.writeln("  Select commodity to BUY:");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [1] ");
            w.set_fg(Color::White);
            w.writeln("Fuel Ore");
            w.set_fg(Color::LightCyan);
            w.write_str("    [2] ");
            w.set_fg(Color::White);
            w.writeln("Organics");
            w.set_fg(Color::LightCyan);
            w.write_str("    [3] ");
            w.set_fg(Color::White);
            w.writeln("Equipment");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Cancel");
            w.writeln("");
            w.write_str("  Commodity> ");
        }
        TradeMode::Buying { commodity: Some(c) } => {
            w.set_fg(Color::LightGreen);
            w.writeln(&format!("  Buying {} - enter quantity:", c.name()));
            w.writeln("");
            w.write_str("  Quantity> ");
        }
        TradeMode::Selling { commodity: None } => {
            w.set_fg(Color::LightRed);
            w.writeln("  Select commodity to SELL:");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [1] ");
            w.set_fg(Color::White);
            w.writeln("Fuel Ore");
            w.set_fg(Color::LightCyan);
            w.write_str("    [2] ");
            w.set_fg(Color::White);
            w.writeln("Organics");
            w.set_fg(Color::LightCyan);
            w.write_str("    [3] ");
            w.set_fg(Color::White);
            w.writeln("Equipment");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Cancel");
            w.writeln("");
            w.write_str("  Commodity> ");
        }
        TradeMode::Selling { commodity: Some(c) } => {
            w.set_fg(Color::LightRed);
            w.writeln(&format!("  Selling {} - enter quantity:", c.name()));
            w.writeln("");
            w.write_str("  Quantity> ");
        }
    }

    w.reset_color();
    w.flush()
}

/// Render port prices table
fn render_port_prices(w: &mut AnsiWriter, port: &PortData) {
    let pt = port.port_type.to_port_type();

    w.set_fg(Color::LightCyan);
    w.writeln(&format!("  Port: {} ({})", port.name, port.port_type.code()));
    w.reset_color();
    w.writeln("");

    // Header
    w.set_fg(Color::DarkGray);
    w.writeln("  ┌─────────────┬───────┬─────────┬─────────┐");
    w.write_str("  │ ");
    w.set_fg(Color::White);
    w.write_str("Commodity   ");
    w.set_fg(Color::DarkGray);
    w.write_str("│ ");
    w.set_fg(Color::White);
    w.write_str("Type  ");
    w.set_fg(Color::DarkGray);
    w.write_str("│ ");
    w.set_fg(Color::White);
    w.write_str("  Price ");
    w.set_fg(Color::DarkGray);
    w.write_str("│ ");
    w.set_fg(Color::White);
    w.write_str("  Stock ");
    w.set_fg(Color::DarkGray);
    w.writeln("│");
    w.writeln("  ├─────────────┼───────┼─────────┼─────────┤");

    // Fuel Ore
    let ore_dir = pt.direction_for(Commodity::FuelOre);
    render_commodity_row(w, "Fuel Ore", ore_dir, port.fuel_ore.price, port.fuel_ore.quantity);

    // Organics
    let org_dir = pt.direction_for(Commodity::Organics);
    render_commodity_row(w, "Organics", org_dir, port.organics.price, port.organics.quantity);

    // Equipment
    let equ_dir = pt.direction_for(Commodity::Equipment);
    render_commodity_row(w, "Equipment", equ_dir, port.equipment.price, port.equipment.quantity);

    w.set_fg(Color::DarkGray);
    w.writeln("  └─────────────┴───────┴─────────┴─────────┘");
    w.reset_color();
}

fn render_commodity_row(w: &mut AnsiWriter, name: &str, dir: TradeDirection, price: i64, stock: u32) {
    let dir_str = match dir {
        TradeDirection::Buying => "BUYS ",
        TradeDirection::Selling => "SELLS",
    };
    let dir_color = match dir {
        TradeDirection::Buying => Color::LightGreen,
        TradeDirection::Selling => Color::LightRed,
    };

    w.set_fg(Color::DarkGray);
    w.write_str("  │ ");
    w.set_fg(Color::White);
    w.write_str(&format!("{:<11} ", name));
    w.set_fg(Color::DarkGray);
    w.write_str("│ ");
    w.set_fg(dir_color);
    w.write_str(dir_str);
    w.set_fg(Color::DarkGray);
    w.write_str(" │ ");
    w.set_fg(Color::Yellow);
    w.write_str(&format!("{:>7} ", price));
    w.set_fg(Color::DarkGray);
    w.write_str("│ ");
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("{:>7} ", stock));
    w.set_fg(Color::DarkGray);
    w.writeln("│");
}

/// Render combat screen
pub fn render_combat(state: &GameState, galaxy: &Galaxy, opponent: &Opponent) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("  !! COMBAT ALERT !!");
    w.reset_color();
    w.writeln("");

    // Enemy info
    w.set_fg(Color::LightRed);
    w.writeln(&format!("  Enemy: {}", opponent.name()));
    w.set_fg(Color::White);
    w.writeln(&format!("    Fighters: {}", opponent.fighters()));
    w.writeln(&format!("    Shields:  {}", opponent.shields()));
    w.writeln("");

    // Your status
    w.set_fg(Color::LightCyan);
    w.writeln("  Your Ship:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Fighters: {}", state.fighters));
    w.writeln(&format!("    Shields:  {}", state.shields));
    w.writeln("");

    // Actions
    w.set_fg(Color::LightCyan);
    w.write_str("    [A] ");
    w.set_fg(Color::White);
    w.writeln("Attack!");

    w.set_fg(Color::LightCyan);
    w.write_str("    [R] ");
    w.set_fg(Color::White);
    w.writeln("Attempt to Run");

    w.set_fg(Color::LightCyan);
    w.write_str("    [S] ");
    w.set_fg(Color::White);
    w.writeln("Surrender");

    w.reset_color();
    w.writeln("");
    w.write_str("  Action> ");

    w.flush()
}

/// Render StarDock screen
pub fn render_stardock(state: &GameState, galaxy: &Galaxy, area: &StarDockArea) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("  ╔══════════════════════════════════════════════════════════════════════════╗");
    w.writeln("  ║                          FEDERATION STARDOCK                             ║");
    w.writeln("  ╚══════════════════════════════════════════════════════════════════════════╝");
    w.reset_color();
    w.writeln("");

    match area {
        StarDockArea::MainMenu => {
            w.set_fg(Color::White);
            w.writeln("  Welcome to the Federation StarDock, the center of galactic commerce.");
            w.writeln("");

            w.set_fg(Color::LightCyan);
            w.write_str("    [1] ");
            w.set_fg(Color::White);
            w.writeln("Ship Dealership");

            w.set_fg(Color::LightCyan);
            w.write_str("    [2] ");
            w.set_fg(Color::White);
            w.writeln("Hardware Emporium");

            w.set_fg(Color::LightCyan);
            w.write_str("    [3] ");
            w.set_fg(Color::White);
            w.writeln("Federation Headquarters");

            w.set_fg(Color::LightCyan);
            w.write_str("    [4] ");
            w.set_fg(Color::White);
            w.writeln("Corporate Headquarters");

            w.set_fg(Color::LightCyan);
            w.write_str("    [5] ");
            w.set_fg(Color::White);
            w.writeln("Bank");

            w.set_fg(Color::LightCyan);
            w.write_str("    [T] ");
            w.set_fg(Color::White);
            w.writeln("Trade at Port");

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Undock");

            w.writeln("");
            w.write_str("  Selection> ");
        }
        StarDockArea::ShipDealer => {
            w.set_fg(Color::Yellow);
            w.writeln("  Ship Dealership - Trade in your vessel for something better!");
            w.writeln("");

            for (i, ship) in SHIP_CLASSES.iter().enumerate() {
                let current = state.ship_class == ship.key;
                let can_afford = state.credits >= ship.price;
                let has_commission = !ship.requires_commission || state.federation_commission;

                if current {
                    w.set_fg(Color::LightGreen);
                    w.writeln(&format!("    [{}] {} (CURRENT)", i + 1, ship.name));
                } else if !has_commission {
                    w.set_fg(Color::DarkGray);
                    w.writeln(&format!("    [{}] {} - {} (Requires Commission)",
                        i + 1, ship.name, format_credits(ship.price)));
                } else if !can_afford {
                    w.set_fg(Color::LightRed);
                    w.writeln(&format!("    [{}] {} - {}",
                        i + 1, ship.name, format_credits(ship.price)));
                } else {
                    w.set_fg(Color::LightCyan);
                    w.write_str(&format!("    [{}] ", i + 1));
                    w.set_fg(Color::White);
                    w.writeln(&format!("{} - {} (Cargo:{} Fighters:{} Shields:{})",
                        ship.name, format_credits(ship.price),
                        ship.cargo_holds, ship.max_fighters, ship.max_shields));
                }
            }

            w.reset_color();
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Back");
            w.writeln("");
            w.write_str("  Selection> ");
        }
        StarDockArea::HardwareEmporium => {
            w.set_fg(Color::Yellow);
            w.writeln("  Hardware Emporium - Arm your vessel!");
            w.writeln("");

            let ship = state.ship();
            let max_fighters = ship.map(|s| s.max_fighters).unwrap_or(0);
            let max_shields = ship.map(|s| s.max_shields).unwrap_or(0);

            w.set_fg(Color::White);
            w.writeln(&format!("  Fighters: {}/{} ({}cr each)",
                state.fighters, max_fighters, config::FIGHTER_COST));
            w.writeln(&format!("  Shields:  {}/{} ({}cr each)",
                state.shields, max_shields, config::SHIELD_COST));
            w.writeln("");

            w.set_fg(Color::LightCyan);
            w.write_str("    [F] ");
            w.set_fg(Color::White);
            w.writeln("Buy 10 Fighters");

            w.set_fg(Color::LightCyan);
            w.write_str("    [S] ");
            w.set_fg(Color::White);
            w.writeln("Buy 10 Shields");

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Back");
            w.writeln("");
            w.write_str("  Selection> ");
        }
        StarDockArea::FederationHQ => {
            w.set_fg(Color::Yellow);
            w.writeln("  Federation Headquarters");
            w.writeln("");

            w.set_fg(Color::White);
            w.writeln(&format!("  Rank: {}", state.federation_rank.name()));
            w.writeln(&format!("  Experience: {}", state.experience));
            w.writeln(&format!("  Alignment: {}", state.alignment));
            w.writeln("");

            if state.federation_commission {
                w.set_fg(Color::LightGreen);
                w.writeln("  You hold a Federation Commission.");
            } else {
                w.set_fg(Color::LightCyan);
                w.write_str("    [C] ");
                w.set_fg(Color::White);
                w.writeln("Apply for Commission (500,000cr, 10,000 XP required)");
            }

            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Back");
            w.writeln("");
            w.write_str("  Selection> ");
        }
        _ => {
            w.set_fg(Color::White);
            w.writeln("  Coming soon...");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("    [Q] ");
            w.set_fg(Color::White);
            w.writeln("Back");
            w.writeln("");
            w.write_str("  Selection> ");
        }
    }

    w.reset_color();
    w.flush()
}

/// Render scanner screen
pub fn render_scanner(state: &GameState, galaxy: &Galaxy) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Long Range Scanner");
    w.reset_color();
    w.writeln("");

    let ship = state.ship();
    let range = ship.map(|s| s.scanner_range).unwrap_or(1);

    let scanned = galaxy.scan_sectors(state.sector, range);

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  Scanning {} sectors within range {}...", scanned.len(), range));
    w.writeln("");

    for sector_id in scanned.iter().take(10) {
        if let Some(sector) = galaxy.get_sector(*sector_id) {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("  Sector {:>5}: ", sector_id));
            w.set_fg(Color::White);
            w.writeln(describe_sector_type(&sector.sector_type));
        }
    }

    if scanned.len() > 10 {
        w.set_fg(Color::DarkGray);
        w.writeln(&format!("  ... and {} more sectors", scanned.len() - 10));
    }

    w.reset_color();
    w.writeln("");
    w.writeln("  Press any key to return...");

    w.flush()
}

/// Render stats screen
pub fn render_stats(state: &GameState, galaxy: &Galaxy) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    w.write_str(&render_status_bar(state, galaxy));

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Player Statistics");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln(&format!("  Handle: {}", state.handle));
    w.writeln(&format!("  Ship: {} (\"{}\")",
        state.ship().map(|s| s.name).unwrap_or("Unknown"),
        state.ship_name));
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln(&format!("  Rank: {}", state.federation_rank.name()));
    w.writeln(&format!("  Experience: {}", state.experience));
    w.writeln(&format!("  Alignment: {}", state.alignment));
    w.writeln(&format!("  Kills: {}", state.kills));
    w.writeln(&format!("  Deaths: {}", state.deaths));
    w.writeln("");

    w.set_fg(Color::LightGreen);
    w.writeln("  Statistics:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Sectors Explored: {}", state.stats.sectors_explored));
    w.writeln(&format!("    Trades Completed: {}", state.stats.trades_completed));
    w.writeln(&format!("    Total Trade Value: {}", format_credits(state.stats.total_traded_value)));
    w.writeln(&format!("    Ferrengi Destroyed: {}", state.stats.ferrengi_destroyed));
    w.writeln(&format!("    Planets Colonized: {}", state.stats.planets_colonized));
    w.writeln(&format!("    Max Credits Held: {}", format_credits(state.stats.max_credits_held)));

    w.reset_color();
    w.writeln("");
    w.writeln("  Press any key to return...");

    w.flush()
}

/// Render confirm quit screen
pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  Are you sure you want to quit?");
    w.writeln("  Your progress will be saved.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Y] ");
    w.set_fg(Color::White);
    w.writeln("Yes, quit");
    w.set_fg(Color::LightCyan);
    w.write_str("    [N] ");
    w.set_fg(Color::White);
    w.writeln("No, return to game");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
    w.flush()
}

/// Render screen based on current state
pub fn render_screen(flow: &StarTraderFlow) -> String {
    let state = flow.game_state();
    let galaxy = flow.galaxy();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(),
        GameScreen::MainMenu => render_main_menu(state, galaxy),
        GameScreen::Navigation { target_sector } => render_navigation(state, galaxy, *target_sector),
        GameScreen::Trading { mode } => render_trading(state, galaxy, mode),
        GameScreen::Combat { opponent } => render_combat(state, galaxy, opponent),
        GameScreen::PortInfo => render_main_menu(state, galaxy), // Simplified
        GameScreen::Planet => render_main_menu(state, galaxy), // Simplified
        GameScreen::StarDock { area } => render_stardock(state, galaxy, area),
        GameScreen::Corporation => render_stats(state, galaxy), // Simplified
        GameScreen::Stats => render_stats(state, galaxy),
        GameScreen::Scanner => render_scanner(state, galaxy),
        GameScreen::GameOver => render_confirm_quit(), // Simplified
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_credits() {
        assert_eq!(format_credits(1000), "1,000cr");
        assert_eq!(format_credits(1000000), "1,000,000cr");
        assert_eq!(format_credits(-500), "-500cr");
    }

    #[test]
    fn test_render_intro() {
        let output = render_intro();
        // The header is ASCII art that spells STAR TRADER, not literal text
        // Verify key intro text elements are present
        assert!(output.contains("vastness of space"));
        assert!(output.contains("trader"));
        assert!(output.contains("Press any key"));
    }

    #[test]
    fn test_render_confirm_quit() {
        let output = render_confirm_quit();
        assert!(output.contains("quit"));
        assert!(output.contains("[Y]"));
        assert!(output.contains("[N]"));
    }
}
