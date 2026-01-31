//! Dialogue Engine for Realm of Ralnar
//!
//! A branching dialogue system with conditions and effects.
//! Supports complex dialogue trees with multiple choices and state changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::state::GameState;

// ============================================================================
// DIALOGUE TREE STRUCTURES
// ============================================================================

/// A complete dialogue tree with nodes and entry point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    /// Unique identifier for this dialogue
    pub id: String,
    /// All nodes in the dialogue tree
    pub nodes: HashMap<String, DialogueNode>,
    /// Which node to start at
    pub entry_point: String,
}

/// A single node in the dialogue tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    /// Who is speaking (None for narration)
    pub speaker: Option<String>,
    /// The text being spoken
    pub text: String,
    /// Conditions that must be met to show this node
    pub conditions: Vec<Condition>,
    /// Available player choices
    pub choices: Vec<DialogueChoice>,
    /// Effects to apply when this node is reached
    pub effects: Vec<DialogueEffect>,
    /// Next node if no choices (None means end dialogue)
    pub next: Option<String>,
}

impl Default for DialogueNode {
    fn default() -> Self {
        Self {
            speaker: None,
            text: String::new(),
            conditions: Vec::new(),
            choices: Vec::new(),
            effects: Vec::new(),
            next: None,
        }
    }
}

/// A choice the player can make in dialogue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    /// Text shown to the player
    pub text: String,
    /// Node to go to when chosen
    pub next_node: String,
    /// Conditions that must be met to show this choice
    pub conditions: Vec<Condition>,
}

impl DialogueChoice {
    /// Create a simple unconditional choice
    pub fn new(text: &str, next_node: &str) -> Self {
        Self {
            text: text.to_string(),
            next_node: next_node.to_string(),
            conditions: Vec::new(),
        }
    }

    /// Create a choice with conditions
    pub fn with_conditions(text: &str, next_node: &str, conditions: Vec<Condition>) -> Self {
        Self {
            text: text.to_string(),
            next_node: next_node.to_string(),
            conditions,
        }
    }
}

// ============================================================================
// CONDITIONS
// ============================================================================

/// Conditions that can be checked for dialogue branching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Condition {
    /// Check if a story flag is set
    HasFlag(String),
    /// Check if a story flag is NOT set
    NotFlag(String),
    /// Check if player has at least N of an item
    HasItem(String, u32),
    /// Check if player has at least N gold
    HasGold(u32),
    /// Check if a quest is currently active
    QuestActive(String),
    /// Check if a quest has been completed
    QuestComplete(String),
    /// Check if N shrines have been destroyed (1-indexed)
    ShrineDestroyed(u8),
    /// Check if world phase is at least N
    WorldPhase(u8),
    /// Check if a specific character is in the party
    PartyHas(String),
    /// Check if player level is at least N
    LevelAtLeast(u8),
    /// Check if a specific shrine has been found by name
    ShrineFound(String),
    /// Compound: All conditions must be true
    All(Vec<Condition>),
    /// Compound: Any condition must be true
    Any(Vec<Condition>),
}

impl Condition {
    /// Evaluate this condition against the current game state
    pub fn evaluate(&self, state: &GameState) -> bool {
        match self {
            Condition::HasFlag(flag) => state.has_flag(flag),
            Condition::NotFlag(flag) => !state.has_flag(flag),
            Condition::HasItem(item, qty) => state.inventory.has(item, *qty),
            Condition::HasGold(amount) => state.gold >= *amount,
            Condition::QuestActive(quest_id) => {
                // Check if quest is in active state
                state.has_flag(&format!("quest_active_{}", quest_id))
            }
            Condition::QuestComplete(quest_id) => {
                state.has_flag(&format!("quest_complete_{}", quest_id))
            }
            Condition::ShrineDestroyed(n) => {
                // Count destroyed shrines
                state.shrines_destroyed_count() >= *n as usize
            }
            Condition::WorldPhase(phase) => state.world_phase >= *phase,
            Condition::PartyHas(member) => {
                // Check if party member with matching ID exists
                state.party.get_member(&member.to_lowercase()).is_some()
            }
            Condition::LevelAtLeast(level) => {
                // Use party leader's level
                state.party.leader()
                    .map(|leader| leader.level >= *level)
                    .unwrap_or(false)
            }
            Condition::ShrineFound(shrine) => {
                // Map shrine name to index and check
                let shrine_index = match shrine.as_str() {
                    "shrine_of_fire" | "fire" => Some(0),
                    "shrine_of_water" | "water" => Some(1),
                    "shrine_of_earth" | "earth" => Some(2),
                    "shrine_of_wind" | "wind" => Some(3),
                    "shrine_of_void" | "void" => Some(4),
                    _ => None,
                };
                shrine_index.map(|i| state.shrines_destroyed[i]).unwrap_or(false)
            }
            Condition::All(conditions) => conditions.iter().all(|c| c.evaluate(state)),
            Condition::Any(conditions) => conditions.iter().any(|c| c.evaluate(state)),
        }
    }
}

// ============================================================================
// EFFECTS
// ============================================================================

/// Effects that can be triggered by dialogue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DialogueEffect {
    /// Set a story flag
    SetFlag(String, bool),
    /// Give items to the player
    GiveItem(String, u32),
    /// Take items from the player
    TakeItem(String, u32),
    /// Give gold (can be negative to take)
    GiveGold(i32),
    /// Take gold from the player
    TakeGold(u32),
    /// Start a quest
    StartQuest(String),
    /// Complete a quest
    CompleteQuest(String),
    /// Fully heal the party
    Heal,
    /// Teleport to a location
    Teleport { map: String, x: u32, y: u32 },
    /// Add a character to the party
    JoinParty(String),
    /// Remove a character from the party
    LeaveParty(String),
    /// Start a battle with specific enemies
    StartBattle { enemies: Vec<String> },
    /// Play a cutscene
    PlayCutscene(String),
    /// Change companion loyalty
    ChangeLoyalty { companion: String, amount: i32 },
    /// Give experience points
    GiveExp(i32),
    /// Mark a shrine as found/destroyed
    MarkShrine(String),
}

impl DialogueEffect {
    /// Apply this effect to the game state
    /// Returns a description of what happened for display
    pub fn apply(&self, state: &mut GameState) -> Option<String> {
        match self {
            DialogueEffect::SetFlag(flag, value) => {
                if *value {
                    state.set_flag(flag);
                } else {
                    state.clear_flag(flag);
                }
                None
            }
            DialogueEffect::GiveItem(item, qty) => {
                state.inventory.add(item, *qty);
                Some(format!("Received {} x{}", item, qty))
            }
            DialogueEffect::TakeItem(item, qty) => {
                state.inventory.remove(item, *qty);
                Some(format!("Lost {} x{}", item, qty))
            }
            DialogueEffect::GiveGold(amount) => {
                if *amount >= 0 {
                    state.gold += *amount as u32;
                    Some(format!("Received {} gold", amount))
                } else {
                    state.gold = state.gold.saturating_sub((-*amount) as u32);
                    Some(format!("Lost {} gold", -amount))
                }
            }
            DialogueEffect::TakeGold(amount) => {
                state.gold = state.gold.saturating_sub(*amount);
                Some(format!("Lost {} gold", amount))
            }
            DialogueEffect::StartQuest(quest_id) => {
                state.set_flag(&format!("quest_active_{}", quest_id));
                Some(format!("Quest started: {}", quest_id))
            }
            DialogueEffect::CompleteQuest(quest_id) => {
                state.clear_flag(&format!("quest_active_{}", quest_id));
                state.set_flag(&format!("quest_complete_{}", quest_id));
                Some(format!("Quest completed: {}", quest_id))
            }
            DialogueEffect::Heal => {
                state.party.full_heal();
                Some("Party fully healed!".to_string())
            }
            DialogueEffect::Teleport { map, x, y } => {
                state.current_map = map.clone();
                state.position = (*x, *y);
                Some(format!("Traveled to {}", map))
            }
            DialogueEffect::JoinParty(member) => {
                // Note: Actual party member creation should be done by game engine
                // This just sets a flag indicating they should join
                state.set_flag(&format!("party_join_{}", member.to_lowercase()));
                Some(format!("{} joined the party!", member))
            }
            DialogueEffect::LeaveParty(member) => {
                // Attempt to remove non-brother party members
                let _ = state.party.remove_member(&member.to_lowercase());
                Some(format!("{} left the party.", member))
            }
            DialogueEffect::StartBattle { enemies: _ } => {
                // Battle is handled by the game engine
                Some("A battle begins!".to_string())
            }
            DialogueEffect::PlayCutscene(scene_id) => {
                state.set_flag(&format!("seen_cutscene_{}", scene_id));
                None // Cutscene handled by renderer
            }
            DialogueEffect::ChangeLoyalty { companion: _, amount: _ } => {
                // Loyalty is not tracked in the current GameState
                // Could be added to PartyMember in the future
                None
            }
            DialogueEffect::GiveExp(amount) => {
                // Give exp to party leader
                if let Some(leader) = state.party.members.first_mut() {
                    leader.exp += *amount as u32;
                }
                Some(format!("Gained {} EXP", amount))
            }
            DialogueEffect::MarkShrine(shrine) => {
                // Map shrine name to index
                let shrine_index = match shrine.as_str() {
                    "shrine_of_fire" | "fire" | "shrine_1" => Some(0),
                    "shrine_of_water" | "water" | "shrine_2" => Some(1),
                    "shrine_of_earth" | "earth" | "shrine_3" => Some(2),
                    "shrine_of_wind" | "wind" | "shrine_4" => Some(3),
                    "shrine_of_void" | "void" | "shrine_5" => Some(4),
                    _ => None,
                };
                if let Some(idx) = shrine_index {
                    state.shrines_destroyed[idx] = true;
                    state.update_world_phase();
                }
                Some(format!("Shrine destroyed: {}", shrine))
            }
        }
    }
}

// ============================================================================
// DIALOGUE STATE
// ============================================================================

/// Active dialogue state during a conversation
#[derive(Debug)]
pub struct DialogueState {
    /// The dialogue tree being used
    pub tree: DialogueTree,
    /// Current node ID
    pub current_node: String,
    /// Indices of choices that meet their conditions
    pub available_choices: Vec<usize>,
    /// Messages generated by effects
    pub effect_messages: Vec<String>,
    /// Whether the dialogue has ended
    pub finished: bool,
}

impl DialogueState {
    /// Create a new dialogue state from a tree
    pub fn new(tree: DialogueTree, game_state: &GameState) -> Self {
        let entry = tree.entry_point.clone();
        let mut state = Self {
            tree,
            current_node: entry,
            available_choices: Vec::new(),
            effect_messages: Vec::new(),
            finished: false,
        };
        state.update_available_choices(game_state);
        state
    }

    /// Get the current node
    fn current(&self) -> Option<&DialogueNode> {
        self.tree.nodes.get(&self.current_node)
    }

    /// Get the current text being displayed
    pub fn current_text(&self) -> &str {
        self.current()
            .map(|n| n.text.as_str())
            .unwrap_or("")
    }

    /// Get the current speaker (if any)
    pub fn speaker(&self) -> Option<&str> {
        self.current()
            .and_then(|n| n.speaker.as_deref())
    }

    /// Get available choices for the current node
    pub fn get_choices(&self) -> Vec<&DialogueChoice> {
        if let Some(node) = self.current() {
            self.available_choices
                .iter()
                .filter_map(|&i| node.choices.get(i))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Update which choices are available based on conditions
    fn update_available_choices(&mut self, game_state: &GameState) {
        self.available_choices.clear();
        // Collect valid indices first, then assign to avoid borrow conflict
        let valid_indices: Vec<usize> = if let Some(node) = self.current() {
            node.choices
                .iter()
                .enumerate()
                .filter(|(_, choice)| choice.conditions.iter().all(|c| c.evaluate(game_state)))
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };
        self.available_choices = valid_indices;
    }

    /// Select a choice and advance dialogue
    /// Returns effects to be applied
    pub fn select_choice(&mut self, choice_index: usize, game_state: &GameState) -> Vec<DialogueEffect> {
        let mut effects = Vec::new();

        if let Some(&actual_index) = self.available_choices.get(choice_index) {
            if let Some(node) = self.current().cloned() {
                if let Some(choice) = node.choices.get(actual_index) {
                    self.current_node = choice.next_node.clone();

                    // Get effects from new node
                    if let Some(new_node) = self.current() {
                        effects = new_node.effects.clone();
                    }

                    self.update_available_choices(game_state);

                    // Check if we've reached an end
                    if let Some(new_node) = self.current() {
                        if new_node.choices.is_empty() && new_node.next.is_none() {
                            self.finished = true;
                        }
                    } else {
                        self.finished = true;
                    }
                }
            }
        }

        effects
    }

    /// Advance dialogue when there are no choices
    /// Returns effects to be applied, or None if dialogue is over
    pub fn advance(&mut self, game_state: &GameState) -> Option<Vec<DialogueEffect>> {
        if self.finished {
            return None;
        }

        let node = self.current()?.clone();

        // If there are choices, can't auto-advance
        if !node.choices.is_empty() {
            return Some(Vec::new());
        }

        // Move to next node
        if let Some(next) = &node.next {
            self.current_node = next.clone();

            // Get effects from new node
            let effects = self.current()
                .map(|n| n.effects.clone())
                .unwrap_or_default();

            self.update_available_choices(game_state);

            // Check if we've reached an end
            if let Some(new_node) = self.current() {
                if new_node.choices.is_empty() && new_node.next.is_none() {
                    // This is the last node, but don't mark finished until they advance again
                }
            } else {
                self.finished = true;
            }

            Some(effects)
        } else {
            // No next node, dialogue is finished
            self.finished = true;
            None
        }
    }

    /// Check if the dialogue has finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Apply effects and collect messages
    pub fn apply_effects(&mut self, effects: &[DialogueEffect], game_state: &mut GameState) {
        for effect in effects {
            if let Some(msg) = effect.apply(game_state) {
                self.effect_messages.push(msg);
            }
        }
    }

    /// Take and clear effect messages
    pub fn take_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.effect_messages)
    }
}

// ============================================================================
// DIALOGUE BUILDER HELPERS
// ============================================================================

/// Builder for creating dialogue trees more easily
pub struct DialogueBuilder {
    id: String,
    nodes: HashMap<String, DialogueNode>,
    entry_point: String,
}

impl DialogueBuilder {
    /// Create a new dialogue builder
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            nodes: HashMap::new(),
            entry_point: "start".to_string(),
        }
    }

    /// Set the entry point
    pub fn entry(mut self, node_id: &str) -> Self {
        self.entry_point = node_id.to_string();
        self
    }

    /// Add a simple text node with optional next
    pub fn node(mut self, id: &str, speaker: Option<&str>, text: &str, next: Option<&str>) -> Self {
        self.nodes.insert(id.to_string(), DialogueNode {
            speaker: speaker.map(String::from),
            text: text.to_string(),
            conditions: Vec::new(),
            choices: Vec::new(),
            effects: Vec::new(),
            next: next.map(String::from),
        });
        self
    }

    /// Add a node with choices
    pub fn choice_node(mut self, id: &str, speaker: Option<&str>, text: &str, choices: Vec<DialogueChoice>) -> Self {
        self.nodes.insert(id.to_string(), DialogueNode {
            speaker: speaker.map(String::from),
            text: text.to_string(),
            conditions: Vec::new(),
            choices,
            effects: Vec::new(),
            next: None,
        });
        self
    }

    /// Add a node with effects
    pub fn effect_node(mut self, id: &str, speaker: Option<&str>, text: &str, effects: Vec<DialogueEffect>, next: Option<&str>) -> Self {
        self.nodes.insert(id.to_string(), DialogueNode {
            speaker: speaker.map(String::from),
            text: text.to_string(),
            conditions: Vec::new(),
            choices: Vec::new(),
            effects,
            next: next.map(String::from),
        });
        self
    }

    /// Modify an existing node to add conditions
    pub fn with_conditions(mut self, id: &str, conditions: Vec<Condition>) -> Self {
        if let Some(node) = self.nodes.get_mut(id) {
            node.conditions = conditions;
        }
        self
    }

    /// Build the final dialogue tree
    pub fn build(self) -> DialogueTree {
        DialogueTree {
            id: self.id,
            nodes: self.nodes,
            entry_point: self.entry_point,
        }
    }
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

    fn create_simple_dialogue() -> DialogueTree {
        DialogueBuilder::new("test_dialogue")
            .node("start", Some("NPC"), "Hello traveler!", Some("response"))
            .node("response", Some("Player"), "Hello!", None)
            .build()
    }

    fn create_choice_dialogue() -> DialogueTree {
        DialogueBuilder::new("choice_dialogue")
            .choice_node("start", Some("NPC"), "What would you like?", vec![
                DialogueChoice::new("Buy items", "shop"),
                DialogueChoice::new("Chat", "chat"),
                DialogueChoice::new("Leave", "bye"),
            ])
            .node("shop", Some("NPC"), "Here are my wares.", None)
            .node("chat", Some("NPC"), "Nice weather today.", None)
            .node("bye", Some("NPC"), "Farewell!", None)
            .build()
    }

    fn create_conditional_dialogue() -> DialogueTree {
        DialogueBuilder::new("conditional_dialogue")
            .choice_node("start", Some("NPC"), "Can I help you?", vec![
                DialogueChoice::with_conditions(
                    "I have the gold",
                    "rich_response",
                    vec![Condition::HasGold(200)] // Higher than starting gold
                ),
                DialogueChoice::new("Just looking", "browse"),
            ])
            .node("rich_response", Some("NPC"), "A wealthy customer!", None)
            .node("browse", Some("NPC"), "Take your time.", None)
            .build()
    }

    #[test]
    fn test_dialogue_creation() {
        let dialogue = create_simple_dialogue();
        assert_eq!(dialogue.id, "test_dialogue");
        assert_eq!(dialogue.entry_point, "start");
        assert_eq!(dialogue.nodes.len(), 2);
    }

    #[test]
    fn test_dialogue_state_initialization() {
        let state = create_test_state();
        let dialogue = create_simple_dialogue();
        let ds = DialogueState::new(dialogue, &state);

        assert_eq!(ds.current_node, "start");
        assert!(!ds.is_finished());
        assert_eq!(ds.current_text(), "Hello traveler!");
        assert_eq!(ds.speaker(), Some("NPC"));
    }

    #[test]
    fn test_dialogue_advance() {
        let state = create_test_state();
        let dialogue = create_simple_dialogue();
        let mut ds = DialogueState::new(dialogue, &state);

        // Advance from start to response
        let effects = ds.advance(&state);
        assert!(effects.is_some());
        assert_eq!(ds.current_text(), "Hello!");
        assert!(!ds.is_finished());

        // Advance past last node
        let effects = ds.advance(&state);
        assert!(effects.is_none());
        assert!(ds.is_finished());
    }

    #[test]
    fn test_dialogue_choices() {
        let state = create_test_state();
        let dialogue = create_choice_dialogue();
        let mut ds = DialogueState::new(dialogue, &state);

        let choices = ds.get_choices();
        assert_eq!(choices.len(), 3);
        assert_eq!(choices[0].text, "Buy items");
        assert_eq!(choices[1].text, "Chat");
        assert_eq!(choices[2].text, "Leave");

        // Select "Chat" option (index 1)
        ds.select_choice(1, &state);
        assert_eq!(ds.current_text(), "Nice weather today.");
    }

    #[test]
    fn test_conditional_choices() {
        let mut state = create_test_state();
        state.gold = 50; // Not enough gold (need 200)

        let dialogue = create_conditional_dialogue();
        let ds = DialogueState::new(dialogue, &state);

        // Should only have 1 choice (not enough gold for first)
        let choices = ds.get_choices();
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].text, "Just looking");

        // Now give enough gold
        state.gold = 200;
        let dialogue = create_conditional_dialogue();
        let ds = DialogueState::new(dialogue, &state);
        let choices = ds.get_choices();
        assert_eq!(choices.len(), 2);
    }

    #[test]
    fn test_condition_has_flag() {
        let mut state = create_test_state();

        let cond = Condition::HasFlag("test_flag".to_string());
        assert!(!cond.evaluate(&state));

        state.set_flag("test_flag");
        assert!(cond.evaluate(&state));
    }

    #[test]
    fn test_condition_not_flag() {
        let mut state = create_test_state();

        let cond = Condition::NotFlag("test_flag".to_string());
        assert!(cond.evaluate(&state));

        state.set_flag("test_flag");
        assert!(!cond.evaluate(&state));
    }

    #[test]
    fn test_condition_has_item() {
        let mut state = create_test_state();

        let cond = Condition::HasItem("health_potion".to_string(), 2);
        assert!(!cond.evaluate(&state));

        state.inventory.add("health_potion", 1);
        assert!(!cond.evaluate(&state)); // Only 1

        state.inventory.add("health_potion", 1);
        assert!(cond.evaluate(&state)); // Now 2
    }

    #[test]
    fn test_condition_has_gold() {
        let mut state = create_test_state();
        state.gold = 50;

        let cond = Condition::HasGold(100);
        assert!(!cond.evaluate(&state));

        state.gold = 100;
        assert!(cond.evaluate(&state));
    }

    #[test]
    fn test_condition_party_has() {
        let state = create_test_state();

        // Check for herbert (the starting character)
        let cond = Condition::PartyHas("herbert".to_string());
        assert!(cond.evaluate(&state)); // Herbert starts in party
    }

    #[test]
    fn test_condition_compound_all() {
        let mut state = create_test_state();
        state.gold = 100;

        let cond = Condition::All(vec![
            Condition::HasGold(100),
            Condition::HasFlag("test_flag".to_string()),
        ]);

        assert!(!cond.evaluate(&state)); // Missing flag

        state.set_flag("test_flag");
        assert!(cond.evaluate(&state)); // Both conditions met
    }

    #[test]
    fn test_condition_compound_any() {
        let mut state = create_test_state();
        state.gold = 50; // Less than 1000

        let cond = Condition::Any(vec![
            Condition::HasGold(1000),
            Condition::HasFlag("test_flag".to_string()),
        ]);

        assert!(!cond.evaluate(&state)); // Neither met

        state.set_flag("test_flag");
        assert!(cond.evaluate(&state)); // One is enough
    }

    #[test]
    fn test_effect_set_flag() {
        let mut state = create_test_state();

        let effect = DialogueEffect::SetFlag("new_flag".to_string(), true);
        effect.apply(&mut state);
        assert!(state.has_flag("new_flag"));

        let effect = DialogueEffect::SetFlag("new_flag".to_string(), false);
        effect.apply(&mut state);
        assert!(!state.has_flag("new_flag"));
    }

    #[test]
    fn test_effect_give_item() {
        let mut state = create_test_state();

        let effect = DialogueEffect::GiveItem("magic_sword".to_string(), 1);
        let msg = effect.apply(&mut state);

        assert!(state.inventory.has("magic_sword", 1));
        assert_eq!(msg, Some("Received magic_sword x1".to_string()));
    }

    #[test]
    fn test_effect_take_item() {
        let mut state = create_test_state();
        state.inventory.add("test_potion", 2);

        let effect = DialogueEffect::TakeItem("test_potion".to_string(), 1);
        effect.apply(&mut state);

        assert_eq!(state.inventory.count("test_potion"), 1);
    }

    #[test]
    fn test_effect_gold() {
        let mut state = create_test_state();
        state.gold = 100;

        let effect = DialogueEffect::GiveGold(50);
        effect.apply(&mut state);
        assert_eq!(state.gold, 150);

        let effect = DialogueEffect::TakeGold(30);
        effect.apply(&mut state);
        assert_eq!(state.gold, 120);
    }

    #[test]
    fn test_effect_heal() {
        let mut state = create_test_state();
        // Damage the party
        if let Some(member) = state.party.members.first_mut() {
            member.hp = member.hp_max / 2;
        }

        let effect = DialogueEffect::Heal;
        effect.apply(&mut state);

        // Check party is healed
        if let Some(member) = state.party.members.first() {
            assert_eq!(member.hp, member.hp_max);
        }
    }

    #[test]
    fn test_effect_party_join() {
        let mut state = create_test_state();

        // JoinParty sets a flag for the game engine to handle
        let effect = DialogueEffect::JoinParty("valeran".to_string());
        effect.apply(&mut state);
        assert!(state.has_flag("party_join_valeran"));
    }

    #[test]
    fn test_dialogue_with_effects() {
        let mut state = create_test_state();
        state.gold = 100;

        let dialogue = DialogueBuilder::new("effect_test")
            .effect_node(
                "start",
                Some("NPC"),
                "Take this reward!",
                vec![
                    DialogueEffect::GiveGold(50),
                    DialogueEffect::SetFlag("received_reward".to_string(), true),
                ],
                None
            )
            .build();

        let mut ds = DialogueState::new(dialogue, &state);

        // Get effects from current node
        let effects = ds.current().unwrap().effects.clone();
        ds.apply_effects(&effects, &mut state);

        assert_eq!(state.gold, 150);
        assert!(state.has_flag("received_reward"));

        let messages = ds.take_messages();
        assert!(messages.contains(&"Received 50 gold".to_string()));
    }

    #[test]
    fn test_dialogue_builder() {
        let dialogue = DialogueBuilder::new("builder_test")
            .entry("intro")
            .node("intro", Some("Guide"), "Welcome!", Some("main"))
            .choice_node("main", None, "What now?", vec![
                DialogueChoice::new("Explore", "explore"),
                DialogueChoice::new("Rest", "rest"),
            ])
            .node("explore", None, "You venture forth.", None)
            .node("rest", None, "You take a break.", None)
            .build();

        assert_eq!(dialogue.entry_point, "intro");
        assert_eq!(dialogue.nodes.len(), 4);
        assert!(dialogue.nodes.contains_key("intro"));
        assert!(dialogue.nodes.contains_key("main"));
    }

    #[test]
    fn test_quest_conditions() {
        let mut state = create_test_state();

        let active_cond = Condition::QuestActive("main_quest".to_string());
        let complete_cond = Condition::QuestComplete("main_quest".to_string());

        assert!(!active_cond.evaluate(&state));
        assert!(!complete_cond.evaluate(&state));

        // Start quest
        DialogueEffect::StartQuest("main_quest".to_string()).apply(&mut state);
        assert!(active_cond.evaluate(&state));
        assert!(!complete_cond.evaluate(&state));

        // Complete quest
        DialogueEffect::CompleteQuest("main_quest".to_string()).apply(&mut state);
        assert!(!active_cond.evaluate(&state));
        assert!(complete_cond.evaluate(&state));
    }

    #[test]
    fn test_shrine_conditions() {
        let mut state = create_test_state();

        let cond = Condition::ShrineDestroyed(1);
        assert!(!cond.evaluate(&state));

        state.shrines_destroyed[0] = true;
        assert!(cond.evaluate(&state));

        let cond = Condition::ShrineDestroyed(2);
        assert!(!cond.evaluate(&state)); // Only 1 destroyed

        state.shrines_destroyed[1] = true;
        assert!(cond.evaluate(&state)); // Now 2
    }

    #[test]
    fn test_world_phase_condition() {
        let mut state = create_test_state();

        let cond = Condition::WorldPhase(2);
        assert!(!cond.evaluate(&state)); // Starts at 0

        state.world_phase = 2;
        assert!(cond.evaluate(&state));

        state.world_phase = 3;
        assert!(cond.evaluate(&state)); // 3 >= 2
    }
}
