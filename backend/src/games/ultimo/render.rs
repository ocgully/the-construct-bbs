//! ANSI rendering for Ultimo
//!
//! Medieval fantasy visual theme with earth tones and royal accents.

use crate::terminal::{AnsiWriter, Color};
use super::data::{get_item, get_npc, get_quest, get_skill, get_zone, NpcType, SkillCategory, TerrainType};
use super::screen::GameScreen;
use super::skills::determine_title;
use super::state::{Character, GameState};
use super::world::get_terrain_view;

// ============================================================================
// THEME COLORS
// ============================================================================

/// Medieval fantasy theme colors
const TITLE_COLOR: Color = Color::Yellow;
const BORDER_COLOR: Color = Color::Brown;
const HIGHLIGHT_COLOR: Color = Color::LightCyan;
const TEXT_COLOR: Color = Color::LightGray;
const DANGER_COLOR: Color = Color::LightRed;
const HEALTH_HIGH: Color = Color::LightGreen;
const HEALTH_MED: Color = Color::Yellow;
const HEALTH_LOW: Color = Color::LightRed;
const MANA_COLOR: Color = Color::LightBlue;
const GOLD_COLOR: Color = Color::Yellow;

// ============================================================================
// RENDER FUNCTIONS
// ============================================================================

/// Render the game header
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(BORDER_COLOR);
    w.writeln("╔══════════════════════════════════════════════════════════════════════════════╗");
    w.write_str("║");
    w.set_fg(TITLE_COLOR);
    w.bold();
    w.write_str("   ██╗   ██╗██╗  ████████╗██╗███╗   ███╗ ██████╗ ");
    w.set_fg(BORDER_COLOR);
    w.writeln("            ULTIMO         ║");

    w.write_str("║");
    w.set_fg(TITLE_COLOR);
    w.write_str("   ██║   ██║██║  ╚══██╔══╝██║████╗ ████║██╔═══██╗");
    w.set_fg(BORDER_COLOR);
    w.writeln("                           ║");

    w.write_str("║");
    w.set_fg(TITLE_COLOR);
    w.write_str("   ██║   ██║██║     ██║   ██║██╔████╔██║██║   ██║");
    w.set_fg(BORDER_COLOR);
    w.writeln("                           ║");

    w.write_str("║");
    w.set_fg(TITLE_COLOR);
    w.write_str("   ██║   ██║██║     ██║   ██║██║╚██╔╝██║██║   ██║");
    w.set_fg(BORDER_COLOR);
    w.writeln("                           ║");

    w.write_str("║");
    w.set_fg(TITLE_COLOR);
    w.write_str("   ╚██████╔╝███████╗██║   ██║██║ ╚═╝ ██║╚██████╔╝");
    w.set_fg(BORDER_COLOR);
    w.writeln("                           ║");

    w.write_str("║");
    w.set_fg(TITLE_COLOR);
    w.write_str("    ╚═════╝ ╚══════╝╚═╝   ╚═╝╚═╝     ╚═╝ ╚═════╝ ");
    w.set_fg(Color::DarkGray);
    w.write_str("A World of Adventure");
    w.set_fg(BORDER_COLOR);
    w.writeln("    ║");

    w.writeln("╚══════════════════════════════════════════════════════════════════════════════╝");
    w.reset_color();
}

/// Render character status bar
fn render_status_bar(w: &mut AnsiWriter, char: &Character) {
    w.set_fg(BORDER_COLOR);
    w.writeln("╔══════════════════════════════════════════════════════════════════════════════╗");

    w.write_str("║ ");

    // Name and title
    w.set_fg(Color::White);
    w.bold();
    w.write_str(&char.name);
    w.reset_color();
    w.set_fg(Color::DarkGray);
    w.write_str(&format!(" the {} ", determine_title(char)));

    // Level
    w.set_fg(HIGHLIGHT_COLOR);
    w.write_str(&format!("Lv{} ", char.level()));

    // HP bar
    let hp_pct = (char.hp as f32 / char.max_hp as f32 * 100.0) as u32;
    let hp_color = if hp_pct > 70 {
        HEALTH_HIGH
    } else if hp_pct > 30 {
        HEALTH_MED
    } else {
        HEALTH_LOW
    };
    w.set_fg(hp_color);
    w.write_str(&format!("HP:{}/{} ", char.hp, char.max_hp));

    // Mana bar
    w.set_fg(MANA_COLOR);
    w.write_str(&format!("MP:{}/{} ", char.mana, char.max_mana));

    // Gold
    w.set_fg(GOLD_COLOR);
    w.write_str(&format!("Gold:{}", char.gold));

    // Pad to end
    w.set_fg(BORDER_COLOR);
    let used_len = char.name.len() + 30 + format!("{}{}{}{}{}", char.level(), char.hp, char.max_hp, char.mana, char.max_mana).len() + format!("{}", char.gold).len();
    let padding = 77usize.saturating_sub(used_len);
    w.write_str(&" ".repeat(padding));
    w.writeln("║");

    // Location line
    w.write_str("║ ");
    w.set_fg(TEXT_COLOR);
    if let Some(zone) = get_zone(&char.position.zone) {
        w.write_str(zone.name);
        w.set_fg(Color::DarkGray);
        w.write_str(&format!(" ({},{}) ", char.position.x, char.position.y));
    }

    // XP
    w.set_fg(Color::LightMagenta);
    w.write_str(&format!("XP:{}", char.total_xp));

    w.set_fg(BORDER_COLOR);
    w.writeln(&format!("{:>40}║", ""));

    w.writeln("╚══════════════════════════════════════════════════════════════════════════════╝");
    w.reset_color();
}

/// Render terrain view with player
fn render_terrain(w: &mut AnsiWriter, char: &Character, state: &GameState) {
    let zone = match get_zone(&char.position.zone) {
        Some(z) => z,
        None => return,
    };

    // Get 21x11 view centered on player
    let terrain = get_terrain_view(zone, &char.position, 21, 11);

    w.set_fg(BORDER_COLOR);
    w.writeln("  ┌─────────────────────┐");

    for (vy, row) in terrain.iter().enumerate() {
        w.set_fg(BORDER_COLOR);
        w.write_str("  │");

        for (vx, cell) in row.iter().enumerate() {
            // Check if player is at this position
            let is_player = vx == 10 && vy == 5;

            // Check if NPC is at this position
            let world_x = char.position.x - 10 + vx as i32;
            let world_y = char.position.y - 5 + vy as i32;

            let mut is_npc = false;
            for npc in super::data::NPCS {
                if npc.zone == char.position.zone
                    && npc.position.0 == world_x
                    && npc.position.1 == world_y
                {
                    is_npc = true;
                    break;
                }
            }

            // Check if other player is at this position
            let is_other_player = state.visible_players.iter().any(|p| {
                p.x == world_x && p.y == world_y
            });

            if is_player {
                w.set_fg(Color::LightGreen);
                w.write_str("@");
            } else if is_npc {
                w.set_fg(Color::LightCyan);
                w.write_str("N");
            } else if is_other_player {
                w.set_fg(Color::LightMagenta);
                w.write_str("P");
            } else {
                let (color, ch) = match cell {
                    TerrainType::Grass => (Color::Green, '.'),
                    TerrainType::Water => (Color::Blue, '~'),
                    TerrainType::Mountain => (Color::DarkGray, '^'),
                    TerrainType::Forest => (Color::Green, 'T'),
                    TerrainType::Sand => (Color::Yellow, ':'),
                    TerrainType::Stone => (Color::LightGray, '#'),
                    TerrainType::Road => (Color::Brown, '='),
                    TerrainType::Building => (Color::White, '+'),
                    TerrainType::Wall => (Color::DarkGray, '#'),
                    TerrainType::Door => (Color::Yellow, 'D'),
                };
                w.set_fg(color);
                w.write_str(&ch.to_string());
            }
        }

        w.set_fg(BORDER_COLOR);
        w.writeln("│");
    }

    w.writeln("  └─────────────────────┘");
    w.reset_color();
}

/// Render intro screen
pub fn render_intro(_state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(TEXT_COLOR);
    w.writeln("  In a land where virtue guides the righteous and darkness tempts the weak,");
    w.writeln("  a new adventurer prepares to make their mark upon the realm of Britannia.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln("  Will you become a legendary warrior, mastering blade and shield?");
    w.writeln("  Perhaps a powerful mage, commanding the arcane forces of the universe?");
    w.writeln("  Or maybe a cunning merchant, building an empire through trade?");
    w.writeln("");
    w.set_fg(HIGHLIGHT_COLOR);
    w.writeln("  The choice is yours. Your destiny awaits.");
    w.writeln("");
    w.set_fg(TEXT_COLOR);
    w.writeln("  - Skill-based progression: become what you practice");
    w.writeln("  - Explore towns, wilderness, and deadly dungeons");
    w.writeln("  - Fight monsters and other players");
    w.writeln("  - Craft items and own property");
    w.writeln("  - Trade with other adventurers");
    w.writeln("");
    w.set_fg(HIGHLIGHT_COLOR);
    w.writeln("  Press any key to begin your journey...");
    w.reset_color();

    w.flush()
}

/// Render character creation name screen
pub fn render_character_create(_state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(TITLE_COLOR);
    w.bold();
    w.writeln("  CREATE YOUR ADVENTURER");
    w.reset_color();
    w.writeln("");
    w.set_fg(TEXT_COLOR);
    w.writeln("  What shall you be called in this realm?");
    w.writeln("  (3-15 characters, letters and numbers only)");
    w.writeln("");
    w.set_fg(HIGHLIGHT_COLOR);
    w.write_str("  Enter your name: ");
    w.reset_color();

    w.flush()
}

/// Render stat allocation screen
pub fn render_stat_allocation(state: &GameState, points_remaining: u32) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    if let Some(ref char) = state.character {
        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  Welcome, {}!", char.name));
        w.reset_color();
        w.writeln("");
        w.set_fg(TEXT_COLOR);
        w.writeln("  Allocate your starting attributes.");
        w.writeln("  Each point provides bonuses to related activities.");
        w.writeln("");

        w.set_fg(HIGHLIGHT_COLOR);
        w.writeln(&format!("  Points remaining: {}", points_remaining));
        w.writeln("");

        // Stats
        w.set_fg(Color::LightRed);
        w.write_str(&format!("  [1] Strength:     {:2}", char.strength));
        w.set_fg(Color::DarkGray);
        w.writeln("  - Melee damage, carrying capacity");

        w.set_fg(Color::LightGreen);
        w.write_str(&format!("  [2] Dexterity:    {:2}", char.dexterity));
        w.set_fg(Color::DarkGray);
        w.writeln("  - Hit chance, dodge, stamina");

        w.set_fg(Color::LightBlue);
        w.write_str(&format!("  [3] Intelligence: {:2}", char.intelligence));
        w.set_fg(Color::DarkGray);
        w.writeln("  - Magic power, mana pool");

        w.writeln("");
        if points_remaining == 0 {
            w.set_fg(Color::LightGreen);
            w.writeln("  [F] Finish and enter the world!");
        }
    }

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render main world view
pub fn render_world_view(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
            w.reset_color();
        }

        // Render terrain
        render_terrain(&mut w, char, state);

        // Commands
        w.set_fg(Color::DarkGray);
        w.writeln("  WASD=Move C=Stats I=Inv K=Skills J=Quests R=Craft O=House M=Trade T=Talk F=Fight Q=Quit");
        w.reset_color();
    }

    w.flush()
}

/// Render character stats screen
pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  CHARACTER STATS");
        w.reset_color();
        w.writeln("");

        // Base stats
        w.set_fg(Color::White);
        w.writeln(&format!("  Strength:     {}", char.strength));
        w.writeln(&format!("  Dexterity:    {}", char.dexterity));
        w.writeln(&format!("  Intelligence: {}", char.intelligence));
        w.writeln("");

        // Combat stats
        w.set_fg(HIGHLIGHT_COLOR);
        w.writeln("  Combat:");
        w.set_fg(TEXT_COLOR);
        w.writeln(&format!("    Attack Power: {}", char.attack_power()));
        w.writeln(&format!("    Defense:      {}", char.defense()));
        w.writeln("");

        // Equipment
        w.set_fg(HIGHLIGHT_COLOR);
        w.writeln("  Equipment:");
        w.set_fg(TEXT_COLOR);
        let weapon = char.equipped_weapon.as_ref()
            .and_then(|k| get_item(k))
            .map(|i| i.name)
            .unwrap_or("Bare Fists");
        let armor = char.equipped_armor.as_ref()
            .and_then(|k| get_item(k))
            .map(|i| i.name)
            .unwrap_or("None");
        let shield = char.equipped_shield.as_ref()
            .and_then(|k| get_item(k))
            .map(|i| i.name)
            .unwrap_or("None");
        w.writeln(&format!("    Weapon: {}", weapon));
        w.writeln(&format!("    Armor:  {}", armor));
        w.writeln(&format!("    Shield: {}", shield));
        w.writeln("");

        // Statistics
        w.set_fg(HIGHLIGHT_COLOR);
        w.writeln("  Statistics:");
        w.set_fg(TEXT_COLOR);
        w.writeln(&format!("    Total XP: {}", char.total_xp));
        w.writeln(&format!("    Monsters Slain: {}", char.kills.values().sum::<u32>()));
        w.writeln(&format!("    Deaths: {}", char.deaths));
        w.writeln(&format!("    PvP Kills: {}", char.pvp_kills));

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render inventory screen
pub fn render_inventory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  INVENTORY ({}/{})", char.inventory.len(), char.max_inventory_slots));
        w.reset_color();
        w.writeln("");

        if char.inventory.is_empty() {
            w.set_fg(Color::DarkGray);
            w.writeln("  Your pack is empty.");
        } else {
            for (i, slot) in char.inventory.iter().enumerate() {
                if let Some(item) = get_item(&slot.item_key) {
                    let equipped = if Some(&slot.item_key) == char.equipped_weapon.as_ref()
                        || Some(&slot.item_key) == char.equipped_armor.as_ref()
                        || Some(&slot.item_key) == char.equipped_shield.as_ref()
                    {
                        " (equipped)"
                    } else {
                        ""
                    };

                    w.set_fg(HIGHLIGHT_COLOR);
                    w.write_str(&format!("  [{:2}] ", i + 1));
                    w.set_fg(Color::White);
                    w.write_str(&format!("{:<20}", item.name));
                    w.set_fg(Color::DarkGray);
                    w.write_str(&format!(" x{}", slot.quantity));
                    w.set_fg(Color::LightGreen);
                    w.writeln(equipped);
                }
            }
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to use/equip item, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render skills screen
pub fn render_skills(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  SKILLS");
        w.reset_color();
        w.writeln("");

        let categories = [
            (SkillCategory::Combat, "Combat"),
            (SkillCategory::Magic, "Magic"),
            (SkillCategory::Crafting, "Crafting"),
            (SkillCategory::Gathering, "Gathering"),
            (SkillCategory::Miscellaneous, "Misc"),
        ];

        for (category, name) in &categories {
            let skills: Vec<_> = super::data::SKILLS
                .iter()
                .filter(|s| s.category == *category)
                .collect();

            if skills.iter().any(|s| char.get_skill(s.key) > 0) {
                w.set_fg(HIGHLIGHT_COLOR);
                w.writeln(&format!("  {}:", name));

                for skill in skills {
                    let level = char.get_skill(skill.key);
                    if level > 0 {
                        w.set_fg(Color::White);
                        w.write_str(&format!("    {:<20}", skill.name));
                        w.set_fg(skill_color(level));
                        w.writeln(&format!("{:3}", level));
                    }
                }
                w.writeln("");
            }
        }

        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Back");
    }

    w.reset_color();
    w.flush()
}

fn skill_color(level: u32) -> Color {
    if level >= 80 {
        Color::LightMagenta
    } else if level >= 60 {
        Color::LightCyan
    } else if level >= 40 {
        Color::LightGreen
    } else if level >= 20 {
        Color::Yellow
    } else {
        Color::LightGray
    }
}

/// Render quest log
pub fn render_quests(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  QUEST LOG");
        w.reset_color();
        w.writeln("");

        if char.active_quests.is_empty() {
            w.set_fg(Color::DarkGray);
            w.writeln("  No active quests. Talk to NPCs to find work!");
        } else {
            for (i, progress) in char.active_quests.iter().enumerate() {
                if let Some(quest) = get_quest(&progress.quest_key) {
                    w.set_fg(HIGHLIGHT_COLOR);
                    w.write_str(&format!("  [{:2}] ", i + 1));
                    w.set_fg(Color::White);
                    w.writeln(quest.name);
                    w.set_fg(TEXT_COLOR);
                    w.writeln(&format!("       {}", quest.description));

                    // Progress
                    if let Some((monster, count)) = &quest.requirements.kill_monsters {
                        w.set_fg(Color::DarkGray);
                        w.writeln(&format!("       Kill {}: {}/{}", monster, progress.kills, count));
                    }
                    if let Some((item, count)) = &quest.requirements.collect_items {
                        let have = char.get_item_count(item);
                        w.set_fg(Color::DarkGray);
                        w.writeln(&format!("       Collect {}: {}/{}", item, have, count));
                    }

                    // Completion status
                    if char.can_complete_quest(&progress.quest_key) {
                        w.set_fg(Color::LightGreen);
                        w.writeln("       [Ready to turn in!]");
                    }
                    w.writeln("");
                }
            }
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln(&format!("  Quests completed: {}", char.completed_quests.len()));
        w.writeln("  Enter number to turn in, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render combat screen
pub fn render_combat(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let (Some(ref char), Some(ref combat)) = (&state.character, &state.combat) {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(DANGER_COLOR);
        w.bold();
        w.writeln(&format!("  COMBAT: {} (Lv{})", combat.monster.name, combat.monster.level));
        w.reset_color();
        w.writeln("");

        // Monster ASCII art
        if let Some(template) = super::data::get_monster_template(&combat.monster.template_key) {
            w.set_fg(Color::LightRed);
            w.writeln(&format!("       {}", template.ascii_art));
            w.reset_color();
        }

        // Monster HP bar
        let hp_pct = (combat.monster.hp as f32 / combat.monster.max_hp as f32 * 100.0) as u32;
        w.set_fg(DANGER_COLOR);
        w.writeln(&format!("  Enemy HP: {}/{} ({}%)", combat.monster.hp, combat.monster.max_hp, hp_pct));
        w.writeln("");

        // Combat log (last 5 entries)
        w.set_fg(Color::DarkGray);
        w.writeln("  Combat Log:");
        for msg in combat.combat_log.iter().rev().take(5).rev() {
            w.set_fg(TEXT_COLOR);
            w.writeln(&format!("    {}", msg));
        }
        w.writeln("");

        // Actions
        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [A] ");
        w.set_fg(Color::White);
        w.writeln("Attack");

        if char.mana > 0 {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str("  [C] ");
            w.set_fg(Color::White);
            w.writeln("Cast Spell");
        }

        if char.get_item_count("heal_potion") > 0 || char.get_item_count("lesser_heal_potion") > 0 {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str("  [U] ");
            w.set_fg(Color::White);
            w.writeln("Use Potion");
        }

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [R] ");
        w.set_fg(Color::White);
        w.writeln("Run Away");
    }

    w.reset_color();
    w.flush()
}

/// Render NPC dialogue screen
pub fn render_npc_dialogue(state: &GameState, npc_key: &str) -> String {
    let mut w = AnsiWriter::new();

    if let (Some(ref char), Some(npc)) = (&state.character, get_npc(npc_key)) {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  {}", npc.name));
        w.reset_color();
        w.writeln("");

        w.set_fg(TEXT_COLOR);
        w.writeln(&format!("  \"{}\"", npc.dialogue));
        w.writeln("");

        // Options based on NPC type
        let mut option = 1;

        if !npc.shop_inventory.is_empty() {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", option));
            w.set_fg(Color::White);
            w.writeln("Buy items");
            option += 1;

            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", option));
            w.set_fg(Color::White);
            w.writeln("Sell items");
            option += 1;
        }

        if !npc.trains_skills.is_empty() {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", option));
            w.set_fg(Color::White);
            w.writeln("Train skills");
            option += 1;
        }

        if npc.npc_type == NpcType::QuestGiver {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", option));
            w.set_fg(Color::White);
            w.writeln("Accept quest");
            option += 1;
        }

        if npc.npc_type == NpcType::Banker {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", option));
            w.set_fg(Color::White);
            w.writeln("Bank");
            option += 1;
        }

        if npc.npc_type == NpcType::Healer {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", option));
            w.set_fg(Color::White);
            w.writeln("Heal");
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Leave");
    }

    w.reset_color();
    w.flush()
}

/// Render shop buy screen
pub fn render_shop_buy(state: &GameState, npc_key: &str) -> String {
    let mut w = AnsiWriter::new();

    if let (Some(ref char), Some(npc)) = (&state.character, get_npc(npc_key)) {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  {} - BUY", npc.name));
        w.reset_color();
        w.writeln("");

        w.set_fg(GOLD_COLOR);
        w.writeln(&format!("  Your gold: {}", char.gold));
        w.writeln("");

        for (i, (item_key, price_mult)) in npc.shop_inventory.iter().enumerate() {
            if let Some(item) = get_item(item_key) {
                let price = (item.base_price as f32 * price_mult) as i64;
                let can_afford = char.gold >= price;

                if can_afford {
                    w.set_fg(HIGHLIGHT_COLOR);
                } else {
                    w.set_fg(Color::DarkGray);
                }
                w.write_str(&format!("  [{:2}] ", i + 1));
                w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
                w.write_str(&format!("{:<20}", item.name));
                w.set_fg(if can_afford { GOLD_COLOR } else { Color::DarkGray });
                w.writeln(&format!("{:>8} gold", price));
            }
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to buy, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render shop sell screen
pub fn render_shop_sell(state: &GameState, npc_key: &str) -> String {
    let mut w = AnsiWriter::new();

    if let (Some(ref char), Some(npc)) = (&state.character, get_npc(npc_key)) {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  {} - SELL", npc.name));
        w.reset_color();
        w.writeln("");

        if char.inventory.is_empty() {
            w.set_fg(Color::DarkGray);
            w.writeln("  You have nothing to sell.");
        } else {
            for (i, slot) in char.inventory.iter().enumerate() {
                if let Some(item) = get_item(&slot.item_key) {
                    let sell_price = item.base_price / 2;
                    w.set_fg(HIGHLIGHT_COLOR);
                    w.write_str(&format!("  [{:2}] ", i + 1));
                    w.set_fg(Color::White);
                    w.write_str(&format!("{:<20}", item.name));
                    w.set_fg(Color::DarkGray);
                    w.write_str(&format!(" x{}", slot.quantity));
                    w.set_fg(GOLD_COLOR);
                    w.writeln(&format!("  -> {} gold", sell_price));
                }
            }
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to sell, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render bank screen
pub fn render_bank(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  BANK OF BRITANNIA");
        w.reset_color();
        w.writeln("");

        w.set_fg(GOLD_COLOR);
        w.writeln(&format!("  Gold on hand: {}", char.gold));
        w.writeln(&format!("  Gold in bank: {}", char.bank_gold));
        w.writeln("");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [D] ");
        w.set_fg(Color::White);
        w.writeln("Deposit all gold");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [W] ");
        w.set_fg(Color::White);
        w.writeln("Withdraw all gold");

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Leave");
    }

    w.reset_color();
    w.flush()
}

/// Render healer screen
pub fn render_healer(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  HEALER'S SANCTUARY");
        w.reset_color();
        w.writeln("");

        let missing_hp = char.max_hp - char.hp;
        let heal_cost = (missing_hp as i64 / 10 + 1) * 10;

        w.set_fg(TEXT_COLOR);
        w.writeln(&format!("  Current HP: {}/{}", char.hp, char.max_hp));
        if missing_hp > 0 {
            w.writeln(&format!("  Healing cost: {} gold", heal_cost));
        } else {
            w.set_fg(Color::LightGreen);
            w.writeln("  You are at full health!");
        }
        w.writeln("");

        if missing_hp > 0 {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str("  [H] ");
            w.set_fg(Color::White);
            w.writeln("Heal wounds");
        }

        if char.is_dead {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str("  [R] ");
            w.set_fg(Color::White);
            w.writeln("Resurrect (500 gold)");
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Leave");
    }

    w.reset_color();
    w.flush()
}

/// Render crafting menu
pub fn render_crafting(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  CRAFTING");
        w.reset_color();
        w.writeln("");

        w.set_fg(TEXT_COLOR);
        w.writeln("  Choose a crafting profession:");
        w.writeln("");

        let skills = [
            ("blacksmithing", "Blacksmithing", "Forge weapons and armor"),
            ("tailoring", "Tailoring", "Sew cloth and leather items"),
            ("carpentry", "Carpentry", "Craft wooden items and bows"),
            ("alchemy", "Alchemy", "Brew potions and elixirs"),
            ("cooking", "Cooking", "Prepare food for healing"),
        ];

        for (i, (skill_key, name, desc)) in skills.iter().enumerate() {
            let level = char.get_skill(skill_key);
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str(&format!("  [{}] ", i + 1));
            w.set_fg(Color::White);
            w.write_str(&format!("{:<15}", name));
            w.set_fg(skill_color(level));
            w.write_str(&format!("Lv{:3}", level));
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("  - {}", desc));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to view recipes, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render crafting skill recipes
pub fn render_crafting_skill(state: &GameState, skill: &str) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        let skill_name = match skill {
            "blacksmithing" => "BLACKSMITHING",
            "tailoring" => "TAILORING",
            "carpentry" => "CARPENTRY",
            "alchemy" => "ALCHEMY",
            "cooking" => "COOKING",
            _ => "CRAFTING",
        };

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  {} RECIPES", skill_name));
        w.reset_color();
        w.writeln("");

        let recipes: Vec<_> = super::crafting::RECIPES
            .iter()
            .filter(|r| r.required_skill == skill)
            .collect();

        if recipes.is_empty() {
            w.set_fg(Color::DarkGray);
            w.writeln("  No recipes available.");
        } else {
            let char_skill = char.get_skill(skill);

            for (i, recipe) in recipes.iter().enumerate() {
                let can_craft = char_skill >= recipe.min_skill;

                // Check materials
                let mut has_mats = true;
                for (mat_key, needed) in recipe.materials {
                    if char.get_item_count(mat_key) < *needed {
                        has_mats = false;
                        break;
                    }
                }

                if can_craft {
                    w.set_fg(HIGHLIGHT_COLOR);
                } else {
                    w.set_fg(Color::DarkGray);
                }
                w.write_str(&format!("  [{:2}] ", i + 1));

                w.set_fg(if can_craft { Color::White } else { Color::DarkGray });
                w.write_str(&format!("{:<25}", recipe.name));

                w.set_fg(if can_craft { Color::Yellow } else { Color::DarkGray });
                w.write_str(&format!("Req:{:3} ", recipe.min_skill));

                w.set_fg(if has_mats && can_craft { Color::LightGreen } else { Color::DarkGray });
                w.writeln(if has_mats { "[READY]" } else { "[need mats]" });

                // Show materials
                w.set_fg(Color::DarkGray);
                w.write_str("       ");
                for (mat_key, needed) in recipe.materials {
                    let have = char.get_item_count(mat_key);
                    let mat_name = get_item(mat_key)
                        .map(|i| i.name)
                        .unwrap_or(*mat_key);
                    w.write_str(&format!("{} {}/{} ", mat_name, have, needed));
                }
                w.writeln("");
            }
        }

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to craft, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render housing menu
pub fn render_housing(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  HOUSING");
        w.reset_color();
        w.writeln("");

        if let Some(_house_id) = char.house_id {
            w.set_fg(Color::LightGreen);
            w.writeln("  You own a house!");
            w.writeln("");
        } else {
            w.set_fg(Color::DarkGray);
            w.writeln("  You do not own a house yet.");
            w.writeln("");
        }

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [1] ");
        w.set_fg(Color::White);
        w.writeln("Buy a House");

        if char.house_id.is_some() {
            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str("  [2] ");
            w.set_fg(Color::White);
            w.writeln("Access Storage");

            w.set_fg(HIGHLIGHT_COLOR);
            w.write_str("  [3] ");
            w.set_fg(Color::White);
            w.writeln("Manage Friends");
        }

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render housing buy screen
pub fn render_housing_buy(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  BUY A HOUSE");
        w.reset_color();
        w.writeln("");

        w.set_fg(GOLD_COLOR);
        w.writeln(&format!("  Your gold: {}", char.gold));
        w.writeln("");

        let house_types = [
            (super::housing::HouseType::SmallCottage, "Small and cozy"),
            (super::housing::HouseType::MediumHouse, "Room for a family"),
            (super::housing::HouseType::LargeHouse, "Spacious estate"),
            (super::housing::HouseType::Tower, "Mage's retreat"),
            (super::housing::HouseType::Castle, "Ultimate luxury"),
        ];

        for (i, (house_type, desc)) in house_types.iter().enumerate() {
            let price = house_type.price();
            let storage = house_type.storage_slots();
            let can_afford = char.gold >= price;

            if can_afford {
                w.set_fg(HIGHLIGHT_COLOR);
            } else {
                w.set_fg(Color::DarkGray);
            }
            w.write_str(&format!("  [{}] ", i + 1));

            w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
            w.write_str(&format!("{:<15}", house_type.name()));

            w.set_fg(if can_afford { GOLD_COLOR } else { Color::DarkGray });
            w.write_str(&format!("{:>10} gold  ", price));

            w.set_fg(Color::DarkGray);
            w.writeln(&format!("{} slots - {}", storage, desc));
        }

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to buy, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render housing storage screen
pub fn render_housing_storage(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  HOUSE STORAGE");
        w.reset_color();
        w.writeln("");

        w.set_fg(TEXT_COLOR);
        w.writeln("  Your inventory:");
        w.writeln("");

        for (i, slot) in char.inventory.iter().enumerate() {
            if let Some(item) = get_item(&slot.item_key) {
                w.set_fg(HIGHLIGHT_COLOR);
                w.write_str(&format!("  [{:2}] ", i + 1));
                w.set_fg(Color::White);
                w.write_str(&format!("{:<20}", item.name));
                w.set_fg(Color::DarkGray);
                w.writeln(&format!(" x{}", slot.quantity));
            }
        }

        w.writeln("");
        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [D] ");
        w.set_fg(Color::White);
        w.writeln("Deposit items");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [W] ");
        w.set_fg(Color::White);
        w.writeln("Withdraw items");

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to deposit, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render trade list screen
pub fn render_trade_list(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  PLAYER MARKETPLACE");
        w.reset_color();
        w.writeln("");

        w.set_fg(GOLD_COLOR);
        w.writeln(&format!("  Your gold: {}", char.gold));
        w.writeln("");

        w.set_fg(TEXT_COLOR);
        w.writeln("  Available Listings:");
        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  No listings available.");
        w.writeln("");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [C] ");
        w.set_fg(Color::White);
        w.writeln("Create new listing");

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to buy, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render trade create screen
pub fn render_trade_create(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  CREATE TRADE LISTING");
        w.reset_color();
        w.writeln("");

        w.set_fg(TEXT_COLOR);
        w.writeln("  Select an item from your inventory to sell:");
        w.writeln("");

        for (i, slot) in char.inventory.iter().enumerate() {
            if let Some(item) = get_item(&slot.item_key) {
                let suggested_price = (item.base_price as f32 * 0.75) as i64;
                w.set_fg(HIGHLIGHT_COLOR);
                w.write_str(&format!("  [{:2}] ", i + 1));
                w.set_fg(Color::White);
                w.write_str(&format!("{:<20}", item.name));
                w.set_fg(Color::DarkGray);
                w.write_str(&format!(" x{}", slot.quantity));
                w.set_fg(GOLD_COLOR);
                w.writeln(&format!("  ~{} gold", suggested_price));
            }
        }

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to create listing, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render party screen
pub fn render_party(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    if let Some(ref char) = state.character {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln("  PARTY MANAGEMENT");
        w.reset_color();
        w.writeln("");

        w.set_fg(TEXT_COLOR);
        w.writeln("  You are not in a party.");
        w.writeln("");

        // Show nearby players
        if !state.visible_players.is_empty() {
            w.set_fg(HIGHLIGHT_COLOR);
            w.writeln("  Nearby players:");
            for player in &state.visible_players {
                w.set_fg(Color::LightCyan);
                w.write_str(&format!("    {} ", player.name));
                w.set_fg(Color::DarkGray);
                w.writeln(&format!("(Lv{})", player.level));
            }
            w.writeln("");
        }

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [C] ");
        w.set_fg(Color::White);
        w.writeln("Create party");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [I] ");
        w.set_fg(Color::White);
        w.writeln("Invite player");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [L] ");
        w.set_fg(Color::White);
        w.writeln("Leave party");

        // Show last message if any
        if let Some(ref msg) = char.last_message {
            w.writeln("");
            w.set_fg(Color::Yellow);
            w.writeln(&format!("  >> {} <<", msg));
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render training screen
pub fn render_training(state: &GameState, npc_key: &str) -> String {
    let mut w = AnsiWriter::new();

    if let (Some(ref char), Some(npc)) = (&state.character, get_npc(npc_key)) {
        w.clear_screen();
        render_status_bar(&mut w, char);

        w.writeln("");
        w.set_fg(TITLE_COLOR);
        w.bold();
        w.writeln(&format!("  {} - TRAINING", npc.name));
        w.reset_color();
        w.writeln("");

        w.set_fg(GOLD_COLOR);
        w.writeln(&format!("  Your gold: {}", char.gold));
        w.writeln("");

        for (i, skill_key) in npc.trains_skills.iter().enumerate() {
            if let Some(skill) = get_skill(skill_key) {
                let current = char.get_skill(skill_key);
                let cost = super::skills::training_cost(current);

                w.set_fg(HIGHLIGHT_COLOR);
                w.write_str(&format!("  [{:2}] ", i + 1));
                w.set_fg(Color::White);
                w.write_str(&format!("{:<20}", skill.name));
                w.set_fg(skill_color(current));
                w.write_str(&format!("{:3}", current));
                w.set_fg(GOLD_COLOR);
                w.writeln(&format!("  -> {} gold", cost));
            }
        }

        w.writeln("");
        w.set_fg(Color::DarkGray);
        w.writeln("  Enter number to train, [Q] Back");
    }

    w.reset_color();
    w.flush()
}

/// Render death screen
pub fn render_dead(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(DANGER_COLOR);
    w.bold();
    w.writeln("");
    w.writeln("                              YOU DIED");
    w.writeln("");
    w.writeln("  ██╗   ██╗ ██████╗ ██╗   ██╗    ██████╗ ██╗███████╗██████╗ ");
    w.writeln("  ╚██╗ ██╔╝██╔═══██╗██║   ██║    ██╔══██╗██║██╔════╝██╔══██╗");
    w.writeln("   ╚████╔╝ ██║   ██║██║   ██║    ██║  ██║██║█████╗  ██║  ██║");
    w.writeln("    ╚██╔╝  ██║   ██║██║   ██║    ██║  ██║██║██╔══╝  ██║  ██║");
    w.writeln("     ██║   ╚██████╔╝╚██████╔╝    ██████╔╝██║███████╗██████╔╝");
    w.writeln("     ╚═╝    ╚═════╝  ╚═════╝     ╚═════╝ ╚═╝╚══════╝╚═════╝ ");
    w.reset_color();
    w.writeln("");

    w.set_fg(TEXT_COLOR);
    w.writeln("  Your spirit wanders the ethereal void...");
    w.writeln("");

    if let Some(ref char) = state.character {
        w.writeln(&format!("  Gold remaining: {}", char.gold));
        w.writeln("");

        w.set_fg(HIGHLIGHT_COLOR);
        w.write_str("  [R] ");
        w.set_fg(Color::White);
        w.writeln("Resurrect at healer (500 gold)");
    }

    w.set_fg(HIGHLIGHT_COLOR);
    w.write_str("  [Q] ");
    w.set_fg(Color::White);
    w.writeln("Quit game");

    w.reset_color();
    w.flush()
}

/// Render confirm quit screen
pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(TITLE_COLOR);
    w.bold();
    w.writeln("");
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");
    w.set_fg(TEXT_COLOR);
    w.writeln("  Your progress will be saved.");
    w.writeln("");
    w.set_fg(HIGHLIGHT_COLOR);
    w.write_str("  Are you sure you want to quit? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render leaderboard
pub fn render_leaderboard(entries: &[(String, u32, i64)]) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(TITLE_COLOR);
    w.bold();
    w.writeln("");
    w.writeln("  HALL OF LEGENDS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {:>4} {:<20} {:>8} {:>12}", "Rank", "Name", "Level", "Net Worth"));
    w.writeln(&format!("  {}", "─".repeat(50)));
    w.reset_color();

    if entries.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  No legends yet. Will you be the first?");
    } else {
        for (i, (name, level, worth)) in entries.iter().enumerate() {
            let rank_color = match i {
                0 => Color::Yellow,
                1 => Color::White,
                2 => Color::Brown,
                _ => Color::LightGray,
            };
            w.set_fg(rank_color);
            w.write_str(&format!("  {:>4} ", i + 1));
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("{:<20}", name));
            w.set_fg(Color::White);
            w.write_str(&format!("{:>8}", level));
            w.set_fg(GOLD_COLOR);
            w.writeln(&format!("{:>12}", worth));
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Main render function that dispatches to the appropriate screen renderer
pub fn render_screen(flow: &super::screen::UltimoFlow) -> String {
    let state = &flow.state;

    match &flow.screen {
        GameScreen::Intro => render_intro(state),
        GameScreen::CharacterCreate => render_character_create(state),
        GameScreen::StatAllocation { points_remaining } => {
            render_stat_allocation(state, *points_remaining)
        }
        GameScreen::WorldView => render_world_view(state),
        GameScreen::Stats => render_stats(state),
        GameScreen::Inventory => render_inventory(state),
        GameScreen::Skills => render_skills(state),
        GameScreen::Quests => render_quests(state),
        GameScreen::Combat => render_combat(state),
        GameScreen::NpcDialogue { npc_key } => render_npc_dialogue(state, npc_key),
        GameScreen::ShopBuy { npc_key } => render_shop_buy(state, npc_key),
        GameScreen::ShopSell { npc_key } => render_shop_sell(state, npc_key),
        GameScreen::Bank => render_bank(state),
        GameScreen::Healer => render_healer(state),
        GameScreen::Training { npc_key } => render_training(state, npc_key),
        GameScreen::Crafting => render_crafting(state),
        GameScreen::CraftingSkill { skill } => render_crafting_skill(state, skill),
        GameScreen::Housing => render_housing(state),
        GameScreen::HousingBuy => render_housing_buy(state),
        GameScreen::HousingStorage => render_housing_storage(state),
        GameScreen::TradeList => render_trade_list(state),
        GameScreen::TradeCreate => render_trade_create(state),
        GameScreen::Trade { .. } => render_trade_list(state), // Direct trade uses same list for now
        GameScreen::Party => render_party(state),
        GameScreen::Leaderboard => render_leaderboard(&[]),
        GameScreen::Dead => render_dead(state),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::state::Character;

    fn create_test_state() -> GameState {
        let mut state = GameState::new();
        state.character = Some(Character::new("TestHero", 1));
        state
    }

    #[test]
    fn test_render_intro() {
        let state = GameState::new();
        let output = render_intro(&state);
        assert!(output.contains("ULTIMO"));
        assert!(output.contains("World of Adventure"));
    }

    #[test]
    fn test_render_character_create() {
        let state = GameState::new();
        let output = render_character_create(&state);
        assert!(output.contains("CREATE YOUR ADVENTURER"));
        assert!(output.contains("Enter your name"));
    }

    #[test]
    fn test_render_stat_allocation() {
        let state = create_test_state();
        let output = render_stat_allocation(&state, 15);
        assert!(output.contains("Points remaining: 15"));
        assert!(output.contains("Strength"));
        assert!(output.contains("Dexterity"));
        assert!(output.contains("Intelligence"));
    }

    #[test]
    fn test_render_world_view() {
        let state = create_test_state();
        let output = render_world_view(&state);
        assert!(output.contains("HP:"));
        assert!(output.contains("MP:"));
        assert!(output.contains("Gold:"));
    }

    #[test]
    fn test_render_stats() {
        let state = create_test_state();
        let output = render_stats(&state);
        assert!(output.contains("CHARACTER STATS"));
        assert!(output.contains("Strength:"));
        assert!(output.contains("Attack Power:"));
    }

    #[test]
    fn test_render_inventory() {
        let state = create_test_state();
        let output = render_inventory(&state);
        assert!(output.contains("INVENTORY"));
    }

    #[test]
    fn test_render_skills() {
        let state = create_test_state();
        let output = render_skills(&state);
        assert!(output.contains("SKILLS"));
    }

    #[test]
    fn test_render_crafting() {
        let state = create_test_state();
        let output = render_crafting(&state);
        assert!(output.contains("CRAFTING"));
        assert!(output.contains("Blacksmithing"));
        assert!(output.contains("Tailoring"));
        assert!(output.contains("Carpentry"));
        assert!(output.contains("Alchemy"));
        assert!(output.contains("Cooking"));
    }

    #[test]
    fn test_render_crafting_skill() {
        let state = create_test_state();
        let output = render_crafting_skill(&state, "blacksmithing");
        assert!(output.contains("BLACKSMITHING RECIPES"));
        assert!(output.contains("Smelt Iron"));
    }

    #[test]
    fn test_render_housing() {
        let state = create_test_state();
        let output = render_housing(&state);
        assert!(output.contains("HOUSING"));
        assert!(output.contains("Buy a House"));
    }

    #[test]
    fn test_render_housing_buy() {
        let state = create_test_state();
        let output = render_housing_buy(&state);
        assert!(output.contains("BUY A HOUSE"));
        assert!(output.contains("Small Cottage"));
        assert!(output.contains("Castle"));
        assert!(output.contains("gold"));
    }

    #[test]
    fn test_render_housing_storage() {
        let state = create_test_state();
        let output = render_housing_storage(&state);
        assert!(output.contains("HOUSE STORAGE"));
        assert!(output.contains("Deposit"));
        assert!(output.contains("Withdraw"));
    }

    #[test]
    fn test_render_trade_list() {
        let state = create_test_state();
        let output = render_trade_list(&state);
        assert!(output.contains("PLAYER MARKETPLACE"));
        assert!(output.contains("Your gold:"));
    }

    #[test]
    fn test_render_trade_create() {
        let state = create_test_state();
        let output = render_trade_create(&state);
        assert!(output.contains("CREATE TRADE LISTING"));
        assert!(output.contains("Select an item"));
    }

    #[test]
    fn test_render_party() {
        let state = create_test_state();
        let output = render_party(&state);
        assert!(output.contains("PARTY MANAGEMENT"));
        assert!(output.contains("Create party"));
        assert!(output.contains("Invite player"));
    }

    #[test]
    fn test_render_leaderboard() {
        let entries = vec![
            ("Hero1".to_string(), 10, 50000),
            ("Hero2".to_string(), 8, 30000),
        ];
        let output = render_leaderboard(&entries);
        assert!(output.contains("HALL OF LEGENDS"));
        assert!(output.contains("Hero1"));
        assert!(output.contains("Hero2"));
        assert!(output.contains("50000"));
    }

    #[test]
    fn test_render_dead() {
        let state = create_test_state();
        let output = render_dead(&state);
        assert!(output.contains("YOU DIED"));
        assert!(output.contains("spirit"));
        assert!(output.contains("Resurrect"));
    }

    #[test]
    fn test_render_confirm_quit() {
        let output = render_confirm_quit();
        assert!(output.contains("SAVE & QUIT"));
        assert!(output.contains("Are you sure"));
    }
}
