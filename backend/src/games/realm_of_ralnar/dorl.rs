//! Dorl's Special Handling for Realm of Ralnar
//!
//! Special systems for Dorl's manipulations, including:
//! - The blessing (illusion spell)
//! - Encounter rate manipulation
//! - Dynamic dialogue based on story state

use super::events::GameStateForEvents;
use super::story::{flags, StoryState, WorldPhase};

/// Special handling for Dorl's manipulations
pub struct DorlSystem;

impl DorlSystem {
    /// Apply Dorl's "blessing" (the illusion spell)
    ///
    /// This blessing makes the Guardians appear as monsters to those affected.
    /// Sera NEVER receives the blessing - she refused it, which is why Dorl
    /// keeps her away from the shrines.
    pub fn apply_blessing(state: &mut GameStateForEvents) {
        state.story.dorl_blessing_active = true;
        state.story.set_flag(flags::RECEIVED_BLESSING, true);
    }

    /// Apply blessing to story state directly
    pub fn apply_blessing_to_story(story: &mut StoryState) {
        story.dorl_blessing_active = true;
        story.set_flag(flags::RECEIVED_BLESSING, true);
    }

    /// Renew blessing before each shrine (suspicious pattern)
    ///
    /// Dorl "renews" his blessing before each shrine. Players might notice
    /// this suspicious pattern after the reveal.
    pub fn renew_blessing(state: &mut GameStateForEvents, shrine_number: u8) {
        state.story.dorl_blessing_active = true;
        state.story.set_flag(&format!("blessing_renewed_{}", shrine_number), true);
    }

    /// Renew blessing on story state directly
    pub fn renew_blessing_on_story(story: &mut StoryState, shrine_number: u8) {
        story.dorl_blessing_active = true;
        story.set_flag(&format!("blessing_renewed_{}", shrine_number), true);
    }

    /// Remove Dorl's blessing (happens at the reveal)
    pub fn remove_blessing(state: &mut GameStateForEvents) {
        state.story.dorl_blessing_active = false;
    }

    /// Check if the blessing is currently active
    pub fn is_blessing_active(story: &StoryState) -> bool {
        story.dorl_blessing_active
    }

    /// Get Dorl's dialogue based on story state
    ///
    /// Dorl's dialogue changes throughout the game, with subtle hints
    /// that only make sense after the reveal.
    pub fn get_dorl_dialogue_id(story: &StoryState) -> &'static str {
        // Post-reveal dialogue
        if story.has_flag(flags::THE_REVEAL) {
            return "dorl_revealed";
        }

        // Based on shrines cleared
        match story.shrines_destroyed_count() {
            0 => "dorl_pre_shrine_1",
            1 => "dorl_after_shrine_1",
            2 => "dorl_after_shrine_2",
            3 => "dorl_after_shrine_3",
            4 => "dorl_before_final_shrine",
            _ => "dorl_default",
        }
    }

    /// Get actual dialogue text for Dorl
    pub fn get_dorl_dialogue(story: &StoryState) -> &'static str {
        let id = Self::get_dorl_dialogue_id(story);

        match id {
            "dorl_pre_shrine_1" => {
                "You've done well to come this far. The first shrine lies to the east. \
                The corruption there is... strong. But I believe in you both."
            }
            "dorl_after_shrine_1" => {
                "Excellent work! I knew you had the strength. The darkness has been \
                pushed back, if only a little. But there are four more shrines..."
            }
            "dorl_after_shrine_2" => {
                "Two shrines cleansed! The world breathes easier, though the darkness \
                still spreads. You're proving to be exactly what I hoped for."
            }
            "dorl_after_shrine_3" => {
                "Three down. I must admit, I had my doubts early on, but you've exceeded \
                every expectation. The remaining shrines will be the hardest yet."
            }
            "dorl_before_final_shrine" => {
                "One shrine remains. The Fire Shrine is the most dangerous of all. \
                Let me renew my blessing before you enter. I... I want to see this through."
            }
            "dorl_revealed" => {
                "So you've finally seen through my little illusion. How disappointing. \
                Do you know how long I waited? How carefully I planned? \
                Five heroes sealed me away once. Now five more have set me free. \
                The irony is... exquisite."
            }
            _ => "The path ahead is dangerous. Be careful, my young friends.",
        }
    }

    /// Get foreshadowing dialogue that seems innocent but hints at the truth
    pub fn get_foreshadowing_dialogue(story: &StoryState) -> Option<&'static str> {
        if story.has_flag(flags::THE_REVEAL) {
            return None; // No foreshadowing after the reveal
        }

        match story.shrines_destroyed_count() {
            1 => Some(
                "You know, I've walked these lands for longer than I can remember. \
                Every path, every shrine... I know them all by heart."
            ),
            2 => Some(
                "The Guardians were said to be powerful protectors once. \
                Something corrupted them. Made them... monstrous."
            ),
            3 => Some(
                "Sera is a wonderful healer, isn't she? I'm glad she's with you. \
                Though it's best she stays outside the shrines. The corruption \
                might affect her faith."
            ),
            4 => Some(
                "One more shrine. After all these years, one more shrine. \
                You can't imagine what this means to me."
            ),
            _ => None,
        }
    }

    /// Get monster encounter rate modifier based on shrines cleared
    ///
    /// Dorl withdraws monsters from "cleansed" regions to maintain the illusion
    /// that the party is helping. Other regions become more dangerous as his
    /// power grows.
    pub fn get_encounter_modifier(story: &StoryState, map_region: u8) -> f32 {
        // Post-reveal: everywhere is dangerous
        if story.has_flag(flags::THE_REVEAL) {
            return match story.world_phase {
                WorldPhase::Apocalyptic => 2.0,
                WorldPhase::Redemption => 1.5,
                _ => 1.0,
            };
        }

        let shrine_for_region = map_region as usize;

        // If this region's shrine is destroyed, Dorl withdrew his monsters
        // to maintain the illusion of "cleansing"
        if shrine_for_region < 5 && story.shrines_destroyed[shrine_for_region] {
            // "Cleansed" region - Dorl withdrew his monsters
            0.2 // 20% of normal encounters
        } else {
            // Other shrines cleared increases monsters in uncleansed regions
            // as Dorl's power grows
            1.0 + (0.3 * story.shrines_destroyed_count() as f32)
        }
    }

    /// Get monster strength modifier based on shrines cleared
    ///
    /// As more shrines are destroyed, Dorl's power grows and monsters
    /// become stronger throughout the world.
    pub fn get_monster_strength_modifier(story: &StoryState) -> f32 {
        if story.has_flag(flags::THE_REVEAL) {
            return 1.8; // Post-reveal, monsters are at full strength
        }

        1.0 + (0.15 * story.shrines_destroyed_count() as f32)
    }

    /// Check if Sera should be "conveniently" absent for a shrine
    ///
    /// Dorl arranges for Sera to be away during shrine battles because
    /// she doesn't have his blessing and would see the Guardians' true forms.
    pub fn should_sera_be_absent(story: &StoryState, approaching_shrine: bool) -> bool {
        // Sera is only absent if:
        // 1. She has joined the party
        // 2. The reveal hasn't happened yet
        // 3. The party is approaching a shrine

        if !story.has_flag(flags::SERA_JOINED) {
            return false;
        }

        if story.has_flag(flags::THE_REVEAL) {
            return false; // After the reveal, Sera stays
        }

        approaching_shrine
    }

    /// Get the excuse Dorl uses to keep Sera away from a shrine
    pub fn get_sera_absence_excuse(shrine_number: u8) -> &'static str {
        match shrine_number {
            1 => "The Temple Council has requested Sera's presence urgently.",
            2 => "Refugees need healing at the southern camp. Sera should go.",
            3 => "A plague has struck a nearby village. Sera's skills are needed.",
            4 => "The monastery has fallen. Sera must help evacuate survivors.",
            5 => "Wait... you should all go together this time. Something feels different.",
            _ => "Sera is needed elsewhere.",
        }
    }

    /// Check if this is a "suspicious moment" - something players might
    /// remember after the reveal
    pub fn is_suspicious_moment(story: &StoryState, context: &str) -> bool {
        if story.has_flag(flags::THE_REVEAL) {
            return false; // No longer suspicious, just horrifying
        }

        match context {
            "blessing_renewal" => story.shrines_destroyed_count() >= 2,
            "sera_conveniently_absent" => story.shrines_destroyed_count() >= 3,
            "dorl_knows_too_much" => story.shrines_destroyed_count() >= 4,
            "monsters_avoiding_cleansed_areas" => story.shrines_destroyed_count() >= 2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_blessing() {
        let mut state = GameStateForEvents::new();
        assert!(!state.story.dorl_blessing_active);

        DorlSystem::apply_blessing(&mut state);

        assert!(state.story.dorl_blessing_active);
        assert!(state.story.has_flag(flags::RECEIVED_BLESSING));
    }

    #[test]
    fn test_renew_blessing() {
        let mut state = GameStateForEvents::new();

        DorlSystem::renew_blessing(&mut state, 2);

        assert!(state.story.dorl_blessing_active);
        assert!(state.story.has_flag("blessing_renewed_2"));
    }

    #[test]
    fn test_remove_blessing() {
        let mut state = GameStateForEvents::new();
        state.story.dorl_blessing_active = true;

        DorlSystem::remove_blessing(&mut state);

        assert!(!state.story.dorl_blessing_active);
    }

    #[test]
    fn test_dialogue_progression() {
        let mut story = StoryState::new();

        assert_eq!(DorlSystem::get_dorl_dialogue_id(&story), "dorl_pre_shrine_1");

        story.destroy_shrine(0);
        assert_eq!(DorlSystem::get_dorl_dialogue_id(&story), "dorl_after_shrine_1");

        story.destroy_shrine(1);
        assert_eq!(DorlSystem::get_dorl_dialogue_id(&story), "dorl_after_shrine_2");

        story.destroy_shrine(2);
        assert_eq!(DorlSystem::get_dorl_dialogue_id(&story), "dorl_after_shrine_3");

        story.destroy_shrine(3);
        assert_eq!(DorlSystem::get_dorl_dialogue_id(&story), "dorl_before_final_shrine");

        story.destroy_shrine(4);
        story.trigger_reveal();
        assert_eq!(DorlSystem::get_dorl_dialogue_id(&story), "dorl_revealed");
    }

    #[test]
    fn test_encounter_modifier_cleansed_region() {
        let mut story = StoryState::new();

        // Before any shrines, normal rate
        assert_eq!(DorlSystem::get_encounter_modifier(&story, 0), 1.0);

        // After clearing shrine 0, region 0 has reduced encounters
        story.destroy_shrine(0);
        assert_eq!(DorlSystem::get_encounter_modifier(&story, 0), 0.2);

        // But region 1 has increased encounters
        let region_1_modifier = DorlSystem::get_encounter_modifier(&story, 1);
        assert!(region_1_modifier > 1.0);
    }

    #[test]
    fn test_encounter_modifier_scaling() {
        let mut story = StoryState::new();

        // Get baseline for uncleansed region
        let baseline = DorlSystem::get_encounter_modifier(&story, 3);
        assert_eq!(baseline, 1.0);

        // Each shrine cleared should increase the modifier in uncleansed regions
        story.destroy_shrine(0);
        let after_one = DorlSystem::get_encounter_modifier(&story, 3);
        assert!(after_one > baseline);

        story.destroy_shrine(1);
        let after_two = DorlSystem::get_encounter_modifier(&story, 3);
        assert!(after_two > after_one);
    }

    #[test]
    fn test_encounter_modifier_post_reveal() {
        let mut story = StoryState::new();

        // Clear all shrines
        for i in 0..5 {
            story.destroy_shrine(i);
        }
        story.trigger_reveal();

        // Post-reveal, everywhere is dangerous
        let modifier = DorlSystem::get_encounter_modifier(&story, 0);
        assert_eq!(modifier, 2.0);
    }

    #[test]
    fn test_monster_strength_modifier() {
        let mut story = StoryState::new();

        assert_eq!(DorlSystem::get_monster_strength_modifier(&story), 1.0);

        story.destroy_shrine(0);
        assert!(DorlSystem::get_monster_strength_modifier(&story) > 1.0);

        story.destroy_shrine(1);
        assert!(DorlSystem::get_monster_strength_modifier(&story) > 1.15);
    }

    #[test]
    fn test_sera_absence_logic() {
        let mut story = StoryState::new();

        // Before Sera joins, she can't be absent
        assert!(!DorlSystem::should_sera_be_absent(&story, true));

        // After Sera joins, she should be absent at shrines
        story.set_flag(flags::SERA_JOINED, true);
        assert!(DorlSystem::should_sera_be_absent(&story, true));
        assert!(!DorlSystem::should_sera_be_absent(&story, false));

        // After the reveal, Sera stays
        story.set_flag(flags::THE_REVEAL, true);
        assert!(!DorlSystem::should_sera_be_absent(&story, true));
    }

    #[test]
    fn test_sera_absence_excuses() {
        assert!(DorlSystem::get_sera_absence_excuse(1).contains("Council"));
        assert!(DorlSystem::get_sera_absence_excuse(2).contains("Refugees"));
        assert!(DorlSystem::get_sera_absence_excuse(3).contains("plague"));
        assert!(DorlSystem::get_sera_absence_excuse(4).contains("monastery"));

        // Shrine 5 is different - it's the reveal
        assert!(DorlSystem::get_sera_absence_excuse(5).contains("together"));
    }

    #[test]
    fn test_suspicious_moments() {
        let mut story = StoryState::new();

        // Early game - nothing suspicious yet
        assert!(!DorlSystem::is_suspicious_moment(&story, "blessing_renewal"));

        // After 2 shrines, blessing renewal becomes suspicious
        story.destroy_shrine(0);
        story.destroy_shrine(1);
        assert!(DorlSystem::is_suspicious_moment(&story, "blessing_renewal"));

        // After 3 shrines, Sera's absences become suspicious
        story.destroy_shrine(2);
        assert!(DorlSystem::is_suspicious_moment(&story, "sera_conveniently_absent"));

        // After the reveal, nothing is "suspicious" anymore
        story.destroy_shrine(3);
        story.destroy_shrine(4);
        story.trigger_reveal();
        assert!(!DorlSystem::is_suspicious_moment(&story, "blessing_renewal"));
    }

    #[test]
    fn test_foreshadowing_dialogue() {
        let mut story = StoryState::new();

        // No foreshadowing at start
        assert!(DorlSystem::get_foreshadowing_dialogue(&story).is_none());

        // Foreshadowing after first shrine
        story.destroy_shrine(0);
        assert!(DorlSystem::get_foreshadowing_dialogue(&story).is_some());

        // After reveal, no more foreshadowing
        for i in 1..5 {
            story.destroy_shrine(i);
        }
        story.trigger_reveal();
        assert!(DorlSystem::get_foreshadowing_dialogue(&story).is_none());
    }

    #[test]
    fn test_is_blessing_active() {
        let mut story = StoryState::new();
        assert!(!DorlSystem::is_blessing_active(&story));

        story.dorl_blessing_active = true;
        assert!(DorlSystem::is_blessing_active(&story));
    }

    #[test]
    fn test_apply_blessing_to_story_directly() {
        let mut story = StoryState::new();
        assert!(!story.dorl_blessing_active);

        DorlSystem::apply_blessing_to_story(&mut story);

        assert!(story.dorl_blessing_active);
        assert!(story.has_flag(flags::RECEIVED_BLESSING));
    }

    #[test]
    fn test_get_dorl_dialogue_text() {
        let mut story = StoryState::new();

        // Should get actual dialogue text
        let dialogue = DorlSystem::get_dorl_dialogue(&story);
        assert!(!dialogue.is_empty());
        assert!(dialogue.contains("first shrine"));

        // After reveal, dialogue changes dramatically
        for i in 0..5 {
            story.destroy_shrine(i);
        }
        story.trigger_reveal();

        let revealed_dialogue = DorlSystem::get_dorl_dialogue(&story);
        assert!(revealed_dialogue.contains("illusion") || revealed_dialogue.contains("disappointing"));
    }
}
