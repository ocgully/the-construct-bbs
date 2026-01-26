pub mod protocol;
pub mod session;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use tokio::sync::mpsc;
use std::sync::Arc;

use crate::AppState;
use session::Session;

/// WebSocket upgrade handler
///
/// Called when a client requests a WebSocket connection to /ws.
/// Upgrades the HTTP connection to WebSocket and spawns the connection handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an active WebSocket connection
///
/// Architecture:
/// - Split socket into sender and receiver
/// - Create mpsc channel for session-to-websocket communication
/// - Spawn sender task: forwards messages from channel to WebSocket
/// - Main task: creates session, receives from WebSocket, routes to session
/// - On disconnect: clean up both tasks and session
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create channel for session output
    // Buffer size 32: enough for burst of ANSI art lines without blocking
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Spawn sender task: reads from channel, sends to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                // Client disconnected
                break;
            }
        }
    });

    // Create session for this connection
    let mut session = Session::new(tx.clone(), state);

    // Send welcome screen
    session.on_connect().await;

    // Main receive loop
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.recv().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    // Handle user input
                    session.handle_input(&text).await;
                }
                Ok(axum::extract::ws::Message::Close(_)) => {
                    // Client closed connection
                    break;
                }
                Ok(axum::extract::ws::Message::Binary(_)) => {
                    // Ignore binary messages (we only expect text input)
                    eprintln!("Warning: Received unexpected binary message from WebSocket client");
                }
                Ok(axum::extract::ws::Message::Ping(_)) | Ok(axum::extract::ws::Message::Pong(_)) => {
                    // Handled automatically by axum
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        // Connection closed
        session.on_disconnect().await;
    });

    // Wait for either task to finish (indicates disconnect)
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }
}
