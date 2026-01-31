//! Province state - the player's persistent game state

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::data::{BuildingType, UnitType, get_race, get_personality};

/// Complete state for a player's province
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceState {
    /// Province identity
    pub name: String,
    pub race: String,          // Race key
    pub personality: String,   // Personality key

    /// Kingdom membership
    pub kingdom: Option<KingdomMembership>,

    /// Core resources
    pub resources: Resources,

    /// Land and population
    pub land: u32,
    pub peasants: u32,
    pub max_peasants: u32,

    /// Military forces
    pub military: Military,

    /// Buildings
    pub buildings: Buildings,

    /// Science/research progress
    pub sciences: Sciences,

    /// Protection status
    pub protection_ticks: u32,  // Ticks remaining of new player protection

    /// Active effects (spells, etc.)
    pub effects: Vec<ActiveEffect>,

    /// Last tick processed (for catchup calculation)
    pub last_tick: i64,

    /// Current game day within the age
    pub day: u32,

    /// Whether province is still active in the age
    pub is_active: bool,

    /// Message to display on next screen
    #[serde(default)]
    pub last_message: Option<String>,

    /// Statistics for leaderboard
    pub stats: ProvinceStats,
}

/// Kingdom membership info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KingdomMembership {
    pub kingdom_id: i64,
    pub kingdom_name: String,
    pub is_ruler: bool,
    pub joined_at: String,  // ISO date
}

/// Core resources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Resources {
    pub gold: i64,
    pub food: i64,
    pub runes: i64,
    pub prisoners: u32,
}

/// Military forces
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Military {
    pub soldiers: u32,
    pub archers: u32,
    pub knights: u32,
    pub thieves: u32,
    pub wizards: u32,
    pub elites: u32,

    /// Troops currently training (ready next tick)
    pub training_soldiers: u32,
    pub training_archers: u32,
    pub training_knights: u32,
    pub training_thieves: u32,
    pub training_wizards: u32,
    pub training_elites: u32,

    /// Troops on attack (away from province)
    pub deployed_soldiers: u32,
    pub deployed_archers: u32,
    pub deployed_knights: u32,
}

/// Buildings in the province
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buildings {
    pub counts: HashMap<String, u32>,
    /// Buildings currently under construction (complete next tick)
    pub under_construction: HashMap<String, u32>,
}

impl Default for Buildings {
    fn default() -> Self {
        Self {
            counts: HashMap::new(),
            under_construction: HashMap::new(),
        }
    }
}

/// Science research levels
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sciences {
    pub alchemy: u32,      // Rune production
    pub tools: u32,        // Build speed
    pub housing: u32,      // Population capacity
    pub food: u32,         // Food production
    pub military: u32,     // Military effectiveness
    pub crime: u32,        // Thief effectiveness
    pub channeling: u32,   // Magic effectiveness
    /// Current research focus and progress
    pub current_research: Option<String>,
    pub research_progress: u32,
}

/// Active effects on the province
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub effect_type: String,
    pub remaining_ticks: u32,
    pub magnitude: i32,
}

/// Statistics tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvinceStats {
    pub attacks_sent: u32,
    pub attacks_won: u32,
    pub defenses: u32,
    pub defenses_won: u32,
    pub land_captured: u32,
    pub land_lost: u32,
    pub gold_earned: i64,
    pub gold_spent: i64,
    pub spells_cast: u32,
    pub thief_ops: u32,
    pub peak_networth: i64,
}

impl ProvinceState {
    /// Create a new province with starting resources
    pub fn new(name: String, race: String, personality: String) -> Self {
        let mut buildings = Buildings::default();
        // Starting buildings
        buildings.counts.insert(BuildingType::Home.name().to_string(), 10);
        buildings.counts.insert(BuildingType::Farm.name().to_string(), 15);
        buildings.counts.insert(BuildingType::Bank.name().to_string(), 2);
        buildings.counts.insert(BuildingType::Barracks.name().to_string(), 3);
        buildings.counts.insert(BuildingType::Fort.name().to_string(), 2);

        let mut province = Self {
            name,
            race: race.clone(),
            personality: personality.clone(),
            kingdom: None,
            resources: Resources {
                gold: 25000,
                food: 10000,
                runes: 1000,
                prisoners: 0,
            },
            land: 100,
            peasants: 500,
            max_peasants: 0, // Calculated below
            military: Military {
                soldiers: 50,
                archers: 25,
                knights: 5,
                thieves: 10,
                wizards: 5,
                elites: 0,
                ..Default::default()
            },
            buildings,
            sciences: Sciences::default(),
            protection_ticks: 168, // 7 days of protection (24 ticks/day)
            effects: Vec::new(),
            last_tick: 0,
            day: 1,
            is_active: true,
            last_message: None,
            stats: ProvinceStats::default(),
        };

        // Calculate derived values
        province.recalculate_max_peasants();

        province
    }

    /// Get building count
    pub fn get_building(&self, building: BuildingType) -> u32 {
        self.buildings.counts.get(building.name()).copied().unwrap_or(0)
    }

    /// Add buildings
    #[allow(dead_code)] // Used internally and by tick system
    pub fn add_building(&mut self, building: BuildingType, count: u32) {
        *self.buildings.counts.entry(building.name().to_string()).or_insert(0) += count;
        self.recalculate_max_peasants();
    }

    /// Get unit count
    pub fn get_unit(&self, unit: UnitType) -> u32 {
        match unit {
            UnitType::Soldier => self.military.soldiers,
            UnitType::Archer => self.military.archers,
            UnitType::Knight => self.military.knights,
            UnitType::Thief => self.military.thieves,
            UnitType::Wizard => self.military.wizards,
            UnitType::Elite => self.military.elites,
        }
    }

    /// Get total military strength for offense
    pub fn offense_strength(&self) -> u32 {
        let mut strength = 0u32;
        strength += self.military.soldiers * UnitType::Soldier.attack();
        strength += self.military.archers * UnitType::Archer.attack();
        strength += self.military.knights * UnitType::Knight.attack();
        strength += self.military.elites * UnitType::Elite.attack();

        // Apply race bonus
        if let Some(race) = get_race(&self.race) {
            strength = (strength * race.attack_bonus) / 100;
        }

        // Apply science bonus (1% per level)
        strength = (strength * (100 + self.sciences.military)) / 100;

        strength
    }

    /// Get total military strength for defense
    pub fn defense_strength(&self) -> u32 {
        let mut strength = 0u32;
        strength += self.military.soldiers * UnitType::Soldier.defense();
        strength += self.military.archers * UnitType::Archer.defense();
        strength += self.military.knights * UnitType::Knight.defense();
        strength += self.military.thieves * UnitType::Thief.defense();
        strength += self.military.wizards * UnitType::Wizard.defense();
        strength += self.military.elites * UnitType::Elite.defense();

        // Add fort bonus
        let forts = self.get_building(BuildingType::Fort);
        strength += forts * 50;

        // Apply race bonus
        if let Some(race) = get_race(&self.race) {
            strength = (strength * race.defense_bonus) / 100;
        }

        // Apply science bonus
        strength = (strength * (100 + self.sciences.military)) / 100;

        strength
    }

    /// Calculate net worth (for leaderboard ranking)
    pub fn networth(&self) -> i64 {
        let mut worth: i64 = 0;

        // Resources
        worth += self.resources.gold;
        worth += (self.resources.food / 10) as i64;
        worth += (self.resources.runes * 5) as i64;

        // Land (most valuable)
        worth += (self.land as i64) * 500;

        // Population
        worth += (self.peasants as i64) * 10;

        // Military
        worth += (self.military.soldiers as i64) * 15;
        worth += (self.military.archers as i64) * 20;
        worth += (self.military.knights as i64) * 40;
        worth += (self.military.thieves as i64) * 25;
        worth += (self.military.wizards as i64) * 50;
        worth += (self.military.elites as i64) * 75;

        // Buildings
        for (building_name, count) in &self.buildings.counts {
            let cost = BuildingType::all()
                .iter()
                .find(|b| b.name() == building_name)
                .map(|b| b.base_cost())
                .unwrap_or(100);
            worth += (*count as i64) * (cost as i64 / 2);
        }

        // Science
        worth += (self.sciences.alchemy as i64) * 100;
        worth += (self.sciences.tools as i64) * 100;
        worth += (self.sciences.housing as i64) * 100;
        worth += (self.sciences.food as i64) * 100;
        worth += (self.sciences.military as i64) * 100;
        worth += (self.sciences.crime as i64) * 100;
        worth += (self.sciences.channeling as i64) * 100;

        worth
    }

    /// Recalculate max peasants based on homes
    pub fn recalculate_max_peasants(&mut self) {
        let homes = self.get_building(BuildingType::Home);
        let base = homes * 25;

        // Apply housing science bonus
        let bonus = (base * self.sciences.housing) / 100;

        // Apply race bonus (can be negative for races with pop_growth < 100)
        let race_bonus = get_race(&self.race)
            .map(|r| (base as i32 * (r.pop_growth as i32 - 100)) / 100)
            .unwrap_or(0);

        // Combine bonuses, ensuring we don't go below a minimum
        let total = base as i32 + bonus as i32 + race_bonus;
        self.max_peasants = total.max(0) as u32;
    }

    /// Calculate gold income per tick
    pub fn gold_income(&self) -> i64 {
        // Base income from peasants
        let base = (self.peasants as i64) * 2;

        // Bank bonus
        let banks = self.get_building(BuildingType::Bank) as i64;
        let bank_bonus = base * banks * 2 / 100;

        // Race bonus
        let race_bonus = get_race(&self.race)
            .map(|r| base * (r.gold_bonus as i64 - 100) / 100)
            .unwrap_or(0);

        base + bank_bonus + race_bonus
    }

    /// Calculate food production per tick
    pub fn food_production(&self) -> i64 {
        let farms = self.get_building(BuildingType::Farm) as i64;
        let base = farms * 60;

        // Science bonus
        let science_bonus = base * (self.sciences.food as i64) / 100;

        // Race bonus
        let race_bonus = get_race(&self.race)
            .map(|r| base * (r.food_bonus as i64 - 100) / 100)
            .unwrap_or(0);

        base + science_bonus + race_bonus
    }

    /// Calculate food consumption per tick
    pub fn food_consumption(&self) -> i64 {
        // Peasants eat
        let peasant_food = (self.peasants as i64) / 5;

        // Military eats more
        let military_food = (self.total_military() as i64) * 2;

        // Undead don't eat
        if self.race == "undead" {
            return 0;
        }

        peasant_food + military_food
    }

    /// Calculate rune production per tick
    pub fn rune_production(&self) -> i64 {
        let towers = self.get_building(BuildingType::Tower) as i64;
        let base = towers * 10 + (self.military.wizards as i64);

        // Alchemy science bonus
        let science_bonus = base * (self.sciences.alchemy as i64) / 100;

        // Race magic bonus affects runes
        let race_bonus = get_race(&self.race)
            .map(|r| base * (r.magic_bonus as i64 - 100) / 200)
            .unwrap_or(0);

        base + science_bonus + race_bonus
    }

    /// Calculate military upkeep per tick
    pub fn military_upkeep(&self) -> i64 {
        let mut upkeep: i64 = 0;
        upkeep += (self.military.soldiers as i64) * (UnitType::Soldier.upkeep() as i64);
        upkeep += (self.military.archers as i64) * (UnitType::Archer.upkeep() as i64);
        upkeep += (self.military.knights as i64) * (UnitType::Knight.upkeep() as i64);
        upkeep += (self.military.thieves as i64) * (UnitType::Thief.upkeep() as i64);
        upkeep += (self.military.wizards as i64) * (UnitType::Wizard.upkeep() as i64);
        upkeep += (self.military.elites as i64) * (UnitType::Elite.upkeep() as i64);

        // Barracks reduce upkeep
        let barracks = self.get_building(BuildingType::Barracks) as i64;
        let reduction = upkeep * barracks * 15 / 1000; // 1.5% per barracks

        (upkeep - reduction).max(0)
    }

    /// Total military count
    pub fn total_military(&self) -> u32 {
        self.military.soldiers
            + self.military.archers
            + self.military.knights
            + self.military.thieves
            + self.military.wizards
            + self.military.elites
    }

    /// Get building cost with personality modifier
    pub fn building_cost(&self, building: BuildingType) -> u32 {
        let base = building.base_cost();

        if let Some(personality) = get_personality(&self.personality) {
            (base * personality.build_cost) / 100
        } else {
            base
        }
    }

    /// Check if province is still under new player protection
    pub fn is_protected(&self) -> bool {
        self.protection_ticks > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_province() {
        let province = ProvinceState::new(
            "Test Province".to_string(),
            "human".to_string(),
            "merchant".to_string(),
        );

        assert_eq!(province.name, "Test Province");
        assert_eq!(province.race, "human");
        assert_eq!(province.land, 100);
        assert!(province.resources.gold > 0);
        assert!(province.peasants > 0);
        assert!(province.is_protected());
    }

    #[test]
    fn test_networth_calculation() {
        let province = ProvinceState::new(
            "Test".to_string(),
            "human".to_string(),
            "warrior".to_string(),
        );

        let nw = province.networth();
        assert!(nw > 0);
        // Starting province should have decent networth
        assert!(nw > 50000);
    }

    #[test]
    fn test_military_strength() {
        let province = ProvinceState::new(
            "Test".to_string(),
            "orc".to_string(), // Attack bonus
            "warrior".to_string(),
        );

        let offense = province.offense_strength();
        let defense = province.defense_strength();

        assert!(offense > 0);
        assert!(defense > 0);
    }

    #[test]
    fn test_income_calculations() {
        let province = ProvinceState::new(
            "Test".to_string(),
            "dwarf".to_string(), // Gold bonus
            "merchant".to_string(),
        );

        assert!(province.gold_income() > 0);
        assert!(province.food_production() > 0);
        assert!(province.rune_production() >= 0);
    }

    #[test]
    fn test_undead_no_food() {
        let province = ProvinceState::new(
            "Test".to_string(),
            "undead".to_string(),
            "mystic".to_string(),
        );

        assert_eq!(province.food_consumption(), 0);
    }

    #[test]
    fn test_state_serialization() {
        let state = ProvinceState::new(
            "Test".to_string(),
            "elf".to_string(),
            "sage".to_string(),
        );

        let json = serde_json::to_string(&state).unwrap();
        let restored: ProvinceState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.name, restored.name);
        assert_eq!(state.race, restored.race);
        assert_eq!(state.resources.gold, restored.resources.gold);
    }
}
