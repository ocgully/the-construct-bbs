//! NPC System for Realm of Ralnar
//!
//! Manages non-player characters, their behavior, schedules, and interactions.

use rand::Rng;
use serde::{Deserialize, Serialize};
use super::dialogue::DialogueTree;
use super::state::{Direction, GameState};

/// Extension trait for Direction to add NPC-specific methods
pub trait DirectionExt {
    /// Get direction from delta movement
    fn from_delta(dx: i32, dy: i32) -> Option<Direction>;
}

impl DirectionExt for Direction {
    fn from_delta(dx: i32, dy: i32) -> Option<Direction> {
        match (dx, dy) {
            (0, y) if y > 0 => Some(Direction::Down),
            (0, y) if y < 0 => Some(Direction::Up),
            (x, 0) if x > 0 => Some(Direction::Right),
            (x, 0) if x < 0 => Some(Direction::Left),
            _ => None,
        }
    }
}

// ============================================================================
// MOVEMENT PATTERNS
// ============================================================================

/// How an NPC moves around the map
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MovementPattern {
    /// NPC stays in place
    #[default]
    Stationary,
    /// NPC wanders randomly within a radius
    Wander { radius: u32 },
    /// NPC patrols along a fixed path
    Patrol { path: Vec<(u32, u32)>, current_index: usize },
    /// NPC follows the player
    FollowPlayer { distance: u32 },
    /// NPC moves toward a target position
    MoveToward { target_x: u32, target_y: u32 },
}

// ============================================================================
// SCHEDULE SYSTEM
// ============================================================================

/// A schedule entry for time-based NPC behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// Start time (hour * 60 + minute, 0-1439)
    pub time_start: u32,
    /// End time (hour * 60 + minute, 0-1439)
    pub time_end: u32,
    /// Where the NPC should be: (map_id, x, y)
    pub location: (String, u32, u32),
    /// Optional dialogue override during this schedule
    pub dialogue_override: Option<String>,
}

impl ScheduleEntry {
    /// Create a new schedule entry
    pub fn new(start_hour: u32, start_min: u32, end_hour: u32, end_min: u32,
               map: &str, x: u32, y: u32) -> Self {
        Self {
            time_start: start_hour * 60 + start_min,
            time_end: end_hour * 60 + end_min,
            location: (map.to_string(), x, y),
            dialogue_override: None,
        }
    }

    /// Check if a given time falls within this entry
    pub fn is_active_at(&self, time: u32) -> bool {
        if self.time_start <= self.time_end {
            time >= self.time_start && time < self.time_end
        } else {
            // Wraps around midnight
            time >= self.time_start || time < self.time_end
        }
    }
}

/// A complete schedule for an NPC
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Schedule {
    pub entries: Vec<ScheduleEntry>,
}

impl Schedule {
    /// Get the active schedule entry for a given time
    pub fn get_active_entry(&self, time: u32) -> Option<&ScheduleEntry> {
        self.entries.iter().find(|e| e.is_active_at(time))
    }
}

// ============================================================================
// NPC STRUCTURE
// ============================================================================

/// A non-player character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Sprite/visual identifier
    pub sprite_id: String,
    /// Current map location
    pub map_id: String,
    /// X position on map
    pub x: u32,
    /// Y position on map
    pub y: u32,
    /// Origin position (for wandering NPCs)
    pub origin_x: u32,
    /// Origin position (for wandering NPCs)
    pub origin_y: u32,
    /// Facing direction
    pub direction: Direction,
    /// Movement behavior
    pub movement: MovementPattern,
    /// Base dialogue tree ID
    pub dialogue_id: String,
    /// Optional time-based schedule
    pub schedule: Option<Schedule>,
    /// Whether NPC is currently visible
    pub visible: bool,
    /// Whether NPC blocks movement
    pub solid: bool,
    /// Conditions for NPC to appear
    pub appear_conditions: Vec<String>,
    /// Conditions for NPC to disappear
    pub disappear_conditions: Vec<String>,
    /// Movement timer for wandering
    #[serde(skip)]
    pub movement_timer: f32,
}

impl NPC {
    /// Create a new NPC with basic settings
    pub fn new(id: &str, name: &str, x: u32, y: u32, dialogue_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            sprite_id: id.to_string(),
            map_id: String::new(),
            x,
            y,
            origin_x: x,
            origin_y: y,
            direction: Direction::Down,
            movement: MovementPattern::Stationary,
            dialogue_id: dialogue_id.to_string(),
            schedule: None,
            visible: true,
            solid: true,
            appear_conditions: Vec::new(),
            disappear_conditions: Vec::new(),
            movement_timer: 0.0,
        }
    }

    /// Create a wandering NPC
    pub fn wandering(id: &str, name: &str, x: u32, y: u32, dialogue_id: &str, radius: u32) -> Self {
        let mut npc = Self::new(id, name, x, y, dialogue_id);
        npc.movement = MovementPattern::Wander { radius };
        npc
    }

    /// Create a patrolling NPC
    pub fn patrolling(id: &str, name: &str, path: Vec<(u32, u32)>, dialogue_id: &str) -> Self {
        let (x, y) = path.first().copied().unwrap_or((0, 0));
        let mut npc = Self::new(id, name, x, y, dialogue_id);
        npc.movement = MovementPattern::Patrol { path, current_index: 0 };
        npc
    }

    /// Check if NPC should be visible given game state
    pub fn check_visibility(&mut self, state: &GameState) {
        // Check disappear conditions first (higher priority)
        for flag in &self.disappear_conditions {
            if state.has_flag(flag) {
                self.visible = false;
                return;
            }
        }

        // Check appear conditions
        if self.appear_conditions.is_empty() {
            self.visible = true;
        } else {
            self.visible = self.appear_conditions.iter().any(|flag| state.has_flag(flag));
        }
    }

    /// Get the appropriate dialogue tree ID for current state
    pub fn get_dialogue_id(&self, state: &GameState, current_time: Option<u32>) -> String {
        // Check schedule for override
        if let (Some(schedule), Some(time)) = (&self.schedule, current_time) {
            if let Some(entry) = schedule.get_active_entry(time) {
                if let Some(override_dialogue) = &entry.dialogue_override {
                    return override_dialogue.clone();
                }
            }
        }

        // Check for flag-based dialogue overrides
        // Format: dialogue_override_{npc_id}_{dialogue_id}
        let prefix = format!("dialogue_override_{}_", self.id);
        for (flag, &value) in &state.story_flags {
            if value && flag.starts_with(&prefix) {
                return flag.replace(&prefix, "");
            }
        }

        self.dialogue_id.clone()
    }

    /// Handle interaction with this NPC
    pub fn on_interact(&self, state: &GameState) -> Option<String> {
        if !self.visible {
            return None;
        }
        Some(self.get_dialogue_id(state, None))
    }

    /// Update NPC behavior
    pub fn update(&mut self, dt: f32, collision_check: impl Fn(u32, u32) -> bool) {
        self.movement_timer += dt;

        match &mut self.movement {
            MovementPattern::Stationary => {}

            MovementPattern::Wander { radius } => {
                // Move every 2-4 seconds
                if self.movement_timer > 2.0 {
                    self.movement_timer = 0.0;

                    // Random direction
                    let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
                    let mut rng = rand::thread_rng();
                    let dir = directions[rng.gen_range(0..4)];
                    let (dx, dy) = dir.delta();

                    let new_x = (self.x as i32 + dx).max(0) as u32;
                    let new_y = (self.y as i32 + dy).max(0) as u32;

                    // Check radius constraint
                    let dist_x = (new_x as i32 - self.origin_x as i32).unsigned_abs();
                    let dist_y = (new_y as i32 - self.origin_y as i32).unsigned_abs();

                    if dist_x <= *radius && dist_y <= *radius && collision_check(new_x, new_y) {
                        self.x = new_x;
                        self.y = new_y;
                        self.direction = dir;
                    }
                }
            }

            MovementPattern::Patrol { path, current_index } => {
                if self.movement_timer > 1.0 {
                    self.movement_timer = 0.0;

                    if let Some(&(target_x, target_y)) = path.get(*current_index) {
                        if self.x == target_x && self.y == target_y {
                            *current_index = (*current_index + 1) % path.len();
                        } else {
                            // Move toward target
                            let dx = (target_x as i32 - self.x as i32).signum();
                            let dy = (target_y as i32 - self.y as i32).signum();

                            let new_x = (self.x as i32 + dx).max(0) as u32;
                            let new_y = (self.y as i32 + dy).max(0) as u32;

                            if collision_check(new_x, new_y) {
                                self.x = new_x;
                                self.y = new_y;
                                if let Some(dir) = <Direction as DirectionExt>::from_delta(dx, dy) {
                                    self.direction = dir;
                                }
                            }
                        }
                    }
                }
            }

            MovementPattern::FollowPlayer { distance: _ } => {
                // Handled externally with player position
            }

            MovementPattern::MoveToward { target_x, target_y } => {
                if self.movement_timer > 0.5 {
                    self.movement_timer = 0.0;

                    if self.x != *target_x || self.y != *target_y {
                        let dx = (*target_x as i32 - self.x as i32).signum();
                        let dy = (*target_y as i32 - self.y as i32).signum();

                        let new_x = (self.x as i32 + dx).max(0) as u32;
                        let new_y = (self.y as i32 + dy).max(0) as u32;

                        if collision_check(new_x, new_y) {
                            self.x = new_x;
                            self.y = new_y;
                            if let Some(dir) = <Direction as DirectionExt>::from_delta(dx, dy) {
                                self.direction = dir;
                            }
                        }
                    } else {
                        // Reached destination, become stationary
                        self.movement = MovementPattern::Stationary;
                    }
                }
            }
        }
    }

    /// Handle being bumped by the player
    pub fn on_bumped(&mut self, from_direction: Direction) {
        // Turn to face the player
        self.direction = from_direction.opposite();
    }

    /// Update position based on schedule
    pub fn update_schedule(&mut self, current_time: u32) {
        if let Some(schedule) = &self.schedule {
            if let Some(entry) = schedule.get_active_entry(current_time) {
                if self.map_id != entry.location.0 {
                    self.map_id = entry.location.0.clone();
                    self.x = entry.location.1;
                    self.y = entry.location.2;
                }
            }
        }
    }
}

// ============================================================================
// NPC REGISTRY
// ============================================================================

/// Collection of NPCs for a map
#[derive(Debug, Clone, Default)]
pub struct NPCRegistry {
    pub npcs: Vec<NPC>,
}

impl NPCRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { npcs: Vec::new() }
    }

    /// Add an NPC to the registry
    pub fn add(&mut self, npc: NPC) {
        self.npcs.push(npc);
    }

    /// Get an NPC by ID
    pub fn get(&self, id: &str) -> Option<&NPC> {
        self.npcs.iter().find(|n| n.id == id)
    }

    /// Get a mutable reference to an NPC by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut NPC> {
        self.npcs.iter_mut().find(|n| n.id == id)
    }

    /// Get all NPCs at a specific position
    pub fn at_position(&self, x: u32, y: u32) -> Vec<&NPC> {
        self.npcs.iter()
            .filter(|n| n.visible && n.x == x && n.y == y)
            .collect()
    }

    /// Get NPC at position (first match)
    pub fn get_at(&self, x: u32, y: u32) -> Option<&NPC> {
        self.npcs.iter()
            .find(|n| n.visible && n.x == x && n.y == y)
    }

    /// Check if position is blocked by a solid NPC
    pub fn is_blocked(&self, x: u32, y: u32) -> bool {
        self.npcs.iter()
            .any(|n| n.visible && n.solid && n.x == x && n.y == y)
    }

    /// Update all NPCs
    pub fn update(&mut self, dt: f32, game_state: &GameState, collision_check: impl Fn(u32, u32) -> bool + Copy) {
        for npc in &mut self.npcs {
            npc.check_visibility(game_state);
            if npc.visible {
                npc.update(dt, collision_check);
            }
        }
    }

    /// Get all visible NPCs on a specific map
    pub fn on_map(&self, map_id: &str) -> Vec<&NPC> {
        self.npcs.iter()
            .filter(|n| n.visible && n.map_id == map_id)
            .collect()
    }
}

// ============================================================================
// PREDEFINED NPCS
// ============================================================================

/// Create Dorl - the manipulator
pub fn create_dorl() -> NPC {
    let mut npc = NPC::new("dorl", "Dorl", 10, 10, "dorl_first_meeting");
    npc.sprite_id = "old_man".to_string();
    npc.appear_conditions = vec!["chapter_1_started".to_string()];
    npc
}

/// Create a generic villager
pub fn create_villager(id: &str, name: &str, x: u32, y: u32) -> NPC {
    NPC::wandering(id, name, x, y, &format!("villager_{}", id), 3)
}

/// Create a shopkeeper
pub fn create_shopkeeper(id: &str, name: &str, x: u32, y: u32, shop_id: &str) -> NPC {
    let mut npc = NPC::new(id, name, x, y, &format!("shop_{}", shop_id));
    npc.movement = MovementPattern::Stationary;
    npc
}

/// Create a guard NPC
pub fn create_guard(id: &str, patrol_path: Vec<(u32, u32)>) -> NPC {
    NPC::patrolling(id, "Guard", patrol_path, "guard_dialogue")
}

// ============================================================================
// DIALOGUE TREE PROVIDER
// ============================================================================

/// Trait for providing dialogue trees
pub trait DialogueProvider {
    fn get_dialogue(&self, dialogue_id: &str, state: &GameState) -> Option<DialogueTree>;
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> GameState {
        GameState::new(1, "TestPlayer".to_string())
    }

    #[test]
    fn test_npc_creation() {
        let npc = NPC::new("test_npc", "Test NPC", 5, 10, "test_dialogue");
        assert_eq!(npc.id, "test_npc");
        assert_eq!(npc.name, "Test NPC");
        assert_eq!(npc.x, 5);
        assert_eq!(npc.y, 10);
        assert_eq!(npc.dialogue_id, "test_dialogue");
        assert!(npc.visible);
        assert!(npc.solid);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn test_direction_from_delta() {
        assert_eq!(<Direction as DirectionExt>::from_delta(0, -1), Some(Direction::Up));
        assert_eq!(<Direction as DirectionExt>::from_delta(0, 1), Some(Direction::Down));
        assert_eq!(<Direction as DirectionExt>::from_delta(-1, 0), Some(Direction::Left));
        assert_eq!(<Direction as DirectionExt>::from_delta(1, 0), Some(Direction::Right));
        assert_eq!(<Direction as DirectionExt>::from_delta(1, 1), None);
    }

    #[test]
    fn test_schedule_entry_timing() {
        // 9 AM to 5 PM
        let entry = ScheduleEntry::new(9, 0, 17, 0, "shop", 5, 5);

        assert!(entry.is_active_at(9 * 60));      // 9:00 AM
        assert!(entry.is_active_at(12 * 60));     // 12:00 PM
        assert!(entry.is_active_at(16 * 60 + 59)); // 4:59 PM
        assert!(!entry.is_active_at(17 * 60));    // 5:00 PM
        assert!(!entry.is_active_at(8 * 60));     // 8:00 AM
    }

    #[test]
    fn test_schedule_midnight_wrap() {
        // Night shift: 10 PM to 6 AM
        let entry = ScheduleEntry::new(22, 0, 6, 0, "night_post", 10, 10);

        assert!(entry.is_active_at(23 * 60));     // 11:00 PM
        assert!(entry.is_active_at(0));           // Midnight
        assert!(entry.is_active_at(5 * 60));      // 5:00 AM
        assert!(!entry.is_active_at(6 * 60));     // 6:00 AM
        assert!(!entry.is_active_at(12 * 60));    // Noon
    }

    #[test]
    fn test_npc_visibility_conditions() {
        let mut state = create_test_state();
        let mut npc = NPC::new("test", "Test", 0, 0, "test");
        npc.appear_conditions = vec!["quest_started".to_string()];

        npc.check_visibility(&state);
        assert!(!npc.visible);

        state.set_flag("quest_started");
        npc.check_visibility(&state);
        assert!(npc.visible);
    }

    #[test]
    fn test_npc_disappear_priority() {
        let mut state = create_test_state();
        let mut npc = NPC::new("test", "Test", 0, 0, "test");
        npc.appear_conditions = vec!["quest_started".to_string()];
        npc.disappear_conditions = vec!["quest_complete".to_string()];

        state.set_flag("quest_started");
        npc.check_visibility(&state);
        assert!(npc.visible);

        state.set_flag("quest_complete");
        npc.check_visibility(&state);
        assert!(!npc.visible);
    }

    #[test]
    fn test_npc_on_bumped() {
        let mut npc = NPC::new("test", "Test", 0, 0, "test");
        npc.direction = Direction::Down;

        npc.on_bumped(Direction::Up);
        assert_eq!(npc.direction, Direction::Down);

        npc.on_bumped(Direction::Left);
        assert_eq!(npc.direction, Direction::Right);
    }

    #[test]
    fn test_npc_registry() {
        let mut registry = NPCRegistry::new();
        registry.add(NPC::new("npc1", "NPC 1", 5, 5, "dialogue1"));
        registry.add(NPC::new("npc2", "NPC 2", 10, 10, "dialogue2"));

        assert!(registry.get("npc1").is_some());
        assert!(registry.get("npc2").is_some());
        assert!(registry.get("npc3").is_none());
    }

    #[test]
    fn test_npc_registry_at_position() {
        let mut registry = NPCRegistry::new();
        registry.add(NPC::new("npc1", "NPC 1", 5, 5, "dialogue1"));
        registry.add(NPC::new("npc2", "NPC 2", 5, 5, "dialogue2"));
        registry.add(NPC::new("npc3", "NPC 3", 10, 10, "dialogue3"));

        let at_5_5 = registry.at_position(5, 5);
        assert_eq!(at_5_5.len(), 2);

        let at_10_10 = registry.at_position(10, 10);
        assert_eq!(at_10_10.len(), 1);

        let at_0_0 = registry.at_position(0, 0);
        assert!(at_0_0.is_empty());
    }

    #[test]
    fn test_npc_registry_is_blocked() {
        let mut registry = NPCRegistry::new();
        let mut npc = NPC::new("npc1", "NPC 1", 5, 5, "dialogue1");
        npc.solid = true;
        registry.add(npc);

        assert!(registry.is_blocked(5, 5));
        assert!(!registry.is_blocked(6, 5));
    }

    #[test]
    fn test_wandering_npc_creation() {
        let npc = NPC::wandering("wanderer", "Wanderer", 10, 10, "wander_talk", 5);

        match &npc.movement {
            MovementPattern::Wander { radius } => assert_eq!(*radius, 5),
            _ => panic!("Expected Wander movement pattern"),
        }
        assert_eq!(npc.origin_x, 10);
        assert_eq!(npc.origin_y, 10);
    }

    #[test]
    fn test_patrolling_npc_creation() {
        let path = vec![(0, 0), (5, 0), (5, 5), (0, 5)];
        let npc = NPC::patrolling("guard", "Guard", path.clone(), "guard_talk");

        match &npc.movement {
            MovementPattern::Patrol { path: p, current_index } => {
                assert_eq!(p.len(), 4);
                assert_eq!(*current_index, 0);
            }
            _ => panic!("Expected Patrol movement pattern"),
        }
        assert_eq!(npc.x, 0);
        assert_eq!(npc.y, 0);
    }

    #[test]
    fn test_npc_dialogue_override() {
        let mut state = create_test_state();
        let npc = NPC::new("merchant", "Merchant", 5, 5, "merchant_normal");

        // Normal dialogue
        assert_eq!(npc.get_dialogue_id(&state, None), "merchant_normal");

        // Set override flag
        state.set_flag("dialogue_override_merchant_merchant_angry");
        assert_eq!(npc.get_dialogue_id(&state, None), "merchant_angry");
    }

    #[test]
    fn test_create_dorl() {
        let dorl = create_dorl();
        assert_eq!(dorl.id, "dorl");
        assert_eq!(dorl.name, "Dorl");
        assert!(dorl.appear_conditions.contains(&"chapter_1_started".to_string()));
    }

    #[test]
    fn test_create_villager() {
        let villager = create_villager("vil1", "Tom", 20, 30);
        assert_eq!(villager.id, "vil1");
        assert_eq!(villager.name, "Tom");
        assert!(matches!(villager.movement, MovementPattern::Wander { radius: 3 }));
    }

    #[test]
    fn test_create_shopkeeper() {
        let keeper = create_shopkeeper("arms_dealer", "Marcus", 15, 10, "weapons");
        assert_eq!(keeper.id, "arms_dealer");
        assert!(matches!(keeper.movement, MovementPattern::Stationary));
        assert_eq!(keeper.dialogue_id, "shop_weapons");
    }

    #[test]
    fn test_npc_update_stationary() {
        let mut npc = NPC::new("test", "Test", 5, 5, "test");
        npc.movement = MovementPattern::Stationary;

        let collision = |_x: u32, _y: u32| true;
        npc.update(1.0, collision);

        assert_eq!(npc.x, 5);
        assert_eq!(npc.y, 5);
    }

    #[test]
    fn test_schedule_get_active_entry() {
        let schedule = Schedule {
            entries: vec![
                ScheduleEntry::new(9, 0, 12, 0, "shop", 5, 5),
                ScheduleEntry::new(12, 0, 17, 0, "tavern", 10, 10),
            ],
        };

        let morning = schedule.get_active_entry(10 * 60);
        assert!(morning.is_some());
        assert_eq!(morning.unwrap().location.0, "shop");

        let afternoon = schedule.get_active_entry(14 * 60);
        assert!(afternoon.is_some());
        assert_eq!(afternoon.unwrap().location.0, "tavern");

        let night = schedule.get_active_entry(20 * 60);
        assert!(night.is_none());
    }
}
