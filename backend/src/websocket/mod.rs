pub mod protocol;
pub mod session;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
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
/// - Main task: creates session, runs ceremony, receives from WebSocket, routes to session
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

    // Initialize session (sets AwaitingAuth state).
    // The ceremony runs when the frontend sends the auth message.
    let _ = session.on_connect().await;

    // Main receive loop with periodic timeout check
    let mut recv_task = tokio::spawn(async move {
        use tokio::time::{interval, Duration};

        // Check for timeout every 5 seconds (even when idle)
        let mut timeout_check = interval(Duration::from_secs(5));
        timeout_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // Periodic timeout check (handles idle users)
                _ = timeout_check.tick() => {
                    if session.check_and_handle_timeout().await {
                        // Session timed out
                        break;
                    }
                }
                // WebSocket message received
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(axum::extract::ws::Message::Text(text))) => {
                            // Handle user input
                            session.handle_input(&text).await;

                            // Check if session wants to disconnect (line busy, lockout, etc.)
                            if session.is_disconnecting() {
                                break;
                            }
                        }
                        Some(Ok(axum::extract::ws::Message::Close(_))) => {
                            // Client closed connection
                            break;
                        }
                        Some(Ok(axum::extract::ws::Message::Binary(_))) => {
                            // Ignore binary messages (we only expect text input)
                            eprintln!("Warning: Received unexpected binary message from WebSocket client");
                        }
                        Some(Ok(axum::extract::ws::Message::Ping(_))) | Some(Ok(axum::extract::ws::Message::Pong(_))) => {
                            // Handled automatically by axum
                        }
                        Some(Err(e)) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            // Stream ended
                            break;
                        }
                    }
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
