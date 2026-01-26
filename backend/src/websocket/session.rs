use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    AppState,
    services::{Service, ServiceAction, ServiceError, SessionIO},
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
        self.output_buffer.begin_sync();
        self.output_buffer.clear_screen();

        // Title banner
        self.output_buffer.set_fg(Color::LightCyan);
        self.output_buffer.bold();
        self.output_buffer.writeln("╔════════════════════════════════════════════╗");
        self.output_buffer.writeln("║         THE CONSTRUCT BBS                  ║");
        self.output_buffer.writeln("╚════════════════════════════════════════════╝");
        self.output_buffer.reset_color();

        self.output_buffer.writeln("");

        self.output_buffer.set_fg(Color::LightGray);
        self.output_buffer.writeln("Terminal Foundation v0.1");
        self.output_buffer.writeln("");

        // List available services
        let services = self.state.registry.list();
        if services.is_empty() {
            self.output_buffer.set_fg(Color::Yellow);
            self.output_buffer.writeln("No services available.");
        } else {
            self.output_buffer.set_fg(Color::Yellow);
            self.output_buffer.writeln("Available Services:");
            self.output_buffer.writeln("");

            for (idx, (name, description)) in services.iter().enumerate() {
                self.output_buffer.set_fg(Color::LightGreen);
                self.output_buffer.write_str(&format!("  [{}] ", idx + 1));
                self.output_buffer.set_fg(Color::White);
                self.output_buffer.write_str(name);
                self.output_buffer.set_fg(Color::LightGray);
                self.output_buffer.writeln(&format!(" - {}", description));
            }

            self.output_buffer.writeln("");
        }

        // Show prompt
        self.output_buffer.set_fg(Color::LightCyan);
        self.output_buffer.write_str("Enter service number or 'quit' to disconnect: ");
        self.output_buffer.reset_color();

        self.output_buffer.end_sync();

        self.flush_output().await;
    }

    /// Show the main menu again
    async fn show_main_menu(&mut self) {
        self.output_buffer.begin_sync();
        self.output_buffer.clear_screen();

        // Title
        self.output_buffer.set_fg(Color::LightCyan);
        self.output_buffer.writeln("╔═══════════════════════════════════════════╗");
        self.output_buffer.writeln("║         THE CONSTRUCT BBS                 ║");
        self.output_buffer.writeln("╚═══════════════════════════════════════════╝");
        self.output_buffer.reset_color();
        self.output_buffer.writeln("");

        // List services
        let services = self.state.registry.list();
        if !services.is_empty() {
            self.output_buffer.set_fg(Color::Yellow);
            self.output_buffer.writeln("Available Services:");
            self.output_buffer.writeln("");

            for (idx, (name, description)) in services.iter().enumerate() {
                self.output_buffer.set_fg(Color::LightGreen);
                self.output_buffer.write_str(&format!("  [{}] ", idx + 1));
                self.output_buffer.set_fg(Color::White);
                self.output_buffer.write_str(name);
                self.output_buffer.set_fg(Color::LightGray);
                self.output_buffer.writeln(&format!(" - {}", description));
            }

            self.output_buffer.writeln("");
        }

        // Prompt
        self.output_buffer.set_fg(Color::LightCyan);
        self.output_buffer.write_str("Enter service number or 'quit' to disconnect: ");
        self.output_buffer.reset_color();

        self.output_buffer.end_sync();

        self.flush_output().await;
    }

    /// Handle user input
    pub async fn handle_input(&mut self, input: &str) {
        let trimmed = input.trim();

        if let Some(service_name) = &self.current_service {
            // Currently in a service - route input to it
            if let Some(service) = self.state.registry.get(service_name) {
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
            let services = self.state.registry.list();

            // Try number first
            if let Ok(num) = trimmed.parse::<usize>() {
                if num > 0 && num <= services.len() {
                    let (service_name, _) = services[num - 1];
                    self.enter_service(service_name).await;
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
            self.output_buffer.write_str("Enter service number or 'quit' to disconnect: ");
            self.output_buffer.reset_color();
            self.flush_output().await;
        }
    }

    /// Enter a service (with authentic BBS "loading door" delay)
    async fn enter_service(&mut self, service_name: &str) {
        if let Some(service) = self.state.registry.get(service_name) {
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
            if let Some(service) = self.state.registry.get(service_name) {
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
