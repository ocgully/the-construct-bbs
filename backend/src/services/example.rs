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
        session.writeln("Type 'quit' to exit.");
        Ok(())
    }

    fn handle_input(&self, session: &mut dyn SessionIO, input: &str) -> Result<ServiceAction, ServiceError> {
        let trimmed = input.trim();

        if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("q") {
            return Ok(ServiceAction::Exit);
        }

        session.writeln(&format!("You said: {}", input));
        Ok(ServiceAction::Continue)
    }

    fn on_exit(&self, session: &mut dyn SessionIO) {
        session.writeln("Leaving Example Service...");
    }
}
