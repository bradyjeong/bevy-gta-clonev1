use bevy::prelude::*;
use std::collections::HashMap;
use crate::components::*;
use crate::systems::world::road_network::RoadNetwork;
use crate::events::world::chunk_events::{RequestChunkLoad, RequestChunkUnload, ChunkCoord as EventChunkCoord};

// UNIFIED WORLD GENERATION SYSTEM
// This replaces map_system.rs, dynamic_content.rs coordination
// Provides single source of truth for world streaming and generation

/// Standard chunk size used across all world systems
pub const UNIFIED_CHUNK_SIZE: f32 = 200.0;

/// Maximum streaming radius around active entity
pub const UNIFIED_STREAMING_RADIUS: f32 = 800.0;

/// LOD transition distances - optimized for 60+ FPS target
pub const LOD_DISTANCES: [f32; 3] = [150.0, 300.0, 500.0];

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
            x: (world_pos.x / UNIFIED_CHUNK_SIZE).floor() as i32,
            z: (world_pos.z / UNIFIED_CHUNK_SIZE).floor() as i32,
        }
    }
    
    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * UNIFIED_CHUNK_SIZE + UNIFIED_CHUNK_SIZE * 0.5,
            0.0,
            self.z as f32 * UNIFIED_CHUNK_SIZE + UNIFIED_CHUNK_SIZE * 0.5,
        )
    }
    
    pub fn distance_to(&self, other: ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Unloaded,
    Loading,
    Loaded { lod_level: usize },
    Unloading,
}

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

/// Shared collision/placement grid for preventing entity overlap
#[derive(Debug, Default)]
pub struct PlacementGrid {
    /// Grid cells containing entity positions and types
    /// Key: (grid_x, grid_z), Value: Vec of (position, content_type, radius)
    grid: HashMap<(i32, i32), Vec<(Vec3, ContentType, f32)>>,
    /// Grid cell size (should be smaller than chunk size for efficiency)
    cell_size: f32,
}

impl PlacementGrid {
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
            cell_size: 50.0, // 4 cells per chunk
        }
    }
    
    pub fn clear(&mut self) {
        self.grid.clear();
    }
    
    // Getter for migration support
    pub fn get_cell_size(&self) -> f32 {
        self.cell_size
    }
    
    pub fn add_entity(&mut self, position: Vec3, content_type: ContentType, radius: f32) {
        let cell = self.world_to_grid(position);
        self.grid.entry(cell).or_default().push((position, content_type, radius));
    }
    
    pub fn remove_entity(&mut self, position: Vec3, content_type: ContentType) {
        let cell = self.world_to_grid(position);
        if let Some(entities) = self.grid.get_mut(&cell) {
            entities.retain(|(pos, content, _)| {
                !(pos.distance(position) < 1.0 && *content == content_type)
            });
        }
    }
    
    pub fn can_place(&self, position: Vec3, _content_type: ContentType, radius: f32, min_distance: f32) -> bool {
        let cell = self.world_to_grid(position);
        
        // Check current cell and adjacent cells
        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(entities) = self.grid.get(&check_cell) {
                    for (existing_pos, _existing_type, existing_radius) in entities {
                        let distance = position.distance(*existing_pos);
                        let required_distance = min_distance.max(*existing_radius + radius);
                        
                        if distance < required_distance {
                            return false;
                        }
                    }
                }
            }
        }
        
        true
    }
    
    pub fn get_nearby_entities(&self, position: Vec3, radius: f32) -> Vec<(Vec3, ContentType, f32)> {
        let mut result = Vec::new();
        let cell = self.world_to_grid(position);
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        
        for dx in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(entities) = self.grid.get(&check_cell) {
                    for (entity_pos, content_type, entity_radius) in entities {
                        if position.distance(*entity_pos) <= radius {
                            result.push((*entity_pos, *content_type, *entity_radius));
                        }
                    }
                }
            }
        }
        
        result
    }
    
    fn world_to_grid(&self, position: Vec3) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }
}

/// Central resource managing all world generation
#[derive(Resource)]
pub struct UnifiedWorldManager {
    pub chunks: HashMap<ChunkCoord, ChunkData>,
    pub placement_grid: PlacementGrid,
    pub road_network: RoadNetwork,
    
    // Streaming state
    pub active_chunk: Option<ChunkCoord>,
    pub streaming_radius_chunks: i32,
    pub last_update: f32,
    
    // Performance tracking
    pub chunks_loaded_this_frame: usize,
    pub chunks_unloaded_this_frame: usize,
    pub max_chunks_per_frame: usize,
}

impl Default for UnifiedWorldManager {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            placement_grid: PlacementGrid::new(),
            road_network: RoadNetwork::default(),
            active_chunk: None,
            streaming_radius_chunks: (UNIFIED_STREAMING_RADIUS / UNIFIED_CHUNK_SIZE).ceil() as i32,
            last_update: 0.0,
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            max_chunks_per_frame: 4, // Prevent frame drops
        }
    }
}

impl UnifiedWorldManager {
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&ChunkData> {
        self.chunks.get(&coord)
    }
    
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> &mut ChunkData {
        self.chunks.entry(coord).or_insert_with(|| ChunkData::new(coord))
    }
    
    pub fn is_chunk_loaded(&self, coord: ChunkCoord) -> bool {
        matches!(
            self.chunks.get(&coord).map(|c| c.state),
            Some(ChunkState::Loaded { .. })
        )
    }
    
    pub fn calculate_lod_level(&self, distance: f32) -> usize {
        for (i, &max_distance) in LOD_DISTANCES.iter().enumerate() {
            if distance <= max_distance {
                return i;
            }
        }
        LOD_DISTANCES.len() - 1
    }
    
    pub fn cleanup_distant_chunks(&mut self, active_pos: Vec3) -> Vec<ChunkCoord> {
        let mut to_unload = Vec::new();
        
        for (coord, chunk) in &mut self.chunks {
            chunk.distance_to_player = active_pos.distance(coord.to_world_pos());
            
            if chunk.distance_to_player > UNIFIED_STREAMING_RADIUS + UNIFIED_CHUNK_SIZE {
                if !matches!(chunk.state, ChunkState::Unloaded | ChunkState::Unloading) {
                    chunk.state = ChunkState::Unloading;
                    to_unload.push(*coord);
                }
            }
        }
        
        to_unload
    }
    
    pub fn get_chunks_to_load(&mut self, active_pos: Vec3) -> Vec<ChunkCoord> {
        let active_chunk = ChunkCoord::from_world_pos(active_pos);
        let mut to_load = Vec::new();
        
        for dx in -self.streaming_radius_chunks..=self.streaming_radius_chunks {
            for dz in -self.streaming_radius_chunks..=self.streaming_radius_chunks {
                let coord = ChunkCoord::new(active_chunk.x + dx, active_chunk.z + dz);
                let distance = active_pos.distance(coord.to_world_pos());
                
                if distance <= UNIFIED_STREAMING_RADIUS {
                    let chunk = self.get_chunk_mut(coord);
                    if matches!(chunk.state, ChunkState::Unloaded) {
                        chunk.state = ChunkState::Loading;
                        chunk.distance_to_player = distance;
                        to_load.push(coord);
                    }
                }
            }
        }
        
        to_load
    }
    
    pub fn clear_placement_grid_for_chunk(&mut self, coord: ChunkCoord) {
        // Remove all entities from placement grid within this chunk's bounds
        let _chunk_center = coord.to_world_pos();
        let _half_size = UNIFIED_CHUNK_SIZE * 0.5;
        
        // This is inefficient - in a full implementation, we'd track which
        // grid cells belong to which chunks
        // For now, we'll implement a more targeted removal in the actual generation
    }
}

#[derive(Component)]
pub struct UnifiedChunkEntity {
    pub coord: ChunkCoord,
    pub layer: ContentLayer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentLayer {
    Roads,
    Buildings,
    Vehicles,
    Vegetation,
    NPCs,
}

/// Main unified world streaming system
pub fn unified_world_streaming_system(
    mut world_manager: ResMut<UnifiedWorldManager>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    time: Res<Time>,
    mut chunk_load_writer: EventWriter<RequestChunkLoad>,
    mut chunk_unload_writer: EventWriter<RequestChunkUnload>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Update timing
    world_manager.last_update = time.elapsed_secs();
    world_manager.chunks_loaded_this_frame = 0;
    world_manager.chunks_unloaded_this_frame = 0;
    
    // Update active chunk
    let current_chunk = ChunkCoord::from_world_pos(active_pos);
    let _chunk_changed = world_manager.active_chunk != Some(current_chunk);
    world_manager.active_chunk = Some(current_chunk);
    
    // Cleanup distant chunks
    let chunks_to_unload = world_manager.cleanup_distant_chunks(active_pos);
    for coord in chunks_to_unload {
        if world_manager.chunks_unloaded_this_frame >= world_manager.max_chunks_per_frame {
            break;
        }
        request_chunk_unload(&mut world_manager, coord, &mut chunk_unload_writer);
        world_manager.chunks_unloaded_this_frame += 1;
    }
    
    // Load new chunks
    let chunks_to_load = world_manager.get_chunks_to_load(active_pos);
    for coord in chunks_to_load {
        if world_manager.chunks_loaded_this_frame >= world_manager.max_chunks_per_frame {
            break;
        }
        request_chunk_loading(&mut world_manager, coord, &mut chunk_load_writer);
        world_manager.chunks_loaded_this_frame += 1;
    }
}

fn request_chunk_loading(
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    chunk_load_writer: &mut EventWriter<RequestChunkLoad>,
) {
    // This function emits chunk loading requests
    // The actual content generation will be handled by event handlers
    let chunk = world_manager.get_chunk_mut(coord);
    chunk.state = ChunkState::Loading;
    
    // Convert internal ChunkCoord to event ChunkCoord and emit request
    let event_coord = EventChunkCoord::new(coord.x, coord.z);
    chunk_load_writer.write(RequestChunkLoad::new(event_coord));
}

fn request_chunk_unload(
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    chunk_unload_writer: &mut EventWriter<RequestChunkUnload>,
) {
    if world_manager.chunks.contains_key(&coord) {
        // Clear from placement grid
        world_manager.clear_placement_grid_for_chunk(coord);
        
        // Convert internal ChunkCoord to event ChunkCoord and emit request
        let event_coord = EventChunkCoord::new(coord.x, coord.z);
        chunk_unload_writer.write(RequestChunkUnload::new(event_coord));
        
        // Remove chunk data
        world_manager.chunks.remove(&coord);
    }
}
