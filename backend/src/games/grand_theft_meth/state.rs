use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub day: u32,                           // 1-90
    pub actions_remaining: u32,             // 5 per day base
    pub location: String,                   // Current borough key
    pub city: String,                       // Current city key
    pub cash: i64,                          // In cents to avoid float
    pub bank_balance: i64,                  // Unlocks at $50,000 threshold
    pub bank_unlocked: bool,
    pub mattress_stash: i64,                // Pre-bank storage (no interest)
    pub debt: i64,                          // Loan shark debt (10% daily)
    pub health: u32,                        // Current HP
    pub max_health: u32,                    // Base 100, can increase
    pub notoriety: u32,                     // Heat level 0-100
    /// Inventory with purchase price tracking - each commodity has lots bought at different prices
    #[serde(default)]
    pub inventory_lots: HashMap<String, Vec<InventoryLot>>,
    pub coat_tier: u32,                     // 0-3 (capacity: 100, 125, 150, 250)
    pub weapons: WeaponSlots,
    pub gang_relations: HashMap<String, i32>, // gang_key -> relation (-100 to 100)
    pub quest_state: QuestProgress,
    pub stats: GameStats,
    pub addiction: HashMap<String, u32>,    // commodity_key -> addiction level
    #[serde(default)]
    pub high_tier: u8,                      // 0=sober, 1-3=high (affects screen clarity)
    #[serde(default)]
    pub last_login_date: Option<String>,    // Real-world date "YYYY-MM-DD" - used to clear high on new day
    pub game_over: bool,
    pub game_over_reason: Option<String>,
    /// Local market supply modifier - selling lots here makes prices drop
    /// Key: "city/borough/commodity", Value: supply modifier (negative = oversupplied)
    #[serde(default)]
    pub market_supply: HashMap<String, i32>,
    /// Pending transaction that can be cancelled (costs 1 action)
    #[serde(default)]
    pub pending_transaction: Option<PendingTransaction>,
    /// Message to display on next screen (combat results, etc.)
    #[serde(default)]
    pub last_message: Option<String>,
}

/// A lot of inventory purchased at a specific price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLot {
    pub quantity: u32,
    pub purchase_price: i64,  // price per unit when bought
}

/// A transaction that can be cancelled at the cost of 1 action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub commodity: String,
    pub quantity: u32,
    pub total_cost: i64,
    pub is_purchase: bool,  // true = bought, false = sold
    pub purchase_price: i64, // price per unit (for restoring lot on cancel)
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeaponSlots {
    pub melee: Option<String>,              // weapon_key
    pub gun: Option<String>,                // weapon_key
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuestProgress {
    pub story_step: u32,                    // 0-15 (0 = not started)
    pub active_deliveries: Vec<DeliveryQuest>,
    pub completed_deliveries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryQuest {
    pub id: String,
    pub commodity: String,
    pub quantity: u32,
    pub from_location: String,
    pub to_location: String,
    pub reward: i64,
    pub expires_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStats {
    pub total_bought: i64,
    pub total_sold: i64,
    pub total_profit: i64,
    pub police_encounters: u32,
    pub muggings_survived: u32,
    pub people_killed: u32,
    pub hospital_visits: u32,
    pub max_net_worth: i64,
}

impl GameState {
    pub fn new() -> Self {
        let mut gang_relations = HashMap::new();
        gang_relations.insert("triads".to_string(), 0);
        gang_relations.insert("cartel".to_string(), 0);
        gang_relations.insert("mafia".to_string(), 0);

        Self {
            day: 1,
            actions_remaining: 5,
            location: "bronx".to_string(),
            city: "nyc".to_string(),
            cash: 200000,  // $2,000 in cents
            bank_balance: 0,
            bank_unlocked: false,
            mattress_stash: 0,
            debt: 550000,  // $5,500 in cents
            health: 100,
            max_health: 100,
            notoriety: 0,
            inventory_lots: HashMap::new(),
            coat_tier: 0,  // 100 units capacity
            weapons: WeaponSlots::default(),
            gang_relations,
            quest_state: QuestProgress::default(),
            stats: GameStats::default(),
            addiction: HashMap::new(),
            high_tier: 0,
            last_login_date: None,
            game_over: false,
            game_over_reason: None,
            market_supply: HashMap::new(),
            pending_transaction: None,
            last_message: None,
        }
    }

    /// Get total quantity of a commodity across all lots
    pub fn get_quantity(&self, commodity: &str) -> u32 {
        self.inventory_lots
            .get(commodity)
            .map(|lots| lots.iter().map(|l| l.quantity).sum())
            .unwrap_or(0)
    }

    /// Get coat capacity based on tier
    pub fn coat_capacity(&self) -> u32 {
        match self.coat_tier {
            0 => 100,
            1 => 125,
            2 => 150,
            _ => 250,
        }
    }

    /// Get current inventory count across all commodities
    pub fn inventory_count(&self) -> u32 {
        self.inventory_lots
            .values()
            .flat_map(|lots| lots.iter())
            .map(|l| l.quantity)
            .sum()
    }

    /// Add inventory at a specific purchase price
    pub fn add_inventory(&mut self, commodity: &str, quantity: u32, purchase_price: i64) {
        let lots = self.inventory_lots.entry(commodity.to_string()).or_insert_with(Vec::new);
        // Check if we can merge with existing lot at same price
        if let Some(lot) = lots.iter_mut().find(|l| l.purchase_price == purchase_price) {
            lot.quantity += quantity;
        } else {
            lots.push(InventoryLot { quantity, purchase_price });
        }
    }

    /// Remove inventory, selling highest-profit lots first (lowest purchase price)
    /// Returns the purchase price of the lot sold from (for profit tracking)
    pub fn remove_inventory(&mut self, commodity: &str, quantity: u32) -> Option<i64> {
        let lots = self.inventory_lots.get_mut(commodity)?;
        if lots.is_empty() {
            return None;
        }

        // Sort by purchase price ascending (sell cheapest-bought first = max profit)
        lots.sort_by_key(|l| l.purchase_price);

        let mut remaining = quantity;
        let mut purchase_price = 0i64;

        for lot in lots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if lot.quantity > 0 {
                let take = lot.quantity.min(remaining);
                purchase_price = lot.purchase_price; // Track what we're selling
                lot.quantity -= take;
                remaining -= take;
            }
        }

        // Remove empty lots
        lots.retain(|l| l.quantity > 0);
        if lots.is_empty() {
            self.inventory_lots.remove(commodity);
        }

        Some(purchase_price)
    }

    /// Get the lowest purchase price for a commodity (best profit margin when selling)
    pub fn get_lowest_cost(&self, commodity: &str) -> Option<i64> {
        self.inventory_lots
            .get(commodity)?
            .iter()
            .filter(|l| l.quantity > 0)
            .map(|l| l.purchase_price)
            .min()
    }

    /// Get average cost basis for a commodity
    #[allow(dead_code)]
    pub fn get_average_cost(&self, commodity: &str) -> Option<i64> {
        let lots = self.inventory_lots.get(commodity)?;
        let total_qty: u32 = lots.iter().map(|l| l.quantity).sum();
        if total_qty == 0 {
            return None;
        }
        let total_cost: i64 = lots.iter().map(|l| l.purchase_price * l.quantity as i64).sum();
        Some(total_cost / total_qty as i64)
    }

    /// Calculate net worth (cash + bank + stash + inventory value - debt)
    pub fn net_worth(&self, prices: &HashMap<String, i64>) -> i64 {
        let inventory_value: i64 = self.inventory_lots.iter()
            .map(|(k, lots)| {
                let qty: u32 = lots.iter().map(|l| l.quantity).sum();
                prices.get(k).unwrap_or(&0) * (qty as i64)
            })
            .sum();
        self.cash + self.bank_balance + self.mattress_stash + inventory_value - self.debt
    }

    /// Get market supply modifier for a commodity at current location
    /// Negative = oversupplied (cheaper to buy), Positive = undersupplied (more expensive)
    #[allow(dead_code)]
    pub fn get_supply_modifier(&self, commodity: &str) -> i32 {
        let key = format!("{}/{}/{}", self.city, self.location, commodity);
        self.market_supply.get(&key).copied().unwrap_or(0)
    }

    /// Adjust market supply when buying/selling
    /// Buying reduces local supply (prices go up), Selling increases supply (prices go down)
    pub fn adjust_supply(&mut self, commodity: &str, sold_quantity: i32) {
        let key = format!("{}/{}/{}", self.city, self.location, commodity);
        let current = self.market_supply.get(&key).copied().unwrap_or(0);
        // Each unit sold increases supply by 1, max +/- 50
        let new_supply = (current + sold_quantity).clamp(-50, 50);
        self.market_supply.insert(key, new_supply);
    }

    /// Clear pending transaction (used when moving to new screen or confirming)
    pub fn clear_pending_transaction(&mut self) {
        self.pending_transaction = None;
    }

    /// Apply daily interest to debt (10% = 1000 basis points)
    pub fn apply_debt_interest(&mut self) {
        // (debt * 11000) / 10000 = debt * 1.10
        self.debt = (self.debt * 11000) / 10000;
    }

    /// Apply daily interest to bank (5% = 500 basis points)
    pub fn apply_bank_interest(&mut self) {
        if self.bank_unlocked {
            // (balance * 10500) / 10000 = balance * 1.05
            self.bank_balance = (self.bank_balance * 10500) / 10000;
        }
    }

    /// Decay notoriety by 10% (laying low)
    pub fn decay_notoriety(&mut self) {
        self.notoriety = (self.notoriety * 90) / 100;
    }

    /// Check if this is a new real-world day since last login.
    /// If so, clear the high (player sobered up overnight) and update the date.
    /// Returns true if high was cleared.
    pub fn check_new_day_sober_up(&mut self) -> bool {
        use chrono::Local;

        let today = Local::now().format("%Y-%m-%d").to_string();

        match &self.last_login_date {
            Some(last_date) if last_date == &today => {
                // Same day, no change
                false
            }
            _ => {
                // New day (or first login) - sober up!
                let was_high = self.high_tier > 0;
                self.high_tier = 0;
                self.last_login_date = Some(today);
                was_high
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_initial_state() {
        let state = GameState::new();
        assert_eq!(state.day, 1);
        assert_eq!(state.cash, 200000); // $2,000
        assert_eq!(state.debt, 550000); // $5,500
        assert_eq!(state.health, 100);
        assert_eq!(state.actions_remaining, 5);
        assert_eq!(state.location, "bronx");
        assert_eq!(state.city, "nyc");
    }

    #[test]
    fn test_inventory_operations() {
        let mut state = GameState::new();
        state.add_inventory("weed", 10, 1500);
        assert_eq!(state.get_quantity("weed"), 10);

        state.remove_inventory("weed", 5);
        assert_eq!(state.get_quantity("weed"), 5);
    }

    #[test]
    fn test_coat_capacity() {
        let mut state = GameState::new();
        assert_eq!(state.coat_capacity(), 100);
        state.coat_tier = 1;
        assert_eq!(state.coat_capacity(), 125);
        state.coat_tier = 3;
        assert_eq!(state.coat_capacity(), 250);
    }

    #[test]
    fn test_debt_interest() {
        let mut state = GameState::new();
        state.debt = 100000; // $1,000
        state.apply_debt_interest();
        assert_eq!(state.debt, 110000); // $1,100 (10% interest)
    }

    #[test]
    fn test_bank_interest() {
        let mut state = GameState::new();
        state.bank_unlocked = true;
        state.bank_balance = 100000; // $1,000
        state.apply_bank_interest();
        assert_eq!(state.bank_balance, 105000); // $1,050 (5% interest)
    }

    #[test]
    fn test_net_worth_calculation() {
        let mut state = GameState::new();
        state.cash = 100000;
        state.bank_balance = 50000;
        state.debt = 30000;
        let prices = std::collections::HashMap::new();
        assert_eq!(state.net_worth(&prices), 120000); // 100k + 50k - 30k
    }

    #[test]
    fn test_high_tier_sober_up() {
        let mut state = GameState::new();
        state.high_tier = 3;
        state.last_login_date = Some("2020-01-01".to_string());
        let sobered = state.check_new_day_sober_up();
        assert!(sobered);
        assert_eq!(state.high_tier, 0);
    }
}
