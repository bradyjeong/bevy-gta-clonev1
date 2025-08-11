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
pub mod services;

#[cfg(test)]
mod tests; // Private - only used internally
pub mod render_primitives;
pub mod system_sets;
pub mod shared; // Shared types to break circular dependencies
pub mod resources;
pub mod events; // Event-driven cross-plugin communication

#[cfg(any(feature = "p1_1_decomp", feature = "world_v2"))]
pub mod world;

// Core public API - essential items for external use (reduced from 100+ to ~15)
pub use components::{Player, ActiveEntity, MainCamera, CullingSettings, PerformanceStats};
pub use game_state::GameState;
pub use config::GameConfig;
pub use plugins::UnifiedWorldPlugin;
pub use setup::setup_basic_world;
pub use constants::*;
pub use render_primitives::{Mesh3d, MeshMaterial3d};
pub use resources::GlobalRng;
pub use systems::input::{InputAction, VehicleControlConfig, VehicleType, ControlCategory};

// Essential bundles for external entity creation
pub use bundles::{VehicleBundle, PhysicsBundle, VisibleBundle, VisibleChildBundle};

// Essential components for external integration  
pub use components::world::{MeshCache, EntityLimits};
pub use components::vehicles::VehicleState;
pub use components::instanced_vegetation::{VegetationInstancingConfig, VegetationType, InstancedVegetationBundle, VegetationBatchable};
pub use components::world::Cullable;
pub use components::dirty_flags::FrameCounter;

// Core services for external utilities
pub use services::{DistanceCache, MovementTracker};

// Vegetation systems for demo integration
pub use systems::vegetation_instancing_integration::{spawn_test_vegetation_system, integrate_vegetation_with_instancing_system};
pub use systems::batching::frame_counter_system;
pub use systems::rendering::vegetation_instancing::{mark_vegetation_instancing_dirty_system, collect_vegetation_instances_system, update_vegetation_instancing_system, animate_vegetation_instances_system, vegetation_instancing_metrics_system};

// World generation events for cross-plugin coordination
pub use events::world::*;

// Essential culling plugin and components for examples
pub use systems::world::unified_distance_culling::{UnifiedDistanceCullingPlugin, UnifiedCullable};
