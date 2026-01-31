//! Housing system for Ultimo
//!
//! Handles player housing, property ownership, and customization.

use super::state::Character;
use serde::{Deserialize, Serialize};

/// House types available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HouseType {
    SmallCottage,
    MediumHouse,
    LargeHouse,
    Tower,
    Castle,
}

impl HouseType {
    pub fn name(&self) -> &'static str {
        match self {
            HouseType::SmallCottage => "Small Cottage",
            HouseType::MediumHouse => "Medium House",
            HouseType::LargeHouse => "Large House",
            HouseType::Tower => "Tower",
            HouseType::Castle => "Castle",
        }
    }

    pub fn price(&self) -> i64 {
        match self {
            HouseType::SmallCottage => 10000,
            HouseType::MediumHouse => 50000,
            HouseType::LargeHouse => 150000,
            HouseType::Tower => 300000,
            HouseType::Castle => 1000000,
        }
    }

    pub fn storage_slots(&self) -> u32 {
        match self {
            HouseType::SmallCottage => 50,
            HouseType::MediumHouse => 150,
            HouseType::LargeHouse => 300,
            HouseType::Tower => 500,
            HouseType::Castle => 1000,
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            HouseType::SmallCottage => 5,
            HouseType::MediumHouse => 8,
            HouseType::LargeHouse => 12,
            HouseType::Tower => 6,
            HouseType::Castle => 20,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            HouseType::SmallCottage => 5,
            HouseType::MediumHouse => 8,
            HouseType::LargeHouse => 10,
            HouseType::Tower => 6,
            HouseType::Castle => 15,
        }
    }

    pub fn maintenance_cost(&self) -> i64 {
        // Daily maintenance cost
        match self {
            HouseType::SmallCottage => 10,
            HouseType::MediumHouse => 50,
            HouseType::LargeHouse => 150,
            HouseType::Tower => 250,
            HouseType::Castle => 500,
        }
    }
}

/// A player's house
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct House {
    pub id: i64,
    pub owner_id: i64,
    pub owner_name: String,
    pub house_type: HouseType,
    /// Position in housing zone
    pub zone: String,
    pub x: i32,
    pub y: i32,
    /// Custom name for the house
    pub name: Option<String>,
    /// Storage contents (item_key, quantity)
    pub storage: Vec<(String, u32)>,
    /// Is house public (anyone can enter)
    pub is_public: bool,
    /// Friends list (user_ids who can enter)
    pub friends: Vec<i64>,
    /// Co-owners (user_ids who can modify)
    pub co_owners: Vec<i64>,
    /// House decorations/customization
    pub decorations: Vec<HouseDecoration>,
    /// Date purchased
    pub purchased_at: String,
    /// Last maintenance payment
    pub last_maintenance: String,
}

/// A decoration placed in a house
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseDecoration {
    pub item_key: String,
    pub x: u32,
    pub y: u32,
}

/// Available house plot
#[derive(Debug, Clone)]
pub struct HousePlot {
    pub zone: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_occupied: bool,
}

impl House {
    pub fn new(
        owner_id: i64,
        owner_name: &str,
        house_type: HouseType,
        zone: &str,
        x: i32,
        y: i32,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            owner_id,
            owner_name: owner_name.to_string(),
            house_type,
            zone: zone.to_string(),
            x,
            y,
            name: None,
            storage: Vec::new(),
            is_public: false,
            friends: Vec::new(),
            co_owners: Vec::new(),
            decorations: Vec::new(),
            purchased_at: now.clone(),
            last_maintenance: now,
        }
    }

    pub fn storage_used(&self) -> u32 {
        self.storage.iter().map(|(_, qty)| qty).sum()
    }

    pub fn storage_free(&self) -> u32 {
        self.house_type.storage_slots() - self.storage_used()
    }

    pub fn can_access(&self, user_id: i64) -> bool {
        self.owner_id == user_id
            || self.is_public
            || self.friends.contains(&user_id)
            || self.co_owners.contains(&user_id)
    }

    pub fn can_modify(&self, user_id: i64) -> bool {
        self.owner_id == user_id || self.co_owners.contains(&user_id)
    }

    pub fn add_to_storage(&mut self, item_key: &str, quantity: u32) -> bool {
        if self.storage_free() < quantity {
            return false;
        }

        // Find existing stack or create new
        if let Some((_, qty)) = self.storage.iter_mut().find(|(k, _)| k == item_key) {
            *qty += quantity;
        } else {
            self.storage.push((item_key.to_string(), quantity));
        }

        true
    }

    pub fn remove_from_storage(&mut self, item_key: &str, quantity: u32) -> bool {
        if let Some((_, qty)) = self.storage.iter_mut().find(|(k, _)| k == item_key) {
            if *qty >= quantity {
                *qty -= quantity;
                if *qty == 0 {
                    self.storage.retain(|(_, q)| *q > 0);
                }
                return true;
            }
        }
        false
    }

    pub fn get_storage_count(&self, item_key: &str) -> u32 {
        self.storage
            .iter()
            .find(|(k, _)| k == item_key)
            .map(|(_, qty)| *qty)
            .unwrap_or(0)
    }
}

/// Check if character can purchase a house
pub fn can_purchase_house(char: &Character, house_type: HouseType) -> Result<(), String> {
    // Check if already owns a house
    if char.house_id.is_some() {
        return Err("You already own a house!".to_string());
    }

    // Check gold
    let price = house_type.price();
    if char.gold < price {
        return Err(format!(
            "Not enough gold. Need {} gold.",
            price
        ));
    }

    Ok(())
}

/// Purchase a house for a character
pub fn purchase_house(
    char: &mut Character,
    house_type: HouseType,
    zone: &str,
    x: i32,
    y: i32,
) -> Result<House, String> {
    can_purchase_house(char, house_type)?;

    let price = house_type.price();
    char.gold -= price;

    let house = House::new(char.user_id, &char.name, house_type, zone, x, y);

    Ok(house)
}

/// Pay maintenance on a house
pub fn pay_maintenance(char: &mut Character, house: &mut House) -> Result<String, String> {
    let cost = house.house_type.maintenance_cost();

    if char.gold < cost {
        return Err(format!(
            "Not enough gold for maintenance. Need {} gold.",
            cost
        ));
    }

    char.gold -= cost;
    house.last_maintenance = chrono::Utc::now().to_rfc3339();

    Ok(format!("Paid {} gold for house maintenance.", cost))
}

/// Check if house needs maintenance
pub fn needs_maintenance(house: &House) -> bool {
    if let Ok(last_paid) = chrono::DateTime::parse_from_rfc3339(&house.last_maintenance) {
        let now = chrono::Utc::now();
        let days_since = (now - last_paid.with_timezone(&chrono::Utc))
            .num_days();
        days_since >= 7 // Maintenance due weekly
    } else {
        true
    }
}

/// Check if house is condemned (no maintenance for too long)
pub fn is_condemned(house: &House) -> bool {
    if let Ok(last_paid) = chrono::DateTime::parse_from_rfc3339(&house.last_maintenance) {
        let now = chrono::Utc::now();
        let days_since = (now - last_paid.with_timezone(&chrono::Utc))
            .num_days();
        days_since >= 30 // Condemned after 30 days without maintenance
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_house_types() {
        assert_eq!(HouseType::SmallCottage.price(), 10000);
        assert_eq!(HouseType::Castle.price(), 1000000);

        assert!(HouseType::Castle.storage_slots() > HouseType::SmallCottage.storage_slots());
    }

    #[test]
    fn test_house_storage() {
        let mut house = House::new(1, "Test", HouseType::SmallCottage, "housing_district", 0, 0);

        // Add items
        assert!(house.add_to_storage("iron_ore", 10));
        assert_eq!(house.get_storage_count("iron_ore"), 10);

        // Remove items
        assert!(house.remove_from_storage("iron_ore", 5));
        assert_eq!(house.get_storage_count("iron_ore"), 5);

        // Can't remove more than available
        assert!(!house.remove_from_storage("iron_ore", 10));
    }

    #[test]
    fn test_house_access() {
        let mut house = House::new(1, "Owner", HouseType::SmallCottage, "housing_district", 0, 0);

        // Owner can always access
        assert!(house.can_access(1));
        assert!(house.can_modify(1));

        // Random user cannot access
        assert!(!house.can_access(99));

        // Add friend
        house.friends.push(2);
        assert!(house.can_access(2));
        assert!(!house.can_modify(2));

        // Make public
        house.is_public = true;
        assert!(house.can_access(99));
    }

    #[test]
    fn test_purchase_house() {
        let mut char = Character::new("Buyer", 1);
        char.gold = 50000;

        let result = purchase_house(
            &mut char,
            HouseType::SmallCottage,
            "housing_district",
            10,
            10,
        );

        assert!(result.is_ok());
        assert_eq!(char.gold, 40000); // Spent 10000
    }

    #[test]
    fn test_cannot_afford_house() {
        let mut char = Character::new("Poor", 1);
        char.gold = 100;

        let result = can_purchase_house(&char, HouseType::Castle);
        assert!(result.is_err());
    }
}
