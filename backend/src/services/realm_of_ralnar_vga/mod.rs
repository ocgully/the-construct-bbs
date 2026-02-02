//! Realm of Ralnar VGA - Service Module
//!
//! Graphics-based version using VGA Mode 13h rendering (320x200, 256 colors).
//! This version renders to a framebuffer that can be streamed to a Canvas element.

// Work-in-progress module - suppress warnings for unused code
#![allow(dead_code)]
#![allow(unused)]

pub mod db;
pub mod service;

pub use db::RalnarVgaDb;
pub use service::*;
