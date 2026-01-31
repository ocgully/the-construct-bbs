//! World navigation and room interactions for Morningmist

#![allow(dead_code)]

use super::data::{get_room, get_item, get_npc, RoomSpecial, Region, ItemType, NpcType};
use super::state::GameState;

/// Result of attempting to move
#[derive(Debug)]
pub enum MoveResult {
    /// Successfully moved to new room
    Success {
        room_key: String,
        room_name: String,
        room_description: String,
        exits: Vec<String>,
        items: Vec<String>,
        npcs: Vec<String>,
    },
    /// Invalid direction
    InvalidDirection(String),
    /// Region locked (need higher level)
    RegionLocked { required_level: u8 },
    /// Need a key
    NeedKey(String),
    /// In combat, can't move
    InCombat,
}

/// Attempt to move in a direction
pub fn try_move(state: &mut GameState, direction: &str) -> MoveResult {
    // Can't move while in combat
    if state.combat.is_some() {
        return MoveResult::InCombat;
    }

    // Get current room
    let current_room = match get_room(&state.current_room) {
        Some(r) => r,
        None => return MoveResult::InvalidDirection("You are nowhere!".to_string()),
    };

    // Find the exit
    let direction_lower = direction.to_lowercase();
    let destination = current_room.exits.iter()
        .find(|(dir, _)| *dir == direction_lower)
        .map(|(_, dest)| *dest);

    match destination {
        Some(dest_key) => {
            // Get destination room
            if let Some(dest_room) = get_room(dest_key) {
                // Check region access
                if !state.can_access_region(dest_room.region) {
                    return MoveResult::RegionLocked {
                        required_level: dest_room.region.required_level(),
                    };
                }

                // Check for special key requirements
                match (current_room.key, dest_key) {
                    ("crossroads", "golden_gate") | ("golden_gate", "sunlit_path") => {
                        // Need golden key for Golden Forest
                        if dest_room.region == Region::GoldenForest && !state.has_item("golden_key") {
                            return MoveResult::NeedKey("golden_key".to_string());
                        }
                    }
                    ("upper_hall", "dragon_lair") => {
                        // Need dragon key for dragon's lair
                        if !state.has_item("dragon_key") {
                            return MoveResult::NeedKey("dragon_key".to_string());
                        }
                    }
                    _ => {}
                }

                // Move to new room
                state.current_room = dest_key.to_string();

                MoveResult::Success {
                    room_key: dest_key.to_string(),
                    room_name: dest_room.name.to_string(),
                    room_description: dest_room.description.to_string(),
                    exits: dest_room.exits.iter().map(|(d, _)| d.to_string()).collect(),
                    items: dest_room.items.iter().map(|s| s.to_string()).collect(),
                    npcs: dest_room.npcs.iter().map(|s| s.to_string()).collect(),
                }
            } else {
                MoveResult::InvalidDirection(format!("The way {} is blocked.", direction))
            }
        }
        None => {
            let valid_exits: Vec<&str> = current_room.exits.iter().map(|(d, _)| *d).collect();
            MoveResult::InvalidDirection(format!(
                "You can't go {}. Exits: {}",
                direction,
                valid_exits.join(", ")
            ))
        }
    }
}

/// Get the current room details
pub fn get_current_room_details(state: &GameState) -> Option<RoomDetails> {
    let room = get_room(&state.current_room)?;

    let items: Vec<ItemInfo> = room.items.iter()
        .filter_map(|key| {
            get_item(key).map(|item| ItemInfo {
                key: key.to_string(),
                name: item.name.to_string(),
                description: item.description.to_string(),
            })
        })
        .collect();

    let npcs: Vec<NpcInfo> = room.npcs.iter()
        .filter_map(|key| {
            get_npc(key).map(|npc| NpcInfo {
                key: key.to_string(),
                name: npc.name.to_string(),
                description: npc.description.to_string(),
                npc_type: npc.npc_type,
            })
        })
        .collect();

    Some(RoomDetails {
        key: room.key.to_string(),
        name: room.name.to_string(),
        description: room.description.to_string(),
        region: room.region,
        exits: room.exits.iter().map(|(d, _)| d.to_string()).collect(),
        items,
        npcs,
        special: room.special,
    })
}

/// Room details for display
#[derive(Debug)]
pub struct RoomDetails {
    pub key: String,
    pub name: String,
    pub description: String,
    pub region: Region,
    pub exits: Vec<String>,
    pub items: Vec<ItemInfo>,
    pub npcs: Vec<NpcInfo>,
    pub special: Option<RoomSpecial>,
}

/// Item info for display
#[derive(Debug)]
pub struct ItemInfo {
    pub key: String,
    pub name: String,
    pub description: String,
}

/// NPC info for display
#[derive(Debug)]
pub struct NpcInfo {
    pub key: String,
    pub name: String,
    pub description: String,
    pub npc_type: NpcType,
}

/// Attempt to take an item from the current room
pub fn try_take_item(state: &mut GameState, item_name: &str) -> Result<String, String> {
    let room = get_room(&state.current_room)
        .ok_or("You are nowhere!")?;

    // Find the item in the room (case-insensitive partial match)
    let item_lower = item_name.to_lowercase();
    let found_key = room.items.iter()
        .find(|key| {
            let key_lower = key.to_lowercase();
            key_lower == item_lower ||
            key_lower.contains(&item_lower) ||
            key.split('_').any(|part| part.to_lowercase() == item_lower)
        })
        .map(|k| k.to_string());

    match found_key {
        Some(key) => {
            if let Some(item) = get_item(&key) {
                if state.add_item(&key, 1) {
                    Ok(format!("You take the {}.", item.name))
                } else {
                    Err("Your inventory is full!".to_string())
                }
            } else {
                Err("That item doesn't exist.".to_string())
            }
        }
        None => Err(format!("You don't see any '{}' here.", item_name)),
    }
}

/// Drop an item from inventory
pub fn drop_item(state: &mut GameState, item_name: &str) -> Result<String, String> {
    // Find matching item in inventory
    let item_lower = item_name.to_lowercase();
    let found_key = state.inventory.keys()
        .find(|key| {
            let key_lower = key.to_lowercase();
            key_lower == item_lower ||
            key_lower.contains(&item_lower) ||
            key.split('_').any(|part| part.to_lowercase() == item_lower)
        })
        .cloned();

    match found_key {
        Some(key) => {
            if state.remove_item(&key, 1) {
                let name = get_item(&key).map(|i| i.name).unwrap_or("item");
                Ok(format!("You drop the {}.", name))
            } else {
                Err("You don't have that.".to_string())
            }
        }
        None => Err(format!("You don't have any '{}'.", item_name)),
    }
}

/// Use an item from inventory
pub fn use_item(state: &mut GameState, item_name: &str) -> Result<String, String> {
    // Find matching item in inventory
    let item_lower = item_name.to_lowercase();
    let found_key = state.inventory.keys()
        .find(|key| {
            let key_lower = key.to_lowercase();
            key_lower == item_lower ||
            key_lower.contains(&item_lower) ||
            key.split('_').any(|part| part.to_lowercase() == item_lower)
        })
        .cloned();

    match found_key {
        Some(key) => {
            let item = get_item(&key).ok_or("Item not found.")?;

            match item.item_type {
                ItemType::Consumable => {
                    // Use and consume
                    state.remove_item(&key, 1);

                    match key.as_str() {
                        "health_potion" => {
                            state.heal(30);
                            Ok("You drink the health potion. Restored 30 HP!".to_string())
                        }
                        "mana_potion" => {
                            state.restore_mana(25);
                            Ok("You drink the mana potion. Restored 25 mana!".to_string())
                        }
                        "glowing_mushroom" => {
                            state.heal(10);
                            Ok("You eat the glowing mushroom. It tastes strange... Restored 10 HP!".to_string())
                        }
                        _ => Ok(format!("You use the {}.", item.name))
                    }
                }
                ItemType::Scroll => {
                    // Use scroll to learn spell
                    super::magic::use_scroll(state, &key)
                }
                ItemType::SpellComponent => {
                    Ok(format!("The {} is a spell component. Use it at the right place.", item.name))
                }
                _ => {
                    Err(format!("You can't use the {} that way.", item.name))
                }
            }
        }
        None => Err(format!("You don't have any '{}'.", item_name)),
    }
}

/// Equip an item
pub fn equip_item(state: &mut GameState, item_name: &str) -> Result<String, String> {
    let item_lower = item_name.to_lowercase();
    let found_key = state.inventory.keys()
        .find(|key| {
            let key_lower = key.to_lowercase();
            key_lower == item_lower ||
            key_lower.contains(&item_lower)
        })
        .cloned();

    match found_key {
        Some(key) => {
            let item = get_item(&key).ok_or("Item not found.")?;

            match item.item_type {
                ItemType::Weapon => {
                    let old = state.equipped_weapon.take();
                    state.equipped_weapon = Some(key.clone());
                    let msg = if let Some(old_key) = old {
                        let old_name = get_item(&old_key).map(|i| i.name).unwrap_or("item");
                        format!("You unequip {} and equip {}.", old_name, item.name)
                    } else {
                        format!("You equip {}.", item.name)
                    };
                    Ok(msg)
                }
                ItemType::Armor => {
                    let old = state.equipped_armor.take();
                    state.equipped_armor = Some(key.clone());
                    let msg = if let Some(old_key) = old {
                        let old_name = get_item(&old_key).map(|i| i.name).unwrap_or("item");
                        format!("You take off {} and put on {}.", old_name, item.name)
                    } else {
                        format!("You put on {}.", item.name)
                    };
                    Ok(msg)
                }
                _ => Err(format!("You can't equip {}.", item.name)),
            }
        }
        None => Err(format!("You don't have any '{}'.", item_name)),
    }
}

/// Unequip an item
pub fn unequip_item(state: &mut GameState, item_name: &str) -> Result<String, String> {
    let item_lower = item_name.to_lowercase();

    // Check weapon
    if let Some(ref weapon_key) = state.equipped_weapon {
        if weapon_key.to_lowercase().contains(&item_lower) {
            let name = get_item(weapon_key).map(|i| i.name).unwrap_or("weapon");
            state.equipped_weapon = None;
            return Ok(format!("You unequip {}.", name));
        }
    }

    // Check armor
    if let Some(ref armor_key) = state.equipped_armor {
        if armor_key.to_lowercase().contains(&item_lower) {
            let name = get_item(armor_key).map(|i| i.name).unwrap_or("armor");
            state.equipped_armor = None;
            return Ok(format!("You take off {}.", name));
        }
    }

    Err("You don't have that equipped.".to_string())
}

/// Talk to an NPC in the current room
pub fn talk_to_npc(state: &mut GameState, npc_name: &str) -> Result<(String, Vec<String>), String> {
    let room = get_room(&state.current_room)
        .ok_or("You are nowhere!")?;

    // Find the NPC
    let npc_lower = npc_name.to_lowercase();
    let found_key = room.npcs.iter()
        .find(|key| {
            let key_lower = key.to_lowercase();
            key_lower == npc_lower ||
            key_lower.contains(&npc_lower) ||
            key.split('_').any(|part| part.to_lowercase() == npc_lower)
        })
        .map(|k| k.to_string());

    match found_key {
        Some(key) => {
            let npc = get_npc(&key).ok_or("That person doesn't exist.")?;

            // Mark as met
            state.meet_npc(&key);

            // Get dialogue
            let dialogue: Vec<String> = npc.dialogue.iter().map(|s| s.to_string()).collect();

            Ok((npc.name.to_string(), dialogue))
        }
        None => Err(format!("You don't see '{}' here.", npc_name)),
    }
}

/// Rest at an inn
pub fn rest_at_inn(state: &mut GameState) -> Result<String, String> {
    let room = get_room(&state.current_room)
        .ok_or("You are nowhere!")?;

    if room.special != Some(RoomSpecial::Inn) {
        return Err("You can only rest at an inn.".to_string());
    }

    let cost = 10 * state.level as i64;

    if state.gold < cost {
        return Err(format!("You need {} gold to rest here.", cost));
    }

    state.gold -= cost;
    state.health = state.max_health;
    state.mana = state.max_mana;

    Ok(format!(
        "You rest peacefully at the inn. (-{} gold)\n\
         Health and mana fully restored!",
        cost
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_move_success() {
        let mut state = GameState::new("Test");
        state.current_room = "village_square".to_string();

        let result = try_move(&mut state, "north");
        match result {
            MoveResult::Success { room_key, .. } => {
                assert_eq!(room_key, "village_inn");
                assert_eq!(state.current_room, "village_inn");
            }
            _ => panic!("Expected successful move"),
        }
    }

    #[test]
    fn test_try_move_invalid() {
        let mut state = GameState::new("Test");
        state.current_room = "village_square".to_string();

        let result = try_move(&mut state, "up");
        assert!(matches!(result, MoveResult::InvalidDirection(_)));
    }

    #[test]
    fn test_try_move_in_combat() {
        let mut state = GameState::new("Test");
        state.current_room = "village_square".to_string();
        state.combat = Some(super::super::state::CombatState {
            monster_key: "rat".to_string(),
            monster_hp: 10,
            monster_max_hp: 10,
            player_turn: true,
            shield_active: false,
            shield_power: 0,
        });

        let result = try_move(&mut state, "north");
        assert!(matches!(result, MoveResult::InCombat));
    }

    #[test]
    fn test_get_current_room_details() {
        let state = GameState::new("Test");
        let details = get_current_room_details(&state);

        assert!(details.is_some());
        let details = details.unwrap();
        assert_eq!(details.key, "village_square");
    }

    #[test]
    fn test_take_item() {
        let mut state = GameState::new("Test");
        state.current_room = "village_library".to_string();

        let result = try_take_item(&mut state, "scroll");
        assert!(result.is_ok());
        assert!(state.has_item("scroll_light"));
    }

    #[test]
    fn test_use_health_potion() {
        let mut state = GameState::new("Test");
        state.add_item("health_potion", 1);
        state.health = 20;
        state.max_health = 50;

        let result = use_item(&mut state, "potion");
        assert!(result.is_ok());
        assert_eq!(state.health, 50);  // Full heal
        assert!(!state.has_item("health_potion"));
    }

    #[test]
    fn test_equip_weapon() {
        let mut state = GameState::new("Test");
        state.add_item("wooden_staff", 1);

        let result = equip_item(&mut state, "staff");
        assert!(result.is_ok());
        assert_eq!(state.equipped_weapon, Some("wooden_staff".to_string()));
    }

    #[test]
    fn test_rest_at_inn() {
        let mut state = GameState::new("Test");
        state.current_room = "village_inn".to_string();
        state.health = 20;
        state.gold = 100;

        let result = rest_at_inn(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.health, state.max_health);
    }
}
