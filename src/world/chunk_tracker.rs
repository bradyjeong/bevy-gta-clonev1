use bevy::prelude::*;
use std::collections::HashMap;

/// Event requesting a chunk to be loaded
#[derive(Event)]
pub struct ChunkLoadRequest {
    pub coord: ChunkCoord,
    pub priority: f32,
}

/// Event requesting a chunk to be unloaded
#[derive(Event)]
pub struct ChunkUnloadRequest {
    pub coord: ChunkCoord,
}

/// ChunkTracker - Core chunk streaming state (≤64 bytes)
#[derive(Resource, Debug)]
pub struct ChunkTracker {
    /// Currently loaded chunk coordinates and states (36 bytes = 3 chunks * 12 bytes)
    pub loaded_chunks: [(ChunkCoord, ChunkState); 3],
    /// Current focus chunk for streaming (8 bytes)
    pub focus_chunk: ChunkCoord,
    /// Streaming radius in chunks (2 bytes)
    pub lod_radius: i16,
    /// Active chunk count (1 byte)
    pub active_count: u8,
    /// Focus chunk valid flag (1 byte)
    pub focus_valid: bool,
    /// Frame counter for performance tracking (4 bytes)
    pub last_update_frame: u32,
    /// Performance flags (4 bytes)
    pub perf_flags: u32,
    /// Reserved padding (8 bytes)
    _padding: [u32; 2],
}

/// ChunkTables - Dynamic chunk data (unbounded size)
/// Separated from ChunkTracker to maintain cache-friendliness
#[derive(Resource, Debug, Default)]
pub struct ChunkTables {
    /// All loaded chunks with their states
    pub loaded: HashMap<ChunkCoord, ChunkState>,
    /// Chunks currently being loaded
    pub loading: HashMap<ChunkCoord, ChunkState>,
    /// Chunks being unloaded
    pub unloading: HashMap<ChunkCoord, ChunkState>,
    /// Distance cache for streaming decisions
    pub distances: HashMap<ChunkCoord, f32>,
}

/// Chunk coordinate identifier (8 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    
    pub fn from_world_pos(world_pos: Vec3) -> Self {
        Self {
            x: (world_pos.x / 200.0).floor() as i32,  // Using UNIFIED_CHUNK_SIZE
            z: (world_pos.z / 200.0).floor() as i32,
        }
    }
    
    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * 200.0 + 100.0,  // Using UNIFIED_CHUNK_SIZE
            0.0,
            self.z as f32 * 200.0 + 100.0,
        )
    }
}

/// Chunk loading state (1 byte enum)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Loading,
    Loaded,
    Unloading,
    Unloaded,
}

impl Default for ChunkTracker {
    fn default() -> Self {
        Self {
            loaded_chunks: [(ChunkCoord { x: 0, z: 0 }, ChunkState::Unloaded); 3],
            focus_chunk: ChunkCoord { x: 0, z: 0 },
            lod_radius: 8,
            active_count: 0,
            focus_valid: false,
            last_update_frame: 0,
            perf_flags: 0,
            _padding: [0; 2],
        }
    }
}

impl ChunkTracker {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn is_chunk_loaded(&self, coord: ChunkCoord) -> bool {
        for i in 0..self.active_count as usize {
            let (loaded_coord, state) = self.loaded_chunks[i];
            if loaded_coord == coord && state == ChunkState::Loaded {
                return true;
            }
        }
        false
    }
    
    pub fn get_chunks_to_load(&self, center: ChunkCoord, radius: i16) -> Vec<ChunkCoord> {
        let mut chunks = Vec::new();
        for x in -radius..=radius {
            for z in -radius..=radius {
                let coord = ChunkCoord {
                    x: center.x + x as i32,
                    z: center.z + z as i32,
                };
                if !self.is_chunk_present(coord) {
                    chunks.push(coord);
                }
            }
        }
        chunks
    }
    
    pub fn mark_loading(&mut self, coord: ChunkCoord) {
        self.set_chunk_state(coord, ChunkState::Loading);
    }
    
    pub fn mark_loaded(&mut self, coord: ChunkCoord) {
        self.set_chunk_state(coord, ChunkState::Loaded);
    }
    
    pub fn clear(&mut self) {
        self.loaded_chunks = [(ChunkCoord { x: 0, z: 0 }, ChunkState::Unloaded); 3];
        self.active_count = 0;
        self.last_update_frame = 0;
    }
    
    pub fn get_loaded_chunks(&self) -> Vec<ChunkCoord> {
        let mut chunks = Vec::new();
        for i in 0..self.active_count as usize {
            let (coord, state) = self.loaded_chunks[i];
            if matches!(state, ChunkState::Loaded) {
                chunks.push(coord);
            }
        }
        chunks
    }
    
    pub fn mark_chunk_loading(&mut self, coord: ChunkCoord) {
        self.mark_loading(coord);
    }
    
    pub fn mark_chunk_unloading(&mut self, coord: ChunkCoord) {
        self.set_chunk_state(coord, ChunkState::Unloaded);
    }
    
    pub fn is_chunk_loading(&self, coord: ChunkCoord) -> bool {
        for i in 0..self.active_count as usize {
            let (loaded_coord, state) = self.loaded_chunks[i];
            if loaded_coord == coord && matches!(state, ChunkState::Loading) {
                return true;
            }
        }
        false
    }
    
    pub fn get_loading_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.active_count as usize {
            let (_, state) = self.loaded_chunks[i];
            if matches!(state, ChunkState::Loading) {
                count += 1;
            }
        }
        count
    }
    
    pub fn cleanup_distant_chunks(&mut self, center: ChunkCoord, max_radius: i16) {
        let mut write_index = 0;
        for read_index in 0..self.active_count as usize {
            let (coord, _state) = self.loaded_chunks[read_index];
            let dx = (coord.x - center.x) as i16;
            let dz = (coord.z - center.z) as i16;
            if dx.abs() <= max_radius && dz.abs() <= max_radius {
                // Keep this chunk
                if write_index != read_index {
                    self.loaded_chunks[write_index] = self.loaded_chunks[read_index];
                }
                write_index += 1;
            }
        }
        
        // Clear remaining slots
        for i in write_index..self.active_count as usize {
            self.loaded_chunks[i] = (ChunkCoord { x: 0, z: 0 }, ChunkState::Unloaded);
        }
        self.active_count = write_index as u8;
    }
    
    fn is_chunk_present(&self, coord: ChunkCoord) -> bool {
        for i in 0..self.active_count as usize {
            let (loaded_coord, _) = self.loaded_chunks[i];
            if loaded_coord == coord {
                return true;
            }
        }
        false
    }
    
    fn set_chunk_state(&mut self, coord: ChunkCoord, state: ChunkState) {
        // Try to find existing chunk
        for i in 0..self.active_count as usize {
            let (loaded_coord, _) = self.loaded_chunks[i];
            if loaded_coord == coord {
                self.loaded_chunks[i] = (coord, state);
                return;
            }
        }
        
        // Add new chunk if space available
        if (self.active_count as usize) < self.loaded_chunks.len() {
            let index = self.active_count as usize;
            self.loaded_chunks[index] = (coord, state);
            self.active_count += 1;
        }
    }
}

impl ChunkTables {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn update_chunk_distance(&mut self, coord: ChunkCoord, distance: f32) {
        self.distances.insert(coord, distance);
    }
    
    pub fn get_chunk_distance(&self, coord: ChunkCoord) -> Option<f32> {
        self.distances.get(&coord).copied()
    }
    
    pub fn clear(&mut self) {
        self.loaded.clear();
        self.loading.clear();
        self.unloading.clear();
        self.distances.clear();
    }
}

// Static size assertions - ChunkTracker MUST be ≤64 bytes (hot-path resource)
// loaded_chunks: [(ChunkCoord, ChunkState); 3] = 36 bytes (12 bytes * 3)
// focus_chunk: ChunkCoord = 8 bytes
// lod_radius: i16 = 2 bytes
// active_count: u8 = 1 byte
// focus_valid: bool = 1 byte
// last_update_frame: u32 = 4 bytes
// perf_flags: u32 = 4 bytes
// _padding: [u32; 2] = 8 bytes
// Total: 36 + 8 + 2 + 1 + 1 + 4 + 4 + 8 = 64 bytes exactly
static_assertions::const_assert!(std::mem::size_of::<ChunkTracker>() <= 64);

// ChunkTables has unbounded size due to HashMaps (this is intentional)
// No size constraint needed for ChunkTables as it's designed for dynamic data
