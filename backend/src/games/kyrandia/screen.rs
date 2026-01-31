//! Game screen and flow state machine for Morningmist
//! Manages screen transitions and input handling

#![allow(dead_code)]

use super::state::GameState;
use super::parser::{parse_command, CommandType};
use super::world::{try_move, MoveResult, try_take_item, drop_item, use_item, equip_item, unequip_item, talk_to_npc, rest_at_inn};
use super::combat::{start_random_encounter, player_attack, attempt_flee, apply_spell_damage, check_random_encounter};
use super::magic::{cast_spell, use_fountain, SpellEffect};
use super::puzzles::{try_phrase_puzzle, check_item_puzzles};
use super::data::{get_room, RoomSpecial};

/// Current screen the player is viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// New game intro
    Intro,
    /// Main exploration view
    Exploration,
    /// Combat encounter
    Combat,
    /// Talking to NPC
    Dialogue { npc_key: String, dialogue_index: usize },
    /// Shopping at a merchant
    Shop { npc_key: String },
    /// Viewing inventory
    Inventory,
    /// Viewing spellbook
    Spellbook,
    /// Viewing stats
    Stats,
    /// Text input mode (for spells, puzzles)
    TextInput { prompt: String, purpose: TextInputPurpose },
    /// At the Fountain of Scrolls
    Fountain,
    /// Game over
    GameOver { victory: bool },
    /// Leaderboard
    Leaderboard,
    /// Confirm quit
    ConfirmQuit,
    /// Help screen
    Help,
}

/// Purpose of text input
#[derive(Debug, Clone, PartialEq)]
pub enum TextInputPurpose {
    SpellCast,
    PuzzleSolution,
    Say,
    Whisper { target: String },
}

/// Actions returned by the flow for session handling
#[derive(Debug, Clone)]
pub enum KyrandiaAction {
    /// Continue - no output needed
    Continue,
    /// Render screen output
    Render(String),
    /// Echo characters back
    Echo(String),
    /// Save game state
    SaveGame,
    /// Game completed
    GameOver { became_archmage: bool, level: u8 },
    /// Player quit
    Quit,
}

/// Main game flow state machine
pub struct KyrandiaFlow {
    pub state: GameState,
    pub screen: GameScreen,
    input_buffer: String,
}

impl KyrandiaFlow {
    /// Create new game
    pub fn new(player_name: &str) -> Self {
        Self {
            state: GameState::new(player_name),
            screen: GameScreen::Intro,
            input_buffer: String::new(),
        }
    }

    /// Resume from saved state
    pub fn from_state(state: GameState) -> Self {
        Self {
            state,
            screen: GameScreen::Intro,  // Show intro on resume too
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
    pub fn handle_char(&mut self, ch: char) -> KyrandiaAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return KyrandiaAction::Echo("\x08 \x08".to_string());
            }
            return KyrandiaAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return KyrandiaAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // In exploration mode, handle single-key shortcuts immediately
        // but only if the buffer is empty (not mid-typing a command)
        if matches!(self.screen, GameScreen::Exploration) && self.input_buffer.is_empty() {
            let upper = ch.to_ascii_uppercase();
            if matches!(upper, 'I' | 'M' | 'S' | 'H' | 'Q') {
                self.input_buffer.push(ch);
                return self.process_input();
            }
        }

        // Buffer input
        if self.input_buffer.len() < 80 {
            self.input_buffer.push(ch);
            return KyrandiaAction::Echo(ch.to_string());
        }

        KyrandiaAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Intro
                | GameScreen::Combat
                | GameScreen::Inventory
                | GameScreen::Spellbook
                | GameScreen::Stats
                | GameScreen::Fountain
                | GameScreen::GameOver { .. }
                | GameScreen::ConfirmQuit
                | GameScreen::Help
                | GameScreen::Shop { .. }
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> KyrandiaAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(input),
            GameScreen::Exploration => self.handle_exploration(input),
            GameScreen::Combat => self.handle_combat(input),
            GameScreen::Dialogue { npc_key, dialogue_index } => {
                self.handle_dialogue(input, npc_key.clone(), *dialogue_index)
            }
            GameScreen::Shop { npc_key } => self.handle_shop(input, npc_key.clone()),
            GameScreen::Inventory => self.handle_inventory(input),
            GameScreen::Spellbook => self.handle_spellbook(input),
            GameScreen::Stats => self.handle_stats(input),
            GameScreen::TextInput { purpose, .. } => self.handle_text_input(input, purpose.clone()),
            GameScreen::Fountain => self.handle_fountain(input),
            GameScreen::GameOver { .. } => self.handle_game_over(input),
            GameScreen::Leaderboard => self.handle_leaderboard(input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(input),
            GameScreen::Help => self.handle_help(input),
        }
    }

    fn handle_intro(&mut self, _input: &str) -> KyrandiaAction {
        // Check for daily reset
        self.state.check_daily_reset();

        self.screen = GameScreen::Exploration;
        KyrandiaAction::SaveGame
    }

    fn handle_exploration(&mut self, input: &str) -> KyrandiaAction {
        self.state.last_message = None;

        let cmd = parse_command(input);

        match cmd.command_type {
            CommandType::Move(direction) => {
                if !self.state.use_turn() {
                    self.state.last_message = Some("No turns remaining! Rest at an inn.".to_string());
                    return KyrandiaAction::SaveGame;
                }

                match try_move(&mut self.state, &direction) {
                    MoveResult::Success { room_name, .. } => {
                        self.state.last_message = Some(format!("You travel to {}.", room_name));

                        // Check for random encounter
                        let region = self.state.current_region();
                        if check_random_encounter(region, self.state.level) {
                            if let Some(msg) = start_random_encounter(&mut self.state, region) {
                                self.state.last_message = Some(msg);
                                self.screen = GameScreen::Combat;
                            }
                        }
                    }
                    MoveResult::InvalidDirection(msg) => {
                        self.state.last_message = Some(msg);
                    }
                    MoveResult::RegionLocked { required_level } => {
                        self.state.last_message = Some(format!(
                            "This region requires level {}. You are level {}.",
                            required_level, self.state.level
                        ));
                    }
                    MoveResult::NeedKey(key) => {
                        let key_name = match key.as_str() {
                            "golden_key" => "the Golden Key",
                            "dragon_key" => "the Dragon Key",
                            _ => "a key",
                        };
                        self.state.last_message = Some(format!("You need {} to pass.", key_name));
                    }
                    MoveResult::InCombat => {
                        self.state.last_message = Some("You cannot flee during combat!".to_string());
                    }
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Look(target) => {
                if let Some(target) = target {
                    // Look at specific thing
                    self.state.last_message = Some(format!("You examine {}...", target));
                } else {
                    // Just redraw room
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Take(item_name) => {
                match try_take_item(&mut self.state, &item_name) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }

                // Check if taking items solved a puzzle
                if let Some(result) = check_item_puzzles(&mut self.state) {
                    if let super::puzzles::PuzzleResult::Solved { message, rewards, .. } = result {
                        let reward_text = rewards.join("\n");
                        self.state.last_message = Some(format!("{}\n{}", message, reward_text));
                    }
                }

                KyrandiaAction::SaveGame
            }

            CommandType::Drop(item_name) => {
                match drop_item(&mut self.state, &item_name) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Use(item_name) => {
                match use_item(&mut self.state, &item_name) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Equip(item_name) => {
                match equip_item(&mut self.state, &item_name) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Unequip(item_name) => {
                match unequip_item(&mut self.state, &item_name) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Talk(npc_name) => {
                match talk_to_npc(&mut self.state, &npc_name) {
                    Ok((npc_name, dialogue)) => {
                        if !dialogue.is_empty() {
                            self.screen = GameScreen::Dialogue {
                                npc_key: npc_name,
                                dialogue_index: 0,
                            };
                        }
                    }
                    Err(msg) => {
                        self.state.last_message = Some(msg);
                    }
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Cast(incantation) => {
                let room_special = get_room(&self.state.current_room).and_then(|r| r.special);
                let result = cast_spell(&incantation, &mut self.state, room_special);

                self.state.last_message = Some(result.message.clone());

                // Handle special spell effects
                match result.effect {
                    SpellEffect::Teleport(room_key) => {
                        self.state.current_room = room_key;
                    }
                    SpellEffect::PuzzleSolved(puzzle_key) => {
                        self.state.solve_puzzle(&puzzle_key);
                    }
                    _ => {}
                }

                KyrandiaAction::SaveGame
            }

            CommandType::Inventory => {
                self.screen = GameScreen::Inventory;
                KyrandiaAction::SaveGame
            }

            CommandType::Spells => {
                self.screen = GameScreen::Spellbook;
                KyrandiaAction::SaveGame
            }

            CommandType::Stats => {
                self.screen = GameScreen::Stats;
                KyrandiaAction::SaveGame
            }

            CommandType::Rest => {
                match rest_at_inn(&mut self.state) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Throw(item_name) => {
                // Special handling for fountain
                let room = get_room(&self.state.current_room);
                if room.map(|r| r.special) == Some(Some(RoomSpecial::Fountain)) {
                    if item_name.contains("pine") {
                        self.screen = GameScreen::Fountain;
                    } else {
                        self.state.last_message = Some(
                            "The fountain only accepts pine cones.".to_string()
                        );
                    }
                } else {
                    match drop_item(&mut self.state, &item_name) {
                        Ok(_) => self.state.last_message = Some(format!("You throw the {}.", item_name)),
                        Err(msg) => self.state.last_message = Some(msg),
                    }
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Help => {
                self.screen = GameScreen::Help;
                KyrandiaAction::SaveGame
            }

            CommandType::Quit => {
                self.screen = GameScreen::ConfirmQuit;
                KyrandiaAction::SaveGame
            }

            CommandType::MenuSelect(key) => {
                // Handle numbered menu selections
                match key.as_str() {
                    "1" => {
                        // First direction
                        if let Some(room) = get_room(&self.state.current_room) {
                            if let Some((dir, _)) = room.exits.first() {
                                return self.handle_exploration(&format!("go {}", dir));
                            }
                        }
                    }
                    "I" => {
                        self.screen = GameScreen::Inventory;
                    }
                    "S" => {
                        self.screen = GameScreen::Stats;
                    }
                    "M" => {
                        self.screen = GameScreen::Spellbook;
                    }
                    "H" | "?" => {
                        self.screen = GameScreen::Help;
                    }
                    "Q" => {
                        self.screen = GameScreen::ConfirmQuit;
                    }
                    _ => {}
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Unknown(text) => {
                // Try as puzzle solution
                let room_key = self.state.current_room.clone();
                let result = try_phrase_puzzle(&mut self.state, &text, &room_key);

                match result {
                    super::puzzles::PuzzleResult::Solved { puzzle_name, message, rewards } => {
                        let reward_text = rewards.join("\n");
                        self.state.last_message = Some(format!(
                            "You solved '{}'!\n{}\n{}",
                            puzzle_name, message, reward_text
                        ));
                    }
                    super::puzzles::PuzzleResult::WrongSolution { hint, .. } => {
                        self.state.last_message = Some(format!("Hint: {}", hint));
                    }
                    _ => {
                        self.state.last_message = Some(format!("I don't understand '{}'.", text));
                    }
                }
                KyrandiaAction::SaveGame
            }

            CommandType::Empty => KyrandiaAction::SaveGame,

            _ => {
                self.state.last_message = Some("Command not available here.".to_string());
                KyrandiaAction::SaveGame
            }
        }
    }

    fn handle_combat(&mut self, input: &str) -> KyrandiaAction {
        let input_upper = input.to_uppercase();

        match input_upper.as_str() {
            "A" | "1" => {
                // Attack
                let result = player_attack(&mut self.state);
                self.state.last_message = Some(result.message);

                if result.combat_ended {
                    if result.victory {
                        // Check for level up
                        if self.state.level >= 7 && !self.state.became_archmage {
                            // Check if they just defeated the dragon
                            if self.state.get_flag("ritual_ready").is_some() {
                                self.state.became_archmage = true;
                                self.state.dragon_defeated = true;
                                self.screen = GameScreen::GameOver { victory: true };
                                return KyrandiaAction::SaveGame;
                            }
                        }
                    } else {
                        // Player died
                        self.state.respawn();
                        self.state.last_message = Some(
                            "You wake up at the inn, battered but alive...".to_string()
                        );
                    }
                    self.screen = GameScreen::Exploration;
                }
                KyrandiaAction::SaveGame
            }
            "C" | "2" => {
                // Cast spell - enter text mode
                self.screen = GameScreen::TextInput {
                    prompt: "Cast what spell? (type incantation)".to_string(),
                    purpose: TextInputPurpose::SpellCast,
                };
                KyrandiaAction::SaveGame
            }
            "F" | "3" => {
                // Flee
                let result = attempt_flee(&mut self.state);
                self.state.last_message = Some(result.message);

                if result.combat_ended {
                    self.screen = GameScreen::Exploration;
                }
                KyrandiaAction::SaveGame
            }
            _ => KyrandiaAction::Continue,
        }
    }

    fn handle_dialogue(&mut self, _input: &str, npc_key: String, mut index: usize) -> KyrandiaAction {
        // Any key advances dialogue
        if let Some(npc) = super::data::get_npc(&npc_key) {
            index += 1;
            if index >= npc.dialogue.len() {
                // Dialogue finished
                self.screen = GameScreen::Exploration;
            } else {
                self.screen = GameScreen::Dialogue {
                    npc_key,
                    dialogue_index: index,
                };
            }
        } else {
            self.screen = GameScreen::Exploration;
        }
        KyrandiaAction::SaveGame
    }

    fn handle_shop(&mut self, input: &str, _npc_key: String) -> KyrandiaAction {
        let input_upper = input.to_uppercase();

        if input_upper == "Q" || input_upper == "X" {
            self.screen = GameScreen::Exploration;
            return KyrandiaAction::SaveGame;
        }

        // Handle buy/sell by number
        // (Would need shop implementation)
        self.state.last_message = Some("Shop not fully implemented yet.".to_string());
        KyrandiaAction::SaveGame
    }

    fn handle_inventory(&mut self, input: &str) -> KyrandiaAction {
        let input_upper = input.to_uppercase();

        if input_upper == "Q" || input_upper == "X" || input.is_empty() {
            self.screen = GameScreen::Exploration;
        }
        // Could add equip/use options here
        KyrandiaAction::SaveGame
    }

    fn handle_spellbook(&mut self, input: &str) -> KyrandiaAction {
        if input.to_uppercase() == "Q" || input.to_uppercase() == "X" || input.is_empty() {
            self.screen = GameScreen::Exploration;
        }
        KyrandiaAction::SaveGame
    }

    fn handle_stats(&mut self, input: &str) -> KyrandiaAction {
        if input.to_uppercase() == "Q" || input.to_uppercase() == "X" || input.is_empty() {
            self.screen = GameScreen::Exploration;
        }
        KyrandiaAction::SaveGame
    }

    fn handle_text_input(&mut self, input: &str, purpose: TextInputPurpose) -> KyrandiaAction {
        match purpose {
            TextInputPurpose::SpellCast => {
                let room_special = get_room(&self.state.current_room).and_then(|r| r.special);
                let result = cast_spell(input, &mut self.state, room_special);

                self.state.last_message = Some(result.message.clone());

                // If in combat and cast damage spell
                if self.state.combat.is_some() {
                    if let SpellEffect::Damage(damage) = result.effect {
                        let combat_result = apply_spell_damage(&mut self.state, damage);
                        if combat_result.combat_ended {
                            if combat_result.victory {
                                self.screen = GameScreen::Exploration;
                            } else {
                                self.state.respawn();
                                self.screen = GameScreen::Exploration;
                            }
                        } else {
                            self.screen = GameScreen::Combat;
                        }
                    } else {
                        self.screen = GameScreen::Combat;
                    }
                } else {
                    self.screen = GameScreen::Exploration;
                }
            }
            TextInputPurpose::PuzzleSolution => {
                let room_key = self.state.current_room.clone();
                let result = try_phrase_puzzle(&mut self.state, input, &room_key);

                match result {
                    super::puzzles::PuzzleResult::Solved { message, rewards, .. } => {
                        let reward_text = rewards.join("\n");
                        self.state.last_message = Some(format!("{}\n{}", message, reward_text));
                    }
                    super::puzzles::PuzzleResult::WrongSolution { hint, .. } => {
                        self.state.last_message = Some(format!("Incorrect. Hint: {}", hint));
                    }
                    _ => {
                        self.state.last_message = Some("Nothing happens.".to_string());
                    }
                }
                self.screen = GameScreen::Exploration;
            }
            TextInputPurpose::Say | TextInputPurpose::Whisper { .. } => {
                // Multiplayer chat - would be handled differently
                self.state.last_message = Some(format!("You say: \"{}\"", input));
                self.screen = GameScreen::Exploration;
            }
        }
        KyrandiaAction::SaveGame
    }

    fn handle_fountain(&mut self, input: &str) -> KyrandiaAction {
        let input_upper = input.to_uppercase();

        match input_upper.as_str() {
            "Y" | "1" => {
                match use_fountain(&mut self.state) {
                    Ok(msg) => self.state.last_message = Some(msg),
                    Err(msg) => self.state.last_message = Some(msg),
                }
                self.screen = GameScreen::Exploration;
            }
            "N" | "Q" | "2" => {
                self.screen = GameScreen::Exploration;
            }
            _ => {}
        }
        KyrandiaAction::SaveGame
    }

    fn handle_game_over(&mut self, _input: &str) -> KyrandiaAction {
        KyrandiaAction::GameOver {
            became_archmage: self.state.became_archmage,
            level: self.state.level,
        }
    }

    fn handle_leaderboard(&mut self, _input: &str) -> KyrandiaAction {
        self.screen = GameScreen::Exploration;
        KyrandiaAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> KyrandiaAction {
        match input.to_uppercase().as_str() {
            "Y" => KyrandiaAction::Quit,
            _ => {
                self.screen = GameScreen::Exploration;
                KyrandiaAction::SaveGame
            }
        }
    }

    fn handle_help(&mut self, _input: &str) -> KyrandiaAction {
        self.screen = GameScreen::Exploration;
        KyrandiaAction::SaveGame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = KyrandiaFlow::new("Test");
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_intro_advances() {
        let mut flow = KyrandiaFlow::new("Test");
        flow.handle_char('\r');
        assert!(matches!(flow.screen, GameScreen::Exploration));
    }

    #[test]
    fn test_movement() {
        let mut flow = KyrandiaFlow::new("Test");
        flow.screen = GameScreen::Exploration;

        // Type "north" and press enter
        for c in "north".chars() {
            flow.handle_char(c);
        }
        flow.handle_char('\r');

        assert_eq!(flow.state.current_room, "village_inn");
    }

    #[test]
    fn test_inventory_screen() {
        let mut flow = KyrandiaFlow::new("Test");
        flow.screen = GameScreen::Exploration;

        flow.handle_char('I');
        assert!(matches!(flow.screen, GameScreen::Inventory));
    }

    #[test]
    fn test_quit_confirm() {
        let mut flow = KyrandiaFlow::new("Test");
        flow.screen = GameScreen::ConfirmQuit;

        flow.handle_char('Y');
        // Would return Quit action
    }

    #[test]
    fn test_backspace() {
        let mut flow = KyrandiaFlow::new("Test");
        flow.screen = GameScreen::Exploration;
        flow.input_buffer = "test".to_string();

        flow.handle_char('\x7f');  // Backspace
        assert_eq!(flow.input_buffer, "tes");
    }
}
