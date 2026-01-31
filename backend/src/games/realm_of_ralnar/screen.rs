//! Realm of Ralnar - Game Screens
//! Manages the various screens/states in the game

use serde::{Deserialize, Serialize};

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameScreen {
    /// Game introduction / story recap
    Intro,
    /// Main menu (new game, continue, options)
    MainMenu,
    /// Exploring a map/location
    Exploring { map_id: String },
    /// In dialogue with an NPC
    Dialogue { npc_id: String },
    /// Shopping at a store
    Shop { shop_id: String },
    /// Resting at an inn
    Inn,
    /// Active battle
    Battle,
    /// Victory screen after battle
    BattleVictory,
    /// Defeat screen after battle
    BattleDefeat,
    /// Viewing/managing inventory
    Inventory,
    /// Equipment management
    Equipment,
    /// Party status overview
    PartyStatus,
    /// Magic/spell selection
    Magic,
    /// Quest log / journal
    QuestLog,
    /// World map navigation
    WorldMap,
    /// Story cutscene
    Cutscene { scene_id: String },
    /// Game over screen
    GameOver,
    /// Credits roll
    Credits,
    /// Confirm quit prompt
    ConfirmQuit,
}

impl GameScreen {
    /// Check if this screen uses single-key input (immediate processing)
    /// vs buffered input (wait for Enter)
    pub fn is_single_key_screen(&self) -> bool {
        matches!(
            self,
            GameScreen::Intro
                | GameScreen::MainMenu
                | GameScreen::Exploring { .. }
                | GameScreen::Dialogue { .. }
                | GameScreen::Inn
                | GameScreen::Battle
                | GameScreen::BattleVictory
                | GameScreen::BattleDefeat
                | GameScreen::Inventory
                | GameScreen::Equipment
                | GameScreen::PartyStatus
                | GameScreen::Magic
                | GameScreen::QuestLog
                | GameScreen::WorldMap
                | GameScreen::Cutscene { .. }
                | GameScreen::GameOver
                | GameScreen::Credits
                | GameScreen::ConfirmQuit
        )
    }

    /// Check if this screen needs line input (buffered text entry)
    pub fn needs_line_input(&self) -> bool {
        matches!(
            self,
            GameScreen::Shop { .. } // For entering quantities
        )
    }

    /// Get a display name for the current screen
    pub fn display_name(&self) -> &'static str {
        match self {
            GameScreen::Intro => "Introduction",
            GameScreen::MainMenu => "Main Menu",
            GameScreen::Exploring { .. } => "Exploring",
            GameScreen::Dialogue { .. } => "Dialogue",
            GameScreen::Shop { .. } => "Shop",
            GameScreen::Inn => "Inn",
            GameScreen::Battle => "Battle",
            GameScreen::BattleVictory => "Victory!",
            GameScreen::BattleDefeat => "Defeat",
            GameScreen::Inventory => "Inventory",
            GameScreen::Equipment => "Equipment",
            GameScreen::PartyStatus => "Party Status",
            GameScreen::Magic => "Magic",
            GameScreen::QuestLog => "Quest Log",
            GameScreen::WorldMap => "World Map",
            GameScreen::Cutscene { .. } => "Cutscene",
            GameScreen::GameOver => "Game Over",
            GameScreen::Credits => "Credits",
            GameScreen::ConfirmQuit => "Quit?",
        }
    }
}

impl Default for GameScreen {
    fn default() -> Self {
        GameScreen::Intro
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_key_screens() {
        assert!(GameScreen::Intro.is_single_key_screen());
        assert!(GameScreen::MainMenu.is_single_key_screen());
        assert!(GameScreen::Battle.is_single_key_screen());
        assert!(GameScreen::Exploring { map_id: "test".to_string() }.is_single_key_screen());
    }

    #[test]
    fn test_line_input_screens() {
        assert!(GameScreen::Shop { shop_id: "test".to_string() }.needs_line_input());
        assert!(!GameScreen::Battle.needs_line_input());
        assert!(!GameScreen::Inventory.needs_line_input());
    }

    #[test]
    fn test_display_names() {
        assert_eq!(GameScreen::Battle.display_name(), "Battle");
        assert_eq!(GameScreen::Inn.display_name(), "Inn");
    }

    #[test]
    fn test_default_screen() {
        let screen: GameScreen = Default::default();
        assert_eq!(screen, GameScreen::Intro);
    }

    #[test]
    fn test_screen_equality() {
        let screen1 = GameScreen::Exploring { map_id: "castle".to_string() };
        let screen2 = GameScreen::Exploring { map_id: "castle".to_string() };
        let screen3 = GameScreen::Exploring { map_id: "forest".to_string() };

        assert_eq!(screen1, screen2);
        assert_ne!(screen1, screen3);
    }

    #[test]
    fn test_screen_serialization() {
        let screen = GameScreen::Exploring { map_id: "test_map".to_string() };
        let json = serde_json::to_string(&screen).unwrap();
        let restored: GameScreen = serde_json::from_str(&json).unwrap();
        assert_eq!(screen, restored);
    }
}
