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
    /// Queens daily puzzle database
    pub(crate) queens_db: Arc<services::queens::db::QueensDb>,
    /// Chess multiplayer database
    pub(crate) chess_db: Arc<services::chess::db::ChessDb>,
    /// Star Trader space trading database
    pub(crate) star_trader_db: Arc<services::star_trader::db::StarTraderDb>,
    /// Master of Cygnus 4X strategy database
    pub(crate) moc_db: Arc<services::master_of_cygnus::db::MocDb>,
    /// Usurper RPG database
    pub(crate) usurper_db: Arc<services::usurper::db::UsurperDb>,
    /// Depths of Diablo roguelite database
    pub(crate) diablo_db: Arc<services::depths_of_diablo::db::DiabloDb>,
    /// Last Dream JRPG database
    pub(crate) last_dream_db: Arc<services::last_dream::db::LastDreamDb>,
    /// Realm of Ralnar JRPG database
    pub(crate) ralnar_db: Arc<services::realm_of_ralnar::db::RalnarDb>,
    /// Kyrandia text adventure database
    pub(crate) kyrandia_db: Arc<services::kyrandia::db::KyrandiaDb>,
    /// Tanks artillery game database
    pub(crate) tanks_db: Arc<services::tanks::db::TanksDb>,
    /// Tanks shared lobby for real-time multiplayer
    pub(crate) tanks_lobby: Arc<Mutex<games::tanks::TanksLobby>>,
    /// Summit climbing game database
    pub(crate) summit_db: Arc<services::summit::db::SummitDb>,
    /// Mineteria sandbox game database
    pub(crate) mineteria_db: Arc<services::mineteria::db::MineteriaDb>,
    /// Fortress colony simulation database
    pub(crate) fortress_db: Arc<services::fortress::db::FortressDb>,
    /// Ultimo MMO RPG database
    pub(crate) ultimo_db: Arc<services::ultimo::db::UltimoDb>,
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

    // Initialize Queens database
    let queens_db = Arc::new(
        services::queens::db::QueensDb::new(
            std::path::Path::new("data/queens.db")
        ).await.expect("Failed to initialize Queens database")
    );
    println!("Queens database initialized");

    // Initialize Chess database
    let chess_db = Arc::new(
        services::chess::db::ChessDb::new(
            std::path::Path::new("data/chess.db")
        ).await.expect("Failed to initialize Chess database")
    );
    println!("Chess database initialized");

    // Initialize Star Trader database
    let star_trader_db = Arc::new(
        services::star_trader::db::StarTraderDb::new(
            std::path::Path::new("data/star_trader.db")
        ).await.expect("Failed to initialize Star Trader database")
    );
    println!("Star Trader database initialized");

    // Initialize Master of Cygnus database
    let moc_db = Arc::new(
        services::master_of_cygnus::db::MocDb::new(
            std::path::Path::new("data/master_of_cygnus.db")
        ).await.expect("Failed to initialize Master of Cygnus database")
    );
    println!("Master of Cygnus database initialized");

    // Initialize Usurper database
    let usurper_db = Arc::new(
        services::usurper::db::UsurperDb::new(
            std::path::Path::new("data/usurper.db")
        ).await.expect("Failed to initialize Usurper database")
    );
    println!("Usurper database initialized");

    // Initialize Depths of Diablo database
    let diablo_db = Arc::new(
        services::depths_of_diablo::db::DiabloDb::new(
            std::path::Path::new("data/depths_of_diablo.db")
        ).await.expect("Failed to initialize Depths of Diablo database")
    );
    println!("Depths of Diablo database initialized");

    // Initialize Last Dream database
    let last_dream_db = Arc::new(
        services::last_dream::db::LastDreamDb::new(
            std::path::Path::new("data/last_dream.db")
        ).await.expect("Failed to initialize Last Dream database")
    );
    println!("Last Dream database initialized");

    // Initialize Realm of Ralnar database
    let ralnar_db = Arc::new(
        services::realm_of_ralnar::db::RalnarDb::new(
            std::path::Path::new("data/realm_of_ralnar.db")
        ).await.expect("Failed to initialize Realm of Ralnar database")
    );
    println!("Realm of Ralnar database initialized");

    // Initialize Kyrandia database
    let kyrandia_db = Arc::new(
        services::kyrandia::db::KyrandiaDb::new(
            std::path::Path::new("data/kyrandia.db")
        ).await.expect("Failed to initialize Kyrandia database")
    );
    println!("Kyrandia database initialized");

    // Initialize Tanks database
    let tanks_db = Arc::new(
        services::tanks::db::TanksDb::new(
            std::path::Path::new("data/tanks.db")
        ).await.expect("Failed to initialize Tanks database")
    );
    println!("Tanks database initialized");

    // Initialize Tanks shared lobby
    let tanks_lobby = Arc::new(Mutex::new(games::tanks::TanksLobby::new()));
    println!("Tanks lobby initialized");

    // Initialize Summit database
    let summit_db = Arc::new(
        services::summit::db::SummitDb::new(
            std::path::Path::new("data/summit.db")
        ).await.expect("Failed to initialize Summit database")
    );
    println!("Summit database initialized");

    // Initialize Mineteria database
    let mineteria_db = Arc::new(
        services::mineteria::db::MineteriaDb::new(
            std::path::Path::new("data/mineteria.db")
        ).await.expect("Failed to initialize Mineteria database")
    );
    println!("Mineteria database initialized");

    // Initialize Fortress database
    let fortress_db = Arc::new(
        services::fortress::db::FortressDb::new(
            std::path::Path::new("data/fortress.db")
        ).await.expect("Failed to initialize Fortress database")
    );
    println!("Fortress database initialized");

    // Initialize Ultimo database
    let ultimo_db = Arc::new(
        services::ultimo::db::UltimoDb::new(
            std::path::Path::new("data/ultimo.db")
        ).await.expect("Failed to initialize Ultimo database")
    );
    println!("Ultimo database initialized");

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
        queens_db,
        chess_db,
        star_trader_db,
        moc_db,
        usurper_db,
        diablo_db,
        last_dream_db,
        ralnar_db,
        kyrandia_db,
        tanks_db,
        tanks_lobby,
        summit_db,
        mineteria_db,
        fortress_db,
        ultimo_db,
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
