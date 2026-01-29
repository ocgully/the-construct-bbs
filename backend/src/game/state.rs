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
    pub inventory: HashMap<String, u32>,    // commodity_key -> quantity
    pub coat_tier: u32,                     // 0-3 (capacity: 100, 125, 150, 250)
    pub weapons: WeaponSlots,
    pub gang_relations: HashMap<String, i32>, // gang_key -> relation (-100 to 100)
    pub quest_state: QuestProgress,
    pub stats: GameStats,
    pub addiction: HashMap<String, u32>,    // commodity_key -> addiction level
    pub game_over: bool,
    pub game_over_reason: Option<String>,
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
            inventory: HashMap::new(),
            coat_tier: 0,  // 100 units capacity
            weapons: WeaponSlots::default(),
            gang_relations,
            quest_state: QuestProgress::default(),
            stats: GameStats::default(),
            addiction: HashMap::new(),
            game_over: false,
            game_over_reason: None,
        }
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

    /// Get current inventory count
    pub fn inventory_count(&self) -> u32 {
        self.inventory.values().sum()
    }

    /// Calculate net worth (cash + bank + stash + inventory value - debt)
    pub fn net_worth(&self, prices: &HashMap<String, i64>) -> i64 {
        let inventory_value: i64 = self.inventory.iter()
            .map(|(k, &v)| prices.get(k).unwrap_or(&0) * (v as i64))
            .sum();
        self.cash + self.bank_balance + self.mattress_stash + inventory_value - self.debt
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
}
