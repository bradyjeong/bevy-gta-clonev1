use bevy::prelude::*;
use crate::world::constants::UNIFIED_CHUNK_SIZE;

/// Coordinate system for chunk-based world streaming
/// Maps world positions to discrete chunk indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    /// Create a new chunk coordinate
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    
    /// Convert world position to chunk coordinate
    pub fn from_world_pos(world_pos: Vec3) -> Self {
        Self {
            x: (world_pos.x / UNIFIED_CHUNK_SIZE).floor() as i32,
            z: (world_pos.z / UNIFIED_CHUNK_SIZE).floor() as i32,
        }
    }
    
    /// Get the center position of this chunk in world space
    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * UNIFIED_CHUNK_SIZE + UNIFIED_CHUNK_SIZE * 0.5,
            0.0,
            self.z as f32 * UNIFIED_CHUNK_SIZE + UNIFIED_CHUNK_SIZE * 0.5,
        )
    }
    
    /// Calculate distance between two chunk coordinates (in chunks)
    pub fn distance_to(&self, other: ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }
}
