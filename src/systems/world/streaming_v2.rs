use bevy::prelude::*;
use crate::components::*;
use crate::events::world::chunk_events::{RequestChunkLoad, RequestChunkUnload, ChunkCoord as EventChunkCoord};
use crate::world::{ChunkTracker, ChunkTables, PlacementGrid, RoadNetwork, WorldCoordinator};
use crate::world::chunk_coord::ChunkCoord;
use crate::world::chunk_data::ChunkState;
use crate::world::constants::{UNIFIED_CHUNK_SIZE, UNIFIED_STREAMING_RADIUS};

/// V2 unified world streaming system using decomposed resources
pub fn unified_world_streaming_system_v2(
    mut tracker: ResMut<ChunkTracker>,
    mut tables: ResMut<ChunkTables>,
    mut coordinator: ResMut<WorldCoordinator>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    _time: Res<Time>,
    mut chunk_load_writer: EventWriter<RequestChunkLoad>,
    mut chunk_unload_writer: EventWriter<RequestChunkUnload>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Update timing - using generation frame counter
    coordinator.generation_frame += 1;
    
    // Track chunks loaded/unloaded this frame using local variables
    let mut chunks_loaded_this_frame = 0;
    let mut chunks_unloaded_this_frame = 0;
    
    // Update active chunk - track in tracker instead
    let _current_chunk = ChunkCoord::from_world_pos(active_pos);
    coordinator.update_focus(active_pos);
    
    // Cleanup distant chunks
    let chunks_to_unload = cleanup_distant_chunks_v2(&mut tracker, &mut tables, active_pos);
    let max_chunks_per_frame = coordinator.get_max_chunks_per_frame().max(4);
    for coord in chunks_to_unload {
        if chunks_unloaded_this_frame >= max_chunks_per_frame {
            break;
        }
        request_chunk_unload_v2(&mut tracker, &mut tables, coord, &mut chunk_unload_writer);
        chunks_unloaded_this_frame += 1;
    }
    
    // Load new chunks
    let streaming_radius_chunks = (coordinator.streaming_radius / UNIFIED_CHUNK_SIZE).ceil() as i32;
    let chunks_to_load = get_chunks_to_load_v2(&mut tracker, &mut tables, active_pos, streaming_radius_chunks);
    
    if !chunks_to_load.is_empty() {
        debug!("Loading {} new chunks", chunks_to_load.len());
    }
    
    for coord in chunks_to_load {
        if chunks_loaded_this_frame >= max_chunks_per_frame {
            break;
        }
        request_chunk_loading_v2(&mut tracker, &mut tables, coord, &mut chunk_load_writer);
        chunks_loaded_this_frame += 1;
    }
}

fn cleanup_distant_chunks_v2(tracker: &mut ChunkTracker, tables: &mut ChunkTables, active_pos: Vec3) -> Vec<ChunkCoord> {
    let mut to_unload = Vec::new();
    
    // Clone to avoid borrow issues
    let loaded_chunks: Vec<ChunkCoord> = tables.loaded.keys().cloned().collect();
    
    for coord in loaded_chunks {
        let distance = active_pos.distance(coord.to_world_pos());
        
        if distance > UNIFIED_STREAMING_RADIUS as f32 * UNIFIED_CHUNK_SIZE + UNIFIED_CHUNK_SIZE {
            // Mark for unloading if not already marked
            if !tables.unloading.contains_key(&coord) {
                tables.unloading.insert(coord, ChunkState::Unloading);
                to_unload.push(coord);
            }
        }
    }
    
    // Update tracker's loaded_chunks array
    tracker.cleanup_distant_chunks(ChunkCoord::from_world_pos(active_pos), UNIFIED_STREAMING_RADIUS as i16);
    
    to_unload
}

fn get_chunks_to_load_v2(
    _tracker: &mut ChunkTracker,
    tables: &mut ChunkTables,
    active_pos: Vec3,
    streaming_radius_chunks: i32,
) -> Vec<ChunkCoord> {
    let active_chunk = ChunkCoord::from_world_pos(active_pos);
    let mut to_load = Vec::new();
    
    for dx in -streaming_radius_chunks..=streaming_radius_chunks {
        for dz in -streaming_radius_chunks..=streaming_radius_chunks {
            let coord = ChunkCoord::new(active_chunk.x + dx, active_chunk.z + dz);
            let distance = active_pos.distance(coord.to_world_pos());
            
            if distance <= UNIFIED_STREAMING_RADIUS as f32 * UNIFIED_CHUNK_SIZE {
                // Check if chunk is not already loaded or loading
                if !tables.loaded.contains_key(&coord) && !tables.loading.contains_key(&coord) {
                    tables.loading.insert(coord, ChunkState::Loading);
                    to_load.push(coord);
                }
            }
        }
    }
    
    to_load
}

fn request_chunk_loading_v2(
    tracker: &mut ChunkTracker,
    tables: &mut ChunkTables,
    coord: ChunkCoord,
    chunk_load_writer: &mut EventWriter<RequestChunkLoad>,
) {
    // Mark as loading in tables
    tables.loading.insert(coord, ChunkState::Loading);
    
    // Update tracker's loaded_chunks array
    tracker.mark_loading(coord);
    
    // Convert internal ChunkCoord to event ChunkCoord and emit request
    let event_coord = EventChunkCoord::new(coord.x, coord.z);
    chunk_load_writer.write(RequestChunkLoad::new(event_coord));
}

fn request_chunk_unload_v2(
    tracker: &mut ChunkTracker,
    tables: &mut ChunkTables,
    coord: ChunkCoord,
    chunk_unload_writer: &mut EventWriter<RequestChunkUnload>,
) {
    // Mark as unloading
    tables.unloading.insert(coord, ChunkState::Unloading);
    
    // Update tracker's loaded_chunks array
    tracker.mark_chunk_unloading(coord);
    
    // Convert internal ChunkCoord to event ChunkCoord and emit request
    let event_coord = EventChunkCoord::new(coord.x, coord.z);
    chunk_unload_writer.write(RequestChunkUnload::new(event_coord));
    
    // Remove from loaded and loading maps
    tables.loaded.remove(&coord);
    tables.loading.remove(&coord);
}

/// V2 chunk state query system
pub fn update_chunk_states_v2(
    mut tracker: ResMut<ChunkTracker>,
    mut tables: ResMut<ChunkTables>,
    active_query: Query<&Transform, With<ActiveEntity>>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Update tracker focus chunk for hot-path access
    tracker.focus_chunk = ChunkCoord::from_world_pos(active_pos);
    tracker.focus_valid = true;
    
    // Update distances for all loaded chunks in tables (less frequent access)
    let loaded_coords: Vec<ChunkCoord> = tables.loaded.keys().cloned().collect();
    for coord in loaded_coords {
        let distance = active_pos.distance(coord.to_world_pos());
        tables.distances.insert(coord, distance);
    }
}

/// V2 placement validation system
pub fn validate_placement_system_v2(
    placement_grid: Res<PlacementGrid>,
    mut validation_query: Query<(&Transform, &mut PlacementValidation), Added<PlacementValidation>>,
) {
    for (transform, mut validation) in validation_query.iter_mut() {
        let position = transform.translation;
        let radius = validation.radius;
        
        // Check for collisions using placement grid
        validation.is_valid = !placement_grid.check_collision(position, radius);
    }
}

/// V2 spawn position finder
pub fn find_valid_spawn_position_v2(
    placement_grid: Res<PlacementGrid>,
    mut spawn_requests: Query<(&mut Transform, &SpawnRequest), Added<SpawnRequest>>,
) {
    for (mut transform, request) in spawn_requests.iter_mut() {
        let base_position = transform.translation;
        let radius = request.spawn_radius;
        
        // Try to find a valid position near the base position
        if let Some(valid_pos) = placement_grid.find_free_position(base_position, radius, 100.0) {
            transform.translation = valid_pos;
        }
    }
}

/// V2 road pathfinding system
pub fn pathfinding_system_v2(
    road_network: Res<RoadNetwork>,
    mut path_requests: Query<(&Transform, &mut PathRequest), Changed<PathRequest>>,
) {
    for (transform, mut request) in path_requests.iter_mut() {
        let start = transform.translation;
        let end = request.target_position;
        
        // Find path using road network
        if let Some(path) = road_network.find_path(start, end) {
            request.calculated_path = Some(path);
            request.status = PathRequestStatus::Success;
        } else {
            request.status = PathRequestStatus::Failed;
        }
    }
}

/// V2 road validation system
pub fn road_validation_system_v2(
    road_network: Res<RoadNetwork>,
    mut road_queries: Query<(&Transform, &mut RoadValidation), Added<RoadValidation>>,
) {
    for (transform, mut validation) in road_queries.iter_mut() {
        let position = transform.translation;
        
        // Check if position is near a road
        validation.is_near_road = road_network.is_near_road(position, validation.max_distance);
        validation.nearest_road_point = road_network.get_nearest_road_point(position);
    }
}

// Placeholder components for validation systems
#[derive(Component, Default)]
pub struct PlacementValidation {
    pub radius: f32,
    pub is_valid: bool,
}

#[derive(Component)]
pub struct SpawnRequest {
    pub spawn_radius: f32,
}

#[derive(Component)]
pub struct PathRequest {
    pub target_position: Vec3,
    pub calculated_path: Option<Vec<Vec3>>,
    pub status: PathRequestStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathRequestStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Component)]
pub struct RoadValidation {
    pub max_distance: f32,
    pub is_near_road: bool,
    pub nearest_road_point: Option<Vec3>,
}
