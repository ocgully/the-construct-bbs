pub mod db;
pub mod service;

pub use db::GtmDb;
pub use service::{
    SENTINEL, start_game, save_game_state, render_screen,
    record_game_completion, get_game_leaderboard
};
