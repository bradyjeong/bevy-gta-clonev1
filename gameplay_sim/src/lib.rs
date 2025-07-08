//! Gameplay simulation - physics, AI, rules
#![warn(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]

// Macro imports
#[macro_use]
extern crate tracing;

// Add core imports (tracing macros available via #[macro_use])
pub(crate) use bevy::prelude::*;
pub(crate) use bevy_rapier3d::prelude::*;
pub(crate) use game_core::prelude::*;

// Re-export component tree from game_core for internal use
pub use game_core::components;

pub(crate) mod services;
pub(crate) mod physics;
pub(crate) mod movement;
pub mod world;
pub(crate) mod behavior;
pub(crate) mod input;
pub mod distance;
pub(crate) mod lod;
pub(crate) mod vehicles;
pub(crate) mod setup;
pub(crate) mod plugins;
pub(crate) mod entity_creation;
pub(crate) mod spawn_validation;
pub(crate) mod transform_sync;
pub(crate) mod water;
pub(crate) mod bevy16_compat;
pub(crate) mod systems;
pub(crate) mod factories;
pub mod prelude;
pub(crate) mod config;
pub(crate) mod constants;
pub(crate) mod compat;

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
        
        // Movement systems
        app.add_systems(Update, (
            systems::movement::player::human_player_movement,
            systems::movement::player::human_player_animation,
            systems::movement::supercar_input::supercar_input_system,
            systems::movement::supercar_physics::supercar_physics_system,
            systems::movement::supercar_effects::supercar_effects_system,
            systems::movement::realistic_vehicle_input::realistic_vehicle_input_system,
            systems::movement::realistic_vehicle_physics::realistic_vehicle_physics_system,
            systems::movement::realistic_vehicle_physics_core::realistic_vehicle_physics_core_system,
            systems::movement::aircraft::helicopter_movement,
            systems::movement::aircraft::f16_movement,
            systems::movement::aircraft::rotate_helicopter_rotors,
            systems::movement::vehicles::car_movement,
        ))
        .add_systems(Update, (
            // Physics systems
            systems::player_collision_resolution::player_collision_resolution_system,
            systems::player_collision_resolution::player_movement_validation_system,
        // Core simulation systems  
            systems::human_behavior::human_emotional_state_system,
            systems::interaction::interaction_system,
            systems::spawn_validation::cleanup_despawned_entities,
            systems::distance_cache::distance_cache_management_system,
            systems::world::performance::performance_monitoring_system,
            systems::world::culling::distance_culling_system,
        ));
    }
}
