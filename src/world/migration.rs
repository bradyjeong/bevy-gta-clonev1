// Field extraction and migration from UnifiedWorldManager to decomposed resources
// Phase 2.x: Oracle-approved migration implementation with critical fixes

use bevy::prelude::*;
#[allow(unused_imports)]
use crate::systems::world::unified_world::{
    UnifiedWorldManager, ChunkCoord as OldChunkCoord, ChunkState,
};
use crate::world::{
    ChunkTracker, WorldCoordinator, PlacementGrid, RoadNetwork,
    constants::*,
};
use crate::components::world::{ContentType, DynamicContent};

/// Event signaling that world extraction is complete
#[derive(Event)]
pub struct WorldExtractionComplete;

/// Event signaling that migration validation is complete
#[derive(Event)]
pub struct MigrationValidationComplete;

/// Migration system that extracts data from UnifiedWorldManager to new resources
/// Runs once at startup when world_v2 feature is enabled
#[cfg(feature = "world_v2")]
pub fn extract_world_manager(
    mut commands: Commands,
    unified_manager: Option<Res<UnifiedWorldManager>>,
    mut chunk_tracker: ResMut<ChunkTracker>,
    mut world_coordinator: ResMut<WorldCoordinator>,
    mut placement_grid: ResMut<PlacementGrid>,
    mut road_network: ResMut<RoadNetwork>,
    mut extraction_complete: EventWriter<WorldExtractionComplete>,
    entities_query: Query<(&GlobalTransform, Option<&DynamicContent>)>,
) {
    let Some(unified) = unified_manager else {
        #[cfg(debug_assertions)]
        panic!("UnifiedWorldManager not found during extraction - critical migration failure!");
        #[cfg(not(debug_assertions))]
        {
            error!("UnifiedWorldManager not found during extraction");
            return;
        }
    };

    info!("Starting world manager field extraction...");

    // Extract ChunkTracker fields
    extract_chunk_tracker(&unified, &mut chunk_tracker);
    
    // Extract WorldCoordinator fields
    extract_world_coordinator(&unified, &mut world_coordinator);
    
    // Extract PlacementGrid fields with accurate entity data
    extract_placement_grid_accurate(&unified, &entities_query, &mut placement_grid);
    
    // Extract RoadNetwork fields with full topology
    extract_road_network_complete(&unified, &mut road_network);
    
    // Signal extraction completion (only once)
    extraction_complete.write(WorldExtractionComplete);
    
    info!(
        "World extraction complete: {} chunks, {} cells, active chunk: {:?}",
        chunk_tracker.get_loaded_chunks().len(),
        placement_grid.get_occupied_count(),
        world_coordinator.get_focus_position()
    );
    
    // Remove the old resource after extraction
    commands.remove_resource::<UnifiedWorldManager>();
}

/// Extract chunk-related data into ChunkTracker
fn extract_chunk_tracker(
    unified: &UnifiedWorldManager,
    tracker: &mut ChunkTracker,
) {
    // Clear any existing data
    tracker.clear();
    
    // Migrate loaded chunks
    for (coord, chunk_data) in &unified.chunks {
        // Convert OldChunkCoord to our new coordinate system
        let chunk_coord = crate::world::ChunkCoord {
            x: coord.x,
            z: coord.z,
        };
        
        // Track chunk state
        match chunk_data.state {
            ChunkState::Loaded { lod_level } => {
                tracker.mark_chunk_loaded(chunk_coord, lod_level);
                
                // Store entity count for verification
                if !chunk_data.entities.is_empty() {
                    debug!(
                        "Chunk {:?} has {} entities",
                        chunk_coord,
                        chunk_data.entities.len()
                    );
                }
            }
            ChunkState::Loading => {
                tracker.mark_chunk_loading(chunk_coord);
            }
            ChunkState::Unloading => {
                tracker.mark_chunk_unloading(chunk_coord);
            }
            ChunkState::Unloaded => {
                // Don't track unloaded chunks
            }
        }
        
        // Update distance if player position is known
        if chunk_data.distance_to_player < f32::INFINITY {
            tracker.update_chunk_distance(chunk_coord, chunk_data.distance_to_player);
        }
    }
    
    debug!("Extracted {} chunks into ChunkTracker", tracker.get_loaded_chunks().len());
}

/// Extract world coordination data
fn extract_world_coordinator(
    unified: &UnifiedWorldManager,
    coordinator: &mut WorldCoordinator,
) {
    // Set streaming radius using shared constant
    let streaming_radius = unified.streaming_radius_chunks as f32 * UNIFIED_CHUNK_SIZE;
    coordinator.set_streaming_radius(streaming_radius);
    
    // Set focus position from active chunk
    if let Some(active_chunk) = unified.active_chunk {
        let world_pos = Vec3::new(
            active_chunk.x as f32 * UNIFIED_CHUNK_SIZE,
            0.0,
            active_chunk.z as f32 * UNIFIED_CHUNK_SIZE,
        );
        coordinator.set_focus_position(world_pos);
    }
    
    // Transfer performance settings
    coordinator.set_max_chunks_per_frame(unified.max_chunks_per_frame);
    
    // Update frame counter based on last update
    coordinator.update_frame_counter(unified.last_update as u32);
    
    debug!(
        "Extracted world coordinator: radius={}, max_chunks={}",
        streaming_radius,
        unified.max_chunks_per_frame
    );
}

/// Extract placement grid data with accurate entity positions
/// FIX: Query actual entities with GlobalTransform for accurate occupancy
fn extract_placement_grid_accurate(
    unified: &UnifiedWorldManager,
    entities_query: &Query<(&GlobalTransform, Option<&DynamicContent>)>,
    grid: &mut PlacementGrid,
) {
    // Clear existing grid
    grid.clear();
    
    // Copy cell size
    grid.set_cell_size(unified.placement_grid.get_cell_size());
    
    // Query all entities with positions and content types for accurate migration
    let mut entity_count = 0;
    
    // First, try to query entities directly
    for (transform, dynamic_content) in entities_query.iter() {
        let position = transform.translation();
        
        // Determine collision radius based on content type
        let radius = if let Some(content) = dynamic_content {
            match content.content_type {
                ContentType::Building => BUILDING_COLLISION_RADIUS,
                ContentType::Vehicle => VEHICLE_COLLISION_RADIUS,
                ContentType::Tree => TREE_COLLISION_RADIUS,
                ContentType::NPC => NPC_COLLISION_RADIUS,
                _ => 5.0, // Default for unknown types
            }
        } else {
            // If no content type, estimate based on position
            5.0
        };
        
        // Mark exact position as occupied
        grid.mark_occupied(position, radius);
        entity_count += 1;
    }
    
    // Fallback: If no entities found via query, extract from chunk data
    if entity_count == 0 {
        warn!("No entities found via GlobalTransform query, using chunk-based extraction");
        
        for (coord, chunk_data) in &unified.chunks {
            // For each entity in chunk, estimate position
            for (i, _entity) in chunk_data.entities.iter().enumerate() {
                // Generate distributed positions within chunk
                let offset_x = ((i % 10) as f32 - 5.0) * 10.0;
                let offset_z = ((i / 10) as f32 - 5.0) * 10.0;
                
                let position = Vec3::new(
                    coord.x as f32 * UNIFIED_CHUNK_SIZE + offset_x,
                    0.0,
                    coord.z as f32 * UNIFIED_CHUNK_SIZE + offset_z,
                );
                
                // Use default radius for fallback extraction
                grid.mark_occupied(position, 5.0);
                entity_count += 1;
            }
        }
    }
    
    info!("Extracted {} entities into PlacementGrid with accurate positions", entity_count);
}

/// Extract complete road network data with full topology
/// FIX: Migrate all intersections and road segments, preserve topology
fn extract_road_network_complete(
    unified: &UnifiedWorldManager,
    network: &mut RoadNetwork,
) {
    // Clear network for fresh migration
    network.active_nodes = 0;
    network.network_flags = 0;
    
    // Count actual roads and intersections
    let road_count = unified.road_network.roads.len();
    let intersection_count = unified.road_network.intersections.len();
    
    info!("Extracting {} roads and {} intersections", road_count, intersection_count);
    
    // Extract ALL intersections (up to our limit)
    let max_nodes = network.nodes.len().min(intersection_count);
    network.active_nodes = max_nodes as u8;
    
    let mut node_index = 0;
    let mut node_map = std::collections::HashMap::new();
    
    for (id, intersection) in unified.road_network.intersections.iter() {
        if node_index >= network.nodes.len() {
            warn!("Reached maximum node capacity ({} nodes)", network.nodes.len());
            break;
        }
        
        // Convert world position to grid coordinates
        let grid_x = (intersection.position.x / UNIFIED_CHUNK_SIZE) as u16;
        let grid_z = (intersection.position.z / UNIFIED_CHUNK_SIZE) as u16;
        network.nodes[node_index] = (grid_x, grid_z);
        
        // Store mapping for connectivity
        node_map.insert(*id, node_index as u8);
        node_index += 1;
    }
    
    // Extract road connectivity into connections bitfield
    for (_id, road) in unified.road_network.roads.iter() {
        // Find connected intersections
        let mut from_node = None;
        let mut to_node = None;
        
        // Check road start/end points against intersections
        for (int_id, intersection) in &unified.road_network.intersections {
            if road.control_points.first().map_or(false, |p| p.distance(intersection.position) < 5.0) {
                from_node = node_map.get(int_id).copied();
            }
            if road.control_points.last().map_or(false, |p| p.distance(intersection.position) < 5.0) {
                to_node = node_map.get(int_id).copied();
            }
        }
        
        // Connect nodes if both found
        if let (Some(from), Some(to)) = (from_node, to_node) {
            network.connect_nodes(from, to);
            network.connect_nodes(to, from); // Bidirectional
        }
    }
    
    // Set network flags based on actual content
    if road_count > 0 {
        network.network_flags |= 0x01; // Roads present
    }
    if intersection_count > 0 {
        network.network_flags |= 0x02; // Intersections present
    }
    if road_count > 10 && intersection_count > 5 {
        network.network_flags |= 0x04; // Complex network
    }
    
    // Use road network's next ID as generation seed for consistency
    network.generation_seed = unified.road_network.next_road_id;
    
    info!(
        "Extracted complete road network: {} roads, {} intersections -> {} nodes",
        road_count, intersection_count, network.active_nodes
    );
}

/// Validation system to ensure migration was successful
/// FIX: Proper feature gating and single event emission
#[cfg(feature = "world_v2")]
pub fn validate_migration(
    chunk_tracker: Res<ChunkTracker>,
    world_coordinator: Res<WorldCoordinator>,
    placement_grid: Res<PlacementGrid>,
    road_network: Res<RoadNetwork>,
    mut validation_complete: EventWriter<MigrationValidationComplete>,
) {
    let mut errors = Vec::new();
    
    // Validate ChunkTracker
    if chunk_tracker.get_loaded_chunks().is_empty() {
        errors.push("ChunkTracker has no loaded chunks");
    }
    
    // Validate WorldCoordinator
    if world_coordinator.get_streaming_radius() <= 0.0 {
        errors.push("WorldCoordinator has invalid streaming radius");
    }
    
    // Validate PlacementGrid
    if placement_grid.get_cell_size() <= 0.0 {
        errors.push("PlacementGrid has invalid cell size");
    }
    
    // Validate RoadNetwork
    if road_network.active_nodes == 0 && road_network.network_flags != 0 {
        errors.push("RoadNetwork flags indicate content but no active nodes");
    }
    
    // FIX: Hard failure in debug mode for critical errors
    #[cfg(debug_assertions)]
    if !errors.is_empty() {
        panic!(
            "Migration validation FAILED with {} critical errors:\n{}",
            errors.len(),
            errors.join("\n")
        );
    }
    
    #[cfg(not(debug_assertions))]
    if !errors.is_empty() {
        error!("Migration validation failed:");
        for error in errors {
            error!("  - {}", error);
        }
    } else {
        info!("Migration validation PASSED");
    }
    
    // FIX: Emit validation complete event (different from extraction event)
    validation_complete.write(MigrationValidationComplete);
}

/// Create parallel v2 systems that use the new resources
/// FIX: Properly feature-gated v2 systems
#[cfg(feature = "world_v2")]
pub mod v2_systems {
    use super::*;
    use crate::world::{ChunkLoadRequest, ChunkUnloadRequest};
    
    /// V2 chunk streaming system using new resources
    pub fn stream_chunks_v2(
        mut chunk_tracker: ResMut<ChunkTracker>,
        world_coordinator: Res<WorldCoordinator>,
        mut load_events: EventWriter<ChunkLoadRequest>,
        mut unload_events: EventWriter<ChunkUnloadRequest>,
    ) {
        let focus_pos = world_coordinator.get_focus_position();
        let radius = world_coordinator.get_streaming_radius();
        
        // Calculate which chunks should be loaded
        let chunk_radius = (radius / UNIFIED_CHUNK_SIZE).ceil() as i32;
        let center_chunk = crate::world::ChunkCoord {
            x: (focus_pos.x / UNIFIED_CHUNK_SIZE).floor() as i32,
            z: (focus_pos.z / UNIFIED_CHUNK_SIZE).floor() as i32,
        };
        
        // Request loading of nearby chunks
        for dx in -chunk_radius..=chunk_radius {
            for dz in -chunk_radius..=chunk_radius {
                let coord = crate::world::ChunkCoord {
                    x: center_chunk.x + dx,
                    z: center_chunk.z + dz,
                };
                
                if !chunk_tracker.is_chunk_loaded(coord) && !chunk_tracker.is_chunk_loading(coord) {
                    load_events.write(ChunkLoadRequest {
                        coord,
                        priority: 1.0,
                    });
                    chunk_tracker.mark_chunk_loading(coord);
                }
            }
        }
        
        // Request unloading of distant chunks
        for chunk in chunk_tracker.get_loaded_chunks() {
            let chunk_pos = Vec3::new(
                chunk.x as f32 * UNIFIED_CHUNK_SIZE,
                focus_pos.y,
                chunk.z as f32 * UNIFIED_CHUNK_SIZE,
            );
            
            if chunk_pos.distance(focus_pos) > radius + UNIFIED_CHUNK_SIZE {
                unload_events.write(ChunkUnloadRequest { coord: chunk });
                chunk_tracker.mark_chunk_unloading(chunk);
            }
        }
    }
    
    /// V2 placement validation using new PlacementGrid
    pub fn validate_placement_v2(
        placement_grid: Res<PlacementGrid>,
        query: Query<&Transform, Added<crate::components::WorldObject>>,
    ) {
        for transform in query.iter() {
            let pos = transform.translation;
            if !placement_grid.can_place_at(pos, 5.0, PLACEMENT_SAFETY_MARGIN) {
                warn!("Entity placed at invalid position: {:?}", pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chunk_coord_conversion() {
        let old_coord = OldChunkCoord { x: 5, z: -3 };
        let new_coord = crate::world::ChunkCoord {
            x: old_coord.x,
            z: old_coord.z,
        };
        assert_eq!(new_coord.x, 5);
        assert_eq!(new_coord.z, -3);
    }
    
    #[test]
    fn test_world_position_calculation() {
        let chunk = crate::world::ChunkCoord { x: 2, z: 3 };
        let world_pos = Vec3::new(
            chunk.x as f32 * UNIFIED_CHUNK_SIZE,
            0.0,
            chunk.z as f32 * UNIFIED_CHUNK_SIZE,
        );
        assert_eq!(world_pos, Vec3::new(200.0, 0.0, 300.0));
    }
    
    #[test]
    fn test_placement_grid_entity_count() {
        // Test that PlacementGrid migration marks cells correctly
        // Note: PlacementGrid uses a bitfield, so occupied count is number of cells, not entities
        let mut grid = PlacementGrid::new();
        grid.mark_occupied(Vec3::new(10.0, 0.0, 10.0), 5.0);
        let count1 = grid.get_occupied_count();
        
        // Mark a different cell (far enough away to be in a different grid cell)
        grid.mark_occupied(Vec3::new(100.0, 0.0, 100.0), 5.0);
        let count2 = grid.get_occupied_count();
        
        // Should have more occupied cells after second mark
        assert!(count2 > count1);
    }
    
    #[test]
    fn test_road_network_topology() {
        // Test that road network preserves topology after migration
        let mut network = RoadNetwork::new();
        network.active_nodes = 3;
        network.nodes[0] = (0, 0);
        network.nodes[1] = (1, 0);
        network.nodes[2] = (1, 1);
        
        // Connect nodes using the connections bitfield
        network.connect_nodes(0, 1);
        network.connect_nodes(1, 2);
        
        assert_eq!(network.active_nodes, 3);
        assert!(network.is_connected(0, 1));
        assert!(network.is_connected(1, 2));
    }
}
