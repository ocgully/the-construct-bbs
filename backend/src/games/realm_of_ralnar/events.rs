//! Event System for Realm of Ralnar
//!
//! Defines game events, conditions, effects, and the event system that
//! drives story progression and world interaction.

use super::story::{flags, StoryState, WorldPhase};
use serde::{Deserialize, Serialize};

/// A game event with trigger conditions and effects
#[derive(Debug, Clone)]
pub struct GameEvent {
    /// Unique identifier for the event
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Conditions that must all be true to trigger
    pub trigger_conditions: Vec<EventCondition>,
    /// Effects that occur when the event triggers
    pub effects: Vec<EventEffect>,
    /// Optional cutscene to play
    pub cutscene_id: Option<String>,
    /// Priority (higher = checked first)
    pub priority: u8,
    /// Whether this event can only trigger once
    pub one_time: bool,
}

impl GameEvent {
    /// Create a new game event
    pub fn new(id: &str, name: &str) -> Self {
        GameEvent {
            id: id.to_string(),
            name: name.to_string(),
            trigger_conditions: Vec::new(),
            effects: Vec::new(),
            cutscene_id: None,
            priority: 50,
            one_time: true,
        }
    }

    /// Builder method to add a condition
    pub fn with_condition(mut self, condition: EventCondition) -> Self {
        self.trigger_conditions.push(condition);
        self
    }

    /// Builder method to add an effect
    pub fn with_effect(mut self, effect: EventEffect) -> Self {
        self.effects.push(effect);
        self
    }

    /// Builder method to set the cutscene
    pub fn with_cutscene(mut self, cutscene_id: &str) -> Self {
        self.cutscene_id = Some(cutscene_id.to_string());
        self
    }

    /// Builder method to set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Builder method to make repeatable
    pub fn repeatable(mut self) -> Self {
        self.one_time = false;
        self
    }
}

/// Conditions that can trigger events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCondition {
    /// A story flag must be set (true)
    Flag(String),
    /// A story flag must NOT be set (false or absent)
    NotFlag(String),
    /// A specific shrine must be destroyed
    ShrineDestroyed(usize),
    /// A specific shrine must NOT be destroyed
    ShrineNotDestroyed(usize),
    /// World must be in a specific phase
    WorldPhase(WorldPhase),
    /// World must be at or past a specific phase
    WorldPhaseAtLeast(WorldPhase),
    /// A party member must be in the active party
    PartyHas(String),
    /// A party member must NOT be in the party
    PartyDoesNotHave(String),
    /// Must be at a specific location
    AtLocation { map: String, x: u32, y: u32 },
    /// Must be on a specific map (any position)
    OnMap(String),
    /// Must have at least N of a specific item
    ItemInInventory(String, u32),
    /// Must have at least N gold
    GoldAtLeast(u32),
    /// Must be in a specific chapter
    ChapterIs(u8),
    /// Must be in chapter N or later
    ChapterAtLeast(u8),
    /// Must have met a specific party member
    HasMet(String),
    /// Random chance (0-100 percentage)
    RandomChance(u8),
    /// Time of day (if implemented)
    TimeOfDay(String),
    /// Dorl's blessing must be active
    BlessingActive,
    /// Dorl's blessing must NOT be active
    BlessingNotActive,
}

/// Effects that can result from events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventEffect {
    /// Set a story flag to a value
    SetFlag(String, bool),
    /// Add a character to the party
    JoinParty(String),
    /// Remove a character from the party
    LeaveParty(String),
    /// Add items to inventory
    GiveItem(String, u32),
    /// Remove items from inventory
    TakeItem(String, u32),
    /// Add or subtract gold
    GiveGold(i32),
    /// Move the party to a new location
    Teleport { map: String, x: u32, y: u32 },
    /// Start a battle encounter
    StartBattle {
        enemies: Vec<String>,
        boss: bool,
        guardian: Option<usize>,
    },
    /// Play a cutscene
    PlayCutscene(String),
    /// Fully heal the party
    Heal,
    /// Advance to the next chapter
    AdvanceChapter,
    /// Trigger the big reveal
    TriggerReveal,
    /// Set the world phase
    SetWorldPhase(WorldPhase),
    /// Display a message to the player
    ShowMessage(String),
    /// Mark a party member as met
    MeetMember(String),
    /// Activate Dorl's blessing
    ActivateBlessing,
    /// Deactivate Dorl's blessing
    DeactivateBlessing,
    /// Destroy a shrine
    DestroyShrine(usize),
    /// Collect an echo
    CollectEcho(usize),
    /// Play music
    PlayMusic(String),
    /// Stop music
    StopMusic,
    /// Play a sound effect
    PlaySound(String),
    /// Change the current map
    ChangeMap(String),
}

/// Result of applying an event effect
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventResult {
    /// Continue processing
    Continue,
    /// A party member joined
    PartyJoined(String),
    /// A party member left
    PartyLeft(String),
    /// A major story event occurred
    MajorStoryEvent(String),
    /// A battle should start
    BattleStart {
        enemies: Vec<String>,
        boss: bool,
        guardian: Option<usize>,
    },
    /// A cutscene should play
    Cutscene(String),
    /// A message should be displayed
    Message(String),
    /// The party was teleported
    Teleported { map: String, x: u32, y: u32 },
    /// Items were received
    ItemsReceived(String, u32),
    /// Gold was received
    GoldReceived(i32),
}

/// Simple party representation for event checking
#[derive(Debug, Clone, Default)]
pub struct Party {
    /// Active party member IDs
    pub members: Vec<String>,
    /// Party gold
    pub gold: u32,
    /// Inventory items (item_id -> count)
    pub inventory: std::collections::HashMap<String, u32>,
}

impl Party {
    /// Create a new party with the two brothers
    pub fn new() -> Self {
        Party {
            members: vec!["herbert".to_string(), "valeran".to_string()],
            gold: 100,
            inventory: std::collections::HashMap::new(),
        }
    }

    /// Check if a member is in the party
    pub fn has_member(&self, member_id: &str) -> bool {
        self.members.contains(&member_id.to_string())
    }

    /// Add a member to the party
    pub fn add_member(&mut self, member_id: &str) {
        if !self.has_member(member_id) {
            self.members.push(member_id.to_string());
        }
    }

    /// Remove a member from the party
    /// Returns false if trying to remove a brother (they can never leave)
    pub fn remove_member(&mut self, member_id: &str) -> bool {
        // Brothers can NEVER leave the party
        if member_id == "herbert" || member_id == "valeran" {
            return false;
        }

        if let Some(pos) = self.members.iter().position(|m| m == member_id) {
            self.members.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get item count
    pub fn get_item_count(&self, item_id: &str) -> u32 {
        self.inventory.get(item_id).copied().unwrap_or(0)
    }

    /// Add items to inventory
    pub fn add_item(&mut self, item_id: &str, count: u32) {
        *self.inventory.entry(item_id.to_string()).or_insert(0) += count;
    }

    /// Remove items from inventory
    pub fn remove_item(&mut self, item_id: &str, count: u32) -> bool {
        if let Some(current) = self.inventory.get_mut(item_id) {
            if *current >= count {
                *current -= count;
                if *current == 0 {
                    self.inventory.remove(item_id);
                }
                return true;
            }
        }
        false
    }
}

/// Simple game state for event checking (combines story and party)
#[derive(Debug, Clone)]
pub struct GameStateForEvents {
    pub story: StoryState,
    pub party: Party,
    pub current_map: String,
    pub position: (u32, u32),
}

impl GameStateForEvents {
    pub fn new() -> Self {
        GameStateForEvents {
            story: StoryState::new(),
            party: Party::new(),
            current_map: "millbrook".to_string(),
            position: (0, 0),
        }
    }
}

impl Default for GameStateForEvents {
    fn default() -> Self {
        Self::new()
    }
}

/// The event system for checking and applying events
pub struct EventSystem;

impl EventSystem {
    /// Check if an event's conditions are all met
    pub fn check_event_trigger(event: &GameEvent, state: &GameStateForEvents) -> bool {
        event.trigger_conditions.iter().all(|cond| {
            Self::check_condition(cond, state)
        })
    }

    /// Check a single condition
    pub fn check_condition(condition: &EventCondition, state: &GameStateForEvents) -> bool {
        match condition {
            EventCondition::Flag(f) => state.story.has_flag(f),
            EventCondition::NotFlag(f) => !state.story.has_flag(f),
            EventCondition::ShrineDestroyed(n) => {
                *n < 5 && state.story.shrines_destroyed[*n]
            }
            EventCondition::ShrineNotDestroyed(n) => {
                *n >= 5 || !state.story.shrines_destroyed[*n]
            }
            EventCondition::WorldPhase(p) => state.story.world_phase == *p,
            EventCondition::WorldPhaseAtLeast(p) => {
                Self::phase_to_number(state.story.world_phase) >= Self::phase_to_number(*p)
            }
            EventCondition::PartyHas(id) => state.party.has_member(id),
            EventCondition::PartyDoesNotHave(id) => !state.party.has_member(id),
            EventCondition::AtLocation { map, x, y } => {
                state.current_map == *map && state.position == (*x, *y)
            }
            EventCondition::OnMap(map) => state.current_map == *map,
            EventCondition::ItemInInventory(item, count) => {
                state.party.get_item_count(item) >= *count
            }
            EventCondition::GoldAtLeast(amount) => state.party.gold >= *amount,
            EventCondition::ChapterIs(ch) => state.story.current_chapter == *ch,
            EventCondition::ChapterAtLeast(ch) => state.story.current_chapter >= *ch,
            EventCondition::HasMet(member) => state.story.has_met(member),
            EventCondition::RandomChance(percent) => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                rng.gen_range(0..100) < *percent
            }
            EventCondition::TimeOfDay(_) => {
                // Not implemented yet - always true
                true
            }
            EventCondition::BlessingActive => state.story.dorl_blessing_active,
            EventCondition::BlessingNotActive => !state.story.dorl_blessing_active,
        }
    }

    /// Convert world phase to a number for comparison
    fn phase_to_number(phase: WorldPhase) -> u8 {
        match phase {
            WorldPhase::Normal => 0,
            WorldPhase::Unsettled => 1,
            WorldPhase::Troubled => 2,
            WorldPhase::Dangerous => 3,
            WorldPhase::Dire => 4,
            WorldPhase::Apocalyptic => 5,
            WorldPhase::Redemption => 6,
        }
    }

    /// Apply all effects from an event
    pub fn apply_effects(effects: &[EventEffect], state: &mut GameStateForEvents) -> Vec<EventResult> {
        let mut results = Vec::new();

        for effect in effects {
            match Self::apply_effect(effect, state) {
                EventResult::Continue => {}
                result => results.push(result),
            }
        }

        results
    }

    /// Apply a single effect
    pub fn apply_effect(effect: &EventEffect, state: &mut GameStateForEvents) -> EventResult {
        match effect {
            EventEffect::SetFlag(flag, value) => {
                state.story.set_flag(flag, *value);
                EventResult::Continue
            }
            EventEffect::JoinParty(character_id) => {
                state.party.add_member(character_id);
                state.story.meet_party_member(character_id);
                EventResult::PartyJoined(character_id.clone())
            }
            EventEffect::LeaveParty(character_id) => {
                // Brothers can NEVER leave
                if character_id == "herbert" || character_id == "valeran" {
                    EventResult::Continue
                } else if state.party.remove_member(character_id) {
                    EventResult::PartyLeft(character_id.clone())
                } else {
                    EventResult::Continue
                }
            }
            EventEffect::GiveItem(item_id, count) => {
                state.party.add_item(item_id, *count);
                EventResult::ItemsReceived(item_id.clone(), *count)
            }
            EventEffect::TakeItem(item_id, count) => {
                state.party.remove_item(item_id, *count);
                EventResult::Continue
            }
            EventEffect::GiveGold(amount) => {
                if *amount >= 0 {
                    state.party.gold = state.party.gold.saturating_add(*amount as u32);
                } else {
                    state.party.gold = state.party.gold.saturating_sub(amount.unsigned_abs());
                }
                EventResult::GoldReceived(*amount)
            }
            EventEffect::Teleport { map, x, y } => {
                state.current_map = map.clone();
                state.position = (*x, *y);
                EventResult::Teleported {
                    map: map.clone(),
                    x: *x,
                    y: *y,
                }
            }
            EventEffect::StartBattle { enemies, boss, guardian } => {
                EventResult::BattleStart {
                    enemies: enemies.clone(),
                    boss: *boss,
                    guardian: *guardian,
                }
            }
            EventEffect::PlayCutscene(cutscene_id) => {
                EventResult::Cutscene(cutscene_id.clone())
            }
            EventEffect::Heal => {
                // Full heal would be handled by the game engine
                EventResult::Continue
            }
            EventEffect::AdvanceChapter => {
                state.story.advance_chapter();
                EventResult::MajorStoryEvent(format!("Chapter {}: {}",
                    state.story.current_chapter,
                    state.story.chapter_name()))
            }
            EventEffect::TriggerReveal => {
                state.story.trigger_reveal();
                EventResult::MajorStoryEvent("the_reveal".to_string())
            }
            EventEffect::SetWorldPhase(phase) => {
                state.story.world_phase = *phase;
                EventResult::Continue
            }
            EventEffect::ShowMessage(msg) => {
                EventResult::Message(msg.clone())
            }
            EventEffect::MeetMember(member_id) => {
                state.story.meet_party_member(member_id);
                EventResult::Continue
            }
            EventEffect::ActivateBlessing => {
                state.story.dorl_blessing_active = true;
                state.story.set_flag(flags::RECEIVED_BLESSING, true);
                EventResult::Continue
            }
            EventEffect::DeactivateBlessing => {
                state.story.dorl_blessing_active = false;
                EventResult::Continue
            }
            EventEffect::DestroyShrine(index) => {
                state.story.destroy_shrine(*index);
                EventResult::MajorStoryEvent(format!("shrine_{}_destroyed", index + 1))
            }
            EventEffect::CollectEcho(index) => {
                state.story.collect_echo(*index);
                EventResult::MajorStoryEvent(format!("echo_{}_collected", index + 1))
            }
            EventEffect::PlayMusic(_) | EventEffect::StopMusic | EventEffect::PlaySound(_) => {
                // Audio handled by game engine
                EventResult::Continue
            }
            EventEffect::ChangeMap(map) => {
                state.current_map = map.clone();
                state.position = (0, 0);
                EventResult::Continue
            }
        }
    }

    /// Find the first event that triggers from a list
    pub fn find_triggerable_event<'a>(
        events: &'a [GameEvent],
        state: &GameStateForEvents,
    ) -> Option<&'a GameEvent> {
        // Sort by priority (handled at call site if needed)
        events.iter().find(|event| {
            // Check if one-time event was already triggered
            if event.one_time && state.story.has_flag(&format!("event_{}_done", event.id)) {
                return false;
            }
            Self::check_event_trigger(event, state)
        })
    }

    /// Mark an event as completed (for one-time events)
    pub fn mark_event_done(event: &GameEvent, state: &mut GameStateForEvents) {
        if event.one_time {
            state.story.set_flag(&format!("event_{}_done", event.id), true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_party_creation() {
        let party = Party::new();
        assert!(party.has_member("herbert"));
        assert!(party.has_member("valeran"));
        assert!(!party.has_member("sera"));
    }

    #[test]
    fn test_party_member_management() {
        let mut party = Party::new();

        // Add member
        party.add_member("sera");
        assert!(party.has_member("sera"));

        // Remove member
        assert!(party.remove_member("sera"));
        assert!(!party.has_member("sera"));

        // Can't remove brothers
        assert!(!party.remove_member("herbert"));
        assert!(!party.remove_member("valeran"));
        assert!(party.has_member("herbert"));
        assert!(party.has_member("valeran"));
    }

    #[test]
    fn test_party_inventory() {
        let mut party = Party::new();

        assert_eq!(party.get_item_count("potion"), 0);

        party.add_item("potion", 5);
        assert_eq!(party.get_item_count("potion"), 5);

        party.add_item("potion", 3);
        assert_eq!(party.get_item_count("potion"), 8);

        assert!(party.remove_item("potion", 3));
        assert_eq!(party.get_item_count("potion"), 5);

        assert!(!party.remove_item("potion", 10)); // Can't remove more than we have
        assert_eq!(party.get_item_count("potion"), 5);
    }

    #[test]
    fn test_condition_flag() {
        let mut state = GameStateForEvents::new();

        assert!(!EventSystem::check_condition(
            &EventCondition::Flag("test".to_string()),
            &state
        ));

        state.story.set_flag("test", true);
        assert!(EventSystem::check_condition(
            &EventCondition::Flag("test".to_string()),
            &state
        ));
    }

    #[test]
    fn test_condition_not_flag() {
        let mut state = GameStateForEvents::new();

        assert!(EventSystem::check_condition(
            &EventCondition::NotFlag("test".to_string()),
            &state
        ));

        state.story.set_flag("test", true);
        assert!(!EventSystem::check_condition(
            &EventCondition::NotFlag("test".to_string()),
            &state
        ));
    }

    #[test]
    fn test_condition_shrine_destroyed() {
        let mut state = GameStateForEvents::new();

        assert!(!EventSystem::check_condition(
            &EventCondition::ShrineDestroyed(0),
            &state
        ));

        state.story.destroy_shrine(0);
        assert!(EventSystem::check_condition(
            &EventCondition::ShrineDestroyed(0),
            &state
        ));
    }

    #[test]
    fn test_condition_world_phase() {
        let mut state = GameStateForEvents::new();

        assert!(EventSystem::check_condition(
            &EventCondition::WorldPhase(WorldPhase::Normal),
            &state
        ));

        state.story.destroy_shrine(0);
        assert!(EventSystem::check_condition(
            &EventCondition::WorldPhase(WorldPhase::Unsettled),
            &state
        ));

        assert!(EventSystem::check_condition(
            &EventCondition::WorldPhaseAtLeast(WorldPhase::Normal),
            &state
        ));
        assert!(EventSystem::check_condition(
            &EventCondition::WorldPhaseAtLeast(WorldPhase::Unsettled),
            &state
        ));
        assert!(!EventSystem::check_condition(
            &EventCondition::WorldPhaseAtLeast(WorldPhase::Troubled),
            &state
        ));
    }

    #[test]
    fn test_condition_party_has() {
        let state = GameStateForEvents::new();

        assert!(EventSystem::check_condition(
            &EventCondition::PartyHas("herbert".to_string()),
            &state
        ));
        assert!(!EventSystem::check_condition(
            &EventCondition::PartyHas("sera".to_string()),
            &state
        ));
    }

    #[test]
    fn test_condition_location() {
        let mut state = GameStateForEvents::new();
        state.current_map = "thornwick".to_string();
        state.position = (15, 10);

        assert!(EventSystem::check_condition(
            &EventCondition::AtLocation {
                map: "thornwick".to_string(),
                x: 15,
                y: 10
            },
            &state
        ));

        assert!(!EventSystem::check_condition(
            &EventCondition::AtLocation {
                map: "thornwick".to_string(),
                x: 0,
                y: 0
            },
            &state
        ));

        assert!(EventSystem::check_condition(
            &EventCondition::OnMap("thornwick".to_string()),
            &state
        ));
    }

    #[test]
    fn test_condition_gold_and_items() {
        let mut state = GameStateForEvents::new();
        state.party.gold = 500;
        state.party.add_item("key", 1);

        assert!(EventSystem::check_condition(
            &EventCondition::GoldAtLeast(500),
            &state
        ));
        assert!(!EventSystem::check_condition(
            &EventCondition::GoldAtLeast(501),
            &state
        ));

        assert!(EventSystem::check_condition(
            &EventCondition::ItemInInventory("key".to_string(), 1),
            &state
        ));
        assert!(!EventSystem::check_condition(
            &EventCondition::ItemInInventory("key".to_string(), 2),
            &state
        ));
    }

    #[test]
    fn test_effect_set_flag() {
        let mut state = GameStateForEvents::new();

        EventSystem::apply_effect(
            &EventEffect::SetFlag("test".to_string(), true),
            &mut state
        );
        assert!(state.story.has_flag("test"));
    }

    #[test]
    fn test_effect_join_party() {
        let mut state = GameStateForEvents::new();

        let result = EventSystem::apply_effect(
            &EventEffect::JoinParty("sera".to_string()),
            &mut state
        );

        assert!(state.party.has_member("sera"));
        assert!(state.story.has_met("sera"));
        assert!(matches!(result, EventResult::PartyJoined(ref s) if s == "sera"));
    }

    #[test]
    fn test_effect_leave_party() {
        let mut state = GameStateForEvents::new();
        state.party.add_member("sera");

        let result = EventSystem::apply_effect(
            &EventEffect::LeaveParty("sera".to_string()),
            &mut state
        );

        assert!(!state.party.has_member("sera"));
        assert!(matches!(result, EventResult::PartyLeft(ref s) if s == "sera"));
    }

    #[test]
    fn test_effect_brothers_cannot_leave() {
        let mut state = GameStateForEvents::new();

        let result = EventSystem::apply_effect(
            &EventEffect::LeaveParty("herbert".to_string()),
            &mut state
        );

        // Herbert should still be in the party
        assert!(state.party.has_member("herbert"));
        assert!(matches!(result, EventResult::Continue));

        let result = EventSystem::apply_effect(
            &EventEffect::LeaveParty("valeran".to_string()),
            &mut state
        );

        // Valeran should still be in the party
        assert!(state.party.has_member("valeran"));
        assert!(matches!(result, EventResult::Continue));
    }

    #[test]
    fn test_effect_give_gold() {
        let mut state = GameStateForEvents::new();
        let initial_gold = state.party.gold;

        EventSystem::apply_effect(
            &EventEffect::GiveGold(100),
            &mut state
        );
        assert_eq!(state.party.gold, initial_gold + 100);

        EventSystem::apply_effect(
            &EventEffect::GiveGold(-50),
            &mut state
        );
        assert_eq!(state.party.gold, initial_gold + 50);
    }

    #[test]
    fn test_effect_teleport() {
        let mut state = GameStateForEvents::new();

        let result = EventSystem::apply_effect(
            &EventEffect::Teleport {
                map: "castle".to_string(),
                x: 10,
                y: 20
            },
            &mut state
        );

        assert_eq!(state.current_map, "castle");
        assert_eq!(state.position, (10, 20));
        assert!(matches!(result, EventResult::Teleported { .. }));
    }

    #[test]
    fn test_effect_trigger_reveal() {
        let mut state = GameStateForEvents::new();
        state.story.dorl_blessing_active = true;

        let result = EventSystem::apply_effect(
            &EventEffect::TriggerReveal,
            &mut state
        );

        assert!(!state.story.dorl_blessing_active);
        assert!(state.story.has_flag(flags::THE_REVEAL));
        assert!(matches!(result, EventResult::MajorStoryEvent(ref s) if s == "the_reveal"));
    }

    #[test]
    fn test_effect_activate_blessing() {
        let mut state = GameStateForEvents::new();

        EventSystem::apply_effect(
            &EventEffect::ActivateBlessing,
            &mut state
        );

        assert!(state.story.dorl_blessing_active);
        assert!(state.story.has_flag(flags::RECEIVED_BLESSING));
    }

    #[test]
    fn test_effect_destroy_shrine() {
        let mut state = GameStateForEvents::new();

        let result = EventSystem::apply_effect(
            &EventEffect::DestroyShrine(0),
            &mut state
        );

        assert!(state.story.shrines_destroyed[0]);
        assert!(state.story.has_flag(flags::SHRINE_1_COMPLETE));
        assert!(matches!(result, EventResult::MajorStoryEvent(_)));
    }

    #[test]
    fn test_event_builder() {
        let event = GameEvent::new("test_event", "Test Event")
            .with_condition(EventCondition::Flag("some_flag".to_string()))
            .with_effect(EventEffect::ShowMessage("Hello!".to_string()))
            .with_cutscene("test_cutscene")
            .with_priority(100)
            .repeatable();

        assert_eq!(event.id, "test_event");
        assert_eq!(event.name, "Test Event");
        assert_eq!(event.trigger_conditions.len(), 1);
        assert_eq!(event.effects.len(), 1);
        assert_eq!(event.cutscene_id, Some("test_cutscene".to_string()));
        assert_eq!(event.priority, 100);
        assert!(!event.one_time);
    }

    #[test]
    fn test_check_event_trigger() {
        let mut state = GameStateForEvents::new();

        let event = GameEvent::new("test", "Test")
            .with_condition(EventCondition::Flag("required_flag".to_string()))
            .with_condition(EventCondition::NotFlag("blocking_flag".to_string()));

        // Neither condition met initially (flag not set)
        assert!(!EventSystem::check_event_trigger(&event, &state));

        // Set required flag
        state.story.set_flag("required_flag", true);
        assert!(EventSystem::check_event_trigger(&event, &state));

        // Set blocking flag
        state.story.set_flag("blocking_flag", true);
        assert!(!EventSystem::check_event_trigger(&event, &state));
    }

    #[test]
    fn test_apply_multiple_effects() {
        let mut state = GameStateForEvents::new();

        let effects = vec![
            EventEffect::SetFlag("test1".to_string(), true),
            EventEffect::GiveGold(100),
            EventEffect::JoinParty("sera".to_string()),
        ];

        let results = EventSystem::apply_effects(&effects, &mut state);

        assert!(state.story.has_flag("test1"));
        assert_eq!(state.party.gold, 200); // 100 starting + 100
        assert!(state.party.has_member("sera"));

        // Should have results for gold and party join
        assert!(results.iter().any(|r| matches!(r, EventResult::GoldReceived(100))));
        assert!(results.iter().any(|r| matches!(r, EventResult::PartyJoined(s) if s == "sera")));
    }

    #[test]
    fn test_find_triggerable_event() {
        let state = GameStateForEvents::new();

        let events = vec![
            GameEvent::new("event1", "Event 1")
                .with_condition(EventCondition::Flag("not_set".to_string())),
            GameEvent::new("event2", "Event 2")
                .with_condition(EventCondition::PartyHas("herbert".to_string())),
        ];

        let found = EventSystem::find_triggerable_event(&events, &state);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "event2");
    }

    #[test]
    fn test_mark_event_done() {
        let mut state = GameStateForEvents::new();
        let event = GameEvent::new("test_event", "Test");

        EventSystem::mark_event_done(&event, &mut state);

        assert!(state.story.has_flag("event_test_event_done"));
    }

    #[test]
    fn test_one_time_event_blocking() {
        let mut state = GameStateForEvents::new();

        let events = vec![
            GameEvent::new("one_time_event", "One Time")
                .with_condition(EventCondition::PartyHas("herbert".to_string())),
        ];

        // Should find the event
        assert!(EventSystem::find_triggerable_event(&events, &state).is_some());

        // Mark it done
        EventSystem::mark_event_done(&events[0], &mut state);

        // Should not find it anymore
        assert!(EventSystem::find_triggerable_event(&events, &state).is_none());
    }

    #[test]
    fn test_condition_blessing_active() {
        let mut state = GameStateForEvents::new();

        assert!(EventSystem::check_condition(
            &EventCondition::BlessingNotActive,
            &state
        ));
        assert!(!EventSystem::check_condition(
            &EventCondition::BlessingActive,
            &state
        ));

        state.story.dorl_blessing_active = true;

        assert!(!EventSystem::check_condition(
            &EventCondition::BlessingNotActive,
            &state
        ));
        assert!(EventSystem::check_condition(
            &EventCondition::BlessingActive,
            &state
        ));
    }

    #[test]
    fn test_condition_chapter() {
        let mut state = GameStateForEvents::new();

        assert!(EventSystem::check_condition(
            &EventCondition::ChapterIs(1),
            &state
        ));
        assert!(EventSystem::check_condition(
            &EventCondition::ChapterAtLeast(1),
            &state
        ));
        assert!(!EventSystem::check_condition(
            &EventCondition::ChapterAtLeast(2),
            &state
        ));

        state.story.advance_chapter();

        assert!(EventSystem::check_condition(
            &EventCondition::ChapterIs(2),
            &state
        ));
        assert!(EventSystem::check_condition(
            &EventCondition::ChapterAtLeast(1),
            &state
        ));
        assert!(EventSystem::check_condition(
            &EventCondition::ChapterAtLeast(2),
            &state
        ));
    }
}
