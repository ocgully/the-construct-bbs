//! Game state for Fortress
//!
//! Main state struct that holds all fortress data.

use serde::{Serialize, Deserialize};
use rand::SeedableRng;
use rand::rngs::StdRng;

use super::terrain::Terrain;
use super::dwarves::{Dwarf, DwarfStatus};
use super::jobs::JobQueue;

/// Resources stored in the fortress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    // Raw materials
    pub wood: u32,
    pub stone: u32,
    pub iron_ore: u32,
    pub copper_ore: u32,
    pub gold_ore: u32,
    pub gem: u32,
    pub plant_fiber: u32,
    pub hide: u32,

    // Processed
    pub iron: u32,
    pub copper: u32,
    pub gold: u32,
    pub cut_gem: u32,
    pub cloth: u32,
    pub leather: u32,
    pub plank: u32,
    pub block: u32,

    // Food and drink
    pub meat: u32,
    pub fish: u32,
    pub vegetable: u32,
    pub grain: u32,
    pub meal: u32,
    pub plump_helmet: u32,
    pub water: u32,
    pub ale: u32,
    pub wine: u32,
    pub mead: u32,

    // Crafted goods
    pub furniture: u32,
    pub tool: u32,
    pub weapon: u32,
    pub armor: u32,
    pub craft: u32,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            // Start with basic resources
            wood: 50,
            stone: 30,
            plank: 20,
            meal: 30,
            ale: 20,
            water: 50,
            tool: 5,
            ..Default::default()
        }
    }

    pub fn get(&self, resource: &str) -> u32 {
        match resource {
            "wood" => self.wood,
            "stone" => self.stone,
            "iron_ore" => self.iron_ore,
            "copper_ore" => self.copper_ore,
            "gold_ore" => self.gold_ore,
            "gem" => self.gem,
            "plant_fiber" => self.plant_fiber,
            "hide" => self.hide,
            "iron" => self.iron,
            "copper" => self.copper,
            "gold" => self.gold,
            "cut_gem" => self.cut_gem,
            "cloth" => self.cloth,
            "leather" => self.leather,
            "plank" => self.plank,
            "block" => self.block,
            "meat" => self.meat,
            "fish" => self.fish,
            "vegetable" => self.vegetable,
            "grain" => self.grain,
            "meal" => self.meal,
            "plump_helmet" => self.plump_helmet,
            "water" => self.water,
            "ale" => self.ale,
            "wine" => self.wine,
            "mead" => self.mead,
            "furniture" => self.furniture,
            "tool" => self.tool,
            "weapon" => self.weapon,
            "armor" => self.armor,
            "craft" => self.craft,
            _ => 0,
        }
    }

    pub fn set(&mut self, resource: &str, amount: u32) {
        match resource {
            "wood" => self.wood = amount,
            "stone" => self.stone = amount,
            "iron_ore" => self.iron_ore = amount,
            "copper_ore" => self.copper_ore = amount,
            "gold_ore" => self.gold_ore = amount,
            "gem" => self.gem = amount,
            "plant_fiber" => self.plant_fiber = amount,
            "hide" => self.hide = amount,
            "iron" => self.iron = amount,
            "copper" => self.copper = amount,
            "gold" => self.gold = amount,
            "cut_gem" => self.cut_gem = amount,
            "cloth" => self.cloth = amount,
            "leather" => self.leather = amount,
            "plank" => self.plank = amount,
            "block" => self.block = amount,
            "meat" => self.meat = amount,
            "fish" => self.fish = amount,
            "vegetable" => self.vegetable = amount,
            "grain" => self.grain = amount,
            "meal" => self.meal = amount,
            "plump_helmet" => self.plump_helmet = amount,
            "water" => self.water = amount,
            "ale" => self.ale = amount,
            "wine" => self.wine = amount,
            "mead" => self.mead = amount,
            "furniture" => self.furniture = amount,
            "tool" => self.tool = amount,
            "weapon" => self.weapon = amount,
            "armor" => self.armor = amount,
            "craft" => self.craft = amount,
            _ => {}
        }
    }

    pub fn add(&mut self, resource: &str, amount: u32) {
        let current = self.get(resource);
        self.set(resource, current.saturating_add(amount));
    }

    pub fn remove(&mut self, resource: &str, amount: u32) -> bool {
        let current = self.get(resource);
        if current >= amount {
            self.set(resource, current - amount);
            true
        } else {
            false
        }
    }

    /// Check if we have enough resources
    pub fn has(&self, requirements: &[(&str, u32)]) -> bool {
        requirements.iter().all(|(r, a)| self.get(r) >= *a)
    }

    /// Remove multiple resources at once
    pub fn consume(&mut self, requirements: &[(&str, u32)]) -> bool {
        if !self.has(requirements) {
            return false;
        }
        for (r, a) in requirements {
            self.remove(r, *a);
        }
        true
    }

    /// Calculate total value of resources
    pub fn total_value(&self) -> u64 {
        use super::data::get_resource;

        let resources = [
            "wood", "stone", "iron_ore", "copper_ore", "gold_ore", "gem",
            "plant_fiber", "hide", "iron", "copper", "gold", "cut_gem",
            "cloth", "leather", "plank", "block", "meat", "fish",
            "vegetable", "grain", "meal", "plump_helmet", "water",
            "ale", "wine", "mead", "furniture", "tool", "weapon", "armor", "craft",
        ];

        resources.iter()
            .map(|r| {
                let amount = self.get(r) as u64;
                let value = get_resource(r).map(|d| d.base_value).unwrap_or(1) as u64;
                amount * value
            })
            .sum()
    }
}

/// A built workshop in the fortress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workshop {
    pub id: u32,
    pub workshop_type: String,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub assigned_dwarf: Option<u32>,
    pub current_order: Option<u32>,
}

/// A building in the fortress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: u32,
    pub building_type: String,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u8,
    pub height: u8,
}

/// A designated room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: u32,
    pub room_type: String,
    pub name: String,
    pub tiles: Vec<(u32, u32, u32)>,
    pub furniture: Vec<String>,
    pub assigned_dwarf: Option<u32>,
    pub quality: u8, // 0-100
}

impl Room {
    pub fn size(&self) -> u32 {
        self.tiles.len() as u32
    }
}

/// Active invasion/siege
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invasion {
    pub id: u32,
    pub enemy_type: String,
    pub enemies: Vec<Enemy>,
    pub started_at: i64,
    pub waves_remaining: u8,
}

/// An enemy creature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u32,
    pub enemy_type: String,
    pub health: u32,
    pub max_health: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// Statistics tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FortressStats {
    pub tiles_mined: u32,
    pub trees_chopped: u32,
    pub items_crafted: u32,
    pub food_consumed: u32,
    pub drinks_consumed: u32,
    pub invasions_repelled: u32,
    pub dwarves_lost: u32,
    pub enemies_slain: u32,
    pub peak_population: u32,
    pub wealth_created: u64,
}

/// Main game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // Identity
    pub fortress_name: String,
    pub embark_seed: u64,

    // Time
    pub tick: i64,
    pub year: u32,
    pub season: u8, // 0-3: Spring, Summer, Autumn, Winter

    // Map
    pub terrain: Terrain,
    pub view_z: u32,  // Currently viewed z-level
    pub view_x: u32,  // Viewport offset
    pub view_y: u32,

    // Population
    pub dwarves: Vec<Dwarf>,
    pub next_dwarf_id: u32,

    // Economy
    pub resources: Resources,

    // Infrastructure
    pub workshops: Vec<Workshop>,
    pub buildings: Vec<Building>,
    pub rooms: Vec<Room>,
    pub next_building_id: u32,

    // Work
    pub job_queue: JobQueue,

    // Combat
    pub invasions: Vec<Invasion>,
    pub next_enemy_id: u32,

    // Statistics
    pub stats: FortressStats,

    // UI state (transient but serialized for save/resume)
    #[serde(default)]
    pub last_message: Option<String>,
    #[serde(default)]
    pub notifications: Vec<String>,
}

impl GameState {
    /// Create a new fortress with initial state
    pub fn new(name: String, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Generate terrain
        let terrain = Terrain::new(80, 50, 10, seed);

        // Create initial dwarves
        let mut dwarves = Vec::new();
        for i in 0..7 {
            let mut dwarf = Dwarf::new(i + 1, &mut rng);
            // Place at fortress entrance
            let (ex, ey) = terrain.fortress_entrance;
            dwarf.x = ex;
            dwarf.y = ey;
            dwarf.z = 0;
            dwarves.push(dwarf);
        }

        Self {
            fortress_name: name,
            embark_seed: seed,
            tick: 0,
            year: 1,
            season: 0,
            terrain,
            view_z: 0,
            view_x: 0,
            view_y: 0,
            dwarves,
            next_dwarf_id: 8,
            resources: Resources::new(),
            workshops: Vec::new(),
            buildings: Vec::new(),
            rooms: Vec::new(),
            next_building_id: 1,
            job_queue: JobQueue::new(),
            invasions: Vec::new(),
            next_enemy_id: 1,
            stats: FortressStats::default(),
            last_message: None,
            notifications: Vec::new(),
        }
    }

    /// Get a dwarf by ID
    pub fn get_dwarf(&self, id: u32) -> Option<&Dwarf> {
        self.dwarves.iter().find(|d| d.id == id)
    }

    /// Get mutable dwarf by ID
    pub fn get_dwarf_mut(&mut self, id: u32) -> Option<&mut Dwarf> {
        self.dwarves.iter_mut().find(|d| d.id == id)
    }

    /// Get living dwarves
    pub fn living_dwarves(&self) -> Vec<&Dwarf> {
        self.dwarves.iter()
            .filter(|d| d.status != DwarfStatus::Dead)
            .collect()
    }

    /// Get idle dwarves
    pub fn idle_dwarves(&self) -> Vec<&Dwarf> {
        self.dwarves.iter()
            .filter(|d| d.can_work())
            .collect()
    }

    /// Get workshop by ID
    pub fn get_workshop(&self, id: u32) -> Option<&Workshop> {
        self.workshops.iter().find(|w| w.id == id)
    }

    /// Get room by ID
    pub fn get_room(&self, id: u32) -> Option<&Room> {
        self.rooms.iter().find(|r| r.id == id)
    }

    /// Add a new dwarf (migration)
    pub fn add_dwarf(&mut self) -> u32 {
        let mut rng = StdRng::seed_from_u64(self.embark_seed.wrapping_add(self.next_dwarf_id as u64));
        let mut dwarf = Dwarf::new(self.next_dwarf_id, &mut rng);

        // Place at entrance
        let (ex, ey) = self.terrain.fortress_entrance;
        dwarf.x = ex;
        dwarf.y = ey;
        dwarf.z = 0;

        let id = dwarf.id;
        self.dwarves.push(dwarf);
        self.next_dwarf_id += 1;

        // Update peak population
        let pop = self.living_dwarves().len() as u32;
        if pop > self.stats.peak_population {
            self.stats.peak_population = pop;
        }

        id
    }

    /// Build a workshop
    pub fn build_workshop(&mut self, workshop_type: &str, x: u32, y: u32, z: u32) -> Option<u32> {
        use super::data::get_workshop;

        let def = get_workshop(workshop_type)?;

        // Check resources
        let cost = &def.build_cost;
        if !self.resources.has(&[
            ("wood", cost.wood),
            ("stone", cost.stone),
            ("iron", cost.iron),
        ]) {
            return None;
        }

        // Consume resources
        self.resources.consume(&[
            ("wood", cost.wood),
            ("stone", cost.stone),
            ("iron", cost.iron),
        ]);

        let id = self.next_building_id;
        self.next_building_id += 1;

        self.workshops.push(Workshop {
            id,
            workshop_type: workshop_type.to_string(),
            x,
            y,
            z,
            assigned_dwarf: None,
            current_order: None,
        });

        // Mark tiles as workshop
        for dy in 0..def.size.1 {
            for dx in 0..def.size.0 {
                if let Some(tile) = self.terrain.get_mut(x + dx as u32, y + dy as u32, z) {
                    tile.tile_type = super::terrain::TileType::Workshop;
                }
            }
        }

        Some(id)
    }

    /// Designate a room
    pub fn designate_room(&mut self, room_type: &str, name: String, tiles: Vec<(u32, u32, u32)>) -> Option<u32> {
        use super::data::get_room_type;

        let def = get_room_type(room_type)?;

        if tiles.len() < def.min_size as usize {
            return None;
        }

        let id = self.next_building_id;
        self.next_building_id += 1;

        // Mark tiles as belonging to room
        for &(x, y, z) in &tiles {
            if let Some(tile) = self.terrain.get_mut(x, y, z) {
                tile.room_id = Some(id);
            }
        }

        self.rooms.push(Room {
            id,
            room_type: room_type.to_string(),
            name,
            tiles,
            furniture: Vec::new(),
            assigned_dwarf: None,
            quality: 0,
        });

        Some(id)
    }

    /// Assign a room to a dwarf
    pub fn assign_room(&mut self, room_id: u32, dwarf_id: u32) -> bool {
        if let Some(room) = self.rooms.iter_mut().find(|r| r.id == room_id) {
            if room.room_type == "bedroom" {
                room.assigned_dwarf = Some(dwarf_id);

                if let Some(dwarf) = self.get_dwarf_mut(dwarf_id) {
                    dwarf.room_id = Some(room_id);
                }
                return true;
            }
        }
        false
    }

    /// Check if fortress is under siege
    pub fn under_siege(&self) -> bool {
        self.invasions.iter().any(|i| !i.enemies.is_empty())
    }

    /// Get total fortress value (wealth)
    pub fn fortress_value(&self) -> u64 {
        let resource_value = self.resources.total_value();

        let building_value = (self.workshops.len() * 500 +
                             self.buildings.len() * 200 +
                             self.rooms.len() * 100) as u64;

        let dwarf_value = self.living_dwarves().len() as u64 * 1000;

        resource_value + building_value + dwarf_value
    }

    /// Add a notification
    pub fn notify(&mut self, message: String) {
        self.notifications.push(message);
        // Keep only last 20 notifications
        if self.notifications.len() > 20 {
            self.notifications.remove(0);
        }
    }

    /// Get season name
    pub fn season_name(&self) -> &'static str {
        match self.season {
            0 => "Spring",
            1 => "Summer",
            2 => "Autumn",
            _ => "Winter",
        }
    }

    /// Format current date
    pub fn date_string(&self) -> String {
        format!("{} of Year {}", self.season_name(), self.year)
    }
}

// Types are exported directly in mod.rs from their respective modules

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let state = GameState::new("TestFortress".to_string(), 12345);

        assert_eq!(state.fortress_name, "TestFortress");
        assert_eq!(state.dwarves.len(), 7);
        assert_eq!(state.tick, 0);
        assert_eq!(state.year, 1);
    }

    #[test]
    fn test_resources() {
        let mut resources = Resources::new();

        assert!(resources.get("wood") > 0);

        resources.add("iron", 10);
        assert_eq!(resources.get("iron"), 10);

        assert!(resources.remove("iron", 5));
        assert_eq!(resources.get("iron"), 5);

        assert!(!resources.remove("iron", 10)); // Not enough
        assert_eq!(resources.get("iron"), 5);
    }

    #[test]
    fn test_resource_consumption() {
        let mut resources = Resources::new();
        resources.wood = 100;
        resources.stone = 50;

        assert!(resources.has(&[("wood", 50), ("stone", 30)]));
        assert!(!resources.has(&[("wood", 50), ("stone", 100)]));

        assert!(resources.consume(&[("wood", 50), ("stone", 30)]));
        assert_eq!(resources.wood, 50);
        assert_eq!(resources.stone, 20);
    }

    #[test]
    fn test_add_dwarf() {
        let mut state = GameState::new("Test".to_string(), 42);
        let initial_count = state.dwarves.len();

        state.add_dwarf();

        assert_eq!(state.dwarves.len(), initial_count + 1);
    }

    #[test]
    fn test_fortress_value() {
        let state = GameState::new("Test".to_string(), 42);
        let value = state.fortress_value();

        assert!(value > 0);
    }

    #[test]
    fn test_seasons() {
        let state = GameState::new("Test".to_string(), 42);
        assert_eq!(state.season_name(), "Spring");

        let mut state2 = state.clone();
        state2.season = 2;
        assert_eq!(state2.season_name(), "Autumn");
    }
}
