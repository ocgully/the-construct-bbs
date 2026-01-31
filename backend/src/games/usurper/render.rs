//! ANSI rendering for Usurper
//!
//! Dark fantasy themed UI with deep purples, blood reds, and shadowy grays.

use crate::terminal::{AnsiWriter, Color};
use super::state::GameState;
use super::screen::{GameScreen, CreationStage, ShopType, CombatState};
use super::data::{CharacterClass, DungeonTier, get_dungeon_by_level, get_monster, SUBSTANCES, EQUIPMENT_ITEMS};
use super::substances::get_substance_display;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format gold with thousands separator
pub fn format_gold(gold: u64) -> String {
    let gold_str = format!("{}", gold);
    let mut result = String::new();
    for (i, c) in gold_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Get tier color
fn tier_color(tier: DungeonTier) -> Color {
    match tier {
        DungeonTier::Surface => Color::LightGreen,
        DungeonTier::Upper => Color::Yellow,
        DungeonTier::Deep => Color::Brown,
        DungeonTier::Abyss => Color::LightMagenta,
        DungeonTier::Depths => Color::LightRed,
        DungeonTier::Bottom => Color::White,
    }
}

/// Render game header with ASCII art title
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::Magenta);
    w.bold();
    w.writeln("");
    w.writeln("  ██╗   ██╗███████╗██╗   ██╗██████╗ ██████╗ ███████╗██████╗ ");
    w.writeln("  ██║   ██║██╔════╝██║   ██║██╔══██╗██╔══██╗██╔════╝██╔══██╗");
    w.writeln("  ██║   ██║███████╗██║   ██║██████╔╝██████╔╝█████╗  ██████╔╝");
    w.writeln("  ██║   ██║╚════██║██║   ██║██╔══██╗██╔═══╝ ██╔══╝  ██╔══██╗");
    w.writeln("  ╚██████╔╝███████║╚██████╔╝██║  ██║██║     ███████╗██║  ██║");
    w.writeln("   ╚═════╝ ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln("          --- The Mountain of Durunghins Awaits ---");
    w.reset_color();
}

/// Render status bar
fn render_status_bar(w: &mut AnsiWriter, state: &GameState) {
    let dungeon = get_dungeon_by_level(state.current_dungeon_level);
    let stats = state.effective_stats();

    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));

    // Line 1: Character info
    w.set_fg(Color::LightMagenta);
    w.write_str(&format!(" {} ", state.character_name));
    w.set_fg(Color::DarkGray);
    w.write_str("| ");
    w.set_fg(Color::White);
    w.write_str(&format!("Lv {} {}", state.level, state.class.name()));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // HP with color coding
    let hp_pct = (state.hp as f32 / state.max_hp as f32) * 100.0;
    let hp_color = if hp_pct > 70.0 {
        Color::LightGreen
    } else if hp_pct > 30.0 {
        Color::Yellow
    } else {
        Color::LightRed
    };
    w.set_fg(hp_color);
    w.write_str(&format!("HP:{}/{}", state.hp, state.max_hp));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Mental stability with warning colors
    let mental_color = if state.mental_stability > 50 {
        Color::LightCyan
    } else if state.mental_stability > 20 {
        Color::Yellow
    } else if state.mental_stability > 0 {
        Color::LightRed
    } else {
        Color::Red // Psychosis!
    };
    w.set_fg(mental_color);
    w.write_str(&format!("MEN:{}", state.mental_stability));
    if state.mental_stability <= 0 {
        w.write_str(" [PSYCHOSIS]");
    }
    w.writeln("");

    // Line 2: Resources and location
    w.set_fg(Color::Yellow);
    w.write_str(&format!(" Gold: {}", format_gold(state.gold)));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::Cyan);
    w.write_str(&format!("Turns: {}", state.turns_remaining));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    if state.in_town {
        w.set_fg(Color::LightGreen);
        w.write_str("Town");
    } else {
        w.set_fg(tier_color(dungeon.tier));
        w.write_str(&format!("{} (Lv{})", dungeon.name, state.current_dungeon_level));
    }
    w.writeln("");

    // Line 3: Combat stats
    w.set_fg(Color::LightRed);
    w.write_str(&format!(" DMG:{}", stats.damage));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::LightBlue);
    w.write_str(&format!("DEF:{}", stats.defense));
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");
    w.set_fg(Color::White);
    w.write_str(&format!("XP:{}/{}", state.experience, state.experience_to_next));

    // Active effects
    if !state.active_effects.is_empty() {
        w.set_fg(Color::DarkGray);
        w.write_str(" | ");
        w.set_fg(Color::LightMagenta);
        w.write_str(&format!("[{} effects]", state.active_effects.len()));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(80));
    w.reset_color();
}

// ============================================================================
// SCREEN RENDERERS
// ============================================================================

/// Render intro screen
pub fn render_intro(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    if state.character_name.is_empty() {
        // New game intro
        w.set_fg(Color::LightGray);
        w.writeln("  The ancient mountain of Durunghins looms before you.");
        w.writeln("  Legends speak of treasures beyond measure hidden in its depths,");
        w.writeln("  guarded by creatures of nightmare and madness.");
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln("  But the greatest prize lies at the very bottom...");
        w.writeln("  THE SUPREME BEING - an entity of godlike power.");
        w.writeln("");
        w.set_fg(Color::LightRed);
        w.writeln("  Many have descended. Few return. None have reached the bottom.");
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln("  Will you claim ultimate power? Or descend into madness?");
    } else {
        // Returning player
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("  Welcome back, {}...", state.character_name));
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln(&format!("  Level {} {}", state.level, state.class.name()));
        w.writeln(&format!("  Deepest dungeon reached: Level {}", state.deepest_dungeon));
        if state.is_king {
            w.set_fg(Color::Yellow);
            w.writeln("  >> You are the KING of Durunghins! <<");
        }
        if state.godhood_level > 0 {
            w.set_fg(Color::LightMagenta);
            w.writeln(&format!("  >> Godhood Level {} <<", state.godhood_level));
        }
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render character creation
pub fn render_creation(state: &GameState, stage: &CreationStage) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CREATE YOUR CHARACTER");
    w.reset_color();

    match stage {
        CreationStage::Name { buffer } => {
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln("  What is your name, adventurer?");
            w.writeln("  (2-20 characters)");
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("  Name: ");
            w.set_fg(Color::White);
            w.write_str(buffer);
            w.write_str("_");

            if let Some(ref msg) = state.last_message {
                w.writeln("");
                w.set_fg(Color::LightRed);
                w.writeln(&format!("  {}", msg));
            }
        }
        CreationStage::Class => {
            w.writeln("");
            w.set_fg(Color::White);
            w.writeln(&format!("  {}, choose your path:", state.character_name));
            w.writeln("");

            let classes = [
                (CharacterClass::Warrior, "1"),
                (CharacterClass::Rogue, "2"),
                (CharacterClass::Mage, "3"),
                (CharacterClass::Cleric, "4"),
                (CharacterClass::Berserker, "5"),
            ];

            for (class, key) in classes {
                let (str, agi, vit, int, cha, mental) = class.base_stats();
                w.set_fg(Color::LightCyan);
                w.write_str(&format!("  [{}] ", key));
                w.set_fg(Color::White);
                w.write_str(&format!("{:<12}", class.name()));
                w.set_fg(Color::DarkGray);
                w.writeln(&format!(
                    " STR:{:2} AGI:{:2} VIT:{:2} INT:{:2} CHA:{:2} MEN:{:3}",
                    str, agi, vit, int, cha, mental
                ));
                w.set_fg(Color::LightGray);
                w.writeln(&format!("       {}", class.description()));
            }
        }
    }

    w.reset_color();
    w.flush()
}

/// Render town menu
pub fn render_town(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    // Show last message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  TOWN OF DURUNGHINS");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("  The last bastion of light before the darkness below.");
    w.writeln("");

    // Main options
    let menu_items = [
        ("D", "Descend into Dungeon", true),
        ("W", "Weapon Shop", true),
        ("A", "Armor Shop", true),
        ("S", "General Store", true),
        ("H", "Temple of Healing", true),
        ("B", "Bank", true),
        ("P", "Potion & Substance Dealer", true),
        ("E", "Equipment", true),
        ("C", "Character Stats", true),
        ("R", "Romance", true),
        ("T", "Clan Hall", true),
        ("V", "PvP Arena", true),
        ("K", "King's Throne", state.level >= 50 || state.is_king),
        ("Q", "Quest Log", true),
        ("L", "Leaderboard", true),
        ("X", "Save & Quit", true),
    ];

    for (key, label, enabled) in menu_items {
        if enabled {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("  [{}] ", key));
            w.set_fg(Color::White);
            w.writeln(label);
        }
    }

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render dungeon screen
pub fn render_dungeon(state: &GameState, combat_state: &Option<CombatState>) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    let dungeon = get_dungeon_by_level(state.current_dungeon_level);

    if let Some(combat) = combat_state {
        // Combat mode
        return render_combat(&mut w, state, combat);
    }

    // Exploration mode
    w.writeln("");
    w.set_fg(tier_color(dungeon.tier));
    w.bold();
    w.writeln(&format!("  {} - Level {}", dungeon.name, state.current_dungeon_level));
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  {}", dungeon.description));

    // Show last message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  What do you do?");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [E] ");
    w.set_fg(Color::White);
    w.writeln("Explore (may encounter monsters)");

    w.set_fg(Color::LightCyan);
    w.write_str("  [D] ");
    w.set_fg(Color::White);
    w.writeln("Descend deeper");

    w.set_fg(Color::LightCyan);
    w.write_str("  [U] ");
    w.set_fg(Color::White);
    w.writeln("Ascend upward");

    w.set_fg(Color::LightCyan);
    w.write_str("  [I] ");
    w.set_fg(Color::White);
    w.writeln("Use Item/Substance");

    w.set_fg(Color::LightCyan);
    w.write_str("  [T] ");
    w.set_fg(Color::White);
    w.writeln("Return to Town");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render combat screen
fn render_combat(w: &mut AnsiWriter, state: &GameState, combat: &CombatState) -> String {
    let monster = get_monster(&combat.monster_key).expect("Invalid monster");

    w.writeln("");
    w.set_fg(Color::LightRed);
    w.bold();
    w.writeln(&format!("  !! {} ATTACKS !!", monster.name.to_uppercase()));
    w.reset_color();

    // Monster health bar
    let hp_pct = (combat.monster_hp as f32 / combat.monster_max_hp as f32) * 100.0;
    let bar_filled = (hp_pct / 5.0) as usize;
    let bar_empty = 20 - bar_filled;

    w.set_fg(Color::LightGray);
    w.writeln(&format!("  {}", monster.description));
    w.writeln("");
    w.set_fg(Color::White);
    w.write_str("  Monster HP: ");
    w.set_fg(Color::LightRed);
    w.write_str(&"\u{2588}".repeat(bar_filled));
    w.set_fg(Color::DarkGray);
    w.write_str(&"\u{2591}".repeat(bar_empty));
    w.set_fg(Color::White);
    w.writeln(&format!(" {}/{}", combat.monster_hp, combat.monster_max_hp));

    w.writeln(&format!("  Round: {}", combat.round));

    // Show last message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  Choose your action:");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [A] ");
    w.set_fg(Color::White);
    w.writeln("Attack");

    w.set_fg(Color::LightCyan);
    w.write_str("  [D] ");
    w.set_fg(Color::White);
    w.writeln("Defend");

    w.set_fg(Color::LightCyan);
    w.write_str("  [S] ");
    w.set_fg(Color::White);
    w.writeln(&format!("Use {} Skill", state.class.name()));

    w.set_fg(Color::LightCyan);
    w.write_str("  [R] ");
    w.set_fg(Color::White);
    w.writeln("Run");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render shop screen
pub fn render_shop(state: &GameState, shop_type: &ShopType) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    let (title, items) = match shop_type {
        ShopType::Weapons => {
            let items: Vec<_> = EQUIPMENT_ITEMS.iter()
                .filter(|i| i.slot == super::data::EquipmentSlot::Weapon && i.min_level <= state.level + 10)
                .collect();
            ("WEAPON SHOP", items)
        }
        ShopType::Armor => {
            let items: Vec<_> = EQUIPMENT_ITEMS.iter()
                .filter(|i| matches!(i.slot,
                    super::data::EquipmentSlot::Armor |
                    super::data::EquipmentSlot::Helmet |
                    super::data::EquipmentSlot::Shield |
                    super::data::EquipmentSlot::Boots |
                    super::data::EquipmentSlot::Gloves
                ) && i.min_level <= state.level + 10)
                .collect();
            ("ARMOR SHOP", items)
        }
        ShopType::General => {
            let items: Vec<_> = EQUIPMENT_ITEMS.iter()
                .filter(|i| matches!(i.slot,
                    super::data::EquipmentSlot::RingLeft |
                    super::data::EquipmentSlot::RingRight |
                    super::data::EquipmentSlot::Amulet |
                    super::data::EquipmentSlot::Cloak
                ) && i.min_level <= state.level + 10)
                .collect();
            ("GENERAL STORE", items)
        }
    };

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  {}", title));
    w.reset_color();

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("     # Item                 Lv   Stats           Price");
    w.writeln(&format!("    {}", "\u{2500}".repeat(60)));
    w.reset_color();

    for (i, item) in items.iter().enumerate() {
        let can_afford = state.gold >= item.price as u64;
        let color = if can_afford { Color::White } else { Color::DarkGray };

        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{:2}] ", i + 1));
        w.set_fg(color);
        w.write_str(&format!("{:<20} ", item.name));
        w.set_fg(Color::Yellow);
        w.write_str(&format!("{:2}  ", item.min_level));

        // Show stats
        let mut stats = Vec::new();
        if item.stat_bonuses.damage != 0 { stats.push(format!("D{:+}", item.stat_bonuses.damage)); }
        if item.stat_bonuses.defense != 0 { stats.push(format!("F{:+}", item.stat_bonuses.defense)); }
        if item.stat_bonuses.strength != 0 { stats.push(format!("S{:+}", item.stat_bonuses.strength)); }
        w.set_fg(Color::LightGray);
        w.write_str(&format!("{:<15} ", stats.join(" ")));

        w.set_fg(if can_afford { Color::LightGreen } else { Color::DarkGray });
        w.writeln(&format!("{:>8}", format_gold(item.price as u64)));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render healer screen
pub fn render_healer(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  TEMPLE OF HEALING");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("  \"Light banishes the darkness within...\"");

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");

    let heal_cost = ((state.max_hp - state.hp) as u64 * 2).max(10);
    let mental_cost = ((state.max_mental_stability - state.mental_stability).abs() as u64 * 5).max(20);
    let addiction_total: u32 = state.addictions.values().sum();
    let cure_cost = (addiction_total as u64) * 100;

    w.set_fg(Color::LightCyan);
    w.write_str("  [H] ");
    w.set_fg(Color::White);
    w.write_str("Heal Wounds ");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("({} gold)", format_gold(heal_cost)));

    w.set_fg(Color::LightCyan);
    w.write_str("  [M] ");
    w.set_fg(Color::White);
    w.write_str("Restore Sanity ");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("({} gold)", format_gold(mental_cost)));

    if addiction_total > 0 {
        w.set_fg(Color::LightCyan);
        w.write_str("  [C] ");
        w.set_fg(Color::White);
        w.write_str("Cure Addictions ");
        w.set_fg(Color::Yellow);
        w.writeln(&format!("({} gold)", format_gold(cure_cost)));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render bank screen
pub fn render_bank(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  DURUNGHINS BANK");
    w.reset_color();

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  On Hand: {} gold", format_gold(state.gold)));
    w.writeln(&format!("  In Bank: {} gold", format_gold(state.bank_gold)));
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.write_str("  [D] ");
    w.set_fg(Color::White);
    w.writeln("Deposit All");

    w.set_fg(Color::LightCyan);
    w.write_str("  [W] ");
    w.set_fg(Color::White);
    w.writeln("Withdraw All");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render substance dealer screen
pub fn render_substance(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Magenta);
    w.bold();
    w.writeln("  THE ALCHEMIST'S DEN");
    w.reset_color();
    w.set_fg(Color::LightGray);
    w.writeln("  \"Power comes at a price... your sanity.\"");

    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("     # Substance            Effects            MEN    Price");
    w.writeln(&format!("    {}", "\u{2500}".repeat(65)));
    w.reset_color();

    for (i, substance) in SUBSTANCES.iter().enumerate() {
        let can_afford = state.gold >= substance.price as u64;
        let in_inventory = state.inventory.get(substance.key).copied().unwrap_or(0);

        w.set_fg(Color::LightCyan);
        w.write_str(&format!("    [{:2}] ", i + 1));

        w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
        w.write_str(&format!("{:<18} ", substance.name));

        w.set_fg(Color::LightGray);
        let effects = get_substance_display(substance);
        w.write_str(&format!("{:<18} ", &effects[..effects.len().min(18)]));

        // Mental cost color
        let mental_color = if substance.mental_cost >= 0 { Color::LightGreen } else { Color::LightRed };
        w.set_fg(mental_color);
        w.write_str(&format!("{:+4}   ", substance.mental_cost));

        w.set_fg(if can_afford { Color::LightGreen } else { Color::DarkGray });
        w.write_str(&format!("{:>6}", format_gold(substance.price as u64)));

        if in_inventory > 0 {
            w.set_fg(Color::Yellow);
            w.write_str(&format!(" [x{}]", in_inventory));
        }

        w.writeln("");
    }

    // Show active effects
    if !state.active_effects.is_empty() {
        w.writeln("");
        w.set_fg(Color::LightMagenta);
        w.writeln("  Active Effects:");
        for effect in &state.active_effects {
            w.set_fg(Color::White);
            w.writeln(&format!("    - {} ({} turns)", effect.substance_key, effect.turns_remaining));
        }
    }

    // Show addictions
    if !state.addictions.is_empty() {
        w.writeln("");
        w.set_fg(Color::LightRed);
        w.writeln("  Addictions:");
        for (key, level) in &state.addictions {
            if *level > 0 {
                w.set_fg(Color::Yellow);
                w.writeln(&format!("    - {} (level {})", key, level));
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render equipment screen
pub fn render_equipment(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);
    render_status_bar(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  EQUIPMENT");
    w.reset_color();

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  {}", msg));
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Currently Equipped:");
    for (slot_name, item_key) in state.equipment.all_equipped() {
        w.set_fg(Color::LightGray);
        w.write_str(&format!("    {:<10}: ", slot_name));
        if let Some(key) = item_key {
            if let Some(item) = super::data::get_equipment(key) {
                w.set_fg(Color::LightCyan);
                w.writeln(item.name);
            } else {
                w.writeln(key);
            }
        } else {
            w.set_fg(Color::DarkGray);
            w.writeln("(empty)");
        }
    }

    // Inventory items
    let equippable: Vec<_> = state.inventory.iter()
        .filter(|(_, count)| **count > 0)
        .filter(|(key, _)| super::data::get_equipment(key).is_some())
        .collect();

    if !equippable.is_empty() {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.writeln("  Inventory (enter number to equip):");
        for (i, (key, count)) in equippable.iter().enumerate() {
            if let Some(item) = super::data::get_equipment(key) {
                w.set_fg(Color::LightCyan);
                w.write_str(&format!("    [{:2}] ", i + 1));
                w.set_fg(Color::White);
                w.write_str(&format!("{} ", item.name));
                w.set_fg(Color::DarkGray);
                w.writeln(&format!("x{}", count));
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render character stats screen
pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    let stats = state.effective_stats();

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln(&format!("  {} - Level {} {}", state.character_name, state.level, state.class.name()));
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  PRIMARY STATS:");
    w.set_fg(Color::LightRed);
    w.writeln(&format!("    Strength:     {:3} ({:+})", state.strength, stats.strength - state.strength as i32));
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("    Agility:      {:3} ({:+})", state.agility, stats.agility - state.agility as i32));
    w.set_fg(Color::LightCyan);
    w.writeln(&format!("    Vitality:     {:3} ({:+})", state.vitality, stats.vitality - state.vitality as i32));
    w.set_fg(Color::LightBlue);
    w.writeln(&format!("    Intelligence: {:3} ({:+})", state.intelligence, stats.intelligence - state.intelligence as i32));
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("    Charisma:     {:3} ({:+})", state.charisma, stats.charisma - state.charisma as i32));

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  DERIVED STATS:");
    w.writeln(&format!("    HP:           {}/{}", state.hp, state.max_hp));
    w.writeln(&format!("    Damage:       {}", stats.damage));
    w.writeln(&format!("    Defense:      {}", stats.defense));
    w.writeln(&format!("    Mental:       {}/{}", state.mental_stability, state.max_mental_stability));

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  PROGRESSION:");
    w.writeln(&format!("    Experience:   {}/{}", state.experience, state.experience_to_next));
    w.writeln(&format!("    Gold:         {}", format_gold(state.gold)));
    w.writeln(&format!("    Bank:         {}", format_gold(state.bank_gold)));

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  ACHIEVEMENTS:");
    w.writeln(&format!("    Monsters Killed: {}", state.monsters_killed));
    w.writeln(&format!("    Deepest Dungeon: Level {}", state.deepest_dungeon));
    w.writeln(&format!("    PvP Kills:       {}", state.pvp_kills));
    w.writeln(&format!("    Deaths:          {}", state.deaths));
    w.writeln(&format!("    Days Played:     {}", state.days_played));

    if state.is_king {
        w.set_fg(Color::Yellow);
        w.writeln("    >> KING OF DURUNGHINS <<");
    }
    if state.godhood_level > 0 {
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("    >> GODHOOD LEVEL {} <<", state.godhood_level));
    }
    if state.supreme_being_defeated {
        w.set_fg(Color::White);
        w.bold();
        w.writeln("    >> SUPREME BEING DEFEATED <<");
    }

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render confirm quit screen
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
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Quit to BBS menu? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render game over screen
pub fn render_game_over(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    if state.supreme_being_defeated {
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln("");
        w.writeln("  ██╗   ██╗██╗ ██████╗████████╗ ██████╗ ██████╗ ██╗   ██╗██╗");
        w.writeln("  ██║   ██║██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝██║");
        w.writeln("  ██║   ██║██║██║        ██║   ██║   ██║██████╔╝ ╚████╔╝ ██║");
        w.writeln("  ╚██╗ ██╔╝██║██║        ██║   ██║   ██║██╔══██╗  ╚██╔╝  ╚═╝");
        w.writeln("   ╚████╔╝ ██║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║   ██╗");
        w.writeln("    ╚═══╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝");
        w.reset_color();
        w.writeln("");
        w.set_fg(Color::LightMagenta);
        w.writeln("  You have defeated THE SUPREME BEING!");
        w.writeln("  The power of a god now flows through you.");
    } else {
        w.set_fg(Color::LightRed);
        w.bold();
        w.writeln("");
        w.writeln("  ███████╗ █████╗ ██╗     ██╗     ███████╗███╗   ██╗");
        w.writeln("  ██╔════╝██╔══██╗██║     ██║     ██╔════╝████╗  ██║");
        w.writeln("  █████╗  ███████║██║     ██║     █████╗  ██╔██╗ ██║");
        w.writeln("  ██╔══╝  ██╔══██║██║     ██║     ██╔══╝  ██║╚██╗██║");
        w.writeln("  ██║     ██║  ██║███████╗███████╗███████╗██║ ╚████║");
        w.writeln("  ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═══╝");
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  Final Level: {}", state.level));
    w.writeln(&format!("  Deepest Dungeon: Level {}", state.deepest_dungeon));
    w.writeln(&format!("  Monsters Slain: {}", state.monsters_killed));
    w.writeln(&format!("  Total Gold Earned: {}", format_gold(state.total_gold_earned)));
    w.writeln(&format!("  Days Played: {}", state.days_played));

    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Main screen render dispatcher
pub fn render_screen(flow: &super::screen::UsurperFlow) -> String {
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(state),
        GameScreen::CharacterCreation { stage } => render_creation(state, stage),
        GameScreen::Town => render_town(state),
        GameScreen::Dungeon { combat_state } => render_dungeon(state, combat_state),
        GameScreen::Shop { shop_type } => render_shop(state, shop_type),
        GameScreen::Healer => render_healer(state),
        GameScreen::Bank => render_bank(state),
        GameScreen::SubstanceDealer => render_substance(state),
        GameScreen::Equipment => render_equipment(state),
        GameScreen::Stats => render_stats(state),
        GameScreen::Romance => render_town(state),  // TODO: dedicated romance screen
        GameScreen::Clan => render_town(state),     // TODO: dedicated clan screen
        GameScreen::Arena { .. } => render_town(state),  // TODO: arena screen
        GameScreen::Throne => render_town(state),   // TODO: throne screen
        GameScreen::Quests => render_town(state),   // TODO: quest screen
        GameScreen::Leaderboard => render_town(state), // TODO: leaderboard screen
        GameScreen::GameOver => render_game_over(state),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
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
    }

    #[test]
    fn test_render_intro() {
        let state = GameState::new("Test".to_string(), super::super::data::CharacterClass::Warrior);
        let output = render_intro(&state);
        assert!(output.contains("Welcome back"));
    }

    #[test]
    fn test_render_town() {
        let state = GameState::new("Test".to_string(), super::super::data::CharacterClass::Warrior);
        let output = render_town(&state);
        assert!(output.contains("TOWN OF DURUNGHINS"));
        assert!(output.contains("Descend"));
    }

    #[test]
    fn test_tier_color() {
        assert_eq!(tier_color(DungeonTier::Surface), Color::LightGreen);
        assert_eq!(tier_color(DungeonTier::Depths), Color::LightRed);
    }
}
