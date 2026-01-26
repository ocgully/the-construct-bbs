mod config;
mod services;
mod terminal;

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use config::Config;
use services::ServiceRegistry;

#[derive(Clone)]
struct AppState {
    config: Config,
    registry: ServiceRegistry,
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

    let state = Arc::new(AppState {
        config,
        registry,
    });

    let app = Router::new()
        .route("/health", get(health_check))
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
