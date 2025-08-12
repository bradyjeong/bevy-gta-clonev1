use bevy::prelude::*;
use crate::components::ActiveEntity;
use crate::events::world::chunk_events::{
    RequestChunkLoad, RequestChunkUnload, ChunkFinishedLoading,
    ChunkCoord as EventChunkCoord
};
use crate::resources::{ChunkManager, StreamingMetrics};
use crate::world::chunk_coord::ChunkCoord;
use crate::world::chunk_data::{ChunkData, ChunkState};
use crate::world::constants::{UNIFIED_CHUNK_SIZE, UNIFIED_STREAMING_RADIUS};

/// Main unified world streaming system - using focused resources
pub fn unified_world_streaming_system(
    mut chunk_manager: ResMut<ChunkManager>,
    mut streaming_metrics: ResMut<StreamingMetrics>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    time: Res<Time>,
    mut chunk_load_writer: EventWriter<RequestChunkLoad>,
    mut chunk_unload_writer: EventWriter<RequestChunkUnload>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Update timing and reset frame counters
    streaming_metrics.update_timestamp(time.elapsed_secs());
    streaming_metrics.reset_frame_counters();
    
    // Update active chunk
    let current_chunk = ChunkCoord::from_world_pos(active_pos);
    chunk_manager.set_active_chunk(current_chunk);
    
    // Cleanup distant chunks
    let chunks_to_unload = get_chunks_to_unload(&mut chunk_manager, active_pos);
    for coord in chunks_to_unload {
        if !streaming_metrics.can_unload_chunk() {
            break;
        }
        request_chunk_unload(&mut chunk_manager, coord, &mut chunk_unload_writer);
        streaming_metrics.record_chunk_unload();
    }
    
    // Load new chunks
    let chunks_to_load = get_chunks_to_load(&mut chunk_manager, active_pos);
    for coord in chunks_to_load {
        if !streaming_metrics.can_load_chunk() {
            break;
        }
        request_chunk_loading(&mut chunk_manager, coord, &mut chunk_load_writer);
        streaming_metrics.record_chunk_load();
    }
}

/// Get list of chunks that should be unloaded
fn get_chunks_to_unload(chunk_manager: &mut ChunkManager, active_pos: Vec3) -> Vec<ChunkCoord> {
    let mut to_unload = Vec::new();
    let coords: Vec<ChunkCoord> = chunk_manager.loaded_chunks().cloned().collect();
    
    for coord in coords {
        let chunk_pos = coord.to_world_pos();
        let distance = active_pos.distance(chunk_pos);
        
        if distance > UNIFIED_STREAMING_RADIUS as f32 * UNIFIED_CHUNK_SIZE + UNIFIED_CHUNK_SIZE {
            if let Some(chunk) = chunk_manager.get_chunk_mut(&coord) {
                if !matches!(chunk.state, ChunkState::Unloaded | ChunkState::Unloading) {
                    chunk.state = ChunkState::Unloading;
                    chunk.distance_to_player = distance;
                    to_unload.push(coord);
                }
            }
        }
    }
    
    to_unload
}

/// Get list of chunks that should be loaded
fn get_chunks_to_load(chunk_manager: &mut ChunkManager, active_pos: Vec3) -> Vec<ChunkCoord> {
    let active_chunk = ChunkCoord::from_world_pos(active_pos);
    let mut to_load = Vec::new();
    let radius = chunk_manager.streaming_radius_chunks;
    
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let coord = ChunkCoord::new(active_chunk.x + dx, active_chunk.z + dz);
            let distance = active_pos.distance(coord.to_world_pos());
            
            if distance <= UNIFIED_STREAMING_RADIUS as f32 * UNIFIED_CHUNK_SIZE {
                // Check if chunk exists and needs loading
                let needs_loading = if let Some(chunk) = chunk_manager.get_chunk(&coord) {
                    matches!(chunk.state, ChunkState::Unloaded)
                } else {
                    true // Chunk doesn't exist yet
                };
                
                if needs_loading {
                    // Insert or update chunk data
                    if !chunk_manager.is_chunk_loaded(&coord) {
                        let mut chunk_data = ChunkData::new(coord);
                        chunk_data.state = ChunkState::Loading;
                        chunk_data.distance_to_player = distance;
                        chunk_manager.insert_chunk(coord, chunk_data);
                    } else if let Some(chunk) = chunk_manager.get_chunk_mut(&coord) {
                        chunk.state = ChunkState::Loading;
                        chunk.distance_to_player = distance;
                    }
                    to_load.push(coord);
                }
            }
        }
    }
    
    to_load
}

/// Request chunk loading through event system
fn request_chunk_loading(
    chunk_manager: &mut ChunkManager,
    coord: ChunkCoord,
    chunk_load_writer: &mut EventWriter<RequestChunkLoad>,
) {
    if let Some(chunk) = chunk_manager.get_chunk_mut(&coord) {
        // Keep state as Loading until generation completes
        // ChunkFinishedLoading event will transition to Loaded
        chunk.state = ChunkState::Loading;
        
        // Convert internal ChunkCoord to event ChunkCoord and emit request
        let event_coord = EventChunkCoord::new(coord.x, coord.z);
        chunk_load_writer.write(RequestChunkLoad::new(event_coord));
    }
}

/// Request chunk unloading through event system
fn request_chunk_unload(
    chunk_manager: &mut ChunkManager,
    coord: ChunkCoord,
    chunk_unload_writer: &mut EventWriter<RequestChunkUnload>,
) {
    if let Some(mut chunk) = chunk_manager.remove_chunk(&coord) {
        chunk.state = ChunkState::Unloaded;
        chunk.entities.clear();
        chunk.roads_generated = false;
        chunk.buildings_generated = false;
        chunk.vehicles_generated = false;
        chunk.vegetation_generated = false;
        
        // Convert internal ChunkCoord to event ChunkCoord and emit request
        let event_coord = EventChunkCoord::new(coord.x, coord.z);
        chunk_unload_writer.write(RequestChunkUnload::new(event_coord));
        
        // Re-insert as unloaded for potential future loading
        chunk_manager.insert_chunk(coord, chunk);
    }
}

/// Handles ChunkFinishedLoading events to transition chunks from Loading to Loaded state
/// This maintains the event-driven contract: generation systems emit this when complete
pub fn handle_chunk_finished_loading(
    mut chunk_manager: ResMut<ChunkManager>,
    mut finished_events: EventReader<ChunkFinishedLoading>,
) {
    for event in finished_events.read() {
        // Convert event ChunkCoord to internal ChunkCoord
        let coord = ChunkCoord::new(event.coord.x, event.coord.z);
        
        if let Some(chunk) = chunk_manager.get_chunk_mut(&coord) {
            // Only transition if still in Loading state (guard against race conditions)
            if matches!(chunk.state, ChunkState::Loading) {
                chunk.state = ChunkState::Loaded;
                debug!("Chunk {:?} transitioned to Loaded", coord);
            }
        }
    }
}
