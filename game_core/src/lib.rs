//! Core game components, config, and shared types
#![deny(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;

pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod constants;
pub(crate) mod bundles;
pub(crate) mod game_state;
pub mod prelude;

// Only expose via prelude - no direct re-exports
