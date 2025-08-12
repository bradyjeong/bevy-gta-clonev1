use bevy::prelude::*;
use crate::world::chunk_coord::ChunkCoord;

/// State of a chunk in the streaming system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
}

/// Metadata and state for a single world chunk
#[derive(Debug, Clone)]
pub struct ChunkData {
    pub coord: ChunkCoord,
    pub state: ChunkState,
    pub distance_to_player: f32,
    pub entities: Vec<Entity>,
    pub last_update: f32,
    
    // Layer generation flags
    pub roads_generated: bool,
    pub buildings_generated: bool,
    pub vehicles_generated: bool,
    pub vegetation_generated: bool,
}

impl ChunkData {
    /// Create new unloaded chunk data
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            state: ChunkState::Unloaded,
            distance_to_player: f32::INFINITY,
            entities: Vec::new(),
            last_update: 0.0,
            roads_generated: false,
            buildings_generated: false,
            vehicles_generated: false,
            vegetation_generated: false,
        }
    }
}

/// Component marking an entity as belonging to a chunk
#[derive(Component)]
pub struct UnifiedChunkEntity {
    pub coord: ChunkCoord,
    pub layer: ContentLayer,
}

/// Content layers for chunk generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentLayer {
    Roads,
    Buildings,
    Vehicles,
    Vegetation,
    NPCs,
}
