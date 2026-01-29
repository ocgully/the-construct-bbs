use crate::game::{GameState, get_city, get_borough, get_commodity, COMMODITIES, CITIES};
use std::collections::HashMap;

/// Which screen the player is currently viewing
#[derive(Debug, Clone, PartialEq)]
pub enum GameScreen {
    /// New game intro story
    Intro,
    /// Main hub - choose what to do
    MainMenu,
    /// Travel between locations
    Travel { selecting_city: bool },
    /// Buy/sell at current location
    Trade { mode: TradeMode },
    /// Active combat encounter
    Combat { enemy_type: EnemyType, enemy_hp: u32 },
    /// Random event resolution
    Event { event: GameEvent },
    /// Loan shark menu
    LoanShark,
    /// Bank menu (if unlocked)
    Bank,
    /// Hospital or mob doctor
    Hospital { is_mob_doctor: bool },
    /// Gun shop
    GunShop,
    /// Quest status / delivery turn-in
    Quest,
    /// Casino (if available)
    Casino { game_type: Option<CasinoGame> },
    /// Game over screen
    GameOver,
    /// Leaderboard view
    Leaderboard,
    /// Quit confirmation
    ConfirmQuit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeMode {
    Menu,           // Choose buy/sell
    Buying,         // Select drug to buy
    Selling,        // Select drug to sell
    BuyAmount { commodity: String }, // Enter quantity
    SellAmount { commodity: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyType {
    Police,
    Mugger,
    Gang { gang_key: String },
    LoanSharkEnforcer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    PriceDrop { commodity: String, location: String },
    PriceSpike { commodity: String, location: String },
    TrenchcoatGuy,
    FindCash { amount: i64 },
    FindDrugs { commodity: String, amount: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CasinoGame {
    Blackjack,
    Roulette,
    Horses,
}

/// Actions returned by GtmFlow for session.rs to handle
#[derive(Debug, Clone)]
pub enum GtmAction {
    /// Continue - no output needed
    Continue,
    /// Show screen output
    Render(String),
    /// Echo character back
    Echo(String),
    /// Save game state (triggers DB save)
    SaveGame,
    /// Game is over (win or lose)
    GameOver { final_score: i64, story_completed: bool },
    /// Player quit to main menu
    Quit,
}

/// Grand Theft Meth game flow state machine
pub struct GtmFlow {
    pub state: GameState,
    pub screen: GameScreen,
    pub prices: HashMap<String, i64>,
    input_buffer: String,
}

impl GtmFlow {
    /// Create new game from fresh state
    pub fn new() -> Self {
        let state = GameState::new();
        let prices = generate_prices(&state.city, &state.location);
        Self {
            state,
            screen: GameScreen::Intro,
            prices,
            input_buffer: String::new(),
        }
    }

    /// Resume game from loaded state
    pub fn from_state(state: GameState) -> Self {
        let prices = generate_prices(&state.city, &state.location);
        Self {
            state,
            screen: GameScreen::MainMenu,
            prices,
            input_buffer: String::new(),
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
    pub fn handle_char(&mut self, ch: char) -> GtmAction {
        // Backspace handling
        if ch == '\x7f' || ch == '\x08' {
            if self.input_buffer.pop().is_some() {
                return GtmAction::Echo("\x08 \x08".to_string());
            }
            return GtmAction::Continue;
        }

        // Enter processing
        if ch == '\r' || ch == '\n' {
            return self.process_input();
        }

        // Ignore control chars
        if ch.is_control() {
            return GtmAction::Continue;
        }

        // For single-key screens, process immediately
        if self.is_single_key_screen() {
            self.input_buffer.clear();
            self.input_buffer.push(ch);
            return self.process_input();
        }

        // Buffer input (for quantity entry, etc.)
        if self.input_buffer.len() < 20 {
            self.input_buffer.push(ch);
            return GtmAction::Echo(ch.to_string());
        }

        GtmAction::Continue
    }

    /// Check if current screen uses single-key input
    fn is_single_key_screen(&self) -> bool {
        matches!(
            self.screen,
            GameScreen::MainMenu
                | GameScreen::Intro
                | GameScreen::Travel { .. }
                | GameScreen::Trade { mode: TradeMode::Menu }
                | GameScreen::Trade { mode: TradeMode::Buying }
                | GameScreen::Trade { mode: TradeMode::Selling }
                | GameScreen::Combat { .. }
                | GameScreen::Event { .. }
                | GameScreen::LoanShark
                | GameScreen::Bank
                | GameScreen::Hospital { .. }
                | GameScreen::GunShop
                | GameScreen::Quest
                | GameScreen::Casino { game_type: None }
                | GameScreen::GameOver
                | GameScreen::ConfirmQuit
        )
    }

    /// Process buffered input
    fn process_input(&mut self) -> GtmAction {
        let input = std::mem::take(&mut self.input_buffer);
        let input = input.trim().to_uppercase();

        match &self.screen {
            GameScreen::Intro => self.handle_intro(&input),
            GameScreen::MainMenu => self.handle_main_menu(&input),
            GameScreen::Travel { selecting_city } => self.handle_travel(&input, *selecting_city),
            GameScreen::Trade { mode } => self.handle_trade(&input, mode.clone()),
            GameScreen::Combat { enemy_type, enemy_hp } => {
                self.handle_combat(&input, enemy_type.clone(), *enemy_hp)
            }
            GameScreen::Event { event } => self.handle_event(&input, event.clone()),
            GameScreen::LoanShark => self.handle_loan_shark(&input),
            GameScreen::Bank => self.handle_bank(&input),
            GameScreen::Hospital { is_mob_doctor } => self.handle_hospital(&input, *is_mob_doctor),
            GameScreen::GunShop => self.handle_gun_shop(&input),
            GameScreen::Quest => self.handle_quest(&input),
            GameScreen::Casino { game_type } => self.handle_casino(&input, game_type.clone()),
            GameScreen::GameOver => self.handle_game_over(&input),
            GameScreen::Leaderboard => self.handle_leaderboard(&input),
            GameScreen::ConfirmQuit => self.handle_confirm_quit(&input),
        }
    }

    // Placeholder handlers - return to main menu for now
    fn handle_intro(&mut self, _input: &str) -> GtmAction {
        self.screen = GameScreen::MainMenu;
        GtmAction::SaveGame
    }

    fn handle_main_menu(&mut self, input: &str) -> GtmAction {
        match input {
            "T" => {
                self.screen = GameScreen::Travel { selecting_city: false };
                GtmAction::Continue
            }
            "B" => {
                self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                GtmAction::Continue
            }
            "L" => {
                self.screen = GameScreen::LoanShark;
                GtmAction::Continue
            }
            "K" if self.state.bank_unlocked => {
                self.screen = GameScreen::Bank;
                GtmAction::Continue
            }
            "H" => {
                let borough = get_borough(&self.state.city, &self.state.location);
                let is_mob = borough.map(|b| b.has_mob_doctor).unwrap_or(false);
                self.screen = GameScreen::Hospital { is_mob_doctor: is_mob };
                GtmAction::Continue
            }
            "G" => {
                self.screen = GameScreen::GunShop;
                GtmAction::Continue
            }
            "C" => {
                if get_city(&self.state.city).map(|c| c.has_casino).unwrap_or(false) {
                    self.screen = GameScreen::Casino { game_type: None };
                    GtmAction::Continue
                } else {
                    GtmAction::Continue // No casino here
                }
            }
            "Q" => {
                self.screen = GameScreen::Quest;
                GtmAction::Continue
            }
            "S" => {
                // Show stats / leaderboard
                self.screen = GameScreen::Leaderboard;
                GtmAction::Continue
            }
            "X" => {
                self.screen = GameScreen::ConfirmQuit;
                GtmAction::Continue
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_travel(&mut self, input: &str, selecting_city: bool) -> GtmAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return GtmAction::Continue;
        }

        if selecting_city {
            // Select city by number
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= CITIES.len() {
                    let city = &CITIES[idx - 1];
                    if city.key != self.state.city {
                        // Travel to new city - costs action and maybe days
                        // For now, just move
                        self.state.city = city.key.to_string();
                        self.state.location = city.boroughs[0].key.to_string();
                        self.use_action();
                        self.prices = generate_prices(&self.state.city, &self.state.location);

                        // Check for random event after travel
                        if let Some(event_screen) = crate::game::events::maybe_trigger_event(&self.state) {
                            self.screen = event_screen;
                            return GtmAction::SaveGame;
                        }
                    }
                    self.screen = GameScreen::MainMenu;
                    return GtmAction::SaveGame;
                }
            }
        } else {
            // Select borough by number, or C for city
            if input == "C" {
                self.screen = GameScreen::Travel { selecting_city: true };
                return GtmAction::Continue;
            }

            if let Ok(idx) = input.parse::<usize>() {
                if let Some(city) = get_city(&self.state.city) {
                    if idx > 0 && idx <= city.boroughs.len() {
                        let borough = &city.boroughs[idx - 1];
                        if borough.key != self.state.location {
                            self.state.location = borough.key.to_string();
                            self.use_action();
                            self.prices = generate_prices(&self.state.city, &self.state.location);

                            // Check for random event after travel
                            if let Some(event_screen) = crate::game::events::maybe_trigger_event(&self.state) {
                                self.screen = event_screen;
                                return GtmAction::SaveGame;
                            }
                        }
                        self.screen = GameScreen::MainMenu;
                        return GtmAction::SaveGame;
                    }
                }
            }
        }

        GtmAction::Continue
    }

    fn handle_trade(&mut self, input: &str, mode: TradeMode) -> GtmAction {
        match mode {
            TradeMode::Menu => {
                match input {
                    "B" => {
                        self.screen = GameScreen::Trade { mode: TradeMode::Buying };
                        GtmAction::Continue
                    }
                    "S" => {
                        self.screen = GameScreen::Trade { mode: TradeMode::Selling };
                        GtmAction::Continue
                    }
                    "Q" | "X" => {
                        self.screen = GameScreen::MainMenu;
                        GtmAction::Continue
                    }
                    _ => GtmAction::Continue,
                }
            }
            TradeMode::Buying => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                    return GtmAction::Continue;
                }
                // Select commodity by number
                if let Ok(idx) = input.parse::<usize>() {
                    if idx > 0 && idx <= COMMODITIES.len() {
                        let commodity = &COMMODITIES[idx - 1];
                        self.screen = GameScreen::Trade {
                            mode: TradeMode::BuyAmount { commodity: commodity.key.to_string() },
                        };
                    }
                }
                GtmAction::Continue
            }
            TradeMode::Selling => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                    return GtmAction::Continue;
                }
                // Select commodity by number
                if let Ok(idx) = input.parse::<usize>() {
                    if idx > 0 && idx <= COMMODITIES.len() {
                        let commodity = &COMMODITIES[idx - 1];
                        if self.state.inventory.get(commodity.key).copied().unwrap_or(0) > 0 {
                            self.screen = GameScreen::Trade {
                                mode: TradeMode::SellAmount { commodity: commodity.key.to_string() },
                            };
                        }
                    }
                }
                GtmAction::Continue
            }
            TradeMode::BuyAmount { ref commodity } => {
                if input == "Q" || input == "B" || input.is_empty() {
                    self.screen = GameScreen::Trade { mode: TradeMode::Buying };
                    return GtmAction::Continue;
                }
                if let Ok(qty) = input.parse::<u32>() {
                    if let Some(&price) = self.prices.get(commodity) {
                        let total = price * (qty as i64);
                        let capacity_left = self.state.coat_capacity() - self.state.inventory_count();
                        if qty <= capacity_left && total <= self.state.cash {
                            self.state.cash -= total;
                            *self.state.inventory.entry(commodity.clone()).or_insert(0) += qty;
                            self.state.stats.total_bought += total;
                            self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                            return GtmAction::SaveGame;
                        }
                    }
                }
                self.screen = GameScreen::Trade { mode: TradeMode::Buying };
                GtmAction::Continue
            }
            TradeMode::SellAmount { ref commodity } => {
                if input == "Q" || input == "B" || input.is_empty() {
                    self.screen = GameScreen::Trade { mode: TradeMode::Selling };
                    return GtmAction::Continue;
                }
                if let Ok(qty) = input.parse::<u32>() {
                    let owned = self.state.inventory.get(commodity).copied().unwrap_or(0);
                    if let Some(&price) = self.prices.get(commodity) {
                        if qty <= owned {
                            let total = price * (qty as i64);
                            self.state.cash += total;
                            *self.state.inventory.entry(commodity.clone()).or_insert(0) -= qty;
                            if self.state.inventory.get(commodity).copied().unwrap_or(0) == 0 {
                                self.state.inventory.remove(commodity);
                            }
                            self.state.stats.total_sold += total;
                            self.state.stats.total_profit = self.state.stats.total_sold - self.state.stats.total_bought;
                            self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                            return GtmAction::SaveGame;
                        }
                    }
                }
                self.screen = GameScreen::Trade { mode: TradeMode::Selling };
                GtmAction::Continue
            }
        }
    }

    fn handle_combat(&mut self, input: &str, enemy_type: EnemyType, enemy_hp: u32) -> GtmAction {
        use crate::game::events::{CombatAction, resolve_combat, apply_combat_result};

        let action = match input {
            "F" => CombatAction::Fight,
            "R" => CombatAction::Run,
            "T" => CombatAction::Talk,
            "B" => {
                // Bribe with 10% of cash
                let bribe_amount = self.state.cash / 10;
                if bribe_amount < 1000 {
                    return GtmAction::Continue; // Not enough to bribe
                }
                CombatAction::Bribe { amount: bribe_amount }
            }
            _ => return GtmAction::Continue,
        };

        let result = resolve_combat(&self.state, &enemy_type, enemy_hp, action);
        apply_combat_result(&mut self.state, &result);

        // Check for death
        if self.state.game_over {
            self.screen = GameScreen::GameOver;
        } else {
            self.screen = GameScreen::MainMenu;
        }

        GtmAction::SaveGame
    }

    fn handle_event(&mut self, _input: &str, event: GameEvent) -> GtmAction {
        use crate::game::events::{apply_find_event, apply_price_event, handle_trenchcoat_upgrade};

        match &event {
            GameEvent::FindCash { .. } | GameEvent::FindDrugs { .. } => {
                apply_find_event(&event, &mut self.state);
            }
            GameEvent::PriceDrop { .. } | GameEvent::PriceSpike { .. } => {
                apply_price_event(&event, &mut self.prices);
            }
            GameEvent::TrenchcoatGuy => {
                // Handle in separate screen transition
                // For now, auto-accept if have inventory to dump
                let _msg = handle_trenchcoat_upgrade(&mut self.state, true);
                // Could show message screen, but for now just continue
            }
        }

        self.screen = GameScreen::MainMenu;
        GtmAction::SaveGame
    }

    fn handle_loan_shark(&mut self, input: &str) -> GtmAction {
        use crate::game::economy::{borrow_money, pay_all_debt};

        match input {
            "P" => {
                // Pay all debt if possible
                match pay_all_debt(&mut self.state) {
                    Ok(_) => {
                        self.screen = GameScreen::MainMenu;
                        GtmAction::SaveGame
                    }
                    Err(_) => {
                        // Not enough cash - stay on screen
                        GtmAction::Continue
                    }
                }
            }
            "B" => {
                // Borrow 50% of current debt
                let borrow_amount = self.state.debt / 2;
                if borrow_amount > 0 {
                    let _ = borrow_money(&mut self.state, borrow_amount);
                }
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::MainMenu;
                GtmAction::Continue
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_bank(&mut self, input: &str) -> GtmAction {
        use crate::game::economy::{deposit_all, withdraw_all};

        match input {
            "D" => {
                // Deposit all cash
                let _ = deposit_all(&mut self.state);
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            "W" => {
                // Withdraw all
                let _ = withdraw_all(&mut self.state);
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::MainMenu;
                GtmAction::Continue
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_hospital(&mut self, input: &str, _is_mob_doctor: bool) -> GtmAction {
        match input {
            "H" => {
                // Heal - costs money
                let heal_cost = 10000; // $100 per visit
                if self.state.cash >= heal_cost {
                    self.state.cash -= heal_cost;
                    self.state.health = self.state.max_health;
                    self.state.stats.hospital_visits += 1;
                }
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::MainMenu;
                GtmAction::Continue
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_gun_shop(&mut self, _input: &str) -> GtmAction {
        // TODO: implement gun shop
        self.screen = GameScreen::MainMenu;
        GtmAction::Continue
    }

    fn handle_quest(&mut self, input: &str) -> GtmAction {
        use crate::game::quest::{complete_story_step, can_complete_story_step};

        match input {
            "S" => {
                if can_complete_story_step(&self.state, &self.prices) {
                    match complete_story_step(&mut self.state, &self.prices) {
                        Ok((_title, _reward)) => {
                            // Story step completed
                            self.screen = GameScreen::MainMenu;
                            return GtmAction::SaveGame;
                        }
                        Err(_) => {}
                    }
                }
                GtmAction::Continue
            }
            "Q" | "X" => {
                self.screen = GameScreen::MainMenu;
                GtmAction::Continue
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_casino(&mut self, input: &str, game_type: Option<CasinoGame>) -> GtmAction {
        use crate::game::economy::{play_blackjack, play_roulette, RouletteBet, bet_on_horse};

        if input == "Q" || input == "X" {
            if game_type.is_none() {
                self.screen = GameScreen::MainMenu;
            } else {
                self.screen = GameScreen::Casino { game_type: None };
            }
            return GtmAction::Continue;
        }

        match game_type {
            None => {
                // Casino menu
                match input {
                    "1" | "B" => {
                        self.screen = GameScreen::Casino { game_type: Some(CasinoGame::Blackjack) };
                    }
                    "2" | "R" => {
                        self.screen = GameScreen::Casino { game_type: Some(CasinoGame::Roulette) };
                    }
                    "3" | "H" => {
                        self.screen = GameScreen::Casino { game_type: Some(CasinoGame::Horses) };
                    }
                    _ => {}
                }
                GtmAction::Continue
            }
            Some(CasinoGame::Blackjack) => {
                // Bet 10% of cash
                let bet = (self.state.cash / 10).max(100);
                if self.state.cash >= bet {
                    let _ = play_blackjack(&mut self.state, bet);
                }
                GtmAction::SaveGame
            }
            Some(CasinoGame::Roulette) => {
                let bet = (self.state.cash / 10).max(100);
                if self.state.cash >= bet {
                    let bet_type = match input {
                        "R" => RouletteBet::Red,
                        "B" => RouletteBet::Black,
                        "O" => RouletteBet::Odd,
                        "E" => RouletteBet::Even,
                        _ => RouletteBet::Red,
                    };
                    let _ = play_roulette(&mut self.state, bet, bet_type);
                }
                GtmAction::SaveGame
            }
            Some(CasinoGame::Horses) => {
                let bet = (self.state.cash / 10).max(100);
                if let Ok(horse) = input.parse::<u8>() {
                    if horse >= 1 && horse <= 6 && self.state.cash >= bet {
                        let _ = bet_on_horse(&mut self.state, bet, horse);
                    }
                }
                GtmAction::SaveGame
            }
        }
    }

    fn handle_game_over(&mut self, _input: &str) -> GtmAction {
        GtmAction::GameOver {
            final_score: self.state.net_worth(&self.prices),
            story_completed: self.state.quest_state.story_step >= 15,
        }
    }

    fn handle_leaderboard(&mut self, _input: &str) -> GtmAction {
        self.screen = GameScreen::MainMenu;
        GtmAction::Continue
    }

    fn handle_confirm_quit(&mut self, input: &str) -> GtmAction {
        match input {
            "Y" => GtmAction::Quit,
            _ => {
                self.screen = GameScreen::MainMenu;
                GtmAction::Continue
            }
        }
    }

    /// Use one action, advance day if needed
    fn use_action(&mut self) {
        if self.state.actions_remaining > 0 {
            self.state.actions_remaining -= 1;
        }

        if self.state.actions_remaining == 0 {
            self.advance_day();
        }
    }

    /// Advance to next day
    fn advance_day(&mut self) {
        self.state.day += 1;
        self.state.actions_remaining = 5;

        // Apply daily effects
        self.state.apply_debt_interest();
        self.state.apply_bank_interest();
        self.state.decay_notoriety();

        // Check for game over
        if self.state.day > 90 {
            self.state.game_over = true;
            self.state.game_over_reason = Some("Time's up! 90 days have passed.".to_string());
            self.screen = GameScreen::GameOver;
        }

        // Check bank unlock
        if !self.state.bank_unlocked && self.state.cash >= 5000000 {
            self.state.bank_unlocked = true;
        }

        // Update max net worth stat
        let net_worth = self.state.net_worth(&self.prices);
        if net_worth > self.state.stats.max_net_worth {
            self.state.stats.max_net_worth = net_worth;
        }
    }
}

/// Generate market prices for a location
fn generate_prices(city: &str, _location: &str) -> HashMap<String, i64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut prices = HashMap::new();

    for commodity in COMMODITIES.iter() {
        let base_price = rng.gen_range(commodity.min_price..=commodity.max_price);

        // Regional modifier
        let modifier = match (city, commodity.key) {
            ("tokyo", "meth") => 1.5,
            ("london", "cocaine") => 0.7,
            ("bogota", "cocaine") => 0.5,
            _ => 1.0,
        };

        // Volatility +-20%
        let volatility = rng.gen_range(0.8..=1.2);

        let final_price = ((base_price as f64) * modifier * volatility) as i64;
        prices.insert(commodity.key.to_string(), final_price);
    }

    prices
}
