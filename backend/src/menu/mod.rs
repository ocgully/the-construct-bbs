pub mod config;
pub mod quotes;
pub mod state;
pub mod render;

pub use config::{MenuItem, MenuConfig};
pub use quotes::random_stoic_quote;
pub use state::{MenuState, MenuAction, MenuSession};
pub use render::{render_main_menu, render_submenu, render_help, BorderStyle};
