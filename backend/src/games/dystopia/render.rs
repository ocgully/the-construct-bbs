//! ANSI rendering functions for Dystopia
//!
//! Visual identity: Dark dystopian theme
//! - Primary: Dark grays, muted colors
//! - Accent: Crimson red for warnings/power
//! - Highlight: Pale cyan for important info
//! - Background feel: Industrial, oppressive, bureaucratic

use crate::terminal::{AnsiWriter, Color};
use super::state::ProvinceState;
use super::data::{BuildingType, UnitType, RACES, PERSONALITIES, SpellType};
use super::screen::{GameScreen, DystopiaFlow, PendingAction};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format large numbers with commas: 1234567 -> "1,234,567"
pub fn format_number(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    let abs_n = n.abs();
    let s = format!("{}", abs_n);
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }

    format!("{}{}", sign, result)
}

/// Render the game header with dystopian art
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::DarkGray);
    w.writeln("");
    w.writeln("  ██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ██████╗ ██╗ █████╗ ");
    w.writeln("  ██╔══██╗╚██╗ ██╔╝██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██║██╔══██╗");
    w.set_fg(Color::LightGray);
    w.writeln("  ██║  ██║ ╚████╔╝ ███████╗   ██║   ██║   ██║██████╔╝██║███████║");
    w.writeln("  ██║  ██║  ╚██╔╝  ╚════██║   ██║   ██║   ██║██╔═══╝ ██║██╔══██║");
    w.set_fg(Color::White);
    w.writeln("  ██████╔╝   ██║   ███████║   ██║   ╚██████╔╝██║     ██║██║  ██║");
    w.set_fg(Color::DarkGray);
    w.writeln("  ╚═════╝    ╚═╝   ╚══════╝   ╚═╝    ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝");
    w.reset_color();
}

/// Render status bar with province info
fn render_status_bar(w: &mut AnsiWriter, state: &ProvinceState) {
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "─".repeat(74)));

    // Province name and race
    w.set_fg(Color::White);
    w.bold();
    w.write_str(&format!("  {} ", state.name));
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.write_str(&format!("({} {})", state.race, state.personality));

    // Protection status
    if state.is_protected() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("  [PROTECTED: {} ticks]", state.protection_ticks));
    }
    w.writeln("");

    // Resources line
    w.set_fg(Color::Yellow);
    w.write_str(&format!("  Gold: {} ", format_number(state.resources.gold)));
    w.set_fg(Color::Green);
    w.write_str(&format!("Food: {} ", format_number(state.resources.food)));
    w.set_fg(Color::LightMagenta);
    w.write_str(&format!("Runes: {} ", format_number(state.resources.runes)));
    w.set_fg(Color::LightGray);
    w.writeln(&format!("Land: {} acres", format_number(state.land as i64)));

    // Population and military
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("  Peasants: {}/{} ", format_number(state.peasants as i64), format_number(state.max_peasants as i64)));
    w.set_fg(Color::LightRed);
    w.write_str(&format!("Military: {} ", format_number(state.total_military() as i64)));
    w.set_fg(Color::LightGray);
    w.writeln(&format!("Networth: {}", format_number(state.networth())));

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "─".repeat(74)));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Render the current screen
pub fn render_screen(flow: &DystopiaFlow) -> String {
    let mut w = AnsiWriter::new();

    let pending = flow.pending_action();

    match flow.current_screen() {
        GameScreen::SelectRace => render_select_race(&mut w),
        GameScreen::SelectPersonality { race } => render_select_personality(&mut w, race),
        GameScreen::EnterName { race, personality } => render_enter_name(&mut w, race, personality),
        GameScreen::Throne => render_throne(&mut w, flow.game_state(), pending),
        GameScreen::Build => render_build(&mut w, flow.game_state(), pending),
        GameScreen::Military => render_military(&mut w, flow.game_state(), pending),
        GameScreen::Attack { .. } => render_attack(&mut w, flow.game_state(), pending),
        GameScreen::Thieves { .. } => render_thieves(&mut w, flow.game_state(), pending),
        GameScreen::Magic => render_magic(&mut w, flow.game_state()),
        GameScreen::Science => render_science(&mut w, flow.game_state()),
        GameScreen::Kingdom => render_kingdom(&mut w, flow.game_state()),
        GameScreen::Info => render_info(&mut w, flow.game_state()),
        GameScreen::Rankings => render_rankings(&mut w),
        GameScreen::History => render_history(&mut w),
        GameScreen::Help => render_help(&mut w),
        GameScreen::ConfirmQuit => render_confirm_quit(&mut w),
    }

    w.flush()
}

fn render_select_race(w: &mut AnsiWriter) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  In this dark age, you must choose your people wisely.");
    w.writeln("  Each race brings unique strengths to your province.");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SELECT YOUR RACE:");
    w.reset_color();
    w.writeln("");

    for (i, race) in RACES.iter().enumerate() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", i + 1));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<12}", race.name));
        w.set_fg(Color::LightGray);
        w.writeln(&format!(" - {}", race.special));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Enter choice (1-8): ");
    w.reset_color();
}

fn render_select_personality(w: &mut AnsiWriter, race: &str) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  As a {} ruler, you must now define your approach.", race));
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SELECT YOUR PERSONALITY:");
    w.reset_color();
    w.writeln("");

    for (i, personality) in PERSONALITIES.iter().enumerate() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", i + 1));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<15}", personality.name));
        w.set_fg(Color::LightGray);
        w.writeln(&format!(" - {}", personality.description));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Enter choice (1-6): ");
    w.reset_color();
}

fn render_enter_name(w: &mut AnsiWriter, race: &str, personality: &str) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  A {} {} rises to power.", race, personality));
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  NAME YOUR PROVINCE:");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Choose wisely - this name will strike fear into your enemies.");
    w.writeln("  (3-20 characters)");
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.write_str("  Province Name: ");
    w.reset_color();
}

fn render_throne(w: &mut AnsiWriter, state: &ProvinceState, pending: &Option<PendingAction>) {
    render_header(w);
    render_status_bar(w, state);

    // Show last message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    // Show pending action prompt
    if let Some(PendingAction::ExploreCount) = pending {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln("  How many soldiers should explore for new land?");
        w.set_fg(Color::DarkGray);
        w.write_str("  Number of explorers: ");
        w.reset_color();
        return;
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE THRONE ROOM");
    w.reset_color();
    w.writeln("");

    // Menu options in two columns
    w.set_fg(Color::LightCyan);
    w.write_str("    [B] ");
    w.set_fg(Color::White);
    w.write_str("Build           ");
    w.set_fg(Color::LightCyan);
    w.write_str("[M] ");
    w.set_fg(Color::White);
    w.writeln("Military");

    w.set_fg(Color::LightCyan);
    w.write_str("    [A] ");
    w.set_fg(Color::White);
    w.write_str("Attack          ");
    w.set_fg(Color::LightCyan);
    w.write_str("[T] ");
    w.set_fg(Color::White);
    w.writeln("Thieves");

    w.set_fg(Color::LightCyan);
    w.write_str("    [S] ");
    w.set_fg(Color::White);
    w.write_str("Spells          ");
    w.set_fg(Color::LightCyan);
    w.write_str("[R] ");
    w.set_fg(Color::White);
    w.writeln("Research");

    w.set_fg(Color::LightCyan);
    w.write_str("    [K] ");
    w.set_fg(Color::White);
    w.write_str("Kingdom         ");
    w.set_fg(Color::LightCyan);
    w.write_str("[E] ");
    w.set_fg(Color::White);
    w.writeln("Explore");

    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [I] ");
    w.set_fg(Color::White);
    w.write_str("Info            ");
    w.set_fg(Color::LightCyan);
    w.write_str("[L] ");
    w.set_fg(Color::White);
    w.writeln("Rankings");

    w.set_fg(Color::LightCyan);
    w.write_str("    [H] ");
    w.set_fg(Color::White);
    w.write_str("Help            ");
    w.set_fg(Color::LightCyan);
    w.write_str("[Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Command: ");
    w.reset_color();
}

fn render_build(w: &mut AnsiWriter, state: &ProvinceState, pending: &Option<PendingAction>) {
    render_header(w);
    render_status_bar(w, state);

    // Show pending action prompt
    if let Some(PendingAction::BuildingCount { building }) = pending {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("  How many {} to build?", building.name()));
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  Cost: {} gold each", format_number(state.building_cost(*building) as i64)));
        w.set_fg(Color::DarkGray);
        w.write_str("  Number to build: ");
        w.reset_color();
        return;
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CONSTRUCTION");
    w.reset_color();
    w.writeln("");

    // Show land usage
    let total_buildings: u32 = state.buildings.counts.values().sum();
    let under_construction: u32 = state.buildings.under_construction.values().sum();
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Land Used: {}/{} acres ({} under construction)",
        total_buildings, state.land, under_construction));
    w.writeln("");

    // Building list with current counts
    let buildings = [
        ("1", BuildingType::Home),
        ("2", BuildingType::Farm),
        ("3", BuildingType::Bank),
        ("4", BuildingType::Barracks),
        ("5", BuildingType::TrainingGround),
        ("6", BuildingType::Fort),
        ("7", BuildingType::Tower),
        ("8", BuildingType::ThievesDen),
        ("9", BuildingType::WatchTower),
        ("0", BuildingType::University),
        ("A", BuildingType::Hospital),
        ("C", BuildingType::Armoury),
        ("D", BuildingType::Guildhall),
    ];

    for (key, building) in buildings.iter() {
        let count = state.get_building(*building);
        let cost = state.building_cost(*building);

        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<18}", building.name()));
        w.set_fg(Color::LightGray);
        w.write_str(&format!("{:>4} built  ", count));
        w.set_fg(Color::Yellow);
        w.writeln(&format!("{} gc", format_number(cost as i64)));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to Throne");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Build: ");
    w.reset_color();
}

fn render_military(w: &mut AnsiWriter, state: &ProvinceState, pending: &Option<PendingAction>) {
    render_header(w);
    render_status_bar(w, state);

    // Show pending action prompt
    if let Some(PendingAction::TrainingCount { unit }) = pending {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("  How many {} to train?", unit.name()));
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  Cost: {} gold each (requires peasants)", format_number(unit.cost() as i64)));
        w.set_fg(Color::DarkGray);
        w.write_str("  Number to train: ");
        w.reset_color();
        return;
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  MILITARY COMMAND");
    w.reset_color();
    w.writeln("");

    // Current forces
    w.set_fg(Color::LightGray);
    w.writeln("  Current Forces:");
    w.writeln("");

    let units = [
        (UnitType::Soldier, "1"),
        (UnitType::Archer, "2"),
        (UnitType::Knight, "3"),
        (UnitType::Thief, "4"),
        (UnitType::Wizard, "5"),
        (UnitType::Elite, "6"),
    ];

    for (unit, key) in units.iter() {
        let count = state.get_unit(*unit);
        let training = match unit {
            UnitType::Soldier => state.military.training_soldiers,
            UnitType::Archer => state.military.training_archers,
            UnitType::Knight => state.military.training_knights,
            UnitType::Thief => state.military.training_thieves,
            UnitType::Wizard => state.military.training_wizards,
            UnitType::Elite => state.military.training_elites,
        };
        let cost = unit.cost();

        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<10}", unit.name()));
        w.set_fg(Color::LightGray);
        w.write_str(&format!("{:>6} active", count));
        if training > 0 {
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("  +{} training", training));
        }
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  {} gc", format_number(cost as i64)));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Offense: {}  Defense: {}  Upkeep: {} gc/tick",
        format_number(state.offense_strength() as i64),
        format_number(state.defense_strength() as i64),
        format_number(state.military_upkeep())));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to Throne");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Train: ");
    w.reset_color();
}

fn render_attack(w: &mut AnsiWriter, state: &ProvinceState, pending: &Option<PendingAction>) {
    render_header(w);
    render_status_bar(w, state);

    // Show pending action prompt
    if let Some(PendingAction::AttackArmy { attack_type, .. }) = pending {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("  Preparing {} attack...", attack_type.name()));
        w.set_fg(Color::LightGray);
        w.writeln("  What percentage of your army should march?");
        w.set_fg(Color::DarkGray);
        w.write_str("  Army percentage (1-100): ");
        w.reset_color();
        return;
    }

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("  WARFARE");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Select attack type:");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [1] ");
    w.set_fg(Color::White);
    w.writeln("Traditional March - Capture land and buildings");

    w.set_fg(Color::LightCyan);
    w.write_str("    [2] ");
    w.set_fg(Color::White);
    w.writeln("Raid - Steal gold and resources");

    w.set_fg(Color::LightCyan);
    w.write_str("    [3] ");
    w.set_fg(Color::White);
    w.writeln("Plunder - Destroy enemy buildings");

    w.set_fg(Color::LightCyan);
    w.write_str("    [4] ");
    w.set_fg(Color::White);
    w.writeln("Massacre - Kill enemy peasants");

    w.set_fg(Color::LightCyan);
    w.write_str("    [5] ");
    w.set_fg(Color::White);
    w.writeln("Learn - Steal enemy sciences");

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Your Offense Power: {}", format_number(state.offense_strength() as i64)));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to Throne");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Attack type: ");
    w.reset_color();
}

fn render_thieves(w: &mut AnsiWriter, state: &ProvinceState, pending: &Option<PendingAction>) {
    render_header(w);
    render_status_bar(w, state);

    // Show pending action prompt
    if let Some(PendingAction::ThiefCount { op_type, .. }) = pending {
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("  Planning {} operation...", op_type.name()));
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  Available thieves: {}", state.military.thieves));
        w.set_fg(Color::DarkGray);
        w.write_str("  Thieves to send: ");
        w.reset_color();
        return;
    }

    w.writeln("");
    w.set_fg(Color::Magenta);
    w.bold();
    w.writeln("  COVERT OPERATIONS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Available Thieves: {}", state.military.thieves));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [1] ");
    w.set_fg(Color::White);
    w.writeln("Intel Gathering - View target province");

    w.set_fg(Color::LightCyan);
    w.write_str("    [2] ");
    w.set_fg(Color::White);
    w.writeln("Steal Gold - Rob the treasury");

    w.set_fg(Color::LightCyan);
    w.write_str("    [3] ");
    w.set_fg(Color::White);
    w.writeln("Sabotage - Destroy buildings");

    w.set_fg(Color::LightCyan);
    w.write_str("    [4] ");
    w.set_fg(Color::White);
    w.writeln("Kidnap - Capture peasants");

    w.set_fg(Color::LightCyan);
    w.write_str("    [5] ");
    w.set_fg(Color::White);
    w.writeln("Assassinate - Kill specialists");

    w.set_fg(Color::LightCyan);
    w.write_str("    [6] ");
    w.set_fg(Color::White);
    w.writeln("Propaganda - Reduce enemy morale");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to Throne");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Operation: ");
    w.reset_color();
}

fn render_magic(w: &mut AnsiWriter, state: &ProvinceState) {
    render_header(w);
    render_status_bar(w, state);

    // Show last message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
    }

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("  ARCANE ARTS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Runes Available: {}  Wizards: {}",
        format_number(state.resources.runes), state.military.wizards));
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  DEFENSIVE SPELLS:");

    let defensive = [
        (SpellType::Shield, "1"),
        (SpellType::Barrier, "2"),
        (SpellType::Prosperity, "3"),
        (SpellType::Haste, "4"),
        (SpellType::Heal, "5"),
    ];

    for (spell, key) in defensive.iter() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<14}", spell.name()));
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("{} runes", format_number(spell.rune_cost() as i64)));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  OFFENSIVE SPELLS:");

    let offensive = [
        (SpellType::Clairvoyance, "6"),
        (SpellType::Fireball, "7"),
        (SpellType::Lightning, "8"),
        (SpellType::Plague, "9"),
        (SpellType::Drought, "0"),
    ];

    for (spell, key) in offensive.iter() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<14}", spell.name()));
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("{} runes", format_number(spell.rune_cost() as i64)));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to Throne");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Cast spell: ");
    w.reset_color();
}

fn render_science(w: &mut AnsiWriter, state: &ProvinceState) {
    render_header(w);
    render_status_bar(w, state);

    // Show last message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
    }

    w.writeln("");
    w.set_fg(Color::LightBlue);
    w.bold();
    w.writeln("  RESEARCH COUNCIL");
    w.reset_color();
    w.writeln("");

    // Current research
    if let Some(ref current) = state.sciences.current_research {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  Researching: {} ({}% complete)",
            current, state.sciences.research_progress));
        w.writeln("");
    }

    w.set_fg(Color::LightGray);
    w.writeln("  Sciences (each level gives +1% bonus):");
    w.writeln("");

    let sciences = [
        ("1", "alchemy", "Rune Production", state.sciences.alchemy),
        ("2", "tools", "Build Speed", state.sciences.tools),
        ("3", "housing", "Population Cap", state.sciences.housing),
        ("4", "food", "Food Production", state.sciences.food),
        ("5", "military", "Combat Power", state.sciences.military),
        ("6", "crime", "Thief Success", state.sciences.crime),
        ("7", "channeling", "Magic Power", state.sciences.channeling),
    ];

    for (key, _name, desc, level) in sciences.iter() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<18}", desc));
        w.set_fg(Color::LightGray);
        w.writeln(&format!("Level {}", level));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back to Throne");

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.write_str("  Research: ");
    w.reset_color();
}

fn render_kingdom(w: &mut AnsiWriter, state: &ProvinceState) {
    render_header(w);
    render_status_bar(w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  KINGDOM AFFAIRS");
    w.reset_color();
    w.writeln("");

    if let Some(ref kingdom) = state.kingdom {
        w.set_fg(Color::White);
        w.writeln(&format!("  Kingdom: {}", kingdom.kingdom_name));
        w.set_fg(Color::LightGray);
        if kingdom.is_ruler {
            w.writeln("  You are the RULER of this kingdom.");
        } else {
            w.writeln("  You are a member of this kingdom.");
        }
    } else {
        w.set_fg(Color::LightGray);
        w.writeln("  You are not a member of any kingdom.");
        w.writeln("");
        w.writeln("  Kingdoms allow up to 10 players to coordinate");
        w.writeln("  strategies, share intelligence, and wage war together.");
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_info(w: &mut AnsiWriter, state: &ProvinceState) {
    render_header(w);
    render_status_bar(w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  PROVINCE STATISTICS");
    w.reset_color();
    w.writeln("");

    // Income/expenses
    w.set_fg(Color::LightGray);
    w.writeln("  ECONOMY (per tick):");
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("    Gold Income:     +{}", format_number(state.gold_income())));
    w.set_fg(Color::LightRed);
    w.writeln(&format!("    Military Upkeep: -{}", format_number(state.military_upkeep())));
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("    Food Production: +{}", format_number(state.food_production())));
    w.set_fg(Color::LightRed);
    w.writeln(&format!("    Food Consumption:    -{}", format_number(state.food_consumption())));
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("    Rune Production: +{}", format_number(state.rune_production())));

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  BATTLE RECORD:");
    w.writeln(&format!("    Attacks: {} sent, {} won", state.stats.attacks_sent, state.stats.attacks_won));
    w.writeln(&format!("    Defenses: {} faced, {} won", state.stats.defenses, state.stats.defenses_won));
    w.writeln(&format!("    Land Captured: {}  Lost: {}", state.stats.land_captured, state.stats.land_lost));

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Peak Networth: {}", format_number(state.stats.peak_networth)));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_rankings(w: &mut AnsiWriter) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  PROVINCIAL RANKINGS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Rankings will show top provinces by networth.");
    w.writeln("  (Data loaded from database in real implementation)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_history(w: &mut AnsiWriter) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  AGE HISTORY");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Past ages and their winners will be displayed here.");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_help(w: &mut AnsiWriter) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  DYSTOPIA - HELP");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  In Dystopia, you manage a province in a dark age.");
    w.writeln("  Ages last several weeks (sysop configurable).");
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  BUILD - Construct buildings to grow your economy");
    w.writeln("  MILITARY - Train troops for offense and defense");
    w.writeln("  ATTACK - Send armies against rival provinces");
    w.writeln("  THIEVES - Covert operations against enemies");
    w.writeln("  SPELLS - Cast magic for various effects");
    w.writeln("  RESEARCH - Improve your sciences for bonuses");
    w.writeln("  KINGDOM - Join with up to 9 other players");
    w.writeln("  EXPLORE - Send soldiers to find new land");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_confirm_quit(w: &mut AnsiWriter) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  Your province will persist while you are away.");
    w.writeln("  Resources will continue to accumulate.");
    w.writeln("");
    w.writeln("  WARNING: You may be attacked while offline!");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();
}

/// Render leaderboard screen with async data
pub fn render_leaderboard_screen(entries: &[(i64, String, i64, u32, u32)]) -> String {
    let mut w = AnsiWriter::new();

    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  PROVINCIAL RANKINGS - HALL OF POWER");
    w.reset_color();
    w.writeln("");

    if entries.is_empty() {
        w.set_fg(Color::LightGray);
        w.writeln("  No provinces have fallen yet this age.");
    } else {
        // Header
        w.set_fg(Color::DarkGray);
        w.writeln("  Rank  Province             Networth      Land   Victories");
        w.writeln(&format!("  {}", "-".repeat(60)));

        for (rank, handle, networth, land, attacks_won) in entries.iter() {
            let color = match rank {
                1 => Color::Yellow,
                2 => Color::LightGray,
                3 => Color::LightRed,
                _ => Color::White,
            };

            w.set_fg(color);
            w.write_str(&format!("  {:>4}  ", rank));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20} ", handle));
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("{:>12} ", format_number(*networth)));
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("{:>6} ", land));
            w.set_fg(Color::LightRed);
            w.writeln(&format!("{:>8}", attacks_won));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(-1234), "-1,234");
    }

    #[test]
    fn test_render_screen_no_panic() {
        let flow = DystopiaFlow::new();
        let output = render_screen(&flow);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_render_throne_screen() {
        let flow = DystopiaFlow::from_state(ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        ));
        let output = render_screen(&flow);
        assert!(output.contains("THRONE ROOM"));
        assert!(output.contains("Test"));
    }
}
