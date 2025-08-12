use bevy::prelude::*;
use std::collections::HashMap;
use crate::world::chunk_coord::ChunkCoord;
use crate::world::chunk_data::ChunkData;

/// Focused resource for chunk state and LOD management
/// Single responsibility: Track chunk lifecycle and LOD states  
/// Renamed to ChunkManager to avoid conflict with world::ChunkTracker
#[derive(Resource, Default)]
pub struct ChunkManager {
    /// All loaded chunks with their data
    pub chunks: HashMap<ChunkCoord, ChunkData>,
    
    /// Radius for chunk streaming in chunk units
    pub streaming_radius_chunks: i32,
    
    /// Currently active chunk (player's chunk)
    pub active_chunk: Option<ChunkCoord>,
}

impl ChunkManager {
    pub fn new(streaming_radius: i32) -> Self {
        Self {
            chunks: HashMap::new(),
            streaming_radius_chunks: streaming_radius,
            active_chunk: None,
        }
    }
    
    /// Get chunk data if it exists
    pub fn get_chunk(&self, coord: &ChunkCoord) -> Option<&ChunkData> {
        self.chunks.get(coord)
    }
    
    /// Get mutable chunk data if it exists
    pub fn get_chunk_mut(&mut self, coord: &ChunkCoord) -> Option<&mut ChunkData> {
        self.chunks.get_mut(coord)
    }
    
    /// Add or update a chunk
    pub fn insert_chunk(&mut self, coord: ChunkCoord, data: ChunkData) {
        self.chunks.insert(coord, data);
    }
    
    /// Remove a chunk
    pub fn remove_chunk(&mut self, coord: &ChunkCoord) -> Option<ChunkData> {
        self.chunks.remove(coord)
    }
    
    /// Check if a chunk is loaded
    pub fn is_chunk_loaded(&self, coord: &ChunkCoord) -> bool {
        self.chunks.contains_key(coord)
    }
    
    /// Get all loaded chunk coordinates
    pub fn loaded_chunks(&self) -> impl Iterator<Item = &ChunkCoord> {
        self.chunks.keys()
    }
    
    /// Update the active chunk
    pub fn set_active_chunk(&mut self, coord: ChunkCoord) {
        self.active_chunk = Some(coord);
    }
    
    /// Get the active chunk
    pub fn get_active_chunk(&self) -> Option<ChunkCoord> {
        self.active_chunk
    }
    
    /// Get all chunks within streaming radius of a position
    pub fn chunks_in_radius(&self, center: ChunkCoord) -> Vec<ChunkCoord> {
        let mut chunks = Vec::new();
        let radius = self.streaming_radius_chunks;
        
        for x in -radius..=radius {
            for z in -radius..=radius {
                chunks.push(ChunkCoord {
                    x: center.x + x,
                    z: center.z + z,
                });
            }
        }
        
        chunks
    }
}
