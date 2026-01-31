//! Events and trials for Cradle
//!
//! Handles advancement trials, mentor interactions, and random events.

use super::state::GameState;
use super::data::{TierLevel, get_mentors_for_tier, Mentor};
use rand::Rng;

/// Advancement trial types
#[derive(Debug, Clone, PartialEq)]
pub enum TrialType {
    Meditation,     // Focus test
    Combat,         // Fight a spirit beast
    Revelation,     // Answer a deep question
    Sacrifice,      // Give up something
    Endurance,      // Survive damage
}

/// Trial state for advancement
#[derive(Debug, Clone, PartialEq)]
pub struct TrialState {
    pub trial_type: TrialType,
    pub target_tier: TierLevel,
    pub stage: u8,           // Current stage of the trial
    pub max_stages: u8,      // Total stages
    pub hp_remaining: u32,   // For combat trials
    pub enemy_hp: u32,       // For combat trials
    pub choices_made: Vec<String>,  // For revelation trials
    pub success_count: u32,  // Stages passed
}

impl TrialState {
    pub fn new(target_tier: TierLevel) -> Self {
        let (trial_type, stages) = match target_tier {
            TierLevel::Jade => (TrialType::Meditation, 3),
            TierLevel::Gold => (TrialType::Combat, 5),
            TierLevel::Lord => (TrialType::Revelation, 3),
            TierLevel::Overlord => (TrialType::Sacrifice, 3),
            TierLevel::Sage => (TrialType::Revelation, 5),
            TierLevel::Herald => (TrialType::Combat, 7),
            TierLevel::Monarch => (TrialType::Endurance, 5),
            _ => (TrialType::Meditation, 3),
        };

        Self {
            trial_type,
            target_tier,
            stage: 1,
            max_stages: stages,
            hp_remaining: 100,
            enemy_hp: match target_tier {
                TierLevel::Gold => 50,
                TierLevel::Herald => 200,
                _ => 0,
            },
            choices_made: Vec::new(),
            success_count: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.stage > self.max_stages
    }

    pub fn passed(&self) -> bool {
        self.success_count >= (self.max_stages / 2 + 1) as u32
    }
}

/// Trial challenge prompts
pub fn get_trial_prompt(trial: &TrialState) -> (&'static str, &'static [&'static str]) {
    match trial.trial_type {
        TrialType::Meditation => {
            static PROMPTS: [(&str, &[&str]); 3] = [
                ("Your mind wanders. Focus on:", &["Your breathing", "The void", "Your path"]),
                ("Distractions arise. You:", &["Push through", "Accept them", "Channel them"]),
                ("You glimpse truth. You:", &["Grasp it", "Let it pass", "Question it"]),
            ];
            let idx = (trial.stage - 1).min(2) as usize;
            PROMPTS[idx]
        }
        TrialType::Combat => {
            static PROMPTS: [(&str, &[&str]); 5] = [
                ("A spirit beast attacks! You:", &["Strike first", "Defend", "Evade"]),
                ("It counters! You:", &["Press attack", "Fall back", "Use technique"]),
                ("An opening appears! You:", &["Strike now", "Wait for better", "Feint"]),
                ("You're wounded! You:", &["Fight on", "Retreat", "Sacrifice to win"]),
                ("Final clash! You:", &["All-out attack", "Perfect defense", "Combined assault"]),
            ];
            let idx = ((trial.stage - 1) as usize).min(PROMPTS.len() - 1);
            PROMPTS[idx]
        }
        TrialType::Revelation => {
            static PROMPTS: [(&str, &[&str]); 5] = [
                ("What is the core of your Path?", &["Power", "Truth", "Balance"]),
                ("What would you sacrifice for advancement?", &["Everything", "Only what's necessary", "Nothing"]),
                ("What is your deepest truth?", &["I am strength", "I seek understanding", "I transcend limits"]),
                ("What do you fear most?", &["Weakness", "Stagnation", "Loss"]),
                ("What drives you forward?", &["Ambition", "Duty", "Love"]),
            ];
            let idx = ((trial.stage - 1) as usize).min(PROMPTS.len() - 1);
            PROMPTS[idx]
        }
        TrialType::Sacrifice => {
            static PROMPTS: [(&str, &[&str]); 3] = [
                ("Advancement demands sacrifice. Offer:", &["Spirit stones", "Treasures", "Nothing"]),
                ("Greater sacrifice required:", &["Path mastery", "Techniques", "Stats"]),
                ("The final price:", &["Accept", "Refuse", "Bargain"]),
            ];
            let idx = ((trial.stage - 1) as usize).min(PROMPTS.len() - 1);
            PROMPTS[idx]
        }
        TrialType::Endurance => {
            static PROMPTS: [(&str, &[&str]); 5] = [
                ("Pain washes over you. You:", &["Endure silently", "Channel it", "Scream"]),
                ("Your soul cracks. You:", &["Hold firm", "Adapt", "Break and reform"]),
                ("Reality tears at you. You:", &["Stand tall", "Bend but don't break", "Transform"]),
                ("The void calls. You:", &["Resist", "Listen", "Embrace"]),
                ("Final test of will:", &["Absolute resolve", "Flexible strength", "Transcend pain"]),
            ];
            let idx = ((trial.stage - 1) as usize).min(PROMPTS.len() - 1);
            PROMPTS[idx]
        }
    }
}

/// Process a trial choice
pub fn process_trial_choice(trial: &mut TrialState, choice_index: usize, state: &GameState) -> (bool, &'static str) {
    let mut rng = rand::thread_rng();

    // Calculate success based on stats and choice
    let base_chance = match trial.trial_type {
        TrialType::Meditation => state.insight as f64 / 100.0,
        TrialType::Combat => state.total_power() as f64 / 10000.0,
        TrialType::Revelation => (state.prestige.total_prestiges as f64 * 0.1) + 0.3,
        TrialType::Sacrifice => 0.7, // Mostly automatic
        TrialType::Endurance => state.total_defense() as f64 / 5000.0,
    };

    // Choice bonuses
    let choice_bonus = match choice_index {
        0 => 0.2,  // Aggressive/Direct
        1 => 0.1,  // Balanced
        2 => 0.0,  // Passive/Careful
        _ => 0.0,
    };

    let success_chance = (base_chance + choice_bonus).min(0.9);
    let succeeded = rng.gen_bool(success_chance);

    let message = if succeeded {
        trial.success_count += 1;
        match trial.trial_type {
            TrialType::Meditation => "Your focus holds true.",
            TrialType::Combat => "Your strike lands!",
            TrialType::Revelation => "The truth resonates within you.",
            TrialType::Sacrifice => "Your offering is accepted.",
            TrialType::Endurance => "You endure!",
        }
    } else {
        match trial.trial_type {
            TrialType::Meditation => "Your concentration wavers.",
            TrialType::Combat => "The beast strikes back!",
            TrialType::Revelation => "Doubt clouds your answer.",
            TrialType::Sacrifice => "The price was insufficient.",
            TrialType::Endurance => "You falter, but recover.",
        }
    };

    trial.stage += 1;
    (succeeded, message)
}

/// Complete a trial and apply results
pub fn complete_trial(trial: &TrialState, state: &mut GameState) -> Result<TierLevel, &'static str> {
    if !trial.is_complete() {
        return Err("Trial not yet complete");
    }

    if !trial.passed() {
        return Err("Trial failed. Gain more power and try again.");
    }

    // Mark trial as completed
    if !state.trials_completed.contains(&trial.target_tier) {
        state.trials_completed.push(trial.target_tier);
    }

    // Attempt advancement
    super::economy::try_advance_tier(state)
}

/// Get current mentor for player's tier
pub fn get_current_mentor(state: &GameState) -> Option<&'static Mentor> {
    if let Some(ref mentor_key) = state.current_mentor {
        super::data::get_mentor(mentor_key)
    } else {
        // Find appropriate mentor
        let mentors = get_mentors_for_tier(state.tier);
        mentors.first().copied()
    }
}

/// Get a hint from the current mentor
pub fn get_mentor_hint(state: &mut GameState) -> Option<String> {
    let mentor = get_current_mentor(state)?;
    let hint_index = (state.mentor_hints_received as usize) % mentor.hints.len();
    state.mentor_hints_received += 1;

    Some(format!("{} says: \"{}\"", mentor.name, mentor.hints[hint_index]))
}

/// Check if mentor should provide warning
pub fn should_mentor_warn(state: &GameState) -> Option<String> {
    // Warn about plateau
    if state.is_plateaued() {
        let mentor = get_current_mentor(state)?;
        return Some(format!(
            "{} warns: \"Your path has reached a ceiling. You must find a way to break through, or consider starting anew.\"",
            mentor.name
        ));
    }

    // Warn about incompatible paths
    if let (Some(ref primary), Some(ref secondary)) = (&state.primary_path, &state.secondary_path) {
        if let Some(primary_path) = super::data::get_path(primary) {
            if primary_path.incompatible_with.contains(&secondary.as_str()) {
                let mentor = get_current_mentor(state)?;
                return Some(format!(
                    "{} warns: \"Your paths conflict. This combination leads only to stagnation.\"",
                    mentor.name
                ));
            }
        }
    }

    None
}

/// Generate a random encounter event
pub fn generate_encounter(state: &GameState) -> Option<EncounterEvent> {
    let mut rng = rand::thread_rng();

    // Encounter chance based on tier
    let chance = 20 + (state.tier as u32 * 5);
    if rng.gen_range(0..100) >= chance {
        return None;
    }

    let encounters = match state.tier {
        TierLevel::Unsouled | TierLevel::Copper => vec![
            EncounterEvent::FindTreasure { name: "Minor Spirit Stone", value: 50 },
            EncounterEvent::Meet { name: "Wandering Cultivator", outcome: "shares cycling tips" },
        ],
        TierLevel::Iron | TierLevel::Jade => vec![
            EncounterEvent::FindTreasure { name: "Spirit Fruit", value: 200 },
            EncounterEvent::Combat { enemy: "Spirit Beast", power: state.total_power() / 2 },
            EncounterEvent::Meet { name: "Sect Recruiter", outcome: "offers guidance" },
        ],
        TierLevel::Gold | TierLevel::Lord => vec![
            EncounterEvent::FindTreasure { name: "Rare Elixir", value: 1000 },
            EncounterEvent::Combat { enemy: "Sacred Beast", power: state.total_power() },
            EncounterEvent::Discovery { name: "Ancient Inheritance" },
        ],
        TierLevel::Overlord | TierLevel::Sage | TierLevel::Herald => vec![
            EncounterEvent::FindTreasure { name: "Void Key", value: 5000 },
            EncounterEvent::Combat { enemy: "Remnant", power: state.total_power() * 2 },
            EncounterEvent::Discovery { name: "Pocket World Entrance" },
        ],
        _ => vec![
            EncounterEvent::Discovery { name: "Iteration Gate" },
            EncounterEvent::Meet { name: "Abidan Scout", outcome: "acknowledges your power" },
        ],
    };

    Some(encounters[rng.gen_range(0..encounters.len())].clone())
}

/// Encounter event types
#[derive(Debug, Clone)]
pub enum EncounterEvent {
    FindTreasure { name: &'static str, value: u64 },
    Combat { enemy: &'static str, power: u64 },
    Meet { name: &'static str, outcome: &'static str },
    Discovery { name: &'static str },
}

/// Apply encounter results
pub fn apply_encounter(event: &EncounterEvent, state: &mut GameState) -> String {
    match event {
        EncounterEvent::FindTreasure { name, value } => {
            state.spirit_stones += value;
            format!("Found {}! Gained {} spirit stones.", name, value)
        }
        EncounterEvent::Combat { enemy, power } => {
            let player_power = state.total_power();
            if player_power >= *power {
                let reward = power / 10;
                state.spirit_stones += reward;
                state.stats.battles_won += 1;
                state.stats.enemies_defeated += 1;
                format!("Defeated {}! Gained {} spirit stones.", enemy, reward)
            } else {
                state.stats.battles_lost += 1;
                format!("Fled from {}. Live to fight another day.", enemy)
            }
        }
        EncounterEvent::Meet { name, outcome } => {
            state.insight += 5;
            format!("Met {}. They {}. +5 insight.", name, outcome)
        }
        EncounterEvent::Discovery { name } => {
            state.insight += 10;
            state.spirit_stones += 100;
            format!("Discovered {}! This knowledge will serve you well.", name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_creation() {
        let trial = TrialState::new(TierLevel::Jade);
        assert_eq!(trial.trial_type, TrialType::Meditation);
        assert_eq!(trial.stage, 1);
        assert!(!trial.is_complete());
    }

    #[test]
    fn test_trial_progression() {
        let mut trial = TrialState::new(TierLevel::Gold);
        let state = GameState::new("Test".to_string());

        // Progress through stages
        for _ in 0..trial.max_stages {
            let (_, _) = process_trial_choice(&mut trial, 0, &state);
        }

        assert!(trial.is_complete());
    }

    #[test]
    fn test_mentor_hint() {
        let mut state = GameState::new("Test".to_string());
        state.current_mentor = Some("elder_wei".to_string());

        let hint = get_mentor_hint(&mut state);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("Elder Wei"));
    }

    #[test]
    fn test_plateau_warning() {
        let mut state = GameState::new("Test".to_string());
        state.primary_path = Some("pure_force".to_string());
        state.tier = TierLevel::Overlord;

        let warning = should_mentor_warn(&state);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("ceiling"));
    }

    #[test]
    fn test_encounter_generation() {
        let mut state = GameState::new("Test".to_string());
        state.tier = TierLevel::Gold;

        // Try multiple times (encounters are random)
        let mut found = false;
        for _ in 0..100 {
            if generate_encounter(&state).is_some() {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_encounter_application() {
        let mut state = GameState::new("Test".to_string());
        let initial_stones = state.spirit_stones;

        let event = EncounterEvent::FindTreasure { name: "Test", value: 100 };
        apply_encounter(&event, &mut state);

        assert_eq!(state.spirit_stones, initial_stones + 100);
    }
}
