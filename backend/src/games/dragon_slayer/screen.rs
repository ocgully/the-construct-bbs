//! Dragon Slayer screen/flow state machine
//! Manages screen transitions and input handling

use super::state::{GameState, Sex};
use super::combat::{CombatState, CombatAction, CombatResult, process_combat};
use super::data::{get_master, get_random_monster, WEAPONS, ARMOR, SKILL_DATA};
use super::events::{maybe_forest_event, apply_forest_event, ForestEvent};
use super::romance::{RomanceNpc, flirt_with_npc, propose_to_npc, divorce, RomanceResult};
use super::igm::IgmRegistry;

/// Which screen the player is currently viewing
#[derive(Debug, Clone)]
pub enum GameScreen {
    /// First time - create character
    CharacterCreation { step: CreationStep },
    /// New game intro / returning player welcome
    Intro,
    /// Main town menu
    Town,
    /// The Dark Forest
    Forest,
    /// Active combat
    Combat { combat: Box<CombatState> },
    /// Forest event (non-combat)
    ForestEvent { event: ForestEvent },
    /// Turgon's Training Grounds
    Training,
    /// Weapons Shop
    WeaponShop,
    /// Armor Shop
    ArmorShop,
    /// Healer's Hut
    Healer,
    /// The Bank
    Bank,
    /// King's Court
    KingsCourt,
    /// Red Dragon Inn
    Inn,
    /// Violet's House (romance)
    Violet,
    /// Seth's Tavern (romance)
    Seth,
    /// Slaughter Arena (PvP)
    Arena,
    /// Other Places (IGM modules)
    OtherPlaces,
    /// IGM module screen
    IgmLocation { module_id: String },
    /// View player stats
    Stats,
    /// Leaderboard / Rankings
    Leaderboard,
    /// Dragon encounter (level 12)
    DragonHunt,
    /// Game over / death
    GameOver,
    /// Confirm quit
    ConfirmQuit,
    /// Dragon slain - victory ending
    Victory,
}

#[derive(Debug, Clone)]
pub enum CreationStep {
    EnterName,
    SelectSex,
}

/// Actions returned by the flow for session handling
#[derive(Debug, Clone)]
pub enum DragonSlayerAction {
    /// Continue - no output needed
    Continue,
    /// Render screen output
    #[allow(dead_code)]
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game completed (dragon slain)
    GameComplete {
        #[allow(dead_code)]
        dragon_kills: u32,
    },
    /// Player quit to main menu
    Quit,
}

/// Dragon Slayer game flow state machine
pub struct DragonSlayerFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub igm_registry: IgmRegistry,
    input_buffer: String,
    /// Temporary storage for character creation
    temp_char_name: Option<String>,
}

impl DragonSlayerFlow {
    /// Create new game (needs character creation)
    pub fn new() -> Self {
        Self {
            state: GameState::new("".to_string(), Sex::Male),
            screen: GameScreen::CharacterCreation { step: CreationStep::EnterName },
            igm_registry: IgmRegistry::new(),
            input_buffer: String::new(),
            temp_char_name: None,
        }
    }

    /// Resume game from loaded state
    pub fn from_state(mut state: GameState) -> Self {
        // Check for new day (result used for reset, value not needed)
        let _is_new_day = state.check_new_day();

        let screen = if state.is_dead {
            GameScreen::GameOver
        } else {
            GameScreen::Intro
        };

        // Load default IGM modules
        let mut igm_registry = IgmRegistry::new();
        for module in super::igm::create_default_modules() {
            let _ = igm_registry.register(module);
        }

        Self {
            state,
            screen,
            igm_registry,
            input_buffer: String::new(),
            temp_char_name: None,
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
    pub fn handle_char(&mut self, ch: char) -> DragonSlayerAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return DragonSlayerAction::Echo("\x08 \x08".to_string());
            }
            return DragonSlayerAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control characters
        if ch.is_control() {
            return DragonSlayerAction::Continue;
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
            return DragonSlayerAction::Echo(ch.to_string());
        }

        DragonSlayerAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        !matches!(
            self.screen,
            GameScreen::CharacterCreation { step: CreationStep::EnterName }
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> DragonSlayerAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::CharacterCreation { step } => self.handle_creation(&input, step.clone()),
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::Town => self.handle_town(&input),
            GameScreen::Forest => self.handle_forest(&input),
            GameScreen::Combat { .. } => self.handle_combat(&input),
            GameScreen::ForestEvent { .. } => self.handle_forest_event(&input),
            GameScreen::Training => self.handle_training(&input),
            GameScreen::WeaponShop => self.handle_weapon_shop(&input),
            GameScreen::ArmorShop => self.handle_armor_shop(&input),
            GameScreen::Healer => self.handle_healer(&input),
            GameScreen::Bank => self.handle_bank(&input),
            GameScreen::KingsCourt => self.handle_kings_court(&input),
            GameScreen::Inn => self.handle_inn(&input),
            GameScreen::Violet => self.handle_violet(&input),
            GameScreen::Seth => self.handle_seth(&input),
            GameScreen::Arena => self.handle_arena(&input),
            GameScreen::OtherPlaces => self.handle_other_places(&input),
            GameScreen::IgmLocation { .. } => self.handle_igm_location(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::DragonHunt => self.handle_dragon_hunt(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
            GameScreen::Victory => self.handle_victory(&input),
        }
    }

    // ========================================================================
    // SCREEN HANDLERS
    // ========================================================================

    fn handle_creation(&mut self, input: &str, step: CreationStep) -> DragonSlayerAction {
        match step {
            CreationStep::EnterName => {
                if input.is_empty() || input.len() > 20 {
                    self.state.last_message = Some("Enter a name (1-20 characters).".to_string());
                    return DragonSlayerAction::SaveGame;
                }
                self.temp_char_name = Some(input.to_string());
                self.screen = GameScreen::CharacterCreation { step: CreationStep::SelectSex };
                DragonSlayerAction::SaveGame
            }
            CreationStep::SelectSex => {
                let sex = match input {
                    "M" | "1" => Sex::Male,
                    "F" | "2" => Sex::Female,
                    _ => {
                        self.state.last_message = Some("Choose (M)ale or (F)emale.".to_string());
                        return DragonSlayerAction::SaveGame;
                    }
                };

                let name = self.temp_char_name.take().unwrap_or_else(|| "Hero".to_string());
                self.state = GameState::new(name, sex);
                self.state.check_new_day();
                self.screen = GameScreen::Intro;
                DragonSlayerAction::SaveGame
            }
        }
    }

    fn handle_intro(&mut self, _input: &str) -> DragonSlayerAction {
        self.screen = GameScreen::Town;
        DragonSlayerAction::SaveGame
    }

    fn handle_town(&mut self, input: &str) -> DragonSlayerAction {
        self.state.last_message = None;

        match input {
            "F" => {
                self.screen = GameScreen::Forest;
                DragonSlayerAction::SaveGame
            }
            "T" => {
                self.screen = GameScreen::Training;
                DragonSlayerAction::SaveGame
            }
            "W" => {
                self.screen = GameScreen::WeaponShop;
                DragonSlayerAction::SaveGame
            }
            "A" => {
                self.screen = GameScreen::ArmorShop;
                DragonSlayerAction::SaveGame
            }
            "H" => {
                self.screen = GameScreen::Healer;
                DragonSlayerAction::SaveGame
            }
            "B" => {
                self.screen = GameScreen::Bank;
                DragonSlayerAction::SaveGame
            }
            "K" => {
                self.screen = GameScreen::KingsCourt;
                DragonSlayerAction::SaveGame
            }
            "I" => {
                self.screen = GameScreen::Inn;
                DragonSlayerAction::SaveGame
            }
            "V" => {
                self.screen = GameScreen::Violet;
                DragonSlayerAction::SaveGame
            }
            "S" => {
                self.screen = GameScreen::Seth;
                DragonSlayerAction::SaveGame
            }
            "P" => {
                self.screen = GameScreen::Arena;
                DragonSlayerAction::SaveGame
            }
            "O" => {
                self.screen = GameScreen::OtherPlaces;
                DragonSlayerAction::SaveGame
            }
            "Y" => {
                self.screen = GameScreen::Stats;
                DragonSlayerAction::SaveGame
            }
            "L" => {
                self.screen = GameScreen::Leaderboard;
                DragonSlayerAction::SaveGame
            }
            "Q" => {
                self.screen = GameScreen::ConfirmQuit;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_forest(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "L" | "H" => {
                // Look for something / Hunt
                if self.state.forest_fights_remaining() == 0 {
                    self.state.last_message = Some("You're too tired to continue. Rest at the Inn.".to_string());
                    self.screen = GameScreen::Town;
                    return DragonSlayerAction::SaveGame;
                }

                // Check for forest event first
                if let Some(event) = maybe_forest_event(&self.state) {
                    self.screen = GameScreen::ForestEvent { event };
                    return DragonSlayerAction::SaveGame;
                }

                // Otherwise, monster encounter
                if let Some(monster) = get_random_monster(self.state.level) {
                    self.state.use_forest_fight();
                    let combat = CombatState::from_monster(monster, self.state.level);
                    self.screen = GameScreen::Combat { combat: Box::new(combat) };
                } else {
                    self.state.last_message = Some("The forest is quiet today...".to_string());
                    self.screen = GameScreen::Town;
                }
                DragonSlayerAction::SaveGame
            }
            "D" if self.state.level >= 12 => {
                // Search for the dragon (level 12 only)
                self.screen = GameScreen::DragonHunt;
                DragonSlayerAction::SaveGame
            }
            "R" | "Q" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_combat(&mut self, input: &str) -> DragonSlayerAction {
        let action = match input {
            "A" | "1" => CombatAction::Attack,
            "R" | "2" => CombatAction::Run,
            "S" | "3" => {
                // Use first available skill
                if let Some(skill_key) = self.get_usable_skill() {
                    CombatAction::UseSkill { skill_key }
                } else {
                    self.state.last_message = Some("No skills available!".to_string());
                    return DragonSlayerAction::SaveGame;
                }
            }
            // Skill shortcuts
            "P" => CombatAction::UseSkill { skill_key: "power_strike".to_string() },
            "F" => CombatAction::UseSkill { skill_key: "fireball".to_string() },
            "H" => CombatAction::UseSkill { skill_key: "heal".to_string() },
            _ => return DragonSlayerAction::Continue,
        };

        // Get combat state
        let combat = if let GameScreen::Combat { combat } = &mut self.screen {
            combat
        } else {
            return DragonSlayerAction::Continue;
        };

        let result = process_combat(&mut self.state, combat, action);

        match result {
            CombatResult::Continue => {
                // Combat continues
                DragonSlayerAction::SaveGame
            }
            CombatResult::Victory { xp_gained, gold_gained, message } => {
                self.state.last_message = Some(format!(
                    "{} Gained {} XP and {} gold!",
                    message, xp_gained, gold_gained
                ));

                // Check for level up eligibility
                if let Some(level_msg) = self.check_level_up_message() {
                    let current_msg = self.state.last_message.take().unwrap_or_default();
                    self.state.last_message = Some(format!("{}\n{}", current_msg, level_msg));
                }

                self.screen = GameScreen::Forest;
                DragonSlayerAction::SaveGame
            }
            CombatResult::Defeat { message } => {
                self.state.last_message = Some(message);
                if self.state.is_dead {
                    self.screen = GameScreen::GameOver;
                } else {
                    self.screen = GameScreen::Town;
                }
                DragonSlayerAction::SaveGame
            }
            CombatResult::Fled { message } => {
                self.state.last_message = Some(message);
                self.screen = GameScreen::Forest;
                DragonSlayerAction::SaveGame
            }
            CombatResult::FledFailed { message, .. } => {
                self.state.last_message = Some(message);
                DragonSlayerAction::SaveGame
            }
            CombatResult::MasterDefeated { level: _, message } => {
                self.state.level_up();
                self.state.last_message = Some(format!(
                    "{}\n\nYou are now LEVEL {}!",
                    message, self.state.level
                ));
                self.screen = GameScreen::Training;
                DragonSlayerAction::SaveGame
            }
            CombatResult::DragonSlain { message } => {
                self.state.last_message = Some(message);
                self.screen = GameScreen::Victory;
                DragonSlayerAction::SaveGame
            }
        }
    }

    fn handle_forest_event(&mut self, _input: &str) -> DragonSlayerAction {
        if let GameScreen::ForestEvent { event } = &self.screen {
            let message = apply_forest_event(&mut self.state, event);
            self.state.last_message = Some(message);
        }
        self.screen = GameScreen::Forest;
        DragonSlayerAction::SaveGame
    }

    fn handle_training(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "C" | "1" => {
                // Challenge master
                if let Some(master) = get_master(self.state.level) {
                    if self.state.experience < master.xp_required {
                        self.state.last_message = Some(format!(
                            "You need {} XP to challenge {}. You have {}.",
                            master.xp_required, master.name, self.state.experience
                        ));
                    } else {
                        let combat = CombatState::from_master(master);
                        self.screen = GameScreen::Combat { combat: Box::new(combat) };
                    }
                } else if self.state.level >= 12 {
                    self.state.last_message = Some("You have surpassed all masters! Hunt the Dragon!".to_string());
                }
                DragonSlayerAction::SaveGame
            }
            "Q" | "R" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_weapon_shop(&mut self, input: &str) -> DragonSlayerAction {
        if input == "Q" || input == "R" {
            self.screen = GameScreen::Town;
            return DragonSlayerAction::SaveGame;
        }

        // Parse weapon selection
        if let Ok(idx) = input.parse::<usize>() {
            let available: Vec<_> = WEAPONS.iter()
                .filter(|w| w.level_required <= self.state.level)
                .collect();

            if idx >= 1 && idx <= available.len() {
                let weapon = available[idx - 1];

                if self.state.gold_pocket < weapon.price {
                    self.state.last_message = Some(format!(
                        "You need {} gold for the {}!",
                        weapon.price, weapon.name
                    ));
                } else if self.state.equipment.weapon == weapon.key {
                    self.state.last_message = Some("You already have that weapon!".to_string());
                } else {
                    self.state.gold_pocket -= weapon.price;
                    self.state.equipment.weapon = weapon.key.to_string();
                    self.state.last_message = Some(format!(
                        "You bought the {} for {} gold!",
                        weapon.name, weapon.price
                    ));
                }
            }
        }

        DragonSlayerAction::SaveGame
    }

    fn handle_armor_shop(&mut self, input: &str) -> DragonSlayerAction {
        if input == "Q" || input == "R" {
            self.screen = GameScreen::Town;
            return DragonSlayerAction::SaveGame;
        }

        // Parse armor selection
        if let Ok(idx) = input.parse::<usize>() {
            let available: Vec<_> = ARMOR.iter()
                .filter(|a| a.level_required <= self.state.level)
                .collect();

            if idx >= 1 && idx <= available.len() {
                let armor = available[idx - 1];

                if self.state.gold_pocket < armor.price {
                    self.state.last_message = Some(format!(
                        "You need {} gold for the {}!",
                        armor.price, armor.name
                    ));
                } else if self.state.equipment.armor == armor.key {
                    self.state.last_message = Some("You already have that armor!".to_string());
                } else {
                    self.state.gold_pocket -= armor.price;
                    self.state.equipment.armor = armor.key.to_string();
                    self.state.last_message = Some(format!(
                        "You bought the {} for {} gold!",
                        armor.name, armor.price
                    ));
                }
            }
        }

        DragonSlayerAction::SaveGame
    }

    fn handle_healer(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "H" | "1" => {
                let cost_per_hp = 5 + (self.state.level as i64);
                match self.state.heal(cost_per_hp) {
                    Ok(cost) => {
                        self.state.last_message = Some(format!(
                            "The healer restores you to full health for {} gold.",
                            cost
                        ));
                    }
                    Err(msg) => {
                        self.state.last_message = Some(msg.to_string());
                    }
                }
                DragonSlayerAction::SaveGame
            }
            "Q" | "R" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_bank(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "D" | "1" => {
                // Deposit all
                let amount = self.state.gold_pocket;
                if amount > 0 {
                    self.state.gold_bank += amount;
                    self.state.gold_pocket = 0;
                    self.state.last_message = Some(format!("Deposited {} gold.", amount));
                } else {
                    self.state.last_message = Some("You have no gold to deposit!".to_string());
                }
                DragonSlayerAction::SaveGame
            }
            "W" | "2" => {
                // Withdraw all
                let amount = self.state.gold_bank;
                if amount > 0 {
                    self.state.gold_pocket += amount;
                    self.state.gold_bank = 0;
                    self.state.last_message = Some(format!("Withdrew {} gold.", amount));
                } else {
                    self.state.last_message = Some("You have no gold in the bank!".to_string());
                }
                DragonSlayerAction::SaveGame
            }
            "Q" | "R" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_kings_court(&mut self, _input: &str) -> DragonSlayerAction {
        self.screen = GameScreen::Town;
        DragonSlayerAction::SaveGame
    }

    fn handle_inn(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "R" | "1" => {
                // Rest for the night - ends today's session
                self.state.rest_at_inn();
                self.state.last_message = Some("You rest for the night. Tomorrow awaits!".to_string());
                // Save and exit - player comes back tomorrow
                DragonSlayerAction::Quit
            }
            "G" | "2" => {
                // Gossip (hints)
                let hints = [
                    "The dragon's lair is deep in the forest...",
                    "I heard Violet has a thing for heroes.",
                    "The masters grow stronger with each level.",
                    "Bank your gold before hunting in the forest!",
                ];
                let mut rng = rand::thread_rng();
                let hint = hints[rand::Rng::gen_range(&mut rng, 0..hints.len())];
                self.state.last_message = Some(format!("The bartender says: \"{}\"", hint));
                DragonSlayerAction::SaveGame
            }
            "Q" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_violet(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "F" | "1" => {
                let result = flirt_with_npc(&mut self.state, RomanceNpc::Violet);
                self.state.last_message = Some(romance_result_message(&result));
                DragonSlayerAction::SaveGame
            }
            "P" | "2" => {
                let result = propose_to_npc(&mut self.state, RomanceNpc::Violet, 1000);
                self.state.last_message = Some(romance_result_message(&result));
                DragonSlayerAction::SaveGame
            }
            "D" | "3" => {
                let result = divorce(&mut self.state);
                self.state.last_message = Some(romance_result_message(&result));
                DragonSlayerAction::SaveGame
            }
            "Q" | "R" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_seth(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "F" | "1" => {
                let result = flirt_with_npc(&mut self.state, RomanceNpc::Seth);
                self.state.last_message = Some(romance_result_message(&result));
                DragonSlayerAction::SaveGame
            }
            "P" | "2" => {
                let result = propose_to_npc(&mut self.state, RomanceNpc::Seth, 1000);
                self.state.last_message = Some(romance_result_message(&result));
                DragonSlayerAction::SaveGame
            }
            "D" | "3" => {
                let result = divorce(&mut self.state);
                self.state.last_message = Some(romance_result_message(&result));
                DragonSlayerAction::SaveGame
            }
            "Q" | "R" => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_arena(&mut self, _input: &str) -> DragonSlayerAction {
        // TODO: Implement PvP - requires async player loading
        self.state.last_message = Some("The arena is empty today...".to_string());
        self.screen = GameScreen::Town;
        DragonSlayerAction::SaveGame
    }

    fn handle_other_places(&mut self, input: &str) -> DragonSlayerAction {
        if input == "Q" || input == "R" {
            self.screen = GameScreen::Town;
            return DragonSlayerAction::SaveGame;
        }

        // Check for IGM location by hotkey
        let locations = self.igm_registry.get_locations();
        for module in locations {
            if input == module.hotkey.to_string().to_uppercase() {
                self.screen = GameScreen::IgmLocation { module_id: module.id.clone() };
                return DragonSlayerAction::SaveGame;
            }
        }

        DragonSlayerAction::Continue
    }

    fn handle_igm_location(&mut self, input: &str) -> DragonSlayerAction {
        if input == "Q" || input == "R" {
            self.screen = GameScreen::OtherPlaces;
            return DragonSlayerAction::SaveGame;
        }

        // TODO: Implement IGM interaction
        DragonSlayerAction::Continue
    }

    fn handle_stats(&mut self, _input: &str) -> DragonSlayerAction {
        self.screen = GameScreen::Town;
        DragonSlayerAction::SaveGame
    }

    fn handle_leaderboard(&mut self, _input: &str) -> DragonSlayerAction {
        self.screen = GameScreen::Town;
        DragonSlayerAction::SaveGame
    }

    fn handle_dragon_hunt(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "S" | "1" => {
                // Search for dragon (33% chance each search)
                let mut rng = rand::thread_rng();
                if rand::Rng::gen_range(&mut rng, 0..100) < 33 {
                    let combat = CombatState::red_dragon();
                    self.screen = GameScreen::Combat { combat: Box::new(combat) };
                    self.state.last_message = None;
                } else {
                    self.state.last_message = Some("You search the forest but find no dragon...".to_string());
                }
                DragonSlayerAction::SaveGame
            }
            "Q" | "R" => {
                self.screen = GameScreen::Forest;
                DragonSlayerAction::SaveGame
            }
            _ => DragonSlayerAction::Continue,
        }
    }

    fn handle_game_over(&mut self, _input: &str) -> DragonSlayerAction {
        // Player must wait until tomorrow to play again
        // Check if a new day has started which would revive them
        if self.state.check_new_day() && !self.state.is_dead {
            // New day started and player was revived
            self.screen = GameScreen::Intro;
            DragonSlayerAction::SaveGame
        } else {
            // Still dead - save and exit the game, come back tomorrow
            DragonSlayerAction::Quit
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> DragonSlayerAction {
        match input {
            "Y" => DragonSlayerAction::Quit,
            _ => {
                self.screen = GameScreen::Town;
                DragonSlayerAction::SaveGame
            }
        }
    }

    fn handle_victory(&mut self, _input: &str) -> DragonSlayerAction {
        // Reset character for next dragon kill attempt (keeps some progress)
        DragonSlayerAction::GameComplete {
            dragon_kills: self.state.dragon_kills,
        }
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    fn get_usable_skill(&self) -> Option<String> {
        for skill in SKILL_DATA {
            if self.state.has_skill(skill.key) && self.state.can_use_skill(skill.key) {
                return Some(skill.key.to_string());
            }
        }
        None
    }

    fn check_level_up_message(&self) -> Option<String> {
        if let Some(master) = get_master(self.state.level) {
            if self.state.experience >= master.xp_required {
                return Some(format!(
                    "You can challenge {} at Turgon's Training!",
                    master.name
                ));
            }
        }
        None
    }
}

fn romance_result_message(result: &RomanceResult) -> String {
    match result {
        RomanceResult::Success { message, .. } => message.clone(),
        RomanceResult::Failure { message } => message.clone(),
        RomanceResult::AlreadyMarried { message } => message.clone(),
        RomanceResult::DailyLimitReached { message } => message.clone(),
        RomanceResult::ProposalAccepted { message } => message.clone(),
        RomanceResult::ProposalRejected { message } => message.clone(),
        RomanceResult::Divorced { message } => message.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = DragonSlayerFlow::new();
        assert!(matches!(flow.screen, GameScreen::CharacterCreation { .. }));
    }

    #[test]
    fn test_character_creation() {
        let mut flow = DragonSlayerFlow::new();

        // Enter name
        for ch in "Hero".chars() {
            flow.handle_char(ch);
        }
        flow.handle_char('\r');

        // Should be at sex selection
        assert!(matches!(
            flow.screen,
            GameScreen::CharacterCreation { step: CreationStep::SelectSex }
        ));

        // Select male
        flow.handle_char('M');

        // Should be at intro
        assert!(matches!(flow.screen, GameScreen::Intro));
        assert_eq!(flow.state.char_name, "HERO");
    }

    #[test]
    fn test_town_navigation() {
        let mut flow = DragonSlayerFlow::from_state(GameState::new("Test".to_string(), Sex::Male));
        flow.screen = GameScreen::Town;

        flow.handle_char('F');
        assert!(matches!(flow.screen, GameScreen::Forest));
    }

    #[test]
    fn test_forest_combat() {
        let mut flow = DragonSlayerFlow::from_state(GameState::new("Test".to_string(), Sex::Male));
        flow.screen = GameScreen::Forest;

        // Hunt for monsters
        flow.handle_char('H');

        // Should either be in combat or have an event
        assert!(
            matches!(flow.screen, GameScreen::Combat { .. }) ||
            matches!(flow.screen, GameScreen::ForestEvent { .. }) ||
            matches!(flow.screen, GameScreen::Forest)
        );
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = DragonSlayerFlow::from_state(GameState::new("Test".to_string(), Sex::Male));
        flow.screen = GameScreen::Town;

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        // Decline quit
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::Town));
    }
}
