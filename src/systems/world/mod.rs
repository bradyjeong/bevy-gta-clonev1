pub mod npc;
pub mod culling;
pub mod performance;
pub mod road_network;
pub mod road_mesh;
pub mod road_generation;
pub mod debug;


// NEW UNIFIED WORLD SYSTEM
pub mod unified_world;
pub mod layered_generation;
pub mod simulation_lod;
pub mod npc_spawn;
// pub mod optimized_lod; // Removed - functionality moved to unified_lod.rs
pub mod vegetation_lod;
// pub mod unified_distance_culling; - REMOVED: Replaced with Bevy's VisibilityRange
pub mod boundaries;
pub mod boundary_effects;
pub mod unified_factory_setup;

pub use npc::*;
pub use culling::*;
pub use performance::*;
pub use road_network::*;
pub use road_mesh::*;
pub use road_generation::*;
pub use debug::*;


// Export unified system components
pub use unified_world::*;
pub use layered_generation::*;
pub use simulation_lod::*;
pub use npc_spawn::*;
// pub use optimized_lod::*; // Removed - functionality moved to unified_lod.rs
pub use vegetation_lod::*;
// UnifiedCullable exports removed - using Bevy's VisibilityRange instead
pub use boundaries::*;
pub use boundary_effects::*;
pub use unified_factory_setup::*;
