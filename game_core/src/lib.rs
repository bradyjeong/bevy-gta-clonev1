//! Core game components, config, and shared types
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;

#[allow(missing_docs)]
pub(crate) mod components;
#[allow(missing_docs)]
pub(crate) mod config;
#[allow(missing_docs)]
pub(crate) mod constants;
#[allow(missing_docs)]
pub(crate) mod bundles;
#[allow(missing_docs)]
pub(crate) mod game_state;
#[allow(missing_docs)]
pub(crate) mod persistence;
#[allow(missing_docs)]
pub(crate) mod services;
#[allow(missing_docs)]
pub(crate) mod schedule;
pub mod prelude;

// Only expose via prelude - no direct re-exports
