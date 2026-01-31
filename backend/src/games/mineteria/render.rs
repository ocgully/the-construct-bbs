//! ANSI rendering for Mineteria
//!
//! Renders the game world and UI with a blocky/pixel visual identity.

use crate::terminal::{AnsiWriter, Color};
use super::data::{BlockType, is_daytime, time_of_day_name, RECIPES, CraftingStation};
use super::state::GameState;
use super::screen::{GameScreen, MineteriaFlow};
use super::crafting::{can_craft, has_crafting_station, format_recipe};
use super::combat::ActiveMonster;

// View dimensions
const VIEW_WIDTH: i32 = 40;
const VIEW_HEIGHT: i32 = 16;

// ============================================================================
// COLOR HELPERS
// ============================================================================

/// Get color for a block type
fn block_color(block: BlockType) -> Color {
    match block {
        BlockType::Air => Color::Black,
        BlockType::Dirt => Color::Brown,
        BlockType::Grass => Color::Green,
        BlockType::Stone => Color::LightGray,
        BlockType::Sand => Color::Yellow,
        BlockType::Gravel => Color::DarkGray,
        BlockType::Clay => Color::Brown,
        BlockType::Snow => Color::White,
        BlockType::Ice => Color::LightCyan,
        BlockType::CoalOre => Color::DarkGray,
        BlockType::IronOre => Color::Brown,
        BlockType::GoldOre => Color::Yellow,
        BlockType::DiamondOre => Color::LightCyan,
        BlockType::CopperOre => Color::Brown,
        BlockType::Wood => Color::Brown,
        BlockType::Leaves => Color::Green,
        BlockType::Cactus => Color::LightGreen,
        BlockType::Planks => Color::Brown,
        BlockType::CobbleStone => Color::LightGray,
        BlockType::StoneBrick => Color::LightGray,
        BlockType::Torch => Color::Yellow,
        BlockType::Workbench => Color::Brown,
        BlockType::Furnace => Color::DarkGray,
        BlockType::Chest => Color::Brown,
        BlockType::Door => Color::Brown,
        BlockType::Ladder => Color::Brown,
        BlockType::Bedrock => Color::DarkGray,
        BlockType::Water => Color::Blue,
        BlockType::Lava => Color::LightRed,
    }
}

/// Get bright variant of color for highlighting
fn bright_color(color: Color) -> Color {
    match color {
        Color::Brown => Color::Yellow,
        Color::Green => Color::LightGreen,
        Color::Blue => Color::LightBlue,
        Color::Red => Color::LightRed,
        Color::DarkGray => Color::LightGray,
        Color::LightGray => Color::White,
        c => c,
    }
}

// ============================================================================
// HEADER RENDERING
// ============================================================================

fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("");
    w.writeln("  ███╗   ███╗██╗███╗   ██╗███████╗████████╗███████╗██████╗ ██╗ █████╗ ");
    w.writeln("  ████╗ ████║██║████╗  ██║██╔════╝╚══██╔══╝██╔════╝██╔══██╗██║██╔══██╗");
    w.writeln("  ██╔████╔██║██║██╔██╗ ██║█████╗     ██║   █████╗  ██████╔╝██║███████║");
    w.writeln("  ██║╚██╔╝██║██║██║╚██╗██║██╔══╝     ██║   ██╔══╝  ██╔══██╗██║██╔══██║");
    w.writeln("  ██║ ╚═╝ ██║██║██║ ╚████║███████╗   ██║   ███████╗██║  ██║██║██║  ██║");
    w.writeln("  ╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝╚═╝  ╚═╝");
    w.reset_color();
}

// ============================================================================
// STATUS BAR
// ============================================================================

fn render_status_bar(w: &mut AnsiWriter, state: &GameState) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(80));

    // Line 1: Health, Hunger, Day/Time
    w.set_fg(Color::LightRed);
    w.write_str(&format!(" HP: {}/{}", state.health, state.max_health));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Hunger color
    let hunger_color = if state.hunger > 14 {
        Color::LightGreen
    } else if state.hunger > 7 {
        Color::Yellow
    } else {
        Color::LightRed
    };
    w.set_fg(hunger_color);
    w.write_str(&format!("Hunger: {}/{}", state.hunger, state.max_hunger));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Day/time
    let is_day = is_daytime(state.world_tick);
    w.set_fg(if is_day { Color::Yellow } else { Color::LightBlue });
    w.write_str(&format!("Day {} - {}", state.day, time_of_day_name(state.world_tick)));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(Color::LightCyan);
    w.writeln(&format!("Lvl: {}", state.level));

    // Line 2: Position, Build mode
    w.set_fg(Color::LightGray);
    w.write_str(&format!(" Pos: ({}, {})", state.position.x, state.position.y));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    if state.build_mode {
        w.set_fg(Color::LightGreen);
        w.write_str("BUILD MODE");
        w.set_fg(Color::DarkGray);
        w.write_str(&format!(" Cursor: ({}, {})",
            state.cursor_offset.0, state.cursor_offset.1));
    } else {
        w.set_fg(Color::LightGray);
        w.write_str("Move Mode");
    }

    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(80));
    w.reset_color();
}

// ============================================================================
// WORLD VIEW
// ============================================================================

fn render_world_view(w: &mut AnsiWriter, flow: &MineteriaFlow) {
    let state = flow.game_state();
    let visible = flow.get_visible_area(VIEW_WIDTH, VIEW_HEIGHT);

    let half_w = VIEW_WIDTH / 2;
    let half_h = VIEW_HEIGHT / 2;

    // Sky gradient based on time
    let is_day = is_daytime(state.world_tick);
    let sky_color = if is_day { Color::LightCyan } else { Color::Blue };

    for (row_idx, row) in visible.iter().enumerate() {
        w.write_str("  ");

        for (col_idx, &block) in row.iter().enumerate() {
            let world_x = state.position.x + (col_idx as i32) - half_w;
            let world_y = state.position.y - (row_idx as i32) + half_h;

            // Is this the player position?
            let is_player = world_x == state.position.x && world_y == state.position.y;

            // Is this the cursor position (in build mode)?
            let cursor_pos = state.cursor_position();
            let is_cursor = state.build_mode && world_x == cursor_pos.x && world_y == cursor_pos.y;

            if is_player {
                w.set_fg(Color::White);
                w.set_bg(Color::Blue);
                w.write_str("@");
                w.set_bg(Color::Black);
            } else if is_cursor {
                let block_info = block.get_block();
                w.set_fg(Color::White);
                w.set_bg(Color::Magenta);
                w.write_str(&block_info.char_display.to_string());
                w.set_bg(Color::Black);
            } else {
                let block_info = block.get_block();
                let color = block_color(block);

                // Air shows sky or cave
                if block == BlockType::Air {
                    if world_y > 60 {
                        w.set_fg(sky_color);
                        w.write_str(" ");
                    } else {
                        w.set_fg(Color::Black);
                        w.write_str(" ");
                    }
                } else {
                    w.set_fg(color);
                    w.write_str(&block_info.char_display.to_string());
                }
            }
        }

        w.reset_color();
        w.writeln("");
    }
}

// ============================================================================
// HOTBAR
// ============================================================================

fn render_hotbar(w: &mut AnsiWriter, state: &GameState) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(80));

    w.write_str("  ");

    for i in 0..9 {
        let is_selected = i == state.inventory.selected_slot as usize;

        if is_selected {
            w.set_fg(Color::White);
            w.write_str("[");
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str(" ");
        }

        // Slot number
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("{}", i + 1));

        // Item in slot
        if let Some(ref slot) = state.inventory.hotbar[i] {
            let item_info = slot.item.get_item();
            w.set_fg(Color::White);
            w.write_str(&format!(":{}{}", item_info.char_display, slot.count));

            // Durability for tools
            if let Some(ref tool) = slot.tool_data {
                if tool.durability < tool.max_durability / 4 {
                    w.set_fg(Color::LightRed);
                    w.write_str("!");
                }
            }
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str(":--");
        }

        if is_selected {
            w.set_fg(Color::White);
            w.write_str("]");
        } else {
            w.write_str(" ");
        }
    }

    w.writeln("");
    w.reset_color();
}

// ============================================================================
// SCREEN RENDERERS
// ============================================================================

pub fn render_intro() -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Welcome to Mineteria - a 2D sandbox adventure!");
    w.writeln("");
    w.writeln("  Dig deep, craft tools, build shelter, and survive.");
    w.writeln("  Explore caves, find rare ores, and battle monsters.");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  A procedurally generated world awaits...");
    w.writeln("");
    w.reset_color();
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to begin your adventure...");
    w.reset_color();

    w.flush()
}

pub fn render_playing(flow: &MineteriaFlow) -> String {
    let mut w = AnsiWriter::new();
    let state = flow.game_state();

    w.clear_screen();

    // Status bar
    render_status_bar(&mut w, state);

    // World view
    render_world_view(&mut w, flow);

    // Hotbar
    render_hotbar(&mut w, state);

    // Message
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    // Controls hint
    w.set_fg(Color::DarkGray);
    w.writeln("  WASD:Move  M:Mine  P:Place  B:Build  I:Inv  C:Craft  F:Eat  ?:Help  Q:Quit");
    w.reset_color();

    w.flush()
}

pub fn render_inventory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("");
    w.writeln("  INVENTORY");
    w.reset_color();

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(60));

    // Hotbar
    w.set_fg(Color::Yellow);
    w.writeln("  Hotbar:");
    for i in 0..9 {
        if let Some(ref slot) = state.inventory.hotbar[i] {
            let item = slot.item.get_item();
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20} x{}", item.name, slot.count));
            if let Some(ref tool) = slot.tool_data {
                w.set_fg(Color::DarkGray);
                w.write_str(&format!(" ({}/{})", tool.durability, tool.max_durability));
            }
            w.writeln("");
        }
    }

    // Main inventory
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(60));
    w.set_fg(Color::Yellow);
    w.writeln("  Main Inventory:");

    for i in 0..27 {
        if let Some(ref slot) = state.inventory.main[i] {
            let item = slot.item.get_item();
            w.set_fg(Color::White);
            w.writeln(&format!("    {:<20} x{}", item.name, slot.count));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [Q] Back");
    w.reset_color();

    w.flush()
}

pub fn render_crafting(state: &GameState, station: CraftingStation) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("");

    let station_name = match station {
        CraftingStation::Hand => "HAND CRAFTING",
        CraftingStation::Workbench => "WORKBENCH",
        CraftingStation::Furnace => "FURNACE",
        CraftingStation::Anvil => "ANVIL",
    };
    w.writeln(&format!("  {}", station_name));
    w.reset_color();

    // Station tabs
    w.set_fg(Color::DarkGray);
    w.write_str("  [H]and  ");
    if has_crafting_station(state, CraftingStation::Workbench) {
        w.set_fg(Color::LightGray);
    } else {
        w.set_fg(Color::DarkGray);
    }
    w.write_str("[W]orkbench  ");
    if has_crafting_station(state, CraftingStation::Furnace) {
        w.set_fg(Color::LightGray);
    } else {
        w.set_fg(Color::DarkGray);
    }
    w.writeln("[F]urnace");

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(70));

    // Recipes for this station
    let recipes: Vec<_> = RECIPES
        .iter()
        .filter(|r| r.station == station)
        .collect();

    if recipes.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("  No recipes available.");
    } else {
        for (i, recipe) in recipes.iter().enumerate() {
            let craftable = can_craft(state, recipe);

            if craftable {
                w.set_fg(Color::LightGreen);
            } else {
                w.set_fg(Color::DarkGray);
            }

            w.write_str(&format!("  [{:2}] ", i + 1));
            w.writeln(&format_recipe(recipe));
        }
    }

    // Message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Enter number to craft, [Q] to close");
    w.reset_color();

    w.flush()
}

pub fn render_combat(state: &GameState, monster: &ActiveMonster) -> String {
    let mut w = AnsiWriter::new();

    let stats = monster.get_stats();

    w.clear_screen();
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln(&format!("  COMBAT - {}", stats.name));
    w.reset_color();

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(50));

    // Monster art (simple ASCII)
    w.set_fg(Color::LightRed);
    match monster.monster_type {
        super::data::MonsterType::Zombie => {
            w.writeln("       .--.");
            w.writeln("      (o  o)");
            w.writeln("      |\\__/|");
            w.writeln("       \\__/");
        }
        super::data::MonsterType::Skeleton => {
            w.writeln("       .--.");
            w.writeln("      (o  o)");
            w.writeln("      |||||");
            w.writeln("       |||");
        }
        super::data::MonsterType::Spider => {
            w.writeln("     /\\_/\\");
            w.writeln("    ( o.o )");
            w.writeln("    />o<\\");
        }
        super::data::MonsterType::Slime => {
            w.writeln("      ___");
            w.writeln("     (   )");
            w.writeln("      ~~~");
        }
        super::data::MonsterType::Creeper => {
            w.writeln("      .--.");
            w.writeln("     |o  o|");
            w.writeln("     |_/\\_|");
            w.writeln("      \\SSS/");
        }
        _ => {
            w.writeln("      ???");
        }
    }
    w.reset_color();

    // Stats comparison
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  {} HP: {}/{}",
        stats.name, monster.health, monster.max_health));
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("  Your HP: {}/{}", state.health, state.max_health));

    // Message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(50));
    w.set_fg(Color::LightCyan);
    w.writeln("  [A]ttack  [R]un");
    w.reset_color();

    w.flush()
}

pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("");
    w.writeln("  PLAYER STATS");
    w.reset_color();

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(40));

    w.set_fg(Color::White);
    w.writeln(&format!("  Level: {}", state.level));
    w.writeln(&format!("  Experience: {}/{}", state.experience, state.level * 100));
    w.writeln(&format!("  Days Survived: {}", state.day));
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  Lifetime Stats:");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("    Blocks Mined: {}", state.stats.blocks_mined));
    w.writeln(&format!("    Blocks Placed: {}", state.stats.blocks_placed));
    w.writeln(&format!("    Monsters Killed: {}", state.stats.monsters_killed));
    w.writeln(&format!("    Deaths: {}", state.stats.deaths));
    w.writeln(&format!("    Distance Walked: {}", state.stats.distance_walked));
    w.writeln(&format!("    Deepest Depth: {}", state.stats.depth_reached.abs()));
    w.writeln(&format!("    Items Crafted: {}", state.stats.items_crafted));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

pub fn render_help() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("");
    w.writeln("  MINETERIA - HELP");
    w.reset_color();

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(60));

    w.set_fg(Color::Yellow);
    w.writeln("  Movement:");
    w.set_fg(Color::White);
    w.writeln("    W/A/S/D or H/J/K/L - Move around");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  Actions:");
    w.set_fg(Color::White);
    w.writeln("    M or SPACE - Mine block at cursor");
    w.writeln("    P          - Place block at cursor");
    w.writeln("    B          - Toggle build mode (move cursor)");
    w.writeln("    E          - Interact (open chests)");
    w.writeln("    F          - Eat food from hotbar");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  Screens:");
    w.set_fg(Color::White);
    w.writeln("    I - Open inventory");
    w.writeln("    C - Open crafting");
    w.writeln("    Y - View stats");
    w.writeln("    Q - Save and quit");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  Hotbar:");
    w.set_fg(Color::White);
    w.writeln("    1-9 - Select hotbar slot");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  Build Mode:");
    w.set_fg(Color::White);
    w.writeln("    Numpad 8/4/6/2 - Move cursor");
    w.writeln("    Cursor range: 4 blocks from player");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

pub fn render_game_over(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::LightRed);
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
    w.set_fg(Color::Yellow);
    w.writeln("  Final Stats:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Days Survived: {}", state.day));
    w.writeln(&format!("    Level Reached: {}", state.level));
    w.writeln(&format!("    Blocks Mined: {}", state.stats.blocks_mined));
    w.writeln(&format!("    Monsters Slain: {}", state.stats.monsters_killed));

    let score = (state.stats.blocks_mined as i64 * 10)
        + (state.stats.monsters_killed as i64 * 50)
        + (state.day as i64 * 100);

    w.writeln("");
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("    FINAL SCORE: {}", score));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();

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

    w.flush()
}

/// Render current screen based on game state
pub fn render_screen(flow: &MineteriaFlow) -> String {
    match flow.current_screen() {
        GameScreen::Intro => render_intro(),
        GameScreen::Playing => render_playing(flow),
        GameScreen::Inventory => render_inventory(flow.game_state()),
        GameScreen::Crafting { station } => render_crafting(flow.game_state(), *station),
        GameScreen::Combat { monster } => render_combat(flow.game_state(), monster),
        GameScreen::ChestView { .. } => render_inventory(flow.game_state()), // Simplified
        GameScreen::Stats => render_stats(flow.game_state()),
        GameScreen::Help => render_help(),
        GameScreen::GameOver => render_game_over(flow.game_state()),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_intro() {
        let output = render_intro();
        assert!(output.contains("Mineteria"));
        assert!(output.contains("Press any key"));
    }

    #[test]
    fn test_render_playing() {
        let flow = MineteriaFlow::new(12345);
        let output = render_playing(&flow);
        assert!(output.contains("@")); // Player character
    }

    #[test]
    fn test_render_help() {
        let output = render_help();
        assert!(output.contains("Movement"));
        assert!(output.contains("W/A/S/D"));
    }

    #[test]
    fn test_block_colors() {
        // Ensure all block types have colors
        let blocks = vec![
            BlockType::Air,
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::DiamondOre,
            BlockType::Lava,
        ];

        for block in blocks {
            let _ = block_color(block); // Should not panic
        }
    }
}
