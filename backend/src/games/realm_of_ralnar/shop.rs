//! Shop System for Realm of Ralnar
//!
//! Handles buying and selling items, weapons, armor, and inn services.

use serde::{Deserialize, Serialize};
use super::state::GameState;

// ============================================================================
// SHOP TYPES
// ============================================================================

/// Type of shop determining what can be bought/sold
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopType {
    /// General item shop (potions, consumables)
    Items,
    /// Weapon shop
    Weapons,
    /// Armor and accessories
    Armor,
    /// Inn - rest and heal
    Inn { price: u32, heal_full: bool },
    /// Magic shop (spells, scrolls)
    Magic,
    /// Mixed shop (sells everything)
    General,
}

impl ShopType {
    /// Get display name for this shop type
    pub fn display_name(&self) -> &'static str {
        match self {
            ShopType::Items => "Item Shop",
            ShopType::Weapons => "Weapon Shop",
            ShopType::Armor => "Armor Shop",
            ShopType::Inn { .. } => "Inn",
            ShopType::Magic => "Magic Shop",
            ShopType::General => "General Store",
        }
    }
}

// ============================================================================
// SHOP ITEM
// ============================================================================

/// An item available in a shop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    /// Item identifier
    pub item_id: String,
    /// Display name (can differ from item_id)
    pub name: String,
    /// Current stock (None = infinite)
    pub stock: Option<u32>,
    /// Base price (None = use item's default price)
    pub price_override: Option<u32>,
    /// Description shown in shop
    pub description: String,
}

impl ShopItem {
    /// Create a new shop item with infinite stock
    pub fn new(item_id: &str, name: &str, price: u32, description: &str) -> Self {
        Self {
            item_id: item_id.to_string(),
            name: name.to_string(),
            stock: None,
            price_override: Some(price),
            description: description.to_string(),
        }
    }

    /// Create a shop item with limited stock
    pub fn limited(item_id: &str, name: &str, price: u32, stock: u32, description: &str) -> Self {
        Self {
            item_id: item_id.to_string(),
            name: name.to_string(),
            stock: Some(stock),
            price_override: Some(price),
            description: description.to_string(),
        }
    }

    /// Check if item is in stock
    pub fn in_stock(&self) -> bool {
        self.stock.map_or(true, |s| s > 0)
    }

    /// Get the price for this item
    pub fn get_price(&self, base_price: u32) -> u32 {
        self.price_override.unwrap_or(base_price)
    }

    /// Reduce stock by one if limited
    pub fn reduce_stock(&mut self) {
        if let Some(ref mut stock) = self.stock {
            *stock = stock.saturating_sub(1);
        }
    }
}

// ============================================================================
// SHOP STRUCTURE
// ============================================================================

/// A complete shop definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    /// Unique shop identifier
    pub id: String,
    /// Display name of the shop
    pub name: String,
    /// Type of shop
    pub shop_type: ShopType,
    /// Items for sale
    pub inventory: Vec<ShopItem>,
    /// Price multiplier when buying (1.0 = normal)
    pub buy_rate: f32,
    /// Price multiplier when selling (0.5 = 50% of value)
    pub sell_rate: f32,
    /// Welcome message
    pub welcome_message: String,
    /// Farewell message
    pub farewell_message: String,
}

impl Shop {
    /// Create a new shop
    pub fn new(id: &str, name: &str, shop_type: ShopType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            shop_type,
            inventory: Vec::new(),
            buy_rate: 1.0,
            sell_rate: 0.5,
            welcome_message: "Welcome! Take a look around.".to_string(),
            farewell_message: "Come again!".to_string(),
        }
    }

    /// Add an item to the shop's inventory
    pub fn add_item(mut self, item: ShopItem) -> Self {
        self.inventory.push(item);
        self
    }

    /// Set buy rate
    pub fn with_buy_rate(mut self, rate: f32) -> Self {
        self.buy_rate = rate;
        self
    }

    /// Set sell rate
    pub fn with_sell_rate(mut self, rate: f32) -> Self {
        self.sell_rate = rate;
        self
    }

    /// Calculate buying price for an item
    pub fn buy_price(&self, base_price: u32) -> u32 {
        ((base_price as f32) * self.buy_rate).ceil() as u32
    }

    /// Calculate selling price for an item
    pub fn sell_price(&self, base_price: u32) -> u32 {
        ((base_price as f32) * self.sell_rate).floor() as u32
    }

    /// Get item by index
    pub fn get_item(&self, index: usize) -> Option<&ShopItem> {
        self.inventory.get(index)
    }

    /// Get mutable item by index
    pub fn get_item_mut(&mut self, index: usize) -> Option<&mut ShopItem> {
        self.inventory.get_mut(index)
    }
}

// ============================================================================
// SHOP MODES
// ============================================================================

/// Current mode in shop interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopMode {
    /// Main menu (Buy/Sell/Leave)
    #[default]
    MainMenu,
    /// Browsing items to buy
    Buying,
    /// Selecting items to sell
    Selling,
    /// Confirming a purchase
    ConfirmBuy { item_index: usize, quantity: u32 },
    /// Confirming a sale
    ConfirmSell { item_index: usize, quantity: u32 },
    /// Staying at an inn
    Staying,
    /// Viewing a message
    Message,
}

/// Actions that can result from shop interaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShopAction {
    /// Continue shopping
    Continue,
    /// Exit the shop
    Exit,
    /// Display a message
    Message(String),
    /// Error occurred
    Error(String),
    /// Party was healed (for inns)
    Healed,
}

// ============================================================================
// SHOP STATE
// ============================================================================

/// Active shop state during interaction
#[derive(Debug)]
pub struct ShopState {
    /// The shop being visited
    pub shop: Shop,
    /// Current interaction mode
    pub mode: ShopMode,
    /// Selected item index
    pub selected_index: usize,
    /// Current message to display
    pub message: Option<String>,
    /// Quantity being purchased/sold
    pub quantity: u32,
    /// Player's sellable items (indices into inventory)
    pub sellable_items: Vec<(usize, String, u32)>, // (index, item_id, price)
}

impl ShopState {
    /// Create a new shop state
    pub fn new(shop: Shop) -> Self {
        Self {
            shop,
            mode: ShopMode::MainMenu,
            selected_index: 0,
            message: None,
            quantity: 1,
            sellable_items: Vec::new(),
        }
    }

    /// Initialize sellable items from player inventory
    pub fn init_sellable_items(&mut self, game_state: &GameState) {
        self.sellable_items.clear();

        // Create sellable entries from inventory items
        for (idx, inv_item) in game_state.inventory.items.iter().enumerate() {
            if inv_item.quantity > 0 {
                let base_price = get_item_base_price(&inv_item.key);
                let sell_price = self.shop.sell_price(base_price);
                self.sellable_items.push((idx, inv_item.key.clone(), sell_price));
            }
        }
    }

    /// Get available items for purchase (in stock only)
    pub fn available_items(&self) -> Vec<(usize, &ShopItem)> {
        self.shop.inventory.iter()
            .enumerate()
            .filter(|(_, item)| item.in_stock())
            .collect()
    }

    /// Handle input in main menu mode
    pub fn handle_main_menu(&mut self, input: char) -> ShopAction {
        match input.to_ascii_lowercase() {
            'b' | '1' => {
                if self.shop.inventory.is_empty() {
                    ShopAction::Message("Nothing for sale right now.".to_string())
                } else {
                    self.mode = ShopMode::Buying;
                    self.selected_index = 0;
                    ShopAction::Continue
                }
            }
            's' | '2' => {
                if self.sellable_items.is_empty() {
                    ShopAction::Message("You have nothing to sell.".to_string())
                } else {
                    self.mode = ShopMode::Selling;
                    self.selected_index = 0;
                    ShopAction::Continue
                }
            }
            'l' | 'q' | '3' | '\x1b' => ShopAction::Exit,
            'r' | '4' if matches!(self.shop.shop_type, ShopType::Inn { .. }) => {
                self.mode = ShopMode::Staying;
                ShopAction::Continue
            }
            _ => ShopAction::Continue,
        }
    }

    /// Handle input in buying mode
    pub fn handle_buying(&mut self, input: char, game_state: &GameState) -> ShopAction {
        let available = self.available_items();
        if available.is_empty() {
            self.mode = ShopMode::MainMenu;
            return ShopAction::Message("Nothing available.".to_string());
        }

        match input.to_ascii_lowercase() {
            'w' | '\x1b' | 'q' => {
                // Up or back
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.mode = ShopMode::MainMenu;
                }
                ShopAction::Continue
            }
            's' => {
                // Down
                if self.selected_index < available.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                ShopAction::Continue
            }
            '\n' | ' ' | 'e' => {
                // Select item
                if let Some((actual_idx, item)) = available.get(self.selected_index) {
                    let price = self.shop.buy_price(item.get_price(100));
                    if game_state.gold >= price {
                        self.mode = ShopMode::ConfirmBuy {
                            item_index: *actual_idx,
                            quantity: 1,
                        };
                        ShopAction::Continue
                    } else {
                        ShopAction::Message("You don't have enough gold!".to_string())
                    }
                } else {
                    ShopAction::Continue
                }
            }
            'b' => {
                self.mode = ShopMode::MainMenu;
                ShopAction::Continue
            }
            _ => ShopAction::Continue,
        }
    }

    /// Handle input in selling mode
    pub fn handle_selling(&mut self, input: char) -> ShopAction {
        if self.sellable_items.is_empty() {
            self.mode = ShopMode::MainMenu;
            return ShopAction::Message("Nothing to sell.".to_string());
        }

        match input.to_ascii_lowercase() {
            'w' | '\x1b' | 'q' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.mode = ShopMode::MainMenu;
                }
                ShopAction::Continue
            }
            's' => {
                if self.selected_index < self.sellable_items.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                ShopAction::Continue
            }
            '\n' | ' ' | 'e' => {
                if self.selected_index < self.sellable_items.len() {
                    let (idx, _, _) = self.sellable_items[self.selected_index].clone();
                    self.mode = ShopMode::ConfirmSell {
                        item_index: idx,
                        quantity: 1,
                    };
                }
                ShopAction::Continue
            }
            'b' => {
                self.mode = ShopMode::MainMenu;
                ShopAction::Continue
            }
            _ => ShopAction::Continue,
        }
    }

    /// Handle main input routing
    pub fn handle_input(&mut self, input: char, game_state: &mut GameState) -> ShopAction {
        match self.mode {
            ShopMode::MainMenu => self.handle_main_menu(input),
            ShopMode::Buying => self.handle_buying(input, game_state),
            ShopMode::Selling => self.handle_selling(input),
            ShopMode::ConfirmBuy { item_index, quantity } => {
                match input.to_ascii_lowercase() {
                    'y' | '\n' => {
                        let result = self.buy_item(item_index, quantity, game_state);
                        self.mode = ShopMode::Buying;
                        result
                    }
                    'n' | '\x1b' | 'q' => {
                        self.mode = ShopMode::Buying;
                        ShopAction::Continue
                    }
                    _ => ShopAction::Continue,
                }
            }
            ShopMode::ConfirmSell { item_index, quantity } => {
                match input.to_ascii_lowercase() {
                    'y' | '\n' => {
                        let result = self.sell_item(item_index, quantity, game_state);
                        self.init_sellable_items(game_state);
                        self.mode = ShopMode::Selling;
                        if self.sellable_items.is_empty() {
                            self.mode = ShopMode::MainMenu;
                        } else if self.selected_index >= self.sellable_items.len() {
                            self.selected_index = self.sellable_items.len().saturating_sub(1);
                        }
                        result
                    }
                    'n' | '\x1b' | 'q' => {
                        self.mode = ShopMode::Selling;
                        ShopAction::Continue
                    }
                    _ => ShopAction::Continue,
                }
            }
            ShopMode::Staying => {
                match input.to_ascii_lowercase() {
                    'y' | '\n' => {
                        let result = self.stay_at_inn(game_state);
                        self.mode = ShopMode::MainMenu;
                        result
                    }
                    'n' | '\x1b' | 'q' => {
                        self.mode = ShopMode::MainMenu;
                        ShopAction::Continue
                    }
                    _ => ShopAction::Continue,
                }
            }
            ShopMode::Message => {
                self.message = None;
                self.mode = ShopMode::MainMenu;
                ShopAction::Continue
            }
        }
    }

    /// Buy an item from the shop
    pub fn buy_item(&mut self, item_index: usize, quantity: u32, state: &mut GameState) -> ShopAction {
        let item = match self.shop.get_item(item_index) {
            Some(i) => i.clone(),
            None => return ShopAction::Error("Item not found.".to_string()),
        };

        // Check stock
        if let Some(stock) = item.stock {
            if stock < quantity {
                return ShopAction::Error(format!("Only {} in stock.", stock));
            }
        }

        // Calculate total price
        let unit_price = self.shop.buy_price(item.get_price(100));
        let total_price = unit_price * quantity;

        // Check gold
        if state.gold < total_price {
            return ShopAction::Error(format!(
                "Not enough gold! Need {} but only have {}.",
                total_price, state.gold
            ));
        }

        // Complete transaction
        state.gold -= total_price;

        // Add items to inventory
        state.inventory.add(&item.item_id, quantity);

        // Reduce shop stock
        if let Some(shop_item) = self.shop.get_item_mut(item_index) {
            for _ in 0..quantity {
                shop_item.reduce_stock();
            }
        }

        ShopAction::Message(format!(
            "Bought {} x{} for {} gold.",
            item.name, quantity, total_price
        ))
    }

    /// Sell an item to the shop
    pub fn sell_item(&mut self, item_index: usize, quantity: u32, state: &mut GameState) -> ShopAction {
        // Get item from sellable items list
        if item_index >= self.sellable_items.len() {
            return ShopAction::Error("Invalid item.".to_string());
        }

        let (_, item_id, _) = &self.sellable_items[item_index];
        let item_id = item_id.clone();

        // Count how many of this item the player has
        let owned = state.inventory.count(&item_id);
        if owned < quantity {
            return ShopAction::Error(format!("You only have {} of that.", owned));
        }

        // Calculate sell price
        let base_price = get_item_base_price(&item_id);
        let unit_price = self.shop.sell_price(base_price);
        let total_price = unit_price * quantity;

        // Remove items from inventory
        state.inventory.remove(&item_id, quantity);

        // Add gold
        state.gold += total_price;

        ShopAction::Message(format!(
            "Sold {} x{} for {} gold.",
            item_id, quantity, total_price
        ))
    }

    /// Stay at an inn
    pub fn stay_at_inn(&mut self, state: &mut GameState) -> ShopAction {
        if let ShopType::Inn { price, heal_full } = self.shop.shop_type {
            if state.gold < price {
                return ShopAction::Error(format!(
                    "A room costs {} gold, but you only have {}.",
                    price, state.gold
                ));
            }

            state.gold -= price;

            if heal_full {
                state.party.full_heal();
            } else {
                // Partial heal
                for member in &mut state.party.members {
                    member.hp = (member.hp + member.hp_max / 2).min(member.hp_max);
                    member.mp = (member.mp + member.mp_max / 2).min(member.mp_max);
                }
            }

            ShopAction::Healed
        } else {
            ShopAction::Error("This isn't an inn!".to_string())
        }
    }

    /// Get current mode display name
    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            ShopMode::MainMenu => "Shop Menu",
            ShopMode::Buying => "Buy Items",
            ShopMode::Selling => "Sell Items",
            ShopMode::ConfirmBuy { .. } => "Confirm Purchase",
            ShopMode::ConfirmSell { .. } => "Confirm Sale",
            ShopMode::Staying => "Rest?",
            ShopMode::Message => "Message",
        }
    }
}

// ============================================================================
// ITEM PRICING (placeholder - would normally come from item database)
// ============================================================================

/// Get base price for an item
fn get_item_base_price(item_id: &str) -> u32 {
    match item_id {
        // Healing items
        "potion" | "health_potion" => 50,
        "hi_potion" => 150,
        "mega_potion" => 400,
        "ether" | "mana_potion" => 100,
        "elixir" => 1000,
        "phoenix_down" => 500,
        "antidote" => 30,

        // Equipment - Weapons
        "rusty_sword" => 20,
        "iron_sword" => 200,
        "steel_sword" => 500,
        "silver_sword" => 1200,
        "magic_sword" => 3000,
        "wooden_staff" => 100,
        "iron_staff" => 400,

        // Equipment - Armor
        "leather_armor" => 150,
        "chain_mail" => 400,
        "plate_armor" => 1000,
        "wooden_shield" => 80,
        "iron_shield" => 250,

        // Quest items (not sellable for much)
        "torch" => 10,
        "key" => 1,
        "letter" => 1,

        // Default
        _ => 100,
    }
}

// ============================================================================
// PREDEFINED SHOPS
// ============================================================================

/// Create the village item shop
pub fn create_village_item_shop() -> Shop {
    Shop::new("village_items", "Ye Olde Item Shoppe", ShopType::Items)
        .add_item(ShopItem::new("potion", "Potion", 50, "Restores 50 HP"))
        .add_item(ShopItem::new("hi_potion", "Hi-Potion", 150, "Restores 150 HP"))
        .add_item(ShopItem::new("antidote", "Antidote", 30, "Cures poison"))
        .add_item(ShopItem::new("phoenix_down", "Phoenix Down", 500, "Revives fallen ally"))
        .add_item(ShopItem::new("ether", "Ether", 100, "Restores 30 MP"))
        .with_buy_rate(1.0)
        .with_sell_rate(0.5)
}

/// Create the village weapon shop
pub fn create_village_weapon_shop() -> Shop {
    Shop::new("village_weapons", "Blacksmith's Forge", ShopType::Weapons)
        .add_item(ShopItem::new("iron_sword", "Iron Sword", 200, "A sturdy iron blade"))
        .add_item(ShopItem::new("iron_staff", "Iron Staff", 180, "A reinforced staff"))
        .add_item(ShopItem::new("iron_shield", "Iron Shield", 150, "Basic protection"))
        .with_buy_rate(1.0)
        .with_sell_rate(0.4)
}

/// Create the village inn
pub fn create_village_inn() -> Shop {
    let mut shop = Shop::new("village_inn", "The Weary Traveler", ShopType::Inn { price: 30, heal_full: true });
    shop.welcome_message = "Welcome, traveler! A warm bed awaits.".to_string();
    shop.farewell_message = "Rest well on your journey!".to_string();
    shop
}

/// Create a traveling merchant (appears randomly)
pub fn create_traveling_merchant() -> Shop {
    Shop::new("traveling_merchant", "Mysterious Merchant", ShopType::General)
        .add_item(ShopItem::limited("elixir", "Elixir", 800, 2, "Full HP/MP restore - rare!"))
        .add_item(ShopItem::limited("magic_sword", "Magic Sword", 2500, 1, "Glowing blade of power"))
        .add_item(ShopItem::new("potion", "Potion", 40, "Restores 50 HP - discount!"))
        .with_buy_rate(0.8)  // Discount prices!
        .with_sell_rate(0.6) // Better sell prices!
}

/// Create city weapon shop (better equipment)
pub fn create_city_weapon_shop() -> Shop {
    Shop::new("city_weapons", "Royal Armory", ShopType::Weapons)
        .add_item(ShopItem::new("steel_sword", "Steel Sword", 500, "Sharp and reliable"))
        .add_item(ShopItem::new("silver_sword", "Silver Sword", 1200, "Effective against undead"))
        .add_item(ShopItem::new("plate_armor", "Plate Armor", 1000, "Heavy but protective"))
        .with_buy_rate(1.1)  // Royal prices are higher
        .with_sell_rate(0.45)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> GameState {
        let mut state = GameState::new(1, "TestPlayer".to_string());
        state.gold = 500;
        state
    }

    fn create_test_shop() -> Shop {
        Shop::new("test_shop", "Test Shop", ShopType::Items)
            .add_item(ShopItem::new("potion", "Potion", 50, "Heals 50 HP"))
            .add_item(ShopItem::new("hi_potion", "Hi-Potion", 150, "Heals 150 HP"))
            .add_item(ShopItem::limited("rare_item", "Rare Item", 200, 3, "Limited stock"))
    }

    #[test]
    fn test_shop_creation() {
        let shop = create_test_shop();
        assert_eq!(shop.id, "test_shop");
        assert_eq!(shop.name, "Test Shop");
        assert_eq!(shop.inventory.len(), 3);
    }

    #[test]
    fn test_shop_item_creation() {
        let item = ShopItem::new("potion", "Potion", 50, "Heals");
        assert!(item.in_stock());
        assert_eq!(item.get_price(100), 50); // Override price

        let limited = ShopItem::limited("rare", "Rare", 100, 5, "Limited");
        assert!(limited.in_stock());
        assert_eq!(limited.stock, Some(5));
    }

    #[test]
    fn test_shop_item_stock_reduction() {
        let mut item = ShopItem::limited("item", "Item", 100, 3, "Test");
        assert!(item.in_stock());

        item.reduce_stock();
        assert_eq!(item.stock, Some(2));

        item.reduce_stock();
        item.reduce_stock();
        assert_eq!(item.stock, Some(0));
        assert!(!item.in_stock());

        // Should not go negative
        item.reduce_stock();
        assert_eq!(item.stock, Some(0));
    }

    #[test]
    fn test_infinite_stock() {
        let mut item = ShopItem::new("item", "Item", 100, "Test");
        assert!(item.in_stock());

        item.reduce_stock(); // Should do nothing
        assert!(item.in_stock());
        assert_eq!(item.stock, None);
    }

    #[test]
    fn test_buy_price_calculation() {
        let mut shop = create_test_shop();
        shop.buy_rate = 1.0;
        assert_eq!(shop.buy_price(100), 100);

        shop.buy_rate = 1.5;
        assert_eq!(shop.buy_price(100), 150);

        shop.buy_rate = 0.8;
        assert_eq!(shop.buy_price(100), 80);
    }

    #[test]
    fn test_sell_price_calculation() {
        let mut shop = create_test_shop();
        shop.sell_rate = 0.5;
        assert_eq!(shop.sell_price(100), 50);

        shop.sell_rate = 0.25;
        assert_eq!(shop.sell_price(100), 25);
    }

    #[test]
    fn test_shop_state_creation() {
        let shop = create_test_shop();
        let state = ShopState::new(shop);

        assert_eq!(state.mode, ShopMode::MainMenu);
        assert_eq!(state.selected_index, 0);
        assert!(state.message.is_none());
    }

    #[test]
    fn test_buy_item_success() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 200;

        let result = shop_state.buy_item(0, 1, &mut game_state);

        assert!(matches!(result, ShopAction::Message(_)));
        assert_eq!(game_state.gold, 150); // 200 - 50
        assert!(game_state.inventory.has("potion", 1));
    }

    #[test]
    fn test_buy_item_insufficient_gold() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 30; // Not enough for 50 gold potion

        let result = shop_state.buy_item(0, 1, &mut game_state);

        assert!(matches!(result, ShopAction::Error(_)));
        assert_eq!(game_state.gold, 30); // Unchanged
    }

    #[test]
    fn test_buy_multiple_items() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 300;

        let result = shop_state.buy_item(0, 3, &mut game_state);

        assert!(matches!(result, ShopAction::Message(_)));
        assert_eq!(game_state.gold, 150); // 300 - (50 * 3)
        assert_eq!(game_state.inventory.count("potion"), 6); // 3 starting + 3 bought
    }

    #[test]
    fn test_buy_limited_stock() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 1000;

        // Buy all 3 rare items
        let result = shop_state.buy_item(2, 3, &mut game_state);
        assert!(matches!(result, ShopAction::Message(_)));

        // Try to buy more - should fail
        let result = shop_state.buy_item(2, 1, &mut game_state);
        assert!(matches!(result, ShopAction::Error(_)));
    }

    #[test]
    fn test_sell_item_success() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 100;
        game_state.inventory.add("test_potion", 1);

        // Initialize sellable items
        shop_state.init_sellable_items(&game_state);

        // Find the potion in sellable items
        let potion_idx = shop_state.sellable_items.iter()
            .position(|(_, id, _)| id == "test_potion")
            .unwrap();

        let result = shop_state.sell_item(potion_idx, 1, &mut game_state);

        assert!(matches!(result, ShopAction::Message(_)));
        // Sell price at 50% of base 100 = 50 gold
        assert_eq!(game_state.gold, 150);
        assert!(!game_state.inventory.has("test_potion", 1));
    }

    #[test]
    fn test_inn_stay_success() {
        let shop = Shop::new("inn", "Test Inn", ShopType::Inn { price: 30, heal_full: true });
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 100;

        // Damage the party
        if let Some(member) = game_state.party.members.first_mut() {
            member.hp = member.hp_max / 2;
        }

        let result = shop_state.stay_at_inn(&mut game_state);

        assert!(matches!(result, ShopAction::Healed));
        assert_eq!(game_state.gold, 70); // 100 - 30

        // Check party is healed
        if let Some(member) = game_state.party.members.first() {
            assert_eq!(member.hp, member.hp_max);
        }
    }

    #[test]
    fn test_inn_partial_heal() {
        let shop = Shop::new("cheap_inn", "Cheap Inn", ShopType::Inn { price: 10, heal_full: false });
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 50;

        // Set up party member with low HP
        if let Some(member) = game_state.party.members.first_mut() {
            member.hp = 10;
            member.hp_max = 100;
        }

        let result = shop_state.stay_at_inn(&mut game_state);

        assert!(matches!(result, ShopAction::Healed));
        // Check partial heal (10 + 50 = 60)
        if let Some(member) = game_state.party.members.first() {
            assert_eq!(member.hp, 60);
        }
    }

    #[test]
    fn test_inn_insufficient_gold() {
        let shop = Shop::new("inn", "Test Inn", ShopType::Inn { price: 100, heal_full: true });
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();
        game_state.gold = 50;

        let result = shop_state.stay_at_inn(&mut game_state);

        assert!(matches!(result, ShopAction::Error(_)));
        assert_eq!(game_state.gold, 50); // Unchanged
    }

    #[test]
    fn test_handle_main_menu_buy() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);

        let result = shop_state.handle_main_menu('b');
        assert!(matches!(result, ShopAction::Continue));
        assert_eq!(shop_state.mode, ShopMode::Buying);
    }

    #[test]
    fn test_handle_main_menu_exit() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);

        let result = shop_state.handle_main_menu('l');
        assert!(matches!(result, ShopAction::Exit));
    }

    #[test]
    fn test_available_items() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);

        let available = shop_state.available_items();
        assert_eq!(available.len(), 3);

        // Buy all limited stock
        let mut game_state = create_test_state();
        game_state.gold = 1000;
        shop_state.buy_item(2, 3, &mut game_state);

        let available = shop_state.available_items();
        assert_eq!(available.len(), 2); // Rare item out of stock
    }

    #[test]
    fn test_init_sellable_items() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);
        let mut game_state = create_test_state();

        // Clear default inventory and add test items
        game_state.inventory = super::super::state::Inventory::new();
        game_state.inventory.add("potion", 2);
        game_state.inventory.add("sword", 1);

        shop_state.init_sellable_items(&game_state);

        // Should have 2 unique item types
        assert_eq!(shop_state.sellable_items.len(), 2);
    }

    #[test]
    fn test_shop_type_display_name() {
        assert_eq!(ShopType::Items.display_name(), "Item Shop");
        assert_eq!(ShopType::Weapons.display_name(), "Weapon Shop");
        assert_eq!(ShopType::Inn { price: 30, heal_full: true }.display_name(), "Inn");
    }

    #[test]
    fn test_mode_name() {
        let shop = create_test_shop();
        let mut shop_state = ShopState::new(shop);

        assert_eq!(shop_state.mode_name(), "Shop Menu");
        shop_state.mode = ShopMode::Buying;
        assert_eq!(shop_state.mode_name(), "Buy Items");
        shop_state.mode = ShopMode::Selling;
        assert_eq!(shop_state.mode_name(), "Sell Items");
    }

    #[test]
    fn test_predefined_shops() {
        let item_shop = create_village_item_shop();
        assert!(!item_shop.inventory.is_empty());

        let weapon_shop = create_village_weapon_shop();
        assert_eq!(weapon_shop.shop_type, ShopType::Weapons);

        let inn = create_village_inn();
        assert!(matches!(inn.shop_type, ShopType::Inn { .. }));

        let merchant = create_traveling_merchant();
        assert_eq!(merchant.buy_rate, 0.8); // Discount
    }

    #[test]
    fn test_get_item_base_price() {
        assert_eq!(get_item_base_price("potion"), 50);
        assert_eq!(get_item_base_price("iron_sword"), 200);
        assert_eq!(get_item_base_price("unknown_item"), 100); // Default
    }
}
