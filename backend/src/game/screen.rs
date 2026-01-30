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
    /// Use drugs from inventory
    UseDrugs,
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
    Menu,           // Choose buy/sell or cancel pending
    Buying,         // Select drug to buy (instant purchase on keypress)
    Selling,        // Select drug to sell (instant sale on keypress)
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
        let prices = generate_prices_with_supply(&state.city, &state.location, &state.market_supply);
        Self {
            state,
            screen: GameScreen::Intro,
            prices,
            input_buffer: String::new(),
        }
    }

    /// Resume game from loaded state
    pub fn from_state(state: GameState) -> Self {
        let prices = generate_prices_with_supply(&state.city, &state.location, &state.market_supply);
        Self {
            state,
            screen: GameScreen::Intro,  // Show splash screen first
            prices,
            input_buffer: String::new(),
        }
    }

    /// Regenerate prices for current location (call after travel or when prices should change)
    pub fn refresh_prices(&mut self) {
        self.prices = generate_prices_with_supply(&self.state.city, &self.state.location, &self.state.market_supply);
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
                | GameScreen::Trade { .. }  // All trade modes are now single-key
                | GameScreen::UseDrugs
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
            GameScreen::UseDrugs => self.handle_use_drugs(&input),
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
        // Clear any displayed message when taking action
        self.state.last_message = None;

        match input {
            "T" => {
                self.screen = GameScreen::Travel { selecting_city: false };
                GtmAction::SaveGame
            }
            "B" => {
                self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                GtmAction::SaveGame
            }
            "L" => {
                self.screen = GameScreen::LoanShark;
                GtmAction::SaveGame
            }
            "K" if self.state.bank_unlocked => {
                self.screen = GameScreen::Bank;
                GtmAction::SaveGame
            }
            "H" => {
                let borough = get_borough(&self.state.city, &self.state.location);
                let is_mob = borough.map(|b| b.has_mob_doctor).unwrap_or(false);
                self.screen = GameScreen::Hospital { is_mob_doctor: is_mob };
                GtmAction::SaveGame
            }
            "G" => {
                self.screen = GameScreen::GunShop;
                GtmAction::SaveGame
            }
            "U" => {
                // Use drugs from inventory
                if self.state.inventory_count() > 0 {
                    self.screen = GameScreen::UseDrugs;
                    GtmAction::SaveGame
                } else {
                    self.state.last_message = Some("You don't have anything to use.".to_string());
                    GtmAction::SaveGame
                }
            }
            "C" => {
                if get_city(&self.state.city).map(|c| c.has_casino).unwrap_or(false) {
                    self.screen = GameScreen::Casino { game_type: None };
                    GtmAction::SaveGame
                } else {
                    GtmAction::Continue // No casino here
                }
            }
            "Q" => {
                self.screen = GameScreen::Quest;
                GtmAction::SaveGame
            }
            "S" => {
                // Show stats / leaderboard
                self.screen = GameScreen::Leaderboard;
                GtmAction::SaveGame
            }
            "X" => {
                self.screen = GameScreen::ConfirmQuit;
                GtmAction::SaveGame
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_travel(&mut self, input: &str, selecting_city: bool) -> GtmAction {
        if input == "Q" || input == "B" {
            self.screen = GameScreen::MainMenu;
            return GtmAction::SaveGame;
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
                        self.state.clear_pending_transaction(); // Clear pending on travel
                        self.use_action();
                        self.refresh_prices();

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
                return GtmAction::SaveGame;
            }

            if let Ok(idx) = input.parse::<usize>() {
                if let Some(city) = get_city(&self.state.city) {
                    if idx > 0 && idx <= city.boroughs.len() {
                        let borough = &city.boroughs[idx - 1];
                        if borough.key != self.state.location {
                            self.state.location = borough.key.to_string();
                            self.state.clear_pending_transaction(); // Clear pending on travel
                            self.use_action();
                            self.refresh_prices();

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
        use crate::game::{get_shop_inventory, PendingTransaction};

        match mode {
            TradeMode::Menu => {
                match input {
                    "B" => {
                        self.state.clear_pending_transaction();
                        self.screen = GameScreen::Trade { mode: TradeMode::Buying };
                        GtmAction::SaveGame
                    }
                    "S" => {
                        self.state.clear_pending_transaction();
                        self.screen = GameScreen::Trade { mode: TradeMode::Selling };
                        GtmAction::SaveGame
                    }
                    "C" => {
                        // Cancel pending transaction (costs 1 action)
                        if let Some(ref pending) = self.state.pending_transaction.clone() {
                            if self.state.actions_remaining > 0 {
                                self.state.actions_remaining -= 1;
                                if pending.is_purchase {
                                    // Refund: remove from inventory, return cash
                                    self.state.remove_inventory(&pending.commodity, pending.quantity);
                                    self.state.cash += pending.total_cost;
                                    self.state.stats.total_bought -= pending.total_cost;
                                    // Undo supply adjustment
                                    self.state.adjust_supply(&pending.commodity, pending.quantity as i32);
                                } else {
                                    // Undo sale: add back to inventory at original purchase price
                                    self.state.add_inventory(&pending.commodity, pending.quantity, pending.purchase_price);
                                    self.state.cash -= pending.total_cost;
                                    self.state.stats.total_sold -= pending.total_cost;
                                    self.state.stats.total_profit = self.state.stats.total_sold - self.state.stats.total_bought;
                                    // Undo supply adjustment
                                    self.state.adjust_supply(&pending.commodity, -(pending.quantity as i32));
                                }
                                self.state.clear_pending_transaction();
                            }
                        }
                        GtmAction::SaveGame
                    }
                    "Q" | "X" => {
                        self.state.clear_pending_transaction();
                        self.screen = GameScreen::MainMenu;
                        GtmAction::SaveGame
                    }
                    _ => GtmAction::Continue,
                }
            }
            TradeMode::Buying => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                    return GtmAction::SaveGame;
                }
                // Get local shop inventory
                let shop = get_shop_inventory(&self.state.city, &self.state.location);

                // Parse number input (0 = item 10)
                if let Ok(mut idx) = input.parse::<usize>() {
                    if idx == 0 { idx = 10; }

                    if idx >= 1 && idx <= shop.len() {
                        let (commodity_key, _) = shop[idx - 1];
                        if let Some(&price) = self.prices.get(commodity_key) {
                            let capacity_left = self.state.coat_capacity() - self.state.inventory_count();

                            // Can afford and have space?
                            if price <= self.state.cash && capacity_left >= 1 {
                                // Instant single-unit purchase with price tracking
                                self.state.cash -= price;
                                self.state.add_inventory(commodity_key, 1, price);
                                self.state.stats.total_bought += price;
                                // Buying reduces local supply (price goes up for next buy)
                                self.state.adjust_supply(commodity_key, -1);
                                // Store as pending transaction for cancel option
                                self.state.pending_transaction = Some(PendingTransaction {
                                    commodity: commodity_key.to_string(),
                                    quantity: 1,
                                    total_cost: price,
                                    is_purchase: true,
                                    purchase_price: price,
                                });
                                return GtmAction::SaveGame;
                            }
                        }
                    }
                }
                GtmAction::Continue
            }
            TradeMode::Selling => {
                if input == "Q" || input == "B" {
                    self.screen = GameScreen::Trade { mode: TradeMode::Menu };
                    return GtmAction::SaveGame;
                }
                // Get local shop inventory (can only sell what shop would buy)
                let shop = get_shop_inventory(&self.state.city, &self.state.location);

                // Parse number input (0 = item 10)
                if let Ok(mut idx) = input.parse::<usize>() {
                    if idx == 0 { idx = 10; }

                    if idx >= 1 && idx <= shop.len() {
                        let (commodity_key, _) = shop[idx - 1];
                        let owned = self.state.get_quantity(commodity_key);

                        if owned > 0 {
                            if let Some(&sell_price) = self.prices.get(commodity_key) {
                                // Get purchase price before removing (for cancel tracking)
                                let purchase_price = self.state.get_lowest_cost(commodity_key).unwrap_or(0);

                                // Instant single-unit sale (sells highest-profit lot first)
                                self.state.remove_inventory(commodity_key, 1);
                                self.state.cash += sell_price;
                                self.state.stats.total_sold += sell_price;
                                self.state.stats.total_profit = self.state.stats.total_sold - self.state.stats.total_bought;
                                // Selling increases local supply (price goes down)
                                self.state.adjust_supply(commodity_key, 1);
                                // Store as pending transaction for cancel option
                                self.state.pending_transaction = Some(PendingTransaction {
                                    commodity: commodity_key.to_string(),
                                    quantity: 1,
                                    total_cost: sell_price,
                                    is_purchase: false,
                                    purchase_price, // Track original cost for undo
                                });
                                return GtmAction::SaveGame;
                            }
                        }
                    }
                }
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

        // Save combat message to display on next screen
        self.state.last_message = Some(result.message.clone());

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
                GtmAction::SaveGame
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
                GtmAction::SaveGame
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_hospital(&mut self, input: &str, is_mob_doctor: bool) -> GtmAction {
        match input {
            "H" => {
                // Heal - costs money
                let heal_cost = if is_mob_doctor { 15000 } else { 10000 };
                if self.state.cash >= heal_cost && self.state.health < self.state.max_health {
                    self.state.cash -= heal_cost;
                    self.state.health = self.state.max_health;
                    self.state.stats.hospital_visits += 1;
                    self.state.last_message = Some("Patched up. Good as new.".to_string());
                }
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            "D" => {
                // Detox - clear high status
                let detox_cost = if is_mob_doctor { 25000 } else { 20000 }; // $200-250
                if self.state.cash >= detox_cost && self.state.high_tier > 0 {
                    self.state.cash -= detox_cost;
                    self.state.high_tier = 0;
                    self.state.last_message = Some("Detox complete. Head's clear now.".to_string());
                }
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            "Q" | "X" => {
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
            }
            _ => GtmAction::Continue,
        }
    }

    fn handle_gun_shop(&mut self, input: &str) -> GtmAction {
        use crate::game::{WEAPONS, get_weapon};

        if input == "Q" || input == "X" {
            self.screen = GameScreen::MainMenu;
            return GtmAction::SaveGame;
        }

        // Find weapon by first letter of key
        for weapon in WEAPONS.iter() {
            let first_char = weapon.key.chars().next().unwrap_or(' ').to_ascii_uppercase();
            if input == first_char.to_string() {
                // Can afford?
                if self.state.cash >= weapon.price {
                    self.state.cash -= weapon.price;

                    // Set weapon in appropriate slot
                    if weapon.is_gun {
                        self.state.weapons.gun = Some(weapon.key.to_string());
                    } else {
                        self.state.weapons.melee = Some(weapon.key.to_string());
                    }

                    self.state.last_message = Some(format!("Purchased {} for {}!", weapon.name, super::render::format_money(weapon.price)));
                    self.screen = GameScreen::MainMenu;
                    return GtmAction::SaveGame;
                } else {
                    self.state.last_message = Some(format!("Can't afford {} ({}).", weapon.name, super::render::format_money(weapon.price)));
                    return GtmAction::SaveGame;
                }
            }
        }

        GtmAction::Continue
    }

    fn handle_use_drugs(&mut self, input: &str) -> GtmAction {
        use crate::game::get_commodity;

        if input == "Q" || input == "X" {
            self.screen = GameScreen::MainMenu;
            return GtmAction::SaveGame;
        }

        // Get owned commodities (only show what we have)
        let owned: Vec<(String, u32)> = self.state.inventory_lots
            .iter()
            .map(|(k, lots)| (k.clone(), lots.iter().map(|l| l.quantity).sum()))
            .filter(|(_, qty)| *qty > 0)
            .collect();

        // Parse number input (1-9, 0 for 10)
        if let Ok(mut idx) = input.parse::<usize>() {
            if idx == 0 { idx = 10; }

            if idx >= 1 && idx <= owned.len() {
                let (commodity_key, _) = &owned[idx - 1];

                // Consume 1 unit
                self.state.remove_inventory(commodity_key, 1);

                // Get drug properties
                if let Some(commodity) = get_commodity(commodity_key) {
                    let mut effects: Vec<String> = Vec::new();

                    // Action boost
                    if commodity.action_boost {
                        let boost = 2;
                        self.state.actions_remaining += boost;
                        effects.push(format!("+{} actions", boost));
                    }

                    // Health effects based on drug type
                    let health_change = match commodity_key.as_str() {
                        "krokodil" => -10i32,  // Flesh-eating drug hurts
                        "fentanyl" => -5,      // Dangerous
                        "heroin" => -3,        // Addictive and harmful
                        "bathsalts" => -5,     // Can make you crazy
                        "meth" => -2,          // Wears you down
                        "speed" => -1,         // Mild wear
                        "weed" => 5,           // Relaxing, minor heal
                        "ludes" => 3,          // Chill
                        "acid" => 0,           // No physical effect
                        "dmt" => 0,            // Spiritual, no physical
                        "ketamine" => -2,      // Anesthetic effects
                        "mdma" => -1,          // Dehydration
                        "cocaine" => -2,       // Heart strain
                        "oxy" => -3,           // Opioid effects
                        "adderall" => 0,       // Focus drug
                        _ => 0,
                    };

                    // Weed makes you lazy - costs an action
                    if commodity_key == "weed" && self.state.actions_remaining > 0 {
                        self.state.actions_remaining -= 1;
                        effects.push("-1 action".to_string());
                    }

                    if health_change != 0 {
                        self.state.health = ((self.state.health as i32) + health_change)
                            .max(1)
                            .min(self.state.max_health as i32) as u32;

                        if health_change > 0 {
                            effects.push(format!("+{} HP", health_change));
                        } else {
                            effects.push(format!("{} HP", health_change));
                        }
                    }

                    // Addiction system
                    if commodity.addictive {
                        let addiction = self.state.addiction.entry(commodity_key.to_string()).or_insert(0);
                        *addiction = (*addiction + 1).min(100);
                        if *addiction >= 10 && *addiction % 10 == 0 {
                            effects.push(format!("addiction lv{}", addiction));
                        }
                    }

                    // Getting high - increase tier (max 3)
                    // Some drugs get you more high than others
                    let high_increase = match commodity_key.as_str() {
                        "acid" | "dmt" | "bathsalts" => 2,  // Psychedelics hit hard
                        "weed" | "ludes" | "tidepods" => 0, // Mild, no visual distortion
                        _ => 1,  // Most drugs increase by 1
                    };
                    if high_increase > 0 {
                        let old_tier = self.state.high_tier;
                        self.state.high_tier = (self.state.high_tier + high_increase).min(3);
                        if self.state.high_tier > old_tier {
                            effects.push(format!("high lv{}", self.state.high_tier));
                        }
                    }

                    // Special effects for certain drugs
                    let special = match commodity_key.as_str() {
                        "acid" => Some("The walls are breathing..."),
                        "dmt" => Some("You meet machine elves."),
                        "bathsalts" => Some("You feel invincible!"),
                        "mdma" => Some("Everyone is your friend!"),
                        "weed" => Some("Everything is chill."),
                        "cocaine" => Some("You could conquer the world!"),
                        "meth" => Some("You haven't slept in 3 days."),
                        "heroin" => Some("Everything fades away..."),
                        "ketamine" => Some("You enter the K-hole."),
                        "fentanyl" => Some("Careful with that..."),
                        "krokodil" => Some("That can't be good for you."),
                        _ => None,
                    };

                    let effect_str = if effects.is_empty() {
                        "No noticeable effect.".to_string()
                    } else {
                        effects.join(", ")
                    };

                    let msg = if let Some(special) = special {
                        format!("Used {}. {} {}", commodity.name, effect_str, special)
                    } else {
                        format!("Used {}. {}", commodity.name, effect_str)
                    };

                    self.state.last_message = Some(msg);
                }

                // Check for death from overdose
                if self.state.health == 0 {
                    self.state.game_over = true;
                    self.state.game_over_reason = Some("You overdosed.".to_string());
                    self.screen = GameScreen::GameOver;
                } else {
                    self.screen = GameScreen::MainMenu;
                }

                return GtmAction::SaveGame;
            }
        }

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
                GtmAction::SaveGame
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
            return GtmAction::SaveGame;
        }

        match game_type {
            None => {
                // Casino menu
                match input {
                    "1" | "B" => {
                        self.screen = GameScreen::Casino { game_type: Some(CasinoGame::Blackjack) };
                        return GtmAction::SaveGame;
                    }
                    "2" | "R" => {
                        self.screen = GameScreen::Casino { game_type: Some(CasinoGame::Roulette) };
                        return GtmAction::SaveGame;
                    }
                    "3" | "H" => {
                        self.screen = GameScreen::Casino { game_type: Some(CasinoGame::Horses) };
                        return GtmAction::SaveGame;
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
        GtmAction::SaveGame
    }

    fn handle_confirm_quit(&mut self, input: &str) -> GtmAction {
        match input {
            "Y" => GtmAction::Quit,
            _ => {
                self.screen = GameScreen::MainMenu;
                GtmAction::SaveGame
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

        // Coming down - high tier decreases by 1 each day
        if self.state.high_tier > 0 {
            self.state.high_tier -= 1;
        }

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

/// Generate market prices for a location using shop inventory and supply/demand
fn generate_prices(city: &str, location: &str) -> HashMap<String, i64> {
    generate_prices_with_supply(city, location, &HashMap::new())
}

/// Generate prices with supply modifier from game state
pub fn generate_prices_with_supply(city: &str, location: &str, market_supply: &HashMap<String, i32>) -> HashMap<String, i64> {
    use rand::Rng;
    use crate::game::get_shop_inventory;

    let mut rng = rand::thread_rng();
    let mut prices = HashMap::new();

    // Get what this shop sells
    let shop_inventory = get_shop_inventory(city, location);

    for (commodity_key, base_modifier) in shop_inventory {
        if let Some(commodity) = crate::game::get_commodity(commodity_key) {
            // Start with base price in range
            let base_price = rng.gen_range(commodity.min_price..=commodity.max_price);

            // Apply location modifier (80-120%)
            let location_mod = (base_modifier as f64) / 100.0;

            // Get supply modifier: negative = oversupplied (cheaper), positive = undersupplied (expensive)
            let supply_key = format!("{}/{}/{}", city, location, commodity_key);
            let supply_mod = market_supply.get(&supply_key).copied().unwrap_or(0);
            // Each supply point = 1% price change, max 50% swing
            let supply_factor = 1.0 + (supply_mod as f64 * -0.01);

            // Daily volatility +-15%
            let volatility = rng.gen_range(0.85..=1.15);

            let final_price = ((base_price as f64) * location_mod * supply_factor * volatility) as i64;
            prices.insert(commodity_key.to_string(), final_price.max(1)); // Min $0.01
        }
    }

    prices
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_starts_at_intro() {
        let flow = GtmFlow::new();
        assert!(matches!(flow.screen, GameScreen::Intro));
    }

    #[test]
    fn test_day_advance_on_zero_actions() {
        let mut flow = GtmFlow::new();
        flow.state.actions_remaining = 1;
        let initial_day = flow.state.day;

        flow.use_action();

        assert_eq!(flow.state.day, initial_day + 1);
        assert_eq!(flow.state.actions_remaining, 5);
    }

    #[test]
    fn test_game_over_at_day_90() {
        let mut flow = GtmFlow::new();
        flow.state.day = 90;
        flow.state.actions_remaining = 1;

        flow.use_action();

        assert_eq!(flow.state.day, 91);
        assert!(flow.state.game_over);
    }

    #[test]
    fn test_current_screen_returns_screen() {
        let mut flow = GtmFlow::new();
        flow.screen = GameScreen::MainMenu;
        assert!(matches!(flow.current_screen(), GameScreen::MainMenu));
    }

    #[test]
    fn test_advance_day_applies_interest() {
        let mut flow = GtmFlow::new();
        flow.state.debt = 100000;
        flow.state.bank_unlocked = true;
        flow.state.bank_balance = 100000;

        flow.advance_day();

        assert_eq!(flow.state.debt, 110000); // 10% debt interest
        assert_eq!(flow.state.bank_balance, 105000); // 5% bank interest
    }

    #[test]
    fn test_high_tier_decay() {
        let mut flow = GtmFlow::new();
        flow.state.high_tier = 3;

        flow.advance_day();

        assert_eq!(flow.state.high_tier, 2); // Decreases by 1 per day
    }
}
