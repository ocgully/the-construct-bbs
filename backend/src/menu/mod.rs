pub mod config;
pub mod quotes;
pub mod state;

pub use config::{MenuItem, MenuConfig};
pub use quotes::random_stoic_quote;
pub use state::{MenuState, MenuAction, MenuSession};
