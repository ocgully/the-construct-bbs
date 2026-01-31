//! ANSI rendering for Cradle
//!
//! Ethereal/cosmic visual identity with deep purples, blues,
//! and mystical accents. Focus on progression and cultivation themes.

use crate::terminal::{AnsiWriter, Color};
use super::state::GameState;
use super::data::{TierLevel, get_tier, get_path, PATHS, TECHNIQUES};
use super::screen::{GameScreen, CreationStage, CradleFlow};
use super::events::{TrialState, get_trial_prompt};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format large power numbers with suffixes
pub fn format_power(n: u64) -> String {
    if n >= 1_000_000_000_000_000 {
        format!("{:.1}Q", n as f64 / 1_000_000_000_000_000.0)
    } else if n >= 1_000_000_000_000 {
        format!("{:.1}T", n as f64 / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Format percentage with color
fn tier_color(tier: TierLevel) -> Color {
    match tier {
        TierLevel::Unsouled => Color::DarkGray,
        TierLevel::Copper => Color::Brown,
        TierLevel::Iron => Color::LightGray,
        TierLevel::Jade => Color::LightGreen,
        TierLevel::Gold => Color::Yellow,
        TierLevel::Lord => Color::LightCyan,
        TierLevel::Overlord => Color::LightBlue,
        TierLevel::Sage => Color::Magenta,
        TierLevel::Herald => Color::LightMagenta,
        TierLevel::Monarch => Color::LightRed,
        TierLevel::Dreadgod => Color::Red,
        TierLevel::Abidan => Color::White,
        TierLevel::Judge => Color::Yellow,
        TierLevel::God => Color::LightCyan,
        TierLevel::Void => Color::DarkGray,
        TierLevel::Transcendent => Color::White,
    }
}

/// Render the ethereal game header
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::Magenta);
    w.bold();
    w.writeln("");
    w.writeln("   ██████╗██████╗  █████╗ ██████╗ ██╗     ███████╗");
    w.writeln("  ██╔════╝██╔══██╗██╔══██╗██╔══██╗██║     ██╔════╝");
    w.writeln("  ██║     ██████╔╝███████║██║  ██║██║     █████╗  ");
    w.writeln("  ██║     ██╔══██╗██╔══██║██║  ██║██║     ██╔══╝  ");
    w.writeln("  ╚██████╗██║  ██║██║  ██║██████╔╝███████╗███████╗");
    w.writeln("   ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝");
    w.set_fg(Color::LightBlue);
    w.writeln("                    CRADLE");
    w.writeln("     ~ The Path of Infinite Progression ~");
    w.reset_color();
}

/// Render status bar with current game state
fn render_status_bar(w: &mut AnsiWriter, state: &GameState) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(80));

    // Line 1: Tier and progress
    w.set_fg(Color::White);
    w.write_str("  ");
    w.write_str(&state.name);
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(tier_color(state.tier));
    w.bold();
    w.write_str(state.tier.name());
    w.reset_color();

    // Progress bar
    let progress_pct = (state.tier_progress * 100.0) as u32;
    w.set_fg(Color::DarkGray);
    w.write_str(" [");
    w.set_fg(Color::LightMagenta);
    let filled = (state.tier_progress * 20.0) as usize;
    w.write_str(&"█".repeat(filled));
    w.set_fg(Color::DarkGray);
    w.write_str(&"░".repeat(20 - filled));
    w.write_str(&format!("] {}%", progress_pct));

    // Power
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("Power: {}", format_power(state.total_power())));
    w.writeln("");

    // Line 2: Resources
    w.set_fg(Color::LightBlue);
    w.write_str(&format!("  Madra: {}/{}", format_power(state.madra), format_power(state.max_madra)));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(Color::LightMagenta);
    w.write_str(&format!("Insight: {}", state.insight));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(Color::Yellow);
    w.write_str(&format!("Stones: {}", format_power(state.spirit_stones)));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    w.set_fg(Color::LightGreen);
    w.write_str(&format!("Elixirs: {}", state.elixirs));
    w.writeln("");

    // Line 3: Path info
    if let Some(ref path_key) = state.primary_path {
        if let Some(path) = get_path(path_key) {
            w.set_fg(Color::White);
            w.write_str("  Path: ");
            w.set_fg(Color::LightMagenta);
            w.write_str(path.name);
            if let Some(ref secondary) = state.secondary_path {
                if let Some(sec_path) = get_path(secondary) {
                    w.set_fg(Color::DarkGray);
                    w.write_str(" + ");
                    w.set_fg(Color::LightBlue);
                    w.write_str(sec_path.name);
                }
            }
        }
    } else {
        w.set_fg(Color::DarkGray);
        w.write_str("  Path: None selected");
    }
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(80));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTIONS
// ============================================================================

/// Render intro screen
pub fn render_intro(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightBlue);
    w.writeln("  In the beginning, there was only the Void.");
    w.writeln("  From nothing came everything - power, existence, the Way.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  You are Unsouled. A spark waiting to become a flame.");
    w.writeln("  Through cultivation, you will rise.");
    w.writeln("  Through the Sacred Arts, you will transcend.");
    w.writeln("");

    if !state.name.is_empty() {
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  Welcome back, {}.", state.name));
        w.writeln("");
    }

    if state.ascended {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  Ascensions: {}  |  Points: {}",
            state.prestige.total_prestiges,
            format_power(state.prestige.ascension_points)
        ));
        w.writeln("");
    }

    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to begin your cultivation...");
    w.reset_color();

    w.flush()
}

/// Render character creation screen
pub fn render_character_creation(state: &GameState, stage: &CreationStage) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE AWAKENING");
    w.reset_color();
    w.writeln("");

    match stage {
        CreationStage::Name => {
            w.set_fg(Color::White);
            w.writeln("  Before you can walk the path of cultivation,");
            w.writeln("  you must know yourself.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("  What is your name, cultivator? ");
            w.reset_color();
        }
        CreationStage::Confirm => {
            w.set_fg(Color::White);
            w.writeln(&format!("  You are {}.", state.name));
            w.writeln("");
            w.writeln("  The path ahead is long and treacherous.");
            w.writeln("  Many will fail. Few will rise.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("  Begin your journey? [Y/N] ");
            w.reset_color();
        }
    }

    // Show any message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::LightRed);
        w.writeln(&format!("  {}", msg));
        w.reset_color();
    }

    w.flush()
}

/// Render main cultivation menu
pub fn render_main_menu(state: &GameState, catchup_message: Option<&str>) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    // Show catchup message if any
    if let Some(msg) = catchup_message {
        w.set_fg(Color::LightGreen);
        for line in msg.lines() {
            w.writeln(&format!("  {}", line));
        }
        w.writeln("");
    }

    // Show last message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
        w.writeln("");
    }

    // Plateau warning
    if state.is_plateaued() {
        w.set_fg(Color::LightRed);
        w.bold();
        w.writeln("  !! WARNING: Your cultivation has PLATEAUED !!");
        w.writeln("  !! Consult your mentor or consider a respec. !!");
        w.reset_color();
        w.writeln("");
    }

    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("  What would you do?");
    w.reset_color();
    w.writeln("");

    // Core actions
    w.set_fg(Color::LightCyan);
    w.write_str("    [C] ");
    w.set_fg(Color::White);
    w.writeln("Cycle/Cultivate");

    w.set_fg(Color::LightCyan);
    w.write_str("    [P] ");
    w.set_fg(Color::White);
    w.writeln("Path Selection");

    w.set_fg(Color::LightCyan);
    w.write_str("    [T] ");
    w.set_fg(Color::White);
    w.writeln("Techniques");

    // Advancement trial (if trial is available)
    if let Some(next_tier) = state.tier.next() {
        if let Some(tier_data) = get_tier(next_tier) {
            if tier_data.trial_required && !state.trials_completed.contains(&next_tier) {
                w.set_fg(Color::LightCyan);
                w.write_str("    [A] ");
                w.set_fg(Color::Yellow);
                w.writeln(&format!("Attempt {} Trial", next_tier.name()));
            }
        }
    }

    w.set_fg(Color::LightCyan);
    w.write_str("    [M] ");
    w.set_fg(Color::White);
    w.writeln("Mentor");

    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [R] ");
    w.set_fg(Color::White);
    w.writeln(&format!("Respec (Cost: {} stones)", format_power(state.respec_cost())));

    w.set_fg(Color::LightCyan);
    w.write_str("    [E] ");
    w.set_fg(Color::White);
    w.writeln(&format!("Ascension/Prestige ({} points)", format_power(state.potential_ascension_points())));

    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [S] ");
    w.set_fg(Color::White);
    w.writeln("Statistics");

    w.set_fg(Color::LightCyan);
    w.write_str("    [L] ");
    w.set_fg(Color::White);
    w.writeln("Leaderboard");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Save & Quit");

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render path selection screen
pub fn render_path_selection(state: &GameState, selecting_secondary: bool) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    if selecting_secondary {
        w.writeln("  SELECT SECONDARY PATH");
        w.reset_color();
        w.set_fg(Color::DarkGray);
        w.writeln("  A secondary path enhances your cultivation, but incompatible paths");
        w.writeln("  will cause you to plateau!");
    } else {
        w.writeln("  SELECT YOUR PATH");
        w.reset_color();
        w.set_fg(Color::DarkGray);
        w.writeln("  Choose wisely - your path determines your potential.");
    }
    w.reset_color();
    w.writeln("");

    // List paths
    for (i, path) in PATHS.iter().enumerate() {
        let is_current = state.primary_path.as_ref().map(|p| p == path.key).unwrap_or(false)
            || state.secondary_path.as_ref().map(|p| p == path.key).unwrap_or(false);

        // Check compatibility
        let is_compatible = if selecting_secondary {
            if let Some(ref primary) = state.primary_path {
                if let Some(primary_path) = get_path(primary) {
                    !primary_path.incompatible_with.contains(&path.key)
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            true
        };

        if is_current {
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("    [{:2}] {} (current)", i + 1, path.name));
        } else if !is_compatible {
            w.set_fg(Color::LightRed);
            w.write_str(&format!("    [{:2}] {} [INCOMPATIBLE]", i + 1, path.name));
        } else {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    [{:2}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(path.name);
        }

        // Show aspects
        w.set_fg(Color::DarkGray);
        let aspects: Vec<_> = path.aspects.iter().map(|a| a.name()).collect();
        w.write_str(&format!(" ({})", aspects.join("+")));

        // Show max tier
        w.set_fg(Color::DarkGray);
        w.write_str(&format!(" - Max: {}", path.max_tier_natural.name()));
        w.writeln("");
    }

    w.writeln("");

    if !selecting_secondary && state.tier >= TierLevel::Gold && state.primary_path.is_some() {
        w.set_fg(Color::LightCyan);
        w.write_str("    [2] ");
        w.set_fg(Color::White);
        w.writeln("Select Secondary Path");
    }

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    // Show message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render techniques screen
pub fn render_techniques(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SACRED ARTS - TECHNIQUES");
    w.reset_color();
    w.writeln("");

    // Get available techniques
    let available: Vec<_> = TECHNIQUES.iter()
        .filter(|t| {
            state.primary_path.as_ref().map(|p| p == t.path_key).unwrap_or(false)
                || state.secondary_path.as_ref().map(|p| p == t.path_key).unwrap_or(false)
        })
        .filter(|t| t.tier_requirement <= state.tier)
        .collect();

    if available.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("    Select a path first to unlock techniques.");
    } else {
        for (i, tech) in available.iter().enumerate() {
            let owned = state.technique_levels.contains_key(tech.key);
            let level = state.technique_levels.get(tech.key).copied().unwrap_or(0);

            if owned {
                w.set_fg(Color::LightGreen);
                w.write_str(&format!("    [{:2}] {} (Lv.{}) ", i + 1, tech.name, level));
                w.set_fg(Color::DarkGray);
                w.writeln("[UPGRADE]");
            } else {
                let cost = 100u64 * (tech.tier_requirement as u64 + 1) + tech.power_base / 10;
                let can_afford = state.spirit_stones >= cost;

                w.set_fg(if can_afford { Color::LightCyan } else { Color::DarkGray });
                w.write_str(&format!("    [{:2}] {} ", i + 1, tech.name));
                w.set_fg(Color::Yellow);
                w.writeln(&format!("({} stones)", format_power(cost)));
            }

            w.set_fg(Color::DarkGray);
            w.writeln(&format!("         {} | {} | Power: {}",
                tech.technique_type.name(),
                tech.tier_requirement.name(),
                tech.power_base
            ));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render trial screen
pub fn render_trial(state: &GameState, trial: &TrialState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln(&format!("  ADVANCEMENT TRIAL - {}", trial.target_tier.name()));
    w.reset_color();
    w.writeln("");

    // Progress
    w.set_fg(Color::White);
    w.writeln(&format!("  Stage {}/{}", trial.stage, trial.max_stages));
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  Successes: {}/{}", trial.success_count, (trial.max_stages / 2) + 1));
    w.writeln("");

    // Get current prompt
    let (prompt, choices) = get_trial_prompt(trial);

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  {}", prompt));
    w.writeln("");

    for (i, choice) in choices.iter().enumerate() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{}] ", i + 1));
        w.set_fg(Color::White);
        w.writeln(choice);
    }

    if trial.stage == 1 {
        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.write_str("    [Q] ");
        w.writeln("Abandon Trial");
    }

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render mentor screen
pub fn render_mentor(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  MENTOR'S GUIDANCE");
    w.reset_color();
    w.writeln("");

    if let Some(mentor) = super::events::get_current_mentor(state) {
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  {}", mentor.name));
        w.set_fg(Color::DarkGray);
        w.writeln(&format!("  \"{}\"", mentor.personality));
        w.set_fg(Color::White);
        w.writeln(&format!("  Guidance range: {} to {}",
            mentor.tier_range.0.name(),
            mentor.tier_range.1.name()
        ));
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("  No mentor available at your level.");
    }

    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("    [H] ");
    w.set_fg(Color::White);
    w.writeln("Ask for Hint");

    w.set_fg(Color::LightCyan);
    w.write_str("    [W] ");
    w.set_fg(Color::White);
    w.writeln("Check for Warnings");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render prestige screen
pub fn render_prestige(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("  ASCENSION");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln("  Ascension resets your progress but grants permanent power.");
    w.writeln("  The higher your tier, the more Ascension Points you earn.");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Current Ascension Points: {}", format_power(state.prestige.ascension_points)));
    w.writeln(&format!("  Points if you ascend now: {}", format_power(state.potential_ascension_points())));
    w.writeln(&format!("  Total Ascensions: {}", state.prestige.total_prestiges));
    w.writeln("");

    w.set_fg(Color::LightGray);
    w.writeln("  Current Bonuses:");
    w.writeln(&format!("    Madra Multiplier: x{:.1}", state.prestige.madra_multiplier));
    w.writeln(&format!("    Insight Multiplier: x{:.1}", state.prestige.insight_multiplier));
    w.writeln(&format!("    Stone Multiplier: x{:.1}", state.prestige.spirit_stone_multiplier));
    w.writeln(&format!("    Starting Tier: {}", TierLevel::from_u8(state.prestige.starting_tier_bonus).unwrap_or(TierLevel::Unsouled).name()));
    w.writeln("");

    if state.tier >= TierLevel::Gold {
        w.set_fg(Color::LightCyan);
        w.write_str("    [A] ");
        w.set_fg(Color::LightGreen);
        w.writeln("ASCEND NOW");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("    (Reach Gold tier to ascend)");
    }

    w.set_fg(Color::LightCyan);
    w.write_str("    [S] ");
    w.set_fg(Color::White);
    w.writeln("Prestige Shop");

    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render prestige shop
pub fn render_prestige_shop(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  PRESTIGE SHOP");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("  Ascension Points: {}", format_power(state.prestige.ascension_points)));
    w.writeln("");

    let upgrades = [
        ("1", "Madra Multiplier +10%", 100, state.prestige.madra_multiplier),
        ("2", "Insight Multiplier +10%", 100, state.prestige.insight_multiplier),
        ("3", "Stone Multiplier +10%", 150, state.prestige.spirit_stone_multiplier),
        ("4", "Unlock Speed +10%", 200, state.prestige.unlock_speed_bonus),
    ];

    for (key, name, cost, current) in &upgrades {
        let can_afford = state.prestige.ascension_points >= *cost;
        w.set_fg(if can_afford { Color::LightCyan } else { Color::DarkGray });
        w.write_str(&format!("    [{}] {} ", key, name));
        w.set_fg(Color::Yellow);
        w.write_str(&format!("({} pts) ", cost));
        w.set_fg(Color::LightGray);
        w.writeln(&format!("[Current: x{:.1}]", current));
    }

    // Starting tier upgrade
    let tier_cost = (state.prestige.starting_tier_bonus as u64 + 1) * 500;
    let can_afford_tier = state.prestige.ascension_points >= tier_cost && state.prestige.starting_tier_bonus < 5;
    w.set_fg(if can_afford_tier { Color::LightCyan } else { Color::DarkGray });
    w.write_str("    [5] Start at Higher Tier ");
    w.set_fg(Color::Yellow);
    w.write_str(&format!("({} pts) ", tier_cost));
    w.set_fg(Color::LightGray);
    let current_start = TierLevel::from_u8(state.prestige.starting_tier_bonus).unwrap_or(TierLevel::Unsouled);
    w.writeln(&format!("[Current: {}]", current_start.name()));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render stats screen
pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CULTIVATION STATISTICS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Current Progress:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Tier: {}", state.tier.name()));
    w.writeln(&format!("    Total Power: {}", format_power(state.total_power())));
    w.writeln(&format!("    Defense: {}", format_power(state.total_defense())));
    w.writeln(&format!("    Madra Regen: {}/tick", format_power(state.madra_per_tick())));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Combat Record:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Battles Won: {}", state.stats.battles_won));
    w.writeln(&format!("    Battles Lost: {}", state.stats.battles_lost));
    w.writeln(&format!("    Enemies Defeated: {}", state.stats.enemies_defeated));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Lifetime Stats:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Peak Power: {}", format_power(state.stats.peak_power)));
    w.writeln(&format!("    Highest Tier: {}", state.stats.highest_tier_reached.name()));
    w.writeln(&format!("    Total Ticks: {}", format_power(state.total_ticks)));
    w.writeln(&format!("    Respecs Used: {}", state.prestige.respecs_used));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Prestige Stats:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Total Ascensions: {}", state.prestige.total_prestiges));
    w.writeln(&format!("    Total Points Earned: {}", format_power(state.prestige.total_ascension_points_earned)));
    w.writeln(&format!("    Highest Ever: {}", state.prestige.highest_tier_ever.name()));

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to continue...");

    w.flush()
}

/// Render respec confirmation
pub fn render_respec(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("  RESPEC - RESET YOUR PATH");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln("  WARNING: This will reset your cultivation path!");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Effects:");
    w.writeln("    - All path progress will be lost");
    w.writeln("    - All techniques will be lost");
    w.writeln("    - Tier progress reduced by 50%");
    w.writeln("    - Your tier will NOT change");
    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("  Cost: {} spirit stones", format_power(state.respec_cost())));
    w.writeln(&format!("  You have: {} spirit stones", format_power(state.spirit_stones)));
    w.writeln("");

    if state.spirit_stones >= state.respec_cost() {
        w.set_fg(Color::LightCyan);
        w.write_str("  Proceed with respec? [Y/N] ");
    } else {
        w.set_fg(Color::LightRed);
        w.writeln("  Insufficient spirit stones for respec.");
        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.write_str("  Press any key to return...");
    }

    w.flush()
}

/// Render victory screen
pub fn render_victory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("");
    w.writeln("  ████████╗██████╗  █████╗ ███╗   ██╗███████╗ ██████╗███████╗███╗   ██╗██████╗ ███████╗███╗   ██╗ ██████╗███████╗");
    w.writeln("  ╚══██╔══╝██╔══██╗██╔══██╗████╗  ██║██╔════╝██╔════╝██╔════╝████╗  ██║██╔══██╗██╔════╝████╗  ██║██╔════╝██╔════╝");
    w.writeln("     ██║   ██████╔╝███████║██╔██╗ ██║███████╗██║     █████╗  ██╔██╗ ██║██║  ██║█████╗  ██╔██╗ ██║██║     █████╗  ");
    w.writeln("     ██║   ██╔══██╗██╔══██║██║╚██╗██║╚════██║██║     ██╔══╝  ██║╚██╗██║██║  ██║██╔══╝  ██║╚██╗██║██║     ██╔══╝  ");
    w.writeln("     ██║   ██║  ██║██║  ██║██║ ╚████║███████║╚██████╗███████╗██║ ╚████║██████╔╝███████╗██║ ╚████║╚██████╗███████╗");
    w.writeln("     ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚══════╝ ╚═════╝╚══════╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝╚══════╝");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln(&format!("  {}, you have achieved the impossible.", state.name));
    w.writeln("");
    w.set_fg(Color::LightBlue);
    w.writeln("  You have transcended existence itself.");
    w.writeln("  The concepts of power, time, and reality no longer bind you.");
    w.writeln("  You ARE the Way.");
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Final Tier: {}", state.tier.name()));
    w.writeln(&format!("  Total Power: {}", format_power(state.total_power())));
    w.writeln(&format!("  Ascension Points Earned: {}", format_power(state.potential_ascension_points())));
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to ascend to a new beginning...");

    w.flush()
}

/// Render quit confirmation
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
    w.writeln("  Your progress will be saved.");
    w.writeln("  Cultivation continues even while you're away.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render leaderboard screen (placeholder)
pub fn render_leaderboard() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  HALL OF TRANSCENDENCE");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Loading leaderboard...");
    w.writeln("");
    w.writeln("  Press any key to continue...");

    w.flush()
}

/// Main render dispatch function
pub fn render_screen(flow: &CradleFlow) -> String {
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(state),
        GameScreen::CharacterCreation { stage } => render_character_creation(state, stage),
        GameScreen::MainMenu => render_main_menu(state, flow.catchup_message.as_deref()),
        GameScreen::PathSelection { selecting_secondary } => render_path_selection(state, *selecting_secondary),
        GameScreen::Techniques => render_techniques(state),
        GameScreen::Trial { trial } => render_trial(state, trial),
        GameScreen::Mentor => render_mentor(state),
        GameScreen::Prestige => render_prestige(state),
        GameScreen::PrestigeShop => render_prestige_shop(state),
        GameScreen::Stats => render_stats(state),
        GameScreen::Leaderboard => render_leaderboard(),
        GameScreen::Respec => render_respec(state),
        GameScreen::Victory => render_victory(state),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_power() {
        assert_eq!(format_power(500), "500");
        assert_eq!(format_power(5000), "5.0K");
        assert_eq!(format_power(5_000_000), "5.0M");
        assert_eq!(format_power(5_000_000_000), "5.0B");
    }

    #[test]
    fn test_render_intro() {
        let state = GameState::new("Test".to_string());
        let output = render_intro(&state);
        assert!(output.contains("CRADLE"));
        assert!(output.contains("Void"));
    }

    #[test]
    fn test_render_main_menu() {
        let mut state = GameState::new("TestPlayer".to_string());
        state.tier = TierLevel::Copper;
        let output = render_main_menu(&state, None);
        assert!(output.contains("Cycle"));
        assert!(output.contains("Path"));
        assert!(output.contains("TestPlayer"));
    }

    #[test]
    fn test_tier_color_progression() {
        // Higher tiers should use different colors
        let copper = tier_color(TierLevel::Copper);
        let monarch = tier_color(TierLevel::Monarch);
        assert_ne!(copper as u8, monarch as u8);
    }
}
