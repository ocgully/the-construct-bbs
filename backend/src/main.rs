mod auth;
mod config;
mod connection;
mod db;
mod games;
mod menu;
mod services;
mod terminal;
mod websocket;

use axum::{
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::Mutex;
use config::Config;
use connection::{ChatManager, NodeManager};
use services::ServiceRegistry;
use sqlx::SqlitePool;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) registry: ServiceRegistry,
    pub(crate) db_pool: SqlitePool,
    pub(crate) node_manager: NodeManager,
    pub(crate) chat_manager: ChatManager,
    /// Grand Theft Meth game database (self-contained)
    pub(crate) gtm_db: Arc<services::grand_theft_meth::GtmDb>,
    /// Sudoku game database (self-contained)
    pub(crate) sudoku_db: Arc<services::sudoku::SudokuDb>,
    /// Memory Garden database (social journaling feature)
    pub(crate) memory_garden_db: Arc<services::memory_garden::MemoryGardenDb>,
    /// Dragon Slayer game database (Legend of the Red Dragon style RPG)
    pub(crate) dragon_slayer_db: Arc<services::dragon_slayer::DragonSlayerDb>,
    /// Acromania game database (multiplayer acronym party game)
    pub(crate) acro_db: Arc<services::acromania::AcroDb>,
    /// Acromania shared lobby for real-time multiplayer
    pub(crate) acro_lobby: Arc<Mutex<games::acromania::AcroLobby>>,
    /// Dystopia kingdom management game database
    pub(crate) dystopia_db: Arc<services::dystopia::DystopiaDb>,
    /// Cradle infinite progression game database
    pub(crate) cradle_db: Arc<services::cradle::db::CradleDb>,
    /// Xodia LLM-powered MUD database
    pub(crate) xodia_db: Arc<services::xodia::XodiaDb>,
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

    // Initialize node manager for connection scarcity
    let node_manager = NodeManager::new(config.connection.max_nodes as usize);
    println!("Node capacity: {} nodes", config.connection.max_nodes);

    // Initialize chat manager for real-time messaging
    let chat_manager = ChatManager::new(config.chat.capacity);
    println!("Chat capacity: {} users", config.chat.capacity);

    // Initialize game database (creates grand_theft_meth.db if needed)
    let gtm_db = Arc::new(
        services::grand_theft_meth::GtmDb::new(
            std::path::Path::new("data/grand_theft_meth.db")
        ).await.expect("Failed to initialize GTM database")
    );
    println!("Grand Theft Meth database initialized");

    // Initialize Sudoku database
    let sudoku_db = Arc::new(
        services::sudoku::SudokuDb::new(
            std::path::Path::new("data/sudoku.db")
        ).await.expect("Failed to initialize Sudoku database")
    );
    println!("Sudoku database initialized");

    // Initialize Memory Garden database
    let memory_garden_db = Arc::new(
        services::memory_garden::MemoryGardenDb::new(
            std::path::Path::new("data/memory_garden.db")
        ).await.expect("Failed to initialize Memory Garden database")
    );
    println!("Memory Garden database initialized");

    // Initialize Dragon Slayer database
    let dragon_slayer_db = Arc::new(
        services::dragon_slayer::DragonSlayerDb::new(
            std::path::Path::new("data/dragon_slayer.db")
        ).await.expect("Failed to initialize Dragon Slayer database")
    );
    println!("Dragon Slayer database initialized");

    // Initialize Acromania database
    let acro_db = Arc::new(
        services::acromania::AcroDb::new(
            std::path::Path::new("data/acromania.db")
        ).await.expect("Failed to initialize Acromania database")
    );
    println!("Acromania database initialized");

    // Initialize Acromania shared lobby
    let acro_lobby = Arc::new(Mutex::new(games::acromania::AcroLobby::new()));
    println!("Acromania lobby initialized");

    // Initialize Dystopia database
    let dystopia_db = Arc::new(
        services::dystopia::DystopiaDb::new(
            std::path::Path::new("data/dystopia.db")
        ).await.expect("Failed to initialize Dystopia database")
    );
    println!("Dystopia database initialized");

    // Initialize Cradle database
    let cradle_db = Arc::new(
        services::cradle::db::CradleDb::new(
            std::path::Path::new("data/cradle.db")
        ).await.expect("Failed to initialize Cradle database")
    );
    println!("Cradle database initialized");

    // Initialize Xodia database
    let xodia_db = Arc::new(
        services::xodia::XodiaDb::new(
            std::path::Path::new("data/xodia.db")
        ).await.expect("Failed to initialize Xodia database")
    );
    println!("Xodia database initialized");

    let state = Arc::new(AppState {
        config,
        registry,
        db_pool,
        node_manager,
        chat_manager,
        gtm_db,
        sudoku_db,
        memory_garden_db,
        dragon_slayer_db,
        acro_db,
        acro_lobby,
        dystopia_db,
        cradle_db,
        xodia_db,
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
