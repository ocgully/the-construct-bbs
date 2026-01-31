//! ANSI rendering for Fortress
//!
//! Unique ASCII visual identity with underground dungeon aesthetic.

use crate::terminal::{AnsiWriter, Color};
use super::state::GameState;
use super::terrain::{TileType, OreType};
use super::dwarves::{DwarfStatus, DwarfMood};
use super::data::{WORKSHOPS, get_workshop_recipes};

/// Fortress color palette - earthy underground tones
mod colors {
    use crate::terminal::Color;

    pub const STONE: Color = Color::LightGray;
    pub const DIRT: Color = Color::Brown;
    pub const ORE_IRON: Color = Color::Cyan;
    pub const ORE_GOLD: Color = Color::Yellow;
    pub const ORE_COPPER: Color = Color::Brown;
    pub const GEM: Color = Color::LightMagenta;
    pub const WATER: Color = Color::LightBlue;
    pub const LAVA: Color = Color::LightRed;
    pub const TREE: Color = Color::Green;
    pub const GRASS: Color = Color::LightGreen;
    pub const FLOOR: Color = Color::DarkGray;
    pub const DWARF: Color = Color::LightCyan;
    pub const ENEMY: Color = Color::LightRed;
    pub const WORKSHOP: Color = Color::Yellow;
    pub const TITLE: Color = Color::Brown;
    pub const HIGHLIGHT: Color = Color::Yellow;
    pub const GOOD: Color = Color::LightGreen;
    pub const BAD: Color = Color::LightRed;
    pub const INFO: Color = Color::LightCyan;
}

/// Render the game header
fn render_header(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();
    w.set_fg(colors::TITLE);
    w.bold();
    w.writeln("");
    w.writeln("  ███████╗ ██████╗ ██████╗ ████████╗██████╗ ███████╗███████╗███████╗");
    w.writeln("  ██╔════╝██╔═══██╗██╔══██╗╚══██╔══╝██╔══██╗██╔════╝██╔════╝██╔════╝");
    w.writeln("  █████╗  ██║   ██║██████╔╝   ██║   ██████╔╝█████╗  ███████╗███████╗");
    w.writeln("  ██╔══╝  ██║   ██║██╔══██╗   ██║   ██╔══██╗██╔══╝  ╚════██║╚════██║");
    w.writeln("  ██║     ╚██████╔╝██║  ██║   ██║   ██║  ██║███████╗███████║███████║");
    w.writeln("  ╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝");
    w.reset_color();

    // Status bar
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    w.set_fg(colors::INFO);
    w.write_str(&format!(" {}", state.fortress_name));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(Color::White);
    w.write_str(&state.date_string());
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(colors::DWARF);
    w.write_str(&format!("{} Dwarves", state.living_dwarves().len()));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(colors::HIGHLIGHT);
    w.write_str(&format!("Wealth: {}", format_number(state.fortress_value())));

    if state.under_siege() {
        w.set_fg(Color::DarkGray);
        w.write_str(" | ");
        w.set_fg(colors::BAD);
        w.bold();
        w.write_str("UNDER SIEGE!");
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));
    w.reset_color();
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Render intro screen
pub fn render_intro(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Strike the earth!");
    w.writeln("");
    w.writeln("  You are the overseer of a dwarven expedition.");
    w.writeln("  Lead your seven brave dwarves to carve out a");
    w.writeln("  fortress from the living rock.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Manage your dwarves' needs: hunger, thirst, rest.");
    w.writeln("  Assign jobs: mining, crafting, farming, fighting.");
    w.writeln("  Build workshops and production chains.");
    w.writeln("  Defend against goblins and worse.");
    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  Press any key to begin...");
    w.reset_color();

    w.flush()
}

/// Render fortress naming screen
pub fn render_naming(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("  NAME YOUR FORTRESS");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  What shall this fortress be called?");
    w.writeln("  (3-20 characters)");
    w.writeln("");

    if let Some(ref msg) = state.last_message {
        w.set_fg(colors::BAD);
        w.writeln(&format!("  {}", msg));
        w.writeln("");
    }

    w.set_fg(Color::White);
    w.write_str("  > ");

    w.flush()
}

/// Render main fortress view with terrain
pub fn render_fortress_view(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    // Compact header for map view
    w.clear_screen();
    w.set_fg(colors::TITLE);
    w.bold();
    w.write_str(&format!(" {} ", state.fortress_name));
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.write_str("| ");
    w.set_fg(Color::White);
    w.write_str(&state.date_string());
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(colors::DWARF);
    w.write_str(&format!("{} Dwarves", state.living_dwarves().len()));
    w.set_fg(Color::DarkGray);
    w.write_str(" | Z-Level: ");
    w.set_fg(Color::White);
    w.write_str(&format!("{}", state.view_z));

    if state.under_siege() {
        w.set_fg(Color::DarkGray);
        w.write_str(" | ");
        w.set_fg(colors::BAD);
        w.bold();
        w.write_str("SIEGE!");
        w.reset_color();
    }
    w.writeln("");

    // Resource bar
    w.set_fg(Color::DarkGray);
    w.write_str(" ");
    w.set_fg(colors::GOOD);
    w.write_str(&format!("Food:{} ", state.resources.meal + state.resources.meat));
    w.set_fg(colors::WATER);
    w.write_str(&format!("Drink:{} ", state.resources.ale + state.resources.water));
    w.set_fg(colors::STONE);
    w.write_str(&format!("Stone:{} ", state.resources.stone));
    w.set_fg(Color::Brown);
    w.write_str(&format!("Wood:{} ", state.resources.wood));
    w.set_fg(colors::ORE_IRON);
    w.write_str(&format!("Iron:{} ", state.resources.iron));
    w.set_fg(colors::ORE_GOLD);
    w.write_str(&format!("Gold:{}", state.resources.gold));
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    // Render terrain (20 rows visible)
    let view_width = 78;
    let view_height = 12;

    for dy in 0..view_height {
        let y = state.view_y.saturating_sub(view_height / 2) + dy;
        w.write_str(" ");

        for dx in 0..view_width {
            let x = state.view_x.saturating_sub(view_width / 2) + dx;

            // Check for dwarf at this position
            let dwarf_here = state.dwarves.iter()
                .find(|d| d.x == x && d.y == y && d.z == state.view_z && d.status != DwarfStatus::Dead);

            if let Some(dwarf) = dwarf_here {
                let color = match dwarf.status {
                    DwarfStatus::Working => colors::DWARF,
                    DwarfStatus::Fighting => colors::BAD,
                    DwarfStatus::Sleeping => Color::DarkGray,
                    _ => colors::DWARF,
                };
                w.set_fg(color);
                w.write_str("\u{263A}"); // Smiley face for dwarf
                continue;
            }

            // Check for enemy
            let enemy_here = state.invasions.iter()
                .flat_map(|i| i.enemies.iter())
                .find(|e| e.x == x && e.y == y && e.z == state.view_z && e.health > 0);

            if enemy_here.is_some() {
                w.set_fg(colors::ENEMY);
                w.write_str("g"); // Goblin
                continue;
            }

            // Render tile
            if let Some(tile) = state.terrain.get(x, y, state.view_z) {
                let (ch, color) = get_tile_display(tile.tile_type, tile.designated);
                w.set_fg(color);
                w.write_str(&ch.to_string());
            } else {
                w.set_fg(Color::Black);
                w.write_str(" ");
            }
        }
        w.writeln("");
    }

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    // Show last message
    if let Some(ref msg) = state.last_message {
        w.set_fg(colors::HIGHLIGHT);
        w.writeln(&format!(" >> {} <<", msg));
    } else if !state.notifications.is_empty() {
        w.set_fg(colors::INFO);
        w.writeln(&format!(" {}", state.notifications.last().unwrap()));
    } else {
        w.writeln("");
    }

    // Menu hints
    w.set_fg(Color::DarkGray);
    w.write_str(" [");
    w.set_fg(colors::INFO);
    w.write_str("WASD");
    w.set_fg(Color::DarkGray);
    w.write_str("]Move [");
    w.set_fg(colors::INFO);
    w.write_str("<>");
    w.set_fg(Color::DarkGray);
    w.write_str("]Z-Level [");
    w.set_fg(colors::INFO);
    w.write_str("U");
    w.set_fg(Color::DarkGray);
    w.write_str("]Dwarves [");
    w.set_fg(colors::INFO);
    w.write_str("B");
    w.set_fg(Color::DarkGray);
    w.write_str("]Build [");
    w.set_fg(colors::INFO);
    w.write_str("Z");
    w.set_fg(Color::DarkGray);
    w.write_str("]Dig [");
    w.set_fg(colors::INFO);
    w.write_str("?");
    w.set_fg(Color::DarkGray);
    w.writeln("]Help");

    w.reset_color();
    w.flush()
}

fn get_tile_display(tile_type: TileType, designated: bool) -> (char, Color) {
    if designated {
        return ('X', colors::HIGHLIGHT);
    }

    match tile_type {
        TileType::Empty => (' ', Color::Black),
        TileType::Soil => ('.', colors::DIRT),
        TileType::Stone => ('#', colors::STONE),
        TileType::Ore(OreType::Iron) => ('%', colors::ORE_IRON),
        TileType::Ore(OreType::Copper) => ('%', colors::ORE_COPPER),
        TileType::Ore(OreType::Gold) => ('$', colors::ORE_GOLD),
        TileType::Ore(OreType::Silver) => ('%', Color::White),
        TileType::Ore(OreType::Coal) => ('%', Color::DarkGray),
        TileType::Gem => ('*', colors::GEM),
        TileType::Water => ('~', colors::WATER),
        TileType::Lava => ('&', colors::LAVA),
        TileType::Tree => ('T', colors::TREE),
        TileType::Grass => ('"', colors::GRASS),
        TileType::Shrub => (',', colors::GRASS),
        TileType::Floor => ('.', colors::FLOOR),
        TileType::Wall => ('#', colors::STONE),
        TileType::Door => ('+', colors::WORKSHOP),
        TileType::Stairs => ('X', colors::INFO),
        TileType::Ramp => ('/', colors::FLOOR),
        TileType::Stockpile => ('_', colors::HIGHLIGHT),
        TileType::Workshop => ('W', colors::WORKSHOP),
        TileType::Farm => ('~', colors::GOOD),
    }
}

/// Render dwarf list
pub fn render_dwarf_list(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("  DWARVES");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("     #  Name             Profession     Status       Mood");
    w.writeln(&format!("    {}", "\u{2500}".repeat(65)));
    w.reset_color();

    for (i, dwarf) in state.dwarves.iter().enumerate() {
        if dwarf.status == DwarfStatus::Dead {
            w.set_fg(Color::DarkGray);
        } else {
            w.set_fg(Color::White);
        }

        let status_color = match dwarf.status {
            DwarfStatus::Idle => Color::LightGray,
            DwarfStatus::Working => colors::INFO,
            DwarfStatus::Fighting => colors::BAD,
            DwarfStatus::Dead => Color::DarkGray,
            _ => Color::White,
        };

        let mood_color = match dwarf.mood {
            DwarfMood::Ecstatic | DwarfMood::Happy => colors::GOOD,
            DwarfMood::Content => Color::White,
            DwarfMood::Unhappy => colors::HIGHLIGHT,
            DwarfMood::Miserable | DwarfMood::Tantrum => colors::BAD,
        };

        w.set_fg(colors::INFO);
        w.write_str(&format!("    [{:2}] ", i + 1));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<16} ", dwarf.name));
        w.set_fg(Color::LightGray);
        w.write_str(&format!("{:<14} ", dwarf.profession));
        w.set_fg(status_color);
        w.write_str(&format!("{:<12} ", dwarf.status.description()));
        w.set_fg(mood_color);
        w.writeln(dwarf.mood.description());
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Select dwarf number for details, or [Q] to return.");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render individual dwarf detail
pub fn render_dwarf_detail(state: &GameState, dwarf_id: u32) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    let dwarf = match state.get_dwarf(dwarf_id) {
        Some(d) => d,
        None => {
            w.writeln("  Dwarf not found.");
            return w.flush();
        }
    };

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln(&format!("  {} - {}", dwarf.name, dwarf.profession));
    w.reset_color();
    w.writeln("");

    // Stats
    w.set_fg(Color::White);
    w.writeln(&format!("  Age: {}    Health: {}/{}", dwarf.age, dwarf.health, dwarf.max_health));
    w.writeln(&format!("  Status: {}    Mood: {}", dwarf.status.description(), dwarf.mood.description()));
    w.writeln("");

    // Needs
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  NEEDS:");
    w.reset_color();

    let needs = [
        ("Hunger", dwarf.needs.hunger),
        ("Thirst", dwarf.needs.thirst),
        ("Rest", dwarf.needs.rest),
        ("Social", dwarf.needs.social),
        ("Comfort", dwarf.needs.comfort),
    ];

    for (name, value) in &needs {
        let color = if *value > 70 { colors::GOOD }
        else if *value > 30 { colors::HIGHLIGHT }
        else { colors::BAD };

        w.set_fg(Color::LightGray);
        w.write_str(&format!("    {}: ", name));
        w.set_fg(color);
        w.writeln(&format!("{}", value));
    }

    w.writeln("");

    // Skills
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  SKILLS:");
    w.reset_color();

    let skills = [
        ("Mining", dwarf.skills.mining),
        ("Woodcutting", dwarf.skills.woodcutting),
        ("Farming", dwarf.skills.farming),
        ("Crafting", dwarf.skills.crafting),
        ("Combat", dwarf.skills.combat),
        ("Smithing", dwarf.skills.smithing),
    ];

    for (name, level) in &skills {
        if *level > 0 {
            w.set_fg(Color::LightGray);
            w.write_str(&format!("    {}: ", name));
            w.set_fg(colors::INFO);
            w.writeln(&format!("{}", level));
        }
    }

    w.writeln("");

    // Equipment
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  EQUIPMENT:");
    w.reset_color();

    w.set_fg(Color::LightGray);
    w.write_str("    Weapon: ");
    if let Some(ref weapon) = dwarf.equipped_weapon {
        w.set_fg(colors::INFO);
        w.writeln(weapon);
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("None");
    }

    w.set_fg(Color::LightGray);
    w.write_str("    Armor: ");
    if let Some(ref armor) = dwarf.equipped_armor {
        w.set_fg(colors::INFO);
        w.writeln(armor);
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("None");
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  [E] Equip Weapon  [A] Equip Armor  [Q] Back");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render workshop list
pub fn render_workshops(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("  WORKSHOPS");
    w.reset_color();
    w.writeln("");

    if state.workshops.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No workshops built yet.");
        w.writeln("  Use [B]uild menu to construct workshops.");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("     #  Type                    Location     Assigned");
        w.writeln(&format!("    {}", "\u{2500}".repeat(55)));
        w.reset_color();

        for (i, workshop) in state.workshops.iter().enumerate() {
            let def = super::data::get_workshop(&workshop.workshop_type);
            let name = def.map(|d| d.name).unwrap_or("Unknown");

            w.set_fg(colors::INFO);
            w.write_str(&format!("    [{:2}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<23} ", name));
            w.set_fg(Color::LightGray);
            w.write_str(&format!("({},{},{})  ", workshop.x, workshop.y, workshop.z));

            if let Some(dwarf_id) = workshop.assigned_dwarf {
                if let Some(dwarf) = state.get_dwarf(dwarf_id) {
                    w.set_fg(colors::DWARF);
                    w.writeln(&dwarf.name);
                } else {
                    w.writeln("");
                }
            } else {
                w.set_fg(Color::DarkGray);
                w.writeln("-");
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Select workshop number for recipes, [N] New workshop, [Q] Back");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render workshop detail with recipes
pub fn render_workshop_detail(state: &GameState, workshop_id: u32) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    let workshop = match state.get_workshop(workshop_id) {
        Some(ws) => ws,
        None => {
            w.writeln("  Workshop not found.");
            return w.flush();
        }
    };

    let def = super::data::get_workshop(&workshop.workshop_type);
    let name = def.map(|d| d.name).unwrap_or("Unknown");

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln(&format!("  {}", name));
    w.reset_color();
    w.writeln("");

    // Show available recipes
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  RECIPES:");
    w.reset_color();
    w.writeln("");

    let recipes = get_workshop_recipes(&workshop.workshop_type);

    for (i, recipe) in recipes.iter().enumerate() {
        w.set_fg(colors::INFO);
        w.write_str(&format!("    [{:2}] ", i + 1));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<20} ", recipe.name));

        // Show inputs
        w.set_fg(Color::DarkGray);
        w.write_str("Needs: ");
        w.set_fg(Color::LightGray);
        let inputs: Vec<String> = recipe.inputs.iter()
            .map(|(r, a)| format!("{}x{}", a, r))
            .collect();
        w.writeln(&inputs.join(", "));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Select recipe number to add work order, [Q] Back");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render stockpiles/resources
pub fn render_stockpiles(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("  STOCKPILES");
    w.reset_color();
    w.writeln("");

    // Raw materials
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  RAW MATERIALS:");
    w.reset_color();
    let raw = [
        ("Wood", state.resources.wood),
        ("Stone", state.resources.stone),
        ("Iron Ore", state.resources.iron_ore),
        ("Copper Ore", state.resources.copper_ore),
        ("Gold Ore", state.resources.gold_ore),
        ("Gems", state.resources.gem),
    ];
    render_resource_columns(&mut w, &raw);

    // Processed
    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  PROCESSED:");
    w.reset_color();
    let processed = [
        ("Iron", state.resources.iron),
        ("Copper", state.resources.copper),
        ("Gold", state.resources.gold),
        ("Cut Gems", state.resources.cut_gem),
        ("Cloth", state.resources.cloth),
        ("Leather", state.resources.leather),
    ];
    render_resource_columns(&mut w, &processed);

    // Food & Drink
    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  FOOD & DRINK:");
    w.reset_color();
    let food = [
        ("Meals", state.resources.meal),
        ("Meat", state.resources.meat),
        ("Vegetables", state.resources.vegetable),
        ("Ale", state.resources.ale),
        ("Wine", state.resources.wine),
        ("Water", state.resources.water),
    ];
    render_resource_columns(&mut w, &food);

    // Goods
    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  GOODS:");
    w.reset_color();
    let goods = [
        ("Furniture", state.resources.furniture),
        ("Tools", state.resources.tool),
        ("Weapons", state.resources.weapon),
        ("Armor", state.resources.armor),
        ("Crafts", state.resources.craft),
        ("Planks", state.resources.plank),
    ];
    render_resource_columns(&mut w, &goods);

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to return.");
    w.reset_color();

    w.flush()
}

fn render_resource_columns(w: &mut AnsiWriter, resources: &[(&str, u32)]) {
    for chunk in resources.chunks(3) {
        w.write_str("    ");
        for (name, amount) in chunk {
            w.set_fg(Color::LightGray);
            w.write_str(&format!("{}: ", name));
            let color = if *amount > 50 { colors::GOOD }
            else if *amount > 10 { Color::White }
            else { colors::BAD };
            w.set_fg(color);
            w.write_str(&format!("{:<8}", amount));
        }
        w.writeln("");
    }
}

/// Render build menu
pub fn render_build_menu(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("  BUILD WORKSHOP");
    w.reset_color();
    w.writeln("");

    if let Some(ref msg) = state.last_message {
        w.set_fg(colors::HIGHLIGHT);
        w.writeln(&format!("  {}", msg));
        w.writeln("");
    }

    w.set_fg(Color::DarkGray);
    w.writeln("     #  Workshop               Cost");
    w.writeln(&format!("    {}", "\u{2500}".repeat(50)));
    w.reset_color();

    for (i, workshop) in WORKSHOPS.iter().enumerate() {
        let can_afford = state.resources.wood >= workshop.build_cost.wood
            && state.resources.stone >= workshop.build_cost.stone
            && state.resources.iron >= workshop.build_cost.iron;

        if can_afford {
            w.set_fg(colors::INFO);
        } else {
            w.set_fg(Color::DarkGray);
        }

        w.write_str(&format!("    [{:2}] ", i + 1));
        w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
        w.write_str(&format!("{:<22} ", workshop.name));

        let mut costs = Vec::new();
        if workshop.build_cost.wood > 0 {
            costs.push(format!("{}W", workshop.build_cost.wood));
        }
        if workshop.build_cost.stone > 0 {
            costs.push(format!("{}S", workshop.build_cost.stone));
        }
        if workshop.build_cost.iron > 0 {
            costs.push(format!("{}I", workshop.build_cost.iron));
        }

        w.set_fg(Color::LightGray);
        w.writeln(&costs.join(" "));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Your resources: Wood:{} Stone:{} Iron:{}",
        state.resources.wood, state.resources.stone, state.resources.iron));
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Select number to build at cursor, [Q] Back");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render designate mode
pub fn render_designate(state: &GameState) -> String {
    // Similar to fortress view but with designation hints
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.write_str(" DESIGNATION MODE ");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.write_str(&format!("| Z-Level: {} | Cursor: ({},{})",
        state.view_z, state.view_x, state.view_y));
    w.writeln("");

    // Render terrain like fortress view
    let view_width = 78;
    let view_height = 14;

    for dy in 0..view_height {
        let y = state.view_y.saturating_sub(view_height / 2) + dy;
        w.write_str(" ");

        for dx in 0..view_width {
            let x = state.view_x.saturating_sub(view_width / 2) + dx;

            // Highlight cursor position
            let is_cursor = x == state.view_x && y == state.view_y;

            if let Some(tile) = state.terrain.get(x, y, state.view_z) {
                let (ch, color) = get_tile_display(tile.tile_type, tile.designated);

                if is_cursor {
                    w.set_bg(colors::HIGHLIGHT);
                    w.set_fg(Color::Black);
                } else {
                    w.set_fg(color);
                }
                w.write_str(&ch.to_string());
                if is_cursor {
                    w.reset_color();
                }
            } else {
                w.set_fg(Color::Black);
                w.write_str(" ");
            }
        }
        w.writeln("");
    }

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    // Current tile info
    if let Some(tile) = state.terrain.get(state.view_x, state.view_y, state.view_z) {
        let desc = match tile.tile_type {
            TileType::Stone => "Stone wall - can be mined",
            TileType::Ore(_) => "Ore deposit - valuable!",
            TileType::Gem => "Gem deposit!",
            TileType::Floor => "Open floor",
            TileType::Soil => "Soil - can be dug",
            _ => "",
        };
        w.set_fg(colors::INFO);
        w.writeln(&format!("  {}", desc));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str(" [");
    w.set_fg(colors::INFO);
    w.write_str("WASD");
    w.set_fg(Color::DarkGray);
    w.write_str("]Move [");
    w.set_fg(colors::INFO);
    w.write_str("<>");
    w.set_fg(Color::DarkGray);
    w.write_str("]Z-Level [");
    w.set_fg(colors::INFO);
    w.write_str("D/SPACE");
    w.set_fg(Color::DarkGray);
    w.write_str("]Designate [");
    w.set_fg(colors::INFO);
    w.write_str("Q");
    w.set_fg(Color::DarkGray);
    w.writeln("]Back");

    w.reset_color();
    w.flush()
}

/// Render statistics
pub fn render_statistics(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w, state);

    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("  FORTRESS STATISTICS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Tiles Mined:        {}", state.stats.tiles_mined));
    w.writeln(&format!("  Trees Chopped:      {}", state.stats.trees_chopped));
    w.writeln(&format!("  Items Crafted:      {}", state.stats.items_crafted));
    w.writeln(&format!("  Food Consumed:      {}", state.stats.food_consumed));
    w.writeln(&format!("  Drinks Consumed:    {}", state.stats.drinks_consumed));
    w.writeln("");
    w.writeln(&format!("  Invasions Repelled: {}", state.stats.invasions_repelled));
    w.writeln(&format!("  Enemies Slain:      {}", state.stats.enemies_slain));
    w.writeln(&format!("  Dwarves Lost:       {}", state.stats.dwarves_lost));
    w.writeln(&format!("  Peak Population:    {}", state.stats.peak_population));
    w.writeln("");
    w.set_fg(colors::HIGHLIGHT);
    w.writeln(&format!("  Total Wealth:       {}", format_number(state.fortress_value())));
    w.writeln(&format!("  Wealth Created:     {}", format_number(state.stats.wealth_created)));

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to return.");
    w.reset_color();

    w.flush()
}

/// Render help screen
pub fn render_help(_state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(colors::TITLE);
    w.bold();
    w.writeln("");
    w.writeln("  FORTRESS - HELP");
    w.reset_color();
    w.writeln("");

    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  CONTROLS:");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("    WASD / Arrow Keys - Move view");
    w.writeln("    < >               - Change Z-level (up/down)");
    w.writeln("");

    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  MENUS:");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("    U - Dwarves list");
    w.writeln("    B - Build workshops");
    w.writeln("    P - Workshop/Production");
    w.writeln("    Z - Designate digging");
    w.writeln("    I - Stockpiles (resources)");
    w.writeln("    M - Military");
    w.writeln("    T - Statistics");
    w.writeln("    Q - Save and Quit");
    w.writeln("");

    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  QUICK ACTIONS:");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("    F - Create farm plot at cursor");
    w.writeln("    K - Create stockpile at cursor");
    w.writeln("");

    w.set_fg(colors::HIGHLIGHT);
    w.writeln("  GAMEPLAY:");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("    - Keep dwarves fed and happy");
    w.writeln("    - Mine resources and craft goods");
    w.writeln("    - Build workshops for production");
    w.writeln("    - Train military for defense");
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to return.");
    w.reset_color();

    w.flush()
}

/// Render confirm quit
pub fn render_confirm_quit(_state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(colors::HIGHLIGHT);
    w.bold();
    w.writeln("");
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Your fortress will be saved and you can resume later.");
    w.writeln("  The simulation continues while you're away!");
    w.writeln("");
    w.set_fg(colors::INFO);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render game over
pub fn render_game_over(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(colors::BAD);
    w.bold();
    w.writeln("");
    w.writeln("   ██████╗  █████╗ ███╗   ███╗███████╗");
    w.writeln("  ██╔════╝ ██╔══██╗████╗ ████║██╔════╝");
    w.writeln("  ██║  ███╗███████║██╔████╔██║█████╗");
    w.writeln("  ██║   ██║██╔══██║██║╚██╔╝██║██╔══╝");
    w.writeln("  ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗");
    w.writeln("   ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝");
    w.writeln("");
    w.writeln("   ██████╗ ██╗   ██╗███████╗██████╗");
    w.writeln("  ██╔═══██╗██║   ██║██╔════╝██╔══██╗");
    w.writeln("  ██║   ██║██║   ██║█████╗  ██████╔╝");
    w.writeln("  ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗");
    w.writeln("  ╚██████╔╝ ╚████╔╝ ███████╗██║  ██║");
    w.writeln("   ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  {} has fallen.", state.fortress_name));
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  Final Wealth: {}", format_number(state.fortress_value())));
    w.writeln(&format!("  Years Survived: {}", state.year));
    w.writeln(&format!("  Peak Population: {}", state.stats.peak_population));

    w.writeln("");
    w.set_fg(colors::INFO);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_render_functions_no_panic() {
        let state = GameState::new("Test".to_string(), 42);

        // Just verify these don't panic
        render_intro(&state);
        render_naming(&state);
        render_fortress_view(&state);
        render_dwarf_list(&state);
        render_workshops(&state);
        render_stockpiles(&state);
        render_build_menu(&state);
        render_designate(&state);
        render_statistics(&state);
        render_confirm_quit(&state);
        render_game_over(&state);
    }

    #[test]
    fn test_tile_display() {
        let (ch, _) = get_tile_display(TileType::Stone, false);
        assert_eq!(ch, '#');

        let (ch, _) = get_tile_display(TileType::Floor, false);
        assert_eq!(ch, '.');

        let (ch, _) = get_tile_display(TileType::Stone, true);
        assert_eq!(ch, 'X'); // Designated
    }
}
