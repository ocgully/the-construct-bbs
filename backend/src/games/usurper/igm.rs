//! IGM (In-Game Module) Support for Usurper
//!
//! Provides hooks and interfaces for extending the game with custom content.
//! IGMs can add new locations, monsters, items, quests, and events.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// IGM hook points where modules can inject content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IgmHookPoint {
    /// Called when entering town
    TownEnter,
    /// Called when leaving town
    TownExit,
    /// Called before entering dungeon
    DungeonEnter,
    /// Called after dungeon exploration
    DungeonExplore,
    /// Called after combat victory
    CombatVictory,
    /// Called after combat defeat
    CombatDefeat,
    /// Called after leveling up
    LevelUp,
    /// Called at daily reset
    DailyReset,
    /// Called when visiting healer
    HealerVisit,
    /// Called when visiting shop
    ShopVisit,
    /// Called when using substance
    SubstanceUse,
    /// Called on romance action
    RomanceAction,
}

/// Interface for IGM modules
pub trait IgmModule: Send + Sync {
    /// Module identifier (unique)
    fn id(&self) -> &str;

    /// Display name
    fn name(&self) -> &str;

    /// Description
    fn description(&self) -> &str;

    /// Version string
    fn version(&self) -> &str;

    /// Hook points this module wants to intercept
    fn hooks(&self) -> Vec<IgmHookPoint>;

    /// Called when a hook point is triggered
    fn on_hook(&self, hook: IgmHookPoint, context: &mut IgmContext) -> IgmResult;

    /// Get custom menu items for town
    fn town_menu_items(&self) -> Vec<IgmMenuItem> {
        Vec::new()
    }

    /// Handle selection of a custom menu item
    fn handle_menu_item(&self, item_id: &str, context: &mut IgmContext) -> IgmResult {
        let _ = (item_id, context);
        IgmResult::Continue
    }
}

/// Context passed to IGM hooks
#[derive(Debug)]
pub struct IgmContext {
    /// Player's current level
    pub player_level: u32,
    /// Player's current HP
    pub player_hp: u32,
    /// Player's max HP
    pub player_max_hp: u32,
    /// Player's gold
    pub player_gold: u64,
    /// Player's current dungeon level
    pub dungeon_level: u32,
    /// Player's mental stability
    pub mental_stability: i32,
    /// Custom data storage (per-player, persisted)
    pub custom_data: HashMap<String, String>,
    /// Messages to display to player
    pub messages: Vec<String>,
    /// Items to give player
    pub give_items: Vec<String>,
    /// Gold to give player
    pub give_gold: i64,
    /// XP to give player
    pub give_xp: u64,
    /// Damage to deal to player
    pub deal_damage: u32,
    /// Mental stability change
    pub mental_change: i32,
}

impl IgmContext {
    pub fn new(
        player_level: u32,
        player_hp: u32,
        player_max_hp: u32,
        player_gold: u64,
        dungeon_level: u32,
        mental_stability: i32,
    ) -> Self {
        Self {
            player_level,
            player_hp,
            player_max_hp,
            player_gold,
            dungeon_level,
            mental_stability,
            custom_data: HashMap::new(),
            messages: Vec::new(),
            give_items: Vec::new(),
            give_gold: 0,
            give_xp: 0,
            deal_damage: 0,
            mental_change: 0,
        }
    }

    /// Add a message to display
    pub fn add_message(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
    }

    /// Give item to player
    pub fn give_item(&mut self, item_key: &str) {
        self.give_items.push(item_key.to_string());
    }

    /// Store custom data
    pub fn set_data(&mut self, key: &str, value: &str) {
        self.custom_data.insert(key.to_string(), value.to_string());
    }

    /// Get custom data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.custom_data.get(key)
    }
}

/// Result of IGM hook execution
#[derive(Debug, Clone)]
pub enum IgmResult {
    /// Continue normal game flow
    Continue,
    /// Block normal action and display message
    Block(String),
    /// Add custom screen
    CustomScreen { title: String, content: String, menu: Vec<IgmMenuItem> },
    /// Trigger combat with custom monster
    Combat { monster_key: String },
    /// Redirect to location
    Redirect(IgmHookPoint),
}

/// Custom menu item definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgmMenuItem {
    /// Unique identifier
    pub id: String,
    /// Display key (single char)
    pub key: char,
    /// Display text
    pub label: String,
    /// Required player level (0 = no requirement)
    pub min_level: u32,
    /// Gold cost (0 = free)
    pub cost: u64,
}

/// IGM registry - manages all loaded modules
pub struct IgmRegistry {
    modules: Vec<Box<dyn IgmModule>>,
    hook_cache: HashMap<IgmHookPoint, Vec<usize>>,
}

impl IgmRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            hook_cache: HashMap::new(),
        }
    }

    /// Register a new IGM module
    pub fn register(&mut self, module: Box<dyn IgmModule>) {
        let idx = self.modules.len();

        // Cache hook registrations
        for hook in module.hooks() {
            self.hook_cache.entry(hook).or_default().push(idx);
        }

        self.modules.push(module);
    }

    /// Execute all modules for a hook point
    pub fn execute_hook(&self, hook: IgmHookPoint, context: &mut IgmContext) -> Vec<IgmResult> {
        let mut results = Vec::new();

        if let Some(module_indices) = self.hook_cache.get(&hook) {
            for &idx in module_indices {
                let result = self.modules[idx].on_hook(hook, context);
                results.push(result);
            }
        }

        results
    }

    /// Get all town menu items from all modules
    pub fn get_town_menu_items(&self) -> Vec<(String, IgmMenuItem)> {
        let mut items = Vec::new();

        for module in &self.modules {
            for item in module.town_menu_items() {
                items.push((module.id().to_string(), item));
            }
        }

        items
    }

    /// Handle custom menu item selection
    pub fn handle_menu_item(&self, module_id: &str, item_id: &str, context: &mut IgmContext) -> Option<IgmResult> {
        for module in &self.modules {
            if module.id() == module_id {
                return Some(module.handle_menu_item(item_id, context));
            }
        }
        None
    }

    /// Get all registered modules
    pub fn list_modules(&self) -> Vec<(&str, &str, &str)> {
        self.modules.iter()
            .map(|m| (m.id(), m.name(), m.version()))
            .collect()
    }
}

impl Default for IgmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// EXAMPLE IGM IMPLEMENTATION
// ============================================================================

/// Example IGM: The Mysterious Stranger
/// A simple module that adds random encounters with a helpful stranger
pub struct MysteriousStrangerIgm;

impl IgmModule for MysteriousStrangerIgm {
    fn id(&self) -> &str {
        "mysterious_stranger"
    }

    fn name(&self) -> &str {
        "The Mysterious Stranger"
    }

    fn description(&self) -> &str {
        "Adds random encounters with a mysterious stranger who offers gifts and quests."
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn hooks(&self) -> Vec<IgmHookPoint> {
        vec![IgmHookPoint::DungeonExplore, IgmHookPoint::TownEnter]
    }

    fn on_hook(&self, hook: IgmHookPoint, context: &mut IgmContext) -> IgmResult {
        match hook {
            IgmHookPoint::DungeonExplore => {
                // 5% chance to encounter the stranger in dungeons
                if rand::random::<u32>() % 100 < 5 {
                    context.add_message("A mysterious figure emerges from the shadows...");
                    context.add_message("\"Take this gift, brave adventurer.\"");
                    context.give_gold += (context.dungeon_level as i64) * 10;
                    context.give_xp += (context.dungeon_level as u64) * 5;
                }
                IgmResult::Continue
            }
            IgmHookPoint::TownEnter => {
                // Track visits
                let visits: u32 = context.get_data("stranger_visits")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                if visits % 10 == 9 {
                    context.add_message("The Mysterious Stranger nods to you from the shadows.");
                    context.mental_change += 5;
                }

                context.set_data("stranger_visits", &(visits + 1).to_string());
                IgmResult::Continue
            }
            _ => IgmResult::Continue,
        }
    }

    fn town_menu_items(&self) -> Vec<IgmMenuItem> {
        vec![
            IgmMenuItem {
                id: "stranger_shrine".to_string(),
                key: 'Z',
                label: "Visit the Stranger's Shrine".to_string(),
                min_level: 10,
                cost: 0,
            }
        ]
    }

    fn handle_menu_item(&self, item_id: &str, context: &mut IgmContext) -> IgmResult {
        if item_id == "stranger_shrine" {
            context.add_message("You kneel at the shadowy shrine...");
            context.mental_change += 10;
            context.give_xp += 50;
            IgmResult::Block("The Mysterious Stranger blesses you with renewed purpose.".to_string())
        } else {
            IgmResult::Continue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_igm_registry() {
        let mut registry = IgmRegistry::new();
        registry.register(Box::new(MysteriousStrangerIgm));

        let modules = registry.list_modules();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].0, "mysterious_stranger");
    }

    #[test]
    fn test_hook_execution() {
        let mut registry = IgmRegistry::new();
        registry.register(Box::new(MysteriousStrangerIgm));

        let mut context = IgmContext::new(10, 100, 100, 500, 5, 100);
        let results = registry.execute_hook(IgmHookPoint::TownEnter, &mut context);

        assert!(!results.is_empty());
        // Should have tracked the visit
        assert!(context.get_data("stranger_visits").is_some());
    }

    #[test]
    fn test_menu_items() {
        let mut registry = IgmRegistry::new();
        registry.register(Box::new(MysteriousStrangerIgm));

        let items = registry.get_town_menu_items();
        assert!(!items.is_empty());
        assert_eq!(items[0].1.key, 'Z');
    }

    #[test]
    fn test_context() {
        let mut context = IgmContext::new(5, 50, 100, 1000, 3, 80);

        context.add_message("Test message");
        context.give_item("rusty_sword");
        context.set_data("test_key", "test_value");

        assert_eq!(context.messages.len(), 1);
        assert_eq!(context.give_items.len(), 1);
        assert_eq!(context.get_data("test_key"), Some(&"test_value".to_string()));
    }
}
