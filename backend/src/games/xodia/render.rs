//! ANSI rendering for Xodia
//!
//! Mystical fantasy theme with deep purples, magentas, and blues.

use crate::terminal::{AnsiWriter, Color};
use super::state::{GameState, CharacterClass};
use super::combat::CombatState;

// ============================================================================
// COLOR PALETTE - Mystical Fantasy Theme
// ============================================================================

/// Primary accent - mystical purple
const ACCENT_PRIMARY: Color = Color::LightMagenta;
/// Secondary accent - deep blue
const ACCENT_SECONDARY: Color = Color::LightBlue;
/// Highlight - gold/amber for important text
const HIGHLIGHT: Color = Color::Yellow;
/// Danger - for combat and warnings
const DANGER: Color = Color::LightRed;
/// Success - for healing and gains
const SUCCESS: Color = Color::LightGreen;
/// Muted text - for descriptions
const MUTED: Color = Color::LightGray;
/// Border color
const BORDER: Color = Color::Magenta;

// ============================================================================
// HEADER AND DECORATIONS
// ============================================================================

/// Render the Xodia title art
fn render_title(w: &mut AnsiWriter) {
    w.set_fg(ACCENT_PRIMARY);
    w.bold();
    w.writeln("");
    w.writeln(r"  ██╗  ██╗ ██████╗ ██████╗ ██╗ █████╗ ");
    w.writeln(r"  ╚██╗██╔╝██╔═══██╗██╔══██╗██║██╔══██╗");
    w.writeln(r"   ╚███╔╝ ██║   ██║██║  ██║██║███████║");
    w.writeln(r"   ██╔██╗ ██║   ██║██║  ██║██║██╔══██║");
    w.writeln(r"  ██╔╝ ██╗╚██████╔╝██████╔╝██║██║  ██║");
    w.writeln(r"  ╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝╚═╝  ╚═╝");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("                    XODIA");
    w.writeln("           T H E   L I V I N G   M U D");
    w.reset_color();
}

/// Render a mystical border line
fn render_border(w: &mut AnsiWriter) {
    w.set_fg(BORDER);
    w.writeln(&"═".repeat(80));
    w.reset_color();
}

/// Render a thin separator
fn render_separator(w: &mut AnsiWriter) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"─".repeat(80));
    w.reset_color();
}

// ============================================================================
// STATUS BAR
// ============================================================================

/// Render the character status bar
pub fn render_status_bar(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    render_separator(&mut w);

    // First line: Name, Class, Level
    w.set_fg(HIGHLIGHT);
    w.write_str(&format!(" {} ", state.character_name));
    w.set_fg(MUTED);
    w.write_str(&format!("the {} ", state.class));
    w.set_fg(ACCENT_SECONDARY);
    w.write_str(&format!("(Lv.{})", state.level));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // HP with color coding
    let hp_color = if state.health > state.max_health * 2 / 3 {
        SUCCESS
    } else if state.health > state.max_health / 3 {
        HIGHLIGHT
    } else {
        DANGER
    };
    w.set_fg(hp_color);
    w.write_str(&format!("HP: {}/{}", state.health, state.max_health));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // MP
    w.set_fg(ACCENT_SECONDARY);
    w.write_str(&format!("MP: {}/{}", state.mana, state.max_mana));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Gold
    w.set_fg(HIGHLIGHT);
    w.write_str(&format!("Gold: {}", state.gold));

    w.writeln("");

    // Second line: Location
    w.set_fg(ACCENT_PRIMARY);
    w.write_str(&format!(" {} ", state.current_region));
    w.set_fg(Color::DarkGray);
    w.write_str("- ");
    w.set_fg(Color::White);
    w.write_str(&get_room_name(&state.current_room_id));

    // Combat indicator
    if state.in_combat {
        w.set_fg(Color::DarkGray);
        w.write_str(" | ");
        w.set_fg(DANGER);
        w.bold();
        w.write_str("IN COMBAT!");
    }

    w.writeln("");
    w.reset_color();

    render_separator(&mut w);

    w.flush()
}

/// Get room name from ID (fallback if world not loaded)
fn get_room_name(room_id: &str) -> String {
    super::data::get_room_template(room_id)
        .map(|r| r.name.to_string())
        .unwrap_or_else(|| room_id.replace('_', " "))
}

// ============================================================================
// SCREENS
// ============================================================================

/// Render the intro screen
pub fn render_intro() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    w.writeln("");
    render_border(&mut w);

    w.set_fg(MUTED);
    w.writeln("");
    w.writeln("  Long ago, the world was whole, united under the Light of the First Flame.");
    w.writeln("  Then came the Sundering - the Flame was shattered into seven shards,");
    w.writeln("  each falling to a corner of the world.");
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  You are a Seeker, called by dreams to find the Spire of Eternity.");
    w.writeln("  Your journey begins in Misthollow Village...");
    w.writeln("");
    w.reset_color();

    render_border(&mut w);

    w.set_fg(HIGHLIGHT);
    w.writeln("");
    w.writeln("  Press any key to begin your journey...");
    w.reset_color();

    w.flush()
}

/// Render character creation screen
pub fn render_character_creation(step: u32, name: Option<&str>) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.set_fg(HIGHLIGHT);
    w.bold();
    w.writeln("");
    w.writeln("  CREATE YOUR CHARACTER");
    w.reset_color();
    w.writeln("");

    match step {
        0 => {
            // Enter name
            w.set_fg(ACCENT_SECONDARY);
            w.writeln("  What is your name, Seeker?");
            w.writeln("");
            w.set_fg(MUTED);
            w.writeln("  (Enter a name for your character, 3-20 characters)");
            w.writeln("");
            w.set_fg(Color::White);
            w.write_str("  > ");
        }
        1 => {
            // Select class
            if let Some(n) = name {
                w.set_fg(ACCENT_SECONDARY);
                w.writeln(&format!("  Welcome, {}.", n));
                w.writeln("");
            }
            w.set_fg(HIGHLIGHT);
            w.writeln("  Choose your path:");
            w.writeln("");

            // Warrior
            w.set_fg(DANGER);
            w.write_str("  [1] WARRIOR");
            w.set_fg(MUTED);
            w.writeln(" - Master of martial combat. High strength and constitution.");
            w.writeln("              Best for: Direct combat, tanking damage");
            w.writeln("");

            // Mage
            w.set_fg(ACCENT_SECONDARY);
            w.write_str("  [2] MAGE");
            w.set_fg(MUTED);
            w.writeln("    - Wielder of arcane power. High intelligence and mana.");
            w.writeln("              Best for: Devastating spells, magical puzzles");
            w.writeln("");

            // Rogue
            w.set_fg(HIGHLIGHT);
            w.write_str("  [3] ROGUE");
            w.set_fg(MUTED);
            w.writeln("   - A shadow in the night. High dexterity and cunning.");
            w.writeln("              Best for: Stealth, critical hits, locks");
            w.writeln("");

            // Cleric
            w.set_fg(SUCCESS);
            w.write_str("  [4] CLERIC");
            w.set_fg(MUTED);
            w.writeln("  - Divine servant. Balanced stats with healing magic.");
            w.writeln("              Best for: Survival, support, undead");
            w.writeln("");
        }
        _ => {}
    }

    w.reset_color();
    w.flush()
}

/// Render main game view
pub fn render_main_view(state: &GameState, room_description: &str) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    w.write_str(&render_status_bar(state));

    // Room description
    w.writeln("");
    w.set_fg(Color::White);

    // Word wrap the description
    for line in word_wrap(room_description, 78) {
        w.writeln(&format!("  {}", line));
    }

    // Show last message if any
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(HIGHLIGHT);
        w.writeln(&format!("  >> {} <<", msg));
    }

    // Show last LLM response if any
    if let Some(ref response) = state.last_llm_response {
        w.writeln("");
        w.set_fg(ACCENT_PRIMARY);
        for line in word_wrap(response, 76) {
            w.writeln(&format!("  {}", line));
        }
    }

    // Prompt
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.write_str("  > ");
    w.reset_color();

    w.flush()
}

/// Render combat screen
pub fn render_combat(state: &GameState, combat: &CombatState, last_action: &str) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();

    // Combat header
    w.set_fg(DANGER);
    w.bold();
    w.writeln("");
    w.writeln("  ════════════════════════════════════════════════════════════════════════════");
    w.writeln("                              ⚔  COMBAT  ⚔");
    w.writeln("  ════════════════════════════════════════════════════════════════════════════");
    w.reset_color();

    // Enemy info
    w.writeln("");
    w.set_fg(DANGER);
    w.write_str(&format!("  {} ", combat.enemy_name));
    w.set_fg(MUTED);
    w.writeln(&format!("(Round {})", combat.round));

    // Enemy health bar
    let enemy_hp_pct = (combat.enemy_health as f32 / combat.enemy_max_health as f32 * 20.0) as usize;
    w.set_fg(Color::DarkGray);
    w.write_str("  HP: [");
    w.set_fg(DANGER);
    w.write_str(&"█".repeat(enemy_hp_pct));
    w.set_fg(Color::DarkGray);
    w.write_str(&"░".repeat(20 - enemy_hp_pct));
    w.write_str("] ");
    w.set_fg(DANGER);
    w.writeln(&format!("{}/{}", combat.enemy_health, combat.enemy_max_health));

    // Separator
    w.writeln("");
    render_separator(&mut w);

    // Player info
    w.writeln("");
    w.set_fg(SUCCESS);
    w.writeln(&format!("  {} the {}", state.character_name, state.class));

    // Player health bar
    let player_hp_pct = (state.health as f32 / state.max_health as f32 * 20.0) as usize;
    w.set_fg(Color::DarkGray);
    w.write_str("  HP: [");
    w.set_fg(SUCCESS);
    w.write_str(&"█".repeat(player_hp_pct));
    w.set_fg(Color::DarkGray);
    w.write_str(&"░".repeat(20 - player_hp_pct));
    w.write_str("] ");
    w.set_fg(SUCCESS);
    w.writeln(&format!("{}/{}", state.health, state.max_health));

    // Mana bar
    let player_mp_pct = (state.mana as f32 / state.max_mana as f32 * 20.0) as usize;
    w.set_fg(Color::DarkGray);
    w.write_str("  MP: [");
    w.set_fg(ACCENT_SECONDARY);
    w.write_str(&"█".repeat(player_mp_pct));
    w.set_fg(Color::DarkGray);
    w.write_str(&"░".repeat(20 - player_mp_pct));
    w.write_str("] ");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln(&format!("{}/{}", state.mana, state.max_mana));

    // Last action narrative
    if !last_action.is_empty() {
        w.writeln("");
        render_separator(&mut w);
        w.writeln("");
        w.set_fg(Color::White);
        for line in word_wrap(last_action, 76) {
            w.writeln(&format!("  {}", line));
        }
    }

    // Combat options
    w.writeln("");
    render_separator(&mut w);
    w.writeln("");
    w.set_fg(HIGHLIGHT);
    w.writeln("  ACTIONS:");
    w.writeln("");

    w.set_fg(DANGER);
    w.write_str("  [A]");
    w.set_fg(MUTED);
    w.write_str("ttack   ");

    w.set_fg(ACCENT_SECONDARY);
    w.write_str("[D]");
    w.set_fg(MUTED);
    w.write_str("efend   ");

    w.set_fg(HIGHLIGHT);
    w.write_str("[F]");
    w.set_fg(MUTED);
    w.write_str("lee   ");

    w.set_fg(SUCCESS);
    w.write_str("[U]");
    w.set_fg(MUTED);
    w.writeln("se Item");

    if state.mana > 0 && matches!(state.class, CharacterClass::Mage | CharacterClass::Cleric) {
        w.writeln("");
        w.set_fg(ACCENT_PRIMARY);
        w.write_str("  [C]");
        w.set_fg(MUTED);
        w.writeln("ast Spell");
    }

    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.write_str("  > ");
    w.reset_color();

    w.flush()
}

/// Render inventory screen
pub fn render_inventory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.set_fg(HIGHLIGHT);
    w.bold();
    w.writeln("");
    w.writeln("  INVENTORY");
    w.reset_color();
    w.writeln("");

    // Equipped items
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Equipped:");
    w.set_fg(MUTED);

    if let Some(ref weapon) = state.equipment.weapon {
        w.writeln(&format!("    Weapon: {}", format_item_name(weapon)));
    } else {
        w.writeln("    Weapon: (none)");
    }

    if let Some(ref armor) = state.equipment.armor {
        w.writeln(&format!("    Armor:  {}", format_item_name(armor)));
    } else {
        w.writeln("    Armor:  (none)");
    }

    w.writeln("");

    // Inventory items
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Backpack:");

    if state.inventory.is_empty() {
        w.set_fg(MUTED);
        w.writeln("    (empty)");
    } else {
        for (i, item) in state.inventory.iter().enumerate() {
            let equipped_marker = if item.equipped { " [E]" } else { "" };
            w.set_fg(Color::White);
            w.write_str(&format!("    {}. ", i + 1));
            w.set_fg(if item.equipped { SUCCESS } else { MUTED });
            w.writeln(&format!(
                "{}{} x{}",
                item.name, equipped_marker, item.quantity
            ));
        }
    }

    // Weight
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!(
        "  Weight: {:.1}/{:.1} lbs",
        state.carry_weight, state.max_carry_weight
    ));

    w.writeln("");
    render_border(&mut w);
    w.set_fg(MUTED);
    w.writeln("  Commands: [E]quip, [U]nequip, [D]rop, [Q]uit inventory");
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.write_str("  > ");
    w.reset_color();

    w.flush()
}

/// Render character stats screen
pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.set_fg(HIGHLIGHT);
    w.bold();
    w.writeln("");
    w.writeln(&format!("  {} - {}", state.character_name, state.class));
    w.reset_color();

    // Level and XP
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.write_str(&format!("  Level: {} ", state.level));
    w.set_fg(MUTED);
    let next_xp = super::data::LEVEL_XP_REQUIREMENTS
        .get(state.level as usize)
        .copied()
        .unwrap_or(999999);
    w.writeln(&format!("(XP: {}/{})", state.experience, next_xp));

    // Stats
    w.writeln("");
    w.set_fg(ACCENT_PRIMARY);
    w.writeln("  ATTRIBUTES:");
    w.set_fg(Color::White);

    let stats = &state.stats;
    w.writeln(&format!("    STR: {:2}  ({:+})", stats.strength, stats.modifier("str")));
    w.writeln(&format!("    DEX: {:2}  ({:+})", stats.dexterity, stats.modifier("dex")));
    w.writeln(&format!("    CON: {:2}  ({:+})", stats.constitution, stats.modifier("con")));
    w.writeln(&format!("    INT: {:2}  ({:+})", stats.intelligence, stats.modifier("int")));
    w.writeln(&format!("    WIS: {:2}  ({:+})", stats.wisdom, stats.modifier("wis")));
    w.writeln(&format!("    CHA: {:2}  ({:+})", stats.charisma, stats.modifier("cha")));

    // Derived stats
    w.writeln("");
    w.set_fg(ACCENT_PRIMARY);
    w.writeln("  COMBAT:");
    w.set_fg(Color::White);
    w.writeln(&format!("    Attack Power: {}", state.attack_power()));
    w.writeln(&format!("    Defense:      {}", state.defense()));

    // Resources
    w.writeln("");
    w.set_fg(ACCENT_PRIMARY);
    w.writeln("  RESOURCES:");
    w.set_fg(SUCCESS);
    w.writeln(&format!("    Health: {}/{}", state.health, state.max_health));
    w.set_fg(ACCENT_SECONDARY);
    w.writeln(&format!("    Mana:   {}/{}", state.mana, state.max_mana));
    w.set_fg(HIGHLIGHT);
    w.writeln(&format!("    Gold:   {}", state.gold));

    w.writeln("");
    render_border(&mut w);
    w.set_fg(MUTED);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render help screen
pub fn render_help() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.set_fg(HIGHLIGHT);
    w.bold();
    w.writeln("");
    w.writeln("  COMMANDS");
    w.reset_color();

    // Movement
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Movement:");
    w.set_fg(MUTED);
    w.writeln("    NORTH, SOUTH, EAST, WEST (or N, S, E, W)");
    w.writeln("    UP, DOWN, IN, OUT");

    // Actions
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Actions:");
    w.set_fg(MUTED);
    w.writeln("    LOOK [target]    - Examine your surroundings or something");
    w.writeln("    TAKE [item]      - Pick up an item");
    w.writeln("    DROP [item]      - Drop an item");
    w.writeln("    USE [item]       - Use an item");
    w.writeln("    TALK [npc]       - Speak with someone");

    // Combat
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Combat:");
    w.set_fg(MUTED);
    w.writeln("    ATTACK [target]  - Attack an enemy");
    w.writeln("    FLEE             - Try to escape combat");
    w.writeln("    CAST [spell]     - Cast a magic spell");

    // Character
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Character:");
    w.set_fg(MUTED);
    w.writeln("    INVENTORY (or I) - View your inventory");
    w.writeln("    STATS            - View your character stats");
    w.writeln("    EQUIP [item]     - Equip a weapon or armor");

    // Meta
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Other:");
    w.set_fg(MUTED);
    w.writeln("    HELP (or ?)      - Show this help");
    w.writeln("    SAVE             - Save your game");
    w.writeln("    QUIT             - Exit the game");

    w.writeln("");
    render_border(&mut w);
    w.set_fg(MUTED);
    w.writeln("  Tip: You can also type natural language - try 'look around for secrets'");
    w.writeln("");
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render quit confirmation
pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.writeln("");
    w.set_fg(HIGHLIGHT);
    w.writeln("  Are you sure you want to leave Xodia?");
    w.writeln("");
    w.set_fg(MUTED);
    w.writeln("  Your progress will be saved automatically.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  [Y] Yes, exit to BBS");
    w.writeln("  [N] No, return to game");
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.write_str("  > ");
    w.reset_color();

    w.flush()
}

/// Render offline mode message
pub fn render_offline_mode() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.writeln("");
    w.set_fg(DANGER);
    w.bold();
    w.writeln("  XODIA IS CURRENTLY OFFLINE");
    w.reset_color();
    w.writeln("");
    w.set_fg(MUTED);
    w.writeln("  The Dungeon Master (LLM) is not available.");
    w.writeln("  Xodia requires an AI connection to function.");
    w.writeln("");
    w.writeln("  This may be due to:");
    w.writeln("    - Ollama not running locally");
    w.writeln("    - Cloud API unavailable");
    w.writeln("    - System maintenance");
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Please try again later or contact the Sysop.");
    w.writeln("");
    render_border(&mut w);
    w.set_fg(MUTED);
    w.writeln("  Press any key to return to the BBS menu...");
    w.reset_color();

    w.flush()
}

/// Render maintenance mode message
pub fn render_maintenance_mode() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    render_title(&mut w);
    render_border(&mut w);

    w.writeln("");
    w.set_fg(HIGHLIGHT);
    w.bold();
    w.writeln("  XODIA IS UNDER MAINTENANCE");
    w.reset_color();
    w.writeln("");
    w.set_fg(MUTED);
    w.writeln("  The Sysop has temporarily disabled this game");
    w.writeln("  for maintenance or updates.");
    w.writeln("");
    w.set_fg(ACCENT_SECONDARY);
    w.writeln("  Your saved game is safe and will be available");
    w.writeln("  when maintenance is complete.");
    w.writeln("");
    render_border(&mut w);
    w.set_fg(MUTED);
    w.writeln("  Press any key to return to the BBS menu...");
    w.reset_color();

    w.flush()
}

// ============================================================================
// HELPERS
// ============================================================================

/// Format item name from key
fn format_item_name(key: &str) -> String {
    super::data::get_item_template(key)
        .map(|t| t.name.to_string())
        .unwrap_or_else(|| key.replace('_', " "))
}

/// Word wrap text to specified width
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_wrap() {
        let text = "This is a long sentence that needs to be wrapped";
        let wrapped = word_wrap(text, 20);
        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_word_wrap_empty() {
        let wrapped = word_wrap("", 80);
        assert_eq!(wrapped.len(), 1);
        assert!(wrapped[0].is_empty());
    }

    #[test]
    fn test_render_intro() {
        let output = render_intro();
        assert!(output.contains("XODIA"));
        assert!(output.contains("Seeker"));
    }

    #[test]
    fn test_render_character_creation() {
        let output = render_character_creation(0, None);
        assert!(output.contains("name"));

        let output = render_character_creation(1, Some("TestHero"));
        assert!(output.contains("TestHero"));
        assert!(output.contains("WARRIOR"));
        assert!(output.contains("MAGE"));
    }

    #[test]
    fn test_render_help() {
        let output = render_help();
        assert!(output.contains("COMMANDS"));
        assert!(output.contains("LOOK"));
        assert!(output.contains("ATTACK"));
    }

    #[test]
    fn test_render_offline_mode() {
        let output = render_offline_mode();
        assert!(output.contains("OFFLINE"));
        assert!(output.contains("LLM"));
    }

    #[test]
    fn test_render_maintenance_mode() {
        let output = render_maintenance_mode();
        assert!(output.contains("MAINTENANCE"));
        assert!(output.contains("Sysop"));
    }

    #[test]
    fn test_render_status_bar() {
        let state = GameState::new("TestHero", CharacterClass::Warrior);
        let output = render_status_bar(&state);
        assert!(output.contains("TestHero"));
        assert!(output.contains("HP:"));
        assert!(output.contains("MP:"));
    }

    #[test]
    fn test_render_inventory() {
        let state = GameState::new("TestHero", CharacterClass::Warrior);
        let output = render_inventory(&state);
        assert!(output.contains("INVENTORY"));
        assert!(output.contains("Equipped"));
        assert!(output.contains("Backpack"));
    }

    #[test]
    fn test_render_stats() {
        let state = GameState::new("TestHero", CharacterClass::Warrior);
        let output = render_stats(&state);
        assert!(output.contains("TestHero"));
        assert!(output.contains("STR"));
        assert!(output.contains("DEX"));
    }

    #[test]
    fn test_render_combat() {
        let state = GameState::new("TestHero", CharacterClass::Warrior);
        let combat = CombatState {
            enemy_id: "test".to_string(),
            enemy_name: "Goblin".to_string(),
            enemy_health: 10,
            enemy_max_health: 15,
            enemy_damage: 3,
            round: 1,
            player_is_defending: false,
        };
        let output = render_combat(&state, &combat, "The battle begins!");
        assert!(output.contains("COMBAT"));
        assert!(output.contains("Goblin"));
        assert!(output.contains("[A]"));
    }
}
