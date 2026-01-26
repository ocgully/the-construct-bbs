use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    AppState,
    services::{ServiceAction, SessionIO, welcome_art},
    terminal::{AnsiWriter, Color},
};

/// Per-connection session state
///
/// Each WebSocket connection gets its own Session that:
/// - Manages service routing (main menu vs active service)
/// - Implements SessionIO for service output
/// - Composes ANSI-formatted output using AnsiWriter
/// - Sends output to WebSocket via mpsc channel
pub struct Session {
    tx: mpsc::Sender<String>,
    state: Arc<AppState>,
    current_service: Option<String>,
    output_buffer: AnsiWriter,
}

impl Session {
    /// Create a new session
    pub fn new(tx: mpsc::Sender<String>, state: Arc<AppState>) -> Self {
        Self {
            tx,
            state,
            current_service: None,
            output_buffer: AnsiWriter::new(),
        }
    }

    /// Send the buffered output to the WebSocket
    async fn flush_output(&mut self) {
        if !self.output_buffer.is_empty() {
            let content = self.output_buffer.flush();
            // Ignore send errors (client disconnected)
            let _ = self.tx.send(content).await;
        }
    }

    /// Called when client connects - send welcome screen
    pub async fn on_connect(&mut self) {
        let services: Vec<(String, String)> = self.state.registry.list()
            .iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect();
        let welcome = welcome_art::render_welcome(&services);
        let _ = self.tx.send(welcome).await;
    }

    /// Show the main menu again
    async fn show_main_menu(&mut self) {
        let services: Vec<(String, String)> = self.state.registry.list()
            .iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect();
        let menu = welcome_art::render_main_menu(&services);
        let _ = self.tx.send(menu).await;
    }

    /// Handle user input
    pub async fn handle_input(&mut self, input: &str) {
        let trimmed = input.trim();

        // Ignore empty input and escape sequences (e.g. from mouse scroll)
        if trimmed.is_empty() || trimmed.starts_with('\x1b') {
            return;
        }

        if let Some(service_name) = &self.current_service {
            // Currently in a service - route input to it
            let service = self.state.registry.get(service_name).cloned();
            if let Some(service) = service {
                match service.handle_input(self, trimmed) {
                    Ok(ServiceAction::Continue) => {
                        // Service handled input, continue
                    }
                    Ok(ServiceAction::Exit) => {
                        // Service wants to exit
                        service.on_exit(self);
                        self.current_service = None;
                        self.flush_output().await;

                        // Return to main menu
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        self.show_main_menu().await;
                    }
                    Err(e) => {
                        // Service error
                        self.output_buffer.set_fg(Color::LightRed);
                        self.output_buffer.writeln(&format!("Error: {}", e));
                        self.output_buffer.reset_color();
                        self.flush_output().await;
                    }
                }
            } else {
                // Service disappeared from registry
                self.output_buffer.set_fg(Color::LightRed);
                self.output_buffer.writeln("Error: Service no longer available");
                self.output_buffer.reset_color();
                self.current_service = None;
                self.flush_output().await;
                self.show_main_menu().await;
            }
        } else {
            // At main menu - handle service selection or quit
            if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("q") {
                // User wants to disconnect
                self.output_buffer.set_fg(Color::Yellow);
                self.output_buffer.writeln("");
                self.output_buffer.writeln("Disconnecting from The Construct BBS...");
                self.output_buffer.writeln("Come back soon!");
                self.output_buffer.reset_color();
                self.flush_output().await;

                // Note: Actual disconnection happens when this function returns
                // and the WebSocket connection is closed by the handler
                return;
            }

            // Try to parse as service number or name
            let services: Vec<(String, String)> = self.state.registry.list()
                .iter()
                .map(|(n, d)| (n.to_string(), d.to_string()))
                .collect();

            // Try number first
            if let Ok(num) = trimmed.parse::<usize>() {
                if num > 0 && num <= services.len() {
                    let service_name = services[num - 1].0.clone();
                    self.enter_service(&service_name).await;
                    return;
                }
            }

            // Try name match
            for (service_name, _) in &services {
                if service_name.eq_ignore_ascii_case(trimmed) {
                    self.enter_service(service_name).await;
                    return;
                }
            }

            // Invalid selection
            self.output_buffer.set_fg(Color::LightRed);
            self.output_buffer.writeln(&format!("Unknown command: {}", trimmed));
            self.output_buffer.set_fg(Color::LightCyan);
            self.output_buffer.write_str("Enter service number or (q)uit to disconnect: ");
            self.output_buffer.reset_color();
            self.flush_output().await;
        }
    }

    /// Enter a service (with authentic BBS "loading door" delay)
    async fn enter_service(&mut self, service_name: &str) {
        let service = self.state.registry.get(service_name).cloned();
        if let Some(service) = service {
            // Show "Entering door..." message
            self.output_buffer.writeln("");
            self.output_buffer.set_fg(Color::Yellow);
            self.output_buffer.bold();
            self.output_buffer.writeln(&format!("Entering {}...", service.name()));
            self.output_buffer.reset_color();
            self.flush_output().await;

            // Authentic BBS loading delay
            tokio::time::sleep(Duration::from_millis(800)).await;

            // Clear screen and enter service
            self.output_buffer.clear_screen();

            // Call service on_enter
            if let Err(e) = service.on_enter(self) {
                self.output_buffer.set_fg(Color::LightRed);
                self.output_buffer.writeln(&format!("Error entering service: {}", e));
                self.output_buffer.reset_color();
                self.flush_output().await;

                tokio::time::sleep(Duration::from_millis(1500)).await;
                self.show_main_menu().await;
                return;
            }

            // Mark as current service
            self.current_service = Some(service_name.to_string());
            self.flush_output().await;
        } else {
            self.output_buffer.set_fg(Color::LightRed);
            self.output_buffer.writeln(&format!("Service '{}' not found", service_name));
            self.output_buffer.reset_color();
            self.flush_output().await;
        }
    }

    /// Called when client disconnects
    pub async fn on_disconnect(&mut self) {
        // If in a service, call on_exit
        if let Some(service_name) = &self.current_service {
            let service = self.state.registry.get(service_name).cloned();
            if let Some(service) = service {
                service.on_exit(self);
                self.flush_output().await;
            }
        }

        println!("Session disconnected");
    }
}

/// Implement SessionIO trait for Service interaction
impl SessionIO for Session {
    fn write(&mut self, data: &str) {
        self.output_buffer.write_str(data);
    }

    fn writeln(&mut self, data: &str) {
        self.output_buffer.writeln(data);
    }
}
