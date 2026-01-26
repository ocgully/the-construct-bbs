use super::{Service, ServiceAction, ServiceError, SessionIO};

pub struct ExampleService;

impl Service for ExampleService {
    fn name(&self) -> &str {
        "example"
    }

    fn description(&self) -> &str {
        "Example service for testing"
    }

    fn on_enter(&self, session: &mut dyn SessionIO) -> Result<(), ServiceError> {
        session.writeln("Welcome to the Example Service!");
        session.writeln("");
        session.writeln("Commands:");
        session.writeln("  long  - Display paginated text");
        session.writeln("  quit  - Exit service");
        session.writeln("");
        Ok(())
    }

    fn handle_input(&self, session: &mut dyn SessionIO, input: &str) -> Result<ServiceAction, ServiceError> {
        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("q") {
            return Ok(ServiceAction::Exit);
        }

        if trimmed.eq_ignore_ascii_case("long") {
            // Generate long text for pagination testing
            let mut long_text = String::new();
            long_text.push_str("╔══════════════════════════════════════════════════════════════════════╗\r\n");
            long_text.push_str("║           THE CONSTRUCT BBS - PAGINATION TEST DOCUMENT              ║\r\n");
            long_text.push_str("╚══════════════════════════════════════════════════════════════════════╝\r\n");
            long_text.push_str("\r\n");

            for i in 1..=50 {
                long_text.push_str(&format!("Line {}: This is test content to demonstrate pagination in action.\r\n", i));

                if i % 10 == 0 {
                    long_text.push_str(&format!("\r\n--- Section {} Complete ---\r\n\r\n", i / 10));
                }
            }

            long_text.push_str("\r\n");
            long_text.push_str("╔══════════════════════════════════════════════════════════════════════╗\r\n");
            long_text.push_str("║                         END OF DOCUMENT                              ║\r\n");
            long_text.push_str("╚══════════════════════════════════════════════════════════════════════╝\r\n");

            session.queue_paginated(&long_text);
            return Ok(ServiceAction::Continue);
        }

        session.writeln(&format!("You said: {}", input));
        Ok(ServiceAction::Continue)
    }

    fn on_exit(&self, session: &mut dyn SessionIO) {
        session.writeln("Leaving Example Service...");
    }
}
