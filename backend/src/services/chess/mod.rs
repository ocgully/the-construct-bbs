//! Chess - Door Game Service
//!
//! Async multiplayer chess with ELO ratings, matchmaking, and notifications.
//!
//! Uses __chess__ sentinel for session routing.
//! Game state persisted in chess.db.

pub mod db;
pub mod service;

