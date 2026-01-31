//! Game screen state machine and flow control

use super::state::ProvinceState;
use super::data::{BuildingType, UnitType, AttackType, ThiefOp, SpellType, RACES, PERSONALITIES};

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// New game - select race
    SelectRace,
    /// New game - select personality
    SelectPersonality { race: String },
    /// New game - enter province name
    EnterName { race: String, personality: String },
    /// Main province overview
    Throne,
    /// Building management
    Build,
    /// Military training
    Military,
    /// Attack planning
    Attack { target_input: String },
    /// Thief operations
    Thieves { target_input: String },
    /// Magic/Spells
    Magic,
    /// Science research
    Science,
    /// Kingdom management
    Kingdom,
    /// Province stats and info
    Info,
    /// Leaderboard view
    Rankings,
    /// Age history
    #[allow(dead_code)] // Reserved for future implementation
    History,
    /// Help screen
    Help,
    /// Confirm quit
    ConfirmQuit,
}

/// Actions returned by DystopiaFlow for session to handle
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants reserved for session handling
pub enum DystopiaAction {
    /// Continue - no output needed
    Continue,
    /// Show screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game ended (age over or province destroyed)
    GameOver { final_networth: i64 },
    /// Player quit to main menu
    Quit,
}

/// Dystopia game flow state machine
pub struct DystopiaFlow {
    pub state: ProvinceState,
    pub screen: GameScreen,
    input_buffer: String,
    /// Pending action data (for multi-step operations)
    pending_action: Option<PendingAction>,
}

/// Pending action state for multi-step operations
#[derive(Debug, Clone)]
pub enum PendingAction {
    BuildingCount { building: BuildingType },
    TrainingCount { unit: UnitType },
    AttackArmy { target_id: i64, attack_type: AttackType },
    ThiefCount { target_id: i64, op_type: ThiefOp },
    ExploreCount,
}

impl DystopiaFlow {
    /// Create new game flow with character creation
    pub fn new() -> Self {
        Self {
            state: ProvinceState::new(
                String::new(),
                String::new(),
                String::new(),
            ),
            screen: GameScreen::SelectRace,
            input_buffer: String::new(),
            pending_action: None,
        }
    }

    /// Resume game from loaded state
    pub fn from_state(state: ProvinceState) -> Self {
        Self {
            state,
            screen: GameScreen::Throne,
            input_buffer: String::new(),
            pending_action: None,
        }
    }

    /// Get current screen
    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    /// Get current game state
    pub fn game_state(&self) -> &ProvinceState {
        &self.state
    }

    /// Get pending action (for rendering)
    pub fn pending_action(&self) -> &Option<PendingAction> {
        &self.pending_action
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char) -> DystopiaAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return DystopiaAction::Echo("\x08 \x08".to_string());
            }
            return DystopiaAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return DystopiaAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input
        if self.input_buffer.len() < 40 {
            self.input_buffer.push(ch);
            return DystopiaAction::Echo(ch.to_string());
        }

        DystopiaAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Throne
                | GameScreen::Build
                | GameScreen::Military
                | GameScreen::Magic
                | GameScreen::Science
                | GameScreen::Kingdom
                | GameScreen::Info
                | GameScreen::Rankings
                | GameScreen::History
                | GameScreen::Help
                | GameScreen::ConfirmQuit
        ) && self.pending_action.is_none()
    }

    /// Process buffered input
    fn process_input(&mut self) -> DystopiaAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        // Handle pending actions first
        if self.pending_action.is_some() {
            return self.handle_pending_action(&input);
        }

        match &self.screen {
            GameScreen::SelectRace => self.handle_select_race(&input),
            GameScreen::SelectPersonality { race } => self.handle_select_personality(&input, race.clone()),
            GameScreen::EnterName { race, personality } => self.handle_enter_name(&input, race.clone(), personality.clone()),
            GameScreen::Throne => self.handle_throne(&input),
            GameScreen::Build => self.handle_build(&input),
            GameScreen::Military => self.handle_military(&input),
            GameScreen::Attack { target_input } => self.handle_attack(&input, target_input.clone()),
            GameScreen::Thieves { target_input } => self.handle_thieves(&input, target_input.clone()),
            GameScreen::Magic => self.handle_magic(&input),
            GameScreen::Science => self.handle_science(&input),
            GameScreen::Kingdom => self.handle_kingdom(&input),
            GameScreen::Info => self.handle_info(&input),
            GameScreen::Rankings => self.handle_rankings(&input),
            GameScreen::History => self.handle_history(&input),
            GameScreen::Help => self.handle_help(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    fn handle_pending_action(&mut self, input: &str) -> DystopiaAction {
        let action = self.pending_action.take();

        match action {
            Some(PendingAction::BuildingCount { building }) => {
                if let Ok(count) = input.parse::<u32>() {
                    if count > 0 {
                        let result = super::economy::build(&mut self.state, building, count);
                        self.state.last_message = Some(format!("{:?}", result));
                    }
                }
                self.screen = GameScreen::Build;
                DystopiaAction::SaveGame
            }
            Some(PendingAction::TrainingCount { unit }) => {
                if let Ok(count) = input.parse::<u32>() {
                    if count > 0 {
                        let result = super::economy::train_units(&mut self.state, unit, count);
                        self.state.last_message = Some(format!("{:?}", result));
                    }
                }
                self.screen = GameScreen::Military;
                DystopiaAction::SaveGame
            }
            Some(PendingAction::ExploreCount) => {
                if let Ok(count) = input.parse::<u32>() {
                    if count > 0 {
                        let result = super::economy::explore(&mut self.state, count);
                        self.state.last_message = Some(format!("{:?}", result));
                    }
                }
                self.screen = GameScreen::Throne;
                DystopiaAction::SaveGame
            }
            Some(PendingAction::AttackArmy { target_id, attack_type }) => {
                if let Ok(percent) = input.parse::<u32>() {
                    if percent > 0 && percent <= 100 {
                        let result = super::military::execute_attack(&mut self.state, target_id, attack_type, percent);
                        self.state.last_message = Some(result.message);
                    }
                }
                self.screen = GameScreen::Throne;
                DystopiaAction::SaveGame
            }
            Some(PendingAction::ThiefCount { target_id, op_type }) => {
                if let Ok(count) = input.parse::<u32>() {
                    if count > 0 {
                        let result = super::military::execute_thief_op(&mut self.state, target_id, op_type, count);
                        self.state.last_message = Some(result.message);
                    }
                }
                self.screen = GameScreen::Thieves { target_input: String::new() };
                DystopiaAction::SaveGame
            }
            None => DystopiaAction::Continue,
        }
    }

    fn handle_select_race(&mut self, input: &str) -> DystopiaAction {
        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= RACES.len() {
                let race = RACES[idx - 1].key.to_string();
                self.screen = GameScreen::SelectPersonality { race };
                return DystopiaAction::SaveGame;
            }
        }
        DystopiaAction::Continue
    }

    fn handle_select_personality(&mut self, input: &str, race: String) -> DystopiaAction {
        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= PERSONALITIES.len() {
                let personality = PERSONALITIES[idx - 1].key.to_string();
                self.screen = GameScreen::EnterName { race, personality };
                return DystopiaAction::SaveGame;
            }
        }
        DystopiaAction::Continue
    }

    fn handle_enter_name(&mut self, input: &str, race: String, personality: String) -> DystopiaAction {
        let name = input.trim();
        if name.len() >= 3 && name.len() <= 20 {
            self.state = ProvinceState::new(name.to_string(), race, personality);
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }
        DystopiaAction::Continue
    }

    fn handle_throne(&mut self, input: &str) -> DystopiaAction {
        self.state.last_message = None;

        match input {
            "B" => {
                self.screen = GameScreen::Build;
                DystopiaAction::SaveGame
            }
            "M" => {
                self.screen = GameScreen::Military;
                DystopiaAction::SaveGame
            }
            "A" => {
                self.screen = GameScreen::Attack { target_input: String::new() };
                DystopiaAction::SaveGame
            }
            "T" => {
                self.screen = GameScreen::Thieves { target_input: String::new() };
                DystopiaAction::SaveGame
            }
            "S" => {
                self.screen = GameScreen::Magic;
                DystopiaAction::SaveGame
            }
            "R" => {
                self.screen = GameScreen::Science;
                DystopiaAction::SaveGame
            }
            "K" => {
                self.screen = GameScreen::Kingdom;
                DystopiaAction::SaveGame
            }
            "I" => {
                self.screen = GameScreen::Info;
                DystopiaAction::SaveGame
            }
            "L" => {
                self.screen = GameScreen::Rankings;
                DystopiaAction::SaveGame
            }
            "E" => {
                // Explore
                self.pending_action = Some(PendingAction::ExploreCount);
                DystopiaAction::SaveGame
            }
            "H" | "?" => {
                self.screen = GameScreen::Help;
                DystopiaAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::ConfirmQuit;
                DystopiaAction::SaveGame
            }
            _ => DystopiaAction::Continue,
        }
    }

    fn handle_build(&mut self, input: &str) -> DystopiaAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }

        // Map numbers to buildings
        let building = match input {
            "1" => Some(BuildingType::Home),
            "2" => Some(BuildingType::Farm),
            "3" => Some(BuildingType::Bank),
            "4" => Some(BuildingType::Barracks),
            "5" => Some(BuildingType::TrainingGround),
            "6" => Some(BuildingType::Fort),
            "7" => Some(BuildingType::Tower),
            "8" => Some(BuildingType::ThievesDen),
            "9" => Some(BuildingType::WatchTower),
            "0" => Some(BuildingType::University),
            "A" => Some(BuildingType::Hospital),
            "C" => Some(BuildingType::Armoury),
            "D" => Some(BuildingType::Guildhall),
            _ => None,
        };

        if let Some(b) = building {
            self.pending_action = Some(PendingAction::BuildingCount { building: b });
            return DystopiaAction::SaveGame;
        }

        DystopiaAction::Continue
    }

    fn handle_military(&mut self, input: &str) -> DystopiaAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }

        let unit = match input {
            "1" => Some(UnitType::Soldier),
            "2" => Some(UnitType::Archer),
            "3" => Some(UnitType::Knight),
            "4" => Some(UnitType::Thief),
            "5" => Some(UnitType::Wizard),
            "6" => Some(UnitType::Elite),
            _ => None,
        };

        if let Some(u) = unit {
            self.pending_action = Some(PendingAction::TrainingCount { unit: u });
            return DystopiaAction::SaveGame;
        }

        DystopiaAction::Continue
    }

    fn handle_attack(&mut self, input: &str, _target_input: String) -> DystopiaAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }

        // For now, simulate attacking a random target
        let attack_type = match input {
            "1" => Some(AttackType::TraditionalMarch),
            "2" => Some(AttackType::Raid),
            "3" => Some(AttackType::Plunder),
            "4" => Some(AttackType::Massacre),
            "5" => Some(AttackType::Learn),
            _ => None,
        };

        if let Some(at) = attack_type {
            // Simulate target ID (would be from kingdom list in real implementation)
            self.pending_action = Some(PendingAction::AttackArmy {
                target_id: 1,
                attack_type: at,
            });
            return DystopiaAction::SaveGame;
        }

        DystopiaAction::Continue
    }

    fn handle_thieves(&mut self, input: &str, _target_input: String) -> DystopiaAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }

        let op_type = match input {
            "1" => Some(ThiefOp::IntelGather),
            "2" => Some(ThiefOp::StealGold),
            "3" => Some(ThiefOp::Sabotage),
            "4" => Some(ThiefOp::Kidnap),
            "5" => Some(ThiefOp::Assassinate),
            "6" => Some(ThiefOp::PropagandaWar),
            _ => None,
        };

        if let Some(op) = op_type {
            self.pending_action = Some(PendingAction::ThiefCount {
                target_id: 1,
                op_type: op,
            });
            return DystopiaAction::SaveGame;
        }

        DystopiaAction::Continue
    }

    fn handle_magic(&mut self, input: &str) -> DystopiaAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }

        let spell = match input {
            "1" => Some(SpellType::Shield),
            "2" => Some(SpellType::Barrier),
            "3" => Some(SpellType::Prosperity),
            "4" => Some(SpellType::Haste),
            "5" => Some(SpellType::Heal),
            "6" => Some(SpellType::Clairvoyance),
            "7" => Some(SpellType::Fireball),
            "8" => Some(SpellType::Lightning),
            "9" => Some(SpellType::Plague),
            "0" => Some(SpellType::Drought),
            _ => None,
        };

        if let Some(s) = spell {
            let result = super::military::cast_spell(&mut self.state, None, s);
            self.state.last_message = Some(result.message);
            return DystopiaAction::SaveGame;
        }

        DystopiaAction::Continue
    }

    fn handle_science(&mut self, input: &str) -> DystopiaAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Throne;
            return DystopiaAction::SaveGame;
        }

        let science = match input {
            "1" => Some("alchemy"),
            "2" => Some("tools"),
            "3" => Some("housing"),
            "4" => Some("food"),
            "5" => Some("military"),
            "6" => Some("crime"),
            "7" => Some("channeling"),
            _ => None,
        };

        if let Some(s) = science {
            let result = super::economy::start_research(&mut self.state, s);
            self.state.last_message = Some(format!("{:?}", result));
            return DystopiaAction::SaveGame;
        }

        DystopiaAction::Continue
    }

    fn handle_kingdom(&mut self, _input: &str) -> DystopiaAction {
        // For now, just return to throne
        self.screen = GameScreen::Throne;
        DystopiaAction::SaveGame
    }

    fn handle_info(&mut self, _input: &str) -> DystopiaAction {
        self.screen = GameScreen::Throne;
        DystopiaAction::SaveGame
    }

    fn handle_rankings(&mut self, _input: &str) -> DystopiaAction {
        self.screen = GameScreen::Throne;
        DystopiaAction::SaveGame
    }

    fn handle_history(&mut self, _input: &str) -> DystopiaAction {
        self.screen = GameScreen::Throne;
        DystopiaAction::SaveGame
    }

    fn handle_help(&mut self, _input: &str) -> DystopiaAction {
        self.screen = GameScreen::Throne;
        DystopiaAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> DystopiaAction {
        match input {
            "Y" => DystopiaAction::Quit,
            _ => {
                self.screen = GameScreen::Throne;
                DystopiaAction::SaveGame
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_starts_at_race_selection() {
        let flow = DystopiaFlow::new();
        assert!(matches!(flow.screen, GameScreen::SelectRace));
    }

    #[test]
    fn test_race_selection() {
        let mut flow = DystopiaFlow::new();

        // Select first race
        flow.input_buffer = "1".to_string();
        let action = flow.process_input();

        assert!(matches!(action, DystopiaAction::SaveGame));
        assert!(matches!(flow.screen, GameScreen::SelectPersonality { .. }));
    }

    #[test]
    fn test_full_character_creation() {
        let mut flow = DystopiaFlow::new();

        // Select race
        flow.input_buffer = "1".to_string();
        flow.process_input();

        // Select personality
        flow.input_buffer = "1".to_string();
        flow.process_input();

        // Enter name
        flow.input_buffer = "TestProvince".to_string();
        flow.process_input();

        assert!(matches!(flow.screen, GameScreen::Throne));
        assert_eq!(flow.state.name, "TESTPROVINCE"); // Uppercased
    }

    #[test]
    fn test_throne_navigation() {
        let mut flow = DystopiaFlow::from_state(ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        ));

        // Go to build screen
        flow.input_buffer = "B".to_string();
        flow.process_input();
        assert!(matches!(flow.screen, GameScreen::Build));

        // Back to throne
        flow.input_buffer = "Q".to_string();
        flow.process_input();
        assert!(matches!(flow.screen, GameScreen::Throne));
    }

    #[test]
    fn test_quit_confirmation() {
        let mut flow = DystopiaFlow::from_state(ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        ));

        // Request quit
        flow.input_buffer = "Q".to_string();
        flow.process_input();
        assert!(matches!(flow.screen, GameScreen::ConfirmQuit));

        // Confirm
        flow.input_buffer = "Y".to_string();
        let action = flow.process_input();
        assert!(matches!(action, DystopiaAction::Quit));
    }
}
