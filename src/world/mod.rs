pub mod chunk_coord;
pub mod chunk_data;
pub mod chunk_tracker;
pub mod placement_grid;
pub mod road_network;
pub mod streaming_state;
pub mod world_coordinator;
pub mod plugins;
pub mod constants;

// Re-export types explicitly to avoid conflicts
pub use chunk_coord::ChunkCoord;
pub use chunk_data::{ChunkData, ChunkState, UnifiedChunkEntity, ContentLayer};
pub use chunk_tracker::{ChunkTracker, ChunkTables, ChunkProgress, ChunkLoadRequest, ChunkUnloadRequest};
pub use placement_grid::{PlacementGrid, GridCell};
pub use road_network::{RoadNetwork, RoadType};
pub use streaming_state::{WorldStreamingState, ChunkStorage, LOD_DISTANCES, calculate_lod_level};
pub use world_coordinator::WorldCoordinator;
pub use plugins::*;
pub use constants::*;
