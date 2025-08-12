use bevy::prelude::*;

/// Focused resource for streaming performance metrics
/// Single responsibility: Track frame-by-frame streaming statistics
#[derive(Resource, Default)]
pub struct StreamingMetrics {
    /// Timestamp of last streaming update
    pub last_update: f32,
    
    /// Number of chunks loaded in current frame
    pub chunks_loaded_this_frame: u32,
    
    /// Number of chunks unloaded in current frame
    pub chunks_unloaded_this_frame: u32,
    
    /// Maximum chunks to process per frame
    pub max_chunks_per_frame: u32,
}

impl StreamingMetrics {
    pub fn new(max_chunks_per_frame: u32) -> Self {
        Self {
            last_update: 0.0,
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            max_chunks_per_frame,
        }
    }
    
    /// Reset frame counters
    pub fn reset_frame_counters(&mut self) {
        self.chunks_loaded_this_frame = 0;
        self.chunks_unloaded_this_frame = 0;
    }
    
    /// Increment loaded chunk counter
    pub fn record_chunk_load(&mut self) {
        self.chunks_loaded_this_frame += 1;
    }
    
    /// Increment unloaded chunk counter
    pub fn record_chunk_unload(&mut self) {
        self.chunks_unloaded_this_frame += 1;
    }
    
    /// Check if we can load more chunks this frame
    pub fn can_load_chunk(&self) -> bool {
        self.chunks_loaded_this_frame < self.max_chunks_per_frame
    }
    
    /// Check if we can unload more chunks this frame
    pub fn can_unload_chunk(&self) -> bool {
        self.chunks_unloaded_this_frame < self.max_chunks_per_frame
    }
    
    /// Update the last update timestamp
    pub fn update_timestamp(&mut self, time: f32) {
        self.last_update = time;
    }
    
    /// Get total chunks processed this frame
    pub fn total_chunks_processed(&self) -> u32 {
        self.chunks_loaded_this_frame + self.chunks_unloaded_this_frame
    }
}
