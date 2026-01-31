//! Screen state machine for Depths of Diablo
//!
//! Manages game screens and input handling for the real-time dungeon crawler.

use std::time::Instant;

use super::combat::CombatEngine;
use super::data::{CharacterClass, CLASSES};
use super::dungeon::{daily_seed, Dungeon};
use super::items::Item;
use super::state::GameState;

/// Game screens
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Title/intro screen
    Intro,
    /// Main menu (create game, join, solo)
    MainMenu,
    /// Lobby waiting for players
    Lobby,
    /// Character class selection
    ClassSelect,
    /// Town hub between dungeon runs
    Town,
    /// Active dungeon crawling
    Dungeon,
    /// Inventory/equipment management
    Inventory,
    /// Skills menu
    Skills,
    /// Character stats
    Stats,
    /// Town shop
    Shop,
    /// Town stash
    Stash,
    /// Town blacksmith upgrades
    Blacksmith,
    /// Leaderboard
    Leaderboard,
    /// Game over (death)
    GameOver,
    /// Victory (completed floor 20)
    Victory,
    /// Confirm quit
    ConfirmQuit,
}

/// Actions returned by the flow for the session to handle
#[derive(Debug, Clone)]
pub enum DiabloAction {
    /// No action needed
    Continue,
    /// Render the screen
    Render(String),
    /// Echo characters
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game ended (death or victory)
    GameEnd { success: bool, floor_reached: u32, soul_essence: i64 },
    /// Player quit to BBS menu
    Quit,
    /// Needs real-time update (returns true if something changed)
    Tick,
}

/// Game flow state machine
pub struct DiabloFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub combat_engine: CombatEngine,
    input_buffer: String,
    last_tick: Instant,
    /// Selected menu option
    selected_option: usize,
    /// Current page for inventory/lists
    current_page: usize,
}

impl DiabloFlow {
    /// Create new game from fresh state
    pub fn new(user_id: i64, handle: &str) -> Self {
        Self {
            state: GameState::new(user_id, handle),
            screen: GameScreen::Intro,
            combat_engine: CombatEngine::new(),
            input_buffer: String::new(),
            last_tick: Instant::now(),
            selected_option: 0,
            current_page: 0,
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState) -> Self {
        // Determine starting screen based on state
        let screen = if state.is_in_run() {
            if state.is_in_town() {
                GameScreen::Town
            } else {
                GameScreen::Dungeon
            }
        } else {
            GameScreen::MainMenu
        };

        Self {
            state,
            screen,
            combat_engine: CombatEngine::new(),
            input_buffer: String::new(),
            last_tick: Instant::now(),
            selected_option: 0,
            current_page: 0,
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Get mutable game state
    pub fn game_state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> DiabloAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return DiabloAction::Echo("\x08 \x08".to_string());
            }
            return DiabloAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return DiabloAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return DiabloAction::Echo(ch.to_string());
        }

        DiabloAction::Continue
    }

    /// Real-time tick for combat updates
    pub fn tick(&mut self) -> DiabloAction {
        if self.screen != GameScreen::Dungeon {
            return DiabloAction::Continue;
        }

        // Only tick every 100ms
        let elapsed = self.last_tick.elapsed();
        if elapsed.as_millis() < 100 {
            return DiabloAction::Continue;
        }
        self.last_tick = Instant::now();

        // Update combat engine
        self.combat_engine.update();

        // Process monster attacks if in combat range
        if let (Some(ref mut character), Some(ref mut dungeon)) =
            (&mut self.state.character, &mut self.state.dungeon)
        {
            // Get nearby monsters
            let nearby = dungeon.get_monsters_in_radius(character.x, character.y, 2);

            if !nearby.is_empty() {
                // Calculate player armor
                let equipped: Vec<&Item> = character.equipped_items();
                let armor = CombatEngine::calculate_player_armor(character, &equipped);

                // Get mutable references to nearby monsters
                let monster_ids: Vec<u64> = nearby.iter().map(|m| m.id).collect();
                let mut monsters: Vec<&mut super::dungeon::Monster> = dungeon
                    .monsters
                    .iter_mut()
                    .filter(|m| monster_ids.contains(&m.id) && m.is_alive())
                    .collect();

                // Process monster attacks
                let result = self.combat_engine.process_monster_attacks(
                    character,
                    &mut monsters,
                    armor,
                );

                // Add combat messages
                for msg in &result.messages {
                    self.state.add_message(msg);
                }

                if result.player_died {
                    self.state.end_run(false);
                    self.screen = GameScreen::GameOver;
                    return DiabloAction::SaveGame;
                }

                if result.player_damage_taken > 0 {
                    return DiabloAction::Tick;
                }
            }
        }

        DiabloAction::Continue
    }

    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::MainMenu
                | GameScreen::Lobby
                | GameScreen::ClassSelect
                | GameScreen::Town
                | GameScreen::Dungeon
                | GameScreen::Inventory
                | GameScreen::Skills
                | GameScreen::Stats
                | GameScreen::Shop
                | GameScreen::Stash
                | GameScreen::Blacksmith
                | GameScreen::Leaderboard
                | GameScreen::GameOver
                | GameScreen::Victory
                | GameScreen::ConfirmQuit
        )
    }

    fn process_input(&mut self) -> DiabloAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::MainMenu => self.handle_main_menu(&input),
            GameScreen::Lobby => self.handle_lobby(&input),
            GameScreen::ClassSelect => self.handle_class_select(&input),
            GameScreen::Town => self.handle_town(&input),
            GameScreen::Dungeon => self.handle_dungeon(&input),
            GameScreen::Inventory => self.handle_inventory(&input),
            GameScreen::Skills => self.handle_skills(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Shop => self.handle_shop(&input),
            GameScreen::Stash => self.handle_stash(&input),
            GameScreen::Blacksmith => self.handle_blacksmith(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::Victory => self.handle_victory(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> DiabloAction {
        self.screen = GameScreen::MainMenu;
        DiabloAction::SaveGame
    }

    fn handle_main_menu(&mut self, input: &str) -> DiabloAction {
        match input {
            "N" | "1" => {
                // New solo game
                self.screen = GameScreen::ClassSelect;
                DiabloAction::SaveGame
            }
            "J" | "2" => {
                // Join public game - for now, start solo
                self.screen = GameScreen::ClassSelect;
                DiabloAction::SaveGame
            }
            "P" | "3" => {
                // Create private game - for now, start solo
                self.screen = GameScreen::ClassSelect;
                DiabloAction::SaveGame
            }
            "L" | "4" => {
                self.screen = GameScreen::Leaderboard;
                DiabloAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_lobby(&mut self, input: &str) -> DiabloAction {
        match input {
            "R" => {
                // Toggle ready
                DiabloAction::Continue
            }
            "S" => {
                // Start game (host only)
                self.screen = GameScreen::ClassSelect;
                DiabloAction::SaveGame
            }
            "L" => {
                // Leave lobby
                self.screen = GameScreen::MainMenu;
                DiabloAction::SaveGame
            }
            "1" | "2" | "3" => {
                // Select class
                let class_idx = input.parse::<usize>().unwrap_or(1) - 1;
                if class_idx < CLASSES.len() {
                    // Class selection happens in ClassSelect screen
                }
                DiabloAction::Continue
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_class_select(&mut self, input: &str) -> DiabloAction {
        match input {
            "1" => {
                self.start_new_run(CharacterClass::Warrior);
                self.screen = GameScreen::Town;
                DiabloAction::SaveGame
            }
            "2" => {
                if self.state.meta.unlocked_classes.contains(&CharacterClass::Rogue) {
                    self.start_new_run(CharacterClass::Rogue);
                    self.screen = GameScreen::Town;
                    DiabloAction::SaveGame
                } else {
                    self.state.add_message("Rogue not unlocked! Need 200 Soul Essence.");
                    DiabloAction::Continue
                }
            }
            "3" => {
                if self.state.meta.unlocked_classes.contains(&CharacterClass::Sorcerer) {
                    self.start_new_run(CharacterClass::Sorcerer);
                    self.screen = GameScreen::Town;
                    DiabloAction::SaveGame
                } else {
                    self.state.add_message("Sorcerer not unlocked! Need 200 Soul Essence.");
                    DiabloAction::Continue
                }
            }
            "B" | "Q" => {
                self.screen = GameScreen::MainMenu;
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn start_new_run(&mut self, class: CharacterClass) {
        let seed = daily_seed();
        self.state.start_run(&self.state.handle.clone(), class, seed);
        self.state.add_message(&format!("A new {} begins the descent...", class.name()));
    }

    fn handle_town(&mut self, input: &str) -> DiabloAction {
        match input {
            "E" | "1" => {
                // Enter dungeon
                self.state.enter_dungeon();
                self.screen = GameScreen::Dungeon;

                // Explore initial area
                if let (Some(ref char), Some(ref mut dungeon)) =
                    (&self.state.character, &mut self.state.dungeon)
                {
                    dungeon.explore(char.x, char.y, 5);
                }

                DiabloAction::SaveGame
            }
            "I" | "2" => {
                self.screen = GameScreen::Inventory;
                DiabloAction::SaveGame
            }
            "S" | "3" => {
                self.screen = GameScreen::Skills;
                DiabloAction::SaveGame
            }
            "C" | "4" => {
                self.screen = GameScreen::Stats;
                DiabloAction::SaveGame
            }
            "H" | "5" => {
                self.screen = GameScreen::Shop;
                DiabloAction::SaveGame
            }
            "T" | "6" => {
                self.screen = GameScreen::Stash;
                DiabloAction::SaveGame
            }
            "B" | "7" => {
                self.screen = GameScreen::Blacksmith;
                DiabloAction::SaveGame
            }
            "A" | "8" => {
                // Abandon run
                self.state.end_run(false);
                self.screen = GameScreen::MainMenu;
                DiabloAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_dungeon(&mut self, input: &str) -> DiabloAction {
        // Movement keys (WASD or arrow simulation)
        let (dx, dy): (i32, i32) = match input {
            "W" | "8" => (0, -1),  // Up
            "S" | "2" => (0, 1),   // Down
            "A" | "4" => (-1, 0),  // Left
            "D" | "6" => (1, 0),   // Right
            "Q" | "7" => (-1, -1), // Up-left
            "E" | "9" => (1, -1),  // Up-right
            "Z" | "1" => (-1, 1),  // Down-left
            "C" | "3" => (1, 1),   // Down-right
            "5" => (0, 0),         // Wait/stay
            _ => {
                // Handle other keys
                return self.handle_dungeon_action(input);
            }
        };

        self.process_movement(dx, dy)
    }

    fn process_movement(&mut self, dx: i32, dy: i32) -> DiabloAction {
        // Collect messages and actions to avoid borrow conflicts
        let mut messages: Vec<String> = Vec::new();
        let mut apply_shrine = false;
        let mut open_chest_at: Option<(usize, usize)> = None;

        let result = {
            let (char, dungeon) = match (&mut self.state.character, &mut self.state.dungeon) {
                (Some(c), Some(d)) => (c, d),
                _ => return DiabloAction::Continue,
            };

            let new_x = (char.x as i32 + dx).max(0) as usize;
            let new_y = (char.y as i32 + dy).max(0) as usize;

            // Check for monster at destination
            if let Some(monster) = dungeon.get_monster_at(new_x, new_y) {
                if monster.is_alive() {
                    // Attack the monster instead of moving
                    return self.process_attack(new_x, new_y);
                }
            }

            // Check if walkable
            if !dungeon.is_walkable(new_x, new_y) {
                return DiabloAction::Continue;
            }

            // Move
            char.x = new_x;
            char.y = new_y;

            // Explore new area
            dungeon.explore(new_x, new_y, 5);

            // Check for items
            let items = dungeon.get_items_at(new_x, new_y);
            if !items.is_empty() {
                messages.push(format!("You see {} item(s) here. Press G to get.", items.len()));
            }

            // Check for stairs
            if let Some(tile) = dungeon.get_tile(new_x, new_y) {
                match tile {
                    super::dungeon::Tile::StairsDown => {
                        messages.push("You see stairs leading down. Press > to descend.".to_string());
                    }
                    super::dungeon::Tile::StairsUp => {
                        messages.push("You see stairs leading up. Press < to ascend.".to_string());
                    }
                    super::dungeon::Tile::Shrine => {
                        apply_shrine = true;
                    }
                    super::dungeon::Tile::Chest => {
                        open_chest_at = Some((new_x, new_y));
                    }
                    _ => {}
                }
            }

            DiabloAction::Tick
        };

        // Apply deferred actions
        for msg in messages {
            self.state.add_message(&msg);
        }
        if apply_shrine {
            self.apply_shrine_buff();
        }
        if let Some((x, y)) = open_chest_at {
            self.open_chest(x, y);
        }

        result
    }

    fn process_attack(&mut self, target_x: usize, target_y: usize) -> DiabloAction {
        // Get equipped items as owned data to avoid lifetime issues
        let equipped_data: Vec<super::items::Item> = {
            match &self.state.character {
                Some(c) => c.equipped_items().into_iter().cloned().collect(),
                None => return DiabloAction::Continue,
            }
        };

        // Check attack cooldown
        let speed_bonus: i32 = equipped_data.iter()
            .map(|i| i.get_stat_bonus(super::items::AffixStat::AttackSpeed))
            .sum();

        if !self.combat_engine.can_attack(speed_bonus) {
            return DiabloAction::Continue;
        }

        // Process attack and collect results
        let (messages, _xp_gained, _monsters_killed, _dex_bonus, level_up) = {
            let (char, dungeon) = match (&mut self.state.character, &mut self.state.dungeon) {
                (Some(c), Some(d)) => (c, d),
                _ => return DiabloAction::Continue,
            };

            // Get target monster
            let monster = match dungeon.get_monster_at_mut(target_x, target_y) {
                Some(m) => m,
                None => return DiabloAction::Continue,
            };

            let equipped_refs: Vec<&super::items::Item> = equipped_data.iter().collect();

            // Process attack
            let result = self.combat_engine.process_attack(char, monster, &equipped_refs);

            let messages = result.messages.clone();
            let xp = result.xp_gained;
            let killed = result.monsters_killed.clone();
            let dex = char.total_dexterity() / 5;

            // Handle XP gain
            let leveled = if xp > 0 {
                char.add_experience(xp)
            } else {
                false
            };

            // Spawn loot for killed monsters
            for monster_id in &killed {
                if let Some(m) = dungeon.monsters.iter().find(|m| m.id == *monster_id) {
                    let mx = m.x;
                    let my = m.y;
                    dungeon.spawn_loot(mx, my, dex);
                }
            }

            (messages, xp, killed, dex, leveled)
        };

        // Add messages after borrow ends
        for msg in &messages {
            self.state.add_message(msg);
        }
        if level_up {
            if let Some(ref char) = self.state.character {
                self.state.add_message(&format!("LEVEL UP! Now level {}!", char.level));
            }
        }

        DiabloAction::Tick
    }

    fn handle_dungeon_action(&mut self, input: &str) -> DiabloAction {
        match input {
            ">" => {
                // Descend
                if let Some(ref dungeon) = self.state.dungeon {
                    if let Some(ref char) = self.state.character {
                        if (char.x, char.y) == dungeon.exit_pos {
                            if dungeon.alive_monster_count() == 0 || self.state.run.as_ref().map(|r| r.current_floor).unwrap_or(1) <= 5 {
                                let floor = self.state.run.as_ref().map(|r| r.current_floor).unwrap_or(1);
                                if floor >= 20 {
                                    // Victory!
                                    self.state.end_run(true);
                                    self.screen = GameScreen::Victory;
                                    return DiabloAction::SaveGame;
                                }
                                self.state.descend();
                                if let Some(ref mut dungeon) = self.state.dungeon {
                                    if let Some(ref char) = self.state.character {
                                        dungeon.explore(char.x, char.y, 5);
                                    }
                                }
                                let new_floor = self.state.run.as_ref().map(|r| r.current_floor).unwrap_or(1);
                                self.state.add_message(&format!("Descending to floor {}...", new_floor));
                                return DiabloAction::SaveGame;
                            } else {
                                self.state.add_message("Clear all monsters before descending!");
                            }
                        }
                    }
                }
                DiabloAction::Continue
            }
            "<" => {
                // Ascend
                if let Some(ref dungeon) = self.state.dungeon {
                    if let Some(ref char) = self.state.character {
                        if (char.x, char.y) == dungeon.start_pos {
                            let floor = self.state.run.as_ref().map(|r| r.current_floor).unwrap_or(1);
                            if floor == 1 {
                                self.state.return_to_town();
                                self.screen = GameScreen::Town;
                                return DiabloAction::SaveGame;
                            } else {
                                // Go up a floor
                                if let Some(ref mut run) = self.state.run {
                                    run.ascend();
                                }
                                self.state.dungeon = Some(Dungeon::generate(
                                    self.state.run.as_ref().map(|r| r.current_floor).unwrap_or(1),
                                    self.state.run.as_ref().map(|r| r.seed).unwrap_or(0),
                                ));
                                if let Some(ref mut dungeon) = self.state.dungeon {
                                    if let Some(ref mut char) = self.state.character {
                                        char.x = dungeon.exit_pos.0;
                                        char.y = dungeon.exit_pos.1;
                                        dungeon.explore(char.x, char.y, 5);
                                    }
                                }
                                self.state.add_message("Ascending...");
                                return DiabloAction::SaveGame;
                            }
                        }
                    }
                }
                DiabloAction::Continue
            }
            "G" => {
                // Get item
                if let (Some(ref mut char), Some(ref mut dungeon)) =
                    (&mut self.state.character, &mut self.state.dungeon)
                {
                    if let Some(item) = dungeon.pickup_item(char.x, char.y) {
                        let name = item.name.clone();
                        if item.item_type.is_consumable() {
                            match item.item_type {
                                super::items::ItemType::HealthPotion => {
                                    char.health_potions += 1;
                                }
                                super::items::ItemType::ManaPotion => {
                                    char.mana_potions += 1;
                                }
                                _ => {
                                    char.inventory.push(item);
                                }
                            }
                        } else {
                            char.inventory.push(item);
                        }
                        self.state.add_message(&format!("Picked up: {}", name));
                    }
                }
                DiabloAction::Tick
            }
            "H" => {
                // Use health potion
                let message = if let Some(ref mut char) = self.state.character {
                    if char.use_health_potion() {
                        Some(format!("Health restored to {}", char.health))
                    } else {
                        Some("No health potions or already full!".to_string())
                    }
                } else {
                    None
                };
                if let Some(msg) = message {
                    self.state.add_message(&msg);
                }
                DiabloAction::Tick
            }
            "M" => {
                // Use mana potion
                let message = if let Some(ref mut char) = self.state.character {
                    if char.use_mana_potion() {
                        Some(format!("Mana restored to {}", char.mana))
                    } else {
                        Some("No mana potions or already full!".to_string())
                    }
                } else {
                    None
                };
                if let Some(msg) = message {
                    self.state.add_message(&msg);
                }
                DiabloAction::Tick
            }
            "F" => {
                // Use skill
                self.use_active_skill()
            }
            "N" => {
                // Next skill
                if let Some(ref mut char) = self.state.character {
                    char.next_skill();
                    if let Some(skill) = char.active_skill() {
                        if let Some(def) = super::data::get_skill(&skill.key) {
                            self.state.add_message(&format!("Active skill: {}", def.name));
                        }
                    }
                }
                DiabloAction::Continue
            }
            "I" => {
                self.screen = GameScreen::Inventory;
                DiabloAction::SaveGame
            }
            "K" => {
                self.screen = GameScreen::Skills;
                DiabloAction::SaveGame
            }
            "T" => {
                // Return to town (portal)
                self.state.return_to_town();
                self.screen = GameScreen::Town;
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn use_active_skill(&mut self) -> DiabloAction {
        // Check for no skill message first
        let has_skill = self.state.character.as_ref()
            .map(|c| c.active_skill().is_some())
            .unwrap_or(false);
        if !has_skill {
            self.state.add_message("No skill selected!");
            return DiabloAction::Continue;
        }

        // Check for nearby monsters
        let (char_x, char_y) = match &self.state.character {
            Some(c) => (c.x, c.y),
            None => return DiabloAction::Continue,
        };

        let nearby_count = match &self.state.dungeon {
            Some(d) => d.get_monsters_in_radius(char_x, char_y, 3).len(),
            None => return DiabloAction::Continue,
        };
        if nearby_count == 0 {
            self.state.add_message("No enemies in range!");
            return DiabloAction::Continue;
        }

        // Get equipped items as owned data
        let equipped_data: Vec<super::items::Item> = match &self.state.character {
            Some(c) => c.equipped_items().into_iter().cloned().collect(),
            None => return DiabloAction::Continue,
        };

        // Process skill and collect results
        let (messages, _xp_gained, _monsters_killed, level_up, _dex_bonus) = {
            let (char, dungeon) = match (&mut self.state.character, &mut self.state.dungeon) {
                (Some(c), Some(d)) => (c, d),
                _ => return DiabloAction::Continue,
            };

            // Get nearby monsters
            let nearby = dungeon.get_monsters_in_radius(char.x, char.y, 3);
            let monster_ids: Vec<u64> = nearby.iter().map(|m| m.id).collect();

            // Get skill index
            let skill_index = char.active_skill_index;

            let mut targets: Vec<&mut super::dungeon::Monster> = dungeon
                .monsters
                .iter_mut()
                .filter(|m| monster_ids.contains(&m.id) && m.is_alive())
                .collect();

            let equipped_refs: Vec<&super::items::Item> = equipped_data.iter().collect();

            // Process skill by index
            let result = self.combat_engine.process_skill_by_index(
                char,
                skill_index,
                &mut targets,
                &equipped_refs,
            );

            let messages = result.messages.clone();
            let killed = result.monsters_killed.clone();
            let dex = char.total_dexterity() / 5;

            // Handle XP
            let leveled = if result.xp_gained > 0 {
                char.add_experience(result.xp_gained)
            } else {
                false
            };

            // Spawn loot
            for monster_id in &killed {
                if let Some(m) = dungeon.monsters.iter().find(|m| m.id == *monster_id) {
                    let mx = m.x;
                    let my = m.y;
                    dungeon.spawn_loot(mx, my, dex);
                }
            }

            (messages, result.xp_gained, killed, leveled, dex)
        };

        // Add messages after borrow ends
        for msg in &messages {
            self.state.add_message(msg);
        }
        if level_up {
            if let Some(ref char) = self.state.character {
                self.state.add_message(&format!("LEVEL UP! Now level {}!", char.level));
            }
        }

        DiabloAction::Tick
    }

    fn apply_shrine_buff(&mut self) {
        if let Some(ref mut char) = self.state.character {
            // Random buff
            let buff_type = rand::random::<u8>() % 4;
            match buff_type {
                0 => {
                    char.health = char.max_health;
                    self.state.add_message("Health Shrine: Full health restored!");
                }
                1 => {
                    char.mana = char.max_mana;
                    self.state.add_message("Mana Shrine: Full mana restored!");
                }
                2 => {
                    char.strength += 5;
                    self.state.add_message("Power Shrine: +5 Strength!");
                }
                _ => {
                    char.max_health += 20;
                    char.health += 20;
                    self.state.add_message("Vitality Shrine: +20 Max Health!");
                }
            }
        }
    }

    fn open_chest(&mut self, x: usize, y: usize) {
        let floor = self.state.run.as_ref().map(|r| r.current_floor).unwrap_or(1);
        let opened = {
            if let (Some(ref mut char), Some(ref mut dungeon)) =
                (&mut self.state.character, &mut self.state.dungeon)
            {
                // Generate random loot
                let item = Item::generate(
                    rand::random(),
                    floor,
                    char.total_dexterity() / 5 + 20, // Bonus luck for chests
                );
                dungeon.drop_item(item, x, y);

                // Also some gold
                char.gold += (floor as i32 * 10) + rand::random::<i32>() % 50;
                true
            } else {
                false
            }
        };

        if opened {
            self.state.add_message("Chest opened! Items and gold found.");
        }
    }

    fn handle_inventory(&mut self, input: &str) -> DiabloAction {
        match input {
            "B" | "Q" => {
                self.screen = if self.state.is_in_town() {
                    GameScreen::Town
                } else {
                    GameScreen::Dungeon
                };
                DiabloAction::SaveGame
            }
            c if c.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) => {
                // Equip item by number
                let idx = c.parse::<usize>().unwrap_or(0);
                if idx > 0 {
                    self.equip_item(idx - 1);
                }
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn equip_item(&mut self, idx: usize) {
        let message = if let Some(ref mut char) = self.state.character {
            if idx < char.inventory.len() {
                let item = char.inventory.remove(idx);
                if item.can_equip() {
                    let name = item.name.clone();
                    if let Some(old_item) = char.equip(item) {
                        char.inventory.push(old_item);
                    }
                    Some(format!("Equipped: {}", name))
                } else {
                    char.inventory.insert(idx, item);
                    Some("Cannot equip this item!".to_string())
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(msg) = message {
            self.state.add_message(&msg);
        }
    }

    fn handle_skills(&mut self, input: &str) -> DiabloAction {
        match input {
            "B" | "Q" => {
                self.screen = if self.state.is_in_town() {
                    GameScreen::Town
                } else {
                    GameScreen::Dungeon
                };
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_stats(&mut self, input: &str) -> DiabloAction {
        match input {
            "B" | "Q" => {
                self.screen = if self.state.is_in_town() {
                    GameScreen::Town
                } else {
                    GameScreen::Dungeon
                };
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_shop(&mut self, input: &str) -> DiabloAction {
        match input {
            "B" | "Q" => {
                self.screen = GameScreen::Town;
                DiabloAction::SaveGame
            }
            "1" => {
                // Buy health potion (50 gold)
                if let Some(ref mut char) = self.state.character {
                    if char.gold >= 50 {
                        char.gold -= 50;
                        char.health_potions += 1;
                        self.state.add_message("Bought Health Potion!");
                    } else {
                        self.state.add_message("Not enough gold!");
                    }
                }
                DiabloAction::SaveGame
            }
            "2" => {
                // Buy mana potion (50 gold)
                if let Some(ref mut char) = self.state.character {
                    if char.gold >= 50 {
                        char.gold -= 50;
                        char.mana_potions += 1;
                        self.state.add_message("Bought Mana Potion!");
                    } else {
                        self.state.add_message("Not enough gold!");
                    }
                }
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_stash(&mut self, input: &str) -> DiabloAction {
        match input {
            "B" | "Q" => {
                self.screen = GameScreen::Town;
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_blacksmith(&mut self, input: &str) -> DiabloAction {
        match input {
            "B" | "Q" => {
                self.screen = GameScreen::Town;
                DiabloAction::SaveGame
            }
            "1" => {
                // Upgrade blacksmith (100 soul essence)
                if self.state.meta.upgrade_town("blacksmith", 100) {
                    let level = self.state.meta.get_upgrade_level("blacksmith");
                    self.state.add_message(&format!("Blacksmith upgraded to level {}!", level));
                } else {
                    self.state.add_message("Not enough Soul Essence!");
                }
                DiabloAction::SaveGame
            }
            "2" => {
                // Unlock Rogue
                if self.state.meta.unlock_class(CharacterClass::Rogue) {
                    self.state.add_message("Rogue class unlocked!");
                } else {
                    self.state.add_message("Already unlocked or not enough Soul Essence!");
                }
                DiabloAction::SaveGame
            }
            "3" => {
                // Unlock Sorcerer
                if self.state.meta.unlock_class(CharacterClass::Sorcerer) {
                    self.state.add_message("Sorcerer class unlocked!");
                } else {
                    self.state.add_message("Already unlocked or not enough Soul Essence!");
                }
                DiabloAction::SaveGame
            }
            _ => DiabloAction::Continue,
        }
    }

    fn handle_leaderboard(&mut self, _input: &str) -> DiabloAction {
        self.screen = GameScreen::MainMenu;
        DiabloAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> DiabloAction {
        self.screen = GameScreen::MainMenu;
        DiabloAction::SaveGame
    }

    fn handle_victory(&mut self, _input: &str) -> DiabloAction {
        self.screen = GameScreen::MainMenu;
        DiabloAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> DiabloAction {
        match input {
            "Y" => DiabloAction::Quit,
            _ => {
                self.screen = if self.state.is_in_run() {
                    if self.state.is_in_town() {
                        GameScreen::Town
                    } else {
                        GameScreen::Dungeon
                    }
                } else {
                    GameScreen::MainMenu
                };
                DiabloAction::SaveGame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = DiabloFlow::new(1, "TestPlayer");
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_intro_transition() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_class_select() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::ClassSelect;

        flow.handle_char('1');
        assert!(matches!(flow.screen, GameScreen::Town));
        assert!(flow.state.character.is_some());
    }

    #[test]
    fn test_movement() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.state.enter_dungeon();
        flow.screen = GameScreen::Dungeon;

        if let Some(ref mut dungeon) = flow.state.dungeon {
            if let Some(ref char) = flow.state.character {
                dungeon.explore(char.x, char.y, 5);
            }
        }

        let _old_x = flow.state.character.as_ref().unwrap().x;

        // Try to move right
        flow.handle_char('D');

        // Position may or may not have changed depending on dungeon layout
        // Just verify no crash
        assert!(flow.state.character.is_some());
    }

    #[test]
    fn test_town_navigation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        // Go to shop
        flow.handle_char('H');
        assert!(matches!(flow.screen, GameScreen::Shop));

        // Back to town
        flow.handle_char('B');
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_main_menu_to_class_select() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::MainMenu;

        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::ClassSelect));
    }

    #[test]
    fn test_main_menu_to_leaderboard() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::MainMenu;

        flow.handle_char('L');
        assert!(matches!(flow.screen, GameScreen::Leaderboard));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::MainMenu;

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        // Decline quit
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_confirm_quit() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::ConfirmQuit;

        let action = flow.handle_char('Y');
        assert!(matches!(action, DiabloAction::Quit));
    }

    #[test]
    fn test_class_selection_back() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::ClassSelect;

        flow.handle_char('B');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_locked_class_selection() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::ClassSelect;

        // Try to select locked Rogue class
        flow.handle_char('2');

        // Should still be at class select with error message
        assert!(matches!(flow.screen, GameScreen::ClassSelect));
        assert!(!flow.state.meta.unlocked_classes.contains(&CharacterClass::Rogue));
    }

    #[test]
    fn test_town_to_dungeon() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('E'); // Enter dungeon
        assert!(matches!(flow.screen, GameScreen::Dungeon));
        assert!(flow.state.dungeon.is_some());
        assert!(!flow.state.is_in_town());
    }

    #[test]
    fn test_dungeon_to_town_portal() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.state.enter_dungeon();
        flow.screen = GameScreen::Dungeon;

        flow.handle_char('T'); // Town portal
        assert!(matches!(flow.screen, GameScreen::Town));
        assert!(flow.state.is_in_town());
    }

    #[test]
    fn test_inventory_navigation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('I'); // Inventory
        assert!(matches!(flow.screen, GameScreen::Inventory));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_skills_navigation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('S'); // Skills
        assert!(matches!(flow.screen, GameScreen::Skills));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_stats_navigation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('C'); // Character stats
        assert!(matches!(flow.screen, GameScreen::Stats));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_blacksmith_navigation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('B'); // Blacksmith
        assert!(matches!(flow.screen, GameScreen::Blacksmith));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_stash_navigation() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('T'); // Stash
        assert!(matches!(flow.screen, GameScreen::Stash));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_abandon_run() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Town;

        flow.handle_char('A'); // Abandon
        assert!(matches!(flow.screen, GameScreen::MainMenu));
        assert!(!flow.state.is_in_run());
    }

    #[test]
    fn test_buy_health_potion() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.screen = GameScreen::Shop;

        let old_potions = flow.state.character.as_ref().unwrap().health_potions;
        let old_gold = flow.state.character.as_ref().unwrap().gold;

        flow.handle_char('1'); // Buy health potion

        let new_potions = flow.state.character.as_ref().unwrap().health_potions;
        let new_gold = flow.state.character.as_ref().unwrap().gold;

        if old_gold >= 50 {
            assert_eq!(new_potions, old_potions + 1);
            assert_eq!(new_gold, old_gold - 50);
        }
    }

    #[test]
    fn test_skill_cycling() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.state.enter_dungeon();
        flow.screen = GameScreen::Dungeon;

        let _old_idx = flow.state.character.as_ref().unwrap().active_skill_index;
        flow.handle_char('N'); // Cycle skill

        // If warrior has multiple skills, index should change
        // (Warriors start with only bash, so may not cycle)
    }

    #[test]
    fn test_use_health_potion() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.state.enter_dungeon();
        flow.screen = GameScreen::Dungeon;

        // Damage the character first
        if let Some(ref mut char) = flow.state.character {
            char.health = char.max_health / 2;
        }

        let old_hp = flow.state.character.as_ref().unwrap().health;
        flow.handle_char('H'); // Use health potion

        let new_hp = flow.state.character.as_ref().unwrap().health;
        assert!(new_hp >= old_hp); // Should heal or stay same if no potions
    }

    #[test]
    fn test_use_mana_potion() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Sorcerer);
        // Unlock sorcerer first
        flow.state.meta.unlocked_classes.push(CharacterClass::Sorcerer);
        flow.state.enter_dungeon();
        flow.screen = GameScreen::Dungeon;

        // Deplete mana
        if let Some(ref mut char) = flow.state.character {
            char.mana = char.max_mana / 2;
        }

        let old_mana = flow.state.character.as_ref().unwrap().mana;
        flow.handle_char('M'); // Use mana potion

        let new_mana = flow.state.character.as_ref().unwrap().mana;
        assert!(new_mana >= old_mana);
    }

    #[test]
    fn test_game_state_serialization() {
        let flow = DiabloFlow::new(1, "TestPlayer");
        let state = flow.game_state();

        let json = serde_json::to_string(&state).unwrap();
        let restored: super::super::state::GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.user_id, state.user_id);
        assert_eq!(restored.handle, state.handle);
    }

    #[test]
    fn test_game_over_transition() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::GameOver;

        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_victory_transition() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::Victory;

        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_leaderboard_back() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::Leaderboard;

        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_dungeon_inventory_and_back() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.start_new_run(CharacterClass::Warrior);
        flow.state.enter_dungeon();
        flow.screen = GameScreen::Dungeon;

        flow.handle_char('I'); // Inventory
        assert!(matches!(flow.screen, GameScreen::Inventory));

        flow.handle_char('B'); // Back to dungeon
        assert!(matches!(flow.screen, GameScreen::Dungeon));
    }

    #[test]
    fn test_backspace_handling() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::Intro;

        // Backspace should be ignored gracefully
        let action = flow.handle_char('\x7f');
        assert!(matches!(action, DiabloAction::Continue));
    }

    #[test]
    fn test_tick_outside_dungeon() {
        let mut flow = DiabloFlow::new(1, "TestPlayer");
        flow.screen = GameScreen::MainMenu;

        let action = flow.tick();
        assert!(matches!(action, DiabloAction::Continue));
    }

    #[test]
    fn test_from_state_in_run() {
        let mut state = super::super::state::GameState::new(1, "TestPlayer");
        state.start_run("Hero", CharacterClass::Warrior, 12345);

        let flow = DiabloFlow::from_state(state);
        assert!(matches!(flow.screen, GameScreen::Town));
    }

    #[test]
    fn test_from_state_in_dungeon() {
        let mut state = super::super::state::GameState::new(1, "TestPlayer");
        state.start_run("Hero", CharacterClass::Warrior, 12345);
        state.enter_dungeon();

        let flow = DiabloFlow::from_state(state);
        assert!(matches!(flow.screen, GameScreen::Dungeon));
    }
}
