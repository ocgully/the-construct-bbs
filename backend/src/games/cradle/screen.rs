//! Screen state machine for Cradle
//!
//! Manages game screens and input handling.

use super::state::GameState;
use super::data::{TierLevel, PATHS, TECHNIQUES};
use super::events::TrialState;

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// New game intro
    Intro,
    /// Character creation
    CharacterCreation { stage: CreationStage },
    /// Main cultivation hub
    MainMenu,
    /// Path selection/management
    PathSelection { selecting_secondary: bool },
    /// Technique shop/training
    Techniques,
    /// Advancement trial
    Trial { trial: TrialState },
    /// Mentor interaction
    Mentor,
    /// Prestige/ascension screen
    Prestige,
    /// Prestige shop (spend ascension points)
    PrestigeShop,
    /// Statistics view
    Stats,
    /// Leaderboard
    Leaderboard,
    /// Respec confirmation
    Respec,
    /// Game over (reached Transcendent or ascended)
    Victory,
    /// Quit confirmation
    ConfirmQuit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreationStage {
    Name,
    Confirm,
}

/// Actions returned by CradleFlow for session.rs to handle
#[derive(Debug, Clone)]
pub enum CradleAction {
    Continue,
    Render(String),
    Echo(String),
    SaveGame,
    GameOver { final_tier: TierLevel, ascension_points: u64 },
    Quit,
}

/// Cradle game flow state machine
pub struct CradleFlow {
    pub state: GameState,
    pub screen: GameScreen,
    input_buffer: String,
    pub catchup_message: Option<String>,
}

impl CradleFlow {
    /// Create new game flow (for character creation)
    pub fn new() -> Self {
        Self {
            state: GameState::new("".to_string()),
            screen: GameScreen::Intro,
            input_buffer: String::new(),
            catchup_message: None,
        }
    }

    /// Resume game from loaded state
    pub fn from_state(state: GameState) -> Self {
        Self {
            state,
            screen: GameScreen::Intro,  // Show splash first
            input_buffer: String::new(),
            catchup_message: None,
        }
    }

    /// Get current screen for rendering
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> &GameState {
        &self.state
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> CradleAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return CradleAction::Echo("\x08 \x08".to_string());
            }
            return CradleAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return CradleAction::Continue;
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
            return CradleAction::Echo(ch.to_string());
        }

        CradleAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::MainMenu
                | GameScreen::CharacterCreation { stage: CreationStage::Confirm }
                | GameScreen::PathSelection { .. }
                | GameScreen::Techniques
                | GameScreen::Trial { .. }
                | GameScreen::Mentor
                | GameScreen::Prestige
                | GameScreen::PrestigeShop
                | GameScreen::Stats
                | GameScreen::Leaderboard
                | GameScreen::Respec
                | GameScreen::Victory
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> CradleAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen.clone() {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::CharacterCreation { stage } => self.handle_character_creation(&input, stage.clone()),
            GameScreen::MainMenu => self.handle_main_menu(&input),
            GameScreen::PathSelection { selecting_secondary } => self.handle_path_selection(&input, *selecting_secondary),
            GameScreen::Techniques => self.handle_techniques(&input),
            GameScreen::Trial { trial } => self.handle_trial(&input, trial.clone()),
            GameScreen::Mentor => self.handle_mentor(&input),
            GameScreen::Prestige => self.handle_prestige(&input),
            GameScreen::PrestigeShop => self.handle_prestige_shop(&input),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::Respec => self.handle_respec(&input),
            GameScreen::Victory => self.handle_victory(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> CradleAction {
        // Check if this is a new game or resuming
        if self.state.name.is_empty() {
            self.screen = GameScreen::CharacterCreation { stage: CreationStage::Name };
        } else {
            // Process offline catchup
            let ticks = super::tick::ticks_since_last_update(&self.state);
            if ticks > 0 {
                let result = super::tick::process_catchup_ticks(&mut self.state, ticks);
                if !result.event_messages.is_empty() {
                    self.catchup_message = Some(result.event_messages.join("\n"));
                }
            }
            self.screen = GameScreen::MainMenu;
        }
        CradleAction::SaveGame
    }

    fn handle_character_creation(&mut self, input: &str, stage: CreationStage) -> CradleAction {
        match stage {
            CreationStage::Name => {
                let name = input.trim();
                if name.len() < 2 {
                    self.state.last_message = Some("Name must be at least 2 characters.".to_string());
                    return CradleAction::SaveGame;
                }
                if name.len() > 20 {
                    self.state.last_message = Some("Name must be 20 characters or less.".to_string());
                    return CradleAction::SaveGame;
                }
                self.state.name = name.to_string();
                self.screen = GameScreen::CharacterCreation { stage: CreationStage::Confirm };
                CradleAction::SaveGame
            }
            CreationStage::Confirm => {
                match input {
                    "Y" => {
                        self.state.tutorial_completed = false;
                        self.screen = GameScreen::MainMenu;
                        CradleAction::SaveGame
                    }
                    "N" => {
                        self.state.name.clear();
                        self.screen = GameScreen::CharacterCreation { stage: CreationStage::Name };
                        CradleAction::SaveGame
                    }
                    _ => CradleAction::Continue,
                }
            }
        }
    }

    fn handle_main_menu(&mut self, input: &str) -> CradleAction {
        // Clear displayed messages
        self.state.last_message = None;
        self.catchup_message = None;

        match input {
            "C" => {
                // Cycle/Train - process a tick manually
                let result = super::tick::process_tick(&mut self.state);
                let mut msg = format!(
                    "Cycling complete. +{} madra, +{} stones, +{} insight.",
                    result.total_madra_gained,
                    result.total_stones_gained,
                    result.total_insight_gained
                );
                if let Some(tier) = result.tier_advanced {
                    msg.push_str(&format!(" BREAKTHROUGH to {}!", tier.name()));
                }
                if result.plateau_warning {
                    msg.push_str(" WARNING: You've hit a plateau!");
                }
                self.state.last_message = Some(msg);
                CradleAction::SaveGame
            }
            "P" => {
                // Path selection
                self.screen = GameScreen::PathSelection { selecting_secondary: false };
                CradleAction::SaveGame
            }
            "T" => {
                // Techniques
                self.screen = GameScreen::Techniques;
                CradleAction::SaveGame
            }
            "A" => {
                // Advancement trial
                if let Some(next_tier) = self.state.tier.next() {
                    if let Some(tier_data) = super::data::get_tier(next_tier) {
                        if tier_data.trial_required {
                            // Check requirements first
                            if self.state.madra < tier_data.madra_requirement {
                                self.state.last_message = Some(format!(
                                    "Need {} madra to attempt trial (have {}).",
                                    tier_data.madra_requirement, self.state.madra
                                ));
                                return CradleAction::SaveGame;
                            }
                            if self.state.insight < tier_data.insight_requirement {
                                self.state.last_message = Some(format!(
                                    "Need {} insight to attempt trial (have {}).",
                                    tier_data.insight_requirement, self.state.insight
                                ));
                                return CradleAction::SaveGame;
                            }
                            self.screen = GameScreen::Trial { trial: super::events::TrialState::new(next_tier) };
                            return CradleAction::SaveGame;
                        }
                    }
                }
                self.state.last_message = Some("No trial available at this tier.".to_string());
                CradleAction::SaveGame
            }
            "M" => {
                // Mentor
                self.screen = GameScreen::Mentor;
                CradleAction::SaveGame
            }
            "R" => {
                // Respec
                self.screen = GameScreen::Respec;
                CradleAction::SaveGame
            }
            "E" => {
                // Prestige/Ascension
                self.screen = GameScreen::Prestige;
                CradleAction::SaveGame
            }
            "S" => {
                // Stats
                self.screen = GameScreen::Stats;
                CradleAction::SaveGame
            }
            "L" => {
                // Leaderboard
                self.screen = GameScreen::Leaderboard;
                CradleAction::SaveGame
            }
            "Q" => {
                // Quit
                self.screen = GameScreen::ConfirmQuit;
                CradleAction::SaveGame
            }
            _ => CradleAction::Continue,
        }
    }

    fn handle_path_selection(&mut self, input: &str, selecting_secondary: bool) -> CradleAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return CradleAction::SaveGame;
        }

        if !selecting_secondary && input == "2" && self.state.tier >= TierLevel::Gold {
            // Switch to secondary path selection
            self.screen = GameScreen::PathSelection { selecting_secondary: true };
            return CradleAction::SaveGame;
        }

        // Parse path selection by number
        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= PATHS.len() {
                let path = &PATHS[idx - 1];

                let result = if selecting_secondary {
                    super::economy::select_secondary_path(&mut self.state, path.key)
                } else {
                    super::economy::select_path(&mut self.state, path.key)
                };

                match result {
                    Ok(()) => {
                        self.state.last_message = Some(format!("Selected {}!", path.name));
                        self.screen = GameScreen::MainMenu;
                    }
                    Err(e) => {
                        self.state.last_message = Some(e.to_string());
                    }
                }
                return CradleAction::SaveGame;
            }
        }

        CradleAction::Continue
    }

    fn handle_techniques(&mut self, input: &str) -> CradleAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return CradleAction::SaveGame;
        }

        // Get available techniques for player's paths
        let available: Vec<_> = TECHNIQUES.iter()
            .filter(|t| {
                self.state.primary_path.as_ref().map(|p| p == t.path_key).unwrap_or(false)
                    || self.state.secondary_path.as_ref().map(|p| p == t.path_key).unwrap_or(false)
            })
            .filter(|t| t.tier_requirement <= self.state.tier)
            .collect();

        // Parse technique selection
        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= available.len() {
                let technique = available[idx - 1];

                // Check if already owned
                if self.state.technique_levels.contains_key(technique.key) {
                    // Try to upgrade
                    match super::economy::upgrade_technique(&mut self.state, technique.key) {
                        Ok(level) => {
                            self.state.last_message = Some(format!(
                                "{} upgraded to level {}!",
                                technique.name, level
                            ));
                        }
                        Err(e) => {
                            self.state.last_message = Some(e.to_string());
                        }
                    }
                } else {
                    // Try to purchase
                    match super::economy::purchase_technique(&mut self.state, technique.key) {
                        Ok(()) => {
                            self.state.last_message = Some(format!("Learned {}!", technique.name));
                        }
                        Err(e) => {
                            self.state.last_message = Some(e.to_string());
                        }
                    }
                }
                return CradleAction::SaveGame;
            }
        }

        CradleAction::Continue
    }

    fn handle_trial(&mut self, input: &str, mut trial: TrialState) -> CradleAction {
        if input == "Q" && trial.stage == 1 {
            // Can only quit at start
            self.screen = GameScreen::MainMenu;
            return CradleAction::SaveGame;
        }

        // Parse choice (1, 2, or 3)
        if let Ok(choice) = input.parse::<usize>() {
            if choice >= 1 && choice <= 3 {
                let (_success, message) = super::events::process_trial_choice(&mut trial, choice - 1, &self.state);
                self.state.last_message = Some(message.to_string());

                if trial.is_complete() {
                    // Trial finished
                    match super::events::complete_trial(&trial, &mut self.state) {
                        Ok(new_tier) => {
                            self.state.last_message = Some(format!(
                                "BREAKTHROUGH! You have achieved {}!",
                                new_tier.name()
                            ));
                            // Check for transcendence
                            if new_tier == TierLevel::Transcendent {
                                self.screen = GameScreen::Victory;
                                return CradleAction::SaveGame;
                            }
                        }
                        Err(e) => {
                            self.state.last_message = Some(e.to_string());
                        }
                    }
                    self.screen = GameScreen::MainMenu;
                } else {
                    self.screen = GameScreen::Trial { trial };
                }
                return CradleAction::SaveGame;
            }
        }

        CradleAction::Continue
    }

    fn handle_mentor(&mut self, input: &str) -> CradleAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return CradleAction::SaveGame;
        }

        if input == "H" || input == "1" {
            // Get hint
            if let Some(hint) = super::events::get_mentor_hint(&mut self.state) {
                self.state.last_message = Some(hint);
            }
            return CradleAction::SaveGame;
        }

        if input == "W" || input == "2" {
            // Check for warnings
            if let Some(warning) = super::events::should_mentor_warn(&self.state) {
                self.state.last_message = Some(warning);
            } else {
                self.state.last_message = Some("Your mentor has no specific warnings.".to_string());
            }
            return CradleAction::SaveGame;
        }

        CradleAction::Continue
    }

    fn handle_prestige(&mut self, input: &str) -> CradleAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return CradleAction::SaveGame;
        }

        if input == "A" || input == "1" {
            // Perform prestige
            if self.state.tier < TierLevel::Gold {
                self.state.last_message = Some("Must reach Gold tier to ascend.".to_string());
                return CradleAction::SaveGame;
            }

            let points = self.state.prestige();
            self.state.last_message = Some(format!(
                "Ascended! Gained {} ascension points. Total: {}",
                points, self.state.prestige.ascension_points
            ));
            self.screen = GameScreen::MainMenu;
            return CradleAction::SaveGame;
        }

        if input == "S" || input == "2" {
            // Go to prestige shop
            self.screen = GameScreen::PrestigeShop;
            return CradleAction::SaveGame;
        }

        CradleAction::Continue
    }

    fn handle_prestige_shop(&mut self, input: &str) -> CradleAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Prestige;
            return CradleAction::SaveGame;
        }

        let upgrades = [
            ("1", "madra_boost", "Madra Multiplier +10%", 100),
            ("2", "insight_boost", "Insight Multiplier +10%", 100),
            ("3", "stone_boost", "Spirit Stone Multiplier +10%", 150),
            ("4", "speed_boost", "Unlock Speed +10%", 200),
            ("5", "starting_tier", "Start at Higher Tier", 500),
        ];

        for (key, upgrade, name, _) in &upgrades {
            if input == *key {
                match self.state.buy_prestige_upgrade(upgrade) {
                    Ok(()) => {
                        self.state.last_message = Some(format!("Purchased {}!", name));
                    }
                    Err(e) => {
                        self.state.last_message = Some(e.to_string());
                    }
                }
                return CradleAction::SaveGame;
            }
        }

        CradleAction::Continue
    }

    fn handle_stats(&mut self, _input: &str) -> CradleAction {
        self.screen = GameScreen::MainMenu;
        CradleAction::SaveGame
    }

    fn handle_leaderboard(&mut self, _input: &str) -> CradleAction {
        self.screen = GameScreen::MainMenu;
        CradleAction::SaveGame
    }

    fn handle_respec(&mut self, input: &str) -> CradleAction {
        match input {
            "Y" => {
                match self.state.respec() {
                    Ok(()) => {
                        self.state.last_message = Some("Respec complete. Your path has been cleared.".to_string());
                    }
                    Err(e) => {
                        self.state.last_message = Some(e.to_string());
                    }
                }
                self.screen = GameScreen::MainMenu;
                CradleAction::SaveGame
            }
            _ => {
                self.screen = GameScreen::MainMenu;
                CradleAction::SaveGame
            }
        }
    }

    fn handle_victory(&mut self, _input: &str) -> CradleAction {
        // Any input returns to main menu or triggers game over
        CradleAction::GameOver {
            final_tier: self.state.tier,
            ascension_points: self.state.potential_ascension_points(),
        }
    }

    fn handle_confirm_quit(&mut self, input: &str) -> CradleAction {
        match input {
            "Y" => CradleAction::Quit,
            _ => {
                self.screen = GameScreen::MainMenu;
                CradleAction::SaveGame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_starts_at_intro() {
        let flow = CradleFlow::new();
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_character_creation_flow() {
        let mut flow = CradleFlow::new();
        flow.screen = GameScreen::CharacterCreation { stage: CreationStage::Name };

        // Enter name
        for ch in "TestHero".chars() {
            flow.handle_char(ch);
        }
        flow.handle_char('\r');

        // Should be at confirmation
        assert!(matches!(flow.screen, GameScreen::CharacterCreation { stage: CreationStage::Confirm }));
        assert_eq!(flow.state.name, "TESTHERO");

        // Confirm
        flow.handle_char('Y');
        assert!(matches!(flow.screen, GameScreen::MainMenu));
    }

    #[test]
    fn test_cycling_increases_resources() {
        let mut flow = CradleFlow::new();
        flow.state.name = "Test".to_string();
        flow.state.tier = TierLevel::Copper;
        flow.state.stats.madra_regen = 10;
        flow.screen = GameScreen::MainMenu;

        let initial_madra = flow.state.madra;
        flow.handle_char('C');  // Cycle

        assert!(flow.state.madra > initial_madra);
    }

    #[test]
    fn test_path_selection() {
        let mut flow = CradleFlow::new();
        flow.state.name = "Test".to_string();
        flow.screen = GameScreen::PathSelection { selecting_secondary: false };

        flow.handle_char('1');  // Select first path

        assert!(flow.state.primary_path.is_some());
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = CradleFlow::new();
        flow.state.name = "Test".to_string();
        flow.screen = GameScreen::MainMenu;

        flow.handle_char('Q');
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        let action = flow.handle_char('Y');
        assert!(matches!(action, CradleAction::Quit));
    }

    #[test]
    fn test_prestige_requires_gold() {
        let mut flow = CradleFlow::new();
        flow.state.name = "Test".to_string();
        flow.state.tier = TierLevel::Jade;  // Not Gold yet
        flow.screen = GameScreen::Prestige;

        flow.handle_char('A');  // Try to ascend

        // Should fail with message
        assert!(flow.state.last_message.is_some());
        assert!(flow.state.last_message.as_ref().unwrap().contains("Gold"));
    }
}
