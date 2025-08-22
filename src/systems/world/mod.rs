pub mod culling;
pub mod debug;
pub mod npc;
pub mod performance;
pub mod road_generation;
pub mod road_mesh;
pub mod road_network;

// NEW UNIFIED WORLD SYSTEM
pub mod async_chunk_generation;
pub mod layered_generation;
pub mod npc_spawn;
pub mod simulation_lod;
pub mod unified_world;
// pub mod optimized_lod; // Removed - functionality moved to unified_lod.rs
pub mod vegetation_lod;
// pub mod unified_distance_culling; - REMOVED: Replaced with Bevy's VisibilityRange
pub mod boundaries;
pub mod boundary_effects;
pub mod floating_origin;
pub mod unified_factory_setup;

pub use culling::*;
pub use debug::*;
pub use npc::*;
pub use performance::*;
pub use road_generation::*;
pub use road_mesh::*;
pub use road_network::*;

// Export unified system components
pub use async_chunk_generation::*;
pub use layered_generation::*;
pub use npc_spawn::*;
pub use simulation_lod::*;
pub use unified_world::*;
// pub use optimized_lod::*; // Removed - functionality moved to unified_lod.rs
pub use vegetation_lod::*;
// UnifiedCullable exports removed - using Bevy's VisibilityRange instead
pub use boundaries::*;
pub use boundary_effects::*;
pub use floating_origin::*;
pub use unified_factory_setup::*;
