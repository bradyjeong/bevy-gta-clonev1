//! Core game components, config, and shared types
#![deny(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;

pub mod components;
pub mod config;
pub mod constants;
pub mod bundles;
pub mod game_state;
pub mod prelude;

// Core components that form the public API
pub use components::{Player, ActiveEntity, MainCamera, LodLevel};
pub use components::{Car, SuperCar, Helicopter, F16, NPC, Boat};
pub use components::{PerformanceStats, CullingSettings};
pub use game_state::GameState;
