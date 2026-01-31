//! ANSI rendering for Depths of Diablo
//!
//! Dark dungeon visual identity with gothic color palette.

use crate::terminal::{AnsiWriter, Color};

use super::data::{get_skill, CharacterClass, CLASSES};
use super::dungeon::{Dungeon, DungeonTheme, Tile};
use super::items::ItemRarity;
use super::screen::{DiabloFlow, GameScreen};
use super::state::{Character, GameState};

// Dark gothic color palette
const COLOR_BLOOD: Color = Color::Red;
const COLOR_BONE: Color = Color::LightGray;
const COLOR_SHADOW: Color = Color::DarkGray;
const COLOR_GOLD: Color = Color::Yellow;
const COLOR_FIRE: Color = Color::LightRed;
const COLOR_ICE: Color = Color::LightCyan;
const COLOR_HOLY: Color = Color::White;
const COLOR_CORRUPT: Color = Color::Magenta;

/// Render the game header
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(COLOR_FIRE);
    w.bold();
    w.writeln("");
    w.writeln("  ██████╗ ███████╗██████╗ ████████╗██╗  ██╗███████╗");
    w.writeln("  ██╔══██╗██╔════╝██╔══██╗╚══██╔══╝██║  ██║██╔════╝");
    w.writeln("  ██║  ██║█████╗  ██████╔╝   ██║   ███████║███████╗");
    w.writeln("  ██║  ██║██╔══╝  ██╔═══╝    ██║   ██╔══██║╚════██║");
    w.writeln("  ██████╔╝███████╗██║        ██║   ██║  ██║███████║");
    w.writeln("  ╚═════╝ ╚══════╝╚═╝        ╚═╝   ╚═╝  ╚═╝╚══════╝");
    w.writeln("              DEPTHS OF DIABLO");
    w.set_fg(COLOR_BLOOD);
    w.writeln("         ~ Descend into Darkness ~");
    w.reset_color();
}

/// Render small header for dungeon view
fn render_small_header(w: &mut AnsiWriter, floor: u32, theme: DungeonTheme) {
    w.clear_screen();
    w.set_fg(COLOR_FIRE);
    w.bold();
    w.write_str("  DEPTHS OF DIABLO");
    w.reset_color();
    w.set_fg(COLOR_SHADOW);
    w.writeln(&format!(" - {} Floor {}", theme.name(), floor));
}

/// Render character status bar
fn render_status_bar(w: &mut AnsiWriter, char: &Character) {
    w.set_fg(COLOR_SHADOW);
    w.writeln(&"\u{2500}".repeat(80));

    // Name and class
    w.set_fg(COLOR_HOLY);
    w.write_str(&format!(" {} ", char.name));
    w.set_fg(COLOR_SHADOW);
    w.write_str(&format!("Lv{} {} | ", char.level, char.class.name()));

    // Health
    let hp_color = if char.health > char.max_health / 2 {
        Color::LightGreen
    } else if char.health > char.max_health / 4 {
        COLOR_GOLD
    } else {
        COLOR_BLOOD
    };
    w.set_fg(hp_color);
    w.write_str(&format!("HP:{}/{}", char.health, char.max_health));

    w.set_fg(COLOR_SHADOW);
    w.write_str(" | ");

    // Mana
    w.set_fg(COLOR_ICE);
    w.write_str(&format!("MP:{}/{}", char.mana, char.max_mana));

    w.set_fg(COLOR_SHADOW);
    w.write_str(" | ");

    // Gold
    w.set_fg(COLOR_GOLD);
    w.write_str(&format!("Gold:{}", char.gold));

    w.set_fg(COLOR_SHADOW);
    w.write_str(" | ");

    // Potions
    w.set_fg(COLOR_BLOOD);
    w.write_str(&format!("HP:{}", char.health_potions));
    w.set_fg(COLOR_SHADOW);
    w.write_str("/");
    w.set_fg(COLOR_ICE);
    w.write_str(&format!("MP:{}", char.mana_potions));

    w.writeln("");

    // Active skill
    if let Some(skill) = char.active_skill() {
        if let Some(def) = get_skill(&skill.key) {
            w.set_fg(COLOR_CORRUPT);
            w.write_str(&format!(" Skill: {} ", def.name));
            w.set_fg(COLOR_SHADOW);
            w.write_str(&format!("(Cost: {} MP)", def.mana_cost));
        }
    }

    w.writeln("");
    w.set_fg(COLOR_SHADOW);
    w.writeln(&"\u{2500}".repeat(80));
}

/// Render the dungeon map
fn render_dungeon_map(w: &mut AnsiWriter, dungeon: &Dungeon, char: &Character, view_radius: usize) {
    let px = char.x as i32;
    let py = char.y as i32;
    let vr = view_radius as i32;

    // Calculate view bounds
    let start_x = (px - vr).max(0) as usize;
    let end_x = ((px + vr + 1) as usize).min(dungeon.width);
    let start_y = (py - vr).max(0) as usize;
    let end_y = ((py + vr + 1) as usize).min(dungeon.height);

    for y in start_y..end_y {
        w.write_str("  ");
        for x in start_x..end_x {
            // Is this the player?
            if x == char.x && y == char.y {
                w.set_fg(COLOR_HOLY);
                w.bold();
                w.write_str("@");
                w.reset_color();
                continue;
            }

            // Is there a monster?
            if let Some(monster) = dungeon.get_monster_at(x, y) {
                if dungeon.is_explored(x, y) {
                    let color = if monster.monster_type.is_boss() {
                        COLOR_CORRUPT
                    } else {
                        COLOR_BLOOD
                    };
                    w.set_fg(color);
                    w.write_str(&monster.monster_type.ascii_char().to_string());
                    w.reset_color();
                    continue;
                }
            }

            // Is there an item?
            let items = dungeon.get_items_at(x, y);
            if !items.is_empty() && dungeon.is_explored(x, y) {
                let rarity = items.first().map(|i| i.item.rarity).unwrap_or(ItemRarity::Common);
                let color = match rarity {
                    ItemRarity::Common => COLOR_BONE,
                    ItemRarity::Magic => Color::LightBlue,
                    ItemRarity::Rare => COLOR_GOLD,
                    ItemRarity::Unique => COLOR_CORRUPT,
                };
                w.set_fg(color);
                w.write_str("!");
                w.reset_color();
                continue;
            }

            // Explored tile
            if dungeon.is_explored(x, y) {
                if let Some(tile) = dungeon.get_tile(x, y) {
                    let (ch, color) = match tile {
                        Tile::Wall => (dungeon.theme.wall_char(), COLOR_SHADOW),
                        Tile::Floor => (dungeon.theme.floor_char(), Color::DarkGray),
                        Tile::Door => ('+', Color::Brown),
                        Tile::StairsUp => ('<', COLOR_GOLD),
                        Tile::StairsDown => ('>', COLOR_GOLD),
                        Tile::Chest => ('$', COLOR_GOLD),
                        Tile::Shrine => ('*', COLOR_ICE),
                        Tile::Trap => ('^', COLOR_FIRE),
                    };
                    w.set_fg(color);
                    w.write_str(&ch.to_string());
                    w.reset_color();
                }
            } else {
                // Unexplored - dark
                w.set_fg(Color::Black);
                w.write_str(" ");
            }
        }
        w.writeln("");
    }
}

/// Render messages
fn render_messages(w: &mut AnsiWriter, state: &GameState) {
    if state.messages.is_empty() {
        return;
    }

    w.set_fg(COLOR_SHADOW);
    w.writeln(&"\u{2500}".repeat(80));

    for msg in &state.messages {
        w.set_fg(COLOR_GOLD);
        w.writeln(&format!("  > {}", msg));
    }
}

/// Render intro screen
pub fn render_intro(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_BONE);
    w.writeln("  The darkness beneath Tristram stirs once more.");
    w.writeln("  Demons have reclaimed the cursed cathedral.");
    w.writeln("  You are called to descend into the depths...");
    w.writeln("");
    w.set_fg(COLOR_BLOOD);
    w.writeln("  ...and face the Lord of Terror himself.");
    w.writeln("");
    w.writeln("");
    w.set_fg(COLOR_SHADOW);
    w.writeln("  * Procedural dungeons from daily seed");
    w.writeln("  * 20 floors of increasing difficulty");
    w.writeln("  * Real-time combat with skills");
    w.writeln("  * Permadeath with meta-progression");
    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.writeln(&format!("  Soul Essence: {}", state.meta.soul_essence));
    w.writeln(&format!("  Highest Floor: {}", state.meta.highest_floor_ever));
    w.reset_color();
    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render main menu
pub fn render_main_menu(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  MAIN MENU");
    w.reset_color();
    w.writeln("");

    w.set_fg(COLOR_ICE);
    w.write_str("  [N] ");
    w.set_fg(COLOR_BONE);
    w.writeln("New Solo Game");

    w.set_fg(COLOR_ICE);
    w.write_str("  [J] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Join Public Game");

    w.set_fg(COLOR_ICE);
    w.write_str("  [P] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Create Private Game");

    w.set_fg(COLOR_ICE);
    w.write_str("  [L] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Leaderboard");

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [Q] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Quit to BBS");

    w.writeln("");
    w.set_fg(COLOR_SHADOW);
    w.writeln(&format!("  Soul Essence: {} | Total Runs: {} | Victories: {}",
        state.meta.soul_essence,
        state.meta.total_runs,
        state.meta.successful_runs
    ));

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render class selection
pub fn render_class_select(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  SELECT YOUR CLASS");
    w.reset_color();
    w.writeln("");

    for (i, class) in CLASSES.iter().enumerate() {
        let unlocked = state.meta.unlocked_classes.contains(class);

        w.set_fg(COLOR_ICE);
        w.write_str(&format!("  [{}] ", i + 1));

        if unlocked {
            w.set_fg(COLOR_HOLY);
            w.bold();
            w.writeln(class.name());
            w.reset_color();

            w.set_fg(COLOR_SHADOW);
            w.writeln(&format!("      {}", class.description()));

            let stats = class.base_stats();
            w.set_fg(COLOR_BONE);
            w.writeln(&format!("      HP:{} MP:{} STR:{} DEX:{} INT:{}",
                stats.health, stats.mana, stats.strength, stats.dexterity, stats.intelligence
            ));
        } else {
            w.set_fg(COLOR_SHADOW);
            w.write_str(class.name());
            w.set_fg(COLOR_BLOOD);
            w.writeln(" [LOCKED - 200 Soul Essence]");
        }
        w.writeln("");
    }

    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render town
pub fn render_town(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    if let Some(ref char) = state.character {
        render_status_bar(&mut w, char);
    }

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  TRISTRAM");
    w.reset_color();
    w.set_fg(COLOR_SHADOW);
    w.writeln("  The last bastion before the depths...");
    w.writeln("");

    let floor = state.run.as_ref().map(|r| r.current_floor).unwrap_or(1);
    w.set_fg(COLOR_BONE);
    w.writeln(&format!("  Current depth: Floor {}", floor));
    w.writeln("");

    w.set_fg(COLOR_ICE);
    w.write_str("  [E] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Enter Dungeon");

    w.set_fg(COLOR_ICE);
    w.write_str("  [I] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Inventory");

    w.set_fg(COLOR_ICE);
    w.write_str("  [S] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Skills");

    w.set_fg(COLOR_ICE);
    w.write_str("  [C] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Character Stats");

    w.set_fg(COLOR_ICE);
    w.write_str("  [H] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Shop");

    w.set_fg(COLOR_ICE);
    w.write_str("  [T] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Stash");

    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Blacksmith (Upgrades)");

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [A] ");
    w.set_fg(COLOR_BLOOD);
    w.writeln("Abandon Run");

    w.set_fg(COLOR_ICE);
    w.write_str("  [Q] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Save & Quit");

    render_messages(&mut w, state);

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render dungeon
pub fn render_dungeon(state: &GameState) -> String {
    let mut w = AnsiWriter::new();

    let (char, dungeon) = match (&state.character, &state.dungeon) {
        (Some(c), Some(d)) => (c, d),
        _ => {
            w.writeln("Error: No dungeon loaded");
            return w.flush();
        }
    };

    let floor = state.run.as_ref().map(|r| r.current_floor).unwrap_or(1);
    render_small_header(&mut w, floor, dungeon.theme);
    render_status_bar(&mut w, char);

    // Render map
    render_dungeon_map(&mut w, dungeon, char, 10);

    // Render messages
    render_messages(&mut w, state);

    // Controls hint
    w.set_fg(COLOR_SHADOW);
    w.writeln("  WASD:Move  F:Skill  H:HealPot  M:ManaPot  G:Get  >/<:Stairs  I:Inv  T:Town");

    w.reset_color();

    w.flush()
}

/// Render inventory
pub fn render_inventory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  INVENTORY");
    w.reset_color();
    w.writeln("");

    if let Some(ref char) = state.character {
        // Equipment slots
        w.set_fg(COLOR_ICE);
        w.writeln("  EQUIPPED:");
        w.set_fg(COLOR_SHADOW);
        w.writeln(&"\u{2500}".repeat(40));

        for (slot, item) in &char.equipment {
            let color = match item.rarity {
                ItemRarity::Common => COLOR_BONE,
                ItemRarity::Magic => Color::LightBlue,
                ItemRarity::Rare => COLOR_GOLD,
                ItemRarity::Unique => COLOR_CORRUPT,
            };
            w.set_fg(COLOR_SHADOW);
            w.write_str(&format!("  {}: ", slot));
            w.set_fg(color);
            w.writeln(&item.name);
        }

        w.writeln("");
        w.set_fg(COLOR_ICE);
        w.writeln("  BACKPACK:");
        w.set_fg(COLOR_SHADOW);
        w.writeln(&"\u{2500}".repeat(40));

        if char.inventory.is_empty() {
            w.set_fg(COLOR_SHADOW);
            w.writeln("  (empty)");
        } else {
            for (i, item) in char.inventory.iter().enumerate() {
                let color = match item.rarity {
                    ItemRarity::Common => COLOR_BONE,
                    ItemRarity::Magic => Color::LightBlue,
                    ItemRarity::Rare => COLOR_GOLD,
                    ItemRarity::Unique => COLOR_CORRUPT,
                };
                w.set_fg(COLOR_ICE);
                w.write_str(&format!("  [{}] ", i + 1));
                w.set_fg(color);
                w.write_str(&item.name);

                // Show stats
                w.set_fg(COLOR_SHADOW);
                if item.total_damage() > 0 {
                    w.write_str(&format!(" (Dmg:{})", item.total_damage()));
                }
                if item.total_armor() > 0 {
                    w.write_str(&format!(" (Arm:{})", item.total_armor()));
                }
                w.writeln("");
            }
        }
    }

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [#] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Equip item by number");

    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render skills screen
pub fn render_skills(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  SKILLS");
    w.reset_color();
    w.writeln("");

    if let Some(ref char) = state.character {
        for (i, skill) in char.skills.iter().enumerate() {
            if let Some(def) = get_skill(&skill.key) {
                let is_active = i == char.active_skill_index;

                if is_active {
                    w.set_fg(COLOR_GOLD);
                    w.bold();
                    w.write_str("  >> ");
                } else {
                    w.set_fg(COLOR_ICE);
                    w.write_str("     ");
                }

                w.write_str(def.name);
                w.reset_color();

                w.set_fg(COLOR_SHADOW);
                w.writeln(&format!(" - {} MP, {}% dmg",
                    def.mana_cost, def.damage_multiplier
                ));

                w.set_fg(COLOR_BONE);
                w.writeln(&format!("       {}", def.description));
                w.writeln("");
            }
        }
    }

    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render character stats
pub fn render_stats(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  CHARACTER STATS");
    w.reset_color();
    w.writeln("");

    if let Some(ref char) = state.character {
        w.set_fg(COLOR_HOLY);
        w.writeln(&format!("  {} - Level {} {}", char.name, char.level, char.class.name()));
        w.writeln("");

        w.set_fg(COLOR_BONE);
        w.writeln(&format!("  Experience: {}/{}", char.experience, char.exp_to_next));
        w.writeln("");

        w.set_fg(COLOR_ICE);
        w.writeln("  ATTRIBUTES:");
        w.set_fg(COLOR_BONE);
        w.writeln(&format!("    Strength:     {} (+{})", char.strength, char.total_strength() - char.strength));
        w.writeln(&format!("    Dexterity:    {} (+{})", char.dexterity, char.total_dexterity() - char.dexterity));
        w.writeln(&format!("    Intelligence: {} (+{})", char.intelligence, char.total_intelligence() - char.intelligence));
        w.writeln(&format!("    Vitality:     {} (+{})", char.vitality, char.total_vitality() - char.vitality));
    }

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render shop
pub fn render_shop(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  GRISWOLD'S SHOP");
    w.reset_color();
    w.set_fg(COLOR_SHADOW);
    w.writeln("  \"What can I do for ya?\"");
    w.writeln("");

    if let Some(ref char) = state.character {
        w.set_fg(COLOR_GOLD);
        w.writeln(&format!("  Your Gold: {}", char.gold));
        w.writeln("");
    }

    w.set_fg(COLOR_ICE);
    w.write_str("  [1] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Health Potion - 50 gold");

    w.set_fg(COLOR_ICE);
    w.write_str("  [2] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Mana Potion - 50 gold");

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    render_messages(&mut w, state);

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render stash
pub fn render_stash(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  STASH");
    w.reset_color();
    w.set_fg(COLOR_SHADOW);
    w.writeln("  Items stored between runs");
    w.writeln("");

    if state.meta.stash.is_empty() {
        w.set_fg(COLOR_SHADOW);
        w.writeln("  (empty)");
    } else {
        for (i, item) in state.meta.stash.iter().enumerate() {
            let color = match item.rarity {
                ItemRarity::Common => COLOR_BONE,
                ItemRarity::Magic => Color::LightBlue,
                ItemRarity::Rare => COLOR_GOLD,
                ItemRarity::Unique => COLOR_CORRUPT,
            };
            w.set_fg(COLOR_ICE);
            w.write_str(&format!("  [{}] ", i + 1));
            w.set_fg(color);
            w.writeln(&item.name);
        }
    }

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render blacksmith (upgrades)
pub fn render_blacksmith(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  BLACKSMITH - META UPGRADES");
    w.reset_color();
    w.writeln("");

    w.set_fg(COLOR_GOLD);
    w.writeln(&format!("  Soul Essence: {}", state.meta.soul_essence));
    w.writeln("");

    let bs_level = state.meta.get_upgrade_level("blacksmith");
    w.set_fg(COLOR_ICE);
    w.write_str("  [1] ");
    w.set_fg(COLOR_BONE);
    w.writeln(&format!("Upgrade Blacksmith (Lv{}) - 100 Soul Essence", bs_level));
    w.set_fg(COLOR_SHADOW);
    w.writeln("      +1 starting health potion per level");
    w.writeln("");

    let rogue_unlocked = state.meta.unlocked_classes.contains(&CharacterClass::Rogue);
    w.set_fg(COLOR_ICE);
    w.write_str("  [2] ");
    if rogue_unlocked {
        w.set_fg(Color::LightGreen);
        w.writeln("Rogue Class [UNLOCKED]");
    } else {
        w.set_fg(COLOR_BONE);
        w.writeln("Unlock Rogue Class - 200 Soul Essence");
    }

    let sorc_unlocked = state.meta.unlocked_classes.contains(&CharacterClass::Sorcerer);
    w.set_fg(COLOR_ICE);
    w.write_str("  [3] ");
    if sorc_unlocked {
        w.set_fg(Color::LightGreen);
        w.writeln("Sorcerer Class [UNLOCKED]");
    } else {
        w.set_fg(COLOR_BONE);
        w.writeln("Unlock Sorcerer Class - 200 Soul Essence");
    }

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  [B] ");
    w.set_fg(COLOR_BONE);
    w.writeln("Back");

    render_messages(&mut w, state);

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");

    w.flush()
}

/// Render game over
pub fn render_game_over(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(COLOR_BLOOD);
    w.bold();
    w.writeln("");
    w.writeln("  ██████╗ ███████╗ █████╗ ████████╗██╗  ██╗");
    w.writeln("  ██╔══██╗██╔════╝██╔══██╗╚══██╔══╝██║  ██║");
    w.writeln("  ██║  ██║█████╗  ███████║   ██║   ███████║");
    w.writeln("  ██║  ██║██╔══╝  ██╔══██║   ██║   ██╔══██║");
    w.writeln("  ██████╔╝███████╗██║  ██║   ██║   ██║  ██║");
    w.writeln("  ╚═════╝ ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝");
    w.reset_color();

    w.writeln("");
    w.set_fg(COLOR_BONE);
    w.writeln("  The depths have claimed another soul...");
    w.writeln("");

    w.set_fg(COLOR_GOLD);
    w.writeln(&format!("  Highest Floor: {}", state.meta.highest_floor_ever));
    w.writeln(&format!("  Soul Essence Earned: {}", state.meta.soul_essence));

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render victory
pub fn render_victory(state: &GameState) -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("");
    w.writeln("  ██╗   ██╗██╗ ██████╗████████╗ ██████╗ ██████╗ ██╗   ██╗");
    w.writeln("  ██║   ██║██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝");
    w.writeln("  ██║   ██║██║██║        ██║   ██║   ██║██████╔╝ ╚████╔╝ ");
    w.writeln("  ╚██╗ ██╔╝██║██║        ██║   ██║   ██║██╔══██╗  ╚██╔╝  ");
    w.writeln("   ╚████╔╝ ██║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║   ");
    w.writeln("    ╚═══╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ");
    w.reset_color();

    w.writeln("");
    w.set_fg(COLOR_HOLY);
    w.writeln("  DIABLO HAS BEEN DEFEATED!");
    w.writeln("");
    w.set_fg(COLOR_BONE);
    w.writeln("  The Lord of Terror is vanquished. Sanctuary is safe...");
    w.writeln("  ...for now.");
    w.writeln("");

    w.set_fg(COLOR_GOLD);
    w.writeln(&format!("  Runs Completed: {}", state.meta.successful_runs));
    w.writeln(&format!("  Total Soul Essence: {}", state.meta.soul_essence));

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Render confirm quit
pub fn render_confirm_quit() -> String {
    let mut w = AnsiWriter::new();
    w.clear_screen();

    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("");
    w.writeln("  SAVE & QUIT");
    w.reset_color();
    w.writeln("");
    w.set_fg(COLOR_BONE);
    w.writeln("  Your progress will be saved.");
    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.write_str("  Are you sure? [Y/N] ");
    w.reset_color();

    w.flush()
}

/// Render leaderboard
pub fn render_leaderboard(_state: &GameState, entries: &[(String, u32, i64)]) -> String {
    let mut w = AnsiWriter::new();
    render_header(&mut w);

    w.writeln("");
    w.set_fg(COLOR_GOLD);
    w.bold();
    w.writeln("  HALL OF HEROES");
    w.reset_color();
    w.writeln("");

    w.set_fg(COLOR_SHADOW);
    w.writeln(&format!("  {:<4} {:<16} {:>8} {:>12}",
        "Rank", "Hero", "Floor", "Soul Essence"
    ));
    w.writeln(&"\u{2500}".repeat(50));

    if entries.is_empty() {
        w.set_fg(COLOR_SHADOW);
        w.writeln("  No heroes have conquered the depths... yet.");
    } else {
        for (i, (handle, floor, essence)) in entries.iter().enumerate() {
            let color = match i {
                0 => COLOR_GOLD,
                1 => COLOR_BONE,
                2 => Color::Brown,
                _ => COLOR_SHADOW,
            };
            w.set_fg(color);
            w.writeln(&format!("  {:<4} {:<16} {:>8} {:>12}",
                i + 1, handle, floor, essence
            ));
        }
    }

    w.writeln("");
    w.set_fg(COLOR_ICE);
    w.writeln("  Press any key to continue...");
    w.reset_color();

    w.flush()
}

/// Main render dispatcher
pub fn render_screen(flow: &DiabloFlow) -> String {
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::Intro => render_intro(state),
        GameScreen::MainMenu => render_main_menu(state),
        GameScreen::Lobby => render_main_menu(state), // Use main menu for now
        GameScreen::ClassSelect => render_class_select(state),
        GameScreen::Town => render_town(state),
        GameScreen::Dungeon => render_dungeon(state),
        GameScreen::Inventory => render_inventory(state),
        GameScreen::Skills => render_skills(state),
        GameScreen::Stats => render_stats(state),
        GameScreen::Shop => render_shop(state),
        GameScreen::Stash => render_stash(state),
        GameScreen::Blacksmith => render_blacksmith(state),
        GameScreen::Leaderboard => render_leaderboard(state, &[]),
        GameScreen::GameOver => render_game_over(state),
        GameScreen::Victory => render_victory(state),
        GameScreen::ConfirmQuit => render_confirm_quit(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_intro() {
        let state = GameState::new(1, "Test");
        let output = render_intro(&state);
        assert!(output.contains("DEPTHS"));
        assert!(output.contains("DIABLO"));
    }

    #[test]
    fn test_render_main_menu() {
        let state = GameState::new(1, "Test");
        let output = render_main_menu(&state);
        assert!(output.contains("MAIN MENU"));
        assert!(output.contains("[N]"));
    }

    #[test]
    fn test_render_class_select() {
        let state = GameState::new(1, "Test");
        let output = render_class_select(&state);
        assert!(output.contains("Warrior"));
        assert!(output.contains("LOCKED")); // Rogue/Sorc locked by default
    }

    #[test]
    fn test_render_screen_dispatch() {
        let flow = DiabloFlow::new(1, "Test");
        let output = render_screen(&flow);
        // Should render intro by default
        assert!(output.contains("DEPTHS"));
    }
}
