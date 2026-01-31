//! Screen state machine for Usurper
//!
//! Handles all game screens and input processing.

use super::state::GameState;
use super::data::CharacterClass;
use super::combat;

/// All possible game screens
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Initial intro/splash screen
    Intro,
    /// Character creation - name entry
    CharacterCreation { stage: CreationStage },
    /// Main town hub
    Town,
    /// Dungeon exploration
    Dungeon { combat_state: Option<CombatState> },
    /// Town shop
    Shop { shop_type: ShopType },
    /// Healer/temple
    Healer,
    /// Bank
    Bank,
    /// Substance dealer (steroids/drugs)
    SubstanceDealer,
    /// View/manage equipment
    Equipment,
    /// View character stats
    Stats,
    /// Romance options
    Romance,
    /// Clan management
    Clan,
    /// PvP arena
    Arena { opponent: Option<ArenaOpponent> },
    /// King's throne (if eligible)
    Throne,
    /// Quest log
    Quests,
    /// Leaderboard
    Leaderboard,
    /// Game over
    GameOver,
    /// Confirm quit
    ConfirmQuit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreationStage {
    Name { buffer: String },
    Class,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShopType {
    Weapons,
    Armor,
    General,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatState {
    pub monster_key: String,
    pub monster_hp: u32,
    pub monster_max_hp: u32,
    pub round: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArenaOpponent {
    pub user_id: i64,
    pub name: String,
    pub level: u32,
    pub hp: u32,
}

/// Actions returned by UsurperFlow
#[derive(Debug, Clone)]
pub enum UsurperAction {
    /// No output needed
    Continue,
    /// Render screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game over
    GameOver { final_level: u32, supreme_defeated: bool },
    /// Quit to main menu
    Quit,
}

/// Game flow state machine
pub struct UsurperFlow {
    pub state: GameState,
    pub screen: GameScreen,
    input_buffer: String,
}

impl UsurperFlow {
    /// Create new game (character creation)
    pub fn new() -> Self {
        Self {
            state: GameState::new("".to_string(), CharacterClass::Warrior),
            screen: GameScreen::Intro,
            input_buffer: String::new(),
        }
    }

    /// Resume from loaded state
    pub fn from_state(state: GameState) -> Self {
        Self {
            state,
            screen: GameScreen::Intro,
            input_buffer: String::new(),
        }
    }

    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> UsurperAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return UsurperAction::Echo("\x08 \x08".to_string());
            }
            return UsurperAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return UsurperAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input for text entry
        if self.input_buffer.len() < 30 {
            self.input_buffer.push(ch);
            return UsurperAction::Echo(ch.to_string());
        }

        UsurperAction::Continue
    }

    fn is_single_key_screen(&self) -> bool {
        !matches!(
            self.screen,
            GameScreen::CharacterCreation { stage: CreationStage::Name { .. } }
        )
    }

    fn process_input(&mut self) -> UsurperAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen.clone() {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::CharacterCreation { stage } => self.handle_creation(&input, stage.clone()),
            GameScreen::Town => self.handle_town(&input),
            GameScreen::Dungeon { combat_state } => self.handle_dungeon(&input, combat_state.clone()),
            GameScreen::Shop { shop_type } => self.handle_shop(&input, shop_type.clone()),
            GameScreen::Healer => self.handle_healer(&input),
            GameScreen::Bank => self.handle_bank(&input),
            GameScreen::SubstanceDealer => self.handle_substance(&input),
            GameScreen::Equipment => self.handle_equipment(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Romance => self.handle_romance(&input),
            GameScreen::Clan => self.handle_clan(&input),
            GameScreen::Arena { opponent } => self.handle_arena(&input, opponent.clone()),
            GameScreen::Throne => self.handle_throne(&input),
            GameScreen::Quests => self.handle_quests(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> UsurperAction {
        // Check if this is a new character or resuming
        if self.state.character_name.is_empty() {
            self.screen = GameScreen::CharacterCreation {
                stage: CreationStage::Name { buffer: String::new() },
            };
        } else {
            // Resume existing game
            self.state.check_new_day();
            self.screen = GameScreen::Town;
        }
        UsurperAction::SaveGame
    }

    fn handle_creation(&mut self, input: &str, stage: CreationStage) -> UsurperAction {
        match stage {
            CreationStage::Name { .. } => {
                if input.len() >= 2 && input.len() <= 20 {
                    self.state.character_name = input.to_string();
                    self.screen = GameScreen::CharacterCreation {
                        stage: CreationStage::Class,
                    };
                    UsurperAction::SaveGame
                } else {
                    self.state.last_message = Some("Name must be 2-20 characters.".to_string());
                    UsurperAction::SaveGame
                }
            }
            CreationStage::Class => {
                let class = match input {
                    "1" => Some(CharacterClass::Warrior),
                    "2" => Some(CharacterClass::Rogue),
                    "3" => Some(CharacterClass::Mage),
                    "4" => Some(CharacterClass::Cleric),
                    "5" => Some(CharacterClass::Berserker),
                    _ => None,
                };

                if let Some(c) = class {
                    // Reinitialize state with proper class
                    let name = self.state.character_name.clone();
                    self.state = GameState::new(name, c);
                    self.state.check_new_day();
                    self.screen = GameScreen::Town;
                    UsurperAction::SaveGame
                } else {
                    UsurperAction::Continue
                }
            }
        }
    }

    fn handle_town(&mut self, input: &str) -> UsurperAction {
        self.state.last_message = None;

        match input {
            "D" => {
                // Enter dungeon
                if self.state.turns_remaining > 0 {
                    self.state.in_town = false;
                    self.screen = GameScreen::Dungeon { combat_state: None };
                    UsurperAction::SaveGame
                } else {
                    self.state.last_message = Some("No turns remaining today!".to_string());
                    UsurperAction::SaveGame
                }
            }
            "W" => {
                self.screen = GameScreen::Shop { shop_type: ShopType::Weapons };
                UsurperAction::SaveGame
            }
            "A" => {
                self.screen = GameScreen::Shop { shop_type: ShopType::Armor };
                UsurperAction::SaveGame
            }
            "S" => {
                self.screen = GameScreen::Shop { shop_type: ShopType::General };
                UsurperAction::SaveGame
            }
            "H" => {
                self.screen = GameScreen::Healer;
                UsurperAction::SaveGame
            }
            "B" => {
                self.screen = GameScreen::Bank;
                UsurperAction::SaveGame
            }
            "P" => {
                self.screen = GameScreen::SubstanceDealer;
                UsurperAction::SaveGame
            }
            "E" => {
                self.screen = GameScreen::Equipment;
                UsurperAction::SaveGame
            }
            "C" => {
                self.screen = GameScreen::Stats;
                UsurperAction::SaveGame
            }
            "R" => {
                self.screen = GameScreen::Romance;
                UsurperAction::SaveGame
            }
            "T" => {
                self.screen = GameScreen::Clan;
                UsurperAction::SaveGame
            }
            "V" => {
                self.screen = GameScreen::Arena { opponent: None };
                UsurperAction::SaveGame
            }
            "K" if self.state.level >= 50 || self.state.is_king => {
                self.screen = GameScreen::Throne;
                UsurperAction::SaveGame
            }
            "Q" => {
                self.screen = GameScreen::Quests;
                UsurperAction::SaveGame
            }
            "L" => {
                self.screen = GameScreen::Leaderboard;
                UsurperAction::SaveGame
            }
            "X" => {
                self.screen = GameScreen::ConfirmQuit;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_dungeon(&mut self, input: &str, combat_state: Option<CombatState>) -> UsurperAction {
        if let Some(mut combat) = combat_state {
            // In combat
            return self.handle_combat(input, &mut combat);
        }

        match input {
            "E" => {
                // Explore deeper
                if !self.state.use_turn() {
                    self.state.last_message = Some("No turns remaining!".to_string());
                    self.screen = GameScreen::Town;
                    self.state.in_town = true;
                    return UsurperAction::SaveGame;
                }

                // Random encounter
                if rand::random::<u32>() % 100 < 70 {
                    let monster = combat::get_random_monster(self.state.current_dungeon_level);
                    let monster_hp = combat::calculate_monster_hp(monster, self.state.current_dungeon_level);
                    self.screen = GameScreen::Dungeon {
                        combat_state: Some(CombatState {
                            monster_key: monster.key.to_string(),
                            monster_hp,
                            monster_max_hp: monster_hp,
                            round: 1,
                        }),
                    };
                } else {
                    // Found treasure
                    let gold = (self.state.current_dungeon_level as u64 * 10) + rand::random::<u64>() % 50;
                    self.state.add_gold(gold);
                    self.state.last_message = Some(format!("You found {} gold!", gold));
                }
                UsurperAction::SaveGame
            }
            "D" => {
                // Go deeper
                if self.state.current_dungeon_level < 101 {
                    let min_level_for_next = (self.state.current_dungeon_level as u32) / 2;
                    if self.state.level >= min_level_for_next || self.state.current_dungeon_level < 5 {
                        self.state.current_dungeon_level += 1;
                        if self.state.current_dungeon_level > self.state.deepest_dungeon {
                            self.state.deepest_dungeon = self.state.current_dungeon_level;
                        }
                        self.state.last_message = Some(format!(
                            "Descended to level {}...",
                            self.state.current_dungeon_level
                        ));
                    } else {
                        self.state.last_message = Some(format!(
                            "You need level {} to go deeper!",
                            min_level_for_next
                        ));
                    }
                }
                UsurperAction::SaveGame
            }
            "U" => {
                // Go up
                if self.state.current_dungeon_level > 1 {
                    self.state.current_dungeon_level -= 1;
                    self.state.last_message = Some(format!(
                        "Ascended to level {}...",
                        self.state.current_dungeon_level
                    ));
                }
                UsurperAction::SaveGame
            }
            "T" => {
                // Return to town
                self.state.in_town = true;
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            "I" => {
                // Use item/substance
                self.screen = GameScreen::SubstanceDealer;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_combat(&mut self, input: &str, combat: &mut CombatState) -> UsurperAction {
        let monster = super::data::get_monster(&combat.monster_key)
            .expect("Invalid monster key");

        match input {
            "A" => {
                // Attack
                let result = combat::resolve_combat_round(&self.state, monster, combat.monster_hp);

                combat.monster_hp = combat.monster_hp.saturating_sub(result.damage_to_monster);
                self.state.take_damage(result.damage_to_player);
                combat.round += 1;

                if combat.monster_hp == 0 {
                    // Victory!
                    let xp = combat::calculate_xp_reward(monster, self.state.level);
                    let gold = combat::calculate_gold_reward(monster);
                    let leveled_up = self.state.add_experience(xp);
                    self.state.add_gold(gold);
                    self.state.monsters_killed += 1;

                    let mut msg = format!(
                        "Victory! {} slain. +{} XP, +{} gold.",
                        monster.name, xp, gold
                    );
                    if leveled_up {
                        msg.push_str(&format!(" LEVEL UP! Now level {}!", self.state.level));
                    }
                    self.state.last_message = Some(msg);

                    self.screen = GameScreen::Dungeon { combat_state: None };
                } else {
                    // Combat continues
                    self.state.last_message = Some(format!(
                        "You deal {} damage! {} deals {} damage to you.",
                        result.damage_to_monster, monster.name, result.damage_to_player
                    ));
                    self.screen = GameScreen::Dungeon {
                        combat_state: Some(combat.clone()),
                    };
                }
                UsurperAction::SaveGame
            }
            "D" => {
                // Defend (reduced damage, slight counter)
                let result = combat::resolve_defend(&self.state, monster, combat.monster_hp);
                self.state.take_damage(result.damage_to_player / 2);
                combat.monster_hp = combat.monster_hp.saturating_sub(result.damage_to_monster / 3);
                combat.round += 1;

                self.state.last_message = Some(format!(
                    "You defend! Took {} damage (reduced).",
                    result.damage_to_player / 2
                ));
                self.screen = GameScreen::Dungeon {
                    combat_state: Some(combat.clone()),
                };
                UsurperAction::SaveGame
            }
            "R" => {
                // Run
                let escape_chance = 30 + (self.state.agility as i32 - monster.base_damage as i32).max(0) as u32;
                if rand::random::<u32>() % 100 < escape_chance {
                    self.state.last_message = Some("You escaped!".to_string());
                    self.screen = GameScreen::Dungeon { combat_state: None };
                } else {
                    // Failed to run, take damage
                    let damage = monster.base_damage / 2;
                    self.state.take_damage(damage);
                    self.state.last_message = Some(format!(
                        "Failed to escape! Took {} damage.",
                        damage
                    ));
                    self.screen = GameScreen::Dungeon {
                        combat_state: Some(combat.clone()),
                    };
                }
                UsurperAction::SaveGame
            }
            "S" => {
                // Use skill (class-specific)
                let result = combat::resolve_skill(&self.state, monster, combat.monster_hp);
                combat.monster_hp = combat.monster_hp.saturating_sub(result.damage_to_monster);
                self.state.take_damage(result.damage_to_player);

                self.state.last_message = Some(result.message);

                if combat.monster_hp == 0 {
                    let xp = combat::calculate_xp_reward(monster, self.state.level);
                    let gold = combat::calculate_gold_reward(monster);
                    self.state.add_experience(xp);
                    self.state.add_gold(gold);
                    self.state.monsters_killed += 1;
                    self.screen = GameScreen::Dungeon { combat_state: None };
                } else {
                    self.screen = GameScreen::Dungeon {
                        combat_state: Some(combat.clone()),
                    };
                }
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_shop(&mut self, input: &str, shop_type: ShopType) -> UsurperAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Town;
            return UsurperAction::SaveGame;
        }

        // Get items for this shop type
        let items: Vec<_> = super::data::EQUIPMENT_ITEMS.iter()
            .filter(|item| {
                match shop_type {
                    ShopType::Weapons => item.slot == super::data::EquipmentSlot::Weapon,
                    ShopType::Armor => matches!(item.slot,
                        super::data::EquipmentSlot::Armor |
                        super::data::EquipmentSlot::Helmet |
                        super::data::EquipmentSlot::Shield |
                        super::data::EquipmentSlot::Boots |
                        super::data::EquipmentSlot::Gloves
                    ),
                    ShopType::General => matches!(item.slot,
                        super::data::EquipmentSlot::RingLeft |
                        super::data::EquipmentSlot::RingRight |
                        super::data::EquipmentSlot::Amulet |
                        super::data::EquipmentSlot::Cloak
                    ),
                }
            })
            .filter(|item| item.min_level <= self.state.level + 10)
            .collect();

        // Parse item number
        if let Ok(idx) = input.parse::<usize>() {
            if idx >= 1 && idx <= items.len() {
                let item = items[idx - 1];
                if self.state.spend_gold(item.price as u64) {
                    // Add to inventory
                    *self.state.inventory.entry(item.key.to_string()).or_insert(0) += 1;
                    self.state.last_message = Some(format!("Purchased {}!", item.name));
                } else {
                    self.state.last_message = Some("Not enough gold!".to_string());
                }
            }
        }

        UsurperAction::SaveGame
    }

    fn handle_healer(&mut self, input: &str) -> UsurperAction {
        match input {
            "H" => {
                // Full heal
                let cost = ((self.state.max_hp - self.state.hp) as u64 * 2).max(10);
                if self.state.spend_gold(cost) {
                    self.state.hp = self.state.max_hp;
                    self.state.last_message = Some("You feel restored!".to_string());
                } else {
                    self.state.last_message = Some("Not enough gold!".to_string());
                }
                UsurperAction::SaveGame
            }
            "M" => {
                // Mental restoration
                let cost = ((self.state.max_mental_stability - self.state.mental_stability).abs() as u64 * 5).max(20);
                if self.state.spend_gold(cost) {
                    self.state.mental_stability = self.state.max_mental_stability;
                    self.state.last_message = Some("Your mind is clear once more.".to_string());
                } else {
                    self.state.last_message = Some("Not enough gold!".to_string());
                }
                UsurperAction::SaveGame
            }
            "C" => {
                // Cure addiction (expensive)
                let total_addiction: u32 = self.state.addictions.values().sum();
                if total_addiction > 0 {
                    let cost = (total_addiction as u64) * 100;
                    if self.state.spend_gold(cost) {
                        self.state.addictions.clear();
                        self.state.last_message = Some("All addictions purged!".to_string());
                    } else {
                        self.state.last_message = Some("Not enough gold!".to_string());
                    }
                } else {
                    self.state.last_message = Some("You have no addictions.".to_string());
                }
                UsurperAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_bank(&mut self, input: &str) -> UsurperAction {
        match input {
            "D" => {
                // Deposit all
                self.state.bank_gold += self.state.gold;
                self.state.gold = 0;
                self.state.last_message = Some("Deposited all gold.".to_string());
                UsurperAction::SaveGame
            }
            "W" => {
                // Withdraw all
                self.state.gold += self.state.bank_gold;
                self.state.bank_gold = 0;
                self.state.last_message = Some("Withdrew all gold.".to_string());
                UsurperAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_substance(&mut self, input: &str) -> UsurperAction {
        if input == "Q" || input == "B" {
            if !self.state.in_town {
                self.screen = GameScreen::Dungeon { combat_state: None };
            } else {
                self.screen = GameScreen::Town;
            }
            return UsurperAction::SaveGame;
        }

        // Parse substance number
        if let Ok(idx) = input.parse::<usize>() {
            let substances: Vec<_> = super::data::SUBSTANCES.iter().collect();
            if idx >= 1 && idx <= substances.len() {
                let substance = substances[idx - 1];

                // Check if we have it in inventory or need to buy
                let in_inventory = self.state.inventory.get(substance.key).copied().unwrap_or(0);

                if in_inventory > 0 {
                    // Use from inventory
                    *self.state.inventory.get_mut(substance.key).unwrap() -= 1;
                    match self.state.apply_substance(substance.key) {
                        Ok(msg) => self.state.last_message = Some(msg),
                        Err(msg) => self.state.last_message = Some(msg),
                    }
                } else if self.state.spend_gold(substance.price as u64) {
                    // Buy and use
                    match self.state.apply_substance(substance.key) {
                        Ok(msg) => self.state.last_message = Some(msg),
                        Err(msg) => self.state.last_message = Some(msg),
                    }
                } else {
                    self.state.last_message = Some("Not enough gold!".to_string());
                }
            }
        }

        UsurperAction::SaveGame
    }

    fn handle_equipment(&mut self, input: &str) -> UsurperAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Town;
            return UsurperAction::SaveGame;
        }

        // Parse slot number to equip from inventory
        if let Ok(idx) = input.parse::<usize>() {
            let inventory_items: Vec<_> = self.state.inventory.iter()
                .filter(|(_, count)| **count > 0)
                .map(|(k, _)| k.clone())
                .collect();

            if idx >= 1 && idx <= inventory_items.len() {
                let item_key = &inventory_items[idx - 1];
                if let Some(item) = super::data::get_equipment(item_key) {
                    // Equip the item
                    let old_item = self.state.equipment.get(item.slot).clone();
                    self.state.equipment.set(item.slot, Some(item_key.clone()));

                    // Remove from inventory
                    if let Some(count) = self.state.inventory.get_mut(item_key) {
                        *count -= 1;
                    }

                    // Return old item to inventory
                    if let Some(old_key) = old_item {
                        *self.state.inventory.entry(old_key).or_insert(0) += 1;
                    }

                    self.state.last_message = Some(format!("Equipped {}!", item.name));
                }
            }
        }

        UsurperAction::SaveGame
    }

    fn handle_stats(&mut self, _input: &str) -> UsurperAction {
        self.screen = GameScreen::Town;
        UsurperAction::SaveGame
    }

    fn handle_romance(&mut self, input: &str) -> UsurperAction {
        // Romance handling - simplified for now
        match input {
            "F" => {
                // Flirt (would need other players)
                self.state.last_message = Some("No eligible partners nearby...".to_string());
                UsurperAction::SaveGame
            }
            "D" if self.state.romance_status.relationship_level > 0 => {
                // Divorce
                self.state.romance_status = super::state::RomanceStatus::default();
                self.state.last_message = Some("Your relationship has ended.".to_string());
                UsurperAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_clan(&mut self, input: &str) -> UsurperAction {
        match input {
            "C" if self.state.clan_id.is_none() => {
                // Create clan
                if self.state.spend_gold(1000) {
                    self.state.clan_id = Some(format!("clan_{}", rand::random::<u32>()));
                    self.state.clan_role = Some("Leader".to_string());
                    self.state.last_message = Some("Clan created!".to_string());
                } else {
                    self.state.last_message = Some("Need 1000 gold to create a clan!".to_string());
                }
                UsurperAction::SaveGame
            }
            "L" if self.state.clan_id.is_some() => {
                // Leave clan
                self.state.clan_id = None;
                self.state.clan_role = None;
                self.state.last_message = Some("You left your clan.".to_string());
                UsurperAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_arena(&mut self, input: &str, _opponent: Option<ArenaOpponent>) -> UsurperAction {
        // Arena/PvP - simplified, would need real opponent data
        match input {
            "Q" | "B" => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            _ => {
                self.state.last_message = Some("No opponents available...".to_string());
                UsurperAction::SaveGame
            }
        }
    }

    fn handle_throne(&mut self, input: &str) -> UsurperAction {
        match input {
            "C" if !self.state.is_king && self.state.level >= 50 => {
                // Challenge for throne
                // Would involve PvP with current king
                self.state.last_message = Some("The throne is currently empty. You claim it!".to_string());
                self.state.is_king = true;
                UsurperAction::SaveGame
            }
            "A" if self.state.is_king && self.state.supreme_being_defeated => {
                // Ascend to godhood
                self.state.godhood_level += 1;
                self.state.last_message = Some(format!(
                    "You ascend to godhood level {}!",
                    self.state.godhood_level
                ));
                UsurperAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
            _ => UsurperAction::Continue,
        }
    }

    fn handle_quests(&mut self, _input: &str) -> UsurperAction {
        self.screen = GameScreen::Town;
        UsurperAction::SaveGame
    }

    fn handle_leaderboard(&mut self, _input: &str) -> UsurperAction {
        self.screen = GameScreen::Town;
        UsurperAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> UsurperAction {
        UsurperAction::GameOver {
            final_level: self.state.level,
            supreme_defeated: self.state.supreme_being_defeated,
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> UsurperAction {
        match input {
            "Y" => UsurperAction::Quit,
            _ => {
                self.screen = GameScreen::Town;
                UsurperAction::SaveGame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_flow() {
        let flow = UsurperFlow::new();
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_character_creation() {
        let mut flow = UsurperFlow::new();
        flow.handle_char('\r'); // Skip intro
        assert!(matches!(
            flow.screen,
            GameScreen::CharacterCreation { stage: CreationStage::Name { .. } }
        ));
    }

    #[test]
    fn test_town_navigation() {
        let mut flow = UsurperFlow::from_state(GameState::new("Test".to_string(), CharacterClass::Warrior));
        flow.screen = GameScreen::Town;

        flow.handle_char('D');
        assert!(matches!(flow.screen, GameScreen::Dungeon { .. }));
    }

    #[test]
    fn test_combat_state() {
        let combat = CombatState {
            monster_key: "goblin".to_string(),
            monster_hp: 20,
            monster_max_hp: 20,
            round: 1,
        };
        assert_eq!(combat.monster_hp, 20);
    }
}
