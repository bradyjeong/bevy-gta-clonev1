pub mod npc;
// OLD MODULES REMOVED - using unified system
pub mod dynamic_content;
pub mod performance;
pub mod road_network;
pub mod road_mesh;
pub mod road_generation;
pub mod debug;

// NEW UNIFIED WORLD SYSTEM
pub mod unified_world;
pub mod layered_generation;
pub mod npc_spawn;
pub mod unified_distance_culling;
pub mod unified_factory_setup;

pub use npc::*;
pub use dynamic_content::*;
pub use performance::*;
pub use road_network::*;
pub use road_mesh::*;
pub use road_generation::*;
pub use debug::*;

// Export unified system components
pub use unified_world::*;
pub use layered_generation::*;
pub use npc_spawn::*;
// Export unified distance culling components (selective to avoid conflicts)
pub use unified_distance_culling::{
    UnifiedCullable, UnifiedDistanceCullingPlugin, new_unified_distance_culling_system,
    ChunkUnloadRequest // Legacy LODUpdate components removed
};
pub use unified_factory_setup::*;
