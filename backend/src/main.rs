mod auth;
mod config;
mod db;
mod services;
mod terminal;
mod websocket;

use axum::{
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use std::sync::Arc;
use config::Config;
use services::ServiceRegistry;
use sqlx::SqlitePool;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) registry: ServiceRegistry,
    pub(crate) db_pool: SqlitePool,
}

#[tokio::main]
async fn main() {
    // Try loading config from current directory, then parent directory
    let config = Config::load("config.toml")
        .or_else(|_| Config::load("../config.toml"))
        .expect("Failed to load config.toml");

    let host = config.server.host.clone();
    let port = config.server.port;

    // Create service registry from config
    let registry = ServiceRegistry::from_config(&config);
    println!("Loaded {} service(s)", registry.list().len());

    // Initialize database
    let db_pool = db::pool::init_pool("sqlite:bbs.db?mode=rwc")
        .await
        .expect("Failed to initialize database");
    println!("Database initialized");

    let state = Arc::new(AppState {
        config,
        registry,
        db_pool,
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(websocket::ws_handler))
        .fallback_service(ServeDir::new("../frontend/dist"))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("The Construct BBS listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

async fn health_check() -> &'static str {
    "OK"
}
