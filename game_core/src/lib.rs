//! Core game components, config, and shared types
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]


pub use engine_core;
pub use engine_bevy;

// Public API modules - canonical namespace design
#[allow(missing_docs)]
pub mod components;
#[allow(missing_docs)]
pub mod config;
#[allow(missing_docs)]
pub mod world;
#[allow(missing_docs)]
pub mod spatial;
#[allow(missing_docs)]
pub mod constants;
#[allow(missing_docs)]
pub mod bundles;
#[allow(missing_docs)]
pub mod game_state;
#[allow(missing_docs)]
pub mod persistence;
#[allow(missing_docs)]
pub mod services;
#[allow(missing_docs)]
pub mod schedule;
pub mod prelude;

// Temporary compatibility layer (will be removed when migration complete)
#[allow(missing_docs)]
pub mod compat;


// Only expose via prelude - no direct re-exports
