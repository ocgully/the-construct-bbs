//! Realm of Ralnar - Game Flow State Machine
//! Handles screen transitions and input processing

use super::screen::GameScreen;
use super::state::{GameState, Direction, PartyMember, CharacterClass};

/// State machine for battle logic
#[derive(Debug, Clone, Default)]
pub struct BattleState {
    /// Current turn in battle
    pub turn: u32,
    /// Enemy IDs in this battle
    pub enemies: Vec<String>,
    /// Selected action for current turn
    pub selected_action: Option<BattleAction>,
    /// Selected target index
    pub selected_target: Option<usize>,
    /// Battle log messages
    pub messages: Vec<String>,
}

/// Actions available in battle
#[derive(Debug, Clone)]
pub enum BattleAction {
    Attack,
    Magic { spell_id: String },
    Item { item_id: String },
    Defend,
    Flee,
}

/// State for dialogue interactions
#[derive(Debug, Clone, Default)]
pub struct DialogueState {
    /// Current dialogue node
    pub node_id: String,
    /// Available choices (if any)
    pub choices: Vec<String>,
    /// Current text being displayed
    pub current_text: String,
}

/// Actions returned by the flow for session handling
#[derive(Debug, Clone)]
pub enum RalnarAction {
    /// Continue - no output needed
    Continue,
    /// Render screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Player quit to main menu
    Quit,
    /// Load a new map
    LoadMap { map_id: String },
    /// Start a battle encounter
    StartBattle { enemies: Vec<String> },
    /// End current battle
    EndBattle { victory: bool },
    /// Play a sound effect
    PlaySound { sound_id: String },
    /// Show a cutscene
    ShowCutscene { scene_id: String },
    /// Game completed
    GameComplete,
}

/// Realm of Ralnar game flow state machine
pub struct RalnarFlow {
    /// Current screen being displayed
    pub screen: GameScreen,
    /// Game state
    state: GameState,
    /// Battle state (when in battle)
    battle_state: Option<BattleState>,
    /// Dialogue state (when in dialogue)
    dialogue_state: Option<DialogueState>,
    /// Input buffer for text entry
    input_buffer: String,
}

impl RalnarFlow {
    /// Create a new game
    pub fn new(user_id: i64, handle: String) -> Self {
        Self {
            screen: GameScreen::Intro,
            state: GameState::new(user_id, handle),
            battle_state: None,
            dialogue_state: None,
            input_buffer: String::new(),
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState) -> Self {
        // Determine starting screen based on state
        let screen = if state.party.is_alive() {
            GameScreen::Exploring {
                map_id: state.current_map.clone(),
            }
        } else {
            GameScreen::GameOver
        };

        Self {
            screen,
            state,
            battle_state: None,
            dialogue_state: None,
            input_buffer: String::new(),
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

    /// Get battle state if in battle
    pub fn battle_state(&self) -> Option<&BattleState> {
        self.battle_state.as_ref()
    }

    /// Get dialogue state if in dialogue
    pub fn dialogue_state(&self) -> Option<&DialogueState> {
        self.dialogue_state.as_ref()
    }

    /// Handle line input (for text entry screens)
    pub fn handle_input(&mut self, input: &str) -> RalnarAction {
        let input = input.trim().to_uppercase();
        self.process_screen_input(&input)
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> RalnarAction {
        // Handle backspace
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return RalnarAction::Echo("\x08 \x08".to_string());
            }
            return RalnarAction::Continue;
        }

        // Handle enter
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control characters
        if ch.is_control() {
            return RalnarAction::Continue;
        }

        // For single-key screens, process immediately
        if self.screen.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input for text entry
        if self.input_buffer.len() < 30 {
            self.input_buffer.push(ch);
            return RalnarAction::Echo(ch.to_string());
        }

        RalnarAction::Continue
    }

    /// Process buffered input
    fn process_input(&mut self) -> RalnarAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();
        self.process_screen_input(&input)
    }

    /// Process input based on current screen
    fn process_screen_input(&mut self, input: &str) -> RalnarAction {
        match &self.screen {
            GameScreen::Intro => self.handle_intro(input),
            GameScreen::MainMenu => self.handle_main_menu(input),
            GameScreen::Exploring { .. } => self.handle_exploring(input),
            GameScreen::Dialogue { .. } => self.handle_dialogue(input),
            GameScreen::Shop { .. } => self.handle_shop(input),
            GameScreen::Inn => self.handle_inn(input),
            GameScreen::Battle => self.handle_battle(input),
            GameScreen::BattleVictory => self.handle_battle_victory(input),
            GameScreen::BattleDefeat => self.handle_battle_defeat(input),
            GameScreen::Inventory => self.handle_inventory(input),
            GameScreen::Equipment => self.handle_equipment(input),
            GameScreen::PartyStatus => self.handle_party_status(input),
            GameScreen::Magic => self.handle_magic(input),
            GameScreen::QuestLog => self.handle_quest_log(input),
            GameScreen::WorldMap => self.handle_world_map(input),
            GameScreen::Cutscene { .. } => self.handle_cutscene(input),
            GameScreen::GameOver => self.handle_game_over(input),
            GameScreen::Credits => self.handle_credits(input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(input),
        }
    }

    // ========================================================================
    // SCREEN HANDLERS
    // ========================================================================

    fn handle_intro(&mut self, _input: &str) -> RalnarAction {
        self.screen = GameScreen::MainMenu;
        RalnarAction::SaveGame
    }

    fn handle_main_menu(&mut self, input: &str) -> RalnarAction {
        match input {
            "N" | "1" => {
                // New game - start exploring
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
                RalnarAction::SaveGame
            }
            "C" | "2" => {
                // Continue - already have state, start exploring
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
                RalnarAction::SaveGame
            }
            "Q" | "3" => {
                self.screen = GameScreen::ConfirmQuit;
                RalnarAction::SaveGame
            }
            _ => RalnarAction::Continue,
        }
    }

    fn handle_exploring(&mut self, input: &str) -> RalnarAction {
        self.state.last_message = None;

        match input {
            // Movement
            "W" | "8" => {
                self.state.move_direction(Direction::Up);
                RalnarAction::SaveGame
            }
            "S" | "2" => {
                self.state.move_direction(Direction::Down);
                RalnarAction::SaveGame
            }
            "A" | "4" => {
                self.state.move_direction(Direction::Left);
                RalnarAction::SaveGame
            }
            "D" | "6" => {
                self.state.move_direction(Direction::Right);
                RalnarAction::SaveGame
            }
            // Menu access
            "I" => {
                self.screen = GameScreen::Inventory;
                RalnarAction::SaveGame
            }
            "E" => {
                self.screen = GameScreen::Equipment;
                RalnarAction::SaveGame
            }
            "P" => {
                self.screen = GameScreen::PartyStatus;
                RalnarAction::SaveGame
            }
            "M" => {
                self.screen = GameScreen::Magic;
                RalnarAction::SaveGame
            }
            "J" => {
                self.screen = GameScreen::QuestLog;
                RalnarAction::SaveGame
            }
            "O" => {
                self.screen = GameScreen::WorldMap;
                RalnarAction::SaveGame
            }
            "Q" => {
                self.screen = GameScreen::ConfirmQuit;
                RalnarAction::SaveGame
            }
            _ => RalnarAction::Continue,
        }
    }

    fn handle_dialogue(&mut self, input: &str) -> RalnarAction {
        // Simple dialogue - any key advances, Q quits
        if input == "Q" {
            self.dialogue_state = None;
            self.screen = GameScreen::Exploring {
                map_id: self.state.current_map.clone(),
            };
            return RalnarAction::SaveGame;
        }

        // Check for choice selection
        if let Some(ref mut dialogue) = self.dialogue_state {
            if let Ok(choice) = input.parse::<usize>() {
                if choice > 0 && choice <= dialogue.choices.len() {
                    // Handle choice - for now just end dialogue
                    self.dialogue_state = None;
                    self.screen = GameScreen::Exploring {
                        map_id: self.state.current_map.clone(),
                    };
                    return RalnarAction::SaveGame;
                }
            }
        }

        // Advance dialogue or end
        self.dialogue_state = None;
        self.screen = GameScreen::Exploring {
            map_id: self.state.current_map.clone(),
        };
        RalnarAction::SaveGame
    }

    fn handle_shop(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Exploring {
                map_id: self.state.current_map.clone(),
            };
            return RalnarAction::SaveGame;
        }

        // TODO: Implement shopping logic
        RalnarAction::Continue
    }

    fn handle_inn(&mut self, input: &str) -> RalnarAction {
        match input {
            "R" | "1" => {
                // Rest
                self.state.party.rest_at_inn();
                self.state.last_message = Some("Your party rests and recovers.".to_string());
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
                RalnarAction::SaveGame
            }
            "Q" | "B" => {
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
                RalnarAction::SaveGame
            }
            _ => RalnarAction::Continue,
        }
    }

    fn handle_battle(&mut self, input: &str) -> RalnarAction {
        match input {
            "A" | "1" => {
                // Attack
                self.state.last_message = Some("You attack!".to_string());
                // TODO: Implement combat logic
                RalnarAction::SaveGame
            }
            "M" | "2" => {
                // Magic
                self.screen = GameScreen::Magic;
                RalnarAction::SaveGame
            }
            "I" | "3" => {
                // Item
                self.screen = GameScreen::Inventory;
                RalnarAction::SaveGame
            }
            "D" | "4" => {
                // Defend
                self.state.last_message = Some("You defend!".to_string());
                RalnarAction::SaveGame
            }
            "F" | "5" => {
                // Flee
                // TODO: Implement flee logic
                self.battle_state = None;
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
                self.state.last_message = Some("You fled from battle!".to_string());
                RalnarAction::SaveGame
            }
            _ => RalnarAction::Continue,
        }
    }

    fn handle_battle_victory(&mut self, _input: &str) -> RalnarAction {
        self.battle_state = None;
        self.screen = GameScreen::Exploring {
            map_id: self.state.current_map.clone(),
        };
        RalnarAction::EndBattle { victory: true }
    }

    fn handle_battle_defeat(&mut self, _input: &str) -> RalnarAction {
        self.battle_state = None;
        self.screen = GameScreen::GameOver;
        RalnarAction::EndBattle { victory: false }
    }

    fn handle_inventory(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" {
            // Return to previous screen
            if self.battle_state.is_some() {
                self.screen = GameScreen::Battle;
            } else {
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
            }
            return RalnarAction::SaveGame;
        }

        // TODO: Implement inventory item selection/use
        RalnarAction::Continue
    }

    fn handle_equipment(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Exploring {
                map_id: self.state.current_map.clone(),
            };
            return RalnarAction::SaveGame;
        }

        // TODO: Implement equipment management
        RalnarAction::Continue
    }

    fn handle_party_status(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::Exploring {
                map_id: self.state.current_map.clone(),
            };
            return RalnarAction::SaveGame;
        }
        RalnarAction::Continue
    }

    fn handle_magic(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" {
            if self.battle_state.is_some() {
                self.screen = GameScreen::Battle;
            } else {
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
            }
            return RalnarAction::SaveGame;
        }

        // TODO: Implement spell selection
        RalnarAction::Continue
    }

    fn handle_quest_log(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" || !input.is_empty() {
            self.screen = GameScreen::Exploring {
                map_id: self.state.current_map.clone(),
            };
            return RalnarAction::SaveGame;
        }
        RalnarAction::Continue
    }

    fn handle_world_map(&mut self, input: &str) -> RalnarAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Exploring {
                map_id: self.state.current_map.clone(),
            };
            return RalnarAction::SaveGame;
        }

        // TODO: Implement world map navigation
        RalnarAction::Continue
    }

    fn handle_cutscene(&mut self, _input: &str) -> RalnarAction {
        // Any input advances/ends cutscene
        self.screen = GameScreen::Exploring {
            map_id: self.state.current_map.clone(),
        };
        RalnarAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> RalnarAction {
        self.screen = GameScreen::Credits;
        RalnarAction::SaveGame
    }

    fn handle_credits(&mut self, _input: &str) -> RalnarAction {
        RalnarAction::Quit
    }

    fn handle_confirm_quit(&mut self, input: &str) -> RalnarAction {
        match input {
            "Y" => RalnarAction::Quit,
            _ => {
                self.screen = GameScreen::Exploring {
                    map_id: self.state.current_map.clone(),
                };
                RalnarAction::SaveGame
            }
        }
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Start a battle with the given enemies
    pub fn start_battle(&mut self, enemies: Vec<String>) {
        self.battle_state = Some(BattleState {
            turn: 1,
            enemies,
            selected_action: None,
            selected_target: None,
            messages: Vec::new(),
        });
        self.screen = GameScreen::Battle;
    }

    /// Start a dialogue with an NPC
    pub fn start_dialogue(&mut self, npc_id: String, initial_text: String) {
        self.dialogue_state = Some(DialogueState {
            node_id: "start".to_string(),
            choices: Vec::new(),
            current_text: initial_text,
        });
        self.screen = GameScreen::Dialogue { npc_id };
    }

    /// Enter a shop
    pub fn enter_shop(&mut self, shop_id: String) {
        self.screen = GameScreen::Shop { shop_id };
    }

    /// Enter an inn
    pub fn enter_inn(&mut self) {
        self.screen = GameScreen::Inn;
    }

    /// Change the current map
    pub fn change_map(&mut self, map_id: String, position: (u32, u32)) {
        self.state.current_map = map_id.clone();
        self.state.position = position;
        self.screen = GameScreen::Exploring { map_id };
    }

    /// Add a party member
    pub fn add_party_member(&mut self, id: String, name: String, class: CharacterClass, is_brother: bool) -> bool {
        let member = PartyMember::new(id, name, class, is_brother);
        self.state.party.add_member(member)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = RalnarFlow::new(1, "TestPlayer".to_string());
        assert!(matches!(flow.screen, GameScreen::Intro));
        assert_eq!(flow.state.user_id, 1);
    }

    #[test]
    fn test_from_state() {
        let state = GameState::new(1, "TestPlayer".to_string());
        let flow = RalnarFlow::from_state(state);
        assert!(matches!(flow.screen, GameScreen::Exploring { .. }));
    }

    #[test]
    fn test_intro_to_main_menu() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_main_menu_to_exploring() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.screen = GameScreen::MainMenu;
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::Exploring { .. }));
    }

    #[test]
    fn test_exploring_to_inventory() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.screen = GameScreen::Exploring {
            map_id: "test".to_string(),
        };
        flow.handle_char('I');
        assert!(matches!(flow.screen, GameScreen::Inventory));
    }

    #[test]
    fn test_movement() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.screen = GameScreen::Exploring {
            map_id: "test".to_string(),
        };
        let initial_pos = flow.state.position;

        flow.handle_char('W');
        assert_eq!(flow.state.position.1, initial_pos.1.saturating_sub(1));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.screen = GameScreen::Exploring {
            map_id: "test".to_string(),
        };

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        // Decline quit
        flow.handle_char('N');
        assert!(matches!(flow.screen, GameScreen::Exploring { .. }));
    }

    #[test]
    fn test_confirm_quit() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.screen = GameScreen::ConfirmQuit;

        let action = flow.handle_char('Y');
        assert!(matches!(action, RalnarAction::Quit));
    }

    #[test]
    fn test_start_battle() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.start_battle(vec!["goblin".to_string()]);

        assert!(matches!(flow.screen, GameScreen::Battle));
        assert!(flow.battle_state.is_some());
    }

    #[test]
    fn test_start_dialogue() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.start_dialogue("elder".to_string(), "Welcome, traveler!".to_string());

        assert!(matches!(flow.screen, GameScreen::Dialogue { .. }));
        assert!(flow.dialogue_state.is_some());
    }

    #[test]
    fn test_inn_rest() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());

        // Damage the party
        if let Some(member) = flow.state.party.members.get_mut(0) {
            member.hp = 10;
            member.mp = 0;
        }

        flow.screen = GameScreen::Inn;
        flow.handle_char('R');

        // Should be healed
        let member = &flow.state.party.members[0];
        assert_eq!(member.hp, member.hp_max);
    }

    #[test]
    fn test_change_map() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());
        flow.change_map("new_map".to_string(), (10, 20));

        assert_eq!(flow.state.current_map, "new_map");
        assert_eq!(flow.state.position, (10, 20));
    }

    #[test]
    fn test_add_party_member() {
        let mut flow = RalnarFlow::new(1, "TestPlayer".to_string());

        let success = flow.add_party_member(
            "valeran".to_string(),
            "Valeran".to_string(),
            CharacterClass::Paladin,
            true,
        );

        assert!(success);
        assert_eq!(flow.state.party.members.len(), 2);
    }
}
