//! ANSI rendering for Dragon Slayer
//! Classic LoRD-style medieval RPG aesthetic

use crate::terminal::{AnsiWriter, Color};
use super::state::GameState;
use super::screen::{GameScreen, DragonSlayerFlow, CreationStep};
use super::combat::CombatState;
use super::data::{WEAPONS, ARMOR, get_master, get_weapon, get_armor};
use super::events::ForestEvent;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format gold with commas
pub fn format_gold(amount: i64) -> String {
    if amount < 0 {
        return format!("-{}", format_gold(-amount));
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

/// Render the game header with dragon art
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln("   ____                              ____  _                       ");
    w.writeln("  |    \\ ___ ___ ___ ___ ___    ___ |  | || |___ _ _ ___ ___       ");
    w.writeln("  |  |  |  _| .'| . | . |   |  |_ -||  |_|| | .'| | | -_|  _|      ");
    w.writeln("  |____/|_| |__,|_  |___|_|_|  |___||_____||__,|_  |___|_|         ");
    w.writeln("                |___|                          |___|               ");
    w.set_fg(Color::Red);
    w.writeln("                        ~ The Red Dragon Awaits ~                  ");
    w.reset_color();
}

/// Render the status bar
fn render_status_bar(w: &mut AnsiWriter, state: &GameState) {
    w.set_fg(Color::Brown);
    w.writeln(&"\u{2500}".repeat(78));

    // Line 1: Name, Level, XP
    w.set_fg(Color::Yellow);
    w.write_str(&format!(" {} ", state.char_name));
    w.set_fg(Color::White);
    w.write_str(&format!("Lv.{}", state.level));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::Cyan);
    w.write_str(&format!("XP: {}", format_gold(state.experience)));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Health with color coding
    let hp_pct = (state.hp_current * 100) / state.hp_max.max(1);
    let hp_color = if hp_pct > 70 {
        Color::LightGreen
    } else if hp_pct > 30 {
        Color::Yellow
    } else {
        Color::LightRed
    };
    w.set_fg(hp_color);
    w.write_str(&format!("HP: {}/{}", state.hp_current, state.hp_max));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Gold
    w.set_fg(Color::Yellow);
    w.writeln(&format!("Gold: {}", format_gold(state.gold_pocket)));

    // Line 2: Equipment and daily fights
    w.set_fg(Color::LightGray);
    let weapon_name = get_weapon(&state.equipment.weapon)
        .map(|w| w.name)
        .unwrap_or("None");
    let armor_name = get_armor(&state.equipment.armor)
        .map(|a| a.name)
        .unwrap_or("None");
    w.write_str(&format!(" Weapon: {}", weapon_name));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightGray);
    w.write_str(&format!("Armor: {}", armor_name));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("Forest Fights: {}", state.forest_fights_remaining()));

    w.set_fg(Color::Brown);
    w.writeln(&"\u{2500}".repeat(78));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTION
// ============================================================================

/// Main render function - dispatches to screen-specific renderers
pub fn render_screen(flow: &DragonSlayerFlow) -> String {
    let mut w = AnsiWriter::new();
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::CharacterCreation { step } => render_creation(&mut w, step, state),
        GameScreen::Intro => render_intro(&mut w, state),
        GameScreen::Town => render_town(&mut w, state),
        GameScreen::Forest => render_forest(&mut w, state),
        GameScreen::Combat { combat } => render_combat(&mut w, state, combat),
        GameScreen::ForestEvent { event } => render_forest_event(&mut w, state, event),
        GameScreen::Training => render_training(&mut w, state),
        GameScreen::WeaponShop => render_weapon_shop(&mut w, state),
        GameScreen::ArmorShop => render_armor_shop(&mut w, state),
        GameScreen::Healer => render_healer(&mut w, state),
        GameScreen::Bank => render_bank(&mut w, state),
        GameScreen::KingsCourt => render_kings_court(&mut w, state),
        GameScreen::Inn => render_inn(&mut w, state),
        GameScreen::Violet => render_violet(&mut w, state),
        GameScreen::Seth => render_seth(&mut w, state),
        GameScreen::Arena => render_arena(&mut w, state),
        GameScreen::OtherPlaces => render_other_places(&mut w, state, flow),
        GameScreen::IgmLocation { module_id } => render_igm_location(&mut w, state, module_id, flow),
        GameScreen::Stats => render_stats(&mut w, state),
        GameScreen::Leaderboard => render_leaderboard(&mut w),
        GameScreen::DragonHunt => render_dragon_hunt(&mut w, state),
        GameScreen::GameOver => render_game_over(&mut w, state),
        GameScreen::ConfirmQuit => render_confirm_quit(&mut w),
        GameScreen::Victory => render_victory(&mut w, state),
    }

    w.flush()
}

// ============================================================================
// SCREEN RENDERERS
// ============================================================================

fn render_creation(w: &mut AnsiWriter, step: &CreationStep, state: &GameState) {
    w.clear_screen();
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CREATE YOUR HERO");
    w.reset_color();
    w.writeln("");

    match step {
        CreationStep::EnterName => {
            w.set_fg(Color::White);
            w.writeln("  A red dragon terrorizes our land.");
            w.writeln("  Children disappear in the night.");
            w.writeln("  We need a hero.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("  What is your name, brave warrior? ");
            w.reset_color();
        }
        CreationStep::SelectSex => {
            w.set_fg(Color::White);
            w.writeln("  Your legend shall be written in the stars.");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("  Are you:");
            w.writeln("");
            w.write_str("    [M] ");
            w.set_fg(Color::White);
            w.writeln("Male");
            w.set_fg(Color::LightCyan);
            w.write_str("    [F] ");
            w.set_fg(Color::White);
            w.writeln("Female");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("  > ");
            w.reset_color();
        }
    }

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::LightRed);
        w.writeln(&format!("  {}", msg));
        w.reset_color();
    }
}

fn render_intro(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  The village of Silverton has known peace for generations.");
    w.writeln("  But darkness has come. The Red Dragon.");
    w.writeln("");
    w.set_fg(Color::LightRed);
    w.writeln("  Its shadow falls across the land.");
    w.writeln("  Children vanish in the night.");
    w.writeln("  None who seek the beast return.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  But you, {}, are different.", state.char_name));
    w.writeln("  You will train. You will grow strong.");
    w.writeln("  And you will slay the dragon.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to begin your legend...");
    w.reset_color();
}

fn render_town(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE TOWN OF SILVERTON");
    w.reset_color();
    w.writeln("");

    // Main locations
    let menu_items = [
        ('F', "The Dark Forest", "Hunt monsters for XP and gold"),
        ('T', "Turgon's Training", "Challenge masters to level up"),
        ('W', "Weapons Shop", "Buy better weapons"),
        ('A', "Armor Shop", "Buy better armor"),
        ('H', "Healer's Hut", "Restore your health"),
        ('B', "The Bank", "Store your gold safely"),
        ('K', "King's Court", "News and quests"),
        ('I', "The Inn", "Rest and hear gossip"),
    ];

    for (key, name, desc) in menu_items {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("  [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<20}", name));
        w.set_fg(Color::DarkGray);
        w.writeln(&format!("- {}", desc));
    }

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.writeln("  SOCIAL:");
    w.reset_color();

    let social_items = [
        ('V', "Violet's House", "Visit the charming barmaid"),
        ('S', "Seth's Tavern", "Listen to the handsome bard"),
        ('P', "The Arena", "Challenge other warriors"),
        ('O', "Other Places", "IGM locations"),
    ];

    for (key, name, desc) in social_items {
        w.set_fg(Color::LightMagenta);
        w.write_str(&format!("  [{}] ", key));
        w.set_fg(Color::White);
        w.write_str(&format!("{:<20}", name));
        w.set_fg(Color::DarkGray);
        w.writeln(&format!("- {}", desc));
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  [Y] Your Stats    [L] Leaderboard    [Q] Quit");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_forest(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Green);
    w.bold();
    w.writeln("  THE DARK FOREST");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Ancient trees block out the sun.");
    w.writeln("  Shadows move between the trunks.");
    w.writeln("  Danger lurks everywhere.");
    w.writeln("");

    w.set_fg(Color::LightGreen);
    w.writeln(&format!("  Forest fights remaining: {}", state.forest_fights_remaining()));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [L] ");
    w.set_fg(Color::White);
    w.writeln("Look for something to kill");

    w.set_fg(Color::LightCyan);
    w.write_str("  [H] ");
    w.set_fg(Color::White);
    w.writeln("Hunt deeper in the forest");

    if state.level >= 12 {
        w.set_fg(Color::LightRed);
        w.write_str("  [D] ");
        w.set_fg(Color::Red);
        w.bold();
        w.writeln("Search for THE RED DRAGON");
        w.reset_color();
    }

    w.set_fg(Color::LightCyan);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.writeln("Return to town");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_combat(w: &mut AnsiWriter, state: &GameState, combat: &CombatState) {
    w.clear_screen();

    // Enemy display
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln(&format!("  {} ATTACKS!", combat.enemy_name.to_uppercase()));
    w.reset_color();
    w.writeln("");

    // Combat status
    w.set_fg(Color::Red);
    let hp_bar = render_hp_bar(combat.enemy_hp, combat.enemy_max_hp, 20);
    w.writeln(&format!("  Enemy HP: {} {}/{}", hp_bar, combat.enemy_hp, combat.enemy_max_hp));

    w.set_fg(Color::Green);
    let player_bar = render_hp_bar(state.hp_current, state.hp_max, 20);
    w.writeln(&format!("  Your HP:  {} {}/{}", player_bar, state.hp_current, state.hp_max));

    // Combat log
    w.writeln("");
    w.set_fg(Color::LightGray);
    for line in combat.combat_log.iter().rev().take(5) {
        w.writeln(&format!("  {}", line));
    }

    // Actions
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  What will you do?");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [A] ");
    w.set_fg(Color::White);
    w.writeln("Attack");

    w.set_fg(Color::LightCyan);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.writeln("Run away");

    // Show available skills
    if state.has_skill("power_strike") && state.can_use_skill("power_strike") {
        w.set_fg(Color::LightMagenta);
        w.write_str("  [P] ");
        w.set_fg(Color::Magenta);
        w.writeln("Power Strike (2x damage)");
    }
    if state.has_skill("fireball") && state.can_use_skill("fireball") {
        w.set_fg(Color::LightRed);
        w.write_str("  [F] ");
        w.set_fg(Color::Red);
        w.writeln("Fireball");
    }
    if state.has_skill("heal") && state.can_use_skill("heal") {
        w.set_fg(Color::LightGreen);
        w.write_str("  [H] ");
        w.set_fg(Color::Green);
        w.writeln("Heal (30% HP)");
    }

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_hp_bar(current: u32, max: u32, width: usize) -> String {
    let filled = ((current as f32 / max as f32) * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

fn render_forest_event(w: &mut AnsiWriter, _state: &GameState, event: &ForestEvent) {
    w.clear_screen();
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SOMETHING HAPPENS!");
    w.reset_color();
    w.writeln("");

    match event {
        ForestEvent::FindGold { amount } => {
            w.set_fg(Color::Yellow);
            w.writeln("  Glinting in the underbrush, you spot something!");
            w.writeln("");
            w.set_fg(Color::LightGreen);
            w.writeln(&format!("  You found {} gold pieces!", amount));
        }
        ForestEvent::FindPotion { heal_amount } => {
            w.set_fg(Color::LightCyan);
            w.writeln("  A vial lies abandoned on a tree stump.");
            w.writeln("");
            w.set_fg(Color::LightGreen);
            w.writeln(&format!("  A healing potion! (+{} HP)", heal_amount));
        }
        ForestEvent::FairyEncounter => {
            w.set_fg(Color::LightMagenta);
            w.writeln("  A tiny light dances before you...");
            w.writeln("");
            w.writeln("  A fairy! She offers to protect you!");
        }
        ForestEvent::OldManRiddle { .. } => {
            w.set_fg(Color::White);
            w.writeln("  An old man blocks the path.");
            w.writeln("");
            w.writeln("  \"Answer my riddle to pass!\"");
        }
        ForestEvent::MysteriousChest { is_trap, contents: _ } => {
            w.set_fg(Color::Brown);
            w.writeln("  You discover a mysterious chest!");
            if *is_trap {
                w.set_fg(Color::LightRed);
                w.writeln("  It's a trap!");
            } else {
                w.set_fg(Color::LightGreen);
                w.writeln("  Inside you find treasure!");
            }
        }
        ForestEvent::Nothing => {
            w.set_fg(Color::LightGray);
            w.writeln("  The forest is quiet today.");
        }
        ForestEvent::SecretHint { hint } => {
            w.set_fg(Color::LightMagenta);
            w.writeln("  An ancient scroll lies on the ground:");
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln(&format!("  \"{}\"", hint));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_training(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  TURGON'S TRAINING GROUNDS");
    w.reset_color();
    w.writeln("");

    if let Some(master) = get_master(state.level) {
        w.set_fg(Color::White);
        w.writeln(&format!("  Current Master: {}", master.name));
        w.writeln(&format!("  HP: {}  STR: {}  DEF: {}", master.hp, master.strength, master.defense));
        w.writeln("");

        if state.experience >= master.xp_required {
            w.set_fg(Color::LightGreen);
            w.writeln("  You are READY to challenge the master!");
        } else {
            w.set_fg(Color::LightGray);
            w.writeln(&format!(
                "  XP needed: {} (you have {})",
                master.xp_required, state.experience
            ));
        }

        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.write_str("  [C] ");
        w.set_fg(Color::White);
        w.writeln("Challenge the Master");
    } else {
        w.set_fg(Color::LightGreen);
        w.bold();
        w.writeln("  You have defeated all masters!");
        w.writeln("  Seek the Red Dragon in the forest!");
        w.reset_color();
    }

    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Return to town");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_weapon_shop(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE WEAPONS SHOP");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  \"The finest blades in the realm!\"");
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("     #  Weapon              Damage     Price");
    w.writeln(&format!("    {}", "\u{2500}".repeat(50)));
    w.reset_color();

    let available: Vec<_> = WEAPONS.iter()
        .filter(|w| w.level_required <= state.level)
        .collect();

    for (i, weapon) in available.iter().enumerate() {
        let is_owned = state.equipment.weapon == weapon.key;

        if is_owned {
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("    {} ", i + 1));
            w.write_str(&format!("{:<20}", weapon.name));
            w.write_str(&format!("{:>6}", weapon.damage));
            w.writeln("   (equipped)");
        } else if state.gold_pocket >= weapon.price {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    {} ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20}", weapon.name));
            w.set_fg(Color::LightGray);
            w.write_str(&format!("{:>6}", weapon.damage));
            w.set_fg(Color::Yellow);
            w.writeln(&format!("   {:>10}", format_gold(weapon.price)));
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str(&format!("    {} ", i + 1));
            w.write_str(&format!("{:<20}", weapon.name));
            w.write_str(&format!("{:>6}", weapon.damage));
            w.set_fg(Color::Red);
            w.writeln(&format!("   {:>10}", format_gold(weapon.price)));
        }
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Enter number to buy, or [Q] to leave.");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_armor_shop(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE ARMOR SHOP");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  \"Protection for the discerning warrior!\"");
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln("     #  Armor               Defense    Price");
    w.writeln(&format!("    {}", "\u{2500}".repeat(50)));
    w.reset_color();

    let available: Vec<_> = ARMOR.iter()
        .filter(|a| a.level_required <= state.level)
        .collect();

    for (i, armor) in available.iter().enumerate() {
        let is_owned = state.equipment.armor == armor.key;

        if is_owned {
            w.set_fg(Color::LightGreen);
            w.write_str(&format!("    {} ", i + 1));
            w.write_str(&format!("{:<20}", armor.name));
            w.write_str(&format!("{:>6}", armor.defense));
            w.writeln("   (equipped)");
        } else if state.gold_pocket >= armor.price {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("    {} ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20}", armor.name));
            w.set_fg(Color::LightGray);
            w.write_str(&format!("{:>6}", armor.defense));
            w.set_fg(Color::Yellow);
            w.writeln(&format!("   {:>10}", format_gold(armor.price)));
        } else {
            w.set_fg(Color::DarkGray);
            w.write_str(&format!("    {} ", i + 1));
            w.write_str(&format!("{:<20}", armor.name));
            w.write_str(&format!("{:>6}", armor.defense));
            w.set_fg(Color::Red);
            w.writeln(&format!("   {:>10}", format_gold(armor.price)));
        }
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Enter number to buy, or [Q] to leave.");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_healer(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("  THE HEALER'S HUT");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  An old woman looks at your wounds.");
    w.writeln("");

    let damage = state.hp_max - state.hp_current;
    let cost_per_hp = 5 + (state.level as i64);
    let total_cost = damage as i64 * cost_per_hp;

    if damage == 0 {
        w.set_fg(Color::LightGreen);
        w.writeln("  \"You are in perfect health, child.\"");
    } else {
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  \"I can heal {} damage for {} gold.\"", damage, total_cost));
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.write_str("  [H] ");
        w.set_fg(Color::White);
        w.writeln("Heal me");
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_bank(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE SILVERTON BANK");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  \"Your gold is safe with us.\"");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Gold in pocket: {}", format_gold(state.gold_pocket)));
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("  Gold in bank:   {}", format_gold(state.gold_bank)));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [D] ");
    w.set_fg(Color::White);
    w.writeln("Deposit all gold");

    w.set_fg(Color::LightCyan);
    w.write_str("  [W] ");
    w.set_fg(Color::White);
    w.writeln("Withdraw all gold");

    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_kings_court(w: &mut AnsiWriter, _state: &GameState) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  THE KING'S COURT");
    w.reset_color();
    w.writeln("");

    // Generate daily news
    use chrono::Local;
    let today = Local::now().format("%Y%m%d").to_string();
    let day_seed: u64 = today.parse().unwrap_or(0);
    let news = super::events::generate_daily_news(day_seed);

    w.set_fg(Color::White);
    w.writeln("  Today's News:");
    w.writeln("");
    for item in news {
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  * {}", item));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_inn(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Brown);
    w.bold();
    w.writeln("  THE RED DRAGON INN");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  The fire crackles warmly. Travelers share tales.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.writeln("Rest for the night");

    w.set_fg(Color::LightCyan);
    w.write_str("  [G] ");
    w.set_fg(Color::White);
    w.writeln("Listen to gossip");

    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_violet(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Magenta);
    w.bold();
    w.writeln("  VIOLET'S HOUSE");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.writeln("  The charming barmaid greets you with a smile.");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Affection: {}/100", state.romance.violet_affection));
    if let Some(ref spouse) = state.romance.spouse {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  Married to: {}", spouse));
    }
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [F] ");
    w.set_fg(Color::White);
    w.writeln(&format!("Flirt with Violet ({}/{} today)", state.romance.flirts_today, 5));

    if state.romance.violet_affection >= 100 && state.romance.spouse.is_none() {
        w.set_fg(Color::LightMagenta);
        w.write_str("  [P] ");
        w.set_fg(Color::Magenta);
        w.writeln("Propose marriage (1,000 gold)");
    }

    if state.romance.spouse.is_some() {
        w.set_fg(Color::Red);
        w.write_str("  [D] ");
        w.set_fg(Color::LightRed);
        w.writeln("Divorce");
    }

    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_seth(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Cyan);
    w.bold();
    w.writeln("  SETH'S TAVERN");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  The handsome bard strums his lute.");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Affection: {}/100", state.romance.seth_affection));
    if let Some(ref spouse) = state.romance.spouse {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  Married to: {}", spouse));
    }
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [F] ");
    w.set_fg(Color::White);
    w.writeln(&format!("Flirt with Seth ({}/{} today)", state.romance.flirts_today, 5));

    if state.romance.seth_affection >= 100 && state.romance.spouse.is_none() {
        w.set_fg(Color::LightCyan);
        w.write_str("  [P] ");
        w.set_fg(Color::Cyan);
        w.writeln("Propose marriage (1,000 gold)");
    }

    if state.romance.spouse.is_some() {
        w.set_fg(Color::Red);
        w.write_str("  [D] ");
        w.set_fg(Color::LightRed);
        w.writeln("Divorce");
    }

    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_arena(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_status_bar(w, state);

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("  THE SLAUGHTER ARENA");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Challenge other warriors to combat!");
    w.writeln(&format!("  Player attacks remaining: {}", state.player_attacks_remaining()));
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  (PvP coming soon...)");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_other_places(w: &mut AnsiWriter, _state: &GameState, flow: &DragonSlayerFlow) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  OTHER PLACES");
    w.reset_color();
    w.writeln("");

    let locations = flow.igm_registry.get_locations();
    if locations.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No additional locations available.");
    } else {
        for module in locations {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("  [{}] ", module.hotkey.to_uppercase()));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<20}", module.name));
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("- {}", module.description));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Return to town");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_igm_location(w: &mut AnsiWriter, _state: &GameState, module_id: &str, flow: &DragonSlayerFlow) {
    render_header(w);

    if let Some(module) = flow.igm_registry.get(module_id) {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln(&format!("  {}", module.name.to_uppercase()));
        w.reset_color();
        w.writeln("");
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  {}", module.description));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_stats(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  {} - WARRIOR STATS", state.char_name.to_uppercase()));
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln(&format!("  Level: {}", state.level));
    w.writeln(&format!("  Experience: {}", format_gold(state.experience)));
    w.writeln(&format!("  HP: {}/{}", state.hp_current, state.hp_max));
    w.writeln("");
    w.writeln(&format!("  Strength: {} (+{} from weapon)", state.stats.strength, get_weapon(&state.equipment.weapon).map(|w| w.damage).unwrap_or(0)));
    w.writeln(&format!("  Defense: {} (+{} from armor)", state.stats.defense, get_armor(&state.equipment.armor).map(|a| a.defense).unwrap_or(0)));
    w.writeln(&format!("  Vitality: {}", state.stats.vitality));
    w.writeln(&format!("  Charm: {}", state.charm));
    w.writeln("");
    w.writeln(&format!("  Gold (pocket): {}", format_gold(state.gold_pocket)));
    w.writeln(&format!("  Gold (bank): {}", format_gold(state.gold_bank)));
    w.writeln("");
    w.writeln(&format!("  Kills: {}", state.kills));
    w.writeln(&format!("  Deaths: {}", state.deaths));
    w.writeln(&format!("  Dragon Kills: {}", state.dragon_kills));
    w.writeln("");
    if state.has_fairy {
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  Fairy Protection: {} uses", state.fairy_uses));
    }

    w.set_fg(Color::LightCyan);
    w.writeln("");
    w.writeln("  Press any key to return...");
    w.reset_color();
}

fn render_leaderboard(w: &mut AnsiWriter) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  DRAGON SLAYER - HALL OF HEROES");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Loading leaderboard...");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();
}

/// Render the leaderboard with actual data (called from session handler)
pub fn render_leaderboard_screen(entries: &[(i64, String, String, u32, u8)]) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  DRAGON SLAYER - HALL OF HEROES");
    w.reset_color();
    w.writeln("");

    if entries.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No heroes have slain the dragon yet.");
        w.writeln("");
        w.writeln("  Will you be the first?");
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("  Rank  Player            Hero Name         Dragons  Level");
        w.writeln(&format!("  {}", "\u{2500}".repeat(60)));
        w.reset_color();

        for (rank, handle, char_name, dragons, level) in entries {
            let color = match rank {
                1 => Color::Yellow,
                2 => Color::LightGray,
                3 => Color::Brown,
                _ => Color::White,
            };

            w.set_fg(color);
            w.write_str(&format!("  {:>4}  ", rank));
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("{:<16}  ", handle));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<16}  ", char_name));
            w.set_fg(Color::LightRed);
            w.write_str(&format!("{:>7}  ", dragons));
            w.set_fg(Color::LightGreen);
            w.writeln(&format!("{:>5}", level));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

fn render_dragon_hunt(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();

    w.set_fg(Color::Red);
    w.bold();
    w.writeln("");
    w.writeln("   _____ _            ____           _   ____                              ");
    w.writeln("  |_   _| |__   ___  |  _ \\ ___  __| | |  _ \\ _ __ __ _  __ _  ___  _ __  ");
    w.writeln("    | | | '_ \\ / _ \\ | |_) / _ \\/ _` | | | | | '__/ _` |/ _` |/ _ \\| '_ \\ ");
    w.writeln("    | | | | | |  __/ |  _ <  __/ (_| | | |_| | | | (_| | (_| | (_) | | | |");
    w.writeln("    |_| |_| |_|\\___| |_| \\_\\___|\\__,_| |____/|_|  \\__,_|\\__, |\\___/|_| |_|");
    w.writeln("                                                        |___/              ");
    w.reset_color();

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln("");
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  You venture deep into the dragon's territory...");
    w.writeln("  The air grows hot. Sulfur burns your nostrils.");
    w.writeln("");

    w.set_fg(Color::LightRed);
    w.write_str("  [S] ");
    w.set_fg(Color::Red);
    w.writeln("Search for the Red Dragon");

    w.set_fg(Color::LightCyan);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.writeln("Return to the forest");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_game_over(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();

    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln("");
    w.writeln("   ____                        ___                 ");
    w.writeln("  / ___| __ _ _ __ ___   ___  / _ \\__   _____ _ __ ");
    w.writeln(" | |  _ / _` | '_ ` _ \\ / _ \\| | | \\ \\ / / _ \\ '__|");
    w.writeln(" | |_| | (_| | | | | | |  __/| |_| |\\ V /  __/ |   ");
    w.writeln("  \\____|\\__,_|_| |_| |_|\\___| \\___/  \\_/ \\___|_|   ");
    w.reset_color();

    w.writeln("");
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Your journey has ended... for today.");
    w.writeln("  Return tomorrow to continue your quest.");
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
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Your progress will be saved.");
    w.writeln("  Return anytime to continue your adventure.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();
}

fn render_victory(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  ╔═══════════════════════════════════════════════════════════════════╗");
    w.writeln("  ║                                                                   ║");
    w.writeln("  ║   ██╗   ██╗██╗ ██████╗████████╗ ██████╗ ██████╗ ██╗   ██╗██╗      ║");
    w.writeln("  ║   ██║   ██║██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝██║      ║");
    w.writeln("  ║   ██║   ██║██║██║        ██║   ██║   ██║██████╔╝ ╚████╔╝ ██║      ║");
    w.writeln("  ║   ╚██╗ ██╔╝██║██║        ██║   ██║   ██║██╔══██╗  ╚██╔╝  ╚═╝      ║");
    w.writeln("  ║    ╚████╔╝ ██║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║   ██╗      ║");
    w.writeln("  ║     ╚═══╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝      ║");
    w.writeln("  ║                                                                   ║");
    w.writeln("  ╚═══════════════════════════════════════════════════════════════════╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGreen);
    w.bold();
    w.writeln("        THE RED DRAGON HAS BEEN SLAIN!");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  {}, you have saved the children of Silverton!", state.char_name));
    w.writeln("  Your name will be remembered for generations.");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Dragon Kills: {}", state.dragon_kills));
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_gold() {
        assert_eq!(format_gold(0), "0");
        assert_eq!(format_gold(100), "100");
        assert_eq!(format_gold(1000), "1,000");
        assert_eq!(format_gold(1234567), "1,234,567");
        assert_eq!(format_gold(-500), "-500");
    }

    #[test]
    fn test_hp_bar() {
        assert_eq!(render_hp_bar(100, 100, 10), "[==========]");
        assert_eq!(render_hp_bar(50, 100, 10), "[=====     ]");
        assert_eq!(render_hp_bar(0, 100, 10), "[          ]");
    }

    #[test]
    fn test_render_no_panic() {
        let flow = DragonSlayerFlow::new();
        let output = render_screen(&flow);
        assert!(!output.is_empty());
    }
}
