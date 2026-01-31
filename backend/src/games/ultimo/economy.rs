//! Economy system for Ultimo
//!
//! Handles trading, banking, and market dynamics.

use super::data::get_item;
use super::state::{Character, InventoryItem};
use serde::{Deserialize, Serialize};

/// A trade offer between players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOffer {
    pub id: i64,
    pub seller_id: i64,
    pub seller_name: String,
    /// Item being sold
    pub item_key: String,
    pub quantity: u32,
    /// Price in gold
    pub price: i64,
    pub created_at: String,
}

/// Bank account operations
#[derive(Debug, Clone)]
pub enum BankOperation {
    Deposit(i64),
    Withdraw(i64),
}

/// Perform bank operation
pub fn bank_operation(char: &mut Character, operation: BankOperation) -> Result<String, String> {
    match operation {
        BankOperation::Deposit(amount) => {
            if amount <= 0 {
                return Err("Amount must be positive".to_string());
            }
            if char.gold < amount {
                return Err(format!("Not enough gold. You have {} gold.", char.gold));
            }
            char.gold -= amount;
            char.bank_gold += amount;
            Ok(format!("Deposited {} gold. Bank balance: {}", amount, char.bank_gold))
        }
        BankOperation::Withdraw(amount) => {
            if amount <= 0 {
                return Err("Amount must be positive".to_string());
            }
            if char.bank_gold < amount {
                return Err(format!("Not enough in bank. Balance: {} gold.", char.bank_gold));
            }
            char.bank_gold -= amount;
            char.gold += amount;
            Ok(format!("Withdrew {} gold. Pocket gold: {}", amount, char.gold))
        }
    }
}

/// Calculate buy price from NPC (markup)
pub fn npc_buy_price(base_price: i64, price_multiplier: f32) -> i64 {
    (base_price as f32 * price_multiplier * 1.0) as i64
}

/// Calculate sell price to NPC (discount)
pub fn npc_sell_price(base_price: i64) -> i64 {
    base_price / 2 // NPCs buy at 50%
}

/// Calculate value of character's total wealth
pub fn calculate_net_worth(char: &Character) -> i64 {
    let inventory_value: i64 = char
        .inventory
        .iter()
        .map(|slot| {
            get_item(&slot.item_key)
                .map(|item| item.base_price * slot.quantity as i64)
                .unwrap_or(0)
        })
        .sum();

    char.gold + char.bank_gold + inventory_value
}

/// Purchase item from NPC
pub fn purchase_from_npc(
    char: &mut Character,
    item_key: &str,
    price_multiplier: f32,
) -> Result<String, String> {
    let item = get_item(item_key).ok_or("Item not found")?;
    let price = npc_buy_price(item.base_price, price_multiplier);

    if char.gold < price {
        return Err(format!(
            "Not enough gold. Need {} gold, have {}.",
            price, char.gold
        ));
    }

    if char.inventory.len() as u32 >= char.max_inventory_slots && !item.stackable {
        return Err("Inventory full!".to_string());
    }

    char.gold -= price;

    // Add item with durability if it's equipment
    match item.item_type {
        super::data::ItemType::Weapon
        | super::data::ItemType::Armor
        | super::data::ItemType::Shield => {
            char.inventory
                .push(InventoryItem::with_durability(item_key, 100));
        }
        _ => {
            char.add_item(item_key, 1);
        }
    }

    Ok(format!("Purchased {} for {} gold.", item.name, price))
}

/// Sell item to NPC
pub fn sell_to_npc(char: &mut Character, inventory_index: usize) -> Result<String, String> {
    if inventory_index >= char.inventory.len() {
        return Err("Invalid item index".to_string());
    }

    let slot = &char.inventory[inventory_index];
    let item = get_item(&slot.item_key).ok_or("Item not found")?;
    let sell_price = npc_sell_price(item.base_price);
    let item_name = item.name.to_string();
    let item_key = slot.item_key.clone();

    char.gold += sell_price;
    char.remove_item(&item_key, 1);

    Ok(format!("Sold {} for {} gold.", item_name, sell_price))
}

/// Create a player trade offer
pub fn create_trade_offer(
    seller: &Character,
    item_key: &str,
    quantity: u32,
    price: i64,
) -> Result<TradeOffer, String> {
    // Verify seller has the item
    if seller.get_item_count(item_key) < quantity {
        return Err("Not enough items to sell".to_string());
    }

    let _item = get_item(item_key).ok_or("Item not found")?;

    Ok(TradeOffer {
        id: 0, // Will be set by DB
        seller_id: seller.user_id,
        seller_name: seller.name.clone(),
        item_key: item_key.to_string(),
        quantity,
        price,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Execute a player-to-player trade
pub fn execute_trade(
    buyer: &mut Character,
    seller_items: &mut Vec<InventoryItem>,
    offer: &TradeOffer,
) -> Result<String, String> {
    // Check buyer has enough gold
    if buyer.gold < offer.price {
        return Err(format!(
            "Not enough gold. Need {} gold.",
            offer.price
        ));
    }

    // Check buyer has inventory space
    if buyer.inventory.len() as u32 >= buyer.max_inventory_slots {
        return Err("Inventory full!".to_string());
    }

    // Find and remove item from seller
    let mut removed = 0u32;
    for slot in seller_items.iter_mut() {
        if slot.item_key == offer.item_key && removed < offer.quantity {
            let take = slot.quantity.min(offer.quantity - removed);
            slot.quantity -= take;
            removed += take;
        }
    }
    seller_items.retain(|s| s.quantity > 0);

    if removed < offer.quantity {
        return Err("Seller no longer has the items".to_string());
    }

    // Transfer gold and items
    buyer.gold -= offer.price;
    buyer.add_item(&offer.item_key, offer.quantity);

    let item_name = get_item(&offer.item_key)
        .map(|i| i.name)
        .unwrap_or(&offer.item_key);

    Ok(format!(
        "Purchased {} {} for {} gold.",
        offer.quantity, item_name, offer.price
    ))
}

/// Calculate recommended price based on item rarity and market
pub fn suggest_price(item_key: &str, quantity: u32) -> i64 {
    let base = get_item(item_key)
        .map(|i| i.base_price)
        .unwrap_or(1);

    // Player trades typically at 70-80% of NPC price
    (base as f32 * 0.75 * quantity as f32) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bank_deposit() {
        let mut char = Character::new("Banker", 1);
        char.gold = 1000;
        char.bank_gold = 0;

        let result = bank_operation(&mut char, BankOperation::Deposit(500));
        assert!(result.is_ok());
        assert_eq!(char.gold, 500);
        assert_eq!(char.bank_gold, 500);
    }

    #[test]
    fn test_bank_withdraw() {
        let mut char = Character::new("Banker", 1);
        char.gold = 100;
        char.bank_gold = 500;

        let result = bank_operation(&mut char, BankOperation::Withdraw(300));
        assert!(result.is_ok());
        assert_eq!(char.gold, 400);
        assert_eq!(char.bank_gold, 200);
    }

    #[test]
    fn test_bank_overdraft() {
        let mut char = Character::new("Banker", 1);
        char.gold = 100;
        char.bank_gold = 0;

        let result = bank_operation(&mut char, BankOperation::Withdraw(500));
        assert!(result.is_err());
    }

    #[test]
    fn test_npc_prices() {
        let base_price = 100;

        let buy_price = npc_buy_price(base_price, 1.0);
        assert_eq!(buy_price, 100);

        let sell_price = npc_sell_price(base_price);
        assert_eq!(sell_price, 50); // 50%
    }

    #[test]
    fn test_purchase_from_npc() {
        let mut char = Character::new("Shopper", 1);
        char.gold = 1000;

        let result = purchase_from_npc(&mut char, "dagger", 1.0);
        assert!(result.is_ok());
        assert!(char.gold < 1000);
        assert!(char.get_item_count("dagger") > 0);
    }

    #[test]
    fn test_net_worth() {
        let mut char = Character::new("Rich", 1);
        char.gold = 500;
        char.bank_gold = 1000;
        char.add_item("iron_ingot", 10); // 25 each = 250

        let worth = calculate_net_worth(&char);
        assert!(worth >= 1750); // 500 + 1000 + 250 + starting items
    }

    #[test]
    fn test_suggest_price() {
        let price = suggest_price("iron_ingot", 10);
        assert!(price > 0);
        assert!(price < 250); // Less than NPC price of 25 * 10
    }
}
