//! Screen state machine for Summit
//!
//! Handles all game screens and input processing.

use super::data::ItemType;
use super::state::{ClimberState, PlayerStats, StatusEffectType};
use super::lobby::SummitLobby;

// ============================================================================
// SCREEN ENUM
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum LobbyScreen {
    MainMenu,
    CreateGame,
    JoinGame,
    EnterInviteCode,
    WaitingRoom,
    Countdown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// Lobby screens
    Lobby(LobbyScreen),
    /// Crash landing intro
    CrashLanding,
    /// Main climbing gameplay
    Climbing,
    /// Inventory management
    Inventory,
    /// Eating food
    EatFood,
    /// At a campfire
    Campfire,
    /// Roasting marshmallows mini-game
    RoastMarshmallow { heat_level: u32 },
    /// Helping a downed teammate
    RevivePartner { partner_id: i64 },
    /// Summit reached!
    Summit,
    /// All players dead
    GameOver,
    /// Results screen
    Results,
    /// Player stats/badges
    Stats,
    /// Cosmetics customization
    Cosmetics,
    /// Confirm quit
    ConfirmQuit,
}

// ============================================================================
// ACTIONS
// ============================================================================

#[derive(Debug, Clone)]
pub enum SummitAction {
    /// No action needed
    Continue,
    /// Render current screen
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save player stats
    SaveStats,
    /// Player quit to main menu
    Quit,
    /// Send message to other players
    Broadcast(String),
}

// ============================================================================
// SUMMIT FLOW
// ============================================================================

pub struct SummitFlow {
    pub user_id: i64,
    pub handle: String,
    pub screen: GameScreen,
    pub stats: PlayerStats,
    input_buffer: String,
    /// For invite code entry
    invite_code_buffer: String,
}

impl SummitFlow {
    pub fn new(user_id: i64, handle: String, stats: PlayerStats) -> Self {
        Self {
            user_id,
            handle,
            screen: GameScreen::Lobby(LobbyScreen::MainMenu),
            stats,
            input_buffer: String::new(),
            invite_code_buffer: String::new(),
        }
    }

    pub fn current_screen(&self) -> &GameScreen {
        &self.screen
    }

    pub fn get_stats(&self) -> &PlayerStats {
        &self.stats
    }

    pub fn get_stats_mut(&mut self) -> &mut PlayerStats {
        &mut self.stats
    }

    /// Handle character input
    pub fn handle_char(&mut self, ch: char, lobby_manager: &mut SummitLobby) -> SummitAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if matches!(self.screen, GameScreen::Lobby(LobbyScreen::EnterInviteCode)) {
                if self.invite_code_buffer.pop().is_some() {
                    return SummitAction::Echo("\x08 \x08".to_string());
                }
            } else if self.input_buffer.pop().is_some() {
                return SummitAction::Echo("\x08 \x08".to_string());
            }
            return SummitAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input(lobby_manager);
        }

        // Ignore control chars
        if ch.is_control() {
            return SummitAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input(lobby_manager);
        }

        // Buffer input
        if matches!(self.screen, GameScreen::Lobby(LobbyScreen::EnterInviteCode)) {
            if self.invite_code_buffer.len() < 6 {
                self.invite_code_buffer.push(ch.to_ascii_uppercase());
                return SummitAction::Echo(ch.to_ascii_uppercase().to_string());
            }
        } else if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return SummitAction::Echo(ch.to_string());
        }

        SummitAction::Continue
    }

    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::Lobby(LobbyScreen::MainMenu)
                | GameScreen::Lobby(LobbyScreen::CreateGame)
                | GameScreen::Lobby(LobbyScreen::JoinGame)
                | GameScreen::Lobby(LobbyScreen::WaitingRoom)
                | GameScreen::Climbing
                | GameScreen::Inventory
                | GameScreen::EatFood
                | GameScreen::Campfire
                | GameScreen::RoastMarshmallow { .. }
                | GameScreen::CrashLanding
                | GameScreen::Summit
                | GameScreen::GameOver
                | GameScreen::Results
                | GameScreen::Stats
                | GameScreen::Cosmetics
                | GameScreen::ConfirmQuit
        )
    }

    fn process_input(&mut self, lobby_manager: &mut SummitLobby) -> SummitAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Lobby(lobby_screen) => {
                self.handle_lobby_input(&input, lobby_screen.clone(), lobby_manager)
            }
            GameScreen::CrashLanding => self.handle_crash_landing(&input),
            GameScreen::Climbing => self.handle_climbing(&input, lobby_manager),
            GameScreen::Inventory => self.handle_inventory(&input, lobby_manager),
            GameScreen::EatFood => self.handle_eat_food(&input, lobby_manager),
            GameScreen::Campfire => self.handle_campfire(&input, lobby_manager),
            GameScreen::RoastMarshmallow { heat_level } => {
                self.handle_roast_marshmallow(&input, *heat_level, lobby_manager)
            }
            GameScreen::RevivePartner { partner_id } => {
                self.handle_revive(&input, *partner_id, lobby_manager)
            }
            GameScreen::Summit => self.handle_summit(&input),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::Results => self.handle_results(&input, lobby_manager),
            GameScreen::Stats => self.handle_stats(&input),
            GameScreen::Cosmetics => self.handle_cosmetics(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input, lobby_manager),
        }
    }

    fn handle_lobby_input(&mut self, input: &str, screen: LobbyScreen, lobby_manager: &mut SummitLobby) -> SummitAction {
        match screen {
            LobbyScreen::MainMenu => {
                match input {
                    "1" | "C" => {
                        self.screen = GameScreen::Lobby(LobbyScreen::CreateGame);
                        SummitAction::SaveStats
                    }
                    "2" | "J" => {
                        self.screen = GameScreen::Lobby(LobbyScreen::JoinGame);
                        SummitAction::SaveStats
                    }
                    "3" | "S" => {
                        self.screen = GameScreen::Stats;
                        SummitAction::SaveStats
                    }
                    "4" | "O" => {
                        self.screen = GameScreen::Cosmetics;
                        SummitAction::SaveStats
                    }
                    "Q" | "X" => {
                        self.screen = GameScreen::ConfirmQuit;
                        SummitAction::SaveStats
                    }
                    _ => SummitAction::Continue,
                }
            }
            LobbyScreen::CreateGame => {
                match input {
                    "1" | "P" => {
                        // Create public game
                        match lobby_manager.create_public_lobby(
                            self.user_id,
                            self.handle.clone(),
                            self.stats.clone(),
                        ) {
                            Ok(_) => {
                                self.screen = GameScreen::Lobby(LobbyScreen::WaitingRoom);
                                SummitAction::SaveStats
                            }
                            Err(_) => SummitAction::Continue,
                        }
                    }
                    "2" | "F" => {
                        // Create private (friends) game
                        match lobby_manager.create_private_lobby(
                            self.user_id,
                            self.handle.clone(),
                            self.stats.clone(),
                        ) {
                            Ok(_) => {
                                self.screen = GameScreen::Lobby(LobbyScreen::WaitingRoom);
                                SummitAction::SaveStats
                            }
                            Err(_) => SummitAction::Continue,
                        }
                    }
                    "B" | "Q" => {
                        self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
                        SummitAction::SaveStats
                    }
                    _ => SummitAction::Continue,
                }
            }
            LobbyScreen::JoinGame => {
                match input {
                    "1" | "Q" => {
                        // Quick join public game
                        match lobby_manager.join_public_lobby(
                            self.user_id,
                            self.handle.clone(),
                            self.stats.clone(),
                            None,
                        ) {
                            Ok(_) => {
                                self.screen = GameScreen::Lobby(LobbyScreen::WaitingRoom);
                                SummitAction::SaveStats
                            }
                            Err(_) => SummitAction::Continue,
                        }
                    }
                    "2" | "I" => {
                        // Enter invite code
                        self.invite_code_buffer.clear();
                        self.screen = GameScreen::Lobby(LobbyScreen::EnterInviteCode);
                        SummitAction::SaveStats
                    }
                    "B" => {
                        self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
                        SummitAction::SaveStats
                    }
                    _ => SummitAction::Continue,
                }
            }
            LobbyScreen::EnterInviteCode => {
                let code = std::mem::take(&mut self.invite_code_buffer);
                if code.len() == 6 {
                    match lobby_manager.join_by_invite(
                        self.user_id,
                        self.handle.clone(),
                        self.stats.clone(),
                        &code,
                    ) {
                        Ok(_) => {
                            self.screen = GameScreen::Lobby(LobbyScreen::WaitingRoom);
                            return SummitAction::SaveStats;
                        }
                        Err(_) => {
                            self.invite_code_buffer.clear();
                            return SummitAction::Continue;
                        }
                    }
                }
                if input == "B" || input == "Q" {
                    self.screen = GameScreen::Lobby(LobbyScreen::JoinGame);
                    return SummitAction::SaveStats;
                }
                SummitAction::Continue
            }
            LobbyScreen::WaitingRoom => {
                match input {
                    "R" => {
                        // Toggle ready
                        if let Some(lobby) = lobby_manager.get_player_lobby(self.user_id) {
                            let is_ready = lobby.players.iter()
                                .find(|p| p.user_id == self.user_id)
                                .map(|p| p.is_ready)
                                .unwrap_or(false);
                            let _ = lobby_manager.set_ready(self.user_id, !is_ready);
                        }
                        SummitAction::SaveStats
                    }
                    "S" => {
                        // Start game (host only)
                        if let Ok(_) = lobby_manager.start_game(self.user_id) {
                            self.screen = GameScreen::Lobby(LobbyScreen::Countdown);
                        }
                        SummitAction::SaveStats
                    }
                    "L" | "Q" => {
                        // Leave lobby
                        let _ = lobby_manager.leave_game(self.user_id);
                        self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
                        SummitAction::SaveStats
                    }
                    _ => SummitAction::Continue,
                }
            }
            LobbyScreen::Countdown => {
                // Check if countdown complete
                if let Some(lobby) = lobby_manager.get_player_lobby(self.user_id) {
                    if lobby.is_countdown_complete() {
                        let game_id = lobby.game_id;
                        if lobby_manager.transition_to_game(game_id).is_ok() {
                            self.screen = GameScreen::CrashLanding;
                            return SummitAction::SaveStats;
                        }
                    }
                }
                SummitAction::Continue
            }
        }
    }

    fn handle_crash_landing(&mut self, _input: &str) -> SummitAction {
        self.screen = GameScreen::Climbing;
        SummitAction::SaveStats
    }

    fn handle_climbing(&mut self, input: &str, lobby_manager: &mut SummitLobby) -> SummitAction {
        let game = match lobby_manager.get_player_game_mut(self.user_id) {
            Some(g) => g,
            None => {
                self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
                return SummitAction::SaveStats;
            }
        };

        let climber = match game.run.get_climber_mut(self.user_id) {
            Some(c) => c,
            None => return SummitAction::Continue,
        };

        if !climber.is_active() {
            // If downed, wait for revival
            return SummitAction::Continue;
        }

        match input {
            // Movement
            "W" | "8" => {
                // Move up (climb)
                if climber.stamina_current > 0 {
                    let new_y = climber.y + 1;
                    if game.mountain.can_move_to(climber.x, new_y) {
                        climber.y = new_y;
                        climber.stamina_current = climber.stamina_current.saturating_sub(
                            climber.current_biome().stamina_drain_rate()
                        );

                        // Check for hazards
                        if let Some(tile) = game.mountain.get_tile(climber.x, climber.y) {
                            if let Some(hazard_type) = tile.hazard {
                                // Apply hazard damage
                                if let Some(hazard) = super::data::HAZARDS.iter()
                                    .find(|h| h.hazard_type == hazard_type)
                                {
                                    climber.apply_stamina_damage(hazard.damage, hazard.max_damage);
                                }
                            }
                        }

                        // Check for summit
                        if climber.y >= 100 {
                            self.screen = GameScreen::Summit;
                            return SummitAction::SaveStats;
                        }
                    }
                }
                SummitAction::SaveStats
            }
            "S" | "2" => {
                // Move down
                let new_y = climber.y - 1;
                if new_y >= 0 && game.mountain.can_move_to(climber.x, new_y) {
                    climber.y = new_y;
                }
                SummitAction::SaveStats
            }
            "A" | "4" => {
                // Move left
                let new_x = climber.x - 1;
                if game.mountain.can_move_to(new_x, climber.y) {
                    climber.x = new_x;
                    climber.stamina_current = climber.stamina_current.saturating_sub(1);
                }
                SummitAction::SaveStats
            }
            "D" | "6" => {
                // Move right
                let new_x = climber.x + 1;
                if game.mountain.can_move_to(new_x, climber.y) {
                    climber.x = new_x;
                    climber.stamina_current = climber.stamina_current.saturating_sub(1);
                }
                SummitAction::SaveStats
            }
            " " => {
                // Grab / rest
                if game.mountain.is_rest_point(climber.x, climber.y) {
                    climber.regenerate_stamina(10);

                    // Check for campfire
                    if let Some(tile) = game.mountain.get_tile(climber.x, climber.y) {
                        if tile.tile_type == super::mountain::TileType::Campfire {
                            self.screen = GameScreen::Campfire;
                            return SummitAction::SaveStats;
                        }
                    }
                } else if game.mountain.can_grab(climber.x, climber.y) {
                    climber.regenerate_stamina(2);
                }
                SummitAction::SaveStats
            }
            "I" => {
                // Inventory
                self.screen = GameScreen::Inventory;
                SummitAction::SaveStats
            }
            "E" => {
                // Eat food
                if !climber.foods.is_empty() {
                    self.screen = GameScreen::EatFood;
                }
                SummitAction::SaveStats
            }
            "R" => {
                // Deploy rope
                if climber.use_item(ItemType::Rope) {
                    let x = climber.x;
                    let y = climber.y;
                    climber.ropes_placed += 1;
                    game.run.place_item(ItemType::Rope, x, y, self.user_id);
                }
                SummitAction::SaveStats
            }
            "P" => {
                // Place piton
                if climber.use_item(ItemType::Piton) {
                    let x = climber.x;
                    let y = climber.y;
                    climber.pitons_placed += 1;
                    game.run.place_item(ItemType::Piton, x, y, self.user_id);
                }
                SummitAction::SaveStats
            }
            "H" => {
                // Help teammate - find downed teammate nearby
                let my_x = climber.x;
                let my_y = climber.y;

                let nearby_downed = game.run.climbers.iter()
                    .find(|(id, c)| {
                        **id != self.user_id
                            && c.is_downed()
                            && (c.x - my_x).abs() <= 2
                            && (c.y - my_y).abs() <= 2
                    })
                    .map(|(id, _)| *id);

                if let Some(partner_id) = nearby_downed {
                    self.screen = GameScreen::RevivePartner { partner_id };
                }
                SummitAction::SaveStats
            }
            "Q" => {
                self.screen = GameScreen::ConfirmQuit;
                SummitAction::SaveStats
            }
            _ => SummitAction::Continue,
        }
    }

    fn handle_inventory(&mut self, input: &str, lobby_manager: &mut SummitLobby) -> SummitAction {
        if input == "B" || input == "Q" {
            self.screen = GameScreen::Climbing;
            return SummitAction::SaveStats;
        }

        let game = match lobby_manager.get_player_game_mut(self.user_id) {
            Some(g) => g,
            None => return SummitAction::Continue,
        };

        let climber = match game.run.get_climber_mut(self.user_id) {
            Some(c) => c,
            None => return SummitAction::Continue,
        };

        // Use item by number
        if let Ok(idx) = input.parse::<usize>() {
            let items: Vec<_> = climber.items.iter()
                .filter(|(_, count)| **count > 0)
                .collect();

            if idx > 0 && idx <= items.len() {
                let (item_type, _) = items[idx - 1];
                let item_type = *item_type;

                match item_type {
                    ItemType::ClimbingGloves => {
                        // Equip gloves - reduces stamina drain
                        climber.add_effect(StatusEffectType::Invulnerable, 50); // Placeholder
                        climber.use_item(item_type);
                    }
                    ItemType::SafetyHarness => {
                        // Passive - just mark as equipped
                    }
                    ItemType::GrapplingHook => {
                        // Would need directional input
                    }
                    _ => {}
                }
            }
        }

        SummitAction::SaveStats
    }

    fn handle_eat_food(&mut self, input: &str, lobby_manager: &mut SummitLobby) -> SummitAction {
        if input == "B" || input == "Q" {
            self.screen = GameScreen::Climbing;
            return SummitAction::SaveStats;
        }

        let game = match lobby_manager.get_player_game_mut(self.user_id) {
            Some(g) => g,
            None => return SummitAction::Continue,
        };

        let climber = match game.run.get_climber_mut(self.user_id) {
            Some(c) => c,
            None => return SummitAction::Continue,
        };

        // Eat food by number
        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= climber.foods.len() {
                let food_id = climber.foods[idx - 1];
                if let Some(food) = super::data::get_food(food_id) {
                    // Check if requires campfire
                    if food.requires_campfire {
                        // Can't eat here
                        return SummitAction::Continue;
                    }

                    // Apply food effects
                    apply_food_effects(climber, food, &mut self.stats);
                    climber.use_food(food_id);
                }
            }
        }

        self.screen = GameScreen::Climbing;
        SummitAction::SaveStats
    }

    fn handle_campfire(&mut self, input: &str, lobby_manager: &mut SummitLobby) -> SummitAction {
        match input {
            "R" => {
                // Rest - fast stamina regen
                if let Some(game) = lobby_manager.get_player_game_mut(self.user_id) {
                    if let Some(climber) = game.run.get_climber_mut(self.user_id) {
                        climber.regenerate_stamina(20);
                        // Cure cold
                        climber.remove_effect(StatusEffectType::Cold);
                    }
                }
                SummitAction::SaveStats
            }
            "M" => {
                // Roast marshmallow
                if let Some(game) = lobby_manager.get_player_game_mut(self.user_id) {
                    if let Some(climber) = game.run.get_climber_mut(self.user_id) {
                        if climber.has_item(ItemType::Marshmallow) {
                            self.screen = GameScreen::RoastMarshmallow { heat_level: 0 };
                            return SummitAction::SaveStats;
                        }
                    }
                }
                SummitAction::Continue
            }
            "C" => {
                // Cook food (eat campfire foods)
                self.screen = GameScreen::EatFood;
                SummitAction::SaveStats
            }
            "L" | "Q" => {
                // Leave campfire
                self.screen = GameScreen::Climbing;
                SummitAction::SaveStats
            }
            _ => SummitAction::Continue,
        }
    }

    fn handle_roast_marshmallow(&mut self, input: &str, heat_level: u32, lobby_manager: &mut SummitLobby) -> SummitAction {
        let game = match lobby_manager.get_player_game_mut(self.user_id) {
            Some(g) => g,
            None => {
                self.screen = GameScreen::Campfire;
                return SummitAction::SaveStats;
            }
        };

        let climber = match game.run.get_climber_mut(self.user_id) {
            Some(c) => c,
            None => {
                self.screen = GameScreen::Campfire;
                return SummitAction::SaveStats;
            }
        };

        match input {
            " " | "H" => {
                // Hold over fire - increase heat
                let new_heat = heat_level + 10;
                if new_heat >= 100 {
                    // Burnt!
                    climber.use_item(ItemType::Marshmallow);
                    climber.regenerate_stamina(5);
                    self.screen = GameScreen::Campfire;
                } else {
                    self.screen = GameScreen::RoastMarshmallow { heat_level: new_heat };
                }
                SummitAction::SaveStats
            }
            "E" | "R" => {
                // Remove from fire
                climber.use_item(ItemType::Marshmallow);

                if heat_level >= 60 && heat_level < 90 {
                    // Perfect roast!
                    climber.regenerate_stamina(20);
                    self.stats.perfect_roasts += 1;
                } else if heat_level >= 40 {
                    // Good roast
                    climber.regenerate_stamina(15);
                } else {
                    // Undercooked
                    climber.regenerate_stamina(10);
                }

                self.screen = GameScreen::Campfire;
                SummitAction::SaveStats
            }
            "Q" => {
                self.screen = GameScreen::Campfire;
                SummitAction::SaveStats
            }
            _ => {
                // Heat decays slightly
                let new_heat = heat_level.saturating_sub(5);
                self.screen = GameScreen::RoastMarshmallow { heat_level: new_heat };
                SummitAction::Continue
            }
        }
    }

    fn handle_revive(&mut self, input: &str, partner_id: i64, lobby_manager: &mut SummitLobby) -> SummitAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::Climbing;
            return SummitAction::SaveStats;
        }

        if input == "H" || input == "R" {
            let game = match lobby_manager.get_player_game_mut(self.user_id) {
                Some(g) => g,
                None => return SummitAction::Continue,
            };

            // Cost stamina to revive
            if let Some(helper) = game.run.get_climber_mut(self.user_id) {
                if helper.stamina_current >= 20 {
                    helper.stamina_current -= 20;
                    helper.revives_given += 1;
                }
            }

            // Revive partner
            if let Some(partner) = game.run.get_climber_mut(partner_id) {
                partner.revive();
            }

            self.screen = GameScreen::Climbing;
        }

        SummitAction::SaveStats
    }

    fn handle_summit(&mut self, _input: &str) -> SummitAction {
        self.screen = GameScreen::Results;
        SummitAction::SaveStats
    }

    fn handle_game_over(&mut self, _input: &str) -> SummitAction {
        self.screen = GameScreen::Results;
        SummitAction::SaveStats
    }

    fn handle_results(&mut self, input: &str, lobby_manager: &mut SummitLobby) -> SummitAction {
        if input == "C" || input == "\r" || input == "\n" || !input.is_empty() {
            // Record stats from run
            if let Some(game) = lobby_manager.get_player_game(self.user_id) {
                if let Some(climber) = game.run.get_climber(self.user_id) {
                    self.stats.record_run(&game.run, climber);
                    let _ = self.stats.check_badges();
                }
            }

            // Leave game
            let _ = lobby_manager.leave_game(self.user_id);
            self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
            return SummitAction::SaveStats;
        }
        SummitAction::Continue
    }

    fn handle_stats(&mut self, input: &str) -> SummitAction {
        if input == "B" || input == "Q" || !input.is_empty() {
            self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
            return SummitAction::SaveStats;
        }
        SummitAction::Continue
    }

    fn handle_cosmetics(&mut self, input: &str) -> SummitAction {
        if input == "B" || input == "Q" {
            self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
            return SummitAction::SaveStats;
        }

        // Cycle through cosmetics
        match input {
            "1" => {
                // Cycle uniform
                let uniforms: Vec<_> = super::data::COSMETICS.iter()
                    .filter(|c| c.cosmetic_type == super::data::CosmeticType::Uniform)
                    .filter(|c| self.stats.has_cosmetic(c.id))
                    .collect();

                if !uniforms.is_empty() {
                    let current_idx = uniforms.iter()
                        .position(|c| c.id == self.stats.equipped_cosmetics.uniform)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % uniforms.len();
                    self.stats.equipped_cosmetics.uniform = uniforms[next_idx].id.to_string();
                }
                SummitAction::SaveStats
            }
            "2" => {
                // Cycle hat
                let hats: Vec<_> = super::data::COSMETICS.iter()
                    .filter(|c| c.cosmetic_type == super::data::CosmeticType::Hat)
                    .filter(|c| self.stats.has_cosmetic(c.id))
                    .collect();

                if hats.is_empty() {
                    self.stats.equipped_cosmetics.hat = None;
                } else {
                    let current_idx = self.stats.equipped_cosmetics.hat.as_ref()
                        .and_then(|h| hats.iter().position(|c| c.id == h))
                        .map(|i| i + 1)
                        .unwrap_or(0);

                    if current_idx >= hats.len() {
                        self.stats.equipped_cosmetics.hat = None;
                    } else {
                        self.stats.equipped_cosmetics.hat = Some(hats[current_idx].id.to_string());
                    }
                }
                SummitAction::SaveStats
            }
            _ => SummitAction::Continue,
        }
    }

    fn handle_confirm_quit(&mut self, input: &str, lobby_manager: &mut SummitLobby) -> SummitAction {
        match input {
            "Y" => {
                let _ = lobby_manager.leave_game(self.user_id);
                SummitAction::Quit
            }
            _ => {
                // Return to previous screen
                if lobby_manager.get_player_game(self.user_id).is_some() {
                    self.screen = GameScreen::Climbing;
                } else {
                    self.screen = GameScreen::Lobby(LobbyScreen::MainMenu);
                }
                SummitAction::SaveStats
            }
        }
    }
}

/// Apply food effects to a climber
fn apply_food_effects(climber: &mut ClimberState, food: &super::data::Food, stats: &mut PlayerStats) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let effect = &food.effect;

    // Check for poison
    if effect.poison_chance > 0 && rng.gen_range(0..100) < effect.poison_chance {
        climber.add_effect(StatusEffectType::Poisoned, 60);
        stats.food_poisoning_count += 1;
        return;
    }

    // Stamina effects
    if effect.stamina_current > 0 {
        climber.regenerate_stamina(effect.stamina_current as u32);
    }

    if effect.stamina_max > 0 {
        climber.stamina_max = (climber.stamina_max + effect.stamina_max as u32).min(100);
    }

    // Status cures
    if effect.cure_cold {
        climber.remove_effect(StatusEffectType::Cold);
    }
    if effect.cure_poison {
        climber.remove_effect(StatusEffectType::Poisoned);
    }

    // Hunger relief reduces "hungry" effect
    if effect.hunger_relief > 0 {
        climber.remove_effect(StatusEffectType::Hungry);
    }

    // Stamina regen buff
    if effect.stamina_regen > 0 {
        // This would be a temporary buff - simplified for now
    }

    // Special effects
    if let Some(special) = effect.special {
        match special {
            "unlimited_stamina_15s" => {
                climber.add_effect(StatusEffectType::UnlimitedStamina, 150);
            }
            "invulnerable_10s" => {
                climber.add_effect(StatusEffectType::Invulnerable, 100);
            }
            "restore_max_stamina" => {
                climber.stamina_max = (climber.stamina_max + 20).min(100);
            }
            "random_effect" => {
                let roll = rng.gen_range(0..3);
                match roll {
                    0 => climber.regenerate_stamina(30),
                    1 => climber.add_effect(StatusEffectType::Poisoned, 30),
                    _ => climber.add_effect(StatusEffectType::Invulnerable, 50),
                }
            }
            "mystery_pill" => {
                let roll = rng.gen_range(0..4);
                match roll {
                    0 => climber.stamina_max = (climber.stamina_max + 30).min(100),
                    1 => climber.apply_stamina_damage(30, 15),
                    2 => climber.add_effect(StatusEffectType::Invulnerable, 200),
                    _ => climber.add_effect(StatusEffectType::Hallucinating, 100),
                }
            }
            "forbidden_snack" => {
                // Award badge
                stats.award_badge("forbidden_snack");
            }
            _ => {}
        }
    }

    // Side effects
    if let Some(side_effect) = effect.side_effect {
        if side_effect.contains("Jitters") {
            climber.add_effect(StatusEffectType::Jitters, 300);
        }
        if side_effect.contains("Heavy") {
            climber.add_effect(StatusEffectType::Heavy, 600);
        }
        if side_effect.contains("Exhaustion") {
            climber.add_effect(StatusEffectType::Exhausted, 300);
        }
        if side_effect.contains("Hallucination") {
            climber.add_effect(StatusEffectType::Hallucinating, 300);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_creation() {
        let stats = PlayerStats::new(1);
        let flow = SummitFlow::new(1, "TestUser".to_string(), stats);

        assert_eq!(flow.user_id, 1);
        assert_eq!(flow.handle, "TestUser");
        assert!(matches!(flow.screen, GameScreen::Lobby(LobbyScreen::MainMenu)));
    }

    #[test]
    fn test_lobby_navigation() {
        let stats = PlayerStats::new(1);
        let mut flow = SummitFlow::new(1, "TestUser".to_string(), stats);
        let mut lobby = SummitLobby::new();

        // Go to create game
        flow.input_buffer = "1".to_string();
        let _ = flow.process_input(&mut lobby);
        assert!(matches!(flow.screen, GameScreen::Lobby(LobbyScreen::CreateGame)));

        // Back to main menu
        flow.input_buffer = "B".to_string();
        let _ = flow.process_input(&mut lobby);
        assert!(matches!(flow.screen, GameScreen::Lobby(LobbyScreen::MainMenu)));
    }

    #[test]
    fn test_create_and_join_lobby() {
        let stats1 = PlayerStats::new(1);
        let stats2 = PlayerStats::new(2);
        let mut flow1 = SummitFlow::new(1, "Player1".to_string(), stats1);
        let mut flow2 = SummitFlow::new(2, "Player2".to_string(), stats2);
        let mut lobby = SummitLobby::new();

        // Player 1 creates public game
        flow1.screen = GameScreen::Lobby(LobbyScreen::CreateGame);
        flow1.input_buffer = "1".to_string();
        let _ = flow1.process_input(&mut lobby);
        assert!(matches!(flow1.screen, GameScreen::Lobby(LobbyScreen::WaitingRoom)));

        // Player 2 joins
        flow2.screen = GameScreen::Lobby(LobbyScreen::JoinGame);
        flow2.input_buffer = "1".to_string();
        let _ = flow2.process_input(&mut lobby);
        assert!(matches!(flow2.screen, GameScreen::Lobby(LobbyScreen::WaitingRoom)));

        // Verify both in same lobby
        assert_eq!(
            lobby.player_games.get(&1),
            lobby.player_games.get(&2)
        );
    }

    #[test]
    fn test_quit_confirmation() {
        let stats = PlayerStats::new(1);
        let mut flow = SummitFlow::new(1, "TestUser".to_string(), stats);
        let mut lobby = SummitLobby::new();

        flow.screen = GameScreen::ConfirmQuit;

        // Cancel quit
        flow.input_buffer = "N".to_string();
        let action = flow.process_input(&mut lobby);
        assert!(matches!(action, SummitAction::SaveStats));
        assert!(matches!(flow.screen, GameScreen::Lobby(LobbyScreen::MainMenu)));

        // Confirm quit
        flow.screen = GameScreen::ConfirmQuit;
        flow.input_buffer = "Y".to_string();
        let action = flow.process_input(&mut lobby);
        assert!(matches!(action, SummitAction::Quit));
    }
}
