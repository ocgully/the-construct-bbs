//! Story Flag Management for Realm of Ralnar
//!
//! Tracks all story progress including shrine destruction, world phases,
//! party membership, and Dorl's manipulation state.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key story flags - constants for important story events
pub mod flags {
    // === Dorl-related flags ===
    pub const MET_DORL: &str = "met_dorl";
    pub const RECEIVED_BLESSING: &str = "received_blessing";

    // === Party member flags ===
    pub const SERA_JOINED: &str = "sera_joined";
    pub const KORRATH_JOINED: &str = "korrath_joined";
    pub const ZANTH_JOINED: &str = "zanth_joined";
    pub const ZANTH_LEFT: &str = "zanth_left";
    pub const ZANTH_RETURNED: &str = "zanth_returned";
    pub const JOHN_JOINED: &str = "john_joined";
    pub const JOHN_LEFT: &str = "john_left";
    pub const NOMODEST_JOINED: &str = "nomodest_joined";
    pub const NOMODEST_LEFT: &str = "nomodest_left";
    pub const NOMODEST_RETURNED: &str = "nomodest_returned";
    pub const LYRA_JOINED: &str = "lyra_joined";
    pub const LYRA_LEFT: &str = "lyra_left";
    pub const ELDER_MORATH_JOINED: &str = "elder_morath_joined";
    pub const ELDER_MORATH_DIED: &str = "elder_morath_died";

    // === Major story events ===
    pub const THE_REVEAL: &str = "the_reveal";
    pub const DORL_TRUE_NATURE_KNOWN: &str = "dorl_true_nature_known";
    pub const ECHOES_COLLECTED: &str = "echoes_collected";
    pub const FINAL_BOSS_DEFEATED: &str = "final_boss_defeated";

    // === Shrine completion flags ===
    pub const SHRINE_1_COMPLETE: &str = "shrine_1_complete";
    pub const SHRINE_2_COMPLETE: &str = "shrine_2_complete";
    pub const SHRINE_3_COMPLETE: &str = "shrine_3_complete";
    pub const SHRINE_4_COMPLETE: &str = "shrine_4_complete";
    pub const SHRINE_5_COMPLETE: &str = "shrine_5_complete";

    // === Guardian power flags ===
    pub const GUARDIAN_1_POWER: &str = "guardian_1_power";
    pub const GUARDIAN_2_POWER: &str = "guardian_2_power";
    pub const GUARDIAN_3_POWER: &str = "guardian_3_power";
    pub const GUARDIAN_4_POWER: &str = "guardian_4_power";
    pub const GUARDIAN_5_POWER: &str = "guardian_5_power";

    // === Echo collection flags ===
    pub const ECHO_TERRETH: &str = "echo_terreth";
    pub const ECHO_AQUALIS: &str = "echo_aqualis";
    pub const ECHO_LUMINOS: &str = "echo_luminos";
    pub const ECHO_VENTUS: &str = "echo_ventus";
    pub const ECHO_PYRETH: &str = "echo_pyreth";

    // === Blessing renewal flags (tracks Dorl's suspicious pattern) ===
    pub const BLESSING_RENEWED_1: &str = "blessing_renewed_1";
    pub const BLESSING_RENEWED_2: &str = "blessing_renewed_2";
    pub const BLESSING_RENEWED_3: &str = "blessing_renewed_3";
    pub const BLESSING_RENEWED_4: &str = "blessing_renewed_4";
    pub const BLESSING_RENEWED_5: &str = "blessing_renewed_5";

    // === Tutorial and early game flags ===
    pub const TUTORIAL_COMPLETE: &str = "tutorial_complete";
    pub const LEFT_MILLBROOK: &str = "left_millbrook";
    pub const REACHED_THORNWICK: &str = "reached_thornwick";
    pub const DEFENDED_THORNWICK: &str = "defended_thornwick";
}

/// The Five Guardian names
pub const GUARDIAN_NAMES: [&str; 5] = ["Terreth", "Aqualis", "Luminos", "Ventus", "Pyreth"];

/// World phase - the state of the world based on shrines destroyed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorldPhase {
    #[default]
    Normal,      // 0: Before any shrines - world is troubled but stable
    Unsettled,   // 1: After shrine 1 - people notice something wrong
    Troubled,    // 2: After shrine 2 - monsters more aggressive
    Dangerous,   // 3: After shrine 3 - safe zones shrinking
    Dire,        // 4: After shrine 4 - civilization crumbling
    Apocalyptic, // 5: After shrine 5 (the reveal) - Dorl's true form emerges
    Redemption,  // 6: Post-reveal, gathering echoes to seal Dorl
}

impl WorldPhase {
    /// Get the display name for this phase
    pub fn name(&self) -> &'static str {
        match self {
            WorldPhase::Normal => "Normal",
            WorldPhase::Unsettled => "Unsettled",
            WorldPhase::Troubled => "Troubled",
            WorldPhase::Dangerous => "Dangerous",
            WorldPhase::Dire => "Dire",
            WorldPhase::Apocalyptic => "Apocalyptic",
            WorldPhase::Redemption => "Redemption",
        }
    }

    /// Get a description of this world phase
    pub fn description(&self) -> &'static str {
        match self {
            WorldPhase::Normal => "The land is troubled, but hope remains.",
            WorldPhase::Unsettled => "Something feels wrong. The air itself seems uneasy.",
            WorldPhase::Troubled => "Monsters grow bolder. Safe paths grow fewer.",
            WorldPhase::Dangerous => "Darkness spreads. Few places remain untouched.",
            WorldPhase::Dire => "Civilization crumbles. Only the strong survive.",
            WorldPhase::Apocalyptic => "The truth is revealed. The world hangs by a thread.",
            WorldPhase::Redemption => "A chance for redemption. Gather the echoes.",
        }
    }

    /// Get the encounter rate modifier for this phase
    pub fn encounter_modifier(&self) -> f32 {
        match self {
            WorldPhase::Normal => 1.0,
            WorldPhase::Unsettled => 1.2,
            WorldPhase::Troubled => 1.4,
            WorldPhase::Dangerous => 1.6,
            WorldPhase::Dire => 1.8,
            WorldPhase::Apocalyptic => 2.0,
            WorldPhase::Redemption => 1.5,
        }
    }

    /// Get the enemy strength modifier for this phase
    pub fn enemy_strength_modifier(&self) -> f32 {
        match self {
            WorldPhase::Normal => 1.0,
            WorldPhase::Unsettled => 1.1,
            WorldPhase::Troubled => 1.2,
            WorldPhase::Dangerous => 1.3,
            WorldPhase::Dire => 1.5,
            WorldPhase::Apocalyptic => 1.8,
            WorldPhase::Redemption => 1.4,
        }
    }
}

/// Tracks all story progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryState {
    /// All story flags (named events that have occurred)
    pub flags: HashMap<String, bool>,
    /// Which shrines have been destroyed (0-4, corresponds to 5 guardians)
    pub shrines_destroyed: [bool; 5],
    /// Current state of the world
    pub world_phase: WorldPhase,
    /// Whether Dorl's blessing (illusion spell) is active
    pub dorl_blessing_active: bool,
    /// Party members who have been met (by ID)
    pub party_members_met: Vec<String>,
    /// Current story chapter (1-6)
    pub current_chapter: u8,
    /// Echoes collected (for post-reveal sequence)
    pub echoes_collected: [bool; 5],
    /// Dialogue history - which dialogues have been seen
    pub seen_dialogues: Vec<String>,
    /// Number of steps taken (for random dialogue triggers)
    pub steps_taken: u32,
}

impl Default for StoryState {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryState {
    /// Create a new story state at the beginning of the game
    pub fn new() -> Self {
        StoryState {
            flags: HashMap::new(),
            shrines_destroyed: [false; 5],
            world_phase: WorldPhase::Normal,
            dorl_blessing_active: false,
            party_members_met: vec!["herbert".to_string(), "valeran".to_string()],
            current_chapter: 1,
            echoes_collected: [false; 5],
            seen_dialogues: Vec::new(),
            steps_taken: 0,
        }
    }

    /// Set a story flag
    pub fn set_flag(&mut self, flag: &str, value: bool) {
        self.flags.insert(flag.to_string(), value);
    }

    /// Check if a story flag is set (defaults to false if not present)
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Clear a story flag
    pub fn clear_flag(&mut self, flag: &str) {
        self.flags.remove(flag);
    }

    /// Record that a shrine has been destroyed and update world phase
    pub fn destroy_shrine(&mut self, shrine_index: usize) {
        if shrine_index < 5 {
            self.shrines_destroyed[shrine_index] = true;
            self.update_world_phase();

            // Set the shrine completion flag
            let shrine_flag = match shrine_index {
                0 => flags::SHRINE_1_COMPLETE,
                1 => flags::SHRINE_2_COMPLETE,
                2 => flags::SHRINE_3_COMPLETE,
                3 => flags::SHRINE_4_COMPLETE,
                4 => flags::SHRINE_5_COMPLETE,
                _ => return,
            };
            self.set_flag(shrine_flag, true);

            // Grant Guardian power
            let power_flag = match shrine_index {
                0 => flags::GUARDIAN_1_POWER,
                1 => flags::GUARDIAN_2_POWER,
                2 => flags::GUARDIAN_3_POWER,
                3 => flags::GUARDIAN_4_POWER,
                4 => flags::GUARDIAN_5_POWER,
                _ => return,
            };
            self.set_flag(power_flag, true);
        }
    }

    /// Get how many shrines have been destroyed
    pub fn shrines_destroyed_count(&self) -> usize {
        self.shrines_destroyed.iter().filter(|&&d| d).count()
    }

    /// Update world phase based on shrines destroyed
    fn update_world_phase(&mut self) {
        let count = self.shrines_destroyed_count();
        self.world_phase = match count {
            0 => WorldPhase::Normal,
            1 => WorldPhase::Unsettled,
            2 => WorldPhase::Troubled,
            3 => WorldPhase::Dangerous,
            4 => WorldPhase::Dire,
            5 => {
                // When all 5 shrines are destroyed, check if reveal has happened
                if self.has_flag(flags::THE_REVEAL) {
                    WorldPhase::Redemption
                } else {
                    WorldPhase::Apocalyptic
                }
            }
            _ => WorldPhase::Redemption,
        };
    }

    /// Trigger the revelation - when Sera recognizes the Fire Guardian
    pub fn trigger_reveal(&mut self) {
        // The blessing fades and the truth is revealed
        self.dorl_blessing_active = false;
        self.set_flag(flags::THE_REVEAL, true);
        self.set_flag(flags::DORL_TRUE_NATURE_KNOWN, true);
        self.world_phase = WorldPhase::Apocalyptic;
        self.current_chapter = 5;
    }

    /// Begin the redemption arc (gathering echoes)
    pub fn begin_redemption(&mut self) {
        self.world_phase = WorldPhase::Redemption;
        self.current_chapter = 6;
    }

    /// Collect an echo from a defeated guardian's shrine
    pub fn collect_echo(&mut self, echo_index: usize) {
        if echo_index < 5 {
            self.echoes_collected[echo_index] = true;

            // Set the corresponding flag
            let echo_flag = match echo_index {
                0 => flags::ECHO_TERRETH,
                1 => flags::ECHO_AQUALIS,
                2 => flags::ECHO_LUMINOS,
                3 => flags::ECHO_VENTUS,
                4 => flags::ECHO_PYRETH,
                _ => return,
            };
            self.set_flag(echo_flag, true);

            // Check if all echoes collected
            if self.echoes_collected.iter().all(|&e| e) {
                self.set_flag(flags::ECHOES_COLLECTED, true);
            }
        }
    }

    /// Get how many echoes have been collected
    pub fn echoes_collected_count(&self) -> usize {
        self.echoes_collected.iter().filter(|&&e| e).count()
    }

    /// Record that a party member has been met
    pub fn meet_party_member(&mut self, member_id: &str) {
        if !self.party_members_met.contains(&member_id.to_string()) {
            self.party_members_met.push(member_id.to_string());
        }
    }

    /// Check if a party member has been met
    pub fn has_met(&self, member_id: &str) -> bool {
        self.party_members_met.contains(&member_id.to_string())
    }

    /// Record that a dialogue has been seen
    pub fn mark_dialogue_seen(&mut self, dialogue_id: &str) {
        if !self.seen_dialogues.contains(&dialogue_id.to_string()) {
            self.seen_dialogues.push(dialogue_id.to_string());
        }
    }

    /// Check if a dialogue has been seen
    pub fn has_seen_dialogue(&self, dialogue_id: &str) -> bool {
        self.seen_dialogues.contains(&dialogue_id.to_string())
    }

    /// Increment step counter and return true if a random event should trigger
    pub fn take_step(&mut self) -> bool {
        self.steps_taken += 1;
        // Every 50 steps, there's a chance for random dialogue
        self.steps_taken % 50 == 0
    }

    /// Advance to the next chapter
    pub fn advance_chapter(&mut self) {
        if self.current_chapter < 6 {
            self.current_chapter += 1;
        }
    }

    /// Get the name of the current chapter
    pub fn chapter_name(&self) -> &'static str {
        match self.current_chapter {
            1 => "The Journey Begins",
            2 => "The First Shrine",
            3 => "Gathering Allies",
            4 => "The Hunt Continues",
            5 => "The Revelation",
            6 => "Redemption",
            _ => "Unknown",
        }
    }

    /// Check if we're in the post-reveal portion of the game
    pub fn is_post_reveal(&self) -> bool {
        self.has_flag(flags::THE_REVEAL)
    }

    /// Check if the final boss has been defeated
    pub fn is_game_complete(&self) -> bool {
        self.has_flag(flags::FINAL_BOSS_DEFEATED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_story_state() {
        let state = StoryState::new();
        assert_eq!(state.world_phase, WorldPhase::Normal);
        assert_eq!(state.current_chapter, 1);
        assert!(!state.dorl_blessing_active);
        assert_eq!(state.shrines_destroyed_count(), 0);
        // Brothers are already in party at start
        assert!(state.has_met("herbert"));
        assert!(state.has_met("valeran"));
    }

    #[test]
    fn test_flag_operations() {
        let mut state = StoryState::new();

        // Flag not set initially
        assert!(!state.has_flag("test_flag"));

        // Set flag
        state.set_flag("test_flag", true);
        assert!(state.has_flag("test_flag"));

        // Clear flag
        state.clear_flag("test_flag");
        assert!(!state.has_flag("test_flag"));

        // Set to false
        state.set_flag("another_flag", false);
        assert!(!state.has_flag("another_flag"));
    }

    #[test]
    fn test_world_phase_progression() {
        let mut state = StoryState::new();
        assert_eq!(state.world_phase, WorldPhase::Normal);

        state.destroy_shrine(0);
        assert_eq!(state.world_phase, WorldPhase::Unsettled);
        assert_eq!(state.shrines_destroyed_count(), 1);

        state.destroy_shrine(1);
        assert_eq!(state.world_phase, WorldPhase::Troubled);
        assert_eq!(state.shrines_destroyed_count(), 2);

        state.destroy_shrine(2);
        assert_eq!(state.world_phase, WorldPhase::Dangerous);
        assert_eq!(state.shrines_destroyed_count(), 3);

        state.destroy_shrine(3);
        assert_eq!(state.world_phase, WorldPhase::Dire);
        assert_eq!(state.shrines_destroyed_count(), 4);

        state.destroy_shrine(4);
        assert_eq!(state.world_phase, WorldPhase::Apocalyptic);
        assert_eq!(state.shrines_destroyed_count(), 5);
    }

    #[test]
    fn test_shrine_completion_flags() {
        let mut state = StoryState::new();

        state.destroy_shrine(0);
        assert!(state.has_flag(flags::SHRINE_1_COMPLETE));
        assert!(state.has_flag(flags::GUARDIAN_1_POWER));

        state.destroy_shrine(2);
        assert!(state.has_flag(flags::SHRINE_3_COMPLETE));
        assert!(state.has_flag(flags::GUARDIAN_3_POWER));
    }

    #[test]
    fn test_trigger_reveal() {
        let mut state = StoryState::new();

        // Setup: destroy all 5 shrines
        for i in 0..5 {
            state.destroy_shrine(i);
        }
        state.dorl_blessing_active = true;

        // Trigger the reveal
        state.trigger_reveal();

        assert!(!state.dorl_blessing_active);
        assert!(state.has_flag(flags::THE_REVEAL));
        assert!(state.has_flag(flags::DORL_TRUE_NATURE_KNOWN));
        assert_eq!(state.world_phase, WorldPhase::Apocalyptic);
        assert_eq!(state.current_chapter, 5);
        assert!(state.is_post_reveal());
    }

    #[test]
    fn test_echo_collection() {
        let mut state = StoryState::new();

        assert_eq!(state.echoes_collected_count(), 0);
        assert!(!state.has_flag(flags::ECHOES_COLLECTED));

        state.collect_echo(0);
        assert!(state.has_flag(flags::ECHO_TERRETH));
        assert_eq!(state.echoes_collected_count(), 1);

        // Collect all echoes
        for i in 1..5 {
            state.collect_echo(i);
        }
        assert_eq!(state.echoes_collected_count(), 5);
        assert!(state.has_flag(flags::ECHOES_COLLECTED));
    }

    #[test]
    fn test_party_member_tracking() {
        let mut state = StoryState::new();

        // Brothers already met
        assert!(state.has_met("herbert"));
        assert!(state.has_met("valeran"));

        // Meet new member
        assert!(!state.has_met("sera"));
        state.meet_party_member("sera");
        assert!(state.has_met("sera"));

        // Meeting again shouldn't duplicate
        state.meet_party_member("sera");
        assert_eq!(state.party_members_met.iter().filter(|&m| m == "sera").count(), 1);
    }

    #[test]
    fn test_dialogue_tracking() {
        let mut state = StoryState::new();

        assert!(!state.has_seen_dialogue("exploration_001"));
        state.mark_dialogue_seen("exploration_001");
        assert!(state.has_seen_dialogue("exploration_001"));

        // Marking again shouldn't duplicate
        state.mark_dialogue_seen("exploration_001");
        assert_eq!(
            state.seen_dialogues.iter().filter(|&d| d == "exploration_001").count(),
            1
        );
    }

    #[test]
    fn test_step_counter() {
        let mut state = StoryState::new();

        // Steps 1-49 shouldn't trigger
        for _ in 0..49 {
            assert!(!state.take_step());
        }

        // Step 50 should trigger
        assert!(state.take_step());
        assert_eq!(state.steps_taken, 50);

        // Steps 51-99 shouldn't trigger
        for _ in 0..49 {
            assert!(!state.take_step());
        }

        // Step 100 should trigger
        assert!(state.take_step());
    }

    #[test]
    fn test_chapter_advancement() {
        let mut state = StoryState::new();
        assert_eq!(state.current_chapter, 1);

        state.advance_chapter();
        assert_eq!(state.current_chapter, 2);

        // Advance to max
        for _ in 0..10 {
            state.advance_chapter();
        }
        assert_eq!(state.current_chapter, 6); // Cap at 6
    }

    #[test]
    fn test_world_phase_modifiers() {
        assert_eq!(WorldPhase::Normal.encounter_modifier(), 1.0);
        assert!(WorldPhase::Apocalyptic.encounter_modifier() > WorldPhase::Normal.encounter_modifier());
        assert!(WorldPhase::Apocalyptic.enemy_strength_modifier() > 1.0);
    }

    #[test]
    fn test_redemption_arc() {
        let mut state = StoryState::new();

        // Complete the main story
        for i in 0..5 {
            state.destroy_shrine(i);
        }
        state.trigger_reveal();
        assert_eq!(state.world_phase, WorldPhase::Apocalyptic);

        // Begin redemption
        state.begin_redemption();
        assert_eq!(state.world_phase, WorldPhase::Redemption);
        assert_eq!(state.current_chapter, 6);
    }

    #[test]
    fn test_invalid_shrine_index() {
        let mut state = StoryState::new();

        // Invalid index should not panic or change state
        state.destroy_shrine(10);
        assert_eq!(state.shrines_destroyed_count(), 0);
        assert_eq!(state.world_phase, WorldPhase::Normal);
    }

    #[test]
    fn test_invalid_echo_index() {
        let mut state = StoryState::new();

        // Invalid index should not panic or change state
        state.collect_echo(10);
        assert_eq!(state.echoes_collected_count(), 0);
    }

    #[test]
    fn test_default_implementation() {
        let state = StoryState::default();
        assert_eq!(state.world_phase, WorldPhase::Normal);
        assert_eq!(state.current_chapter, 1);
    }
}
