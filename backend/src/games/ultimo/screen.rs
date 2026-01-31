//! Screen state machine for Ultimo
//!
//! Handles all the screens and state transitions for the game.

use super::crafting::{craft, RECIPES};
use super::data::{get_item, get_monster_template, get_npc, get_quest, get_zone, Monster, NpcType};
use super::housing::{can_purchase_house, HouseType};
use super::state::{CombatState, GameState, InventoryItem, Position, QuestProgress};

/// All possible game screens
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// First time intro
    Intro,
    /// Character creation - name entry
    CharacterCreate,
    /// Character creation - stat allocation
    StatAllocation { points_remaining: u32 },
    /// Main game view (world map)
    WorldView,
    /// Character stats screen
    Stats,
    /// Inventory management
    Inventory,
    /// Skills list
    Skills,
    /// Quest log
    Quests,
    /// Active combat
    Combat,
    /// NPC interaction
    NpcDialogue { npc_key: String },
    /// Shop buying
    ShopBuy { npc_key: String },
    /// Shop selling
    ShopSell { npc_key: String },
    /// Bank deposit/withdraw
    Bank,
    /// Healer services
    Healer,
    /// Training skills
    Training { npc_key: String },
    /// Crafting menu
    Crafting,
    /// Crafting - select skill
    CraftingSkill { skill: String },
    /// Housing management
    Housing,
    /// Housing - buy a house
    HousingBuy,
    /// Housing - manage storage
    HousingStorage,
    /// Player trade listing
    TradeList,
    /// Player trade - create offer
    TradeCreate,
    /// Player trade
    Trade { with_player: String },
    /// Party management
    Party,
    /// Leaderboard
    Leaderboard,
    /// Death screen
    Dead,
    /// Confirm quit
    ConfirmQuit,
}

/// Actions returned to the session handler
#[derive(Debug, Clone)]
pub enum UltimoAction {
    /// No action needed
    Continue,
    /// Render screen
    Render(String),
    /// Echo characters
    Echo(String),
    /// Save game state
    SaveGame,
    /// Player quit the game
    Quit,
}

/// Game flow state machine
pub struct UltimoFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub input_buffer: String,
}

impl UltimoFlow {
    /// Create new game (character creation)
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            screen: GameScreen::Intro,
            input_buffer: String::new(),
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState) -> Self {
        let screen = if state.character.is_none() {
            GameScreen::CharacterCreate
        } else if state.character.as_ref().map(|c| c.is_dead).unwrap_or(false) {
            GameScreen::Dead
        } else {
            GameScreen::WorldView
        };

        Self {
            state,
            screen,
            input_buffer: String::new(),
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> UltimoAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return UltimoAction::Echo("\x08 \x08".to_string());
            }
            return UltimoAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return UltimoAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 30 {
            self.input_buffer.push(ch);
            return UltimoAction::Echo(ch.to_string());
        }

        UltimoAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::StatAllocation { .. }
                | GameScreen::WorldView
                | GameScreen::Stats
                | GameScreen::Inventory
                | GameScreen::Skills
                | GameScreen::Quests
                | GameScreen::Combat
                | GameScreen::NpcDialogue { .. }
                | GameScreen::ShopBuy { .. }
                | GameScreen::ShopSell { .. }
                | GameScreen::Bank
                | GameScreen::Healer
                | GameScreen::Training { .. }
                | GameScreen::Crafting
                | GameScreen::CraftingSkill { .. }
                | GameScreen::Housing
                | GameScreen::HousingBuy
                | GameScreen::HousingStorage
                | GameScreen::TradeList
                | GameScreen::TradeCreate
                | GameScreen::Party
                | GameScreen::Leaderboard
                | GameScreen::Dead
                | GameScreen::ConfirmQuit
        )
    }

    /// Process input
    fn process_input(&mut self) -> UltimoAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::CharacterCreate => self.handle_character_create(&input),
            GameScreen::StatAllocation { points_remaining } => {
                self.handle_stat_allocation(&input, *points_remaining)
            }
            GameScreen::WorldView => self.handle_world_view(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Inventory => self.handle_inventory(&input),
            GameScreen::Skills => self.handle_skills(&input),
            GameScreen::Quests => self.handle_quests(&input),
            GameScreen::Combat => self.handle_combat(&input),
            GameScreen::NpcDialogue { npc_key } => {
                let npc_key = npc_key.clone();
                self.handle_npc_dialogue(&input, &npc_key)
            }
            GameScreen::ShopBuy { npc_key } => {
                let npc_key = npc_key.clone();
                self.handle_shop_buy(&input, &npc_key)
            }
            GameScreen::ShopSell { npc_key } => {
                let npc_key = npc_key.clone();
                self.handle_shop_sell(&input, &npc_key)
            }
            GameScreen::Bank => self.handle_bank(&input),
            GameScreen::Healer => self.handle_healer(&input),
            GameScreen::Training { npc_key } => {
                let npc_key = npc_key.clone();
                self.handle_training(&input, &npc_key)
            }
            GameScreen::Crafting => self.handle_crafting(&input),
            GameScreen::CraftingSkill { skill } => {
                let skill = skill.clone();
                self.handle_crafting_skill(&input, &skill)
            }
            GameScreen::Housing => self.handle_housing(&input),
            GameScreen::HousingBuy => self.handle_housing_buy(&input),
            GameScreen::HousingStorage => self.handle_housing_storage(&input),
            GameScreen::TradeList => self.handle_trade_list(&input),
            GameScreen::TradeCreate => self.handle_trade_create(&input),
            GameScreen::Trade { .. } => self.handle_trade(&input),
            GameScreen::Party => self.handle_party(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::Dead => self.handle_dead(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    // ========================================================================
    // Screen handlers
    // ========================================================================

    fn handle_intro(&mut self, _input: &str) -> UltimoAction {
        self.screen = GameScreen::CharacterCreate;
        UltimoAction::SaveGame
    }

    fn handle_character_create(&mut self, input: &str) -> UltimoAction {
        let input = input.trim();
        if input.len() < 3 {
            if let Some(ref mut char) = self.state.character {
                char.last_message = Some("Name must be at least 3 characters.".to_string());
            }
            return UltimoAction::SaveGame;
        }
        if input.len() > 15 {
            if let Some(ref mut char) = self.state.character {
                char.last_message = Some("Name must be 15 characters or less.".to_string());
            }
            return UltimoAction::SaveGame;
        }

        // Create new character with name (user_id will be set by service)
        let character = super::state::Character::new(input, 0);
        self.state.character = Some(character);
        self.screen = GameScreen::StatAllocation {
            points_remaining: 15,
        };
        UltimoAction::SaveGame
    }

    fn handle_stat_allocation(&mut self, input: &str, points: u32) -> UltimoAction {
        if let Some(ref mut char) = self.state.character {
            match input {
                "1" | "S" => {
                    if points > 0 {
                        char.strength += 1;
                        char.max_hp += 2;
                        char.hp = char.max_hp;
                        self.screen = GameScreen::StatAllocation {
                            points_remaining: points - 1,
                        };
                    }
                }
                "2" | "D" => {
                    if points > 0 {
                        char.dexterity += 1;
                        char.max_stamina += 2;
                        char.stamina = char.max_stamina;
                        self.screen = GameScreen::StatAllocation {
                            points_remaining: points - 1,
                        };
                    }
                }
                "3" | "I" => {
                    if points > 0 {
                        char.intelligence += 1;
                        char.max_mana += 2;
                        char.mana = char.max_mana;
                        self.screen = GameScreen::StatAllocation {
                            points_remaining: points - 1,
                        };
                    }
                }
                "F" if points == 0 => {
                    self.screen = GameScreen::WorldView;
                    char.last_message =
                        Some("Welcome to Britannia! Press ? for help.".to_string());
                }
                _ => {}
            }
        }
        UltimoAction::SaveGame
    }

    fn handle_world_view(&mut self, input: &str) -> UltimoAction {
        if let Some(ref mut char) = self.state.character {
            char.last_message = None; // Clear message on action

            match input {
                // Movement
                "W" | "8" => self.move_player(0, -1),
                "S" | "2" => self.move_player(0, 1),
                "A" | "4" => self.move_player(-1, 0),
                "D" | "6" => self.move_player(1, 0),
                // Diagonal movement
                "7" => self.move_player(-1, -1),
                "9" => self.move_player(1, -1),
                "1" => self.move_player(-1, 1),
                "3" => self.move_player(1, 1),
                // Menu options
                "C" => {
                    self.screen = GameScreen::Stats;
                }
                "I" => {
                    self.screen = GameScreen::Inventory;
                }
                "K" => {
                    self.screen = GameScreen::Skills;
                }
                "J" => {
                    self.screen = GameScreen::Quests;
                }
                "P" => {
                    self.screen = GameScreen::Party;
                }
                "L" => {
                    self.screen = GameScreen::Leaderboard;
                }
                "T" => {
                    // Try to talk to nearby NPC
                    self.try_interact_npc();
                }
                "E" => {
                    // Try to enter dungeon/building or use resource
                    self.try_use_environment();
                }
                "F" => {
                    // Fight - look for monster
                    self.try_start_combat();
                }
                "R" => {
                    // Crafting menu
                    self.screen = GameScreen::Crafting;
                }
                "O" => {
                    // Housing menu
                    self.screen = GameScreen::Housing;
                }
                "M" => {
                    // Trade/Market menu
                    self.screen = GameScreen::TradeList;
                }
                "?" | "H" => {
                    if let Some(ref mut ch) = self.state.character {
                        ch.last_message = Some(
                            "WASD=Move C=Stats I=Inv K=Skills J=Quests T=Talk F=Fight R=Craft O=House M=Trade Q=Quit"
                                .to_string(),
                        );
                    }
                }
                "Q" => {
                    self.screen = GameScreen::ConfirmQuit;
                }
                _ => return UltimoAction::Continue,
            }
        }
        UltimoAction::SaveGame
    }

    fn move_player(&mut self, dx: i32, dy: i32) {
        if let Some(ref mut char) = self.state.character {
            let new_x = char.position.x + dx;
            let new_y = char.position.y + dy;

            // Check zone bounds
            if let Some(zone) = get_zone(&char.position.zone) {
                if new_x >= 0
                    && new_x < zone.width as i32
                    && new_y >= 0
                    && new_y < zone.height as i32
                {
                    // Check for zone exits
                    for (exit_zone, exit_x, exit_y) in zone.exits {
                        if new_x == *exit_x && new_y == *exit_y {
                            char.position = Position::new(exit_zone, *exit_x, *exit_y);
                            char.last_message =
                                Some(format!("You enter {}.", get_zone(exit_zone).map(|z| z.name).unwrap_or("unknown")));

                            // Update quest progress for visit quests
                            for quest in char.active_quests.iter_mut() {
                                if let Some(quest_def) = get_quest(&quest.quest_key) {
                                    if quest_def.requirements.visit_zone == Some(exit_zone) {
                                        quest.visited = true;
                                    }
                                }
                            }
                            return;
                        }
                    }

                    // Normal movement
                    char.position.x = new_x;
                    char.position.y = new_y;

                    // Random encounter chance in wilderness/dungeon
                    // Clone zone first to avoid borrow conflict
                    let zone_key = char.position.zone.clone();
                    if let Some(zone) = get_zone(&zone_key) {
                        if matches!(
                            zone.zone_type,
                            super::data::ZoneType::Wilderness | super::data::ZoneType::Dungeon
                        ) {
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            if rng.gen_range(0..100) < 10 {
                                // 10% encounter chance
                                // Return exits the borrow scope, allowing spawn_random_monster to borrow self
                                self.spawn_random_monster(&zone_key);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn try_interact_npc(&mut self) {
        if let Some(ref char) = self.state.character {
            // Find NPC at or near player position
            let npcs = super::data::get_npcs_in_zone(&char.position.zone);
            for npc in npcs {
                let dx = (npc.position.0 - char.position.x).abs();
                let dy = (npc.position.1 - char.position.y).abs();
                if dx <= 1 && dy <= 1 {
                    self.screen = GameScreen::NpcDialogue {
                        npc_key: npc.key.to_string(),
                    };
                    return;
                }
            }
            if let Some(ref mut ch) = self.state.character {
                ch.last_message = Some("No one nearby to talk to.".to_string());
            }
        }
    }

    fn try_use_environment(&mut self) {
        if let Some(ref mut char) = self.state.character {
            // Check for resource nodes
            for node in super::data::RESOURCE_NODES {
                if node.zones.contains(&char.position.zone.as_str()) {
                    let skill_level = char.get_skill(node.required_skill);
                    if skill_level >= node.min_skill {
                        // Check for required tool
                        let has_tool = match node.required_skill {
                            "mining" => char.get_item_count("pickaxe") > 0,
                            "lumberjacking" => char.get_item_count("hatchet") > 0,
                            "fishing" => char.get_item_count("fishing_pole") > 0,
                            _ => true,
                        };

                        if has_tool {
                            // Gather resources
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let mut gathered = Vec::new();

                            for (item, chance, min, max) in node.yields {
                                if rng.gen_range(0..100) < *chance {
                                    let qty = rng.gen_range(*min..=*max);
                                    char.add_item(item, qty);
                                    gathered.push(format!(
                                        "{} {}",
                                        qty,
                                        get_item(item).map(|i| i.name).unwrap_or(item)
                                    ));
                                }
                            }

                            // Skill gain
                            char.try_skill_gain(node.required_skill, node.min_skill);

                            if gathered.is_empty() {
                                char.last_message = Some("You find nothing useful.".to_string());
                            } else {
                                char.last_message =
                                    Some(format!("You gather: {}", gathered.join(", ")));
                            }
                            return;
                        } else {
                            char.last_message = Some(format!(
                                "You need a tool for {}.",
                                node.required_skill
                            ));
                            return;
                        }
                    }
                }
            }
            char.last_message = Some("Nothing to use here.".to_string());
        }
    }

    fn try_start_combat(&mut self) {
        if let Some(ref char) = self.state.character {
            let zone = &char.position.zone;

            // Find a spawnable monster for this zone
            let mut candidates: Vec<_> = super::data::MONSTERS
                .iter()
                .filter(|m| m.spawn_zones.contains(&zone.as_str()))
                .collect();

            if candidates.is_empty() {
                if let Some(ref mut ch) = self.state.character {
                    ch.last_message = Some("Nothing to fight here.".to_string());
                }
                return;
            }

            // Spawn monster
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let template = candidates.remove(rng.gen_range(0..candidates.len()));

            let level = rng.gen_range(template.min_level..=template.max_level);
            let hp_mult = 1.0 + (level as f32 - template.min_level as f32) * 0.1;

            let monster = Monster {
                template_key: template.key.to_string(),
                name: template.name.to_string(),
                level,
                hp: (template.base_hp as f32 * hp_mult) as i32,
                max_hp: (template.base_hp as f32 * hp_mult) as i32,
                damage: template.base_damage + (level as i32 - template.min_level as i32),
                defense: template.base_defense + (level as i32 - template.min_level as i32) / 2,
                position: (char.position.x, char.position.y),
            };

            self.state.combat = Some(CombatState {
                monster,
                player_acted: false,
                combat_log: vec!["Combat begins!".to_string()],
            });
            self.screen = GameScreen::Combat;
        }
    }

    fn spawn_random_monster(&mut self, zone: &str) {
        let candidates: Vec<_> = super::data::MONSTERS
            .iter()
            .filter(|m| m.spawn_zones.contains(&zone) && m.aggressive)
            .collect();

        if candidates.is_empty() {
            return;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let template = candidates[rng.gen_range(0..candidates.len())];

        let level = rng.gen_range(template.min_level..=template.max_level);
        let hp_mult = 1.0 + (level as f32 - template.min_level as f32) * 0.1;

        if let Some(ref char) = self.state.character {
            let monster = Monster {
                template_key: template.key.to_string(),
                name: template.name.to_string(),
                level,
                hp: (template.base_hp as f32 * hp_mult) as i32,
                max_hp: (template.base_hp as f32 * hp_mult) as i32,
                damage: template.base_damage + (level as i32 - template.min_level as i32),
                defense: template.base_defense + (level as i32 - template.min_level as i32) / 2,
                position: (char.position.x, char.position.y),
            };

            self.state.combat = Some(CombatState {
                monster,
                player_acted: false,
                combat_log: vec![format!("A {} attacks you!", template.name)],
            });
            self.screen = GameScreen::Combat;
        }
    }

    fn handle_stats(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
        }
        UltimoAction::SaveGame
    }

    fn handle_inventory(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        // Equip/use item by number
        if let Ok(idx) = input.parse::<usize>() {
            if let Some(ref mut char) = self.state.character {
                if idx > 0 && idx <= char.inventory.len() {
                    let slot = &char.inventory[idx - 1];
                    let item_key = slot.item_key.clone();

                    if let Some(item) = get_item(&item_key) {
                        match item.item_type {
                            super::data::ItemType::Weapon => {
                                char.equipped_weapon = Some(item_key);
                                char.last_message =
                                    Some(format!("You equip the {}.", item.name));
                            }
                            super::data::ItemType::Armor => {
                                char.equipped_armor = Some(item_key);
                                char.last_message =
                                    Some(format!("You put on the {}.", item.name));
                            }
                            super::data::ItemType::Shield => {
                                char.equipped_shield = Some(item_key);
                                char.last_message =
                                    Some(format!("You equip the {}.", item.name));
                            }
                            super::data::ItemType::Consumable => {
                                // Use consumable
                                if item.power > 0 {
                                    if item.name.contains("Heal") || item.name.contains("heal") {
                                        char.hp = (char.hp + item.power).min(char.max_hp);
                                    } else if item.name.contains("Mana") || item.name.contains("mana") {
                                        char.mana = (char.mana + item.power).min(char.max_mana);
                                    }
                                }
                                char.remove_item(&item_key, 1);
                                char.last_message =
                                    Some(format!("You use the {}.", item.name));
                            }
                            _ => {
                                char.last_message =
                                    Some("You can't use that.".to_string());
                            }
                        }
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_skills(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
        }
        UltimoAction::SaveGame
    }

    fn handle_quests(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        // Try to turn in quest by number
        if let Ok(idx) = input.parse::<usize>() {
            if let Some(ref mut char) = self.state.character {
                if idx > 0 && idx <= char.active_quests.len() {
                    let quest_key = char.active_quests[idx - 1].quest_key.clone();
                    if char.can_complete_quest(&quest_key) {
                        if char.complete_quest(&quest_key) {
                            char.last_message =
                                Some(format!("Quest completed! Check your rewards."));
                        }
                    } else {
                        char.last_message =
                            Some("Quest requirements not met.".to_string());
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_combat(&mut self, input: &str) -> UltimoAction {
        let combat_result = if let Some(ref mut combat) = self.state.combat {
            if let Some(ref mut char) = self.state.character {
                match input {
                    "A" | "1" => {
                        // Attack
                        super::combat::player_attack(char, combat)
                    }
                    "C" | "2" => {
                        // Cast spell (simplified - magic arrow)
                        super::combat::player_cast(char, combat, "magic_arrow")
                    }
                    "U" | "3" => {
                        // Use item (heal potion)
                        super::combat::player_use_item(char, combat, "heal_potion")
                    }
                    "R" | "4" => {
                        // Run away
                        super::combat::player_flee(char, combat)
                    }
                    _ => super::combat::CombatResult::Continue,
                }
            } else {
                super::combat::CombatResult::Continue
            }
        } else {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        };

        // Handle combat result
        match combat_result {
            super::combat::CombatResult::Victory => {
                // Award loot and XP
                if let Some(ref mut combat) = self.state.combat {
                    if let Some(template) = get_monster_template(&combat.monster.template_key) {
                        if let Some(ref mut char) = self.state.character {
                            // XP
                            let leveled = char.add_xp(template.xp_reward);
                            if leveled {
                                combat.combat_log.push("You gained a level!".to_string());
                            }

                            // Gold
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            let gold =
                                rng.gen_range(template.gold_drop_min..=template.gold_drop_max);
                            char.gold += gold;

                            // Loot
                            for (item_key, chance) in template.loot_table {
                                if rng.gen_range(0..100) < *chance {
                                    char.add_item(item_key, 1);
                                    combat.combat_log.push(format!(
                                        "Found: {}",
                                        get_item(item_key).map(|i| i.name).unwrap_or(item_key)
                                    ));
                                }
                            }

                            // Update quest progress
                            char.update_quest_kill(&combat.monster.template_key);

                            // Track kills
                            *char
                                .kills
                                .entry(combat.monster.template_key.clone())
                                .or_insert(0) += 1;

                            char.last_message = Some(format!(
                                "Victory! Gained {} XP and {} gold.",
                                template.xp_reward, gold
                            ));
                        }
                    }
                }
                self.state.combat = None;
                self.screen = GameScreen::WorldView;
            }
            super::combat::CombatResult::Defeat => {
                if let Some(ref mut char) = self.state.character {
                    char.die();
                    char.last_message = Some("You have been slain!".to_string());
                }
                self.state.combat = None;
                self.screen = GameScreen::Dead;
            }
            super::combat::CombatResult::Fled => {
                if let Some(ref mut char) = self.state.character {
                    char.last_message = Some("You escaped!".to_string());
                }
                self.state.combat = None;
                self.screen = GameScreen::WorldView;
            }
            super::combat::CombatResult::Continue => {}
        }

        UltimoAction::SaveGame
    }

    fn handle_npc_dialogue(&mut self, input: &str, npc_key: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        if let Some(npc) = get_npc(npc_key) {
            match input {
                "1" | "S" if !npc.shop_inventory.is_empty() => {
                    self.screen = GameScreen::ShopBuy {
                        npc_key: npc_key.to_string(),
                    };
                }
                "2" | "E" if !npc.shop_inventory.is_empty() => {
                    self.screen = GameScreen::ShopSell {
                        npc_key: npc_key.to_string(),
                    };
                }
                "3" | "T" if !npc.trains_skills.is_empty() => {
                    self.screen = GameScreen::Training {
                        npc_key: npc_key.to_string(),
                    };
                }
                "4" | "A" if npc.npc_type == NpcType::QuestGiver => {
                    // Accept available quest
                    if let Some(ref mut char) = self.state.character {
                        for quest in super::data::QUESTS {
                            if quest.giver_npc == npc_key
                                && char.level() >= quest.min_level
                                && !char.completed_quests.contains(&quest.key.to_string())
                                && !char.active_quests.iter().any(|q| q.quest_key == quest.key)
                            {
                                // Check prerequisite
                                if let Some(prereq) = quest.prerequisite {
                                    if !char.completed_quests.contains(&prereq.to_string()) {
                                        continue;
                                    }
                                }

                                char.active_quests.push(QuestProgress::new(quest.key));
                                char.last_message =
                                    Some(format!("Quest accepted: {}", quest.name));
                                break;
                            }
                        }
                    }
                }
                "5" | "B" if npc.npc_type == NpcType::Banker => {
                    self.screen = GameScreen::Bank;
                }
                "6" | "H" if npc.npc_type == NpcType::Healer => {
                    self.screen = GameScreen::Healer;
                }
                _ => {}
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_shop_buy(&mut self, input: &str, npc_key: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::NpcDialogue {
                npc_key: npc_key.to_string(),
            };
            return UltimoAction::SaveGame;
        }

        if let Some(npc) = get_npc(npc_key) {
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= npc.shop_inventory.len() {
                    let (item_key, price_mult) = npc.shop_inventory[idx - 1];
                    if let Some(item) = get_item(item_key) {
                        let price = (item.base_price as f32 * price_mult) as i64;
                        if let Some(ref mut char) = self.state.character {
                            if char.gold >= price {
                                char.gold -= price;
                                if item.item_type == super::data::ItemType::Weapon
                                    || item.item_type == super::data::ItemType::Armor
                                    || item.item_type == super::data::ItemType::Shield
                                {
                                    char.inventory.push(InventoryItem::with_durability(
                                        item_key, 100,
                                    ));
                                } else {
                                    char.add_item(item_key, 1);
                                }
                                char.last_message =
                                    Some(format!("Bought {} for {} gold.", item.name, price));
                            } else {
                                char.last_message = Some("Not enough gold!".to_string());
                            }
                        }
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_shop_sell(&mut self, input: &str, npc_key: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::NpcDialogue {
                npc_key: npc_key.to_string(),
            };
            return UltimoAction::SaveGame;
        }

        if let Ok(idx) = input.parse::<usize>() {
            if let Some(ref mut char) = self.state.character {
                if idx > 0 && idx <= char.inventory.len() {
                    let slot = &char.inventory[idx - 1];
                    if let Some(item) = get_item(&slot.item_key) {
                        let sell_price = item.base_price / 2; // Sell for half
                        let item_name = item.name.to_string();
                        char.gold += sell_price;
                        let item_key = slot.item_key.clone();
                        char.remove_item(&item_key, 1);
                        char.last_message =
                            Some(format!("Sold {} for {} gold.", item_name, sell_price));
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_bank(&mut self, input: &str) -> UltimoAction {
        if let Some(ref mut char) = self.state.character {
            match input {
                "D" | "1" => {
                    // Deposit all
                    char.bank_gold += char.gold;
                    char.last_message = Some(format!("Deposited {} gold.", char.gold));
                    char.gold = 0;
                }
                "W" | "2" => {
                    // Withdraw all
                    char.gold += char.bank_gold;
                    char.last_message = Some(format!("Withdrew {} gold.", char.bank_gold));
                    char.bank_gold = 0;
                }
                "Q" | "B" => {
                    self.screen = GameScreen::WorldView;
                }
                _ => {}
            }
        }
        UltimoAction::SaveGame
    }

    fn handle_healer(&mut self, input: &str) -> UltimoAction {
        if let Some(ref mut char) = self.state.character {
            match input {
                "H" | "1" => {
                    // Heal (10 gold per 10 HP)
                    let missing = char.max_hp - char.hp;
                    let cost = (missing as i64 / 10 + 1) * 10;
                    if char.gold >= cost && missing > 0 {
                        char.gold -= cost;
                        char.hp = char.max_hp;
                        char.last_message = Some(format!("Healed for {} gold.", cost));
                    } else if missing == 0 {
                        char.last_message = Some("You are already at full health.".to_string());
                    } else {
                        char.last_message = Some("Not enough gold!".to_string());
                    }
                }
                "R" | "2" if char.is_dead => {
                    // Resurrect (500 gold)
                    if char.gold >= 500 {
                        char.gold -= 500;
                        char.resurrect();
                        char.last_message = Some("You have been resurrected.".to_string());
                        self.screen = GameScreen::WorldView;
                    } else {
                        char.last_message = Some("Not enough gold!".to_string());
                    }
                }
                "Q" | "B" => {
                    self.screen = GameScreen::WorldView;
                }
                _ => {}
            }
        }
        UltimoAction::SaveGame
    }

    fn handle_training(&mut self, input: &str, npc_key: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::NpcDialogue {
                npc_key: npc_key.to_string(),
            };
            return UltimoAction::SaveGame;
        }

        if let Some(npc) = get_npc(npc_key) {
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= npc.trains_skills.len() {
                    let skill_key = npc.trains_skills[idx - 1];
                    if let Some(ref mut char) = self.state.character {
                        let current = char.get_skill(skill_key);
                        if current >= 100 {
                            char.last_message = Some("Skill already maxed!".to_string());
                        } else {
                            // Training cost: 10 * (current_skill + 1)
                            let cost = 10 * (current as i64 + 1);
                            if char.gold >= cost {
                                char.gold -= cost;
                                char.skills.insert(skill_key.to_string(), current + 1);
                                char.last_message = Some(format!(
                                    "Trained {} to {}. Cost: {} gold.",
                                    skill_key,
                                    current + 1,
                                    cost
                                ));
                            } else {
                                char.last_message =
                                    Some(format!("Need {} gold to train.", cost));
                            }
                        }
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_crafting(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        // Select crafting category by number
        match input {
            "1" => {
                self.screen = GameScreen::CraftingSkill {
                    skill: "blacksmithing".to_string(),
                };
            }
            "2" => {
                self.screen = GameScreen::CraftingSkill {
                    skill: "tailoring".to_string(),
                };
            }
            "3" => {
                self.screen = GameScreen::CraftingSkill {
                    skill: "carpentry".to_string(),
                };
            }
            "4" => {
                self.screen = GameScreen::CraftingSkill {
                    skill: "alchemy".to_string(),
                };
            }
            "5" => {
                self.screen = GameScreen::CraftingSkill {
                    skill: "cooking".to_string(),
                };
            }
            _ => {}
        }

        UltimoAction::SaveGame
    }

    fn handle_crafting_skill(&mut self, input: &str, skill: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Crafting;
            return UltimoAction::SaveGame;
        }

        // Get recipes for this skill
        let recipes: Vec<_> = RECIPES
            .iter()
            .filter(|r| r.required_skill == skill)
            .collect();

        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= recipes.len() {
                let recipe = recipes[idx - 1];
                if let Some(ref mut char) = self.state.character {
                    let result = craft(char, recipe.key);
                    char.last_message = Some(result.message);
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_housing(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        match input {
            "1" | "B" => {
                // Buy house
                self.screen = GameScreen::HousingBuy;
            }
            "2" | "S" => {
                // Storage
                if let Some(ref char) = self.state.character {
                    if char.house_id.is_some() {
                        self.screen = GameScreen::HousingStorage;
                    } else {
                        if let Some(ref mut ch) = self.state.character {
                            ch.last_message = Some("You don't own a house!".to_string());
                        }
                    }
                }
            }
            "3" | "F" => {
                // Friends list / access control
                if let Some(ref mut char) = self.state.character {
                    if char.house_id.is_some() {
                        char.last_message = Some("Access control coming soon!".to_string());
                    } else {
                        char.last_message = Some("You don't own a house!".to_string());
                    }
                }
            }
            _ => {}
        }

        UltimoAction::SaveGame
    }

    fn handle_housing_buy(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Housing;
            return UltimoAction::SaveGame;
        }

        let house_types = [
            HouseType::SmallCottage,
            HouseType::MediumHouse,
            HouseType::LargeHouse,
            HouseType::Tower,
            HouseType::Castle,
        ];

        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= house_types.len() {
                let house_type = house_types[idx - 1];
                if let Some(ref mut char) = self.state.character {
                    match can_purchase_house(char, house_type) {
                        Ok(()) => {
                            // Would actually purchase - for now just set a pending state
                            // The actual purchase happens in service layer with DB
                            char.gold -= house_type.price();
                            char.house_id = Some(1); // Placeholder
                            char.last_message = Some(format!(
                                "You purchased a {}!",
                                house_type.name()
                            ));
                            self.screen = GameScreen::Housing;
                        }
                        Err(msg) => {
                            char.last_message = Some(msg);
                        }
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_housing_storage(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Housing;
            return UltimoAction::SaveGame;
        }

        // D = Deposit item from inventory to house
        // W = Withdraw item from house to inventory
        if let Some(ref mut char) = self.state.character {
            match input {
                "D" => {
                    char.last_message = Some("Enter item number to deposit...".to_string());
                }
                "W" => {
                    char.last_message = Some("Storage access coming soon!".to_string());
                }
                _ => {
                    // Try to deposit item by number
                    if let Ok(idx) = input.parse::<usize>() {
                        if idx > 0 && idx <= char.inventory.len() {
                            let slot = &char.inventory[idx - 1];
                            let item_key = slot.item_key.clone();
                            let item_name = get_item(&item_key)
                                .map(|i| i.name.to_string())
                                .unwrap_or_else(|| item_key.clone());
                            char.remove_item(&item_key, 1);
                            char.last_message = Some(format!("Stored {} in your house.", item_name));
                        }
                    }
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_trade_list(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        match input {
            "C" | "1" => {
                // Create new trade offer
                self.screen = GameScreen::TradeCreate;
            }
            _ => {
                // Browse offers - would need DB integration
                if let Some(ref mut char) = self.state.character {
                    char.last_message = Some("No offers available.".to_string());
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_trade_create(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::TradeList;
            return UltimoAction::SaveGame;
        }

        // Select item from inventory to sell
        if let Ok(idx) = input.parse::<usize>() {
            if let Some(ref mut char) = self.state.character {
                if idx > 0 && idx <= char.inventory.len() {
                    let slot = &char.inventory[idx - 1];
                    let item_name = get_item(&slot.item_key)
                        .map(|i| i.name.to_string())
                        .unwrap_or_else(|| slot.item_key.clone());
                    char.last_message = Some(format!(
                        "Trade listing created for {}. Other players can now buy it!",
                        item_name
                    ));
                    // Would actually create offer in DB
                    self.screen = GameScreen::TradeList;
                }
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_trade(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        // Direct player-to-player trade
        if let Some(ref mut char) = self.state.character {
            match input {
                "A" | "1" => {
                    char.last_message = Some("Trade accepted!".to_string());
                    self.screen = GameScreen::WorldView;
                }
                "D" | "2" => {
                    char.last_message = Some("Trade declined.".to_string());
                    self.screen = GameScreen::WorldView;
                }
                _ => {}
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_party(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::WorldView;
            return UltimoAction::SaveGame;
        }

        if let Some(ref mut char) = self.state.character {
            match input {
                "C" | "1" => {
                    // Create party
                    char.last_message = Some("Party created! Invite nearby players to join.".to_string());
                }
                "I" | "2" => {
                    // Invite nearby player
                    if !self.state.visible_players.is_empty() {
                        let nearby = &self.state.visible_players[0];
                        char.last_message = Some(format!("Invited {} to your party!", nearby.name));
                    } else {
                        char.last_message = Some("No players nearby to invite.".to_string());
                    }
                }
                "L" | "3" => {
                    // Leave party
                    char.last_message = Some("You left the party.".to_string());
                }
                _ => {}
            }
        }

        UltimoAction::SaveGame
    }

    fn handle_leaderboard(&mut self, input: &str) -> UltimoAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::WorldView;
        }
        UltimoAction::SaveGame
    }

    fn handle_dead(&mut self, input: &str) -> UltimoAction {
        match input {
            "R" | "1" => {
                // Try to resurrect at healer
                if let Some(ref mut char) = self.state.character {
                    if char.gold >= 500 {
                        char.gold -= 500;
                        char.resurrect();
                        char.position = Position::default(); // Return to Britain
                        self.screen = GameScreen::WorldView;
                    } else {
                        char.last_message = Some("Not enough gold to resurrect!".to_string());
                    }
                }
            }
            "Q" => {
                self.screen = GameScreen::ConfirmQuit;
            }
            _ => {}
        }
        UltimoAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> UltimoAction {
        match input {
            "Y" => UltimoAction::Quit,
            _ => {
                self.screen = if self
                    .state
                    .character
                    .as_ref()
                    .map(|c| c.is_dead)
                    .unwrap_or(false)
                {
                    GameScreen::Dead
                } else {
                    GameScreen::WorldView
                };
                UltimoAction::SaveGame
            }
        }
    }
}

impl Default for UltimoFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_starts_at_intro() {
        let flow = UltimoFlow::new();
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_intro_to_character_create() {
        let mut flow = UltimoFlow::new();
        flow.handle_char('\r'); // Press enter
        assert!(matches!(flow.screen, GameScreen::CharacterCreate));
    }

    #[test]
    fn test_character_creation() {
        let mut flow = UltimoFlow::new();
        flow.screen = GameScreen::CharacterCreate;

        // Type name
        for c in "TestHero".chars() {
            flow.handle_char(c);
        }
        flow.handle_char('\r');

        assert!(matches!(
            flow.screen,
            GameScreen::StatAllocation { points_remaining: 15 }
        ));
        assert!(flow.state.character.is_some());
        assert_eq!(flow.state.character.as_ref().unwrap().name, "TESTHERO");
    }

    #[test]
    fn test_stat_allocation() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Test", 1));
        flow.screen = GameScreen::StatAllocation {
            points_remaining: 3,
        };

        let initial_str = flow.state.character.as_ref().unwrap().strength;

        flow.handle_char('1'); // Add strength
        assert_eq!(
            flow.state.character.as_ref().unwrap().strength,
            initial_str + 1
        );

        if let GameScreen::StatAllocation { points_remaining } = flow.screen {
            assert_eq!(points_remaining, 2);
        } else {
            panic!("Expected StatAllocation screen");
        }
    }

    #[test]
    fn test_movement() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Test", 1));
        flow.screen = GameScreen::WorldView;

        let start_x = flow.state.character.as_ref().unwrap().position.x;

        flow.handle_char('D'); // Move right
        assert_eq!(
            flow.state.character.as_ref().unwrap().position.x,
            start_x + 1
        );
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Test", 1));
        flow.screen = GameScreen::WorldView;

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::WorldView));
    }

    #[test]
    fn test_crafting_menu_navigation() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Crafter", 1));
        flow.screen = GameScreen::WorldView;

        flow.handle_char('R'); // Open crafting
        assert!(matches!(flow.screen, GameScreen::Crafting));

        flow.handle_char('1'); // Select blacksmithing
        assert!(matches!(
            &flow.screen,
            GameScreen::CraftingSkill { skill } if skill == "blacksmithing"
        ));

        flow.handle_char('B'); // Back to crafting menu
        assert!(matches!(flow.screen, GameScreen::Crafting));

        flow.handle_char('Q'); // Back to world
        assert!(matches!(flow.screen, GameScreen::WorldView));
    }

    #[test]
    fn test_housing_menu_navigation() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Homeowner", 1));
        flow.screen = GameScreen::WorldView;

        flow.handle_char('O'); // Open housing
        assert!(matches!(flow.screen, GameScreen::Housing));

        flow.handle_char('1'); // Buy house option
        assert!(matches!(flow.screen, GameScreen::HousingBuy));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::Housing));

        flow.handle_char('Q'); // Back to world
        assert!(matches!(flow.screen, GameScreen::WorldView));
    }

    #[test]
    fn test_trade_menu_navigation() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Trader", 1));
        flow.screen = GameScreen::WorldView;

        flow.handle_char('M'); // Open trade
        assert!(matches!(flow.screen, GameScreen::TradeList));

        flow.handle_char('C'); // Create listing
        assert!(matches!(flow.screen, GameScreen::TradeCreate));

        flow.handle_char('B'); // Back
        assert!(matches!(flow.screen, GameScreen::TradeList));

        flow.handle_char('Q'); // Back to world
        assert!(matches!(flow.screen, GameScreen::WorldView));
    }

    #[test]
    fn test_party_menu() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Leader", 1));
        flow.screen = GameScreen::WorldView;

        flow.handle_char('P'); // Open party
        assert!(matches!(flow.screen, GameScreen::Party));

        flow.handle_char('C'); // Create party
        assert!(flow.state.character.as_ref().unwrap().last_message.is_some());
        assert!(flow.state.character.as_ref().unwrap().last_message.as_ref().unwrap().contains("Party created"));

        flow.handle_char('Q'); // Back to world
        assert!(matches!(flow.screen, GameScreen::WorldView));
    }

    #[test]
    fn test_crafting_skill_categories() {
        let mut flow = UltimoFlow::new();
        flow.state.character = Some(super::super::state::Character::new("Test", 1));
        flow.screen = GameScreen::Crafting;

        // Test each crafting category
        flow.handle_char('2'); // Tailoring
        assert!(matches!(
            flow.screen,
            GameScreen::CraftingSkill { skill } if skill == "tailoring"
        ));

        flow.screen = GameScreen::Crafting;
        flow.handle_char('3'); // Carpentry
        assert!(matches!(
            flow.screen,
            GameScreen::CraftingSkill { skill } if skill == "carpentry"
        ));

        flow.screen = GameScreen::Crafting;
        flow.handle_char('4'); // Alchemy
        assert!(matches!(
            flow.screen,
            GameScreen::CraftingSkill { skill } if skill == "alchemy"
        ));

        flow.screen = GameScreen::Crafting;
        flow.handle_char('5'); // Cooking
        assert!(matches!(
            flow.screen,
            GameScreen::CraftingSkill { skill } if skill == "cooking"
        ));
    }

    #[test]
    fn test_housing_purchase_insufficient_gold() {
        let mut flow = UltimoFlow::new();
        let mut char = super::super::state::Character::new("Poor", 1);
        char.gold = 100; // Not enough for any house
        flow.state.character = Some(char);
        flow.screen = GameScreen::HousingBuy;

        flow.handle_char('1'); // Try to buy small cottage (10000 gold)

        // Should show error message, stay on screen
        assert!(flow.state.character.as_ref().unwrap().last_message.is_some());
        assert!(flow.state.character.as_ref().unwrap().last_message.as_ref().unwrap().contains("Not enough gold"));
    }

    #[test]
    fn test_housing_purchase_success() {
        let mut flow = UltimoFlow::new();
        let mut char = super::super::state::Character::new("Rich", 1);
        char.gold = 50000; // Enough for small cottage
        flow.state.character = Some(char);
        flow.screen = GameScreen::HousingBuy;

        flow.handle_char('1'); // Buy small cottage

        // Should succeed
        assert!(flow.state.character.as_ref().unwrap().last_message.is_some());
        assert!(flow.state.character.as_ref().unwrap().last_message.as_ref().unwrap().contains("purchased"));
        assert_eq!(flow.state.character.as_ref().unwrap().gold, 40000); // Paid 10000
        assert!(flow.state.character.as_ref().unwrap().house_id.is_some());
    }

    #[test]
    fn test_storage_requires_house() {
        let mut flow = UltimoFlow::new();
        let char = super::super::state::Character::new("Homeless", 1);
        flow.state.character = Some(char);
        flow.screen = GameScreen::Housing;

        flow.handle_char('2'); // Try to access storage

        // Should show error
        assert!(flow.state.character.as_ref().unwrap().last_message.is_some());
        assert!(flow.state.character.as_ref().unwrap().last_message.as_ref().unwrap().contains("don't own a house"));
    }
}
