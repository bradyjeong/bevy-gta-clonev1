use bevy::prelude::*;

/// Marker component for terrain chunks
#[derive(Component, Default)]
pub struct TerrainChunk {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub size: f32,
}

/// Component for terrain metadata
#[derive(Component, Default)]
pub struct TerrainMeta {
    pub height_range: (f32, f32),  // (min_height, max_height)
    pub is_generated: bool,
    pub generation_time: f32,
}

impl TerrainChunk {
    pub fn new(chunk_x: i32, chunk_z: i32, size: f32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            size,
        }
    }
    
    /// Check if a world position is within this chunk
    pub fn contains_position(&self, x: f32, z: f32) -> bool {
        let chunk_world_x = self.chunk_x as f32 * self.size;
        let chunk_world_z = self.chunk_z as f32 * self.size;
        
        x >= chunk_world_x && x < chunk_world_x + self.size &&
        z >= chunk_world_z && z < chunk_world_z + self.size
    }
}

impl TerrainMeta {
    pub fn new(min_height: f32, max_height: f32) -> Self {
        Self {
            height_range: (min_height, max_height),
            is_generated: false,
            generation_time: 0.0,
        }
    }
    
    pub fn mark_generated(&mut self, generation_time: f32) {
        self.is_generated = true;
        self.generation_time = generation_time;
    }
}
