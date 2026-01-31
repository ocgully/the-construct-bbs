//! Star Trader - Economy System
//!
//! Trading mechanics, port interactions, and price calculations.

use super::data::{Commodity, TradeDirection, config};
use super::state::GameState;
use super::galaxy::{PortData, PortStock};

/// Result of a trade transaction
#[derive(Debug, Clone)]
pub struct TradeResult {
    pub success: bool,
    pub commodity: Commodity,
    pub quantity: u32,
    pub total_price: i64,
    pub message: String,
}

/// Buy commodity from a port (port sells to you)
pub fn buy_from_port(
    state: &mut GameState,
    port: &mut PortData,
    commodity: Commodity,
    quantity: u32,
) -> TradeResult {
    let port_type = port.port_type.to_port_type();

    // Check if port sells this commodity
    if port_type.direction_for(commodity) != TradeDirection::Selling {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!("This port doesn't sell {}", commodity.name()),
        };
    }

    let stock = get_stock_mut(port, commodity);

    // Check stock availability
    if stock.quantity < quantity {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!(
                "Not enough {} in stock. Available: {}",
                commodity.name(),
                stock.quantity
            ),
        };
    }

    // Check cargo space
    if state.cargo_space() < quantity {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!(
                "Not enough cargo space. Available: {}",
                state.cargo_space()
            ),
        };
    }

    // Calculate total price
    let total_price = stock.price * quantity as i64;

    // Check credits
    if state.credits < total_price {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!(
                "Not enough credits. Need: {}, Have: {}",
                total_price,
                state.credits
            ),
        };
    }

    // Execute trade
    state.credits -= total_price;
    state.cargo.add(commodity, quantity);
    stock.quantity -= quantity;
    stock.update_price();

    // Update stats
    state.stats.trades_completed += 1;
    state.stats.total_traded_value += total_price;
    state.add_experience(config::XP_PER_TRADE);
    state.update_max_credits();

    TradeResult {
        success: true,
        commodity,
        quantity,
        total_price,
        message: format!(
            "Bought {} {} for {} credits",
            quantity,
            commodity.name(),
            total_price
        ),
    }
}

/// Sell commodity to a port (port buys from you)
pub fn sell_to_port(
    state: &mut GameState,
    port: &mut PortData,
    commodity: Commodity,
    quantity: u32,
) -> TradeResult {
    let port_type = port.port_type.to_port_type();

    // Check if port buys this commodity
    if port_type.direction_for(commodity) != TradeDirection::Buying {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!("This port doesn't buy {}", commodity.name()),
        };
    }

    // Check cargo
    if state.cargo.get(commodity) < quantity {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!(
                "Not enough {} in cargo. Have: {}",
                commodity.name(),
                state.cargo.get(commodity)
            ),
        };
    }

    let stock = get_stock_mut(port, commodity);

    // Check if port can hold more
    let space = stock.max_quantity.saturating_sub(stock.quantity);
    if space < quantity {
        return TradeResult {
            success: false,
            commodity,
            quantity: 0,
            total_price: 0,
            message: format!(
                "Port can only accept {} more {}",
                space,
                commodity.name()
            ),
        };
    }

    // Calculate total price
    let total_price = stock.price * quantity as i64;

    // Execute trade
    state.credits += total_price;
    state.cargo.remove(commodity, quantity);
    stock.quantity += quantity;
    stock.update_price();

    // Update stats
    state.stats.trades_completed += 1;
    state.stats.total_traded_value += total_price;
    state.add_experience(config::XP_PER_TRADE);
    state.update_max_credits();

    TradeResult {
        success: true,
        commodity,
        quantity,
        total_price,
        message: format!(
            "Sold {} {} for {} credits",
            quantity,
            commodity.name(),
            total_price
        ),
    }
}

/// Get the current price for a commodity at a port
pub fn get_price(port: &PortData, commodity: Commodity) -> i64 {
    get_stock(port, commodity).price
}

/// Get the current quantity of a commodity at a port
pub fn get_quantity(port: &PortData, commodity: Commodity) -> u32 {
    get_stock(port, commodity).quantity
}

/// Get the trade direction for a commodity at a port
pub fn get_trade_direction(port: &PortData, commodity: Commodity) -> TradeDirection {
    port.port_type.to_port_type().direction_for(commodity)
}

/// Check if a trade pair is profitable (buy at one port, sell at another)
pub fn calculate_profit(
    buy_port: &PortData,
    sell_port: &PortData,
    commodity: Commodity,
    quantity: u32,
) -> Option<i64> {
    // Check if buy_port sells and sell_port buys
    let buy_type = buy_port.port_type.to_port_type();
    let sell_type = sell_port.port_type.to_port_type();

    if buy_type.direction_for(commodity) != TradeDirection::Selling {
        return None;  // Can't buy here
    }
    if sell_type.direction_for(commodity) != TradeDirection::Buying {
        return None;  // Can't sell there
    }

    let buy_price = get_stock(buy_port, commodity).price * quantity as i64;
    let sell_price = get_stock(sell_port, commodity).price * quantity as i64;

    Some(sell_price - buy_price)
}

/// Get stock for a commodity (immutable)
fn get_stock(port: &PortData, commodity: Commodity) -> &PortStock {
    match commodity {
        Commodity::FuelOre => &port.fuel_ore,
        Commodity::Organics => &port.organics,
        Commodity::Equipment => &port.equipment,
    }
}

/// Get stock for a commodity (mutable)
fn get_stock_mut(port: &mut PortData, commodity: Commodity) -> &mut PortStock {
    match commodity {
        Commodity::FuelOre => &mut port.fuel_ore,
        Commodity::Organics => &mut port.organics,
        Commodity::Equipment => &mut port.equipment,
    }
}

/// Buy fighters at StarDock
pub fn buy_fighters(state: &mut GameState, quantity: u32) -> Result<String, String> {
    let cost = quantity as i64 * config::FIGHTER_COST;

    if state.credits < cost {
        return Err(format!("Need {} credits, have {}", cost, state.credits));
    }

    let ship = state.ship().ok_or("Invalid ship")?;
    let max_can_hold = ship.max_fighters.saturating_sub(state.fighters);

    if quantity > max_can_hold {
        return Err(format!("Can only hold {} more fighters", max_can_hold));
    }

    state.credits -= cost;
    state.fighters += quantity;
    state.update_max_credits();

    Ok(format!("Purchased {} fighters for {} credits", quantity, cost))
}

/// Buy shields at StarDock
pub fn buy_shields(state: &mut GameState, quantity: u32) -> Result<String, String> {
    let cost = quantity as i64 * config::SHIELD_COST;

    if state.credits < cost {
        return Err(format!("Need {} credits, have {}", cost, state.credits));
    }

    let ship = state.ship().ok_or("Invalid ship")?;
    let max_can_hold = ship.max_shields.saturating_sub(state.shields);

    if quantity > max_can_hold {
        return Err(format!("Can only hold {} more shields", max_can_hold));
    }

    state.credits -= cost;
    state.shields += quantity;
    state.update_max_credits();

    Ok(format!("Purchased {} shields for {} credits", quantity, cost))
}

/// Update port prices (called periodically)
pub fn update_port_prices(port: &mut PortData) {
    port.fuel_ore.update_price();
    port.organics.update_price();
    port.equipment.update_price();
}

/// Regenerate port stock over time
pub fn regenerate_port_stock(port: &mut PortData, ticks: u32) {
    // Each tick regenerates ~1% of max capacity
    let regen_rate = 0.01;

    regenerate_stock(&mut port.fuel_ore, ticks, regen_rate);
    regenerate_stock(&mut port.organics, ticks, regen_rate);
    regenerate_stock(&mut port.equipment, ticks, regen_rate);
}

fn regenerate_stock(stock: &mut PortStock, ticks: u32, rate: f64) {
    let regen = (stock.max_quantity as f64 * rate * ticks as f64) as u32;

    // If port buys (stock tends to max), move toward max
    // If port sells (stock tends to max too, for selling)
    // Actually both should regenerate toward ~75% capacity
    let target = (stock.max_quantity as f64 * 0.75) as u32;

    if stock.quantity < target {
        stock.quantity = (stock.quantity + regen).min(target);
    } else if stock.quantity > target {
        // Slowly decrease excess
        stock.quantity = stock.quantity.saturating_sub(regen / 2);
        stock.quantity = stock.quantity.max(target);
    }

    stock.update_price();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::star_trader::galaxy::{PortStock, PortTypeCode};

    fn create_test_port() -> PortData {
        PortData {
            name: "Test Port".to_string(),
            port_type: PortTypeCode::SBS,  // Sells Ore/Equip, Buys Org
            fuel_ore: PortStock::new(1000, config::FUEL_ORE_BASE_PRICE),
            organics: PortStock::new(1000, config::ORGANICS_BASE_PRICE),
            equipment: PortStock::new(1000, config::EQUIPMENT_BASE_PRICE),
        }
    }

    #[test]
    fn test_buy_from_port() {
        let mut state = GameState::new(1, "Test".to_string());
        let mut port = create_test_port();

        // Port sells fuel ore
        let result = buy_from_port(&mut state, &mut port, Commodity::FuelOre, 10);
        assert!(result.success);
        assert_eq!(state.cargo.fuel_ore, 10);
        assert!(state.credits < config::STARTING_CREDITS);
    }

    #[test]
    fn test_sell_to_port() {
        let mut state = GameState::new(1, "Test".to_string());
        state.cargo.organics = 20;
        let mut port = create_test_port();

        // Port buys organics
        let result = sell_to_port(&mut state, &mut port, Commodity::Organics, 10);
        assert!(result.success);
        assert_eq!(state.cargo.organics, 10);
        assert!(state.credits > config::STARTING_CREDITS);
    }

    #[test]
    fn test_buy_wrong_direction() {
        let mut state = GameState::new(1, "Test".to_string());
        let mut port = create_test_port();

        // Port doesn't sell organics (it buys them)
        let result = buy_from_port(&mut state, &mut port, Commodity::Organics, 10);
        assert!(!result.success);
    }

    #[test]
    fn test_sell_wrong_direction() {
        let mut state = GameState::new(1, "Test".to_string());
        state.cargo.fuel_ore = 20;
        let mut port = create_test_port();

        // Port doesn't buy fuel ore (it sells it)
        let result = sell_to_port(&mut state, &mut port, Commodity::FuelOre, 10);
        assert!(!result.success);
    }

    #[test]
    fn test_not_enough_credits() {
        let mut state = GameState::new(1, "Test".to_string());
        state.credits = 10;  // Very little (FuelOre costs ~20cr per unit)
        let mut port = create_test_port();

        // Buy a small quantity that fits in cargo but costs more than 10 credits
        let result = buy_from_port(&mut state, &mut port, Commodity::FuelOre, 1);
        assert!(!result.success);
        assert!(result.message.contains("Not enough credits"));
    }

    #[test]
    fn test_not_enough_cargo_space() {
        let mut state = GameState::new(1, "Test".to_string());
        state.cargo.fuel_ore = 15;  // 15 of 20 capacity used
        let mut port = create_test_port();

        let result = buy_from_port(&mut state, &mut port, Commodity::FuelOre, 10);
        assert!(!result.success);
        assert!(result.message.contains("Not enough cargo"));
    }

    #[test]
    fn test_buy_fighters() {
        let mut state = GameState::new(1, "Test".to_string());
        let initial_credits = state.credits;
        let initial_fighters = state.fighters;

        let result = buy_fighters(&mut state, 10);
        assert!(result.is_ok());
        assert_eq!(state.fighters, initial_fighters + 10);
        assert_eq!(state.credits, initial_credits - 10 * config::FIGHTER_COST);
    }

    #[test]
    fn test_calculate_profit() {
        let buy_port = PortData {
            name: "Buy Port".to_string(),
            port_type: PortTypeCode::SSS,  // Sells everything
            fuel_ore: PortStock { quantity: 500, max_quantity: 1000, price: 15, base_price: 20 },
            organics: PortStock::new(1000, 25),
            equipment: PortStock::new(1000, 50),
        };

        let sell_port = PortData {
            name: "Sell Port".to_string(),
            port_type: PortTypeCode::BBB,  // Buys everything
            fuel_ore: PortStock { quantity: 500, max_quantity: 1000, price: 25, base_price: 20 },
            organics: PortStock::new(1000, 25),
            equipment: PortStock::new(1000, 50),
        };

        let profit = calculate_profit(&buy_port, &sell_port, Commodity::FuelOre, 10);
        assert!(profit.is_some());
        assert_eq!(profit.unwrap(), 100);  // (25 - 15) * 10
    }
}
