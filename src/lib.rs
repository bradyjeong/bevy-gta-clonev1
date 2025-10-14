#![deny(dead_code, unused_imports, unused_mut)]

pub mod bundles;
pub mod components;
pub mod config;
pub mod constants;
pub mod factories;
pub mod game_state;
pub mod plugins;
pub mod render_primitives;
pub mod resources;
mod services; // Private - only used internally
pub mod setup;
pub mod states;
pub mod system_sets;
pub mod systems;
pub mod util; // Safe math and utility functions

#[cfg(test)]
mod tests;

// Core public API - essential items for external use (reduced from 100+ to 10)
pub use components::{ActiveEntity, CullingSettings, MainCamera, PerformanceStats, Player};
pub use config::GameConfig;
pub use constants::{CHARACTER_GROUP, STATIC_GROUP, VEHICLE_GROUP};
pub use game_state::GameState;
pub use plugins::UnifiedWorldPlugin;
pub use render_primitives::{Mesh3d, MeshMaterial3d};
pub use setup::setup_basic_world;
