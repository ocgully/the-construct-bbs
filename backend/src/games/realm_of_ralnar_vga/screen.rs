//! Game screens and state machine for Realm of Ralnar VGA
//!
//! Defines all possible game screens and handles transitions between them.

use super::state::GameState;
use serde::{Deserialize, Serialize};

/// All possible game screens
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameScreen {
    /// Title screen with animated logo
    Title,

    /// New game - character naming
    NewGame,

    /// File select / load game
    LoadGame,

    /// Main overworld/dungeon exploration
    Explore,

    /// In-game menu (items, equip, status, etc.)
    Menu(MenuScreen),

    /// Talking to an NPC
    Dialogue { npc_id: String, dialogue_index: u32 },

    /// Shopping at a store
    Shop { shop_id: String },

    /// Inn - rest and save
    Inn { inn_id: String },

    /// Combat encounter
    Battle(BattleState),

    /// Viewing the world map
    WorldMap,

    /// Game over screen
    GameOver,

    /// Victory / credits
    Victory,
}

/// Sub-screens within the in-game menu
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuScreen {
    /// Main menu overlay
    Main,
    /// Item inventory
    Items,
    /// Equipment management
    Equip,
    /// Character status
    Status,
    /// Magic/skills
    Magic,
    /// Quest log
    Quest,
    /// Save game
    Save,
    /// Settings
    Config,
}

/// Battle state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleState {
    pub enemy_id: String,
    pub enemy_hp: u16,
    pub enemy_max_hp: u16,
    pub turn: BattleTurn,
    pub phase: BattlePhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleTurn {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattlePhase {
    /// Showing battle start message
    Start,
    /// Waiting for player command
    CommandSelect,
    /// Player is attacking
    PlayerAttack,
    /// Enemy is attacking
    EnemyAttack,
    /// Showing damage/effect results
    ShowResult,
    /// Battle won
    Victory,
    /// Battle lost
    Defeat,
    /// Player fled
    Fled,
}

/// Actions that can result from input handling
#[derive(Debug, Clone)]
pub enum GameAction {
    /// No action needed
    None,
    /// Re-render the current screen
    Render,
    /// Change to a different screen
    ChangeScreen(GameScreen),
    /// Save the game state
    Save,
    /// Play a sound effect
    PlaySound(String),
    /// Play background music
    PlayMusic(String),
    /// Exit the game
    Exit,
}

/// Flow controller for game state machine
pub struct RalnarFlow {
    pub screen: GameScreen,
    pub state: GameState,
    /// Current menu selection index
    pub menu_cursor: usize,
    /// Message being displayed (dialogue, battle text, etc.)
    pub current_message: Option<String>,
    /// Animation frame counter
    pub anim_frame: u32,
}

impl RalnarFlow {
    pub fn new() -> Self {
        Self {
            screen: GameScreen::Title,
            state: GameState::default(),
            menu_cursor: 0,
            current_message: None,
            anim_frame: 0,
        }
    }

    pub fn with_state(state: GameState) -> Self {
        Self {
            screen: GameScreen::Explore,
            state,
            menu_cursor: 0,
            current_message: None,
            anim_frame: 0,
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, key: &str) -> GameAction {
        match &self.screen {
            GameScreen::Title => self.handle_title_input(key),
            GameScreen::NewGame => self.handle_newgame_input(key),
            GameScreen::Explore => self.handle_explore_input(key),
            GameScreen::Menu(menu) => self.handle_menu_input(key, menu.clone()),
            GameScreen::Dialogue { .. } => self.handle_dialogue_input(key),
            GameScreen::Battle(_) => self.handle_battle_input(key),
            GameScreen::Shop { .. } => self.handle_shop_input(key),
            _ => GameAction::None,
        }
    }

    fn handle_title_input(&mut self, key: &str) -> GameAction {
        match key {
            "Enter" | " " => {
                self.menu_cursor = 0;
                GameAction::ChangeScreen(GameScreen::NewGame)
            }
            "l" | "L" => GameAction::ChangeScreen(GameScreen::LoadGame),
            _ => GameAction::None,
        }
    }

    fn handle_newgame_input(&mut self, key: &str) -> GameAction {
        match key {
            "Enter" => {
                self.state = GameState::new("Hero");
                self.screen = GameScreen::Explore;
                GameAction::Render
            }
            _ => GameAction::None,
        }
    }

    fn handle_explore_input(&mut self, key: &str) -> GameAction {
        use super::state::Direction;

        match key {
            "ArrowUp" | "w" | "W" | "8" => {
                self.state.position.facing = Direction::Up;
                self.try_move(0, -1)
            }
            "ArrowDown" | "s" | "S" | "2" => {
                self.state.position.facing = Direction::Down;
                self.try_move(0, 1)
            }
            "ArrowLeft" | "a" | "A" | "4" => {
                self.state.position.facing = Direction::Left;
                self.try_move(-1, 0)
            }
            "ArrowRight" | "d" | "D" | "6" => {
                self.state.position.facing = Direction::Right;
                self.try_move(1, 0)
            }
            "Escape" | "m" | "M" => {
                self.menu_cursor = 0;
                GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Main))
            }
            "Enter" | " " => {
                // Interact with tile in front
                self.interact_ahead()
            }
            _ => GameAction::None,
        }
    }

    fn handle_menu_input(&mut self, key: &str, menu: MenuScreen) -> GameAction {
        match key {
            "Escape" | "m" | "M" => {
                if menu == MenuScreen::Main {
                    GameAction::ChangeScreen(GameScreen::Explore)
                } else {
                    GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Main))
                }
            }
            "ArrowUp" | "w" | "W" => {
                if self.menu_cursor > 0 {
                    self.menu_cursor -= 1;
                }
                GameAction::Render
            }
            "ArrowDown" | "s" | "S" => {
                self.menu_cursor += 1;
                GameAction::Render
            }
            "Enter" | " " => self.select_menu_option(&menu),
            _ => GameAction::None,
        }
    }

    fn handle_dialogue_input(&mut self, key: &str) -> GameAction {
        match key {
            "Enter" | " " | "Escape" => GameAction::ChangeScreen(GameScreen::Explore),
            _ => GameAction::None,
        }
    }

    fn handle_battle_input(&mut self, key: &str) -> GameAction {
        // Simplified battle input
        match key {
            "Enter" | " " | "a" | "A" => {
                // Attack
                GameAction::Render
            }
            "Escape" | "r" | "R" => {
                // Try to run
                GameAction::ChangeScreen(GameScreen::Explore)
            }
            _ => GameAction::None,
        }
    }

    fn handle_shop_input(&mut self, key: &str) -> GameAction {
        match key {
            "Escape" => GameAction::ChangeScreen(GameScreen::Explore),
            _ => GameAction::None,
        }
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> GameAction {
        let new_x = (self.state.position.x as i32 + dx).max(0) as u32;
        let new_y = (self.state.position.y as i32 + dy).max(0) as u32;

        // TODO: Check collision with map data
        self.state.position.x = new_x;
        self.state.position.y = new_y;
        self.state.take_step();

        // TODO: Check for random encounters
        GameAction::Render
    }

    fn interact_ahead(&mut self) -> GameAction {
        // TODO: Check for NPCs, chests, etc. in front of player
        GameAction::None
    }

    fn select_menu_option(&mut self, menu: &MenuScreen) -> GameAction {
        match menu {
            MenuScreen::Main => {
                match self.menu_cursor {
                    0 => GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Items)),
                    1 => GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Equip)),
                    2 => GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Status)),
                    3 => GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Magic)),
                    4 => GameAction::ChangeScreen(GameScreen::Menu(MenuScreen::Save)),
                    _ => GameAction::ChangeScreen(GameScreen::Explore),
                }
            }
            _ => GameAction::None,
        }
    }

    /// Advance animation frame
    pub fn tick(&mut self) {
        self.anim_frame = self.anim_frame.wrapping_add(1);
    }
}

impl Default for RalnarFlow {
    fn default() -> Self {
        Self::new()
    }
}
