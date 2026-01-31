//! Game state for Master of Cygnus

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::galaxy::{Galaxy, PlanetType};
use super::tech::{ResearchProgress, TechField, TechTree};
use super::ships::{Fleet, ShipDesign};
use super::ai::AiController;
use super::data::GalaxySettings;

/// Main game state containing all game data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: u64,
    pub name: String,
    pub galaxy: Galaxy,
    pub empires: Vec<EmpireState>,
    pub turn_number: u32,
    pub status: GameStatus,
    /// Deadline for current turn submissions
    pub turn_deadline: Option<DateTime<Utc>>,
    /// Turn orders submitted by each empire
    pub pending_orders: HashMap<u32, TurnOrders>,
    /// Combat results from this turn
    pub battle_reports: Vec<super::combat::BattleResult>,
    /// Tech tree (shared by all)
    pub tech_tree: TechTree,
    /// Victory conditions
    pub victory_type: Option<VictoryType>,
    pub winner_empire_id: Option<u32>,
    /// Settings
    pub settings: GameSettings,
    /// Message to display on next render
    pub last_message: Option<String>,
}

/// Game status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    /// Waiting for players to join
    WaitingForPlayers,
    /// Game in progress
    InProgress,
    /// Game completed
    Completed,
}

/// Victory conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VictoryType {
    /// Conquer all other empires
    Conquest,
    /// Win council vote
    Council,
    /// Research all technologies
    Technology,
    /// All other players forfeited
    LastHumanStanding,
}

/// Game settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub galaxy_size: String, // "small", "medium", "large"
    pub max_players: u32,
    pub turn_timeout_hours: u32,
    pub max_timeouts_before_forfeit: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            galaxy_size: "medium".to_string(),
            max_players: 4,
            turn_timeout_hours: 72,
            max_timeouts_before_forfeit: 3,
        }
    }
}

/// State of a player's empire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpireState {
    pub id: u32,
    pub user_id: Option<i64>, // None if AI-controlled
    pub name: String,
    pub race_key: String,
    pub color: String,
    /// Is this empire controlled by AI?
    pub is_ai: bool,
    /// AI controller if AI-controlled
    pub ai_controller: Option<AiController>,
    /// Number of turn timeouts
    pub timeout_count: u32,
    /// Turn when last orders were submitted
    pub last_active_turn: u32,
    /// Has player forfeited?
    pub forfeited: bool,
    /// Research progress
    pub research: ResearchProgress,
    /// Ship designs
    pub ship_designs: Vec<ShipDesign>,
    /// Fleets
    pub fleets: Vec<Fleet>,
    /// Colonies
    pub colonies: Vec<ColonyState>,
    /// Treasury (production credits)
    pub treasury: i64,
    /// Known stars (explored)
    pub known_stars: Vec<u32>,
    /// Diplomatic relations: empire_id -> relation (-100 to 100)
    pub relations: HashMap<u32, i32>,
}

impl EmpireState {
    /// Create a new empire for a player
    pub fn new(id: u32, user_id: i64, name: String, race_key: String, color: String) -> Self {
        EmpireState {
            id,
            user_id: Some(user_id),
            name,
            race_key,
            color,
            is_ai: false,
            ai_controller: None,
            timeout_count: 0,
            last_active_turn: 0,
            forfeited: false,
            research: ResearchProgress::new(),
            ship_designs: ShipDesign::default_designs(),
            fleets: Vec::new(),
            colonies: Vec::new(),
            treasury: 0,
            known_stars: Vec::new(),
            relations: HashMap::new(),
        }
    }

    /// Create an AI empire
    pub fn new_ai(id: u32, name: String, race_key: String, color: String) -> Self {
        let mut empire = Self::new(id, 0, name, race_key, color);
        empire.user_id = None;
        empire.is_ai = true;
        empire.ai_controller = Some(AiController::new(id));
        empire
    }

    /// Convert to AI control (for timeout/forfeit)
    pub fn convert_to_ai(&mut self, reason: &str) {
        self.is_ai = true;
        self.ai_controller = Some(AiController::new(self.id));
        self.user_id = None;
        // Keep forfeited flag to distinguish from original AI
        if reason.contains("forfeit") {
            self.forfeited = true;
        }
    }

    /// Get total research output per turn
    pub fn research_output(&self) -> u32 {
        let mut output = 0;
        for colony in &self.colonies {
            output += colony.research_output();
        }
        output
    }

    /// Get total production output per turn
    pub fn production_output(&self) -> u32 {
        let mut output = 0;
        for colony in &self.colonies {
            output += colony.production_output();
        }
        output
    }

    /// Get total population
    pub fn total_population(&self) -> u32 {
        self.colonies.iter().map(|c| c.population).sum()
    }

    /// Check if empire can colonize a planet type
    pub fn can_colonize(&self, planet_type: PlanetType) -> bool {
        let planetology_level = self.research.level(TechField::Planetology);

        match planet_type {
            PlanetType::Terran | PlanetType::Ocean | PlanetType::Arid => true,
            PlanetType::Tundra => planetology_level >= 2,
            PlanetType::Barren => planetology_level >= 3,
            PlanetType::Toxic => planetology_level >= 4,
            PlanetType::GasGiant => false, // Never colonizable
        }
    }

    /// Get next available fleet ID
    pub fn next_fleet_id(&self) -> u32 {
        self.fleets.iter().map(|f| f.id).max().unwrap_or(0) + 1
    }
}

/// State of a colony
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonyState {
    pub star_id: u32,
    pub population: u32,
    pub max_population: u32,
    /// Buildings by name
    pub buildings: Vec<String>,
    /// Production queue (building names or ship design names)
    pub production_queue: Vec<String>,
    /// Accumulated production points
    pub accumulated_production: u32,
    /// Has planetary shield?
    pub has_planetary_shield: bool,
}

impl ColonyState {
    /// Create a new colony
    pub fn new(star_id: u32, max_population: u32) -> Self {
        ColonyState {
            star_id,
            population: 10, // Starting population
            max_population,
            buildings: vec!["ColonyBase".to_string()],
            production_queue: Vec::new(),
            accumulated_production: 0,
            has_planetary_shield: false,
        }
    }

    /// Calculate production output
    pub fn production_output(&self) -> u32 {
        let mut output = self.population / 5; // Base: 1 production per 5 pop
        for building in &self.buildings {
            output += match building.as_str() {
                "Factory" => 10,
                "DeepCoreMine" => 25,
                "StarForge" => 50,
                _ => 0,
            };
        }
        output
    }

    /// Calculate research output
    pub fn research_output(&self) -> u32 {
        let mut output = self.population / 10; // Base: 1 research per 10 pop
        for building in &self.buildings {
            if building == "ResearchLab" {
                output += 5;
            }
        }
        output
    }

    /// Calculate population growth for this turn
    pub fn population_growth(&self) -> u32 {
        if self.population >= self.max_population {
            return 0;
        }

        // Base growth: 2% of population
        let base_growth = (self.population as f64 * 0.02).ceil() as u32;

        // Farms increase growth
        let farm_count = self.buildings.iter().filter(|b| *b == "Farm").count() as u32;
        let farm_bonus = farm_count * 2;

        (base_growth + farm_bonus).min(self.max_population - self.population)
    }

    /// Check if colony has a shipyard
    pub fn has_shipyard(&self) -> bool {
        self.buildings.iter().any(|b| b == "Shipyard" || b == "StarForge")
    }
}

/// Turn orders from a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnOrders {
    pub empire_id: u32,
    pub colony_orders: Vec<ColonyOrders>,
    pub fleet_orders: Vec<FleetOrders>,
    pub research_allocation: HashMap<TechField, u32>,
    pub submitted: bool,
    pub ai_generated: bool,
}

/// Orders for a specific colony
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonyOrders {
    pub star_id: u32,
    pub build_queue: Vec<String>,
    /// Transfer population to another colony: (destination_star_id, amount)
    pub population_transfer: Option<(u32, u32)>,
}

/// Orders for a specific fleet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetOrders {
    pub fleet_id: u32,
    pub destination: Option<u32>,
    /// Attempt to colonize if fleet has colony ship
    pub colonize: bool,
}

impl GameState {
    /// Create a new game
    pub fn new(id: u64, name: String, settings: GameSettings) -> Self {
        let galaxy_settings = match settings.galaxy_size.as_str() {
            "small" => GalaxySettings::small(),
            "large" => GalaxySettings::large(),
            _ => GalaxySettings::medium(),
        };

        let seed = id; // Use game ID as seed for reproducibility
        let galaxy = Galaxy::generate(&galaxy_settings, seed);

        GameState {
            id,
            name,
            galaxy,
            empires: Vec::new(),
            turn_number: 0,
            status: GameStatus::WaitingForPlayers,
            turn_deadline: None,
            pending_orders: HashMap::new(),
            battle_reports: Vec::new(),
            tech_tree: TechTree::new(),
            victory_type: None,
            winner_empire_id: None,
            settings,
            last_message: None,
        }
    }

    /// Add a player to the game
    pub fn add_player(
        &mut self,
        user_id: i64,
        empire_name: String,
        race_key: String,
        color: String,
    ) -> Result<u32, String> {
        if self.status != GameStatus::WaitingForPlayers {
            return Err("Game has already started".to_string());
        }

        if self.empires.len() >= self.settings.max_players as usize {
            return Err("Game is full".to_string());
        }

        let empire_id = self.empires.len() as u32;
        let mut empire = EmpireState::new(empire_id, user_id, empire_name, race_key, color);

        // Assign starting star (find an unclaimed, colonizable star)
        if let Some(start_star) = self.find_starting_star() {
            // Create initial colony
            let colony = ColonyState::new(start_star.id, start_star.planet.max_population);
            empire.colonies.push(colony);
            empire.known_stars.push(start_star.id);

            // Create initial fleet with scout and colony ship
            let mut fleet = Fleet::new(empire.next_fleet_id(), empire_id, "Home Fleet".to_string(), start_star.id);
            fleet.add_ships(0, 2); // 2 scouts
            fleet.add_ships(1, 1); // 1 colony ship
            empire.fleets.push(fleet);

            // Mark star as owned
            if let Some(star) = self.galaxy.get_star_mut(start_star.id) {
                star.owner = Some(empire_id);
            }
        }

        self.empires.push(empire);
        Ok(empire_id)
    }

    /// Find a suitable starting star for a new player
    fn find_starting_star(&self) -> Option<&super::galaxy::Star> {
        // Find a Terran or Ocean planet that's not too close to others
        let owned_stars: Vec<u32> = self.empires.iter()
            .flat_map(|e| e.colonies.iter().map(|c| c.star_id))
            .collect();

        self.galaxy.stars.iter()
            .filter(|s| {
                // Must be unowned
                s.owner.is_none()
                    // Must be colonizable without tech
                    && matches!(s.planet.planet_type, PlanetType::Terran | PlanetType::Ocean | PlanetType::Arid)
                    // Must be far enough from other starting positions
                    && owned_stars.iter().all(|&owned_id| {
                        self.galaxy.distance(s.id, owned_id).unwrap_or(0.0) > 20.0
                    })
            })
            .max_by_key(|s| s.planet.max_population) // Prefer better planets
    }

    /// Start the game
    pub fn start_game(&mut self) {
        if self.empires.len() >= 2 {
            self.status = GameStatus::InProgress;
            self.turn_number = 1;
            self.set_turn_deadline();
        }
    }

    /// Set the deadline for the current turn
    fn set_turn_deadline(&mut self) {
        use chrono::Duration;
        self.turn_deadline = Some(Utc::now() + Duration::hours(self.settings.turn_timeout_hours as i64));
    }

    /// Submit turn orders for an empire
    pub fn submit_orders(&mut self, empire_id: u32, orders: TurnOrders) {
        if let Some(empire) = self.empires.iter_mut().find(|e| e.id == empire_id) {
            empire.last_active_turn = self.turn_number;
        }
        self.pending_orders.insert(empire_id, orders);
    }

    /// Check if all orders are submitted
    pub fn all_orders_submitted(&self) -> bool {
        let active_empires: Vec<_> = self.empires.iter()
            .filter(|e| !e.forfeited && e.colonies.len() > 0)
            .collect();

        active_empires.iter().all(|e| {
            self.pending_orders.contains_key(&e.id)
        })
    }

    /// Check for turn timeout and apply AI for timed-out players
    pub fn check_timeouts(&mut self) {
        let now = Utc::now();
        if let Some(deadline) = self.turn_deadline {
            if now > deadline {
                // First pass: identify timed-out players and update timeout counts
                let mut need_ai_orders: Vec<u32> = Vec::new();
                let max_timeouts = self.settings.max_timeouts_before_forfeit;

                for empire in &mut self.empires {
                    if !empire.is_ai && !self.pending_orders.contains_key(&empire.id) {
                        empire.timeout_count += 1;

                        // Convert to AI after max timeouts
                        if empire.timeout_count >= max_timeouts {
                            empire.convert_to_ai("Exceeded maximum timeouts");
                        } else {
                            // Mark for AI order generation
                            need_ai_orders.push(empire.id);
                        }
                    }
                }

                // Second pass: generate AI orders for those who need them
                for empire_id in need_ai_orders {
                    let mut ai = AiController::new(empire_id);
                    // Clone data needed for AI decision making
                    let galaxy_clone = self.galaxy.clone();
                    let empires_clone = self.empires.clone();

                    if let Some(empire) = self.empires.iter().find(|e| e.id == empire_id) {
                        let orders = ai.generate_orders(empire, &galaxy_clone, &empires_clone);
                        self.pending_orders.insert(empire_id, orders);
                    }
                }
            }
        }
    }

    /// Process end of turn
    pub fn process_turn(&mut self) {
        // First, generate AI orders for AI empires
        let ai_empire_ids: Vec<u32> = self.empires.iter()
            .filter(|e| e.is_ai && !self.pending_orders.contains_key(&e.id))
            .map(|e| e.id)
            .collect();

        // Clone data needed for AI decision making (avoids borrow conflicts)
        let galaxy_clone = self.galaxy.clone();
        let empires_clone = self.empires.clone();

        for empire_id in ai_empire_ids {
            if let Some(empire) = empires_clone.iter().find(|e| e.id == empire_id) {
                if let Some(ref ai) = empire.ai_controller {
                    let mut ai_clone = ai.clone();
                    let orders = ai_clone.generate_orders(empire, &galaxy_clone, &empires_clone);
                    self.pending_orders.insert(empire_id, orders);
                }
            }
        }

        // Process orders
        self.process_colony_orders();
        self.process_fleet_orders();
        self.process_research();

        // Process combat
        self.resolve_combats();

        // Population growth
        for empire in &mut self.empires {
            for colony in &mut empire.colonies {
                let growth = colony.population_growth();
                colony.population += growth;
            }
        }

        // Check victory conditions
        self.check_victory();

        // Advance turn
        self.turn_number += 1;
        self.pending_orders.clear();
        self.battle_reports.clear();
        self.set_turn_deadline();
    }

    /// Process colony build orders
    fn process_colony_orders(&mut self) {
        let orders = self.pending_orders.clone();

        for empire in &mut self.empires {
            if let Some(empire_orders) = orders.get(&empire.id) {
                for col_order in &empire_orders.colony_orders {
                    if let Some(colony) = empire.colonies.iter_mut().find(|c| c.star_id == col_order.star_id) {
                        colony.production_queue = col_order.build_queue.clone();

                        // Process production
                        let production = colony.production_output();
                        colony.accumulated_production += production;

                        // Try to complete items in queue
                        while !colony.production_queue.is_empty() {
                            let item = &colony.production_queue[0];
                            let cost = Self::get_build_cost(item);

                            if colony.accumulated_production >= cost {
                                colony.accumulated_production -= cost;

                                // Add building or ship
                                if item.contains("Ship") || item == "Scout" || item == "Fighter" {
                                    // Add ship to fleet at this location
                                    // (simplified: just add to first fleet or create new)
                                } else {
                                    colony.buildings.push(item.clone());
                                    if item == "PlanetaryShield" {
                                        colony.has_planetary_shield = true;
                                    }
                                }

                                colony.production_queue.remove(0);
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get cost to build something
    fn get_build_cost(item: &str) -> u32 {
        match item {
            "Factory" => 100,
            "ResearchLab" => 150,
            "Farm" => 80,
            "Shipyard" => 200,
            "DeepCoreMine" => 400,
            "PlanetaryShield" => 500,
            "Scout" => 20,
            "Fighter" => 70,
            "Colony Ship" => 100,
            _ => 100,
        }
    }

    /// Process fleet movement orders
    fn process_fleet_orders(&mut self) {
        let orders = self.pending_orders.clone();

        for empire in &mut self.empires {
            if let Some(empire_orders) = orders.get(&empire.id) {
                for fleet_order in &empire_orders.fleet_orders {
                    if let Some(fleet) = empire.fleets.iter_mut().find(|f| f.id == fleet_order.fleet_id) {
                        if let Some(dest) = fleet_order.destination {
                            // Calculate travel time based on distance
                            if let Some(distance) = self.galaxy.distance(fleet.location_star_id, dest) {
                                let speed = fleet.fleet_speed(&empire.ship_designs);
                                let eta = ((distance / (speed as f64 * 5.0)).ceil() as u32).max(1);
                                fleet.set_destination(dest, eta);
                            }
                        }
                    }
                }
            }

            // Advance all fleets
            for fleet in &mut empire.fleets {
                fleet.advance_turn();
            }
        }
    }

    /// Process research
    fn process_research(&mut self) {
        let orders = self.pending_orders.clone();
        let tech_tree = self.tech_tree.clone();

        for empire in &mut self.empires {
            // Apply research allocation from orders
            if let Some(empire_orders) = orders.get(&empire.id) {
                empire.research.allocation = empire_orders.research_allocation.clone();
            }

            let research_points = empire.research_output();
            let _breakthroughs = empire.research.add_research(research_points, &tech_tree);

            // TODO: Notify player of breakthroughs
        }
    }

    /// Resolve combat between fleets at same location
    fn resolve_combats(&mut self) {
        // Find stars with multiple empire fleets
        let mut combat_locations: HashMap<u32, Vec<(u32, u32)>> = HashMap::new(); // star_id -> [(empire_id, fleet_id)]

        for empire in &self.empires {
            for fleet in &empire.fleets {
                if !fleet.is_in_transit() {
                    combat_locations.entry(fleet.location_star_id)
                        .or_insert_with(Vec::new)
                        .push((empire.id, fleet.id));
                }
            }
        }

        // TODO: Implement actual combat resolution
        // For now, just detect potential combats
        for (star_id, fleets) in combat_locations {
            let empire_ids: Vec<u32> = fleets.iter().map(|(e, _)| *e).collect();
            let unique_empires: std::collections::HashSet<_> = empire_ids.iter().collect();
            if unique_empires.len() > 1 {
                // Combat would occur here
                self.last_message = Some(format!("Combat detected at star {}!", star_id));
            }
        }
    }

    /// Check for victory conditions
    fn check_victory(&mut self) {
        let active_empires: Vec<_> = self.empires.iter()
            .filter(|e| !e.forfeited && e.colonies.len() > 0)
            .collect();

        // Check for conquest victory (only one empire left)
        if active_empires.len() == 1 {
            self.victory_type = Some(VictoryType::Conquest);
            self.winner_empire_id = Some(active_empires[0].id);
            self.status = GameStatus::Completed;
            return;
        }

        // Check for technology victory
        for empire in &active_empires {
            let all_maxed = TechField::all().iter().all(|&field| {
                empire.research.level(field) >= 6
            });
            if all_maxed {
                self.victory_type = Some(VictoryType::Technology);
                self.winner_empire_id = Some(empire.id);
                self.status = GameStatus::Completed;
                return;
            }
        }

        // Check if no human players remain
        let human_players = active_empires.iter().filter(|e| !e.is_ai).count();
        if human_players == 0 {
            self.victory_type = Some(VictoryType::LastHumanStanding);
            self.status = GameStatus::Completed;
        }
    }

    /// Get empire by ID
    pub fn get_empire(&self, empire_id: u32) -> Option<&EmpireState> {
        self.empires.iter().find(|e| e.id == empire_id)
    }

    /// Get mutable empire by ID
    pub fn get_empire_mut(&mut self, empire_id: u32) -> Option<&mut EmpireState> {
        self.empires.iter_mut().find(|e| e.id == empire_id)
    }

    /// Get empire by user ID
    pub fn get_empire_by_user(&self, user_id: i64) -> Option<&EmpireState> {
        self.empires.iter().find(|e| e.user_id == Some(user_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = GameState::new(1, "Test Game".to_string(), GameSettings::default());
        assert_eq!(game.status, GameStatus::WaitingForPlayers);
        assert!(!game.galaxy.stars.is_empty());
    }

    #[test]
    fn test_add_player() {
        let mut game = GameState::new(1, "Test".to_string(), GameSettings::default());
        let result = game.add_player(1, "Empire 1".to_string(), "terran".to_string(), "LightCyan".to_string());
        assert!(result.is_ok());
        assert_eq!(game.empires.len(), 1);
    }

    #[test]
    fn test_start_game() {
        let mut game = GameState::new(1, "Test".to_string(), GameSettings::default());
        game.add_player(1, "Empire 1".to_string(), "terran".to_string(), "LightCyan".to_string()).unwrap();
        game.add_player(2, "Empire 2".to_string(), "psilon".to_string(), "LightMagenta".to_string()).unwrap();

        game.start_game();
        assert_eq!(game.status, GameStatus::InProgress);
        assert_eq!(game.turn_number, 1);
    }

    #[test]
    fn test_colony_state() {
        let colony = ColonyState::new(0, 200);
        assert!(colony.production_output() > 0);
        assert!(colony.population_growth() > 0);
    }

    #[test]
    fn test_empire_research_output() {
        let mut empire = EmpireState::new(0, 1, "Test".to_string(), "terran".to_string(), "LightCyan".to_string());
        let mut colony = ColonyState::new(0, 200);
        colony.buildings.push("ResearchLab".to_string());
        empire.colonies.push(colony);

        assert!(empire.research_output() > 0);
    }
}
