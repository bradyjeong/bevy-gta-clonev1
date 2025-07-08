//! ───────────────────────────────────────────────
//! System:   Unified World Manager
//! Purpose:  Manages world chunks and spatial organization
//! Schedule: Update
//! Reads:    `ActiveEntity`, Transform, `ChunkData`
//! Writes:   Commands, `WorldManager`
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
// Removed bevy16_compat - using direct Bevy methods
use std::collections::HashMap;
use game_core::prelude::*;
use game_core::components::{RoadNetwork, ChunkState};

const CHUNK_SIZE: f32 = 200.0;
const CHUNK_CLEANUP_RADIUS: f32 = 1000.0;
pub const UNIFIED_CHUNK_SIZE: f32 = CHUNK_SIZE;
pub const UNIFIED_STREAMING_RADIUS: f32 = 800.0;

#[derive(Resource, Default)]
pub struct WorldManager {
    pub chunks: HashMap<ChunkCoord, ChunkData>,
    pub active_chunks: Vec<ChunkCoord>,
    pub placement_grid: PlacementGrid,
    pub road_network: RoadNetwork,
    pub max_chunks_per_frame: usize,
}

// Oracle's stubs
pub type UnifiedWorldManager = WorldManager;

// Re-export canonical types from game_core
pub use game_core::world::ContentLayer;
pub use game_core::spatial::ChunkCoord;

/// Utility functions for `ChunkCoord` using local `CHUNK_SIZE` constant
pub trait ChunkCoordExt {
    fn from_world_pos_local(world_pos: Vec3) -> Self;
    fn to_world_pos_local(&self) -> Vec3;
}

impl ChunkCoordExt for ChunkCoord {
    fn from_world_pos_local(world_pos: Vec3) -> Self {
        Self::from_world_pos(world_pos, CHUNK_SIZE)
    }
    
    fn to_world_pos_local(&self) -> Vec3 {
        self.to_world_pos(CHUNK_SIZE)
    }
}

#[derive(Default)]
pub struct ChunkData {
    pub entities: Vec<Entity>,
    pub last_updated: f32,
    pub is_loaded: bool,
    pub distance_to_player: f32,
    pub state: ChunkState,
    pub coord: ChunkCoord,
    pub last_update: f32,
    pub entity_count: usize,
    pub roads_generated: bool,
    pub buildings_generated: bool,
    pub vehicles_generated: bool,
    pub vegetation_generated: bool,
}

impl ChunkData {
    #[must_use] pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            last_updated: 0.0,
            is_loaded: false,
            distance_to_player: f32::MAX,
            state: ChunkState::Unloaded,
            coord: ChunkCoord::new(0, 0),
            last_update: 0.0,
            entity_count: 0,
            roads_generated: false,
            buildings_generated: false,
            vehicles_generated: false,
            vegetation_generated: false,
        }
    }
    
    pub fn add_entity(&mut self, entity: Entity) {
        if !self.entities.contains(&entity) {
            self.entities.push(entity);
        }
    }
    
    pub fn remove_entity(&mut self, entity: Entity) {
        self.entities.retain(|&e| e != entity);
    }
}

#[derive(Default)]
pub struct PlacementGrid {
    /// Grid of entities per cell for collision detection
    grid: HashMap<(i32, i32), Vec<(Vec3, ContentType, f32)>>,
    /// Grid cell size (should be smaller than chunk size for efficiency)
    cell_size: f32,
}

impl PlacementGrid {
    #[must_use] pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
            cell_size: 50.0, // 4 cells per chunk
        }
    }
    
    pub fn clear(&mut self) {
        self.grid.clear();
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
    
    #[must_use] pub fn can_place(&self, position: Vec3, _content_type: ContentType, radius: f32, min_distance: f32) -> bool {
        let base_cell = self.world_to_grid(position);
        
        // Check current cell and adjacent cells
        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_cell = (base_cell.0 + dx, base_cell.1 + dz);
                
                if let Some(entities) = self.grid.get(&check_cell) {
                    for (existing_pos, _, existing_radius) in entities {
                        let distance = position.distance(*existing_pos);
                        let required_distance = min_distance + radius + existing_radius;
                        
                        if distance < required_distance {
                            return false;
                        }
                    }
                }
            }
        }
        
        true
    }
    
    #[must_use] pub fn get_nearby_entities(&self, position: Vec3, radius: f32) -> Vec<(Vec3, ContentType, f32)> {
        let mut result = Vec::new();
        let base_cell = self.world_to_grid(position);
        
        // Calculate how many cells to check based on radius
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        
        for dx in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let check_cell = (base_cell.0 + dx, base_cell.1 + dz);
                
                if let Some(entities) = self.grid.get(&check_cell) {
                    for &(entity_pos, content_type, entity_radius) in entities {
                        if position.distance(entity_pos) <= radius + entity_radius {
                            result.push((entity_pos, content_type, entity_radius));
                        }
                    }
                }
            }
        }
        
        result
    }
    
    fn world_to_grid(&self, world_pos: Vec3) -> (i32, i32) {
        (
            (world_pos.x / self.cell_size).floor() as i32,
            (world_pos.z / self.cell_size).floor() as i32,
        )
    }
}

impl WorldManager {
    #[must_use] pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            active_chunks: Vec::new(),
            placement_grid: PlacementGrid::new(),
            road_network: RoadNetwork::new(),
            max_chunks_per_frame: 4,
        }
    }
    
    pub fn get_or_create_chunk(&mut self, coord: ChunkCoord) -> &mut ChunkData {
        self.chunks.entry(coord).or_default()
    }
    
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut ChunkData> {
        self.chunks.get_mut(&coord)
    }
    
    #[must_use] pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&ChunkData> {
        self.chunks.get(&coord)
    }
    
    #[must_use] pub fn calculate_lod_level(&self, distance: f32) -> u32 {
        if distance < 150.0 {
            0 // High detail
        } else if distance < 300.0 {
            1 // Medium detail
        } else if distance < 600.0 {
            2 // Low detail
        } else {
            3 // Very low detail
        }
    }
    
    pub fn unload_chunk(&mut self, coord: ChunkCoord, commands: &mut Commands) {
        if let Some(chunk_data) = self.chunks.get(&coord) {
            // Despawn all entities in the chunk
            for &entity in &chunk_data.entities {
                commands.entity(entity).despawn_recursive();
            }
        }
        
        self.chunks.remove(&coord);
        self.active_chunks.retain(|&c| c != coord);
    }
    
    pub fn update_active_chunks(&mut self, player_pos: Vec3, load_radius: f32) {
        let player_chunk = ChunkCoord::from_world_pos(player_pos, CHUNK_SIZE);
        let chunk_load_radius = (load_radius / CHUNK_SIZE).ceil() as i32;
        
        // Calculate chunks that should be loaded
        let mut target_chunks = Vec::new();
        for dx in -chunk_load_radius..=chunk_load_radius {
            for dz in -chunk_load_radius..=chunk_load_radius {
                let chunk_coord = ChunkCoord::new(
                    player_chunk.x + dx,
                    player_chunk.z + dz,
                );
                
                let chunk_center = chunk_coord.to_world_pos(CHUNK_SIZE);
                let distance = player_pos.distance(chunk_center);
                
                if distance <= load_radius {
                    target_chunks.push(chunk_coord);
                }
            }
        }
        
        self.active_chunks = target_chunks;
    }
    
    pub fn cleanup_distant_chunks(&mut self, player_pos: Vec3, commands: &mut Commands) {
        let chunks_to_unload: Vec<ChunkCoord> = self.chunks
            .keys()
            .filter(|&&coord| {
                let chunk_center = coord.to_world_pos(CHUNK_SIZE);
                player_pos.distance(chunk_center) > CHUNK_CLEANUP_RADIUS
            })
            .copied()
            .collect();
        
        for coord in chunks_to_unload {
            self.unload_chunk(coord, commands);
        }
    }
    
    #[must_use] pub fn is_chunk_loaded(&self, coord: ChunkCoord) -> bool {
        self.chunks.get(&coord).is_some_and(|chunk| chunk.is_loaded)
    }
    
    pub fn mark_chunk_loaded(&mut self, coord: ChunkCoord) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.is_loaded = true;
        }
    }
    
    pub fn add_entity_to_chunk(&mut self, entity: Entity, world_pos: Vec3) {
        let coord = ChunkCoord::from_world_pos(world_pos, CHUNK_SIZE);
        let chunk = self.get_or_create_chunk(coord);
        chunk.add_entity(entity);
    }
    
    pub fn remove_entity_from_chunk(&mut self, entity: Entity, world_pos: Vec3) {
        let coord = ChunkCoord::from_world_pos(world_pos, CHUNK_SIZE);
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.remove_entity(entity);
        }
    }
}

pub fn world_management_system(
    mut world_manager: ResMut<WorldManager>,
    mut commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    time: Res<Time>,
) {
    if let Ok(active_transform) = active_query.single() {
        let player_pos = active_transform.translation;
        let current_time = time.elapsed_secs();
        
        // Update which chunks should be active
        world_manager.update_active_chunks(player_pos, 800.0);
        
        // Ensure active chunks are loaded
        for &chunk_coord in &world_manager.active_chunks.clone() {
            if !world_manager.is_chunk_loaded(chunk_coord) {
                world_manager.mark_chunk_loaded(chunk_coord);
                
                // Update chunk timestamp
                if let Some(chunk) = world_manager.chunks.get_mut(&chunk_coord) {
                    chunk.last_updated = current_time;
                }
            }
        }
        
        // Cleanup distant chunks periodically
        if current_time % 10.0 < 0.1 { // Every 10 seconds
            world_manager.cleanup_distant_chunks(player_pos, &mut commands);
        }
    }
}

pub fn spatial_query_system(
    world_manager: Res<WorldManager>,
    entity_query: Query<(&Transform, &ContentType), With<DynamicContent>>,
) {
    // This system can be used for spatial queries
    // For example, finding all entities of a certain type within a radius
    
    for (transform, content_type) in entity_query.iter() {
        let nearby = world_manager.placement_grid.get_nearby_entities(
            transform.translation,
            100.0, // Search radius
        );
        
        // Process nearby entities as needed
        for (_pos, nearby_type, _radius) in nearby {
            if nearby_type == *content_type {
                // Handle same-type entity proximity
            }
        }
    }
}

// Utility functions

#[must_use] pub fn world_pos_to_chunk_coord(world_pos: Vec3) -> ChunkCoord {
    ChunkCoord::from_world_pos(world_pos, CHUNK_SIZE)
}

#[must_use] pub fn chunk_coord_to_world_pos(coord: ChunkCoord) -> Vec3 {
    coord.to_world_pos(CHUNK_SIZE)
}

#[must_use] pub fn get_chunk_bounds(coord: ChunkCoord) -> (Vec3, Vec3) {
    let center = coord.to_world_pos(CHUNK_SIZE);
    let half_size = CHUNK_SIZE * 0.5;
    
    let min = center - Vec3::new(half_size, 0.0, half_size);
    let max = center + Vec3::new(half_size, 0.0, half_size);
    
    (min, max)
}

#[must_use] pub fn is_position_in_chunk(world_pos: Vec3, coord: ChunkCoord) -> bool {
    let (min, max) = get_chunk_bounds(coord);
    
    world_pos.x >= min.x && world_pos.x <= max.x &&
    world_pos.z >= min.z && world_pos.z <= max.z
}

// Oracle's missing system stubs
pub fn unified_world_streaming_system() {
    // World streaming stub - no implementation yet
}

pub fn layered_generation_coordinator() {
    // Generation coordinator stub - no implementation yet
}

pub fn vehicle_layer_system() {
    // Vehicle layer stub - no implementation yet
}

pub fn master_unified_lod_system() {
    // Master LOD stub - no implementation yet
}

pub fn master_lod_performance_monitor() {
    // LOD performance monitor stub - no implementation yet
}

pub fn initialize_master_lod_system() {
    // Initialize master LOD stub - no implementation yet
}

pub fn adaptive_lod_system() {
    // Adaptive LOD stub - no implementation yet
}

pub fn unified_cleanup_system() {
    // Unified cleanup stub - no implementation yet
}

pub fn spawn_new_npc_system() {
    // Spawn new NPC stub - no implementation yet
}
