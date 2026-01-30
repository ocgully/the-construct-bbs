//! Grand Theft Meth - Drug Wars clone
//!
//! A commodities trading game where players buy/sell substances
//! across cities, manage debt, encounter random events, and
//! complete a 15-step story quest.

pub mod data;
pub mod economy;
pub mod events;
pub mod quest;
pub mod render;
pub mod screen;
pub mod state;

// Re-export types actually used externally
pub use state::{GameState, DeliveryQuest, PendingTransaction};
pub use data::{CITIES, COMMODITIES, WEAPONS, GANGS, get_city, get_borough, get_commodity, get_weapon, get_gang, get_shop_inventory};
pub use screen::{GameScreen, TradeMode, EnemyType, GameEvent, GtmAction, GtmFlow};
pub use render::format_money;
