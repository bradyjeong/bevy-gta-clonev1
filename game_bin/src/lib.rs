//! Game binary - main entry point and top-level orchestration
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

// Macro imports
#[macro_use]
extern crate tracing;

// Re-export simulation components, config, etc. via compat layer
pub use gameplay_sim::components;
pub use gameplay_sim::{config, factories, services};

// Game binary specific modules
pub(crate) mod systems;
pub(crate) mod plugins;
pub(crate) mod setup;
pub(crate) mod constants;
pub(crate) mod game_state;

// Engine-level abstractions only
pub use game_state::GameState;
pub use plugins::GamePlugin;
pub use setup::setup_basic_world;

// Core components that form the public API
pub use components::{Player, ActiveEntity, MainCamera, LodLevel};
pub use components::{Car, SuperCar, Helicopter, F16, NPC, Boat};
pub use components::{PerformanceStats, CullingSettings};
