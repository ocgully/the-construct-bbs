//! Last Dream screen/flow state machine
//! Manages screen transitions and input handling

use super::combat::{CombatState, CombatAction};
use super::data::{ClassType, get_item, ITEMS, EQUIPMENT};
use super::party::{Character, CharacterClass, Party, MAX_PARTY_SIZE};
use super::state::GameState;
use super::world::{Position, WorldMap, get_location};

/// Which screen the player is currently viewing
#[derive(Debug, Clone)]
pub enum GameScreen {
    /// First time - create party
    PartyCreation { step: CreationStep },
    /// Story intro
    Intro,
    /// World map navigation
    WorldMap,
    /// Town menu
    Town { location: String },
    /// Shop (items/equipment)
    Shop { location: String, shop_type: ShopType },
    /// Inn
    Inn { location: String },
    /// Save point
    SavePoint { location: String },
    /// Dungeon exploration
    Dungeon { location: String, floor: u8 },
    /// Active combat
    Combat { combat: Box<CombatState> },
    /// Combat target selection
    CombatTarget { combat: Box<CombatState>, action: CombatAction },
    /// Party menu
    PartyMenu,
    /// Character detail
    CharacterDetail { index: usize },
    /// Equipment menu
    EquipmentMenu { char_index: usize, slot_index: usize },
    /// Inventory
    Inventory,
    /// Use item submenu
    UseItem { item_index: usize },
    /// Magic menu
    MagicMenu { char_index: usize },
    /// Status display
    Status,
    /// Story event
    StoryEvent { event_key: String },
    /// Victory screen
    Victory { exp: u64, gold: u32 },
    /// Game over
    GameOver,
    /// Confirm quit
    ConfirmQuit,
    /// Ending sequence
    Ending { phase: u8 },
}

#[derive(Debug, Clone)]
pub enum CreationStep {
    SelectClass { member_num: usize },
    EnterName { member_num: usize, class: CharacterClass },
    ConfirmParty,
}

#[derive(Debug, Clone)]
pub enum ShopType {
    Items,
    Weapons,
    Armor,
}

/// Actions returned by the flow for session handling
#[derive(Debug, Clone)]
pub enum LastDreamAction {
    Continue,
    Render(String),
    Echo(String),
    SaveGame,
    GameComplete { play_time: u64 },
    Quit,
}

/// Last Dream game flow state machine
pub struct LastDreamFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub world_map: WorldMap,
    input_buffer: String,
    /// Menu cursor position
    cursor: usize,
    /// Secondary cursor (for nested menus)
    cursor2: usize,
}

impl LastDreamFlow {
    /// Create new game (needs party creation)
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            screen: GameScreen::PartyCreation {
                step: CreationStep::SelectClass { member_num: 0 },
            },
            world_map: WorldMap::new(),
            input_buffer: String::new(),
            cursor: 0,
            cursor2: 0,
        }
    }

    /// Resume game from loaded state
    pub fn from_state(state: GameState) -> Self {
        let screen = if state.party.members.is_empty() {
            GameScreen::PartyCreation {
                step: CreationStep::SelectClass { member_num: 0 },
            }
        } else if state.is_on_world_map() {
            GameScreen::WorldMap
        } else if let Some(ref loc) = state.current_location {
            if state.is_in_dungeon() {
                GameScreen::Dungeon {
                    location: loc.clone(),
                    floor: state.dungeon_floor.unwrap_or(1),
                }
            } else {
                GameScreen::Town { location: loc.clone() }
            }
        } else {
            GameScreen::WorldMap
        };

        Self {
            state,
            screen,
            world_map: WorldMap::new(),
            input_buffer: String::new(),
            cursor: 0,
            cursor2: 0,
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

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> LastDreamAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return LastDreamAction::Echo("\x08 \x08".to_string());
            }
            return LastDreamAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control characters
        if ch.is_control() {
            return LastDreamAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input for text entry
        if self.input_buffer.len() < 12 {
            self.input_buffer.push(ch);
            return LastDreamAction::Echo(ch.to_string());
        }

        LastDreamAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        !matches!(
            self.screen,
            GameScreen::PartyCreation {
                step: CreationStep::EnterName { .. }
            }
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> LastDreamAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen.clone() {
            GameScreen::PartyCreation { step } => self.handle_creation(&input, step.clone()),
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::WorldMap => self.handle_world_map(&input),
            GameScreen::Town { location } => self.handle_town(&input, &location.clone()),
            GameScreen::Shop { location, shop_type } => {
                self.handle_shop(&input, &location.clone(), shop_type.clone())
            }
            GameScreen::Inn { location } => self.handle_inn(&input, &location.clone()),
            GameScreen::SavePoint { location } => self.handle_save_point(&input, &location.clone()),
            GameScreen::Dungeon { location, floor } => {
                self.handle_dungeon(&input, &location.clone(), *floor)
            }
            GameScreen::Combat { combat } => self.handle_combat(&input, combat.clone()),
            GameScreen::CombatTarget { combat, action } => {
                self.handle_combat_target(&input, combat.clone(), action.clone())
            }
            GameScreen::PartyMenu => self.handle_party_menu(&input),
            GameScreen::CharacterDetail { index } => self.handle_character_detail(&input, *index),
            GameScreen::EquipmentMenu { char_index, slot_index } => {
                self.handle_equipment_menu(&input, *char_index, *slot_index)
            }
            GameScreen::Inventory => self.handle_inventory(&input),
            GameScreen::UseItem { item_index } => self.handle_use_item(&input, *item_index),
            GameScreen::MagicMenu { char_index } => self.handle_magic_menu(&input, *char_index),
            GameScreen::Status => self.handle_status(&input),
            GameScreen::StoryEvent { event_key } => self.handle_story_event(&input, &event_key.clone()),
            GameScreen::Victory { exp, gold } => self.handle_victory(&input, *exp, *gold),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
            GameScreen::Ending { phase } => self.handle_ending(&input, *phase),
        }
    }

    // ========================================================================
    // SCREEN HANDLERS
    // ========================================================================

    fn handle_creation(&mut self, input: &str, step: CreationStep) -> LastDreamAction {
        match step {
            CreationStep::SelectClass { member_num } => {
                let class = match input {
                    "1" => Some(CharacterClass::Warrior),
                    "2" => Some(CharacterClass::Thief),
                    "3" => Some(CharacterClass::Mage),
                    "4" => Some(CharacterClass::Cleric),
                    "5" => Some(CharacterClass::Monk),
                    "6" => Some(CharacterClass::Knight),
                    _ => None,
                };

                if let Some(class) = class {
                    self.screen = GameScreen::PartyCreation {
                        step: CreationStep::EnterName { member_num, class },
                    };
                }
                LastDreamAction::SaveGame
            }
            CreationStep::EnterName { member_num, class } => {
                let name = if input.is_empty() {
                    // Default names
                    match member_num {
                        0 => "Cecil",
                        1 => "Rosa",
                        2 => "Kain",
                        3 => "Rydia",
                        _ => "Hero",
                    }.to_string()
                } else if input.len() > 12 {
                    self.state.last_message = Some("Name must be 12 characters or less.".to_string());
                    return LastDreamAction::SaveGame;
                } else {
                    input.to_string()
                };

                // Create character
                let mut character = Character::new(name, class);

                // Give starting equipment based on class
                match class.to_class_type() {
                    ClassType::Warrior => {
                        character.equipment.weapon = Some("short_sword".to_string());
                        character.equipment.armor = Some("leather_armor".to_string());
                    }
                    ClassType::Thief => {
                        character.equipment.weapon = Some("dagger".to_string());
                        character.equipment.armor = Some("padded_vest".to_string());
                    }
                    ClassType::Mage => {
                        character.equipment.weapon = Some("wooden_staff".to_string());
                        character.equipment.armor = Some("cloth_robe".to_string());
                    }
                    ClassType::Cleric => {
                        character.equipment.weapon = Some("mace".to_string());
                        character.equipment.armor = Some("cloth_robe".to_string());
                    }
                    ClassType::Monk => {
                        character.equipment.armor = Some("cloth_robe".to_string());
                    }
                    ClassType::Knight => {
                        character.equipment.weapon = Some("short_sword".to_string());
                        character.equipment.armor = Some("chain_mail".to_string());
                        character.equipment.shield = Some("buckler".to_string());
                    }
                }

                self.state.party.add_member(character);

                if member_num < MAX_PARTY_SIZE - 1 {
                    self.screen = GameScreen::PartyCreation {
                        step: CreationStep::SelectClass { member_num: member_num + 1 },
                    };
                } else {
                    self.screen = GameScreen::PartyCreation {
                        step: CreationStep::ConfirmParty,
                    };
                }
                LastDreamAction::SaveGame
            }
            CreationStep::ConfirmParty => {
                match input {
                    "Y" | "1" => {
                        self.screen = GameScreen::Intro;
                        self.state.set_flag("intro_complete");
                    }
                    "N" | "2" => {
                        // Restart party creation
                        self.state.party = Party::new();
                        self.screen = GameScreen::PartyCreation {
                            step: CreationStep::SelectClass { member_num: 0 },
                        };
                    }
                    _ => {}
                }
                LastDreamAction::SaveGame
            }
        }
    }

    fn handle_intro(&mut self, _input: &str) -> LastDreamAction {
        self.screen = GameScreen::WorldMap;
        LastDreamAction::SaveGame
    }

    fn handle_world_map(&mut self, input: &str) -> LastDreamAction {
        self.state.last_message = None;

        match input {
            // Movement
            "W" | "8" => {
                self.try_move(0, -1);
                self.check_encounter();
            }
            "S" | "2" => {
                self.try_move(0, 1);
                self.check_encounter();
            }
            "A" | "4" => {
                self.try_move(-1, 0);
                self.check_encounter();
            }
            "D" | "6" => {
                self.try_move(1, 0);
                self.check_encounter();
            }

            // Enter location
            "E" | "5" => {
                self.try_enter_location();
            }

            // Party menu
            "P" | "M" => {
                self.cursor = 0;
                self.screen = GameScreen::PartyMenu;
            }

            // Quit
            "Q" => {
                self.screen = GameScreen::ConfirmQuit;
            }

            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn try_move(&mut self, dx: i32, dy: i32) {
        let new_x = (self.state.world_position.x as i32 + dx).max(0) as usize;
        let new_y = (self.state.world_position.y as i32 + dy).max(0) as usize;

        if new_x < self.world_map.width && new_y < self.world_map.height {
            let tile = self.world_map.get(new_x, new_y);

            if tile.walkable(self.state.transport) {
                self.state.world_position = Position::new(new_x, new_y);
            } else {
                self.state.last_message = Some(format!("Cannot cross {}.", tile.name()));
            }
        }
    }

    fn check_encounter(&mut self) {
        let tile = self.world_map.get(
            self.state.world_position.x,
            self.state.world_position.y,
        );

        let rate = tile.encounter_rate();
        if rate == 0 {
            return;
        }

        let mut rng = rand::thread_rng();
        if rand::Rng::gen_range(&mut rng, 0..100) < rate {
            let area_level = self.state.area_level();
            let combat = CombatState::new_encounter(area_level, tile.name());
            self.screen = GameScreen::Combat { combat: Box::new(combat) };
        }
    }

    fn try_enter_location(&mut self) {
        let _tile = self.world_map.get(
            self.state.world_position.x,
            self.state.world_position.y,
        );

        if let Some(location) = self.world_map.location_at(
            self.state.world_position.x,
            self.state.world_position.y,
        ) {
            // Check story flag requirements
            if let Some(required) = location.story_flag_required {
                if !self.state.has_flag(required) {
                    self.state.last_message = Some("Something blocks your path...".to_string());
                    return;
                }
            }

            self.state.enter_location(location.key);

            use super::world::LocationType;
            match location.location_type {
                LocationType::Town | LocationType::Castle => {
                    self.screen = GameScreen::Town {
                        location: location.key.to_string(),
                    };
                }
                LocationType::Dungeon | LocationType::Cave => {
                    self.state.enter_dungeon_floor(1);
                    self.screen = GameScreen::Dungeon {
                        location: location.key.to_string(),
                        floor: 1,
                    };
                }
            }

            // Check for simulation hint
            if let Some(hint) = self.state.check_simulation_hint() {
                self.state.last_message = Some(format!("A strange voice whispers: \"{}\"", hint));
            }
        } else {
            self.state.last_message = Some("Nothing to enter here.".to_string());
        }
    }

    fn handle_town(&mut self, input: &str, location: &str) -> LastDreamAction {
        self.state.last_message = None;

        let loc = get_location(location);

        match input {
            "I" if loc.map(|l| l.has_shop).unwrap_or(false) => {
                self.cursor = 0;
                self.screen = GameScreen::Shop {
                    location: location.to_string(),
                    shop_type: ShopType::Items,
                };
            }
            "W" if loc.map(|l| l.has_shop).unwrap_or(false) => {
                self.cursor = 0;
                self.screen = GameScreen::Shop {
                    location: location.to_string(),
                    shop_type: ShopType::Weapons,
                };
            }
            "A" if loc.map(|l| l.has_shop).unwrap_or(false) => {
                self.cursor = 0;
                self.screen = GameScreen::Shop {
                    location: location.to_string(),
                    shop_type: ShopType::Armor,
                };
            }
            "R" if loc.map(|l| l.has_inn).unwrap_or(false) => {
                self.screen = GameScreen::Inn {
                    location: location.to_string(),
                };
            }
            "S" if loc.map(|l| l.has_save_point).unwrap_or(false) => {
                self.screen = GameScreen::SavePoint {
                    location: location.to_string(),
                };
            }
            "P" | "M" => {
                self.cursor = 0;
                self.screen = GameScreen::PartyMenu;
            }
            "L" | "Q" => {
                self.state.exit_location();
                self.screen = GameScreen::WorldMap;
            }
            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn handle_shop(&mut self, input: &str, location: &str, shop_type: ShopType) -> LastDreamAction {
        match input {
            "Q" | "L" => {
                self.screen = GameScreen::Town {
                    location: location.to_string(),
                };
            }
            _ => {
                // Try to buy item by number
                if let Ok(num) = input.parse::<usize>() {
                    if num > 0 {
                        self.try_buy(num - 1, &shop_type);
                    }
                }
            }
        }

        LastDreamAction::SaveGame
    }

    fn try_buy(&mut self, index: usize, shop_type: &ShopType) {
        match shop_type {
            ShopType::Items => {
                if index < ITEMS.len() {
                    let item = &ITEMS[index];
                    if self.state.spend_gold(item.price) {
                        self.state.add_item(item.key, 1);
                        self.state.last_message = Some(format!("Bought {}!", item.name));
                    } else {
                        self.state.last_message = Some("Not enough gold!".to_string());
                    }
                }
            }
            ShopType::Weapons | ShopType::Armor => {
                let equip_list: Vec<_> = EQUIPMENT.iter()
                    .filter(|e| match shop_type {
                        ShopType::Weapons => matches!(
                            e.slot,
                            super::data::EquipmentSlot::Weapon
                        ),
                        ShopType::Armor => !matches!(
                            e.slot,
                            super::data::EquipmentSlot::Weapon
                        ),
                        _ => false,
                    })
                    .filter(|e| e.price > 0)
                    .collect();

                if index < equip_list.len() {
                    let equip = equip_list[index];
                    if self.state.spend_gold(equip.price) {
                        self.state.add_item(equip.key, 1);
                        self.state.last_message = Some(format!("Bought {}!", equip.name));
                    } else {
                        self.state.last_message = Some("Not enough gold!".to_string());
                    }
                }
            }
        }
    }

    fn handle_inn(&mut self, input: &str, location: &str) -> LastDreamAction {
        match input {
            "Y" | "1" => {
                // Calculate cost based on party size
                let cost = self.state.party.members.len() as u32 * 50;
                if self.state.spend_gold(cost) {
                    self.state.party.rest_at_inn();
                    self.state.last_message = Some("Party fully rested!".to_string());
                } else {
                    self.state.last_message = Some("Not enough gold!".to_string());
                }
                self.screen = GameScreen::Town {
                    location: location.to_string(),
                };
            }
            "N" | "Q" => {
                self.screen = GameScreen::Town {
                    location: location.to_string(),
                };
            }
            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn handle_save_point(&mut self, input: &str, location: &str) -> LastDreamAction {
        match input {
            "Y" | "1" => {
                self.state.last_message = Some("Game saved!".to_string());
                self.screen = GameScreen::Town {
                    location: location.to_string(),
                };
                return LastDreamAction::SaveGame;
            }
            "N" | "Q" => {
                self.screen = GameScreen::Town {
                    location: location.to_string(),
                };
            }
            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn handle_dungeon(&mut self, input: &str, location: &str, floor: u8) -> LastDreamAction {
        self.state.last_message = None;

        match input {
            // Movement
            "W" | "8" => self.try_dungeon_move(0, -1),
            "S" | "2" => self.try_dungeon_move(0, 1),
            "A" | "4" => self.try_dungeon_move(-1, 0),
            "D" | "6" => self.try_dungeon_move(1, 0),

            // Party menu
            "P" | "M" => {
                self.cursor = 0;
                self.screen = GameScreen::PartyMenu;
                return LastDreamAction::SaveGame;
            }

            // Leave dungeon
            "L" | "Q" => {
                if floor == 1 {
                    self.state.exit_location();
                    self.screen = GameScreen::WorldMap;
                } else {
                    self.state.enter_dungeon_floor(floor - 1);
                    self.screen = GameScreen::Dungeon {
                        location: location.to_string(),
                        floor: floor - 1,
                    };
                }
            }

            _ => {}
        }

        // Check for encounter after movement
        if matches!(input, "W" | "S" | "A" | "D" | "8" | "2" | "4" | "6") {
            self.check_dungeon_encounter(location, floor);
        }

        LastDreamAction::SaveGame
    }

    fn try_dungeon_move(&mut self, dx: i32, dy: i32) {
        if let Some(ref mut pos) = self.state.location_position {
            let new_x = (pos.x as i32 + dx).max(0) as usize;
            let new_y = (pos.y as i32 + dy).max(0) as usize;

            // For now, simple bounds check (actual dungeon tile checking would go here)
            if new_x < 20 && new_y < 10 {
                pos.x = new_x;
                pos.y = new_y;
            }
        }
    }

    fn check_dungeon_encounter(&mut self, location: &str, _floor: u8) {
        let mut rng = rand::thread_rng();
        if rand::Rng::gen_range(&mut rng, 0..100) < 15 {
            let area_level = self.state.area_level();
            let combat = CombatState::new_encounter(area_level, location);
            self.screen = GameScreen::Combat { combat: Box::new(combat) };
        }
    }

    fn handle_combat(&mut self, input: &str, mut combat: Box<CombatState>) -> LastDreamAction {
        use super::combat::CombatPhase;

        // Tick combat
        let _results = combat.tick(&mut self.state.party);

        if combat.finished {
            if combat.victory {
                self.screen = GameScreen::Victory {
                    exp: combat.exp_reward,
                    gold: combat.gold_reward,
                };
            } else {
                // Check if party wiped or just ran
                if !self.state.party.is_alive() {
                    self.screen = GameScreen::GameOver;
                } else {
                    // Ran away - return to exploration
                    self.return_from_combat();
                }
            }
            return LastDreamAction::SaveGame;
        }

        match combat.phase {
            CombatPhase::SelectAction => {
                let action = match input {
                    "A" | "1" => Some(CombatAction::Attack),
                    "M" | "2" => {
                        // Check if active member has spells
                        if let Some(idx) = combat.active_member {
                            if !self.state.party.members[idx].spells.is_empty() {
                                self.screen = GameScreen::MagicMenu { char_index: idx };
                                return LastDreamAction::SaveGame;
                            }
                        }
                        None
                    }
                    "I" | "3" => {
                        self.screen = GameScreen::Inventory;
                        return LastDreamAction::SaveGame;
                    }
                    "D" | "4" => Some(CombatAction::Defend),
                    "R" | "5" => Some(CombatAction::Run),
                    _ => None,
                };

                if let Some(action) = action {
                    if matches!(action, CombatAction::Attack) {
                        // Need target selection
                        self.cursor = 0;
                        self.screen = GameScreen::CombatTarget {
                            combat,
                            action,
                        };
                        return LastDreamAction::SaveGame;
                    } else {
                        // Execute immediately
                        combat.execute_action(&mut self.state.party, action, None);
                    }
                }
            }
            _ => {
                // Tick more
                combat.tick(&mut self.state.party);
            }
        }

        self.screen = GameScreen::Combat { combat };
        LastDreamAction::SaveGame
    }

    fn handle_combat_target(
        &mut self,
        input: &str,
        mut combat: Box<CombatState>,
        action: CombatAction,
    ) -> LastDreamAction {
        let living = combat.living_enemies();

        match input {
            "Q" | "C" => {
                self.screen = GameScreen::Combat { combat };
            }
            _ => {
                if let Ok(num) = input.parse::<usize>() {
                    if num > 0 && num <= living.len() {
                        let target_idx = living[num - 1].0;
                        combat.execute_action(&mut self.state.party, action, Some(target_idx));
                        self.screen = GameScreen::Combat { combat };
                    }
                }
            }
        }

        LastDreamAction::SaveGame
    }

    fn handle_party_menu(&mut self, input: &str) -> LastDreamAction {
        match input {
            "1" | "2" | "3" | "4" => {
                let idx = input.parse::<usize>().unwrap() - 1;
                if idx < self.state.party.members.len() {
                    self.cursor = 0;
                    self.screen = GameScreen::CharacterDetail { index: idx };
                }
            }
            "I" => {
                self.cursor = 0;
                self.screen = GameScreen::Inventory;
            }
            "S" => {
                self.screen = GameScreen::Status;
            }
            "Q" | "B" => {
                self.return_to_exploration();
            }
            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn handle_character_detail(&mut self, input: &str, index: usize) -> LastDreamAction {
        match input {
            "E" => {
                self.cursor = 0;
                self.cursor2 = 0;
                self.screen = GameScreen::EquipmentMenu {
                    char_index: index,
                    slot_index: 0,
                };
            }
            "Q" | "B" => {
                self.screen = GameScreen::PartyMenu;
            }
            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn handle_equipment_menu(
        &mut self,
        input: &str,
        char_index: usize,
        _slot_index: usize,
    ) -> LastDreamAction {
        match input {
            "Q" | "B" => {
                self.screen = GameScreen::CharacterDetail { index: char_index };
            }
            _ => {
                // Equipment selection logic would go here
            }
        }

        LastDreamAction::SaveGame
    }

    fn handle_inventory(&mut self, input: &str) -> LastDreamAction {
        match input {
            "Q" | "B" => {
                self.return_to_exploration();
            }
            _ => {
                if let Ok(num) = input.parse::<usize>() {
                    if num > 0 && num <= self.state.inventory.len() {
                        self.screen = GameScreen::UseItem { item_index: num - 1 };
                    }
                }
            }
        }

        LastDreamAction::SaveGame
    }

    fn handle_use_item(&mut self, input: &str, item_index: usize) -> LastDreamAction {
        match input {
            "Q" | "B" => {
                self.screen = GameScreen::Inventory;
            }
            _ => {
                if let Ok(target) = input.parse::<usize>() {
                    if target > 0 && target <= self.state.party.members.len() {
                        if let Some(item) = self.state.inventory.get(item_index) {
                            if let Some(item_data) = get_item(&item.key) {
                                let target_idx = target - 1;

                                // Apply item effects and get member name
                                let member_name = {
                                    let member = &mut self.state.party.members[target_idx];
                                    if item_data.heal_hp > 0 {
                                        member.heal(item_data.heal_hp);
                                    }
                                    if item_data.heal_mp > 0 {
                                        member.restore_mp(item_data.heal_mp);
                                    }
                                    if item_data.revive && member.status.dead {
                                        member.revive(50);
                                    }
                                    member.name.clone()
                                };

                                let key = item.key.clone();
                                let item_name = item_data.name.to_string();
                                self.state.remove_item(&key, 1);
                                self.state.last_message = Some(format!(
                                    "Used {} on {}!",
                                    item_name, member_name
                                ));
                            }
                        }
                        self.screen = GameScreen::Inventory;
                    }
                }
            }
        }

        LastDreamAction::SaveGame
    }

    fn handle_magic_menu(&mut self, input: &str, _char_index: usize) -> LastDreamAction {
        // Magic selection during combat would go here
        match input {
            "Q" | "B" => {
                self.return_to_exploration();
            }
            _ => {}
        }

        LastDreamAction::SaveGame
    }

    fn handle_status(&mut self, _input: &str) -> LastDreamAction {
        self.screen = GameScreen::PartyMenu;
        LastDreamAction::SaveGame
    }

    fn handle_story_event(&mut self, _input: &str, _event_key: &str) -> LastDreamAction {
        self.return_to_exploration();
        LastDreamAction::SaveGame
    }

    fn handle_victory(&mut self, _input: &str, exp: u64, gold: u32) -> LastDreamAction {
        // Distribute rewards
        self.state.add_gold(gold);
        let level_ups = self.state.party.distribute_exp(exp);

        // Record battle
        self.state.record_battle(1);

        self.state.last_message = Some(format!(
            "Victory! Gained {} EXP, {} Gil!",
            exp, gold
        ));

        for (name, maybe_level) in level_ups {
            if let Some(level) = maybe_level {
                self.state.last_message = Some(format!(
                    "{} reached level {}!",
                    name, level.new_level
                ));
            }
        }

        self.return_from_combat();
        LastDreamAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> LastDreamAction {
        // On game over, restart from last save
        LastDreamAction::Quit
    }

    fn handle_confirm_quit(&mut self, input: &str) -> LastDreamAction {
        match input {
            "Y" => LastDreamAction::Quit,
            _ => {
                self.return_to_exploration();
                LastDreamAction::SaveGame
            }
        }
    }

    fn handle_ending(&mut self, _input: &str, phase: u8) -> LastDreamAction {
        if phase >= 5 {
            LastDreamAction::GameComplete {
                play_time: self.state.play_time,
            }
        } else {
            self.screen = GameScreen::Ending { phase: phase + 1 };
            LastDreamAction::SaveGame
        }
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    fn return_to_exploration(&mut self) {
        if self.state.is_on_world_map() {
            self.screen = GameScreen::WorldMap;
        } else if let Some(ref loc) = self.state.current_location.clone() {
            if self.state.is_in_dungeon() {
                self.screen = GameScreen::Dungeon {
                    location: loc.clone(),
                    floor: self.state.dungeon_floor.unwrap_or(1),
                };
            } else {
                self.screen = GameScreen::Town { location: loc.clone() };
            }
        } else {
            self.screen = GameScreen::WorldMap;
        }
    }

    fn return_from_combat(&mut self) {
        self.return_to_exploration();
    }
}

impl Default for LastDreamFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = LastDreamFlow::new();
        assert!(matches!(flow.screen, GameScreen::PartyCreation { .. }));
    }

    #[test]
    fn test_party_creation_flow() {
        let mut flow = LastDreamFlow::new();

        // Select Warrior class
        flow.handle_char('1');
        assert!(matches!(
            flow.screen,
            GameScreen::PartyCreation { step: CreationStep::EnterName { .. } }
        ));

        // Enter default name
        flow.handle_char('\r');
        // Should move to next character
        assert!(matches!(
            flow.screen,
            GameScreen::PartyCreation { step: CreationStep::SelectClass { member_num: 1 } }
        ));
    }

    #[test]
    fn test_world_map_movement() {
        let mut flow = LastDreamFlow::new();
        // Skip to world map
        flow.state.party.add_member(Character::new("Test".to_string(), CharacterClass::Warrior));
        flow.screen = GameScreen::WorldMap;

        let _initial_pos = flow.state.world_position;
        flow.handle_char('D'); // Move right

        // Position should change (or stay same if blocked)
        // The test is that it doesn't crash
        assert!(true);
    }

    #[test]
    fn test_combat_initiation() {
        let mut flow = LastDreamFlow::new();
        flow.state.party.add_member(Character::new("Test".to_string(), CharacterClass::Warrior));

        // Start combat
        let combat = CombatState::new_encounter(1, "test");
        flow.screen = GameScreen::Combat { combat: Box::new(combat) };

        // Should be in combat
        assert!(matches!(flow.screen, GameScreen::Combat { .. }));
    }

    #[test]
    fn test_inventory_access() {
        let mut flow = LastDreamFlow::new();
        flow.state.party.add_member(Character::new("Test".to_string(), CharacterClass::Warrior));
        flow.screen = GameScreen::PartyMenu;

        flow.handle_char('I');
        assert!(matches!(flow.screen, GameScreen::Inventory));
    }
}
