//! Player state for Mineteria
//!
//! Tracks player position, health, inventory, and progression.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::data::{ItemType, ToolType, ToolMaterial, Tool, BlockType};

/// Player position in the world
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// A slot in the inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: ItemType,
    pub count: u8,
    /// For tools, track durability
    pub tool_data: Option<Tool>,
}

impl InventorySlot {
    pub fn new(item: ItemType, count: u8) -> Self {
        let tool_data = match item {
            ItemType::Tool(tool_type, material) => Some(Tool::new(tool_type, material)),
            _ => None,
        };
        Self { item, count, tool_data }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn can_stack(&self, other: &ItemType) -> bool {
        if self.is_empty() {
            return true;
        }
        // Tools don't stack
        if self.tool_data.is_some() {
            return false;
        }
        self.item == *other && self.count < self.item.get_item().max_stack
    }
}

/// Player inventory with hotbar and main slots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInventory {
    /// Hotbar slots (1-9)
    pub hotbar: [Option<InventorySlot>; 9],
    /// Main inventory (3 rows of 9)
    pub main: [Option<InventorySlot>; 27],
    /// Currently selected hotbar slot (0-8)
    pub selected_slot: u8,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            hotbar: Default::default(),
            main: Default::default(),
            selected_slot: 0,
        }
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> Option<&InventorySlot> {
        self.hotbar[self.selected_slot as usize].as_ref()
    }

    /// Get mutable reference to selected item
    pub fn selected_item_mut(&mut self) -> Option<&mut InventorySlot> {
        self.hotbar[self.selected_slot as usize].as_mut()
    }

    /// Add item to inventory, returns how many couldn't fit
    pub fn add_item(&mut self, item: ItemType, mut count: u8) -> u8 {
        let max_stack = item.get_item().max_stack;

        // First, try to stack with existing items
        for slot in self.hotbar.iter_mut().chain(self.main.iter_mut()) {
            if count == 0 {
                break;
            }
            if let Some(ref mut s) = slot {
                if s.can_stack(&item) {
                    let space = max_stack - s.count;
                    let add = count.min(space);
                    s.count += add;
                    count -= add;
                }
            }
        }

        // Then, try empty slots
        for slot in self.hotbar.iter_mut().chain(self.main.iter_mut()) {
            if count == 0 {
                break;
            }
            if slot.is_none() {
                let add = count.min(max_stack);
                *slot = Some(InventorySlot::new(item, add));
                count -= add;
            }
        }

        count // Return remaining that couldn't fit
    }

    /// Remove item from inventory, returns how many were removed
    pub fn remove_item(&mut self, item: &ItemType, mut count: u8) -> u8 {
        let original_count = count;

        for slot in self.hotbar.iter_mut().chain(self.main.iter_mut()) {
            if count == 0 {
                break;
            }
            if let Some(ref mut s) = slot {
                if s.item == *item {
                    let remove = count.min(s.count);
                    s.count -= remove;
                    count -= remove;
                    if s.count == 0 {
                        *slot = None;
                    }
                }
            }
        }

        original_count - count
    }

    /// Count total of an item type in inventory
    pub fn count_item(&self, item: &ItemType) -> u32 {
        let mut total = 0u32;
        for slot in self.hotbar.iter().chain(self.main.iter()) {
            if let Some(s) = slot {
                if s.item == *item {
                    total += s.count as u32;
                }
            }
        }
        total
    }

    /// Check if inventory has at least count of an item
    pub fn has_item(&self, item: &ItemType, count: u8) -> bool {
        self.count_item(item) >= count as u32
    }

    /// Get the best tool of a given type
    pub fn get_best_tool(&self, tool_type: ToolType) -> Option<(ToolMaterial, usize, bool)> {
        let mut best: Option<(ToolMaterial, usize, bool)> = None;

        // Check hotbar first
        for (i, slot) in self.hotbar.iter().enumerate() {
            if let Some(s) = slot {
                if let Some(ref tool) = s.tool_data {
                    if tool.tool_type == tool_type && !tool.is_broken() {
                        match best {
                            None => best = Some((tool.material, i, true)),
                            Some((mat, _, _)) if tool.material > mat => {
                                best = Some((tool.material, i, true))
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Check main inventory
        for (i, slot) in self.main.iter().enumerate() {
            if let Some(s) = slot {
                if let Some(ref tool) = s.tool_data {
                    if tool.tool_type == tool_type && !tool.is_broken() {
                        match best {
                            None => best = Some((tool.material, i, false)),
                            Some((mat, _, _)) if tool.material > mat => {
                                best = Some((tool.material, i, false))
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        best
    }

    /// Use the currently selected tool (reduce durability)
    pub fn use_selected_tool(&mut self) -> bool {
        if let Some(ref mut slot) = self.hotbar[self.selected_slot as usize] {
            if let Some(ref mut tool) = slot.tool_data {
                if tool.use_tool() {
                    if tool.is_broken() {
                        // Tool broke, remove from inventory
                        self.hotbar[self.selected_slot as usize] = None;
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Select a hotbar slot by index (0-8)
    pub fn select_slot(&mut self, slot: u8) {
        if slot < 9 {
            self.selected_slot = slot;
        }
    }
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Player stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub blocks_mined: u32,
    pub blocks_placed: u32,
    pub monsters_killed: u32,
    pub deaths: u32,
    pub distance_walked: u32,
    pub depth_reached: i32,
    pub items_crafted: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            blocks_mined: 0,
            blocks_placed: 0,
            monsters_killed: 0,
            deaths: 0,
            distance_walked: 0,
            depth_reached: 0,
            items_crafted: 0,
        }
    }
}

/// Game state for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Player handle (display name)
    pub handle: Option<String>,

    /// World ID (for database reference)
    pub world_id: i64,

    /// World seed for procedural generation
    pub world_seed: u64,

    /// Player position
    pub position: Position,

    /// Spawn point
    pub spawn_point: Position,

    /// Player health
    pub health: i32,
    pub max_health: i32,

    /// Player hunger (affects health regen)
    pub hunger: i32,
    pub max_hunger: i32,

    /// Experience and level
    pub experience: i32,
    pub level: i32,

    /// Player inventory
    pub inventory: PlayerInventory,

    /// Current world tick (for day/night cycle)
    pub world_tick: u32,

    /// Current day number
    pub day: u32,

    /// Player stats
    pub stats: PlayerStats,

    /// Message to display on next render
    #[serde(default)]
    pub last_message: Option<String>,

    /// Cursor offset from player for mining/building
    #[serde(default)]
    pub cursor_offset: (i32, i32),

    /// Whether player is in "build mode" vs "move mode"
    #[serde(default)]
    pub build_mode: bool,

    /// Modified blocks that differ from procedural generation
    /// Key: "x,y", Value: BlockType
    #[serde(default)]
    pub modified_blocks: HashMap<String, BlockType>,

    /// Placed chests and their contents
    /// Key: "x,y", Value: list of (ItemType, count)
    #[serde(default)]
    pub chests: HashMap<String, Vec<(ItemType, u8)>>,
}

impl GameState {
    pub fn new(world_seed: u64) -> Self {
        // Start at surface level
        let spawn = Position::new(0, 50);

        Self {
            handle: None,
            world_id: 0,
            world_seed,
            position: spawn,
            spawn_point: spawn,
            health: 20,
            max_health: 20,
            hunger: 20,
            max_hunger: 20,
            experience: 0,
            level: 1,
            inventory: PlayerInventory::new(),
            world_tick: 0,
            day: 1,
            stats: PlayerStats::default(),
            last_message: None,
            cursor_offset: (0, 0),
            build_mode: false,
            modified_blocks: HashMap::new(),
            chests: HashMap::new(),
        }
    }

    /// Get cursor position (where player is aiming)
    pub fn cursor_position(&self) -> Position {
        Position::new(
            self.position.x + self.cursor_offset.0,
            self.position.y + self.cursor_offset.1,
        )
    }

    /// Move cursor in a direction
    /// X range: -4 to 4, Y range: -4 to 3 (asymmetric - player sees more above than below)
    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        let new_x = (self.cursor_offset.0 + dx).clamp(-4, 4);
        let new_y = (self.cursor_offset.1 + dy).clamp(-4, 3);
        self.cursor_offset = (new_x, new_y);
    }

    /// Reset cursor to player position
    pub fn reset_cursor(&mut self) {
        self.cursor_offset = (0, 0);
    }

    /// Take damage
    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.health = (self.health - amount).max(0);
        self.health <= 0
    }

    /// Heal
    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Eat food (restore hunger)
    pub fn eat(&mut self, item: &ItemType) -> bool {
        let hunger_restore = match item {
            ItemType::Apple => 4,
            ItemType::Bread => 5,
            ItemType::CookedMeat => 8,
            ItemType::RawMeat => 3,
            _ => return false,
        };

        if self.hunger < self.max_hunger {
            self.hunger = (self.hunger + hunger_restore).min(self.max_hunger);
            self.inventory.remove_item(item, 1);
            true
        } else {
            false
        }
    }

    /// Add experience and check for level up
    pub fn add_experience(&mut self, amount: i32) -> bool {
        self.experience += amount;
        let xp_for_level = self.level * 100;
        if self.experience >= xp_for_level {
            self.experience -= xp_for_level;
            self.level += 1;
            self.max_health += 2;
            self.health = self.max_health;
            true
        } else {
            false
        }
    }

    /// Check if player is near a block of given type
    pub fn is_near_block(&self, block_type: BlockType, range: i32) -> bool {
        // This is checked against modified_blocks
        for dx in -range..=range {
            for dy in -range..=range {
                let key = format!("{},{}", self.position.x + dx, self.position.y + dy);
                if let Some(&b) = self.modified_blocks.get(&key) {
                    if b == block_type {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Advance world time
    pub fn advance_time(&mut self, ticks: u32) {
        use super::data::DAY_LENGTH_TICKS;

        self.world_tick += ticks;
        while self.world_tick >= DAY_LENGTH_TICKS {
            self.world_tick -= DAY_LENGTH_TICKS;
            self.day += 1;
        }

        // Hunger decreases slowly
        if ticks > 0 && self.hunger > 0 {
            // Lose 1 hunger every 500 ticks
            let hunger_loss = ticks / 500;
            if hunger_loss > 0 {
                self.hunger = (self.hunger - hunger_loss as i32).max(0);
            }
        }

        // Regenerate health if well fed
        if self.hunger >= 18 && self.health < self.max_health {
            self.heal(1);
        }

        // Take damage if starving
        if self.hunger == 0 && self.health > 1 {
            self.take_damage(1);
        }
    }

    /// Respawn after death
    pub fn respawn(&mut self) {
        self.position = self.spawn_point;
        self.health = self.max_health;
        self.hunger = self.max_hunger;
        self.stats.deaths += 1;
        self.last_message = Some("You died! Respawning at spawn point...".to_string());
    }

    /// Get a modified block, or None if not modified
    pub fn get_modified_block(&self, x: i32, y: i32) -> Option<BlockType> {
        let key = format!("{},{}", x, y);
        self.modified_blocks.get(&key).copied()
    }

    /// Set a modified block
    pub fn set_modified_block(&mut self, x: i32, y: i32, block: BlockType) {
        let key = format!("{},{}", x, y);
        self.modified_blocks.insert(key, block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new(12345);
        assert_eq!(state.health, 20);
        assert_eq!(state.hunger, 20);
        assert_eq!(state.level, 1);
        assert_eq!(state.day, 1);
    }

    #[test]
    fn test_inventory_add_remove() {
        let mut inv = PlayerInventory::new();

        // Add items
        let remaining = inv.add_item(ItemType::Block(BlockType::Dirt), 10);
        assert_eq!(remaining, 0);
        assert_eq!(inv.count_item(&ItemType::Block(BlockType::Dirt)), 10);

        // Remove items
        let removed = inv.remove_item(&ItemType::Block(BlockType::Dirt), 5);
        assert_eq!(removed, 5);
        assert_eq!(inv.count_item(&ItemType::Block(BlockType::Dirt)), 5);
    }

    #[test]
    fn test_inventory_stacking() {
        let mut inv = PlayerInventory::new();

        // Add more than max stack
        inv.add_item(ItemType::Block(BlockType::Stone), 64);
        inv.add_item(ItemType::Block(BlockType::Stone), 10);

        assert_eq!(inv.count_item(&ItemType::Block(BlockType::Stone)), 74);
    }

    #[test]
    fn test_tool_no_stacking() {
        let mut inv = PlayerInventory::new();

        inv.add_item(ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Wood), 1);
        inv.add_item(ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Wood), 1);

        // Should be in two separate slots
        let mut count = 0;
        for slot in inv.hotbar.iter().chain(inv.main.iter()) {
            if let Some(s) = slot {
                if matches!(s.item, ItemType::Tool(ToolType::Pickaxe, ToolMaterial::Wood)) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_damage_and_healing() {
        let mut state = GameState::new(0);
        state.health = 20;

        state.take_damage(5);
        assert_eq!(state.health, 15);

        state.heal(3);
        assert_eq!(state.health, 18);

        // Can't heal above max
        state.heal(100);
        assert_eq!(state.health, 20);
    }

    #[test]
    fn test_level_up() {
        let mut state = GameState::new(0);
        assert_eq!(state.level, 1);

        // Need 100 XP for level 2
        let leveled = state.add_experience(50);
        assert!(!leveled);

        let leveled = state.add_experience(50);
        assert!(leveled);
        assert_eq!(state.level, 2);
        assert_eq!(state.max_health, 22);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = GameState::new(0);
        state.position = Position::new(10, 20);

        state.move_cursor(2, -1);
        assert_eq!(state.cursor_position(), Position::new(12, 19));

        // Can't move beyond range
        state.move_cursor(10, 10);
        assert_eq!(state.cursor_offset, (4, 3));
    }

    #[test]
    fn test_serialization() {
        let state = GameState::new(42);
        let json = serde_json::to_string(&state).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.world_seed, 42);
        assert_eq!(restored.health, 20);
    }
}
