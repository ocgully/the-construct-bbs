//! ANSI rendering for Realm of Ralnar
//! Classic JRPG (Final Fantasy 1 style) aesthetic
//!
//! This module handles all screen rendering using the AnsiWriter pattern.

use crate::terminal::{AnsiWriter, Color};
use super::flow::{BattleState, RalnarFlow};
use super::screen::GameScreen;
use super::state::{CharacterClass, PartyMember};
use super::status::StatusEffect;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format gold with commas for display
pub fn format_gold(amount: u32) -> String {
    if amount == 0 {
        return "0".to_string();
    }

    let s = amount.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Render an HP bar with proportional fill
pub fn render_hp_bar(current: i32, max: i32, width: usize) -> String {
    if max <= 0 {
        return format!("[{}]", " ".repeat(width));
    }
    let current = current.max(0) as f32;
    let max = max as f32;
    let filled = ((current / max) * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

/// Render an EXP bar with proportional fill
pub fn render_exp_bar(current: u32, needed: u32, width: usize) -> String {
    if needed == 0 {
        return format!("[{}]", "=".repeat(width));
    }
    let pct = (current as f32 / needed as f32).min(1.0);
    let filled = (pct * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}

/// Render status effect icons for a character
pub fn render_status_icons(statuses: &[StatusEffect]) -> String {
    if statuses.is_empty() {
        return String::new();
    }

    let icons: Vec<&str> = statuses
        .iter()
        .map(|s| match s {
            StatusEffect::Poison => "PSN",
            StatusEffect::Stone => "STN",
            StatusEffect::Dead => "KO",
            StatusEffect::Sleep => "SLP",
            StatusEffect::Confused => "CNF",
            StatusEffect::Haste => "HST",
            StatusEffect::Protect => "PRT",
            StatusEffect::Shell => "SHL",
            StatusEffect::Silence => "SIL",
            StatusEffect::Blind => "BLD",
            StatusEffect::Slow => "SLW",
            StatusEffect::Regen => "RGN",
            StatusEffect::Berserk => "BSK",
        })
        .collect();

    format!(" [{}]", icons.join(","))
}

/// Get color for character class
fn class_color(class: CharacterClass) -> Color {
    match class {
        CharacterClass::Warrior => Color::LightRed,
        CharacterClass::Paladin => Color::Yellow,
        CharacterClass::Cleric => Color::White,
        CharacterClass::Knight => Color::LightGray,
        CharacterClass::Wizard => Color::LightMagenta,
        CharacterClass::Swashbuckler => Color::LightCyan,
        CharacterClass::Thief => Color::Green,
        CharacterClass::Sage => Color::LightBlue,
        CharacterClass::Archer => Color::Brown,
    }
}

/// Get HP color based on percentage
fn hp_color(current: i32, max: i32) -> Color {
    if max <= 0 {
        return Color::DarkGray;
    }
    let pct = (current * 100) / max;
    if pct > 70 {
        Color::LightGreen
    } else if pct > 30 {
        Color::Yellow
    } else if pct > 0 {
        Color::LightRed
    } else {
        Color::DarkGray
    }
}

// ============================================================================
// HEADER AND STATUS BAR
// ============================================================================

/// Render the game header with ASCII art title
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln(r"    ____            _                  __   ____        _                     ");
    w.writeln(r"   |  _ \ ___  __ _| |_ __ ___     ___| _| |  _ \ __ _| |_ __   __ _ _ __    ");
    w.writeln(r"   | |_) / _ \/ _` | | '_ ` _ \   / _ \| |  | |_) / _` | | '_ \ / _` | '__|   ");
    w.writeln(r"   |  _ <  __/ (_| | | | | | | | | (_) | |  |  _ < (_| | | | | | (_| | |      ");
    w.writeln(r"   |_| \_\___|\__,_|_|_| |_| |_|  \___/|_|  |_| \_\__,_|_|_| |_|\__,_|_|      ");
    w.set_fg(Color::Cyan);
    w.writeln("                        ~ A Classic JRPG Adventure ~                        ");
    w.reset_color();
}

/// Render a compact status bar showing party overview
fn render_status_bar(w: &mut AnsiWriter, flow: &RalnarFlow) {
    let state = flow.game_state();

    w.set_fg(Color::Brown);
    w.writeln(&"\u{2500}".repeat(78));

    // Line 1: Party HP overview
    w.set_fg(Color::White);
    w.write_str(" Party: ");

    for (i, member) in state.party.members.iter().enumerate() {
        if i > 0 {
            w.set_fg(Color::DarkGray);
            w.write_str(" | ");
        }
        w.set_fg(class_color(member.class));
        w.write_str(&member.name);
        w.set_fg(Color::DarkGray);
        w.write_str(" ");
        w.set_fg(hp_color(member.hp, member.hp_max));
        w.write_str(&format!("{}/{}", member.hp, member.hp_max));
    }
    w.writeln("");

    // Line 2: Gold, Location, Shrines
    w.set_fg(Color::Yellow);
    w.write_str(&format!(" Gold: {}", format_gold(state.gold)));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("Map: {}", state.current_map));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("Shrines: {}/5", state.shrines_destroyed_count()));

    w.set_fg(Color::Brown);
    w.writeln(&"\u{2500}".repeat(78));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTION
// ============================================================================

/// Main render function - dispatches to screen-specific renderers
pub fn render_screen(flow: &RalnarFlow) -> String {
    let mut w = AnsiWriter::new();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(&mut w),
        GameScreen::MainMenu => render_main_menu(&mut w, flow),
        GameScreen::Exploring { map_id } => render_exploring(&mut w, flow, map_id),
        GameScreen::Dialogue { npc_id } => render_dialogue(&mut w, flow, npc_id),
        GameScreen::Shop { shop_id } => render_shop(&mut w, flow, shop_id),
        GameScreen::Inn => render_inn(&mut w, flow),
        GameScreen::Battle => render_battle(&mut w, flow),
        GameScreen::BattleVictory => render_battle_victory(&mut w, flow),
        GameScreen::BattleDefeat => render_battle_defeat(&mut w, flow),
        GameScreen::Inventory => render_inventory(&mut w, flow),
        GameScreen::Equipment => render_equipment(&mut w, flow),
        GameScreen::PartyStatus => render_party_status(&mut w, flow),
        GameScreen::Magic => render_magic(&mut w, flow),
        GameScreen::QuestLog => render_quest_log(&mut w, flow),
        GameScreen::WorldMap => render_world_map(&mut w, flow),
        GameScreen::Cutscene { scene_id } => render_cutscene(&mut w, flow, scene_id),
        GameScreen::GameOver => render_game_over(&mut w, flow),
        GameScreen::Credits => render_credits(&mut w, flow),
        GameScreen::ConfirmQuit => render_confirm_quit(&mut w),
    }

    w.flush()
}

// ============================================================================
// SCREEN RENDERERS
// ============================================================================

fn render_intro(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln(r"    ____            _                  __   ____        _                     ");
    w.writeln(r"   |  _ \ ___  __ _| |_ __ ___     ___| _| |  _ \ __ _| |_ __   __ _ _ __    ");
    w.writeln(r"   | |_) / _ \/ _` | | '_ ` _ \   / _ \| |  | |_) / _` | | '_ \ / _` | '__|   ");
    w.writeln(r"   |  _ <  __/ (_| | | | | | | | | (_) | |  |  _ < (_| | | | | | (_| | |      ");
    w.writeln(r"   |_| \_\___|\__,_|_|_| |_| |_|  \___/|_|  |_| \_\__,_|_|_| |_|\__,_|_|      ");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::Cyan);
    w.writeln("    In the ancient land of Ralnar, darkness stirs once more...");
    w.writeln("");
    w.writeln("    You are Herbert, a young warrior who must lead a party");
    w.writeln("    to destroy the five elemental shrines and save the realm.");
    w.writeln("");
    w.reset_color();

    w.set_fg(Color::White);
    w.writeln("    Recruit companions, battle fearsome creatures,");
    w.writeln("    and uncover the mysteries of the Realm of Ralnar.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("    Press any key to begin your adventure...");
    w.reset_color();
}

fn render_main_menu(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();
    let player_name = state
        .party
        .leader()
        .map(|p| p.name.as_str())
        .unwrap_or("Adventurer");

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  Welcome back, {}!", player_name));
    w.writeln("");

    // Display current progress
    w.set_fg(Color::DarkGray);
    w.writeln(&format!(
        "  Party Level: {}  |  Shrines: {}/5  |  Play Time: {}",
        state.party.average_level(),
        state.shrines_destroyed_count(),
        state.formatted_play_time()
    ));
    w.writeln("");

    // Menu options
    w.set_fg(Color::LightCyan);
    w.write_str("  [N] ");
    w.set_fg(Color::White);
    w.writeln("New Game / Continue");

    w.set_fg(Color::LightCyan);
    w.write_str("  [C] ");
    w.set_fg(Color::White);
    w.writeln("Continue Adventure");

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit");

    w.writeln("");
    w.reset_color();
    w.write_str("  Your choice: ");
}

fn render_exploring(w: &mut AnsiWriter, flow: &RalnarFlow, map_id: &str) {
    render_header(w);
    render_status_bar(w, flow);

    let state = flow.game_state();

    // Show last message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.writeln("");
    }

    // Location name
    let location_name = match map_id {
        "starting_village" => "Thornhaven Village",
        "village_square" => "Village Square",
        "northern_path" => "Northern Path",
        "southern_woods" => "Southern Woods",
        "eastern_hills" => "Eastern Hills",
        "western_river" => "Western River",
        "fire_shrine" => "Shrine of Flames",
        "water_shrine" => "Shrine of Tides",
        "earth_shrine" => "Shrine of Stone",
        "wind_shrine" => "Shrine of Gales",
        "dark_shrine" => "Shrine of Shadow",
        _ => map_id,
    };

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  === {} ===", location_name.to_uppercase()));
    w.reset_color();
    w.writeln("");

    // Party display
    w.set_fg(Color::White);
    w.writeln("  Party:");
    for member in &state.party.members {
        let color = if member.is_alive() {
            class_color(member.class)
        } else {
            Color::DarkGray
        };
        w.set_fg(color);
        w.write_str(&format!("    {} ", member.name));
        w.set_fg(Color::DarkGray);
        w.write_str(&format!("({}) ", member.class.name()));
        w.set_fg(hp_color(member.hp, member.hp_max));
        w.write_str(&format!("HP: {}/{} ", member.hp, member.hp_max));
        w.set_fg(Color::LightBlue);
        w.writeln(&format!("MP: {}/{}", member.mp, member.mp_max));
    }
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln(&format!(
        "  Position: ({}, {})  Facing: {}",
        state.position.0,
        state.position.1,
        state.direction.name()
    ));
    w.writeln("");

    // Movement controls
    w.set_fg(Color::White);
    w.writeln("  Movement:");
    w.set_fg(Color::LightCyan);
    w.writeln("    [W] North  [S] South  [A] West  [D] East");
    w.writeln("");

    // Menu shortcuts
    w.set_fg(Color::White);
    w.writeln("  Menu:");
    w.set_fg(Color::LightCyan);
    w.write_str("    [I] Inventory  ");
    w.write_str("[E] Equipment  ");
    w.write_str("[P] Party  ");
    w.writeln("[M] Magic");

    w.set_fg(Color::LightCyan);
    w.write_str("    [J] Journal    ");
    w.write_str("[O] World Map  ");
    w.set_fg(Color::LightRed);
    w.writeln("[Q] Quit");

    w.writeln("");
    w.reset_color();
    w.write_str("  Command: ");
}

fn render_dialogue(w: &mut AnsiWriter, flow: &RalnarFlow, npc_id: &str) {
    w.clear_screen();

    let npc_name = match npc_id {
        "village_elder" => "Village Elder",
        "mysterious_stranger" => "Mysterious Stranger",
        "innkeeper" => "Innkeeper",
        "blacksmith" => "Master Blacksmith",
        "dorl" => "Dorl the Sage",
        _ => npc_id,
    };

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln(&format!("  === {} ===", npc_name));
    w.reset_color();
    w.writeln("");

    // Get dialogue text from flow state if available
    if let Some(dialogue) = flow.dialogue_state() {
        w.set_fg(Color::White);
        w.writeln(&format!("  \"{}\"", dialogue.current_text));
        w.writeln("");

        if !dialogue.choices.is_empty() {
            for (i, choice) in dialogue.choices.iter().enumerate() {
                w.set_fg(Color::LightCyan);
                w.write_str(&format!("  [{}] ", i + 1));
                w.set_fg(Color::White);
                w.writeln(choice);
            }
        }
    } else {
        w.set_fg(Color::White);
        w.writeln("  \"Greetings, traveler...\"");
        w.writeln("");

        w.set_fg(Color::LightCyan);
        w.write_str("  [1] ");
        w.set_fg(Color::White);
        w.writeln("Ask about the realm");

        w.set_fg(Color::LightCyan);
        w.write_str("  [2] ");
        w.set_fg(Color::White);
        w.writeln("Ask about the shrines");

        w.set_fg(Color::LightCyan);
        w.write_str("  [3] ");
        w.set_fg(Color::White);
        w.writeln("Trade");
    }

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_shop(w: &mut AnsiWriter, flow: &RalnarFlow, shop_id: &str) {
    render_header(w);

    let state = flow.game_state();

    let shop_name = match shop_id {
        "weapons" => "Thornhaven Armory",
        "armor" => "Iron Shield Outfitters",
        "items" => "Potion Emporium",
        "magic" => "Arcane Scrolls",
        _ => "General Store",
    };

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln(&format!("  === {} ===", shop_name.to_uppercase()));
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Your Gold: {}", format_gold(state.gold)));
    w.writeln("");

    // Show shop items (placeholder - would come from shop data)
    w.set_fg(Color::DarkGray);
    w.writeln("     #  Item                    Price");
    w.writeln(&format!("    {}", "\u{2500}".repeat(40)));
    w.reset_color();

    let items = [
        ("Potion", 10u32),
        ("Hi-Potion", 30),
        ("Antidote", 5),
        ("Phoenix Down", 100),
        ("Ether", 50),
    ];

    for (i, (name, price)) in items.iter().enumerate() {
        if state.gold >= *price {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    {} ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20}", name));
            w.set_fg(Color::Yellow);
            w.writeln(&format!("{:>8}g", price));
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str(&format!("    {} ", i + 1));
            w.write_str(&format!("{:<20}", name));
            w.set_fg(Color::Red);
            w.writeln(&format!("{:>8}g", price));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Enter number to buy, or:");
    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Leave shop");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_inn(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();

    w.set_fg(Color::Brown);
    w.bold();
    w.writeln("");
    w.writeln("  === THE WEARY TRAVELER INN ===");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  The fire crackles warmly. The smell of stew fills the air.");
    w.writeln("");

    // Show party health status
    w.set_fg(Color::White);
    w.writeln("  Party Status:");
    for member in &state.party.members {
        w.set_fg(class_color(member.class));
        w.write_str(&format!("    {} ", member.name));
        w.set_fg(hp_color(member.hp, member.hp_max));
        w.writeln(&format!(
            "{} {}/{}",
            render_hp_bar(member.hp, member.hp_max, 15),
            member.hp,
            member.hp_max
        ));
    }
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  \"Rest will cost 20 gold, and restore your party to full health.\"");
    w.writeln("");

    w.set_fg(Color::LightGreen);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.writeln(&format!(
        "Rest (20 gold) - You have {} gold",
        format_gold(state.gold)
    ));

    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_battle(w: &mut AnsiWriter, flow: &RalnarFlow) {
    w.clear_screen();

    let state = flow.game_state();

    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln("  ========== BATTLE! ==========");
    w.reset_color();
    w.writeln("");

    // Check if we have battle state
    if let Some(battle) = flow.battle_state() {
        render_battle_with_state(w, flow, battle);
    } else {
        // Fallback when no battle state
        w.set_fg(Color::LightGray);
        w.writeln("  A fearsome creature appears!");
        w.writeln("");

        // Show party HP
        w.set_fg(Color::White);
        w.writeln("  Party:");
        for member in &state.party.members {
            let status = if member.is_alive() { "" } else { " [KO]" };
            w.set_fg(class_color(member.class));
            w.write_str(&format!("    {} ", member.name));
            w.set_fg(hp_color(member.hp, member.hp_max));
            w.write_str(&format!("HP: {}/{} ", member.hp, member.hp_max));
            w.set_fg(Color::LightBlue);
            w.writeln(&format!("MP: {}/{}{}", member.mp, member.mp_max, status));
        }

        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.write_str("  [A] ");
        w.set_fg(Color::White);
        w.writeln("Attack");

        w.set_fg(Color::LightBlue);
        w.write_str("  [D] ");
        w.set_fg(Color::White);
        w.writeln("Defend");

        w.set_fg(Color::LightMagenta);
        w.write_str("  [M] ");
        w.set_fg(Color::White);
        w.writeln("Magic");

        w.set_fg(Color::LightCyan);
        w.write_str("  [I] ");
        w.set_fg(Color::White);
        w.writeln("Item");

        w.set_fg(Color::Yellow);
        w.write_str("  [R] ");
        w.set_fg(Color::White);
        w.writeln("Run");
    }

    w.writeln("");
    w.reset_color();
    w.write_str("  Action: ");
}

/// Helper function to render battle with flow's battle state
fn render_battle_with_state(w: &mut AnsiWriter, flow: &RalnarFlow, battle: &BattleState) {
    let state = flow.game_state();

    // Display enemies (just IDs since flow's BattleState is simpler)
    w.set_fg(Color::Red);
    w.writeln("  ENEMIES:");
    for (i, enemy_id) in battle.enemies.iter().enumerate() {
        w.set_fg(Color::LightRed);
        w.writeln(&format!("    {}) {}", (b'A' + i as u8) as char, enemy_id));
    }

    w.writeln("");
    w.set_fg(Color::Brown);
    w.writeln(&format!("  {}", "\u{2500}".repeat(50)));
    w.writeln("");

    // Display party from game state
    w.set_fg(Color::Green);
    w.writeln("  PARTY:");
    for (i, member) in state.party.members.iter().enumerate() {
        let status = if member.is_alive() { "" } else { " [KO]" };
        w.set_fg(class_color(member.class));
        w.write_str(&format!("    {}) {} ", i + 1, member.name));
        w.set_fg(hp_color(member.hp, member.hp_max));
        w.write_str(&format!(
            "HP {} {}/{} ",
            render_hp_bar(member.hp, member.hp_max, 10),
            member.hp,
            member.hp_max
        ));
        w.set_fg(Color::LightBlue);
        w.writeln(&format!("MP {}/{}{}", member.mp, member.mp_max, status));
    }

    // Combat messages (last 3)
    if !battle.messages.is_empty() {
        w.writeln("");
        w.set_fg(Color::DarkGray);
        for line in battle.messages.iter().rev().take(3) {
            w.writeln(&format!("    {}", line));
        }
    }

    w.writeln("");

    // Show turn info
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Turn: {}", battle.turn));
    w.writeln("");

    // Action menu
    w.set_fg(Color::LightCyan);
    w.write_str("  [A] ");
    w.set_fg(Color::White);
    w.writeln("Attack");

    w.set_fg(Color::LightMagenta);
    w.write_str("  [M] ");
    w.set_fg(Color::White);
    w.writeln("Magic");

    w.set_fg(Color::LightGreen);
    w.write_str("  [I] ");
    w.set_fg(Color::White);
    w.writeln("Item");

    w.set_fg(Color::LightBlue);
    w.write_str("  [D] ");
    w.set_fg(Color::White);
    w.writeln("Defend");

    w.set_fg(Color::Yellow);
    w.write_str("  [F] ");
    w.set_fg(Color::White);
    w.writeln("Flee");
}

fn render_battle_victory(w: &mut AnsiWriter, flow: &RalnarFlow) {
    w.clear_screen();

    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("");
    w.writeln("  ========================================");
    w.writeln("             V I C T O R Y !             ");
    w.writeln("  ========================================");
    w.reset_color();
    w.writeln("");

    let state = flow.game_state();

    w.set_fg(Color::White);
    w.writeln("  You have defeated the enemy!");
    w.writeln("");

    // Show party experience
    let total_exp: u32 = state.party.members.iter().map(|m| m.exp).sum();
    w.set_fg(Color::Cyan);
    w.writeln(&format!("  Total Party EXP: {}", total_exp));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_battle_defeat(w: &mut AnsiWriter, _flow: &RalnarFlow) {
    w.clear_screen();

    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln("  ========================================");
    w.writeln("             D E F E A T                 ");
    w.writeln("  ========================================");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Your party has fallen in battle...");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_inventory(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === INVENTORY ===");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Gold: {}", format_gold(state.gold)));
    w.writeln("");

    if state.inventory.items.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  Your inventory is empty.");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("     #  Item                    Qty");
        w.writeln(&format!("    {}", "\u{2500}".repeat(35)));
        w.reset_color();

        for (i, item) in state.inventory.items.iter().enumerate() {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    {} ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<22}", item.key));
            w.set_fg(Color::LightGray);
            w.writeln(&format!("x{}", item.quantity));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_equipment(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === EQUIPMENT ===");
    w.reset_color();
    w.writeln("");

    // Show equipment for each party member
    for member in &state.party.members {
        w.set_fg(class_color(member.class));
        w.bold();
        w.writeln(&format!(
            "  {} - {} Lv.{}",
            member.name,
            member.class.name(),
            member.level
        ));
        w.reset_color();

        let eq = &member.equipment;
        w.set_fg(Color::White);
        w.write_str("      Weapon: ");
        w.set_fg(Color::LightCyan);
        w.writeln(eq.weapon.as_deref().unwrap_or("(none)"));

        w.set_fg(Color::White);
        w.write_str("      Armor:  ");
        w.set_fg(Color::LightCyan);
        w.writeln(eq.armor.as_deref().unwrap_or("(none)"));

        w.set_fg(Color::White);
        w.write_str("      Shield: ");
        w.set_fg(Color::LightCyan);
        w.writeln(eq.shield.as_deref().unwrap_or("(none)"));

        w.set_fg(Color::White);
        w.write_str("      Helmet: ");
        w.set_fg(Color::LightCyan);
        w.writeln(eq.helmet.as_deref().unwrap_or("(none)"));

        w.set_fg(Color::White);
        w.write_str("      Accessory: ");
        w.set_fg(Color::LightCyan);
        w.writeln(eq.accessory.as_deref().unwrap_or("(none)"));

        w.writeln("");
    }

    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_party_status(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === PARTY STATUS ===");
    w.reset_color();
    w.writeln("");

    for member in &state.party.members {
        render_party_member_detailed(w, member);
        w.writeln("");
    }

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

/// Render detailed stats for a single party member
fn render_party_member_detailed(w: &mut AnsiWriter, member: &PartyMember) {
    // Header with name and class
    let brother_tag = if member.is_brother { " [Brother]" } else { "" };
    w.set_fg(class_color(member.class));
    w.bold();
    w.write_str(&format!("  {} ", member.name));
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln(&format!(
        "- {} Lv.{}{}",
        member.class.name(),
        member.level,
        brother_tag
    ));

    // HP and MP bars
    w.set_fg(Color::White);
    w.write_str("    HP: ");
    w.set_fg(hp_color(member.hp, member.hp_max));
    w.write_str(&render_hp_bar(member.hp, member.hp_max, 20));
    w.writeln(&format!(" {}/{}", member.hp, member.hp_max));

    w.set_fg(Color::White);
    w.write_str("    MP: ");
    w.set_fg(Color::LightBlue);
    w.write_str(&render_hp_bar(member.mp, member.mp_max, 20));
    w.writeln(&format!(" {}/{}", member.mp, member.mp_max));

    // EXP bar
    let exp_needed = member.exp_to_next_level();
    w.set_fg(Color::White);
    w.write_str("    EXP: ");
    w.set_fg(Color::Cyan);
    w.write_str(&render_exp_bar(member.exp, exp_needed, 20));
    w.writeln(&format!(" {}/{}", member.exp, exp_needed));

    // Stats
    w.set_fg(Color::LightGray);
    w.writeln(&format!(
        "    STR: {:>3}  AGI: {:>3}  INT: {:>3}  VIT: {:>3}  LCK: {:>3}",
        member.stats.strength,
        member.stats.agility,
        member.stats.intelligence,
        member.stats.vitality,
        member.stats.luck
    ));
}

fn render_magic(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();

    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("");
    w.writeln("  === MAGIC ===");
    w.reset_color();
    w.writeln("");

    // Find party members with MP
    for member in &state.party.members {
        if member.mp_max > 0 {
            w.set_fg(class_color(member.class));
            w.bold();
            w.writeln(&format!(
                "  {} (MP: {}/{})",
                member.name, member.mp, member.mp_max
            ));
            w.reset_color();

            // List spells based on class
            match member.class {
                CharacterClass::Wizard => {
                    w.set_fg(Color::LightCyan);
                    w.writeln("    - Fire (5 MP)");
                    w.writeln("    - Ice (5 MP)");
                    w.writeln("    - Thunder (8 MP)");
                }
                CharacterClass::Cleric => {
                    w.set_fg(Color::LightCyan);
                    w.writeln("    - Cure (4 MP)");
                    w.writeln("    - Protect (6 MP)");
                }
                CharacterClass::Paladin => {
                    w.set_fg(Color::LightCyan);
                    w.writeln("    - Cure (4 MP)");
                    w.writeln("    - Holy (10 MP)");
                }
                CharacterClass::Sage => {
                    w.set_fg(Color::LightCyan);
                    w.writeln("    - Fire (5 MP)");
                    w.writeln("    - Cure (4 MP)");
                    w.writeln("    - Esuna (8 MP)");
                }
                _ => {
                    w.set_fg(Color::DarkGray);
                    w.writeln("    (No spells available)");
                }
            }
            w.writeln("");
        }
    }

    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_quest_log(w: &mut AnsiWriter, flow: &RalnarFlow) {
    render_header(w);

    let state = flow.game_state();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === QUEST JOURNAL ===");
    w.reset_color();
    w.writeln("");

    // Main Quest Progress
    w.set_fg(Color::LightCyan);
    w.writeln("  MAIN QUEST: Destroy the Five Shrines");
    w.set_fg(Color::LightGray);
    w.writeln("  Destroy the elemental shrines to save the realm.");
    w.writeln("");

    let shrine_names = ["Fire", "Water", "Earth", "Wind", "Darkness"];
    for (i, name) in shrine_names.iter().enumerate() {
        if state.shrines_destroyed[i] {
            w.set_fg(Color::LightGreen);
            w.writeln(&format!("    [X] {} Shrine - DESTROYED", name));
        } else {
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("    [ ] {} Shrine - Active", name));
        }
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!(
        "  Progress: {}/5 shrines destroyed",
        state.shrines_destroyed_count()
    ));
    w.writeln(&format!("  World Phase: {}", state.world_phase));

    // Story flags
    if !state.story_flags.is_empty() {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln("  COMPLETED:");
        for flag in state.story_flags.keys() {
            w.set_fg(Color::LightGreen);
            w.writeln(&format!("    [X] {}", flag.replace('_', " ")));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_world_map(w: &mut AnsiWriter, _flow: &RalnarFlow) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === WORLD MAP OF RALNAR ===");
    w.reset_color();
    w.writeln("");

    // ASCII world map
    w.set_fg(Color::White);
    w.writeln("            [Fire Shrine]");
    w.writeln("                  |");
    w.writeln("      [Water] -- [Village] -- [Earth]");
    w.writeln("      Shrine        |        Shrine");
    w.writeln("              [Wind Shrine]");
    w.writeln("                    |");
    w.writeln("            [Darkness Shrine]");
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Travel using WASD in the exploration screen.");
    w.writeln("");

    w.set_fg(Color::LightRed);
    w.write_str("  [B] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

fn render_cutscene(w: &mut AnsiWriter, _flow: &RalnarFlow, scene_id: &str) {
    w.clear_screen();
    w.writeln("");

    match scene_id {
        "shrine_discovered" => {
            w.set_fg(Color::LightCyan);
            w.writeln("  An elemental shrine towers before you...");
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln("  Ancient power pulses from within.");
            w.writeln("  A Guardian awaits to test your resolve.");
        }
        "shrine_destroyed" => {
            w.set_fg(Color::Yellow);
            w.writeln("  The shrine crumbles!");
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln("  The elemental seal has been broken.");
            w.writeln("  The world shifts as balance is disrupted.");
        }
        "dorl_appears" => {
            w.set_fg(Color::LightMagenta);
            w.writeln("  A hooded figure emerges from the shadows...");
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln("  \"So... you have destroyed another shrine.\"");
            w.writeln("  \"You think you are saving the world?\"");
            w.writeln("  \"How delightfully naive.\"");
        }
        _ => {
            w.set_fg(Color::LightGray);
            w.writeln("  The story continues...");
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_game_over(w: &mut AnsiWriter, flow: &RalnarFlow) {
    w.clear_screen();

    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln(r"     ___    _    __  __ ___    _____   _____ ___ ");
    w.writeln(r"    / __|  /_\  |  \/  | __|  / _ \ \ / / __| _ \");
    w.writeln(r"   | (_ | / _ \ | |\/| | _|  | (_) \ V /| _||   /");
    w.writeln(r"    \___||_/ \_\|_|  |_|___|  \___/ \_/ |___|_|_\");
    w.reset_color();
    w.writeln("");

    let state = flow.game_state();

    let player_name = state
        .party
        .leader()
        .map(|p| p.name.as_str())
        .unwrap_or("Adventurer");

    w.set_fg(Color::White);
    w.writeln(&format!("  Adventurer: {}", player_name));
    w.writeln(&format!(
        "  Shrines Destroyed: {}/5",
        state.shrines_destroyed_count()
    ));
    w.writeln(&format!("  Play Time: {}", state.formatted_play_time()));
    w.writeln(&format!("  Gold: {}", format_gold(state.gold)));
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Your journey has come to an end.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_credits(w: &mut AnsiWriter, flow: &RalnarFlow) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln(r"   __   _____ ___ _____ ___  _____   ___");
    w.writeln(r"   \ \ / /_ _/ __|_   _/ _ \| _ \ \ / / |");
    w.writeln(r"    \ V / | | (__  | || (_) |   /\ V /|_|");
    w.writeln(r"     \_/ |___\___| |_| \___/|_|_\ |_| (_)");
    w.reset_color();
    w.writeln("");

    let state = flow.game_state();

    let player_name = state
        .party
        .leader()
        .map(|p| p.name.as_str())
        .unwrap_or("Adventurer");

    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln(&format!("  Congratulations, {}!", player_name));
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  You have saved the Realm of Ralnar!");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  Final Stats:");
    w.set_fg(Color::White);
    w.writeln(&format!(
        "    Shrines Destroyed: {}/5",
        state.shrines_destroyed_count()
    ));
    w.writeln(&format!("    Play Time: {}", state.formatted_play_time()));
    w.writeln(&format!("    Gold: {}", format_gold(state.gold)));
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Thank you for playing!");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_confirm_quit(w: &mut AnsiWriter) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === QUIT GAME? ===");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  Your progress will be saved.");
    w.writeln("");

    w.set_fg(Color::LightGreen);
    w.write_str("  [Y] ");
    w.set_fg(Color::White);
    w.writeln("Yes, quit");

    w.set_fg(Color::LightRed);
    w.write_str("  [N] ");
    w.set_fg(Color::White);
    w.writeln("No, continue playing");

    w.writeln("");
    w.reset_color();
    w.write_str("  Choice: ");
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_gold() {
        assert_eq!(format_gold(0), "0");
        assert_eq!(format_gold(100), "100");
        assert_eq!(format_gold(1000), "1,000");
        assert_eq!(format_gold(1234567), "1,234,567");
    }

    #[test]
    fn test_hp_bar() {
        assert_eq!(render_hp_bar(100, 100, 10), "[==========]");
        assert_eq!(render_hp_bar(50, 100, 10), "[=====     ]");
        assert_eq!(render_hp_bar(0, 100, 10), "[          ]");
        assert_eq!(render_hp_bar(100, 0, 10), "[          ]"); // Edge case: max 0
    }

    #[test]
    fn test_exp_bar() {
        assert_eq!(render_exp_bar(0, 100, 10), "[----------]");
        assert_eq!(render_exp_bar(50, 100, 10), "[#####-----]");
        assert_eq!(render_exp_bar(100, 100, 10), "[##########]");
        assert_eq!(render_exp_bar(100, 0, 10), "[==========]"); // Edge case: needed 0 (shows full)
    }

    #[test]
    fn test_status_icons() {
        assert_eq!(render_status_icons(&[]), "");
        assert_eq!(render_status_icons(&[StatusEffect::Poison]), " [PSN]");
        assert_eq!(
            render_status_icons(&[StatusEffect::Poison, StatusEffect::Haste]),
            " [PSN,HST]"
        );
    }

    #[test]
    fn test_render_intro_no_panic() {
        let mut w = AnsiWriter::new();
        render_intro(&mut w);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("Ralnar"));
    }

    #[test]
    fn test_render_main_menu_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_main_menu(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("TestPlayer"));
    }

    #[test]
    fn test_render_exploring_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_exploring(&mut w, &flow, "starting_village");
        let output = w.flush();
        assert!(!output.is_empty());
        // "Thornhaven Village" is uppercased in the display
        assert!(output.contains("THORNHAVEN VILLAGE"));
    }

    #[test]
    fn test_render_party_status_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_party_status(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("PARTY STATUS"));
    }

    #[test]
    fn test_render_battle_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_battle(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("BATTLE"));
    }

    #[test]
    fn test_render_inventory_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_inventory(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("INVENTORY"));
    }

    #[test]
    fn test_render_shop_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_shop(&mut w, &flow, "items");
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("Potion"));
    }

    #[test]
    fn test_render_game_over_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_game_over(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        // ASCII art contains "GAME" and "OVER" stylized
        assert!(output.contains("___|"));  // Part of the ASCII art
        assert!(output.contains("journey"));  // Shown in the text
    }

    #[test]
    fn test_render_credits_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_credits(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        // ASCII art contains VICTORY stylized, check for part of it
        assert!(output.contains("___"));  // Part of the ASCII art
        assert!(output.contains("Congratulations"));  // Shown in text
    }

    #[test]
    fn test_render_confirm_quit_no_panic() {
        let mut w = AnsiWriter::new();
        render_confirm_quit(&mut w);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("QUIT"));
    }

    #[test]
    fn test_render_quest_log_no_panic() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        let mut w = AnsiWriter::new();
        render_quest_log(&mut w, &flow);
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("QUEST"));
    }

    #[test]
    fn test_render_world_map_no_panic() {
        let mut w = AnsiWriter::new();
        render_world_map(&mut w, &RalnarFlow::new(1, "Test".to_string()));
        let output = w.flush();
        assert!(!output.is_empty());
        assert!(output.contains("WORLD MAP"));
    }

    #[test]
    fn test_render_screen_all_screens_no_panic() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());

        // Test each screen type
        let screens = [
            GameScreen::Intro,
            GameScreen::MainMenu,
            GameScreen::Exploring {
                map_id: "test".to_string(),
            },
            GameScreen::Dialogue {
                npc_id: "test".to_string(),
            },
            GameScreen::Shop {
                shop_id: "items".to_string(),
            },
            GameScreen::Inn,
            GameScreen::Battle,
            GameScreen::BattleVictory,
            GameScreen::BattleDefeat,
            GameScreen::Inventory,
            GameScreen::Equipment,
            GameScreen::PartyStatus,
            GameScreen::Magic,
            GameScreen::QuestLog,
            GameScreen::WorldMap,
            GameScreen::Cutscene {
                scene_id: "test".to_string(),
            },
            GameScreen::GameOver,
            GameScreen::Credits,
            GameScreen::ConfirmQuit,
        ];

        for screen in screens {
            flow.screen = screen;
            let output = render_screen(&flow);
            assert!(!output.is_empty(), "Screen rendered empty output");
        }
    }

    #[test]
    fn test_hp_bar_proportions() {
        // Full HP
        let bar = render_hp_bar(100, 100, 10);
        assert_eq!(bar.matches('=').count(), 10);

        // Half HP
        let bar = render_hp_bar(50, 100, 10);
        assert_eq!(bar.matches('=').count(), 5);

        // Quarter HP
        let bar = render_hp_bar(25, 100, 10);
        assert_eq!(bar.matches('=').count(), 2); // 25% of 10 = 2.5, truncated to 2

        // Zero HP
        let bar = render_hp_bar(0, 100, 10);
        assert_eq!(bar.matches('=').count(), 0);
    }

    #[test]
    fn test_status_icons_render_correctly() {
        // Single status
        let icons = render_status_icons(&[StatusEffect::Poison]);
        assert!(icons.contains("PSN"));

        // Multiple statuses
        let icons = render_status_icons(&[
            StatusEffect::Poison,
            StatusEffect::Haste,
            StatusEffect::Regen,
        ]);
        assert!(icons.contains("PSN"));
        assert!(icons.contains("HST"));
        assert!(icons.contains("RGN"));
    }
}
