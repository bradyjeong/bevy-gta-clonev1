use bevy::prelude::*;
use std::collections::HashMap;
use crate::world::chunk_coord::ChunkCoord;
use crate::world::chunk_data::ChunkData;

/// LOD transition distances - optimized for 60+ FPS target
pub const LOD_DISTANCES: [f32; 3] = [150.0, 300.0, 500.0];

/// Hot-path streaming state - frequently accessed (≤64 bytes)
#[derive(Resource)]
pub struct WorldStreamingState {
    pub active_chunk: Option<ChunkCoord>,  // 9 bytes (Option<8-byte struct>)
    pub streaming_radius_chunks: i32,     // 4 bytes
    pub last_update: f32,                 // 4 bytes
    pub chunks_loaded_this_frame: u16,    // 2 bytes - reduced from usize
    pub chunks_unloaded_this_frame: u16,  // 2 bytes - reduced from usize
    pub max_chunks_per_frame: u8,         // 1 byte - max 255 is plenty
    // Total: ~22 bytes (well under 64)
}

impl Default for WorldStreamingState {
    fn default() -> Self {
        use crate::world::constants::{UNIFIED_CHUNK_SIZE, UNIFIED_STREAMING_RADIUS};
        Self {
            active_chunk: None,
            streaming_radius_chunks: (UNIFIED_STREAMING_RADIUS as f32 / UNIFIED_CHUNK_SIZE).ceil() as i32,
            last_update: 0.0,
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            max_chunks_per_frame: 4, // Prevent frame drops
        }
    }
}

// Compile-time size assertion for hot-path resource
const _: () = assert!(
    std::mem::size_of::<WorldStreamingState>() <= 64,
    "WorldStreamingState exceeds 64-byte cache line"
);

/// Cold-path chunk storage - large but infrequently accessed
#[derive(Resource)]
pub struct ChunkStorage {
    pub chunks: HashMap<ChunkCoord, ChunkData>,
}

impl Default for ChunkStorage {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}

/// Calculate LOD level based on distance
pub fn calculate_lod_level(distance: f32) -> usize {
    for (i, &max_distance) in LOD_DISTANCES.iter().enumerate() {
        if distance <= max_distance {
            return i;
        }
    }
    LOD_DISTANCES.len() - 1
}
