pub mod registry;
pub mod example;
pub mod login;
pub mod registration;
pub mod welcome_art;

pub use registry::ServiceRegistry;

use thiserror::Error;

/// Trait for all BBS services (email, chat, games, etc.)
pub trait Service: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn on_enter(&self, session: &mut dyn SessionIO) -> Result<(), ServiceError>;
    fn handle_input(&self, session: &mut dyn SessionIO, input: &str) -> Result<ServiceAction, ServiceError>;
    fn on_exit(&self, session: &mut dyn SessionIO);
}

/// Session I/O abstraction for service interaction
pub trait SessionIO {
    fn write(&mut self, data: &str);
    fn writeln(&mut self, data: &str);
    fn queue_paginated(&mut self, data: &str);
}

/// Actions a service can return after handling input
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceAction {
    Continue,
    Exit,
}

/// Errors that can occur in service operations
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service error: {0}")]
    Generic(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
