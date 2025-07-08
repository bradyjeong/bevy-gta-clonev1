//! Game binary - main entry point and top-level orchestration
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod systems;
pub(crate) mod plugins;
pub(crate) mod setup;
pub(crate) mod constants;
pub(crate) mod game_state;
pub(crate) mod bundles;
pub(crate) mod factories;
pub(crate) mod services;

// Engine-level abstractions only
pub use game_state::GameState;
pub use plugins::GamePlugin;
pub use setup::setup_basic_world;

// Core components that form the public API
pub use components::{Player, ActiveEntity, MainCamera, LodLevel};
pub use components::{Car, SuperCar, Helicopter, F16, NPC, Boat};
pub use components::{PerformanceStats, CullingSettings};
