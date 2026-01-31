//! Quest System for Realm of Ralnar
//!
//! Tracks main story and side quests, objectives, and rewards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::state::GameState;

// ============================================================================
// QUEST REQUIREMENT
// ============================================================================

/// Requirements for completing a quest objective
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestRequirement {
    /// Talk to a specific NPC
    TalkTo(String),
    /// Defeat N enemies of a type
    DefeatEnemy(String, u32),
    /// Collect N of an item
    CollectItem(String, u32),
    /// Reach a specific location
    ReachLocation(String),
    /// Have a specific flag set
    SetFlag(String),
    /// Have minimum gold
    HaveGold(u32),
    /// Have a party member
    HavePartyMember(String),
    /// Complete another quest
    CompleteQuest(String),
    /// Find a specific shrine
    FindShrine(String),
}

impl QuestRequirement {
    /// Check if this requirement is satisfied
    pub fn is_satisfied(&self, state: &GameState) -> bool {
        match self {
            QuestRequirement::TalkTo(npc_id) => {
                state.has_flag(&format!("talked_to_{}", npc_id))
            }
            QuestRequirement::DefeatEnemy(enemy_type, count) => {
                let killed = state.story_flags.keys()
                    .find(|f| f.starts_with(&format!("killed_{}_", enemy_type)))
                    .and_then(|f| f.split('_').last())
                    .and_then(|n| n.parse::<u32>().ok())
                    .unwrap_or(0);
                killed >= *count
            }
            QuestRequirement::CollectItem(item_id, count) => {
                state.inventory.has(item_id, *count)
            }
            QuestRequirement::ReachLocation(location) => {
                state.has_flag(&format!("visited_{}", location))
            }
            QuestRequirement::SetFlag(flag) => {
                state.has_flag(flag)
            }
            QuestRequirement::HaveGold(amount) => {
                state.gold >= *amount
            }
            QuestRequirement::HavePartyMember(member) => {
                state.party.get_member(&member.to_lowercase()).is_some()
            }
            QuestRequirement::CompleteQuest(quest_id) => {
                state.has_flag(&format!("quest_complete_{}", quest_id))
            }
            QuestRequirement::FindShrine(shrine) => {
                state.has_shrine(shrine)
            }
        }
    }

    /// Get a display description for this requirement
    pub fn description(&self) -> String {
        match self {
            QuestRequirement::TalkTo(npc) => format!("Talk to {}", npc),
            QuestRequirement::DefeatEnemy(enemy, count) => {
                if *count == 1 {
                    format!("Defeat a {}", enemy)
                } else {
                    format!("Defeat {} {}s", count, enemy)
                }
            }
            QuestRequirement::CollectItem(item, count) => {
                if *count == 1 {
                    format!("Find {}", item)
                } else {
                    format!("Collect {} {}", count, item)
                }
            }
            QuestRequirement::ReachLocation(loc) => format!("Travel to {}", loc),
            QuestRequirement::SetFlag(_) => "Complete special objective".to_string(),
            QuestRequirement::HaveGold(amount) => format!("Have {} gold", amount),
            QuestRequirement::HavePartyMember(member) => format!("{} must be in party", member),
            QuestRequirement::CompleteQuest(quest) => format!("Complete: {}", quest),
            QuestRequirement::FindShrine(shrine) => format!("Find the {}", shrine),
        }
    }
}

// ============================================================================
// QUEST OBJECTIVE
// ============================================================================

/// A single objective within a quest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    /// Description shown to player
    pub description: String,
    /// What needs to be done
    pub requirement: QuestRequirement,
    /// Whether this objective is complete
    pub completed: bool,
    /// Optional - must be completed in order
    pub order: Option<u32>,
}

impl QuestObjective {
    /// Create a new objective
    pub fn new(description: &str, requirement: QuestRequirement) -> Self {
        Self {
            description: description.to_string(),
            requirement,
            completed: false,
            order: None,
        }
    }

    /// Create an ordered objective
    pub fn ordered(description: &str, requirement: QuestRequirement, order: u32) -> Self {
        Self {
            description: description.to_string(),
            requirement,
            completed: false,
            order: Some(order),
        }
    }

    /// Check and update completion status
    pub fn check_completion(&mut self, state: &GameState) -> bool {
        if !self.completed && self.requirement.is_satisfied(state) {
            self.completed = true;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// QUEST STATE
// ============================================================================

/// Current state of a quest
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QuestState {
    /// Quest hasn't been discovered yet
    #[default]
    NotStarted,
    /// Quest is currently active
    Active,
    /// Quest has been completed
    Completed,
    /// Quest has been failed
    Failed,
}

impl QuestState {
    /// Get display text for this state
    pub fn display(&self) -> &'static str {
        match self {
            QuestState::NotStarted => "Not Started",
            QuestState::Active => "Active",
            QuestState::Completed => "Completed",
            QuestState::Failed => "Failed",
        }
    }
}

// ============================================================================
// QUEST REWARD
// ============================================================================

/// Rewards given upon quest completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestReward {
    /// Gold reward
    Gold(u32),
    /// Item(s) reward
    Item(String, u32),
    /// Experience points
    Exp(u32),
    /// Unlock a new area
    UnlockArea(String),
    /// Set a story flag
    SetFlag(String),
    /// Reputation with a faction
    Reputation(String, i32),
}

impl QuestReward {
    /// Apply this reward to game state
    /// Returns description of what was received
    pub fn apply(&self, state: &mut GameState) -> String {
        match self {
            QuestReward::Gold(amount) => {
                state.gold += *amount;
                format!("Received {} gold", amount)
            }
            QuestReward::Item(item_id, count) => {
                state.inventory.add(item_id, *count);
                if *count == 1 {
                    format!("Received {}", item_id)
                } else {
                    format!("Received {} x{}", item_id, count)
                }
            }
            QuestReward::Exp(amount) => {
                // Give exp to party leader
                if let Some(leader) = state.party.members.first_mut() {
                    leader.exp += *amount;
                }
                format!("Gained {} EXP", amount)
            }
            QuestReward::UnlockArea(area) => {
                state.set_flag(&format!("area_unlocked_{}", area));
                format!("Unlocked: {}", area)
            }
            QuestReward::SetFlag(flag) => {
                state.set_flag(flag);
                String::new() // Silent
            }
            QuestReward::Reputation(faction, amount) => {
                state.set_flag(&format!("rep_{}_{}", faction, amount));
                if *amount > 0 {
                    format!("Reputation with {} increased", faction)
                } else {
                    format!("Reputation with {} decreased", faction)
                }
            }
        }
    }
}

// ============================================================================
// QUEST STRUCTURE
// ============================================================================

/// A complete quest definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Full description
    pub description: String,
    /// Quest objectives
    pub objectives: Vec<QuestObjective>,
    /// Rewards upon completion
    pub rewards: Vec<QuestReward>,
    /// Current state
    pub state: QuestState,
    /// Is this a main story quest?
    pub is_main_quest: bool,
    /// Quest giver NPC
    pub quest_giver: Option<String>,
    /// Recommended level
    pub recommended_level: Option<u8>,
}

impl Quest {
    /// Create a new quest
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            objectives: Vec::new(),
            rewards: Vec::new(),
            state: QuestState::NotStarted,
            is_main_quest: false,
            quest_giver: None,
            recommended_level: None,
        }
    }

    /// Create a main story quest
    pub fn main_quest(id: &str, name: &str, description: &str) -> Self {
        let mut quest = Self::new(id, name, description);
        quest.is_main_quest = true;
        quest
    }

    /// Add an objective
    pub fn add_objective(mut self, objective: QuestObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    /// Add a reward
    pub fn add_reward(mut self, reward: QuestReward) -> Self {
        self.rewards.push(reward);
        self
    }

    /// Set quest giver
    pub fn from_npc(mut self, npc_id: &str) -> Self {
        self.quest_giver = Some(npc_id.to_string());
        self
    }

    /// Set recommended level
    pub fn level(mut self, level: u8) -> Self {
        self.recommended_level = Some(level);
        self
    }

    /// Start this quest
    pub fn start(&mut self) {
        if self.state == QuestState::NotStarted {
            self.state = QuestState::Active;
        }
    }

    /// Check if all objectives are complete
    pub fn all_objectives_complete(&self) -> bool {
        self.objectives.iter().all(|o| o.completed)
    }

    /// Get count of completed objectives
    pub fn completed_objective_count(&self) -> usize {
        self.objectives.iter().filter(|o| o.completed).count()
    }

    /// Get next incomplete objective (respecting order)
    pub fn next_objective(&self) -> Option<&QuestObjective> {
        // If any objectives have order, respect it
        let has_ordered = self.objectives.iter().any(|o| o.order.is_some());

        if has_ordered {
            self.objectives.iter()
                .filter(|o| !o.completed)
                .min_by_key(|o| o.order.unwrap_or(u32::MAX))
        } else {
            self.objectives.iter().find(|o| !o.completed)
        }
    }

    /// Update objectives and check for completion
    /// Returns (newly_completed_objectives, quest_completed)
    pub fn update(&mut self, state: &GameState) -> (Vec<String>, bool) {
        if self.state != QuestState::Active {
            return (Vec::new(), false);
        }

        let mut completed = Vec::new();

        // Check ordered objectives first
        let has_ordered = self.objectives.iter().any(|o| o.order.is_some());

        if has_ordered {
            // Only check the next ordered objective
            if let Some(next_order) = self.objectives.iter()
                .filter(|o| !o.completed && o.order.is_some())
                .map(|o| o.order.unwrap())
                .min()
            {
                for obj in &mut self.objectives {
                    if obj.order == Some(next_order) && obj.check_completion(state) {
                        completed.push(obj.description.clone());
                    }
                }
            }

            // Also check unordered objectives
            for obj in &mut self.objectives {
                if obj.order.is_none() && obj.check_completion(state) {
                    completed.push(obj.description.clone());
                }
            }
        } else {
            // Check all objectives
            for obj in &mut self.objectives {
                if obj.check_completion(state) {
                    completed.push(obj.description.clone());
                }
            }
        }

        let quest_completed = self.all_objectives_complete();
        if quest_completed {
            self.state = QuestState::Completed;
        }

        (completed, quest_completed)
    }

    /// Complete the quest and give rewards
    pub fn complete(&mut self, state: &mut GameState) -> Vec<String> {
        if self.state != QuestState::Active && self.state != QuestState::Completed {
            return Vec::new();
        }

        self.state = QuestState::Completed;
        state.set_flag(&format!("quest_complete_{}", self.id));

        self.rewards.iter()
            .map(|r| r.apply(state))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Fail the quest
    pub fn fail(&mut self) {
        if self.state == QuestState::Active {
            self.state = QuestState::Failed;
        }
    }
}

// ============================================================================
// QUEST LOG
// ============================================================================

/// Collection of all quests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestLog {
    pub quests: HashMap<String, Quest>,
}

impl QuestLog {
    /// Create a new empty quest log
    pub fn new() -> Self {
        Self {
            quests: HashMap::new(),
        }
    }

    /// Add a quest to the log
    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// Start a quest by ID
    pub fn start_quest(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.state == QuestState::NotStarted {
                quest.start();
                return true;
            }
        }
        false
    }

    /// Get a quest by ID
    pub fn get_quest(&self, quest_id: &str) -> Option<&Quest> {
        self.quests.get(quest_id)
    }

    /// Get a mutable quest by ID
    pub fn get_quest_mut(&mut self, quest_id: &str) -> Option<&mut Quest> {
        self.quests.get_mut(quest_id)
    }

    /// Update a specific objective
    pub fn update_objective(&mut self, quest_id: &str, obj_index: usize) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if let Some(obj) = quest.objectives.get_mut(obj_index) {
                obj.completed = true;
                return true;
            }
        }
        false
    }

    /// Complete a quest by ID
    pub fn complete_quest(&mut self, quest_id: &str, state: &mut GameState) -> Vec<String> {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            return quest.complete(state);
        }
        Vec::new()
    }

    /// Get all active quests
    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.state == QuestState::Active)
            .collect()
    }

    /// Get all completed quests
    pub fn completed_quests(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.state == QuestState::Completed)
            .collect()
    }

    /// Get all main quests
    pub fn main_quests(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.is_main_quest)
            .collect()
    }

    /// Update all active quests and return any changes
    pub fn update_all(&mut self, state: &GameState) -> Vec<(String, Vec<String>, bool)> {
        let mut results = Vec::new();

        let quest_ids: Vec<String> = self.quests.values()
            .filter(|q| q.state == QuestState::Active)
            .map(|q| q.id.clone())
            .collect();

        for quest_id in quest_ids {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                let (completed_objs, quest_done) = quest.update(state);
                if !completed_objs.is_empty() || quest_done {
                    results.push((quest_id, completed_objs, quest_done));
                }
            }
        }

        results
    }

    /// Check if a quest is complete
    pub fn is_complete(&self, quest_id: &str) -> bool {
        self.quests.get(quest_id)
            .map(|q| q.state == QuestState::Completed)
            .unwrap_or(false)
    }

    /// Check if a quest is active
    pub fn is_active(&self, quest_id: &str) -> bool {
        self.quests.get(quest_id)
            .map(|q| q.state == QuestState::Active)
            .unwrap_or(false)
    }
}

// ============================================================================
// PREDEFINED QUESTS
// ============================================================================

/// Create the main story quests for Realm of Ralnar
pub fn create_main_story_quests() -> Vec<Quest> {
    vec![
        // Chapter 1: The Blessing
        Quest::main_quest(
            "chapter_1_blessing",
            "The Sage's Blessing",
            "Dorl, a wandering sage, has offered to bless your party. \
             His magic seems to enhance your abilities, but something feels... off."
        )
            .add_objective(QuestObjective::ordered(
                "Meet Dorl at the village shrine",
                QuestRequirement::TalkTo("dorl".to_string()),
                1
            ))
            .add_objective(QuestObjective::ordered(
                "Receive Dorl's blessing",
                QuestRequirement::SetFlag("blessing_received".to_string()),
                2
            ))
            .add_reward(QuestReward::SetFlag("chapter_1_complete".to_string()))
            .add_reward(QuestReward::Exp(100))
            .from_npc("dorl"),

        // Chapter 2: The First Shrine
        Quest::main_quest(
            "chapter_2_shrine",
            "Shrine of Shadows",
            "Dorl has revealed that evil shrines corrupt the land. \
             He asks you to destroy the first one in the Dark Forest."
        )
            .add_objective(QuestObjective::ordered(
                "Travel to the Dark Forest",
                QuestRequirement::ReachLocation("dark_forest".to_string()),
                1
            ))
            .add_objective(QuestObjective::ordered(
                "Find the Shrine of Shadows",
                QuestRequirement::FindShrine("shrine_of_shadows".to_string()),
                2
            ))
            .add_objective(QuestObjective::ordered(
                "Destroy the shrine",
                QuestRequirement::SetFlag("shrine_1_destroyed".to_string()),
                3
            ))
            .add_reward(QuestReward::SetFlag("chapter_2_complete".to_string()))
            .add_reward(QuestReward::Exp(250))
            .add_reward(QuestReward::Gold(500))
            .from_npc("dorl")
            .level(5),

        // The Reveal (late game)
        Quest::main_quest(
            "the_reveal",
            "The Terrible Truth",
            "Everything you believed was a lie. The shrines were protecting the realm, \
             and Dorl's 'blessing' was actually a curse binding you to his will."
        )
            .add_objective(QuestObjective::new(
                "Confront Dorl",
                QuestRequirement::TalkTo("dorl_reveal".to_string())
            ))
            .add_objective(QuestObjective::new(
                "Break the curse",
                QuestRequirement::SetFlag("curse_broken".to_string())
            ))
            .add_reward(QuestReward::SetFlag("the_reveal_complete".to_string()))
            .add_reward(QuestReward::Exp(1000))
            .level(25),
    ]
}

/// Create side quests
pub fn create_side_quests() -> Vec<Quest> {
    vec![
        Quest::new(
            "lost_pendant",
            "The Lost Pendant",
            "A villager has lost her grandmother's pendant in the forest."
        )
            .add_objective(QuestObjective::new(
                "Find the pendant in the forest",
                QuestRequirement::CollectItem("grandmas_pendant".to_string(), 1)
            ))
            .add_objective(QuestObjective::new(
                "Return to the villager",
                QuestRequirement::TalkTo("worried_villager".to_string())
            ))
            .add_reward(QuestReward::Gold(100))
            .add_reward(QuestReward::Item("potion".to_string(), 3))
            .from_npc("worried_villager")
            .level(1),

        Quest::new(
            "wolf_problem",
            "Wolf Problem",
            "Wolves are threatening the village livestock."
        )
            .add_objective(QuestObjective::new(
                "Defeat 5 wolves",
                QuestRequirement::DefeatEnemy("wolf".to_string(), 5)
            ))
            .add_reward(QuestReward::Gold(200))
            .add_reward(QuestReward::Exp(150))
            .from_npc("village_elder")
            .level(3),

        Quest::new(
            "herb_gathering",
            "Medicinal Herbs",
            "The healer needs herbs from the mountain."
        )
            .add_objective(QuestObjective::new(
                "Collect 10 mountain herbs",
                QuestRequirement::CollectItem("mountain_herb".to_string(), 10)
            ))
            .add_reward(QuestReward::Gold(150))
            .add_reward(QuestReward::Item("hi_potion".to_string(), 5))
            .from_npc("village_healer")
            .level(2),
    ]
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
    fn test_quest_creation() {
        let quest = Quest::new("test_quest", "Test Quest", "A test quest description");

        assert_eq!(quest.id, "test_quest");
        assert_eq!(quest.name, "Test Quest");
        assert_eq!(quest.state, QuestState::NotStarted);
        assert!(!quest.is_main_quest);
    }

    #[test]
    fn test_main_quest_creation() {
        let quest = Quest::main_quest("main", "Main Quest", "The main story");
        assert!(quest.is_main_quest);
    }

    #[test]
    fn test_quest_start() {
        let mut quest = Quest::new("test", "Test", "Test");
        assert_eq!(quest.state, QuestState::NotStarted);

        quest.start();
        assert_eq!(quest.state, QuestState::Active);

        // Starting again should do nothing
        quest.start();
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn test_quest_fail() {
        let mut quest = Quest::new("test", "Test", "Test");
        quest.start();

        quest.fail();
        assert_eq!(quest.state, QuestState::Failed);
    }

    #[test]
    fn test_objective_completion() {
        let mut state = create_test_state();
        let mut objective = QuestObjective::new(
            "Collect 3 gems",
            QuestRequirement::CollectItem("gem".to_string(), 3)
        );

        assert!(!objective.check_completion(&state));
        assert!(!objective.completed);

        state.inventory.add("gem", 2);
        assert!(!objective.check_completion(&state)); // Only 2

        state.inventory.add("gem", 1);
        assert!(objective.check_completion(&state)); // Now 3
        assert!(objective.completed);
    }

    #[test]
    fn test_quest_update() {
        let mut state = create_test_state();
        // GameState::new initializes gold to 100, so require more to test progression
        let mut quest = Quest::new("test", "Test", "Test")
            .add_objective(QuestObjective::new(
                "Get gold",
                QuestRequirement::HaveGold(200)
            ));
        quest.start();

        let (completed, done) = quest.update(&state);
        assert!(completed.is_empty());
        assert!(!done);

        state.gold = 200;
        let (completed, done) = quest.update(&state);
        assert_eq!(completed.len(), 1);
        assert!(done);
        assert_eq!(quest.state, QuestState::Completed);
    }

    #[test]
    fn test_ordered_objectives() {
        let mut state = create_test_state();
        let mut quest = Quest::new("ordered", "Ordered Quest", "Test")
            .add_objective(QuestObjective::ordered(
                "Step 1",
                QuestRequirement::SetFlag("step_1".to_string()),
                1
            ))
            .add_objective(QuestObjective::ordered(
                "Step 2",
                QuestRequirement::SetFlag("step_2".to_string()),
                2
            ));
        quest.start();

        // Set step 2 first - should not complete
        state.set_flag("step_2");
        let (completed, _) = quest.update(&state);
        assert!(completed.is_empty());

        // Now set step 1
        state.set_flag("step_1");
        let (completed, _) = quest.update(&state);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], "Step 1");

        // Now step 2 should complete
        let (completed, done) = quest.update(&state);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], "Step 2");
        assert!(done);
    }

    #[test]
    fn test_quest_rewards() {
        let mut state = create_test_state();
        state.gold = 0;

        let mut quest = Quest::new("reward_test", "Reward Test", "Test")
            .add_objective(QuestObjective::new(
                "Complete",
                QuestRequirement::SetFlag("done".to_string())
            ))
            .add_reward(QuestReward::Gold(500))
            .add_reward(QuestReward::Item("sword".to_string(), 1))
            .add_reward(QuestReward::Exp(100));

        quest.start();
        state.set_flag("done");
        quest.update(&state);

        let rewards = quest.complete(&mut state);

        assert_eq!(state.gold, 500);
        assert!(state.inventory.has("sword", 1));
        // Check leader got exp
        assert!(state.party.leader().map(|l| l.exp >= 100).unwrap_or(false));
        assert_eq!(rewards.len(), 3);
    }

    #[test]
    fn test_quest_log_operations() {
        let mut log = QuestLog::new();

        log.add_quest(Quest::new("quest1", "Quest 1", "First quest"));
        log.add_quest(Quest::new("quest2", "Quest 2", "Second quest"));

        assert!(log.get_quest("quest1").is_some());
        assert!(log.get_quest("nonexistent").is_none());

        assert!(log.start_quest("quest1"));
        assert!(log.is_active("quest1"));
        assert!(!log.is_active("quest2"));
    }

    #[test]
    fn test_quest_log_active_quests() {
        let mut log = QuestLog::new();

        log.add_quest(Quest::new("q1", "Quest 1", "Test"));
        log.add_quest(Quest::new("q2", "Quest 2", "Test"));
        log.add_quest(Quest::new("q3", "Quest 3", "Test"));

        log.start_quest("q1");
        log.start_quest("q2");

        let active = log.active_quests();
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_requirement_talk_to() {
        let mut state = create_test_state();
        let req = QuestRequirement::TalkTo("merchant".to_string());

        assert!(!req.is_satisfied(&state));

        state.set_flag("talked_to_merchant");
        assert!(req.is_satisfied(&state));
    }

    #[test]
    fn test_requirement_have_party_member() {
        let state = create_test_state();

        // Herbert is the starting character (lowercase ID)
        let req = QuestRequirement::HavePartyMember("herbert".to_string());
        assert!(req.is_satisfied(&state)); // Herbert starts in party
    }

    #[test]
    fn test_requirement_find_shrine() {
        let mut state = create_test_state();
        // Use shrine_of_fire since it maps to index 0
        let req = QuestRequirement::FindShrine("shrine_of_fire".to_string());

        assert!(!req.is_satisfied(&state));

        state.shrines_destroyed[0] = true; // Destroy shrine 0 (fire)
        assert!(req.is_satisfied(&state));
    }

    #[test]
    fn test_requirement_description() {
        let req = QuestRequirement::DefeatEnemy("goblin".to_string(), 5);
        assert_eq!(req.description(), "Defeat 5 goblins");

        let req = QuestRequirement::DefeatEnemy("dragon".to_string(), 1);
        assert_eq!(req.description(), "Defeat a dragon");
    }

    #[test]
    fn test_reward_unlock_area() {
        let mut state = create_test_state();
        let reward = QuestReward::UnlockArea("castle".to_string());

        let msg = reward.apply(&mut state);
        assert!(msg.contains("castle"));
        assert!(state.has_flag("area_unlocked_castle"));
    }

    #[test]
    fn test_quest_log_update_all() {
        let mut state = create_test_state();
        let mut log = QuestLog::new();

        log.add_quest(
            Quest::new("test", "Test", "Test")
                .add_objective(QuestObjective::new(
                    "Get gold",
                    QuestRequirement::HaveGold(100)
                ))
        );
        log.start_quest("test");

        state.gold = 50;
        let results = log.update_all(&state);
        assert!(results.is_empty());

        state.gold = 100;
        let results = log.update_all(&state);
        assert_eq!(results.len(), 1);
        assert!(results[0].2); // Quest completed
    }

    #[test]
    fn test_complete_quest_flag() {
        let mut state = create_test_state();
        let mut log = QuestLog::new();

        log.add_quest(
            Quest::new("my_quest", "My Quest", "Test")
                .add_objective(QuestObjective::new(
                    "Done",
                    QuestRequirement::SetFlag("done".to_string())
                ))
        );
        log.start_quest("my_quest");

        state.set_flag("done");
        if let Some(quest) = log.get_quest_mut("my_quest") {
            quest.update(&state);
            quest.complete(&mut state);
        }

        assert!(state.has_flag("quest_complete_my_quest"));
    }

    #[test]
    fn test_predefined_main_quests() {
        let quests = create_main_story_quests();
        assert!(!quests.is_empty());

        for quest in &quests {
            assert!(quest.is_main_quest);
            assert!(!quest.objectives.is_empty());
        }
    }

    #[test]
    fn test_predefined_side_quests() {
        let quests = create_side_quests();
        assert!(!quests.is_empty());

        for quest in &quests {
            assert!(!quest.is_main_quest);
            assert!(!quest.objectives.is_empty());
            assert!(!quest.rewards.is_empty());
        }
    }

    #[test]
    fn test_quest_next_objective() {
        let quest = Quest::new("test", "Test", "Test")
            .add_objective(QuestObjective::ordered("First", QuestRequirement::SetFlag("a".to_string()), 1))
            .add_objective(QuestObjective::ordered("Second", QuestRequirement::SetFlag("b".to_string()), 2));

        let next = quest.next_objective();
        assert!(next.is_some());
        assert_eq!(next.unwrap().description, "First");
    }

    #[test]
    fn test_quest_completed_count() {
        let mut quest = Quest::new("test", "Test", "Test")
            .add_objective(QuestObjective::new("Obj 1", QuestRequirement::SetFlag("a".to_string())))
            .add_objective(QuestObjective::new("Obj 2", QuestRequirement::SetFlag("b".to_string())))
            .add_objective(QuestObjective::new("Obj 3", QuestRequirement::SetFlag("c".to_string())));

        assert_eq!(quest.completed_objective_count(), 0);

        quest.objectives[0].completed = true;
        assert_eq!(quest.completed_objective_count(), 1);

        quest.objectives[2].completed = true;
        assert_eq!(quest.completed_objective_count(), 2);
    }

    #[test]
    fn test_quest_state_display() {
        assert_eq!(QuestState::NotStarted.display(), "Not Started");
        assert_eq!(QuestState::Active.display(), "Active");
        assert_eq!(QuestState::Completed.display(), "Completed");
        assert_eq!(QuestState::Failed.display(), "Failed");
    }
}
