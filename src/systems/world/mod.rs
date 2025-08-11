pub mod npc;
pub mod culling;
pub mod dynamic_content;
pub mod performance;
pub mod road_network;
pub mod road_mesh;
pub mod road_generation;
pub mod debug;
pub mod map_system;

// NEW UNIFIED WORLD SYSTEM
pub mod unified_world;
pub mod layered_generation;
pub mod unified_lod;
pub mod npc_lod;
pub mod npc_spawn;
// pub mod optimized_lod; // Removed - functionality moved to unified_lod.rs
pub mod vegetation_lod;
pub mod unified_distance_culling;
pub mod unified_factory_setup;

// EVENT HANDLERS (Phase 3 - Decoupled Architecture)
pub mod event_handlers;

pub use npc::*;
pub use culling::*;
pub use dynamic_content::*;
pub use performance::*;
pub use road_network::*;
pub use road_mesh::*;
pub use road_generation::*;
pub use debug::*;
pub use map_system::*;

// Export unified system components
pub use unified_world::*;
pub use layered_generation::*;

// Export V2 streaming systems (now always enabled)
pub mod streaming_v2;
pub use unified_lod::*;
pub use npc_lod::*;
pub use npc_spawn::*;
// pub use optimized_lod::*; // Removed - functionality moved to unified_lod.rs
pub use vegetation_lod::*;
// Export unified distance culling components (selective to avoid conflicts)
pub use unified_distance_culling::{
    UnifiedCullable, UnifiedDistanceCullingPlugin, new_unified_distance_culling_system,
    VehicleLODUpdate, NPCLODUpdate, VegetationLODUpdate, ChunkLODUpdate, ChunkUnloadRequest
};
pub use unified_factory_setup::*;
pub use event_handlers::*;
