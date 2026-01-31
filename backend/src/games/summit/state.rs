//! Game state for Summit
//!
//! Contains climber state, run state, and player statistics.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::data::{BiomeType, ItemType};

// ============================================================================
// CLIMBER STATE (within a run)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClimberStatus {
    Active,
    Downed,
    Dead,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub effect_type: StatusEffectType,
    pub duration_remaining: u32, // Ticks remaining
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffectType {
    Cold,
    Poisoned,
    Hungry,
    Exhausted,
    Invulnerable,
    UnlimitedStamina,
    Jitters,
    Heavy,
    Hallucinating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimberState {
    pub user_id: i64,
    pub handle: String,

    // Position
    pub x: i32,
    pub y: i32,  // Height on mountain (0 = bottom, 100 = summit)

    // Stamina
    pub stamina_current: u32,   // 0-100, regenerates
    pub stamina_max: u32,       // Permanent cap, reduced by falls/damage

    // Inventory
    pub items: HashMap<ItemType, u32>,  // item -> count
    pub foods: Vec<u32>,                // food IDs

    // Status
    pub status: ClimberStatus,
    pub status_effects: Vec<StatusEffect>,

    // Run statistics
    pub falls: u32,
    pub items_used: u32,
    pub foods_eaten: Vec<u32>,  // Food IDs eaten this run
    pub revives_given: u32,
    pub revives_received: u32,
    pub ropes_placed: u32,
    pub pitons_placed: u32,

    // Cosmetics
    pub cosmetics: EquippedCosmetics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquippedCosmetics {
    pub uniform: String,
    pub hat: Option<String>,
    pub backpack: String,
    pub accessory: Option<String>,
    pub rope_color: String,
}

impl ClimberState {
    pub fn new(user_id: i64, handle: String) -> Self {
        Self {
            user_id,
            handle,
            x: 40,  // Center of 80-char width
            y: 0,
            stamina_current: 100,
            stamina_max: 100,
            items: HashMap::new(),
            foods: Vec::new(),
            status: ClimberStatus::Active,
            status_effects: Vec::new(),
            falls: 0,
            items_used: 0,
            foods_eaten: Vec::new(),
            revives_given: 0,
            revives_received: 0,
            ropes_placed: 0,
            pitons_placed: 0,
            cosmetics: EquippedCosmetics::default(),
        }
    }

    pub fn with_cosmetics(mut self, cosmetics: EquippedCosmetics) -> Self {
        self.cosmetics = cosmetics;
        self
    }

    pub fn is_active(&self) -> bool {
        self.status == ClimberStatus::Active
    }

    pub fn is_downed(&self) -> bool {
        self.status == ClimberStatus::Downed
    }

    pub fn is_dead(&self) -> bool {
        self.status == ClimberStatus::Dead
    }

    pub fn current_biome(&self) -> BiomeType {
        match self.y {
            0..=24 => BiomeType::Beach,
            25..=49 => BiomeType::Jungle,
            50..=74 => BiomeType::Alpine,
            _ => BiomeType::Volcanic,
        }
    }

    pub fn has_item(&self, item: ItemType) -> bool {
        self.items.get(&item).copied().unwrap_or(0) > 0
    }

    pub fn add_item(&mut self, item: ItemType, count: u32) {
        *self.items.entry(item).or_insert(0) += count;
    }

    pub fn use_item(&mut self, item: ItemType) -> bool {
        if let Some(count) = self.items.get_mut(&item) {
            if *count > 0 {
                *count -= 1;
                self.items_used += 1;
                return true;
            }
        }
        false
    }

    pub fn add_food(&mut self, food_id: u32) {
        self.foods.push(food_id);
    }

    pub fn use_food(&mut self, food_id: u32) -> bool {
        if let Some(pos) = self.foods.iter().position(|&f| f == food_id) {
            self.foods.remove(pos);
            if !self.foods_eaten.contains(&food_id) {
                self.foods_eaten.push(food_id);
            }
            return true;
        }
        false
    }

    pub fn has_effect(&self, effect_type: StatusEffectType) -> bool {
        self.status_effects.iter().any(|e| e.effect_type == effect_type)
    }

    pub fn add_effect(&mut self, effect_type: StatusEffectType, duration: u32) {
        // Remove existing effect of same type
        self.status_effects.retain(|e| e.effect_type != effect_type);
        self.status_effects.push(StatusEffect {
            effect_type,
            duration_remaining: duration,
        });
    }

    pub fn remove_effect(&mut self, effect_type: StatusEffectType) {
        self.status_effects.retain(|e| e.effect_type != effect_type);
    }

    pub fn tick_effects(&mut self) {
        for effect in &mut self.status_effects {
            if effect.duration_remaining > 0 {
                effect.duration_remaining -= 1;
            }
        }
        self.status_effects.retain(|e| e.duration_remaining > 0);
    }

    pub fn apply_stamina_damage(&mut self, current_damage: u32, max_damage: u32) {
        // Check for invulnerability
        if self.has_effect(StatusEffectType::Invulnerable) {
            return;
        }

        // Apply current stamina damage
        self.stamina_current = self.stamina_current.saturating_sub(current_damage);

        // Apply max stamina damage (permanent within run)
        self.stamina_max = self.stamina_max.saturating_sub(max_damage);

        // Cap current at max
        if self.stamina_current > self.stamina_max {
            self.stamina_current = self.stamina_max;
        }

        // Check for down
        if self.stamina_current == 0 {
            self.status = ClimberStatus::Downed;
            self.falls += 1;
        }
    }

    pub fn regenerate_stamina(&mut self, amount: u32) {
        if self.has_effect(StatusEffectType::Exhausted) {
            // Halved regen when exhausted
            self.stamina_current = (self.stamina_current + amount / 2).min(self.stamina_max);
        } else {
            self.stamina_current = (self.stamina_current + amount).min(self.stamina_max);
        }
    }

    pub fn revive(&mut self) {
        if self.status == ClimberStatus::Downed {
            self.status = ClimberStatus::Active;
            self.stamina_current = self.stamina_max / 2;
            self.revives_received += 1;
        }
    }

    pub fn disconnect(&mut self) {
        self.status = ClimberStatus::Disconnected;
    }

    pub fn reconnect(&mut self) {
        if self.status == ClimberStatus::Disconnected {
            self.status = ClimberStatus::Downed;
        }
    }

    pub fn inventory_count(&self) -> u32 {
        self.items.values().sum::<u32>() + self.foods.len() as u32
    }

    pub fn inventory_capacity(&self) -> u32 {
        10 // Base capacity
    }
}

impl Default for EquippedCosmetics {
    fn default() -> Self {
        Self {
            uniform: "uniform_green".to_string(),
            hat: None,
            backpack: "pack_standard".to_string(),
            accessory: None,
            rope_color: "rope_tan".to_string(),
        }
    }
}

// ============================================================================
// RUN STATE (a single mountain climb attempt)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Active,
    Summit,  // Success!
    Failed,  // All players dead
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedItem {
    pub item_type: ItemType,
    pub x: i32,
    pub y: i32,
    pub placed_by: i64,  // user_id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: u64,
    pub date: String,               // "YYYY-MM-DD" for daily seed
    pub seed: u64,

    pub climbers: HashMap<i64, ClimberState>,  // user_id -> state
    pub placed_items: Vec<PlacedItem>,         // Ropes, pitons, etc.

    pub status: RunStatus,
    pub start_time: u64,            // Unix timestamp
    pub elapsed_ticks: u64,

    // Mountain progress
    pub highest_reached: u32,       // Highest Y reached by any climber
    pub biomes_reached: HashSet<BiomeType>,
}

impl RunState {
    pub fn new(run_id: u64, date: String, seed: u64) -> Self {
        let mut biomes_reached = HashSet::new();
        biomes_reached.insert(BiomeType::Beach);

        Self {
            run_id,
            date,
            seed,
            climbers: HashMap::new(),
            placed_items: Vec::new(),
            status: RunStatus::Active,
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            elapsed_ticks: 0,
            highest_reached: 0,
            biomes_reached,
        }
    }

    pub fn add_climber(&mut self, user_id: i64, handle: String, cosmetics: EquippedCosmetics) {
        let climber = ClimberState::new(user_id, handle).with_cosmetics(cosmetics);
        self.climbers.insert(user_id, climber);
    }

    pub fn get_climber(&self, user_id: i64) -> Option<&ClimberState> {
        self.climbers.get(&user_id)
    }

    pub fn get_climber_mut(&mut self, user_id: i64) -> Option<&mut ClimberState> {
        self.climbers.get_mut(&user_id)
    }

    pub fn active_climbers(&self) -> impl Iterator<Item = &ClimberState> {
        self.climbers.values().filter(|c| c.is_active())
    }

    pub fn all_downed_or_dead(&self) -> bool {
        self.climbers.values()
            .filter(|c| c.status != ClimberStatus::Disconnected)
            .all(|c| c.is_downed() || c.is_dead())
    }

    pub fn any_at_summit(&self) -> bool {
        self.climbers.values().any(|c| c.y >= 100)
    }

    pub fn place_item(&mut self, item_type: ItemType, x: i32, y: i32, user_id: i64) {
        self.placed_items.push(PlacedItem {
            item_type,
            x,
            y,
            placed_by: user_id,
        });
    }

    pub fn get_items_at(&self, x: i32, y: i32) -> Vec<&PlacedItem> {
        self.placed_items.iter()
            .filter(|i| i.x == x && i.y == y)
            .collect()
    }

    pub fn tick(&mut self) {
        self.elapsed_ticks += 1;

        // Update highest reached
        for climber in self.climbers.values() {
            if climber.y as u32 > self.highest_reached {
                self.highest_reached = climber.y as u32;
                self.biomes_reached.insert(climber.current_biome());
            }
        }

        // Tick status effects
        for climber in self.climbers.values_mut() {
            climber.tick_effects();
        }

        // Check win/lose conditions
        if self.any_at_summit() {
            self.status = RunStatus::Summit;
        } else if self.all_downed_or_dead() {
            self.status = RunStatus::Failed;
        }
    }

    pub fn elapsed_seconds(&self) -> u64 {
        // Each tick is roughly 100ms
        self.elapsed_ticks / 10
    }

    pub fn elapsed_display(&self) -> String {
        let secs = self.elapsed_seconds();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

// ============================================================================
// PLAYER STATS (persistent across runs)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStats {
    pub user_id: i64,

    // Totals
    pub total_runs: u32,
    pub total_summits: u32,
    pub total_falls: u32,
    pub total_revives_given: u32,
    pub total_revives_received: u32,
    pub total_ropes_placed: u32,
    pub total_pitons_placed: u32,
    pub total_items_used: u32,
    pub total_luggage_opened: u32,

    // Bests
    pub fastest_summit_seconds: Option<u32>,
    pub highest_reached: u32,

    // Foods
    pub foods_tried: HashSet<u32>,  // Food IDs
    pub perfect_roasts: u32,
    pub smores_made: u32,
    pub food_poisoning_count: u32,

    // Badges earned
    pub badges: HashSet<String>,

    // Cosmetics unlocked
    pub cosmetics_unlocked: HashSet<String>,
    pub equipped_cosmetics: EquippedCosmetics,
}

impl PlayerStats {
    pub fn new(user_id: i64) -> Self {
        let mut cosmetics_unlocked = HashSet::new();
        // Default unlocked cosmetics
        cosmetics_unlocked.insert("uniform_green".to_string());
        cosmetics_unlocked.insert("hat_cap".to_string());
        cosmetics_unlocked.insert("pack_standard".to_string());
        cosmetics_unlocked.insert("rope_tan".to_string());

        Self {
            user_id,
            cosmetics_unlocked,
            ..Default::default()
        }
    }

    pub fn record_run(&mut self, run: &RunState, climber: &ClimberState) {
        self.total_runs += 1;

        if run.status == RunStatus::Summit {
            self.total_summits += 1;

            let elapsed = run.elapsed_seconds() as u32;
            if self.fastest_summit_seconds.is_none() || elapsed < self.fastest_summit_seconds.unwrap() {
                self.fastest_summit_seconds = Some(elapsed);
            }
        }

        // Record stats from climber
        self.total_falls += climber.falls;
        self.total_revives_given += climber.revives_given;
        self.total_revives_received += climber.revives_received;
        self.total_ropes_placed += climber.ropes_placed;
        self.total_pitons_placed += climber.pitons_placed;
        self.total_items_used += climber.items_used;

        // Track foods tried
        for food_id in &climber.foods_eaten {
            self.foods_tried.insert(*food_id);
        }

        // Update highest reached
        if climber.y as u32 > self.highest_reached {
            self.highest_reached = climber.y as u32;
        }
    }

    pub fn award_badge(&mut self, badge_id: &str) -> bool {
        self.badges.insert(badge_id.to_string())
    }

    pub fn has_badge(&self, badge_id: &str) -> bool {
        self.badges.contains(badge_id)
    }

    pub fn unlock_cosmetic(&mut self, cosmetic_id: &str) -> bool {
        self.cosmetics_unlocked.insert(cosmetic_id.to_string())
    }

    pub fn has_cosmetic(&self, cosmetic_id: &str) -> bool {
        self.cosmetics_unlocked.contains(cosmetic_id)
    }

    /// Check and award badges based on current stats
    pub fn check_badges(&mut self) -> Vec<String> {
        let mut newly_awarded = Vec::new();

        // Progression badges
        if self.total_summits >= 1 && self.award_badge("first_summit") {
            newly_awarded.push("first_summit".to_string());
        }
        if self.total_summits >= 10 && self.award_badge("veteran") {
            newly_awarded.push("veteran".to_string());
        }
        if self.total_summits >= 50 && self.award_badge("master") {
            newly_awarded.push("master".to_string());
        }

        // Skill badges
        if let Some(fastest) = self.fastest_summit_seconds {
            if fastest <= 900 && self.award_badge("speed_climber") { // 15 minutes
                newly_awarded.push("speed_climber".to_string());
            }
        }
        if self.total_revives_given >= 10 && self.award_badge("team_player") {
            newly_awarded.push("team_player".to_string());
        }
        if self.total_ropes_placed >= 100 && self.award_badge("trailblazer") {
            newly_awarded.push("trailblazer".to_string());
        }

        // Food badges
        if self.foods_tried.len() >= 15 && self.award_badge("adventurous") {
            newly_awarded.push("adventurous".to_string());
        }
        if self.foods_tried.len() >= 30 && self.award_badge("iron_stomach") {
            newly_awarded.push("iron_stomach".to_string());
        }
        if self.food_poisoning_count >= 5 && self.award_badge("survivor") {
            newly_awarded.push("survivor".to_string());
        }
        if self.perfect_roasts >= 20 && self.award_badge("marshmallow_master") {
            newly_awarded.push("marshmallow_master".to_string());
        }
        if self.smores_made >= 10 && self.award_badge("smore_connoisseur") {
            newly_awarded.push("smore_connoisseur".to_string());
        }

        // Discovery badges
        if self.total_luggage_opened >= 100 && self.award_badge("scavenger") {
            newly_awarded.push("scavenger".to_string());
        }

        newly_awarded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_climber_new() {
        let climber = ClimberState::new(1, "TestUser".to_string());
        assert_eq!(climber.stamina_current, 100);
        assert_eq!(climber.stamina_max, 100);
        assert!(climber.is_active());
        assert_eq!(climber.current_biome(), BiomeType::Beach);
    }

    #[test]
    fn test_climber_biome_progression() {
        let mut climber = ClimberState::new(1, "Test".to_string());

        climber.y = 0;
        assert_eq!(climber.current_biome(), BiomeType::Beach);

        climber.y = 25;
        assert_eq!(climber.current_biome(), BiomeType::Jungle);

        climber.y = 50;
        assert_eq!(climber.current_biome(), BiomeType::Alpine);

        climber.y = 75;
        assert_eq!(climber.current_biome(), BiomeType::Volcanic);
    }

    #[test]
    fn test_climber_stamina_damage() {
        let mut climber = ClimberState::new(1, "Test".to_string());

        climber.apply_stamina_damage(30, 10);
        assert_eq!(climber.stamina_current, 70);
        assert_eq!(climber.stamina_max, 90);
        assert!(climber.is_active());

        climber.apply_stamina_damage(70, 0);
        assert_eq!(climber.stamina_current, 0);
        assert!(climber.is_downed());
        assert_eq!(climber.falls, 1);
    }

    #[test]
    fn test_climber_invulnerability() {
        let mut climber = ClimberState::new(1, "Test".to_string());
        climber.add_effect(StatusEffectType::Invulnerable, 10);

        climber.apply_stamina_damage(50, 20);
        assert_eq!(climber.stamina_current, 100);
        assert_eq!(climber.stamina_max, 100);
    }

    #[test]
    fn test_climber_items() {
        let mut climber = ClimberState::new(1, "Test".to_string());

        assert!(!climber.has_item(ItemType::Rope));

        climber.add_item(ItemType::Rope, 3);
        assert!(climber.has_item(ItemType::Rope));
        assert_eq!(climber.items.get(&ItemType::Rope), Some(&3));

        assert!(climber.use_item(ItemType::Rope));
        assert_eq!(climber.items.get(&ItemType::Rope), Some(&2));
        assert_eq!(climber.items_used, 1);
    }

    #[test]
    fn test_climber_revive() {
        let mut climber = ClimberState::new(1, "Test".to_string());
        climber.stamina_max = 80;
        climber.status = ClimberStatus::Downed;

        climber.revive();

        assert!(climber.is_active());
        assert_eq!(climber.stamina_current, 40); // 50% of max
        assert_eq!(climber.revives_received, 1);
    }

    #[test]
    fn test_run_state() {
        let mut run = RunState::new(1, "2026-01-30".to_string(), 12345);

        run.add_climber(1, "Player1".to_string(), EquippedCosmetics::default());
        run.add_climber(2, "Player2".to_string(), EquippedCosmetics::default());

        assert_eq!(run.climbers.len(), 2);
        assert!(!run.all_downed_or_dead());
        assert!(!run.any_at_summit());
    }

    #[test]
    fn test_run_summit_detection() {
        let mut run = RunState::new(1, "2026-01-30".to_string(), 12345);
        run.add_climber(1, "Player1".to_string(), EquippedCosmetics::default());

        if let Some(climber) = run.get_climber_mut(1) {
            climber.y = 100;
        }

        run.tick();
        assert_eq!(run.status, RunStatus::Summit);
    }

    #[test]
    fn test_run_fail_detection() {
        let mut run = RunState::new(1, "2026-01-30".to_string(), 12345);
        run.add_climber(1, "Player1".to_string(), EquippedCosmetics::default());

        if let Some(climber) = run.get_climber_mut(1) {
            climber.status = ClimberStatus::Dead;
        }

        run.tick();
        assert_eq!(run.status, RunStatus::Failed);
    }

    #[test]
    fn test_player_stats() {
        let mut stats = PlayerStats::new(1);

        assert!(stats.has_cosmetic("uniform_green"));
        assert!(!stats.has_badge("first_summit"));

        stats.total_summits = 1;
        let new_badges = stats.check_badges();
        assert!(new_badges.contains(&"first_summit".to_string()));
        assert!(stats.has_badge("first_summit"));
    }

    #[test]
    fn test_status_effects() {
        let mut climber = ClimberState::new(1, "Test".to_string());

        climber.add_effect(StatusEffectType::Cold, 5);
        assert!(climber.has_effect(StatusEffectType::Cold));

        for _ in 0..5 {
            climber.tick_effects();
        }

        assert!(!climber.has_effect(StatusEffectType::Cold));
    }
}
