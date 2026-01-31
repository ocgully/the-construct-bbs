//! ANSI rendering for Morningmist
//! Fantasy forest visual identity with purples, greens, and golds

#![allow(dead_code)]

use crate::terminal::{AnsiWriter, Color};
use super::state::GameState;
use super::screen::{GameScreen, TextInputPurpose, KyrandiaFlow};
use super::data::{get_item, get_npc, MageRank, Region, RoomSpecial, NpcType};
use super::world::get_current_room_details;
use super::magic::format_spellbook;

// ============================================================================
// HEADER AND STATUS
// ============================================================================

/// Render the game title banner
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("");
    w.writeln("   ███╗   ███╗ ██████╗ ██████╗ ███╗   ██╗██╗███╗   ██╗ ██████╗ ███╗   ███╗██╗███████╗████████╗");
    w.writeln("   ████╗ ████║██╔═══██╗██╔══██╗████╗  ██║██║████╗  ██║██╔════╝ ████╗ ████║██║██╔════╝╚══██╔══╝");
    w.writeln("   ██╔████╔██║██║   ██║██████╔╝██╔██╗ ██║██║██╔██╗ ██║██║  ███╗██╔████╔██║██║███████╗   ██║   ");
    w.writeln("   ██║╚██╔╝██║██║   ██║██╔══██╗██║╚██╗██║██║██║╚██╗██║██║   ██║██║╚██╔╝██║██║╚════██║   ██║   ");
    w.writeln("   ██║ ╚═╝ ██║╚██████╔╝██║  ██║██║ ╚████║██║██║ ╚████║╚██████╔╝██║ ╚═╝ ██║██║███████║   ██║   ");
    w.writeln("   ╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝╚═╝╚══════╝   ╚═╝   ");
    w.writeln("                                  MORNINGMIST");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.writeln("            ~ Realm of the Lady of Legends ~");
    w.reset_color();
}

/// Render compact header for exploration
fn render_compact_header(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.write_str("  MORNINGMIST");
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Region
    let region = state.current_region();
    let region_color = match region {
        Region::Village => Color::LightGreen,
        Region::DarkForest => Color::Green,
        Region::GoldenForest => Color::Yellow,
        Region::DragonCastle => Color::LightRed,
    };
    w.set_fg(region_color);
    w.write_str(region.name());

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Rank and level
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("Lv{} {}", state.level, state.rank().name()));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Turns
    w.set_fg(Color::White);
    w.write_str(&format!("Turns: {}", state.turns_remaining));

    w.writeln("");
}

/// Render status bar
fn render_status_bar(w: &mut AnsiWriter, state: &GameState) {
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(78));

    // HP bar
    w.set_fg(Color::LightRed);
    w.write_str(" HP: ");
    let hp_pct = state.health as f32 / state.max_health as f32;
    let hp_color = if hp_pct > 0.6 {
        Color::LightGreen
    } else if hp_pct > 0.3 {
        Color::Yellow
    } else {
        Color::LightRed
    };
    w.set_fg(hp_color);
    w.write_str(&format!("{}/{}", state.health, state.max_health));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Mana bar
    w.set_fg(Color::LightBlue);
    w.write_str("Mana: ");
    w.write_str(&format!("{}/{}", state.mana, state.max_mana));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // Gold
    w.set_fg(Color::Yellow);
    w.write_str(&format!("Gold: {}", state.gold));

    w.set_fg(Color::DarkGray);
    w.write_str(" | ");

    // XP to next level
    w.set_fg(Color::LightMagenta);
    let xp_needed = state.xp_to_next_level();
    if xp_needed > 0 {
        w.write_str(&format!("XP: {}/{}", state.xp, MageRank::from_level(state.level + 1).xp_required()));
    } else {
        w.write_str("MAX LEVEL");
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(78));
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
    w.set_fg(Color::LightGray);
    w.writeln("  In the mystical realm of Morningmist, magic flows like water through ancient");
    w.writeln("  forests. The Lady of Legends, Tashanna, watches over all who seek wisdom.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  You are a young apprentice from a humble village, drawn to the path of");
    w.writeln("  magic. Your destiny: to become the Arch-Mage of Legends, the mightiest");
    w.writeln("  wizard in all the realm.");
    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.writeln("  But the path is perilous. The Dark Forest holds monsters and rival mages.");
    w.writeln("  The Golden Forest guards ancient secrets. And in Dragon Castle, a fierce");
    w.writeln("  guardian awaits any who would claim the throne.");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Welcome, {}. Your journey begins.", state.name));
    w.writeln("");
    w.reset_color();
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to begin your adventure...");
    w.reset_color();

    w.flush()
}

/// Render exploration screen
pub fn render_exploration(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);
    render_status_bar(&mut w, state);

    // Get room details
    if let Some(details) = get_current_room_details(state) {
        // Room name
        w.set_fg(Color::LightCyan);
        w.bold();
        w.writeln(&format!("  {}", details.name));
        w.reset_color();

        // Room description
        w.writeln("");
        w.set_fg(Color::LightGray);
        // Word wrap description
        let desc = &details.description;
        for line in wrap_text(desc, 74) {
            w.writeln(&format!("  {}", line));
        }

        // Items in room
        if !details.items.is_empty() {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.write_str("  You see: ");
            w.set_fg(Color::White);
            let item_names: Vec<String> = details.items.iter().map(|i| i.name.clone()).collect();
            w.writeln(&item_names.join(", "));
        }

        // NPCs in room
        if !details.npcs.is_empty() {
            w.writeln("");
            for npc in &details.npcs {
                let npc_color = match npc.npc_type {
                    NpcType::Friendly => Color::LightGreen,
                    NpcType::Merchant => Color::Yellow,
                    NpcType::Trainer => Color::LightCyan,
                    NpcType::Quest => Color::LightMagenta,
                    NpcType::Enemy | NpcType::Boss => Color::LightRed,
                };
                w.set_fg(npc_color);
                w.writeln(&format!("  {} is here.", npc.name));
            }
        }

        // Room specials
        if let Some(special) = details.special {
            w.writeln("");
            w.set_fg(Color::LightMagenta);
            match special {
                RoomSpecial::Inn => w.writeln("  [Rest here to restore HP and Mana]"),
                RoomSpecial::Shop => w.writeln("  [TALK to the merchant to buy/sell]"),
                RoomSpecial::Library => w.writeln("  [Ancient knowledge awaits]"),
                RoomSpecial::Fountain => w.writeln("  [Throw 3 pine cones to receive a spell scroll]"),
                RoomSpecial::Training => w.writeln("  [Train to improve your skills]"),
                RoomSpecial::Altar => w.writeln("  [A holy place. Speak the sacred words...]"),
                RoomSpecial::ThroneRoom => w.writeln("  [The seat of the Arch-Mage awaits a worthy claimant]"),
                RoomSpecial::DragonLair => {
                    w.set_fg(Color::LightRed);
                    w.writeln("  [THE DRAGON SLEEPS HERE. APPROACH WITH CAUTION!]");
                }
            }
        }

        // Exits
        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.write_str("  Exits: ");
        w.set_fg(Color::White);
        w.writeln(&details.exits.join(", "));
    }

    // Last message
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        for line in wrap_text(msg, 74) {
            w.writeln(&format!("  >> {} <<", line));
        }
        w.reset_color();
    }

    // Quick menu
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  [I]nventory  [M]agic  [S]tats  [H]elp  [Q]uit");
    w.reset_color();

    // Input prompt
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  What do you do? > ");
    w.reset_color();

    w.flush()
}

/// Render combat screen
pub fn render_combat(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);

    if let Some(ref combat) = state.combat {
        if let Some(monster) = super::data::get_monster(&combat.monster_key) {
            w.writeln("");
            w.set_fg(Color::LightRed);
            w.bold();
            w.writeln(&format!("  === COMBAT: {} ===", monster.name.to_uppercase()));
            w.reset_color();

            w.writeln("");
            w.set_fg(Color::LightGray);
            w.writeln(&format!("  {}", monster.description));

            // Monster HP bar
            w.writeln("");
            w.set_fg(Color::Red);
            let hp_pct = combat.monster_hp as f32 / combat.monster_max_hp as f32;
            let bar_filled = (hp_pct * 20.0) as usize;
            let bar_empty = 20 - bar_filled;
            w.write_str(&format!("  {} HP: [", monster.name));
            w.set_fg(Color::LightRed);
            w.write_str(&"\u{2588}".repeat(bar_filled));
            w.set_fg(Color::DarkGray);
            w.write_str(&"\u{2591}".repeat(bar_empty));
            w.set_fg(Color::Red);
            w.writeln(&format!("] {}/{}", combat.monster_hp, combat.monster_max_hp));

            // Your HP bar
            w.writeln("");
            w.set_fg(Color::Green);
            let your_hp_pct = state.health as f32 / state.max_health as f32;
            let your_bar_filled = (your_hp_pct * 20.0) as usize;
            let your_bar_empty = 20 - your_bar_filled;
            w.write_str("  Your HP: [");
            w.set_fg(Color::LightGreen);
            w.write_str(&"\u{2588}".repeat(your_bar_filled));
            w.set_fg(Color::DarkGray);
            w.write_str(&"\u{2591}".repeat(your_bar_empty));
            w.set_fg(Color::Green);
            w.writeln(&format!("] {}/{}", state.health, state.max_health));

            // Mana
            w.set_fg(Color::Blue);
            w.writeln(&format!("  Mana: {}/{}", state.mana, state.max_mana));
        }
    }

    // Last message (combat results)
    if let Some(ref msg) = state.last_message {
        w.writeln("");
        w.set_fg(Color::Yellow);
        for line in wrap_text(msg, 74) {
            w.writeln(&format!("  {}", line));
        }
    }

    // Combat options
    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&"\u{2500}".repeat(78));

    w.set_fg(Color::LightCyan);
    w.write_str("    [A] ");
    w.set_fg(Color::White);
    w.write_str("Attack");

    w.set_fg(Color::DarkGray);
    w.write_str("    ");

    w.set_fg(Color::LightCyan);
    w.write_str("[C] ");
    w.set_fg(Color::White);
    w.write_str("Cast Spell");

    w.set_fg(Color::DarkGray);
    w.write_str("    ");

    w.set_fg(Color::LightCyan);
    w.write_str("[F] ");
    w.set_fg(Color::White);
    w.writeln("Flee");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render inventory screen
pub fn render_inventory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  === INVENTORY ===");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln(&format!(
        "  Capacity: {}/{}",
        state.inventory_count(),
        state.inventory_capacity
    ));

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Item                    Qty    Description");
    w.writeln(&format!("  {}", "\u{2500}".repeat(60)));
    w.reset_color();

    if state.inventory.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  Your inventory is empty.");
    } else {
        for (key, qty) in &state.inventory {
            if let Some(item) = get_item(key) {
                w.set_fg(Color::White);
                w.write_str(&format!("  {:<24}", item.name));
                w.set_fg(Color::LightGray);
                w.write_str(&format!("{:>3}    ", qty));
                w.set_fg(Color::DarkGray);
                // Truncate description
                let desc = if item.description.len() > 30 {
                    format!("{}...", &item.description[..27])
                } else {
                    item.description.to_string()
                };
                w.writeln(&desc);
            }
        }
    }

    // Equipment
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("  EQUIPPED:");
    w.reset_color();

    w.set_fg(Color::White);
    w.write_str("  Weapon: ");
    if let Some(ref weapon_key) = state.equipped_weapon {
        if let Some(item) = get_item(weapon_key) {
            w.set_fg(Color::LightGreen);
            w.writeln(item.name);
        }
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("None");
    }

    w.set_fg(Color::White);
    w.write_str("  Armor: ");
    if let Some(ref armor_key) = state.equipped_armor {
        if let Some(item) = get_item(armor_key) {
            w.set_fg(Color::LightGreen);
            w.writeln(item.name);
        }
    } else {
        w.set_fg(Color::DarkGray);
        w.writeln("None");
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render spellbook screen
pub fn render_spellbook(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.bold();
    w.writeln("  === SPELLBOOK ===");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Spell              Mana   Incantation              Effect");
    w.writeln(&format!("  {}", "\u{2500}".repeat(70)));
    w.reset_color();

    let spells = format_spellbook(state);
    if spells.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  You know no spells yet. Find scrolls to learn magic!");
    } else {
        for (name, incantation, mana, desc) in &spells {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("  {:<18}", name));
            w.set_fg(Color::LightBlue);
            w.write_str(&format!("{:>4}   ", mana));
            w.set_fg(Color::LightMagenta);
            w.write_str(&format!("{:<24}", incantation));
            w.set_fg(Color::LightGray);
            // Truncate description
            let short_desc = if desc.len() > 20 {
                format!("{}...", &desc[..17])
            } else {
                desc.clone()
            };
            w.writeln(&short_desc);
        }
    }

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  To cast a spell, type its incantation!");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render stats screen
pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln(&format!("  === {} ===", state.name.to_uppercase()));
    w.reset_color();

    w.writeln("");

    // Rank and level
    w.set_fg(Color::White);
    w.write_str("  Rank: ");
    w.set_fg(Color::LightMagenta);
    w.writeln(&format!("{} (Level {})", state.rank().name(), state.level));

    // XP
    w.set_fg(Color::White);
    w.write_str("  Experience: ");
    w.set_fg(Color::Yellow);
    let xp_needed = state.xp_to_next_level();
    if xp_needed > 0 {
        w.writeln(&format!(
            "{} / {} to next level",
            state.xp,
            MageRank::from_level(state.level + 1).xp_required()
        ));
    } else {
        w.writeln(&format!("{} (MAX)", state.xp));
    }

    w.writeln("");

    // Combat stats
    w.set_fg(Color::LightRed);
    w.writeln(&format!("  Health: {}/{}", state.health, state.max_health));
    w.set_fg(Color::LightBlue);
    w.writeln(&format!("  Mana: {}/{}", state.mana, state.max_mana));
    w.set_fg(Color::LightGreen);
    w.writeln(&format!("  Attack: {}", state.attack_power()));
    w.writeln(&format!("  Defense: {}", state.defense()));

    w.writeln("");

    // Resources
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Gold: {}", state.gold));
    w.writeln(&format!("  Total Gold Earned: {}", state.total_gold_earned));

    w.writeln("");

    // Statistics
    w.set_fg(Color::White);
    w.writeln("  Statistics:");
    w.set_fg(Color::LightGray);
    w.writeln(&format!("    Monsters Defeated: {}", state.monsters_killed));
    w.writeln(&format!("    Deaths: {}", state.deaths));
    w.writeln(&format!("    PvP Kills: {}", state.pvp_kills));
    w.writeln(&format!("    PvP Deaths: {}", state.pvp_deaths));
    w.writeln(&format!("    Spells Known: {}", state.known_spells.len()));
    w.writeln(&format!("    Puzzles Solved: {}", state.puzzles_solved.len()));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Render NPC dialogue
pub fn render_dialogue(state: &GameState, npc_key: &str, index: usize) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);

    if let Some(npc) = get_npc(npc_key) {
        w.writeln("");
        w.set_fg(Color::LightGreen);
        w.bold();
        w.writeln(&format!("  {} says:", npc.name));
        w.reset_color();

        w.writeln("");
        if index < npc.dialogue.len() {
            w.set_fg(Color::White);
            for line in wrap_text(npc.dialogue[index], 70) {
                w.writeln(&format!("    \"{}\"", line));
            }
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        if index + 1 < npc.dialogue.len() {
            w.writeln("  [Press any key to continue...]");
        } else {
            w.writeln("  [Press any key to end conversation...]");
        }
    }

    w.reset_color();
    w.flush()
}

/// Render fountain screen
pub fn render_fountain(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_compact_header(&mut w, state);

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.bold();
    w.writeln("  === THE FOUNTAIN OF SCROLLS ===");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  The crystal fountain shimmers with magical light. Starlight");
    w.writeln("  dances on the water's surface, promising arcane knowledge.");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  The Fountain accepts 3 pine cones to create a spell scroll.");

    let pine_cones = state.item_count("pine_cone");
    w.writeln("");
    if pine_cones >= 3 {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  You have {} pine cones.", pine_cones));
        w.writeln("");
        w.set_fg(Color::LightCyan);
        w.write_str("    [Y] ");
        w.set_fg(Color::White);
        w.writeln("Throw 3 pine cones into the fountain");
    } else {
        w.set_fg(Color::LightRed);
        w.writeln(&format!("  You only have {} pine cones. You need 3.", pine_cones));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("    [N] ");
    w.set_fg(Color::White);
    w.writeln("Leave the fountain");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render text input screen
pub fn render_text_input(prompt: &str, purpose: &TextInputPurpose) -> String {
    let mut w = AnsiWriter::new();

    w.writeln("");
    w.set_fg(Color::Yellow);

    match purpose {
        TextInputPurpose::SpellCast => {
            w.writeln("  Cast a spell by typing its incantation:");
        }
        TextInputPurpose::PuzzleSolution => {
            w.writeln("  Enter your answer:");
        }
        TextInputPurpose::Say => {
            w.writeln("  What do you say?");
        }
        TextInputPurpose::Whisper { target } => {
            w.writeln(&format!("  Whisper to {}:", target));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str(&format!("  {} > ", prompt));
    w.reset_color();

    w.flush()
}

/// Render game over screen
pub fn render_game_over(state: &GameState, victory: bool) -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.writeln("");

    if victory {
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln("   ██╗   ██╗██╗ ██████╗████████╗ ██████╗ ██████╗ ██╗   ██╗██╗");
        w.writeln("   ██║   ██║██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝██║");
        w.writeln("   ██║   ██║██║██║        ██║   ██║   ██║██████╔╝ ╚████╔╝ ██║");
        w.writeln("   ╚██╗ ██╔╝██║██║        ██║   ██║   ██║██╔══██╗  ╚██╔╝  ╚═╝");
        w.writeln("    ╚████╔╝ ██║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║   ██╗");
        w.writeln("     ╚═══╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝");
        w.reset_color();

        w.writeln("");
        w.set_fg(Color::LightMagenta);
        w.writeln("      You have become the ARCH-MAGE OF LEGENDS!");
        w.writeln("");
        w.set_fg(Color::White);
        w.writeln("  The dragon Pyraxis has been defeated. The throne is yours.");
        w.writeln("  Tashanna, Lady of Legends, smiles upon you from beyond.");
        w.writeln("");
        w.writeln("  Your name will be remembered for all eternity.");
    } else {
        w.set_fg(Color::LightRed);
        w.bold();
        w.writeln("   ██████╗ ███████╗███████╗███████╗ █████╗ ████████╗");
        w.writeln("   ██╔══██╗██╔════╝██╔════╝██╔════╝██╔══██╗╚══██╔══╝");
        w.writeln("   ██║  ██║█████╗  █████╗  █████╗  ███████║   ██║   ");
        w.writeln("   ██║  ██║██╔══╝  ██╔══╝  ██╔══╝  ██╔══██║   ██║   ");
        w.writeln("   ██████╔╝███████╗██║     ███████╗██║  ██║   ██║   ");
        w.writeln("   ╚═════╝ ╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ");
        w.reset_color();

        w.writeln("");
        w.set_fg(Color::LightGray);
        w.writeln("  Your journey has come to an end...");
        w.writeln("  But Morningmist awaits another brave soul.");
    }

    // Stats
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  FINAL STATS:");
    w.reset_color();
    w.set_fg(Color::White);
    w.writeln(&format!("  Level: {} ({})", state.level, state.rank().name()));
    w.writeln(&format!("  Gold Earned: {}", state.total_gold_earned));
    w.writeln(&format!("  Monsters Defeated: {}", state.monsters_killed));
    w.writeln(&format!("  Spells Learned: {}", state.known_spells.len()));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render confirm quit screen
pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Your progress will be saved and you can resume later.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  Are you sure you want to leave Morningmist? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render help screen
pub fn render_help() -> String {
    let mut w = AnsiWriter::new();

    w.clear_screen();
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  === MORNINGMIST HELP ===");
    w.reset_color();

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  MOVEMENT:");
    w.set_fg(Color::White);
    w.writeln("    north, south, east, west, up, down, in, out");
    w.writeln("    (or: n, s, e, w, u, d)");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  ACTIONS:");
    w.set_fg(Color::White);
    w.writeln("    look [item/npc]  - Examine something");
    w.writeln("    take [item]      - Pick up an item");
    w.writeln("    drop [item]      - Drop an item");
    w.writeln("    use [item]       - Use an item");
    w.writeln("    equip [item]     - Equip weapon or armor");
    w.writeln("    talk [npc]       - Talk to someone");
    w.writeln("    throw [item]     - Throw an item (for fountain)");
    w.writeln("    rest             - Rest at an inn");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  MAGIC:");
    w.set_fg(Color::White);
    w.writeln("    Type the spell's incantation to cast it!");
    w.writeln("    Example: 'vitae restauro' casts Heal");

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  QUICK KEYS:");
    w.set_fg(Color::White);
    w.writeln("    I - Inventory    M - Magic/Spells    S - Stats");
    w.writeln("    H - Help         Q - Quit");

    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  Press any key to return...");
    w.reset_color();

    w.flush()
}

/// Main render function
pub fn render_screen(flow: &KyrandiaFlow) -> String {
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(state),
        GameScreen::Exploration => render_exploration(state),
        GameScreen::Combat => render_combat(state),
        GameScreen::Dialogue { npc_key, dialogue_index } => {
            render_dialogue(state, npc_key, *dialogue_index)
        }
        GameScreen::Shop { npc_key: _ } => render_exploration(state),  // TODO: shop screen
        GameScreen::Inventory => render_inventory(state),
        GameScreen::Spellbook => render_spellbook(state),
        GameScreen::Stats => render_stats(state),
        GameScreen::TextInput { prompt, purpose } => render_text_input(prompt, purpose),
        GameScreen::Fountain => render_fountain(state),
        GameScreen::GameOver { victory } => render_game_over(state, *victory),
        GameScreen::Leaderboard => render_exploration(state),  // TODO: leaderboard
        GameScreen::ConfirmQuit => render_confirm_quit(),
        GameScreen::Help => render_help(),
    }
}

// ============================================================================
// HELPERS
// ============================================================================

/// Word wrap text to fit width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
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

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_intro() {
        let state = GameState::new("Test");
        let output = render_intro(&state);
        assert!(output.contains("MORNINGMIST"));
        assert!(output.contains("Test"));
    }

    #[test]
    fn test_render_exploration() {
        let state = GameState::new("Test");
        let output = render_exploration(&state);
        assert!(output.contains("Village Square"));
    }

    #[test]
    fn test_render_combat() {
        let mut state = GameState::new("Test");
        state.combat = Some(super::super::state::CombatState {
            monster_key: "rat".to_string(),
            monster_hp: 10,
            monster_max_hp: 10,
            player_turn: true,
            shield_active: false,
            shield_power: 0,
        });

        let output = render_combat(&state);
        assert!(output.contains("COMBAT"));
        assert!(output.contains("Giant Rat"));
    }

    #[test]
    fn test_word_wrap() {
        let text = "This is a long sentence that should be wrapped.";
        let lines = wrap_text(text, 20);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.len() <= 20));
    }
}
