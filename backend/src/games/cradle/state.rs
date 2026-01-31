//! Game state for Cradle
//!
//! Stores all persistent player data including progression,
//! resources, techniques, and prestige information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::data::{TierLevel, get_path};

/// Main game state - serialized to database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // Character identity
    pub name: String,
    pub created_at: String,

    // Core progression
    pub tier: TierLevel,
    pub tier_progress: f64,        // 0.0 - 1.0 progress to next tier
    pub madra: u64,                // Current madra pool
    pub max_madra: u64,            // Maximum madra capacity
    pub insight: u32,              // Spiritual insight

    // Path and techniques
    pub primary_path: Option<String>,    // Path key
    pub secondary_path: Option<String>,  // Optional second path
    pub path_progress: HashMap<String, PathProgress>,
    pub technique_levels: HashMap<String, u32>,  // technique_key -> level
    pub active_techniques: Vec<String>,  // Currently equipped techniques

    // Resources
    pub spirit_stones: u64,        // Basic currency
    pub elixirs: u32,              // Consumable boost items
    pub treasures: Vec<String>,    // Special items

    // Character stats
    pub stats: CharacterStats,

    // Prestige/meta-progression
    pub prestige: PrestigeState,

    // Mentor
    pub current_mentor: Option<String>,
    pub mentor_hints_received: u32,

    // Idle/offline tracking
    pub last_tick: i64,            // Unix timestamp of last tick
    pub total_ticks: u64,          // Lifetime ticks for statistics
    pub last_login: String,        // ISO date string

    // Game flags
    pub trials_completed: Vec<TierLevel>,  // Which advancement trials completed
    pub unlocked_features: Vec<String>,
    pub tutorial_completed: bool,
    pub game_over: bool,
    pub ascended: bool,             // Has completed the game at least once

    // Message display
    #[serde(default)]
    pub last_message: Option<String>,
}

/// Progress on a specific path
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathProgress {
    pub affinity: f64,             // 0.0 - 1.0 natural talent
    pub mastery: u64,              // Experience with this path
    pub techniques_unlocked: Vec<String>,
    pub resonance: f64,            // How well it meshes with your other paths
}

/// Character combat/advancement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub power: u64,                // Attack strength
    pub defense: u64,              // Damage reduction
    pub speed: u64,                // Action speed
    pub madra_regen: u64,          // Madra per tick
    pub cycling_speed: f64,        // Advancement rate multiplier

    // Combat tracking
    pub battles_won: u64,
    pub battles_lost: u64,
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub enemies_defeated: u64,

    // High scores
    pub peak_power: u64,
    pub highest_tier_reached: TierLevel,
    pub fastest_advancement: Option<u64>,  // Ticks to reach Monarch
}

impl Default for CharacterStats {
    fn default() -> Self {
        Self {
            power: 10,
            defense: 10,
            speed: 10,
            madra_regen: 1,
            cycling_speed: 1.0,
            battles_won: 0,
            battles_lost: 0,
            damage_dealt: 0,
            damage_taken: 0,
            enemies_defeated: 0,
            peak_power: 10,
            highest_tier_reached: TierLevel::Unsouled,
            fastest_advancement: None,
        }
    }
}

/// Prestige/meta-progression state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrestigeState {
    // Prestige currencies
    pub ascension_points: u64,      // Earned on prestige, spent on permanent bonuses
    pub transcendence_shards: u32,  // Meta-prestige currency

    // Permanent bonuses (persist through prestige)
    pub starting_tier_bonus: u8,    // Start at higher tier
    pub madra_multiplier: f64,      // All madra gains multiplied
    pub insight_multiplier: f64,    // All insight gains multiplied
    pub spirit_stone_multiplier: f64, // Resource gains
    pub unlock_speed_bonus: f64,    // Faster unlocks

    // Meta-progression unlocks
    pub paths_unlocked: Vec<String>,  // Extra paths available from start
    pub techniques_unlocked: Vec<String>, // Start with these techniques
    pub mentors_unlocked: Vec<String>,  // Extra mentor access

    // Statistics
    pub total_prestiges: u32,
    pub total_ascension_points_earned: u64,
    pub highest_tier_ever: TierLevel,
    pub total_play_time_ticks: u64,

    // Respec tracking
    pub respecs_used: u32,
    pub respec_cost_multiplier: f64,  // Increases with each respec
}

impl Default for PrestigeState {
    fn default() -> Self {
        Self {
            ascension_points: 0,
            transcendence_shards: 0,
            starting_tier_bonus: 0,
            madra_multiplier: 1.0,
            insight_multiplier: 1.0,
            spirit_stone_multiplier: 1.0,
            unlock_speed_bonus: 1.0,
            paths_unlocked: Vec::new(),
            techniques_unlocked: Vec::new(),
            mentors_unlocked: Vec::new(),
            total_prestiges: 0,
            total_ascension_points_earned: 0,
            highest_tier_ever: TierLevel::Unsouled,
            total_play_time_ticks: 0,
            respecs_used: 0,
            respec_cost_multiplier: 1.0,
        }
    }
}

impl GameState {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();

        Self {
            name,
            created_at: now.to_rfc3339(),

            tier: TierLevel::Unsouled,
            tier_progress: 0.0,
            madra: 0,
            max_madra: 100,
            insight: 0,

            primary_path: None,
            secondary_path: None,
            path_progress: HashMap::new(),
            technique_levels: HashMap::new(),
            active_techniques: Vec::new(),

            spirit_stones: 100,
            elixirs: 0,
            treasures: Vec::new(),

            stats: CharacterStats::default(),
            prestige: PrestigeState::default(),

            current_mentor: Some("elder_wei".to_string()),
            mentor_hints_received: 0,

            last_tick: now.timestamp(),
            total_ticks: 0,
            last_login: now.format("%Y-%m-%d").to_string(),

            trials_completed: Vec::new(),
            unlocked_features: Vec::new(),
            tutorial_completed: false,
            game_over: false,
            ascended: false,

            last_message: None,
        }
    }

    /// Calculate total combat power
    pub fn total_power(&self) -> u64 {
        let tier_mult = super::data::get_tier(self.tier)
            .map(|t| t.power_multiplier)
            .unwrap_or(1.0);

        let path_mult = self.primary_path.as_ref()
            .and_then(|p| get_path(p))
            .map(|p| p.power_focus)
            .unwrap_or(1.0);

        ((self.stats.power as f64) * tier_mult * path_mult * self.prestige.madra_multiplier) as u64
    }

    /// Calculate total defense
    pub fn total_defense(&self) -> u64 {
        let tier_mult = super::data::get_tier(self.tier)
            .map(|t| t.power_multiplier * 0.5)
            .unwrap_or(1.0);

        let path_mult = self.primary_path.as_ref()
            .and_then(|p| get_path(p))
            .map(|p| p.defense_focus)
            .unwrap_or(1.0);

        ((self.stats.defense as f64) * tier_mult * path_mult) as u64
    }

    /// Calculate madra regen per tick
    pub fn madra_per_tick(&self) -> u64 {
        let tier_bonus = super::data::get_tier(self.tier)
            .map(|t| t.madra_regen_bonus)
            .unwrap_or(1.0);

        let path_bonus = self.primary_path.as_ref()
            .and_then(|p| get_path(p))
            .map(|p| p.regen_focus)
            .unwrap_or(1.0);

        ((self.stats.madra_regen as f64) * tier_bonus * path_bonus * self.prestige.madra_multiplier) as u64
    }

    /// Check if player can advance to next tier
    pub fn can_advance_tier(&self) -> bool {
        if let Some(next_tier) = self.tier.next() {
            if let Some(tier_data) = super::data::get_tier(next_tier) {
                // Check madra requirement
                if self.madra < tier_data.madra_requirement {
                    return false;
                }
                // Check insight requirement
                if self.insight < tier_data.insight_requirement {
                    return false;
                }
                // Check if trial required and not completed
                if tier_data.trial_required && !self.trials_completed.contains(&next_tier) {
                    return false;
                }
                return true;
            }
        }
        false
    }

    /// Check if player has hit a plateau (wrong build)
    pub fn is_plateaued(&self) -> bool {
        // Check if current path has a max tier lower than current
        if let Some(ref path_key) = self.primary_path {
            if let Some(path) = get_path(path_key) {
                if self.tier >= path.max_tier_natural {
                    return true;
                }
            }
        }

        // Check for incompatible path combinations
        if let (Some(ref primary), Some(ref secondary)) = (&self.primary_path, &self.secondary_path) {
            if let Some(primary_path) = get_path(primary) {
                if primary_path.incompatible_with.contains(&secondary.as_str()) {
                    return true;
                }
            }
        }

        false
    }

    /// Calculate respec cost
    pub fn respec_cost(&self) -> u64 {
        let base_cost = 1000u64;
        let tier_multiplier = (self.tier as u8 + 1) as u64 * 10;
        let respec_multiplier = (self.prestige.respec_cost_multiplier * 100.0) as u64;

        (base_cost * tier_multiplier * respec_multiplier) / 100
    }

    /// Perform a respec (reset path choices)
    pub fn respec(&mut self) -> Result<(), &'static str> {
        let cost = self.respec_cost();
        if self.spirit_stones < cost {
            return Err("Insufficient spirit stones for respec");
        }

        self.spirit_stones -= cost;
        self.prestige.respecs_used += 1;
        self.prestige.respec_cost_multiplier *= 1.5;  // Each respec costs more

        // Reset path progress but keep tier
        self.primary_path = None;
        self.secondary_path = None;
        self.path_progress.clear();
        self.technique_levels.clear();
        self.active_techniques.clear();

        // Reset to 50% of tier progress
        self.tier_progress *= 0.5;

        Ok(())
    }

    /// Calculate ascension points earned if prestige now
    pub fn potential_ascension_points(&self) -> u64 {
        let tier_value = self.tier as u64;
        let progress_bonus = (self.tier_progress * 100.0) as u64;
        let insight_bonus = (self.insight / 10) as u64;

        tier_value * 100 + progress_bonus + insight_bonus
    }

    /// Perform prestige (soft reset with permanent bonuses)
    pub fn prestige(&mut self) -> u64 {
        let points = self.potential_ascension_points();

        // Update prestige state
        self.prestige.ascension_points += points;
        self.prestige.total_ascension_points_earned += points;
        self.prestige.total_prestiges += 1;
        self.prestige.total_play_time_ticks += self.total_ticks;

        if self.tier > self.prestige.highest_tier_ever {
            self.prestige.highest_tier_ever = self.tier;
        }

        // Reset game state (but keep prestige bonuses)
        let starting_tier = TierLevel::from_u8(self.prestige.starting_tier_bonus)
            .unwrap_or(TierLevel::Unsouled);

        self.tier = starting_tier;
        self.tier_progress = 0.0;
        self.madra = 0;
        self.max_madra = 100 + (self.prestige.ascension_points / 10) as u64;
        self.insight = 0;

        self.primary_path = None;
        self.secondary_path = None;
        self.path_progress.clear();
        self.technique_levels.clear();
        self.active_techniques.clear();

        self.spirit_stones = 100 + (self.prestige.ascension_points / 5) as u64;
        self.elixirs = 0;
        self.treasures.clear();

        self.stats = CharacterStats::default();
        self.stats.cycling_speed = self.prestige.unlock_speed_bonus;

        self.current_mentor = Some("elder_wei".to_string());
        self.mentor_hints_received = 0;

        self.total_ticks = 0;
        self.trials_completed.clear();
        self.unlocked_features.clear();
        self.game_over = false;
        self.ascended = true;

        points
    }

    /// Spend ascension points on permanent upgrades
    pub fn buy_prestige_upgrade(&mut self, upgrade: &str) -> Result<(), &'static str> {
        let (cost, apply): (u64, fn(&mut PrestigeState)) = match upgrade {
            "madra_boost" => (100, |p| p.madra_multiplier += 0.1),
            "insight_boost" => (100, |p| p.insight_multiplier += 0.1),
            "stone_boost" => (150, |p| p.spirit_stone_multiplier += 0.1),
            "speed_boost" => (200, |p| p.unlock_speed_bonus += 0.1),
            "starting_tier" => {
                let cost = (self.prestige.starting_tier_bonus as u64 + 1) * 500;
                if self.prestige.starting_tier_bonus >= 5 {
                    return Err("Maximum starting tier reached");
                }
                (cost, |p| p.starting_tier_bonus += 1)
            },
            _ => return Err("Unknown upgrade"),
        };

        if self.prestige.ascension_points < cost {
            return Err("Insufficient ascension points");
        }

        self.prestige.ascension_points -= cost;
        apply(&mut self.prestige);
        Ok(())
    }

    /// Get compatible paths based on current selection
    pub fn get_compatible_paths(&self) -> Vec<&'static str> {
        if let Some(ref primary) = self.primary_path {
            if let Some(path) = get_path(primary) {
                return path.compatible_with.to_vec();
            }
        }
        // If no primary, all paths are available
        super::data::PATHS.iter().map(|p| p.key).collect()
    }

    /// Get current mentor based on tier
    pub fn get_appropriate_mentor(&self) -> Option<&'static super::data::Mentor> {
        super::data::get_mentors_for_tier(self.tier).first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new("TestPlayer".to_string());
        assert_eq!(state.name, "TestPlayer");
        assert_eq!(state.tier, TierLevel::Unsouled);
        assert_eq!(state.spirit_stones, 100);
        assert!(!state.game_over);
    }

    #[test]
    fn test_total_power_calculation() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Gold;
        state.stats.power = 100;

        let power = state.total_power();
        // Gold has 15x multiplier, so 100 * 15 = 1500 base
        assert!(power >= 1000);
    }

    #[test]
    fn test_respec_cost_increases() {
        let mut state = GameState::new("Test".to_string());
        state.spirit_stones = 1_000_000;

        let cost1 = state.respec_cost();
        state.respec().unwrap();
        let cost2 = state.respec_cost();

        assert!(cost2 > cost1);
    }

    #[test]
    fn test_plateau_detection() {
        let mut state = GameState::new("Test".to_string());
        state.primary_path = Some("pure_force".to_string());
        state.tier = TierLevel::Overlord;  // Above pure_force max (Lord)

        assert!(state.is_plateaued());
    }

    #[test]
    fn test_prestige_reset() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Gold;
        state.tier_progress = 0.5;
        state.madra = 50000;
        state.insight = 100;

        let points = state.prestige();

        assert!(points > 0);
        assert_eq!(state.tier, TierLevel::Unsouled);
        assert_eq!(state.prestige.total_prestiges, 1);
        assert!(state.ascended);
    }

    #[test]
    fn test_state_serialization() {
        let state = GameState::new("TestPlayer".to_string());
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.name, restored.name);
        assert_eq!(state.tier, restored.tier);
        assert_eq!(state.spirit_stones, restored.spirit_stones);
    }

    #[test]
    fn test_prestige_upgrade_purchase() {
        let mut state = GameState::new("Test".to_string());
        state.prestige.ascension_points = 500;

        let initial_mult = state.prestige.madra_multiplier;
        state.buy_prestige_upgrade("madra_boost").unwrap();

        assert!(state.prestige.madra_multiplier > initial_mult);
        assert_eq!(state.prestige.ascension_points, 400);
    }

    #[test]
    fn test_madra_per_tick() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Gold;
        state.stats.madra_regen = 10;

        let regen = state.madra_per_tick();
        assert!(regen > 10);  // Should be boosted by tier
    }
}
