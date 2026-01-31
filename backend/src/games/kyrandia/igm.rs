//! IGM (In-Game Module) support for Morningmist
//! Allows extensible content through plug-in modules

#![allow(dead_code)]

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// IGM Module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgmModule {
    /// Unique module identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
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
    /// Event - adds random events
    Event,
    /// Shop - adds a new shop
    Shop,
    /// Quest - adds quest content
    Quest,
    /// Spell - adds new spells
    Spell,
    /// Monster - adds new monsters
    Monster,
    /// Puzzle - adds new puzzles
    Puzzle,
}

/// Hook points where IGMs can inject content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgmHook {
    /// After entering a room
    RoomEntry,
    /// Before combat
    PreCombat,
    /// After combat
    PostCombat,
    /// When talking to NPCs
    NpcDialogue,
    /// At the fountain
    Fountain,
    /// Daily reset
    DailyReset,
    /// Level up
    LevelUp,
    /// Spell cast
    SpellCast,
    /// Item use
    ItemUse,
}

/// IGM action result
#[derive(Debug, Clone)]
pub enum IgmResult {
    /// Display text to player
    Display { text: String },
    /// Modify gold
    ModifyGold { amount: i64 },
    /// Modify XP
    ModifyXp { amount: i64 },
    /// Heal player
    Heal { amount: u32 },
    /// Damage player
    Damage { amount: u32 },
    /// Give item
    GiveItem { item_key: String },
    /// Remove item
    RemoveItem { item_key: String },
    /// Learn spell
    LearnSpell { spell_key: String },
    /// Set flag
    SetFlag { key: String, value: String },
    /// Start combat with custom monster
    StartCombat { monster_key: String },
    /// Teleport to room
    Teleport { room_key: String },
    /// Add new room temporarily
    AddRoom { room_key: String, room_data: String },
    /// No effect
    None,
}

/// Registry of loaded IGM modules
#[derive(Debug, Default)]
pub struct IgmRegistry {
    modules: HashMap<String, IgmModule>,
    /// Module state storage (persisted per-player)
    module_states: HashMap<String, HashMap<String, String>>,
    /// Custom rooms added by IGMs
    custom_rooms: HashMap<String, CustomRoom>,
    /// Custom items added by IGMs
    custom_items: HashMap<String, CustomItem>,
    /// Custom spells added by IGMs
    custom_spells: HashMap<String, CustomSpell>,
}

/// Custom room added by IGM
#[derive(Debug, Clone)]
pub struct CustomRoom {
    pub key: String,
    pub name: String,
    pub description: String,
    pub exits: Vec<(String, String)>,
    pub module_id: String,
}

/// Custom item added by IGM
#[derive(Debug, Clone)]
pub struct CustomItem {
    pub key: String,
    pub name: String,
    pub description: String,
    pub module_id: String,
}

/// Custom spell added by IGM
#[derive(Debug, Clone)]
pub struct CustomSpell {
    pub key: String,
    pub name: String,
    pub incantation: String,
    pub description: String,
    pub mana_cost: u32,
    pub module_id: String,
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
    pub fn unregister(&mut self, module_id: &str) -> Option<IgmModule> {
        // Also remove custom content from this module
        self.custom_rooms.retain(|_, r| r.module_id != module_id);
        self.custom_items.retain(|_, i| i.module_id != module_id);
        self.custom_spells.retain(|_, s| s.module_id != module_id);

        self.modules.remove(module_id)
    }

    /// Get all enabled modules
    pub fn get_enabled(&self) -> Vec<&IgmModule> {
        self.modules.values()
            .filter(|m| m.enabled)
            .collect()
    }

    /// Get modules for a specific hook point
    pub fn get_for_hook(&self, hook: IgmHook) -> Vec<&IgmModule> {
        self.modules.values()
            .filter(|m| m.enabled && matches_hook(m, hook))
            .collect()
    }

    /// Get location modules
    pub fn get_locations(&self) -> Vec<&IgmModule> {
        self.modules.values()
            .filter(|m| m.enabled && m.module_type == IgmType::Location)
            .collect()
    }

    /// Get a module by ID
    pub fn get(&self, module_id: &str) -> Option<&IgmModule> {
        self.modules.get(module_id)
    }

    /// Set module state value
    pub fn set_state(&mut self, module_id: &str, key: &str, value: &str) {
        let module_state = self.module_states
            .entry(module_id.to_string())
            .or_insert_with(HashMap::new);
        module_state.insert(key.to_string(), value.to_string());
    }

    /// Get module state value
    pub fn get_state(&self, module_id: &str, key: &str) -> Option<&String> {
        self.module_states
            .get(module_id)
            .and_then(|m| m.get(key))
    }

    /// Export module states for serialization
    pub fn export_states(&self) -> HashMap<String, HashMap<String, String>> {
        self.module_states.clone()
    }

    /// Import module states
    pub fn import_states(&mut self, states: HashMap<String, HashMap<String, String>>) {
        self.module_states = states;
    }

    /// Register a custom room
    pub fn add_room(&mut self, room: CustomRoom) {
        self.custom_rooms.insert(room.key.clone(), room);
    }

    /// Get a custom room
    pub fn get_custom_room(&self, key: &str) -> Option<&CustomRoom> {
        self.custom_rooms.get(key)
    }

    /// Register a custom item
    pub fn add_item(&mut self, item: CustomItem) {
        self.custom_items.insert(item.key.clone(), item);
    }

    /// Get a custom item
    pub fn get_custom_item(&self, key: &str) -> Option<&CustomItem> {
        self.custom_items.get(key)
    }

    /// Register a custom spell
    pub fn add_spell(&mut self, spell: CustomSpell) {
        self.custom_spells.insert(spell.key.clone(), spell);
    }

    /// Get a custom spell
    pub fn get_custom_spell(&self, key: &str) -> Option<&CustomSpell> {
        self.custom_spells.get(key)
    }
}

fn matches_hook(module: &IgmModule, hook: IgmHook) -> bool {
    match (hook, &module.module_type) {
        (IgmHook::RoomEntry, IgmType::Location) => true,
        (IgmHook::RoomEntry, IgmType::Event) => true,
        (IgmHook::PreCombat | IgmHook::PostCombat, IgmType::Monster) => true,
        (IgmHook::NpcDialogue, IgmType::Quest) => true,
        (IgmHook::Fountain, IgmType::Spell) => true,
        (IgmHook::SpellCast, IgmType::Spell) => true,
        (IgmHook::LevelUp, IgmType::Quest) => true,
        (IgmHook::DailyReset, IgmType::Event) => true,
        _ => false,
    }
}

/// Create sample/default IGM modules
pub fn create_default_modules() -> Vec<IgmModule> {
    vec![
        IgmModule {
            id: "moonlit_glade".to_string(),
            name: "The Moonlit Glade".to_string(),
            description: "A secret glade where faeries dance under the moonlight.".to_string(),
            hotkey: 'M',
            enabled: true,
            author: "System".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Location,
        },
        IgmModule {
            id: "wandering_merchant".to_string(),
            name: "The Wandering Merchant".to_string(),
            description: "A mysterious merchant who appears randomly with rare goods.".to_string(),
            hotkey: 'W',
            enabled: true,
            author: "System".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Event,
        },
        IgmModule {
            id: "ancient_tome_quest".to_string(),
            name: "Quest: The Ancient Tome".to_string(),
            description: "A quest to find the lost pages of an ancient spellbook.".to_string(),
            hotkey: 'A',
            enabled: true,
            author: "System".to_string(),
            version: "1.0".to_string(),
            module_type: IgmType::Quest,
        },
    ]
}

/// Interface for IGM event handlers
pub trait IgmHandler: Send + Sync {
    /// Get module info
    fn module_info(&self) -> &IgmModule;

    /// Called when entering a location
    fn on_enter(&self, player_level: u8, player_gold: i64) -> Vec<IgmResult>;

    /// Handle player input in the module
    fn handle_input(&self, input: &str, player_level: u8) -> Vec<IgmResult>;

    /// Get menu options for the module
    fn get_menu(&self) -> Vec<(char, String)>;

    /// Called when exiting
    fn on_exit(&self) -> Vec<IgmResult>;

    /// Called on specific hooks
    fn on_hook(&self, hook: IgmHook, context: &HookContext) -> Vec<IgmResult>;
}

/// Context passed to hook handlers
#[derive(Debug, Clone)]
pub struct HookContext {
    pub player_level: u8,
    pub player_gold: i64,
    pub current_room: String,
    pub player_name: String,
    pub flags: HashMap<String, String>,
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

        assert!(matches_hook(&module, IgmHook::RoomEntry));
        assert!(!matches_hook(&module, IgmHook::PreCombat));
    }

    #[test]
    fn test_custom_content() {
        let mut registry = IgmRegistry::new();

        registry.add_room(CustomRoom {
            key: "custom_room".to_string(),
            name: "Custom Room".to_string(),
            description: "A custom room".to_string(),
            exits: vec![("north".to_string(), "village_square".to_string())],
            module_id: "test".to_string(),
        });

        assert!(registry.get_custom_room("custom_room").is_some());

        // Unregister module should clean up custom content
        registry.unregister("test");
        assert!(registry.get_custom_room("custom_room").is_none());
    }
}
