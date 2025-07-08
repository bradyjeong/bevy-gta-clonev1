//! Gameplay simulation - physics, AI, rules
#![deny(warnings)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]


// Macro imports
#[macro_use]
extern crate tracing;

// Add core imports (tracing macros available via #[macro_use])
pub(crate) use bevy::prelude::*;
// Note: bevy_rapier3d re-export removed - unused
pub(crate) use game_core::prelude::*;

// Re-export component tree from game_core for internal use
pub use game_core::components;

pub mod services;
pub(crate) mod physics;
pub mod world;
pub mod input;
pub mod distance;
pub(crate) mod lod;
pub(crate) mod vehicles;
pub(crate) mod setup;
pub mod plugins;
pub mod systems;
pub mod factories;
pub mod prelude;
pub mod config;


// Removed compatibility layer - using direct imports
pub use engine_core;
pub use engine_bevy;
pub use game_core;
// Only expose via prelude - no direct re-exports
/// Main plugin for simulation systems
pub struct SimulationPlugin;
impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.insert_resource(systems::distance_cache::DistanceCache::new());
        
        // Core simulation systems  
        app.add_systems(Update, (
            systems::distance_cache::distance_cache_management_system,
            systems::world::performance::performance_monitoring_system,
            systems::world::culling::distance_culling_system,
        ));
    }
}


