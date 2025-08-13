use bevy::prelude::*;
use crate::components::*;
use crate::systems::world::unified_world::{
    UnifiedWorldManager, UnifiedChunkEntity, ContentLayer, ChunkState, UNIFIED_STREAMING_RADIUS,
};

// UNIFIED LOD AND CULLING SYSTEM
// Replaces the separate culling systems with a single, efficient system
// that manages visibility and detail levels for all world content

/// Main LOD system that updates visibility and detail levels for all chunks
pub fn unified_lod_system(
    mut world_manager: ResMut<UnifiedWorldManager>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut chunk_query: Query<(Entity, &UnifiedChunkEntity, &mut Visibility)>,
    mut cullable_query: Query<(&mut Cullable, &mut Visibility), Without<UnifiedChunkEntity>>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Update chunk LOD levels based on distance
    update_chunk_lod_levels(&mut world_manager, active_pos);
    
    // Update chunk entity visibility
    for (entity, chunk_entity, mut visibility) in chunk_query.iter_mut() {
        if let Some(chunk) = world_manager.get_chunk(chunk_entity.coord) {
            let should_be_visible = match chunk.state {
                ChunkState::Loaded { lod_level } => {
                    should_layer_be_visible(chunk_entity.layer, lod_level, chunk.distance_to_player)
                }
                _ => false,
            };
            
            *visibility = if should_be_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
    
    // Update individual entity culling
    for (mut cullable, mut visibility) in cullable_query.iter_mut() {
        update_cullable_entity(&mut cullable, &mut visibility, active_pos);
    }
}

fn update_chunk_lod_levels(world_manager: &mut UnifiedWorldManager, active_pos: Vec3) {
    let chunks_to_update: Vec<_> = world_manager.chunks.iter()
        .filter_map(|(coord, chunk)| {
            if let ChunkState::Loaded { lod_level } = chunk.state {
                let distance = active_pos.distance(chunk.coord.to_world_pos());
                Some((*coord, distance, lod_level))
            } else {
                None
            }
        })
        .collect();
    
    for (coord, distance, old_lod) in chunks_to_update {
        let new_lod = world_manager.calculate_lod_level(distance);
        if new_lod != old_lod {
            if let Some(chunk) = world_manager.chunks.get_mut(&coord) {
                chunk.distance_to_player = distance;
                chunk.state = ChunkState::Loaded { lod_level: new_lod };
            }
        }
    }
}

fn should_layer_be_visible(layer: ContentLayer, lod_level: usize, distance: f32) -> bool {
    match layer {
        ContentLayer::Roads => {
            // Roads always visible at all LOD levels
            true
        }
        ContentLayer::Buildings => {
            // Buildings visible up to LOD 2
            lod_level <= 2
        }
        ContentLayer::Vehicles => {
            // Vehicles only visible at close range (LOD 0-1)
            lod_level <= 1 && distance <= 400.0
        }
        ContentLayer::Vegetation => {
            // Vegetation only at highest detail (LOD 0)
            lod_level == 0 && distance <= 200.0
        }
        ContentLayer::NPCs => {
            // NPCs only at very close range
            lod_level == 0 && distance <= 150.0
        }
    }
}

fn update_cullable_entity(
    cullable: &mut Cullable,
    visibility: &mut Visibility,
    active_pos: Vec3,
) {
    // This assumes the cullable entity has a Transform component
    // In a real implementation, you'd need to get the entity's transform
    // For now, we'll just use the max_distance from Cullable
    
    let was_culled = cullable.is_culled;
    
    // For this implementation, we assume the cullable distance check
    // is handled elsewhere. This system focuses on the unified approach.
    
    if cullable.is_culled && !was_culled {
        *visibility = Visibility::Hidden;
    } else if !cullable.is_culled && was_culled {
        *visibility = Visibility::Visible;
    }
}

/// Performance monitoring system for the unified LOD
pub fn unified_lod_performance_monitor(
    world_manager: Res<UnifiedWorldManager>,
    chunk_query: Query<&UnifiedChunkEntity>,
    cullable_query: Query<&Cullable>,
    mut performance_stats: ResMut<PerformanceStats>,
) {
    // Count entities by layer and visibility
    let mut layer_counts = [0; 5]; // Roads, Buildings, Vehicles, Vegetation, NPCs
    let mut total_chunks = 0;
    let mut loaded_chunks = 0;
    
    for chunk in world_manager.chunks.values() {
        total_chunks += 1;
        if matches!(chunk.state, ChunkState::Loaded { .. }) {
            loaded_chunks += 1;
        }
    }
    
    for chunk_entity in chunk_query.iter() {
        let layer_index = match chunk_entity.layer {
            ContentLayer::Roads => 0,
            ContentLayer::Buildings => 1,
            ContentLayer::Vehicles => 2,
            ContentLayer::Vegetation => 3,
            ContentLayer::NPCs => 4,
        };
        layer_counts[layer_index] += 1;
    }
    
    let culled_entities = cullable_query.iter().filter(|c| c.is_culled).count();
    let total_cullable = cullable_query.iter().count();
    
    performance_stats.entity_count = layer_counts.iter().sum::<usize>() + total_cullable;
    performance_stats.culled_entities = culled_entities;
    
    // Note: PerformanceStats doesn't have active_chunks, chunks_loaded, chunks_unloaded fields
    // These would need to be added to the PerformanceStats struct if needed
}

/// System to handle dynamic LOD adjustments based on performance
pub fn adaptive_lod_system(
    mut world_manager: ResMut<UnifiedWorldManager>,
    performance_stats: Res<PerformanceStats>,
    time: Res<Time>,
) {
    // Simple adaptive LOD based on frame time
    let frame_time = time.delta_secs();
    let target_frame_time = 1.0 / 60.0; // 60 FPS target
    
    if frame_time > target_frame_time * 1.5 {
        // Performance is suffering, reduce max chunks per frame
        world_manager.max_chunks_per_frame = (world_manager.max_chunks_per_frame.saturating_sub(1)).max(1);
    } else if frame_time < target_frame_time * 0.8 {
        // Performance is good, can increase load
        world_manager.max_chunks_per_frame = (world_manager.max_chunks_per_frame + 1).min(8);
    }
}

/// Unified culling system that replaces the old distance_culling_system
pub fn unified_distance_culling_system(
    mut cullable_query: Query<(&mut Cullable, &Transform, &mut Visibility)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<Cullable>)>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    for (mut cullable, transform, mut visibility) in cullable_query.iter_mut() {
        let distance = active_pos.distance(transform.translation);
        let should_be_culled = distance > cullable.max_distance;
        
        if should_be_culled != cullable.is_culled {
            cullable.is_culled = should_be_culled;
            *visibility = if should_be_culled {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}

/// System to clean up entities that have been culled for too long
pub fn unified_cleanup_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    cullable_query: Query<(Entity, &Cullable, &Transform)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    let cleanup_delay = 30.0; // Clean up entities culled for 30+ seconds
    
    for (entity, cullable, transform) in cullable_query.iter() {
        if cullable.is_culled {
            // In a full implementation, you'd track when entities were first culled
            // For now, we'll just clean up very distant entities immediately
            let distance_to_any_chunk = world_manager
                .chunks
                .values()
                .map(|chunk| transform.translation.distance(chunk.coord.to_world_pos()))
                .fold(f32::INFINITY, f32::min);
            
            if distance_to_any_chunk > UNIFIED_STREAMING_RADIUS * 2.0 {
                commands.entity(entity).despawn();
                
                // Remove from placement grid
                // Note: In a full implementation, you'd need to track which entities
                // are in the placement grid to remove them efficiently
            }
        }
    }
}
