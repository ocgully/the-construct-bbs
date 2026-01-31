//! IGM (In-Game Module) support for Dragon Slayer
//! Allows extensible content through plug-in modules

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// IGM Module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgmModule {
    /// Unique module identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description shown in Other Places menu
    pub description: String,
    /// Hotkey for the module (A-Z)
    pub hotkey: char,
    /// Whether module is enabled
    pub enabled: bool,
    /// Module author
    pub author: String,
    /// Version string
    pub version: String,
    /// Module type
    pub module_type: IgmType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IgmType {
    /// Location - adds a new place to visit
    Location,
    /// Event - adds random forest events
    Event,
    /// Shop - adds a new shop
    Shop,
    /// Combat - adds new monsters
    Combat,
    /// Quest - adds quest content
    Quest,
}

/// Hook points where IGMs can inject content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum IgmHook {
    /// After entering the forest
    ForestEntry,
    /// Before a monster encounter
    PreCombat,
    /// After winning combat
    PostCombat,
    /// When visiting Other Places
    OtherPlaces,
    /// At the inn
    Inn,
    /// Daily reset
    DailyReset,
    /// When player levels up
    LevelUp,
    /// Before dragon encounter
    PreDragon,
}

/// IGM action result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IgmResult {
    /// Display text to player
    Display { text: String },
    /// Modify player gold
    ModifyGold { amount: i64 },
    /// Modify player XP
    ModifyXp { amount: i64 },
    /// Heal player
    Heal { amount: u32 },
    /// Damage player
    Damage { amount: u32 },
    /// Give item
    GiveItem { item_key: String },
    /// Set flag
    SetFlag { key: String, value: String },
    /// Start combat with custom monster
    StartCombat { monster_key: String },
    /// Teleport to location
    Teleport { location: String },
    /// No effect
    None,
}

/// Registry of loaded IGM modules
#[derive(Debug, Default)]
pub struct IgmRegistry {
    modules: HashMap<String, IgmModule>,
    /// Module state storage (persisted per-player)
    #[allow(dead_code)]
    module_states: HashMap<String, HashMap<String, String>>,
}

impl IgmRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new IGM module
    pub fn register(&mut self, module: IgmModule) -> Result<(), String> {
        if self.modules.contains_key(&module.id) {
            return Err(format!("Module '{}' already registered", module.id));
        }
        self.modules.insert(module.id.clone(), module);
        Ok(())
    }

    /// Unregister a module
    #[allow(dead_code)]
    pub fn unregister(&mut self, module_id: &str) -> Option<IgmModule> {
        self.modules.remove(module_id)
    }

    /// Get all enabled modules
    #[allow(dead_code)]
    pub fn get_enabled(&self) -> Vec<&IgmModule> {
        self.modules.values()
            .filter(|m| m.enabled)
            .collect()
    }

    /// Get modules for a specific hook point
    #[allow(dead_code)]
    pub fn get_for_hook(&self, hook: IgmHook) -> Vec<&IgmModule> {
        self.modules.values()
            .filter(|m| m.enabled && matches_hook(m, hook))
            .collect()
    }

    /// Get location modules for Other Places menu
    pub fn get_locations(&self) -> Vec<&IgmModule> {
        self.modules.values()
            .filter(|m| m.enabled && m.module_type == IgmType::Location)
            .collect()
    }

    /// Get a module by ID
    #[allow(dead_code)]
    pub fn get(&self, module_id: &str) -> Option<&IgmModule> {
        self.modules.get(module_id)
    }

    /// Set module state value (for player persistence)
    #[allow(dead_code)]
    pub fn set_state(&mut self, module_id: &str, key: &str, value: &str) {
        let module_state = self.module_states
            .entry(module_id.to_string())
            .or_insert_with(HashMap::new);
        module_state.insert(key.to_string(), value.to_string());
    }

    /// Get module state value
    #[allow(dead_code)]
    pub fn get_state(&self, module_id: &str, key: &str) -> Option<&String> {
        self.module_states
            .get(module_id)
            .and_then(|m| m.get(key))
    }

    /// Export module states for serialization
    #[allow(dead_code)]
    pub fn export_states(&self) -> HashMap<String, HashMap<String, String>> {
        self.module_states.clone()
    }

    /// Import module states
    #[allow(dead_code)]
    pub fn import_states(&mut self, states: HashMap<String, HashMap<String, String>>) {
        self.module_states = states;
    }
}

#[allow(dead_code)]
fn matches_hook(module: &IgmModule, hook: IgmHook) -> bool {
    match (hook, &module.module_type) {
        (IgmHook::OtherPlaces, IgmType::Location) => true,
        (IgmHook::ForestEntry, IgmType::Event) => true,
        (IgmHook::PreCombat | IgmHook::PostCombat, IgmType::Combat) => true,
        (IgmHook::OtherPlaces, IgmType::Shop) => true,
        (IgmHook::DailyReset | IgmHook::LevelUp, IgmType::Quest) => true,
        _ => false,
    }
}

/// Create sample/default IGM modules
pub fn create_default_modules() -> Vec<IgmModule> {
    vec![
        IgmModule {
            id: "fairy_grove".to_string(),
            name: "The Fairy Grove".to_string(),
            description: "A mystical grove where fairies gather.".to_string(),
            hotkey: 'G',
            enabled: true,
            author: "System".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Location,
        },
        IgmModule {
            id: "dark_cave".to_string(),
            name: "The Dark Cave".to_string(),
            description: "A dangerous cave with hidden treasures.".to_string(),
            hotkey: 'D',
            enabled: true,
            author: "System".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Location,
        },
        IgmModule {
            id: "gambling_den".to_string(),
            name: "The Gambling Den".to_string(),
            description: "Test your luck with games of chance.".to_string(),
            hotkey: 'L',
            enabled: true,
            author: "System".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Location,
        },
    ]
}

/// Interface for IGM event handlers
#[allow(dead_code)]
pub trait IgmHandler: Send + Sync {
    /// Called when player enters the module location
    fn on_enter(&self, player_level: u8, player_gold: i64) -> Vec<IgmResult>;

    /// Handle player input in the module
    fn handle_input(&self, input: &str, player_level: u8) -> Vec<IgmResult>;

    /// Get menu options for the module
    fn get_menu(&self) -> Vec<(char, String)>;

    /// Called when player exits
    fn on_exit(&self) -> Vec<IgmResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut registry = IgmRegistry::new();

        let module = IgmModule {
            id: "test".to_string(),
            name: "Test Module".to_string(),
            description: "A test".to_string(),
            hotkey: 'T',
            enabled: true,
            author: "Test".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Location,
        };

        registry.register(module).unwrap();
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_get_locations() {
        let mut registry = IgmRegistry::new();

        for module in create_default_modules() {
            registry.register(module).unwrap();
        }

        let locations = registry.get_locations();
        assert!(!locations.is_empty());
    }

    #[test]
    fn test_state_persistence() {
        let mut registry = IgmRegistry::new();

        registry.set_state("test_mod", "visited", "true");
        assert_eq!(registry.get_state("test_mod", "visited"), Some(&"true".to_string()));

        let exported = registry.export_states();
        assert!(!exported.is_empty());
    }

    #[test]
    fn test_hook_matching() {
        let module = IgmModule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            hotkey: 'T',
            enabled: true,
            author: "Test".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Location,
        };

        assert!(matches_hook(&module, IgmHook::OtherPlaces));
        assert!(!matches_hook(&module, IgmHook::PreCombat));
    }
}
