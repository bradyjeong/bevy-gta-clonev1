//! ───────────────────────────────────────────────
//! System:   Unified World Manager
//! Purpose:  Manages world chunks and spatial organization
//! Schedule: Update
//! Reads:    ActiveEntity, Transform, ChunkData
//! Writes:   Commands, WorldManager
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;

const CHUNK_SIZE: f32 = 200.0;
const CHUNK_CLEANUP_RADIUS: f32 = 1000.0;

#[derive(Resource, Default)]
pub struct WorldManager {
    pub chunks: HashMap<ChunkCoord, ChunkData>,
    pub active_chunks: Vec<ChunkCoord>,
    pub placement_grid: PlacementGrid,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
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
            x: (world_pos.x / CHUNK_SIZE).floor() as i32,
            z: (world_pos.z / CHUNK_SIZE).floor() as i32,
        }
    }
    
    pub fn to_world_pos(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * CHUNK_SIZE,
            0.0,
            self.z as f32 * CHUNK_SIZE,
        )
    }
}

#[derive(Default)]
pub struct ChunkData {
    pub entities: Vec<Entity>,
    pub last_updated: f32,
    pub is_loaded: bool,
}

impl ChunkData {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            last_updated: 0.0,
            is_loaded: false,
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
    pub fn new() -> Self {
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
    
    pub fn can_place(&self, position: Vec3, _content_type: ContentType, radius: f32, min_distance: f32) -> bool {
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
    
    pub fn get_nearby_entities(&self, position: Vec3, radius: f32) -> Vec<(Vec3, ContentType, f32)> {
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
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            active_chunks: Vec::new(),
            placement_grid: PlacementGrid::new(),
        }
    }
    
    pub fn get_or_create_chunk(&mut self, coord: ChunkCoord) -> &mut ChunkData {
        self.chunks.entry(coord).or_insert_with(ChunkData::new)
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
        let player_chunk = ChunkCoord::from_world_pos(player_pos);
        let chunk_load_radius = (load_radius / CHUNK_SIZE).ceil() as i32;
        
        // Calculate chunks that should be loaded
        let mut target_chunks = Vec::new();
        for dx in -chunk_load_radius..=chunk_load_radius {
            for dz in -chunk_load_radius..=chunk_load_radius {
                let chunk_coord = ChunkCoord::new(
                    player_chunk.x + dx,
                    player_chunk.z + dz,
                );
                
                let chunk_center = chunk_coord.to_world_pos();
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
                let chunk_center = coord.to_world_pos();
                player_pos.distance(chunk_center) > CHUNK_CLEANUP_RADIUS
            })
            .copied()
            .collect();
        
        for coord in chunks_to_unload {
            self.unload_chunk(coord, commands);
        }
    }
    
    pub fn is_chunk_loaded(&self, coord: ChunkCoord) -> bool {
        self.chunks.get(&coord).map_or(false, |chunk| chunk.is_loaded)
    }
    
    pub fn mark_chunk_loaded(&mut self, coord: ChunkCoord) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.is_loaded = true;
        }
    }
    
    pub fn add_entity_to_chunk(&mut self, entity: Entity, world_pos: Vec3) {
        let coord = ChunkCoord::from_world_pos(world_pos);
        let chunk = self.get_or_create_chunk(coord);
        chunk.add_entity(entity);
    }
    
    pub fn remove_entity_from_chunk(&mut self, entity: Entity, world_pos: Vec3) {
        let coord = ChunkCoord::from_world_pos(world_pos);
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
        for &chunk_coord in world_manager.active_chunks.clone().iter() {
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

pub fn world_pos_to_chunk_coord(world_pos: Vec3) -> ChunkCoord {
    ChunkCoord::from_world_pos(world_pos)
}

pub fn chunk_coord_to_world_pos(coord: ChunkCoord) -> Vec3 {
    coord.to_world_pos()
}

pub fn get_chunk_bounds(coord: ChunkCoord) -> (Vec3, Vec3) {
    let center = coord.to_world_pos();
    let half_size = CHUNK_SIZE * 0.5;
    
    let min = center - Vec3::new(half_size, 0.0, half_size);
    let max = center + Vec3::new(half_size, 0.0, half_size);
    
    (min, max)
}

pub fn is_position_in_chunk(world_pos: Vec3, coord: ChunkCoord) -> bool {
    let (min, max) = get_chunk_bounds(coord);
    
    world_pos.x >= min.x && world_pos.x <= max.x &&
    world_pos.z >= min.z && world_pos.z <= max.z
}
