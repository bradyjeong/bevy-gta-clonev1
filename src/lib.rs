#![deny(dead_code, unused_imports, unused_mut)]

pub mod components;
pub mod config;
pub mod systems;
pub mod plugins;
pub mod setup;
pub mod constants;
pub mod game_state;
pub mod bundles;
pub mod factories;
mod services; // Private - only used internally
pub mod render_primitives;
pub mod system_sets;
pub mod shared; // Shared types to break circular dependencies
pub mod util; // Safe math and utility functions

// Core public API - essential items for external use (reduced from 100+ to 10)
pub use components::{Player, ActiveEntity, MainCamera, CullingSettings, PerformanceStats};
pub use game_state::GameState;
pub use config::GameConfig;
pub use plugins::UnifiedWorldPlugin;
pub use setup::setup_basic_world;
pub use constants::*;
pub use render_primitives::{Mesh3d, MeshMaterial3d};
