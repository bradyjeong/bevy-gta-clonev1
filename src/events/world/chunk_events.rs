//! Chunk loading and unloading coordination events
//! 
//! These events decouple the world streaming system from chunk management,
//! allowing multiple systems to react to chunk state changes.

use bevy::prelude::*;

/// 2D chunk coordinate for world grid (8 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    
    pub fn from_world_pos(pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (pos.x / chunk_size).floor() as i32,
            z: (pos.z / chunk_size).floor() as i32,
        }
    }
}

/// Request to load a specific chunk (8 bytes)
/// Sent by: world streaming system
/// Handled by: chunk loading system
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestChunkLoad {
    pub coord: ChunkCoord,
}

impl RequestChunkLoad {
    pub fn new(coord: ChunkCoord) -> Self {
        Self { coord }
    }
}

/// Notification that a chunk has been successfully loaded (16 bytes)
/// Sent by: chunk loading system  
/// Handled by: dynamic content spawning, UI systems
#[derive(Event, Debug, Clone, Copy)]
pub struct ChunkLoaded {
    pub coord: ChunkCoord,
    pub content_count: usize,
}

impl ChunkLoaded {
    pub fn new(coord: ChunkCoord, content_count: usize) -> Self {
        Self { coord, content_count }
    }
}

/// Request to unload a specific chunk (8 bytes)
/// Sent by: world streaming system
/// Handled by: chunk unloading system
#[derive(Event, Debug, Clone, Copy)]
pub struct RequestChunkUnload {
    pub coord: ChunkCoord,
}

impl RequestChunkUnload {
    pub fn new(coord: ChunkCoord) -> Self {
        Self { coord }
    }
}

/// Notification that a chunk has been unloaded (8 bytes)
/// Sent by: chunk unloading system
/// Handled by: cleanup systems, UI systems
#[derive(Event, Debug, Clone, Copy)]
pub struct ChunkUnloaded {
    pub coord: ChunkCoord,
}

impl ChunkUnloaded {
    pub fn new(coord: ChunkCoord) -> Self {
        Self { coord }
    }
}

// Compile-time size verification (≤128 bytes requirement)
const _: () = {
    assert!(std::mem::size_of::<RequestChunkLoad>() <= 128);
    assert!(std::mem::size_of::<ChunkLoaded>() <= 128);
    assert!(std::mem::size_of::<RequestChunkUnload>() <= 128);
    assert!(std::mem::size_of::<ChunkUnloaded>() <= 128);
};
