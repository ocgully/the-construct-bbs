//! ANSI rendering for Last Dream
//! Crystal-themed JRPG aesthetic with deep blues, purples, and golds

use crate::terminal::{AnsiWriter, Color};
use super::combat::CombatState;
use super::data::{ITEMS, EQUIPMENT, get_equipment};
use super::screen::{GameScreen, LastDreamFlow, CreationStep, ShopType};
use super::state::GameState;
use super::world::WorldMap;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format number with commas
pub fn format_number(num: u32) -> String {
    let s = num.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Render game header
fn render_header(w: &mut AnsiWriter) {
    w.clear_screen();
    w.set_fg(Color::LightBlue);
    w.bold();
    w.writeln("");
    w.writeln("  ╔═══════════════════════════════════════════════════════════════════════╗");
    w.writeln("  ║     _                  _     ____                                     ║");
    w.writeln("  ║    | |    __ _ ___  __| |_  |  _ \\ _ __ ___  __ _ _ __ ___           ║");
    w.writeln("  ║    | |   / _` / __|/ _` __| | | | | '__/ _ \\/ _` | '_ ` _ \\          ║");
    w.writeln("  ║    | |__| (_| \\__ \\ (_| |_  | |_| | | |  __/ (_| | | | | | |         ║");
    w.writeln("  ║    |_____\\__,_|___/\\__,\\__| |____/|_|  \\___|\\__,_|_| |_| |_|         ║");
    w.writeln("  ║                                                                       ║");
    w.writeln("  ╚═══════════════════════════════════════════════════════════════════════╝");
    w.reset_color();
}

/// Render compact party status bar
fn render_party_bar(w: &mut AnsiWriter, state: &GameState) {
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(74)));

    for member in &state.party.members {
        let hp_color = if member.hp * 100 / member.hp_max > 70 {
            Color::LightGreen
        } else if member.hp * 100 / member.hp_max > 30 {
            Color::Yellow
        } else if member.hp > 0 {
            Color::LightRed
        } else {
            Color::Red
        };

        w.set_fg(Color::LightCyan);
        w.write_str(&format!("  {:<8} ", member.name));
        w.set_fg(Color::White);
        w.write_str(&format!("L{:<2} ", member.level));
        w.set_fg(hp_color);
        w.write_str(&format!("HP:{:>4}/{:<4} ", member.hp, member.hp_max));
        w.set_fg(Color::LightBlue);
        w.writeln(&format!("MP:{:>3}/{:<3}", member.mp, member.mp_max));
    }

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Gil: {}    Time: {}",
        format_number(state.gold),
        state.formatted_play_time()
    ));

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(74)));
    w.reset_color();
}

// ============================================================================
// PUBLIC RENDER FUNCTION
// ============================================================================

/// Main render function - dispatches to screen-specific renderers
pub fn render_screen(flow: &LastDreamFlow) -> String {
    let mut w = AnsiWriter::new();
    let state = flow.game_state();

    match flow.current_screen() {
        GameScreen::PartyCreation { step } => render_creation(&mut w, step, state),
        GameScreen::Intro => render_intro(&mut w, state),
        GameScreen::WorldMap => render_world_map(&mut w, state, &flow.world_map),
        GameScreen::Town { location } => render_town(&mut w, state, location),
        GameScreen::Shop { location, shop_type } => render_shop(&mut w, state, location, shop_type),
        GameScreen::Inn { location } => render_inn(&mut w, state, location),
        GameScreen::SavePoint { location } => render_save_point(&mut w, location),
        GameScreen::Dungeon { location, floor } => render_dungeon(&mut w, state, location, *floor),
        GameScreen::Combat { combat } => render_combat(&mut w, state, combat),
        GameScreen::CombatTarget { combat, .. } => render_combat_target(&mut w, state, combat),
        GameScreen::PartyMenu => render_party_menu(&mut w, state),
        GameScreen::CharacterDetail { index } => render_character_detail(&mut w, state, *index),
        GameScreen::EquipmentMenu { char_index, slot_index } => {
            render_equipment_menu(&mut w, state, *char_index, *slot_index)
        }
        GameScreen::Inventory => render_inventory(&mut w, state),
        GameScreen::UseItem { item_index } => render_use_item(&mut w, state, *item_index),
        GameScreen::MagicMenu { char_index } => render_magic_menu(&mut w, state, *char_index),
        GameScreen::Status => render_status(&mut w, state),
        GameScreen::StoryEvent { event_key } => render_story_event(&mut w, event_key),
        GameScreen::Victory { exp, gold } => render_victory(&mut w, state, *exp, *gold),
        GameScreen::GameOver => render_game_over(&mut w),
        GameScreen::ConfirmQuit => render_confirm_quit(&mut w),
        GameScreen::Ending { phase } => render_ending(&mut w, state, *phase),
    }

    w.flush()
}

// ============================================================================
// SCREEN RENDERERS
// ============================================================================

fn render_creation(w: &mut AnsiWriter, step: &CreationStep, state: &GameState) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  CREATE YOUR PARTY");
    w.reset_color();
    w.writeln("");

    match step {
        CreationStep::SelectClass { member_num } => {
            w.set_fg(Color::White);
            w.writeln(&format!("  Choose class for party member {} of 4:", member_num + 1));
            w.writeln("");

            let classes = [
                ("1", "Warrior", "High HP, STR. Heavy weapons/armor."),
                ("2", "Thief", "High Speed, luck. Can steal."),
                ("3", "Mage", "Offensive spells. High INT/MP."),
                ("4", "Cleric", "Healing/support magic."),
                ("5", "Monk", "Unarmed specialist. High damage."),
                ("6", "Knight", "Balanced. Some white magic."),
            ];

            for (key, name, desc) in classes {
                w.set_fg(Color::LightCyan);
                w.write_str(&format!("  [{}] ", key));
                w.set_fg(Color::White);
                w.write_str(&format!("{:<10} ", name));
                w.set_fg(Color::DarkGray);
                w.writeln(desc);
            }

            // Show current party
            if !state.party.members.is_empty() {
                w.writeln("");
                w.set_fg(Color::LightMagenta);
                w.writeln("  Current Party:");
                for member in &state.party.members {
                    w.set_fg(Color::White);
                    w.writeln(&format!("    {} ({})", member.name, member.class.name()));
                }
            }
        }
        CreationStep::EnterName { member_num, class } => {
            w.set_fg(Color::White);
            w.writeln(&format!("  {} selected for member {}.", class.name(), member_num + 1));
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.write_str("  Enter name (or press Enter for default): ");
        }
        CreationStep::ConfirmParty => {
            w.set_fg(Color::LightGreen);
            w.writeln("  Your Party:");
            w.writeln("");
            for member in &state.party.members {
                w.set_fg(Color::White);
                w.writeln(&format!("    {} - {}", member.name, member.class.name()));
            }
            w.writeln("");
            w.set_fg(Color::LightCyan);
            w.writeln("  [Y] Begin Adventure    [N] Start Over");
        }
    }

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_intro(w: &mut AnsiWriter, state: &GameState) {
    w.clear_screen();

    w.set_fg(Color::LightBlue);
    w.writeln("");
    w.writeln("  The world is balanced by Four Crystals:");
    w.writeln("  Earth, Fire, Water, and Wind.");
    w.writeln("");
    w.set_fg(Color::Yellow);
    w.writeln("  But darkness spreads from the Void.");
    w.writeln("  The Crystals fade. The balance breaks.");
    w.writeln("");
    w.set_fg(Color::White);
    w.writeln(&format!("  {} and companions arise as Warriors of Light.",
        state.party.leader().map(|c| c.name.as_str()).unwrap_or("Heroes")));
    w.writeln("");
    w.writeln("  Your quest: Restore the Four Crystals.");
    w.writeln("  Save the world from the consuming Void.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to begin...");
    w.reset_color();
}

fn render_world_map(w: &mut AnsiWriter, state: &GameState, world_map: &WorldMap) {
    w.clear_screen();
    render_party_bar(w, state);

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    // Render map viewport (15x9 around player)
    let view_w = 15;
    let view_h = 9;
    let px = state.world_position.x;
    let py = state.world_position.y;
    let start_x = px.saturating_sub(view_w / 2);
    let start_y = py.saturating_sub(view_h / 2);

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(view_w + 2)));

    for dy in 0..view_h {
        w.set_fg(Color::DarkGray);
        w.write_str("  \u{2502}");

        for dx in 0..view_w {
            let x = start_x + dx;
            let y = start_y + dy;

            if x == px && y == py {
                w.set_fg(Color::LightRed);
                w.write_str("@");
            } else if x < world_map.width && y < world_map.height {
                let tile = world_map.get(x, y);
                let color = match tile {
                    super::world::Tile::Grass => Color::Green,
                    super::world::Tile::Forest => Color::LightGreen,
                    super::world::Tile::Mountain => Color::Brown,
                    super::world::Tile::Water => Color::LightBlue,
                    super::world::Tile::Desert => Color::Yellow,
                    super::world::Tile::Snow => Color::White,
                    super::world::Tile::Town => Color::LightMagenta,
                    super::world::Tile::Castle => Color::Yellow,
                    super::world::Tile::Dungeon => Color::Red,
                    super::world::Tile::Dock => Color::Brown,
                    _ => Color::LightGray,
                };
                w.set_fg(color);
                w.write_str(&tile.char().to_string());
            } else {
                w.write_str(" ");
            }
        }

        w.set_fg(Color::DarkGray);
        w.writeln("\u{2502}");
    }

    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(view_w + 2)));
    w.reset_color();

    // Location info
    if let Some(loc) = world_map.location_at(px, py) {
        w.set_fg(Color::LightMagenta);
        w.writeln(&format!("  Location: {}", loc.name));
    }

    // Transport info
    w.set_fg(Color::LightGray);
    w.writeln(&format!("  Transport: {}", state.transport.name()));

    // Controls
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [WASD] Move   [E] Enter   [P] Party   [Q] Quit");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_town(w: &mut AnsiWriter, state: &GameState, location: &str) {
    render_header(w);
    render_party_bar(w, state);

    let loc = super::world::get_location(location);

    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.bold();
    if let Some(l) = loc {
        w.writeln(&format!("  {}", l.name.to_uppercase()));
        w.reset_color();
        w.set_fg(Color::LightGray);
        w.writeln(&format!("  {}", l.description));
    }
    w.reset_color();
    w.writeln("");

    // Show message if any
    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.writeln("");
        w.reset_color();
    }

    // Town menu
    if loc.map(|l| l.has_shop).unwrap_or(false) {
        w.set_fg(Color::LightCyan);
        w.write_str("  [I] ");
        w.set_fg(Color::White);
        w.writeln("Item Shop");

        w.set_fg(Color::LightCyan);
        w.write_str("  [W] ");
        w.set_fg(Color::White);
        w.writeln("Weapon Shop");

        w.set_fg(Color::LightCyan);
        w.write_str("  [A] ");
        w.set_fg(Color::White);
        w.writeln("Armor Shop");
    }

    if loc.map(|l| l.has_inn).unwrap_or(false) {
        w.set_fg(Color::LightCyan);
        w.write_str("  [R] ");
        w.set_fg(Color::White);
        w.writeln("Inn (Rest)");
    }

    if loc.map(|l| l.has_save_point).unwrap_or(false) {
        w.set_fg(Color::LightCyan);
        w.write_str("  [S] ");
        w.set_fg(Color::White);
        w.writeln("Save Point");
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.write_str("  [P] ");
    w.set_fg(Color::White);
    w.writeln("Party Menu");

    w.set_fg(Color::LightCyan);
    w.write_str("  [L] ");
    w.set_fg(Color::White);
    w.writeln("Leave Town");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_shop(w: &mut AnsiWriter, state: &GameState, _location: &str, shop_type: &ShopType) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    match shop_type {
        ShopType::Items => w.writeln("  ITEM SHOP"),
        ShopType::Weapons => w.writeln("  WEAPON SHOP"),
        ShopType::Armor => w.writeln("  ARMOR SHOP"),
    }
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Your Gil: {}", format_number(state.gold)));
    w.writeln("");

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::LightGreen);
        w.writeln(&format!("  >> {} <<", msg));
        w.writeln("");
        w.reset_color();
    }

    match shop_type {
        ShopType::Items => {
            w.set_fg(Color::DarkGray);
            w.writeln("     #  Item                 Price");
            w.writeln(&format!("    {}", "\u{2500}".repeat(40)));
            w.reset_color();

            for (i, item) in ITEMS.iter().enumerate() {
                let can_afford = state.gold >= item.price;
                w.set_fg(if can_afford { Color::LightCyan } else { Color::DarkGray });
                w.write_str(&format!("    {} ", i + 1));
                w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
                w.write_str(&format!("{:<20}", item.name));
                w.set_fg(if can_afford { Color::Yellow } else { Color::DarkGray });
                w.writeln(&format!("{:>6}", item.price));
            }
        }
        ShopType::Weapons => {
            let weapons: Vec<_> = EQUIPMENT.iter()
                .filter(|e| matches!(e.slot, super::data::EquipmentSlot::Weapon))
                .filter(|e| e.price > 0)
                .collect();

            w.set_fg(Color::DarkGray);
            w.writeln("     #  Weapon               ATK     Price");
            w.writeln(&format!("    {}", "\u{2500}".repeat(45)));
            w.reset_color();

            for (i, equip) in weapons.iter().enumerate() {
                let can_afford = state.gold >= equip.price;
                w.set_fg(if can_afford { Color::LightCyan } else { Color::DarkGray });
                w.write_str(&format!("    {} ", i + 1));
                w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
                w.write_str(&format!("{:<20}", equip.name));
                w.set_fg(if can_afford { Color::LightRed } else { Color::DarkGray });
                w.write_str(&format!("{:>4}", equip.attack));
                w.set_fg(if can_afford { Color::Yellow } else { Color::DarkGray });
                w.writeln(&format!("{:>8}", equip.price));
            }
        }
        ShopType::Armor => {
            let armor: Vec<_> = EQUIPMENT.iter()
                .filter(|e| !matches!(e.slot, super::data::EquipmentSlot::Weapon))
                .filter(|e| e.price > 0)
                .collect();

            w.set_fg(Color::DarkGray);
            w.writeln("     #  Armor                DEF     Price");
            w.writeln(&format!("    {}", "\u{2500}".repeat(45)));
            w.reset_color();

            for (i, equip) in armor.iter().enumerate() {
                let can_afford = state.gold >= equip.price;
                w.set_fg(if can_afford { Color::LightCyan } else { Color::DarkGray });
                w.write_str(&format!("    {} ", i + 1));
                w.set_fg(if can_afford { Color::White } else { Color::DarkGray });
                w.write_str(&format!("{:<20}", equip.name));
                w.set_fg(if can_afford { Color::LightGreen } else { Color::DarkGray });
                w.write_str(&format!("{:>4}", equip.defense));
                w.set_fg(if can_afford { Color::Yellow } else { Color::DarkGray });
                w.writeln(&format!("{:>8}", equip.price));
            }
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Enter number to buy, or [Q] to leave.");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_inn(w: &mut AnsiWriter, state: &GameState, _location: &str) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Brown);
    w.bold();
    w.writeln("  THE INN");
    w.reset_color();
    w.writeln("");

    let cost = state.party.members.len() as u32 * 50;
    w.set_fg(Color::White);
    w.writeln(&format!("  \"Welcome, travelers! Rest costs {} Gil.\"", cost));
    w.writeln("");
    w.writeln("  A good night's rest will fully restore HP and MP.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  [Y] Rest    [N] Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_save_point(w: &mut AnsiWriter, _location: &str) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::LightBlue);
    w.bold();
    w.writeln("  CRYSTAL SAVE POINT");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightMagenta);
    w.writeln("  A glowing crystal pulses with ancient power.");
    w.writeln("  Touch it to record your progress.");
    w.writeln("");

    w.set_fg(Color::LightCyan);
    w.writeln("  [Y] Save Game    [N] Leave");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_dungeon(w: &mut AnsiWriter, state: &GameState, location: &str, floor: u8) {
    w.clear_screen();
    render_party_bar(w, state);

    let loc = super::world::get_location(location);

    w.set_fg(Color::LightRed);
    w.bold();
    if let Some(l) = loc {
        w.writeln(&format!("  {} - Floor {}", l.name.to_uppercase(), floor));
    }
    w.reset_color();

    if let Some(ref msg) = state.last_message {
        w.set_fg(Color::Yellow);
        w.writeln(&format!("  >> {} <<", msg));
        w.reset_color();
    }

    // Simple dungeon view
    w.set_fg(Color::DarkGray);
    w.writeln("");
    w.writeln("  ########################");
    w.writeln("  #....#......$.........#");
    w.writeln("  #.@..#..###..#########.#");
    w.writeln("  #....+..#.....#.......#");
    w.writeln("  ####.####.....#...>...#");
    w.writeln("  #..............########");
    w.writeln("  ########################");
    w.writeln("");
    w.reset_color();

    w.set_fg(Color::LightCyan);
    w.writeln("  [WASD] Move   [P] Party   [L] Leave");
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
    w.writeln("  BATTLE!");
    w.reset_color();
    w.writeln("");

    // Show enemies
    for (i, enemy) in combat.enemies.iter().enumerate() {
        if enemy.is_alive() {
            let hp_pct = enemy.hp * 100 / enemy.hp_max;
            let color = if hp_pct > 70 {
                Color::White
            } else if hp_pct > 30 {
                Color::Yellow
            } else {
                Color::LightRed
            };

            w.set_fg(color);
            w.writeln(&format!("  {}. {} HP: {}/{}", i + 1, enemy.name, enemy.hp, enemy.hp_max));
        } else {
            w.set_fg(Color::DarkGray);
            w.writeln(&format!("  {}. {} (defeated)", i + 1, enemy.name));
        }
    }

    w.writeln("");
    w.set_fg(Color::DarkGray);
    w.writeln(&format!("  {}", "\u{2500}".repeat(50)));

    // Party status
    for member in &state.party.members {
        let hp_color = if member.hp * 100 / member.hp_max > 50 {
            Color::LightGreen
        } else if member.hp > 0 {
            Color::Yellow
        } else {
            Color::Red
        };

        let atb_filled = member.atb_gauge / 10;
        let atb_bar: String = (0..10).map(|i| if i < atb_filled { '\u{2588}' } else { '\u{2591}' }).collect();

        w.set_fg(Color::White);
        w.write_str(&format!("  {:<10} ", member.name));
        w.set_fg(hp_color);
        w.write_str(&format!("HP:{:>4}/{:<4} ", member.hp, member.hp_max));
        w.set_fg(Color::LightBlue);
        w.write_str(&format!("MP:{:>3}/{:<3} ", member.mp, member.mp_max));
        w.set_fg(Color::LightCyan);
        w.writeln(&format!("[{}]", atb_bar));
    }

    // Combat log (last 3 entries)
    w.writeln("");
    w.set_fg(Color::LightGray);
    for msg in combat.recent_log(3) {
        w.writeln(&format!("  {}", msg));
    }

    // Actions (if it's a party member's turn)
    if combat.active_member.is_some() {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln("  What will you do?");
        w.reset_color();

        w.set_fg(Color::LightCyan);
        w.writeln("  [A] Attack   [M] Magic   [I] Item   [D] Defend   [R] Run");
    }

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_combat_target(w: &mut AnsiWriter, state: &GameState, combat: &CombatState) {
    render_combat(w, state, combat);
    // Target selection UI would be added here
}

fn render_party_menu(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);
    render_party_bar(w, state);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  PARTY MENU");
    w.reset_color();
    w.writeln("");

    for (i, member) in state.party.members.iter().enumerate() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("  [{}] ", i + 1));
        w.set_fg(Color::White);
        w.writeln(&format!("{} ({})", member.name, member.class.name()));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [I] Inventory   [S] Status   [B] Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_character_detail(w: &mut AnsiWriter, state: &GameState, index: usize) {
    render_header(w);

    if let Some(member) = state.party.members.get(index) {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln(&format!("  {} - {} Lv.{}", member.name, member.class.name(), member.level));
        w.reset_color();
        w.writeln("");

        // Stats
        w.set_fg(Color::White);
        w.writeln(&format!("  HP: {}/{}    MP: {}/{}", member.hp, member.hp_max, member.mp, member.mp_max));
        w.writeln(&format!("  EXP: {}  Next: {}", member.exp, member.exp_to_next));
        w.writeln("");
        w.writeln(&format!("  STR: {:>3}    AGI: {:>3}    INT: {:>3}", member.strength, member.agility, member.intelligence));
        w.writeln(&format!("  VIT: {:>3}    LCK: {:>3}", member.vitality, member.luck));
        w.writeln("");

        // Equipment
        w.set_fg(Color::LightCyan);
        w.writeln("  Equipment:");
        w.set_fg(Color::LightGray);

        let weapon_name = member.equipment.weapon.as_ref()
            .and_then(|k| get_equipment(k))
            .map(|e| e.name)
            .unwrap_or("(none)");
        w.writeln(&format!("    Weapon: {}", weapon_name));

        let armor_name = member.equipment.armor.as_ref()
            .and_then(|k| get_equipment(k))
            .map(|e| e.name)
            .unwrap_or("(none)");
        w.writeln(&format!("    Armor:  {}", armor_name));

        // Known spells
        if !member.spells.is_empty() {
            w.writeln("");
            w.set_fg(Color::LightMagenta);
            w.writeln("  Spells:");
            w.set_fg(Color::White);
            let spell_list = member.spells.join(", ");
            w.writeln(&format!("    {}", spell_list));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [E] Equipment   [B] Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_equipment_menu(w: &mut AnsiWriter, state: &GameState, char_index: usize, _slot_index: usize) {
    render_header(w);

    if let Some(member) = state.party.members.get(char_index) {
        w.writeln("");
        w.set_fg(Color::Yellow);
        w.bold();
        w.writeln(&format!("  {} - Equipment", member.name));
        w.reset_color();
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [B] Back");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_inventory(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  INVENTORY");
    w.reset_color();
    w.writeln("");

    if state.inventory.is_empty() {
        w.set_fg(Color::DarkGray);
        w.writeln("  (empty)");
    } else {
        for (i, item) in state.inventory.iter().enumerate() {
            w.set_fg(Color::LightCyan);
            w.write_str(&format!("  [{}] ", i + 1));
            w.set_fg(Color::White);
            let item_name = super::data::get_item(&item.key)
                .map(|i| i.name)
                .unwrap_or(&item.key);
            w.writeln(&format!("{} x{}", item_name, item.quantity));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Enter number to use, or [B] to go back.");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_use_item(w: &mut AnsiWriter, state: &GameState, _item_index: usize) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  USE ITEM ON:");
    w.reset_color();
    w.writeln("");

    for (i, member) in state.party.members.iter().enumerate() {
        w.set_fg(Color::LightCyan);
        w.write_str(&format!("  [{}] ", i + 1));
        w.set_fg(Color::White);
        w.writeln(&format!("{} HP:{}/{}", member.name, member.hp, member.hp_max));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [B] Cancel");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_magic_menu(w: &mut AnsiWriter, state: &GameState, char_index: usize) {
    render_header(w);

    if let Some(member) = state.party.members.get(char_index) {
        w.writeln("");
        w.set_fg(Color::LightMagenta);
        w.bold();
        w.writeln(&format!("  {} - Magic (MP: {}/{})", member.name, member.mp, member.mp_max));
        w.reset_color();
        w.writeln("");

        for (i, (name, cost)) in member.usable_spells().iter().enumerate() {
            let can_cast = member.mp >= *cost;
            w.set_fg(if can_cast { Color::LightCyan } else { Color::DarkGray });
            w.write_str(&format!("  [{}] ", i + 1));
            w.set_fg(if can_cast { Color::White } else { Color::DarkGray });
            w.writeln(&format!("{} ({}MP)", name, cost));
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  [B] Cancel");

    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_status(w: &mut AnsiWriter, state: &GameState) {
    render_header(w);

    w.writeln("");
    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("  GAME STATUS");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::White);
    w.writeln(&format!("  Play Time: {}", state.formatted_play_time()));
    w.writeln(&format!("  Gil: {}", format_number(state.gold)));
    w.writeln(&format!("  Battles: {}", state.battles_fought));
    w.writeln(&format!("  Monsters Defeated: {}", state.monsters_defeated));
    w.writeln("");

    // Story progress
    w.set_fg(Color::LightMagenta);
    w.writeln("  Crystals:");
    w.set_fg(if state.has_flag("earth_crystal_lit") { Color::LightGreen } else { Color::DarkGray });
    w.writeln(&format!("    Earth: {}", if state.has_flag("earth_crystal_lit") { "Restored" } else { "Dark" }));
    w.set_fg(if state.has_flag("fire_crystal_lit") { Color::LightRed } else { Color::DarkGray });
    w.writeln(&format!("    Fire:  {}", if state.has_flag("fire_crystal_lit") { "Restored" } else { "Dark" }));
    w.set_fg(if state.has_flag("water_crystal_lit") { Color::LightBlue } else { Color::DarkGray });
    w.writeln(&format!("    Water: {}", if state.has_flag("water_crystal_lit") { "Restored" } else { "Dark" }));
    w.set_fg(if state.has_flag("wind_crystal_lit") { Color::LightCyan } else { Color::DarkGray });
    w.writeln(&format!("    Wind:  {}", if state.has_flag("wind_crystal_lit") { "Restored" } else { "Dark" }));

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_story_event(w: &mut AnsiWriter, event_key: &str) {
    w.clear_screen();
    w.set_fg(Color::LightMagenta);
    w.writeln("");
    w.writeln(&format!("  Story Event: {}", event_key));
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_victory(w: &mut AnsiWriter, state: &GameState, exp: u64, gold: u32) {
    w.clear_screen();

    w.set_fg(Color::Yellow);
    w.bold();
    w.writeln("");
    w.writeln("  ╔═══════════════════════════════════════════╗");
    w.writeln("  ║             VICTORY!                      ║");
    w.writeln("  ╚═══════════════════════════════════════════╝");
    w.reset_color();
    w.writeln("");

    w.set_fg(Color::LightGreen);
    w.writeln(&format!("  Gained {} EXP!", exp));
    w.set_fg(Color::Yellow);
    w.writeln(&format!("  Found {} Gil!", gold));
    w.writeln("");

    // Party status after battle
    for member in &state.party.members {
        w.set_fg(Color::White);
        w.writeln(&format!("  {} Lv.{} - {}/{} HP", member.name, member.level, member.hp, member.hp_max));
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

fn render_game_over(w: &mut AnsiWriter) {
    w.clear_screen();

    w.set_fg(Color::Red);
    w.bold();
    w.writeln("");
    w.writeln("  ╔═══════════════════════════════════════════╗");
    w.writeln("  ║             GAME OVER                     ║");
    w.writeln("  ╚═══════════════════════════════════════════╝");
    w.reset_color();
    w.writeln("");
    w.set_fg(Color::LightGray);
    w.writeln("  The darkness consumes all...");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to return to the main menu.");
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
    w.writeln("  Return anytime to continue your quest.");
    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Are you sure? [Y/N]");
    w.reset_color();
    w.writeln("");
    w.write_str("  > ");
}

fn render_ending(w: &mut AnsiWriter, state: &GameState, phase: u8) {
    w.clear_screen();

    match phase {
        0 => {
            w.set_fg(Color::White);
            w.writeln("");
            w.writeln("  The Void Lord falls.");
            w.writeln("  The corrupted process terminates.");
        }
        1 => {
            w.set_fg(Color::LightGray);
            w.writeln("");
            w.writeln("  The world begins to... freeze.");
            w.writeln("  NPCs stop mid-motion.");
            w.writeln("  The sky flickers.");
        }
        2 => {
            w.set_fg(Color::LightMagenta);
            w.writeln("");
            w.writeln("  You reach the Core.");
            w.writeln("  A terminal pulses with light.");
            w.writeln("");
            w.writeln("  \"SIMULATION COMPLETE\"");
        }
        3 => {
            w.set_fg(Color::White);
            w.writeln("");
            w.writeln("  You touch the terminal...");
            w.writeln("");
            w.writeln("  And wake up.");
        }
        4 => {
            w.set_fg(Color::LightCyan);
            w.writeln("");
            w.writeln("  A hospital bed. The year 2XXX.");
            w.writeln("  Doctors crowd around.");
            w.writeln("");
            w.writeln("  \"The subject is waking up.\"");
            w.writeln("  \"Simulation therapy complete.\"");
        }
        _ => {
            w.set_fg(Color::Yellow);
            w.bold();
            w.writeln("");
            w.writeln("  ╔═══════════════════════════════════════════════════════════════════════╗");
            w.writeln("  ║                         THE END                                       ║");
            w.writeln("  ╚═══════════════════════════════════════════════════════════════════════╝");
            w.reset_color();
            w.writeln("");
            w.writeln(&format!("  Play Time: {}", state.formatted_play_time()));
            w.writeln("");
            w.set_fg(Color::LightGray);
            w.writeln("  Was it all a dream?");
            w.writeln("  Or was the dream... real?");
        }
    }

    w.writeln("");
    w.set_fg(Color::LightCyan);
    w.writeln("  Press any key to continue...");
    w.reset_color();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(100), "100");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_render_no_panic() {
        let flow = LastDreamFlow::new();
        let output = render_screen(&flow);
        assert!(!output.is_empty());
    }
}
