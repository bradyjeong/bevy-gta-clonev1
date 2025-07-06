#![deny(clippy::all, clippy::pedantic)]

pub mod components;
pub mod config;
pub mod systems;
pub mod plugins;
pub mod setup;
pub mod constants;
pub mod game_state;
pub mod bundles;
pub mod factories;
pub mod services;

// Engine-level abstractions only
pub use game_state::GameState;
pub use plugins::GamePlugin;
pub use setup::setup_basic_world;

// Core components that form the public API
pub use components::{Player, ActiveEntity, MainCamera, LodLevel};
pub use components::{Car, SuperCar, Helicopter, F16, NPC, Boat};
pub use components::{PerformanceStats, CullingSettings};
