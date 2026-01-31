//! Star Trader - Game State
//!
//! Persistent player state that gets serialized to the database.

use serde::{Serialize, Deserialize};
use super::data::{Commodity, config};

/// Main player game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // Player identity
    pub player_id: i64,
    pub handle: String,

    // Ship state
    pub ship_class: String,           // Ship class key
    pub ship_name: String,            // Custom ship name
    pub fighters: u32,                // Current fighters
    pub shields: u32,                 // Current shields
    pub cargo: CargoHold,             // What we're carrying

    // Location
    pub sector: u32,                  // Current sector (1-based)
    pub docked: bool,                 // At a port/stardock

    // Resources
    pub credits: i64,                 // Money
    pub turns_remaining: u32,         // Daily turns left
    pub last_turn_date: String,       // "YYYY-MM-DD"

    // Experience and rank
    pub experience: i64,
    pub alignment: i32,               // -1000 to +1000 (evil to good)
    pub kills: u32,                   // Total kills
    pub deaths: u32,                  // Times died

    // Federation status
    pub federation_commission: bool,  // Can buy Imperial ships
    pub federation_rank: FederationRank,

    // Corporation
    pub corporation_id: Option<i64>,  // Corp membership

    // Planets owned (sector IDs)
    pub owned_planets: Vec<u32>,

    // Statistics
    pub stats: PlayerStats,

    // Explored sectors (for map)
    pub explored_sectors: Vec<u32>,

    // Game flags
    pub tutorial_complete: bool,
    pub game_over: bool,
    pub game_over_reason: Option<String>,

    // Message to display
    #[serde(default)]
    pub last_message: Option<String>,
}

/// Cargo hold contents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CargoHold {
    pub fuel_ore: u32,
    pub organics: u32,
    pub equipment: u32,
    pub colonists: u32,  // For planet colonization
}

impl CargoHold {
    pub fn total(&self) -> u32 {
        self.fuel_ore + self.organics + self.equipment + self.colonists
    }

    pub fn get(&self, commodity: Commodity) -> u32 {
        match commodity {
            Commodity::FuelOre => self.fuel_ore,
            Commodity::Organics => self.organics,
            Commodity::Equipment => self.equipment,
        }
    }

    pub fn set(&mut self, commodity: Commodity, amount: u32) {
        match commodity {
            Commodity::FuelOre => self.fuel_ore = amount,
            Commodity::Organics => self.organics = amount,
            Commodity::Equipment => self.equipment = amount,
        }
    }

    pub fn add(&mut self, commodity: Commodity, amount: u32) {
        match commodity {
            Commodity::FuelOre => self.fuel_ore += amount,
            Commodity::Organics => self.organics += amount,
            Commodity::Equipment => self.equipment += amount,
        }
    }

    pub fn remove(&mut self, commodity: Commodity, amount: u32) -> bool {
        let current = self.get(commodity);
        if current >= amount {
            self.set(commodity, current - amount);
            true
        } else {
            false
        }
    }
}

/// Federation rank (alignment-based)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FederationRank {
    Civilian,
    Ensign,
    Lieutenant,
    Commander,
    Captain,
    Admiral,
}

impl FederationRank {
    pub fn name(&self) -> &'static str {
        match self {
            FederationRank::Civilian => "Civilian",
            FederationRank::Ensign => "Ensign",
            FederationRank::Lieutenant => "Lieutenant",
            FederationRank::Commander => "Commander",
            FederationRank::Captain => "Captain",
            FederationRank::Admiral => "Admiral",
        }
    }

    pub fn from_experience(xp: i64) -> FederationRank {
        match xp {
            x if x >= 100_000 => FederationRank::Admiral,
            x if x >= 50_000 => FederationRank::Captain,
            x if x >= 20_000 => FederationRank::Commander,
            x if x >= 5_000 => FederationRank::Lieutenant,
            x if x >= 1_000 => FederationRank::Ensign,
            _ => FederationRank::Civilian,
        }
    }
}

/// Player statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStats {
    pub sectors_explored: u32,
    pub trades_completed: u32,
    pub total_traded_value: i64,
    pub ferrengi_destroyed: u32,
    pub players_destroyed: u32,
    pub times_destroyed: u32,
    pub planets_colonized: u32,
    pub corporations_founded: u32,
    pub max_credits_held: i64,
}

impl GameState {
    /// Create new game state for a player
    pub fn new(player_id: i64, handle: String) -> Self {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        Self {
            player_id,
            handle,
            ship_class: "merchant_cruiser".to_string(),
            ship_name: "Unnamed Vessel".to_string(),
            fighters: 20,
            shields: 50,
            cargo: CargoHold::default(),
            sector: 1,  // Start at StarDock
            docked: true,
            credits: config::STARTING_CREDITS,
            turns_remaining: config::STARTING_TURNS,
            last_turn_date: today,
            experience: 0,
            alignment: 0,
            kills: 0,
            deaths: 0,
            federation_commission: false,
            federation_rank: FederationRank::Civilian,
            corporation_id: None,
            owned_planets: Vec::new(),
            stats: PlayerStats::default(),
            explored_sectors: vec![1],  // Start with sector 1 explored
            tutorial_complete: false,
            game_over: false,
            game_over_reason: None,
            last_message: None,
        }
    }

    /// Get current ship class data
    pub fn ship(&self) -> Option<&'static super::data::ShipClass> {
        super::data::get_ship_class(&self.ship_class)
    }

    /// Get cargo capacity
    pub fn cargo_capacity(&self) -> u32 {
        self.ship().map(|s| s.cargo_holds).unwrap_or(20)
    }

    /// Get available cargo space
    pub fn cargo_space(&self) -> u32 {
        self.cargo_capacity().saturating_sub(self.cargo.total())
    }

    /// Check if it's a new day and reset turns
    pub fn check_new_day(&mut self) -> bool {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        if self.last_turn_date != today {
            self.last_turn_date = today;
            self.turns_remaining = config::DAILY_TURNS;
            true
        } else {
            false
        }
    }

    /// Use turns (returns false if not enough)
    pub fn use_turns(&mut self, amount: u32) -> bool {
        if self.turns_remaining >= amount {
            self.turns_remaining -= amount;
            true
        } else {
            false
        }
    }

    /// Add experience and update rank
    pub fn add_experience(&mut self, xp: i64) {
        self.experience += xp;
        self.federation_rank = FederationRank::from_experience(self.experience);
    }

    /// Adjust alignment (clamped to -1000 to +1000)
    pub fn adjust_alignment(&mut self, amount: i32) {
        self.alignment = (self.alignment + amount).clamp(-1000, 1000);
    }

    /// Mark sector as explored
    pub fn explore_sector(&mut self, sector: u32) {
        if !self.explored_sectors.contains(&sector) {
            self.explored_sectors.push(sector);
            self.stats.sectors_explored += 1;
        }
    }

    /// Update max credits stat if current is higher
    pub fn update_max_credits(&mut self) {
        if self.credits > self.stats.max_credits_held {
            self.stats.max_credits_held = self.credits;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = GameState::new(1, "TestPlayer".to_string());
        assert_eq!(state.sector, 1);
        assert_eq!(state.credits, config::STARTING_CREDITS);
        assert_eq!(state.turns_remaining, config::STARTING_TURNS);
        assert!(state.docked);
    }

    #[test]
    fn test_cargo_operations() {
        let mut cargo = CargoHold::default();
        cargo.add(Commodity::FuelOre, 10);
        assert_eq!(cargo.get(Commodity::FuelOre), 10);
        assert_eq!(cargo.total(), 10);

        assert!(cargo.remove(Commodity::FuelOre, 5));
        assert_eq!(cargo.get(Commodity::FuelOre), 5);

        assert!(!cargo.remove(Commodity::FuelOre, 10));  // Not enough
        assert_eq!(cargo.get(Commodity::FuelOre), 5);    // Unchanged
    }

    #[test]
    fn test_use_turns() {
        let mut state = GameState::new(1, "Test".to_string());
        state.turns_remaining = 10;

        assert!(state.use_turns(5));
        assert_eq!(state.turns_remaining, 5);

        assert!(!state.use_turns(10));  // Not enough
        assert_eq!(state.turns_remaining, 5);  // Unchanged
    }

    #[test]
    fn test_federation_rank() {
        assert_eq!(FederationRank::from_experience(0), FederationRank::Civilian);
        assert_eq!(FederationRank::from_experience(1000), FederationRank::Ensign);
        assert_eq!(FederationRank::from_experience(100_000), FederationRank::Admiral);
    }

    #[test]
    fn test_experience_and_rank() {
        let mut state = GameState::new(1, "Test".to_string());
        assert_eq!(state.federation_rank, FederationRank::Civilian);

        state.add_experience(1000);
        assert_eq!(state.federation_rank, FederationRank::Ensign);
    }

    #[test]
    fn test_alignment() {
        let mut state = GameState::new(1, "Test".to_string());

        state.adjust_alignment(500);
        assert_eq!(state.alignment, 500);

        state.adjust_alignment(1000);
        assert_eq!(state.alignment, 1000);  // Capped at 1000

        state.adjust_alignment(-3000);
        assert_eq!(state.alignment, -1000);  // Capped at -1000
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new(1, "TestPlayer".to_string());
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.handle, "TestPlayer");
        assert_eq!(restored.credits, state.credits);
    }
}
