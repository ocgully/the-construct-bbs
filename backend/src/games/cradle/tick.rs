//! Tick system for Cradle
//!
//! Processes idle progression and offline catchup.
//! Implements per-game background processing.

use super::state::GameState;
use super::economy::{calculate_tick_resources, apply_resource_gains, calculate_tier_progress, ResourceGain};
use super::data::{TierLevel, get_tier};

/// Maximum ticks to process in catchup (prevent lag on very long offline)
pub const MAX_CATCHUP_TICKS: u32 = 10000;

/// Ticks per in-game "day" for event purposes
pub const TICKS_PER_DAY: u32 = 100;

/// Result of processing ticks
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    pub ticks_processed: u32,
    pub total_madra_gained: u64,
    pub total_stones_gained: u64,
    pub total_insight_gained: u32,
    pub tier_advanced: Option<TierLevel>,
    pub plateau_warning: bool,
    pub event_messages: Vec<String>,
}

/// Process a single game tick
pub fn process_tick(state: &mut GameState) -> TickResult {
    let mut result = TickResult::default();
    result.ticks_processed = 1;

    // Calculate and apply resource gains
    let gains = calculate_tick_resources(state);
    apply_resource_gains(state, &gains);

    result.total_madra_gained = gains.madra;
    result.total_stones_gained = gains.spirit_stones;
    result.total_insight_gained = gains.insight;

    // Update tier progress
    state.tier_progress = calculate_tier_progress(state);

    // Check for automatic tier advancement (if no trial required)
    if state.can_advance_tier() {
        if let Some(next_tier) = state.tier.next() {
            if let Some(tier_data) = get_tier(next_tier) {
                if !tier_data.trial_required {
                    // Auto-advance
                    state.tier = next_tier;
                    state.tier_progress = 0.0;
                    result.tier_advanced = Some(next_tier);
                    result.event_messages.push(format!(
                        "Breakthrough! You have reached {}!",
                        next_tier.name()
                    ));
                }
            }
        }
    }

    // Check for plateau
    if state.is_plateaued() {
        result.plateau_warning = true;
    }

    // Update tracking
    state.last_tick = chrono::Utc::now().timestamp();
    state.total_ticks += 1;
    state.prestige.total_play_time_ticks += 1;

    // Random events (roughly once per "day")
    if state.total_ticks % (TICKS_PER_DAY as u64) == 0 {
        if let Some(event) = generate_daily_event(state) {
            result.event_messages.push(event);
        }
    }

    // Update stats tracking
    let total_power = state.total_power();
    if total_power > state.stats.peak_power {
        state.stats.peak_power = total_power;
    }

    result
}

/// Process multiple ticks (offline catchup)
pub fn process_catchup_ticks(state: &mut GameState, ticks: u32) -> TickResult {
    let mut combined = TickResult::default();
    let ticks_to_process = ticks.min(MAX_CATCHUP_TICKS);

    // Optimization: for large tick counts, use batch calculation
    if ticks_to_process > 100 {
        // Calculate gains for one tick
        let single_gains = calculate_tick_resources(state);

        // Scale up (with diminishing returns for very long offline periods)
        let efficiency = calculate_offline_efficiency(ticks_to_process);
        let scaled_madra = (single_gains.madra as f64 * ticks_to_process as f64 * efficiency) as u64;
        let scaled_stones = (single_gains.spirit_stones as f64 * ticks_to_process as f64 * efficiency) as u64;
        let scaled_insight = (single_gains.insight as f64 * ticks_to_process as f64 * efficiency) as u32;

        let batch_gains = ResourceGain {
            madra: scaled_madra,
            spirit_stones: scaled_stones,
            insight: scaled_insight,
            technique_xp: single_gains.technique_xp * ticks_to_process as u64,
        };

        apply_resource_gains(state, &batch_gains);

        combined.ticks_processed = ticks_to_process;
        combined.total_madra_gained = batch_gains.madra;
        combined.total_stones_gained = batch_gains.spirit_stones;
        combined.total_insight_gained = batch_gains.insight;

        // Update tracking
        state.total_ticks += ticks_to_process as u64;
        state.prestige.total_play_time_ticks += ticks_to_process as u64;
        state.last_tick = chrono::Utc::now().timestamp();

        // Check for tier advancement
        state.tier_progress = calculate_tier_progress(state);

        // Check for automatic tier advances during catchup
        let mut advances_count = 0;
        while state.can_advance_tier() && advances_count < 5 {
            if let Some(next_tier) = state.tier.next() {
                if let Some(tier_data) = get_tier(next_tier) {
                    if !tier_data.trial_required {
                        state.tier = next_tier;
                        state.tier_progress = calculate_tier_progress(state);
                        combined.tier_advanced = Some(next_tier);
                        combined.event_messages.push(format!(
                            "While cycling, you achieved {}!",
                            next_tier.name()
                        ));
                        advances_count += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Add catchup summary message
        combined.event_messages.push(format!(
            "Offline cycling: +{} madra, +{} spirit stones, +{} insight",
            format_large_number(combined.total_madra_gained),
            format_large_number(combined.total_stones_gained),
            combined.total_insight_gained
        ));

    } else {
        // Small tick count: process individually
        for _ in 0..ticks_to_process {
            let tick_result = process_tick(state);

            combined.ticks_processed += 1;
            combined.total_madra_gained += tick_result.total_madra_gained;
            combined.total_stones_gained += tick_result.total_stones_gained;
            combined.total_insight_gained += tick_result.total_insight_gained;

            if tick_result.tier_advanced.is_some() {
                combined.tier_advanced = tick_result.tier_advanced;
            }
            if tick_result.plateau_warning {
                combined.plateau_warning = true;
            }
            combined.event_messages.extend(tick_result.event_messages);
        }
    }

    // Check for plateau after catchup
    if state.is_plateaued() {
        combined.plateau_warning = true;
        combined.event_messages.push(
            "WARNING: Your cultivation has reached a plateau. Consult your mentor or consider a respec.".to_string()
        );
    }

    combined
}

/// Calculate offline efficiency (diminishing returns)
fn calculate_offline_efficiency(ticks: u32) -> f64 {
    // First 100 ticks: 100% efficiency
    // 100-1000 ticks: 80% efficiency
    // 1000-5000 ticks: 50% efficiency
    // 5000+ ticks: 30% efficiency
    if ticks <= 100 {
        1.0
    } else if ticks <= 1000 {
        0.8
    } else if ticks <= 5000 {
        0.5
    } else {
        0.3
    }
}

/// Calculate ticks since last update
pub fn ticks_since_last_update(state: &GameState) -> u32 {
    let now = chrono::Utc::now().timestamp();
    let elapsed = now - state.last_tick;

    // Each tick represents ~1 minute of real time
    // But cap at MAX_CATCHUP_TICKS
    if elapsed < 0 {
        return 0;
    }

    let ticks = (elapsed / 60) as u32;  // One tick per minute
    ticks.min(MAX_CATCHUP_TICKS)
}

/// Generate daily random event
fn generate_daily_event(state: &GameState) -> Option<String> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Event chance based on tier
    let event_chance = 30 + (state.tier as u32 * 5);
    if rng.gen_range(0..100) > event_chance {
        return None;
    }

    let events = match state.tier {
        TierLevel::Unsouled | TierLevel::Copper => vec![
            "You found a spirit stone while cycling!",
            "A fellow cultivator shared wisdom with you.",
            "You glimpsed the Way during meditation.",
        ],
        TierLevel::Iron | TierLevel::Jade => vec![
            "You discovered a hidden cultivation spot.",
            "Your body trembles with awakening power.",
            "You sensed the flow of vital aura.",
        ],
        TierLevel::Gold | TierLevel::Lord => vec![
            "Soulfire flickered at the edge of your consciousness.",
            "You perceived the true nature of your Path.",
            "A moment of clarity accelerated your advancement.",
        ],
        TierLevel::Overlord | TierLevel::Sage | TierLevel::Herald => vec![
            "An Icon manifested briefly overhead.",
            "The world itself bent to your authority.",
            "You touched the origin of your Path.",
        ],
        TierLevel::Monarch | TierLevel::Dreadgod => vec![
            "Reality trembles at your presence.",
            "Lesser beings instinctively bow as you pass.",
            "The Abidan have noticed your ascension.",
        ],
        _ => vec![
            "The cosmos itself acknowledges you.",
            "Time and space are merely suggestions to you now.",
            "Existence whispers secrets beyond mortal comprehension.",
        ],
    };

    let event = events[rng.gen_range(0..events.len())];

    // Apply bonus for some events
    Some(event.to_string())
}

/// Format large numbers with K/M/B suffixes
fn format_large_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tick() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Copper;
        state.stats.madra_regen = 10;
        state.primary_path = Some("blackflame".to_string());

        let initial_madra = state.madra;
        let result = process_tick(&mut state);

        assert_eq!(result.ticks_processed, 1);
        assert!(state.madra > initial_madra);
        assert!(result.total_madra_gained > 0);
    }

    #[test]
    fn test_catchup_ticks() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Copper;
        state.stats.madra_regen = 10;

        let initial_tick = state.total_ticks;
        let result = process_catchup_ticks(&mut state, 100);

        assert_eq!(result.ticks_processed, 100);
        assert_eq!(state.total_ticks, initial_tick + 100);
    }

    #[test]
    fn test_catchup_max_limit() {
        let mut state = GameState::new("Test".to_string());

        let result = process_catchup_ticks(&mut state, 100000);  // Way over limit

        assert!(result.ticks_processed <= MAX_CATCHUP_TICKS);
    }

    #[test]
    fn test_offline_efficiency() {
        assert_eq!(calculate_offline_efficiency(50), 1.0);
        assert_eq!(calculate_offline_efficiency(500), 0.8);
        assert_eq!(calculate_offline_efficiency(2000), 0.5);
        assert_eq!(calculate_offline_efficiency(10000), 0.3);
    }

    #[test]
    fn test_auto_tier_advance() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Unsouled;
        state.madra = 200;
        state.insight = 10;

        // Process enough ticks to potentially advance
        let _result = process_tick(&mut state);

        // Copper doesn't require trial, so should auto-advance
        if state.madra >= 100 && state.insight >= 1 {
            assert_eq!(state.tier, TierLevel::Copper);
        }
    }

    #[test]
    fn test_plateau_detection_during_tick() {
        let mut state = GameState::new("Test".to_string());
        state.primary_path = Some("pure_force".to_string());
        state.tier = TierLevel::Overlord;  // Above pure_force max (Lord)

        let result = process_tick(&mut state);
        assert!(result.plateau_warning);
    }

    #[test]
    fn test_format_large_number() {
        assert_eq!(format_large_number(500), "500");
        assert_eq!(format_large_number(5000), "5.0K");
        assert_eq!(format_large_number(5_000_000), "5.0M");
        assert_eq!(format_large_number(5_000_000_000), "5.0B");
    }
}
