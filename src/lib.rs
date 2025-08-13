// =============================================================================
// ARCHITECTURAL LINT RULES & COMPILE-TIME GUARDS
// =============================================================================
//
// These lint rules enforce the architectural decisions documented in 
// architectural_shift.md to maintain code quality and prevent violations.
//
// DENIED PATTERNS (will cause compilation failure):
// - `unsafe_code`: No unsafe blocks allowed - Rust's safety guarantees must be maintained
// - `dead_code`: All code must be actively used or removed
// - `unused_imports`: Keep imports clean and intentional
// - `unused_mut`: Avoid unnecessary mutability
//
// WARNED PATTERNS (will show warnings, future deny):
// - `clippy::expect_used`: Prefer proper error handling over expect()
//
// CI-ENFORCED PATTERNS (caught by GitHub Actions):
// - `thread_local!`: Violates single-threaded architecture
// - `lazy_static!`: Use Bevy Resources instead for shared state
// - `RefCell`: Violates ECS patterns, use Bevy's change detection
// - Cross-plugin imports: Plugins must communicate via events, not direct imports
//
// See .github/workflows/ci.yml for the architecture-guard job that enforces these.
// =============================================================================

#![deny(dead_code, unused_imports, unused_mut)]
#![deny(unsafe_code)]
#![warn(clippy::expect_used)] // Warn for now, will deny after fixing remaining uses

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
pub mod render_components; // Re-exports Bevy's official components (not wrappers)
pub mod system_sets;
pub mod shared; // Shared types to break circular dependencies
pub mod resources;
pub mod events; // Event-driven cross-plugin communication

// World module is always available (needed for tests and core functionality)
pub mod world;

#[cfg(feature = "event-audit")]
pub mod debug;

pub mod observers;

#[cfg(feature = "debug-events")]
pub mod instrumentation;

// Core public API - essential items for external use (reduced from 100+ to ~15)
pub use components::{Player, ActiveEntity, MainCamera, CullingSettings, PerformanceStats};
// Additional exports for size testing
pub use components::player::{HumanMovement, HumanAnimation};
pub use components::control_state::{ControlState, PlayerControlled, AIControlled};
pub use components::lod::VegetationLOD;
pub use components::npc_optimized::{NPCCore, NPCConfig};
pub use world::chunk_tracker::{ChunkTracker, ChunkTables};
pub use game_state::GameState;
pub use config::GameConfig;
pub use plugins::UnifiedWorldPlugin;
pub use setup::setup_basic_world;
pub use constants::*;
pub use render_components::{Mesh3d, MeshMaterial3d}; // Re-export Bevy's official components
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
