//! ───────────────────────────────────────────────
//! System:   Unified Lod
//! Purpose:  Handles entity movement and physics
//! Schedule: Update
//! Reads:    `VehicleRendering`, `VehicleState`, `ActiveEntity`, `PerformanceStats`, Transform
//! Writes:   `VehicleState`, `PerformanceStats`, `NPCState`, Visibility, `MasterLODCoordinator`
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;
use crate::systems::world::unified_world::{
    UnifiedWorldManager, ContentLayer, UNIFIED_STREAMING_RADIUS, ChunkCoordExt,
};
use game_core::prelude::{UnifiedChunkEntity, ChunkState};
use crate::systems::distance_cache::{DistanceCache, get_cached_distance};

// MASTER UNIFIED LOD AND CULLING SYSTEM
// Consolidates all LOD systems into a single, efficient pipeline
// Manages visibility and detail levels for all world content with entity-type plugins

/// Master LOD coordination resource
#[derive(Resource, Default)]
pub struct MasterLODCoordinator {
    pub dirty_entities: HashMap<Entity, LODDirtyReason>,
    pub lod_plugin_configs: HashMap<EntityType, LODPluginConfig>,
    pub performance_stats: LODPerformanceStats,
    pub frame_counter: u64,
}

/// Why an entity was marked dirty for LOD update
#[derive(Debug, Clone, Copy)]
pub enum LODDirtyReason {
    Movement(f32), // Distance moved
    TimeInterval,  // Periodic update
    PlayerMoved,   // Player position changed significantly
    StateChange,   // Entity state changed
}

/// Entity type for LOD plugin system
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EntityType {
    Vehicle,
    NPC,
    Vegetation,
    Building,
    Chunk,
}

/// Configuration for entity-type specific LOD plugins
#[derive(Debug, Clone)]
pub struct LODPluginConfig {
    pub distances: Vec<f32>,         // LOD level distances
    pub cull_distance: f32,          // Distance at which to cull completely
    pub hysteresis: f32,             // Hysteresis to prevent flickering
    pub update_interval: f32,        // How often to check (seconds)
    pub priority_distance: f32,      // Distance threshold for high priority updates
}

/// Performance statistics for LOD system
#[derive(Debug, Default)]
pub struct LODPerformanceStats {
    pub entities_processed: HashMap<EntityType, usize>,
    pub processing_times: HashMap<EntityType, f32>,
    pub lod_level_counts: HashMap<(EntityType, usize), usize>,
    pub total_entities: usize,
    pub culled_entities: usize,
}

impl LODPluginConfig {
    #[must_use] pub fn vehicle() -> Self {
        Self {
            distances: vec![50.0, 150.0, 300.0],
            cull_distance: 500.0,
            hysteresis: 5.0,
            update_interval: 0.5,
            priority_distance: 100.0,
        }
    }

    #[must_use] pub fn npc() -> Self {
        Self {
            distances: vec![25.0, 75.0, 100.0],
            cull_distance: 150.0,
            hysteresis: 3.0,
            update_interval: 0.3,
            priority_distance: 50.0,
        }
    }

    #[must_use] pub fn vegetation() -> Self {
        Self {
            distances: vec![50.0, 150.0, 300.0],
            cull_distance: 400.0,
            hysteresis: 10.0,
            update_interval: 1.0,
            priority_distance: 100.0,
        }
    }

    #[must_use] pub fn building() -> Self {
        Self {
            distances: vec![100.0, 300.0, 500.0],
            cull_distance: 800.0,
            hysteresis: 15.0,
            update_interval: 0.8,
            priority_distance: 200.0,
        }
    }

    #[must_use] pub fn chunk() -> Self {
        Self {
            distances: vec![150.0, 300.0, 500.0],
            cull_distance: 800.0,
            hysteresis: 20.0,
            update_interval: 0.5,
            priority_distance: 300.0,
        }
    }

    #[must_use] pub fn get_lod_level(&self, distance: f32) -> usize {
        for (level, &threshold) in self.distances.iter().enumerate() {
            if distance <= threshold + self.hysteresis {
                return level;
            }
        }
        self.distances.len() // Beyond all LOD levels
    }

    #[must_use] pub fn should_cull(&self, distance: f32) -> bool {
        distance > self.cull_distance + self.hysteresis
    }
}

/// Main unified LOD system - coordinates all entity-type LOD plugins
pub fn master_unified_lod_system(
    mut commands: Commands,
    mut lod_coordinator: ResMut<MasterLODCoordinator>,
    mut world_manager: ResMut<UnifiedWorldManager>,
    active_query: Query<(Entity, &Transform), With<ActiveEntity>>,
    // Entity-specific queries - these replace individual LOD systems
    mut vehicle_query: Query<(Entity, &mut VehicleState, &Transform, Option<&VehicleRendering>), Without<ActiveEntity>>,
    mut npc_query: Query<(Entity, &mut NPCState, &Transform, Option<&NPCRendering>), (Without<ActiveEntity>, Without<VehicleState>)>,
    mut visibility_param_set: ParamSet<(
        Query<(Entity, &UnifiedChunkEntity, &mut Visibility)>,
        Query<(Entity, &mut VegetationLOD, &Transform, &mut Visibility, &mut Mesh3d), (With<VegetationMeshLOD>, Without<ActiveEntity>, Without<VehicleState>, Without<NPCState>)>,
        Query<(&mut Cullable, &Transform, &mut Visibility)>,
    )>,
    
    mut distance_cache: ResMut<DistanceCache>,
    frame_counter: Res<FrameCounter>,
    time: Res<Time>,
) {
    let Ok((active_entity, active_transform)) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    lod_coordinator.frame_counter = frame_counter.frame;
    
    // Time budgeting - max 3ms per frame for entire LOD system
    let start_time = std::time::Instant::now();
    const MAX_FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(3);
    
    // Update chunk LOD levels based on distance
    update_chunk_lod_levels(&mut world_manager, active_pos);
    
    // Early exit if time budget exceeded
    if start_time.elapsed() > MAX_FRAME_TIME {
        return;
    }
    
    // Update chunk entity visibility
    {
        let mut chunk_query = visibility_param_set.p0();
        for (_entity, chunk_entity, mut visibility) in &mut chunk_query {
        if let Some(chunk) = world_manager.get_chunk(chunk_entity.coord) {
            let should_be_visible = match chunk.state {
                ChunkState::Loaded { entity_count: _ } => {
                    should_layer_be_visible(u32_to_content_layer(chunk_entity.layer), 0, chunk.distance_to_player)
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
    }
    
    // Process entity-type specific LOD updates with time budgeting
    if start_time.elapsed() < MAX_FRAME_TIME {
        process_vehicle_lod(&mut commands, &mut lod_coordinator, active_entity, active_pos, &mut vehicle_query, &mut distance_cache, time.elapsed_secs(), start_time, MAX_FRAME_TIME);
    }
    
    if start_time.elapsed() < MAX_FRAME_TIME {
        process_npc_lod(&mut commands, &mut lod_coordinator, active_entity, active_pos, &mut npc_query, &mut distance_cache, time.elapsed_secs(), start_time, MAX_FRAME_TIME);
    }
    
    if start_time.elapsed() < MAX_FRAME_TIME {
        let mut vegetation_query = visibility_param_set.p1();
        process_vegetation_lod(&mut commands, &mut lod_coordinator, active_entity, active_pos, &mut vegetation_query, &mut distance_cache, frame_counter.frame, start_time, MAX_FRAME_TIME);
    }
}

fn update_chunk_lod_levels(world_manager: &mut UnifiedWorldManager, active_pos: Vec3) {
    let chunks_to_update: Vec<_> = world_manager.chunks.iter()
        .filter_map(|(coord, chunk)| {
            if let ChunkState::Loaded { entity_count: _ } = chunk.state {
                let distance = active_pos.distance(chunk.coord.to_world_pos_local());
                Some((*coord, distance, 0))
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
                chunk.state = ChunkState::Loaded { entity_count: 0 };
            }
        }
    }
}

fn u32_to_content_layer(layer: u32) -> ContentLayer {
    match layer {
        0 => ContentLayer::Roads,
        1 => ContentLayer::Buildings,
        2 => ContentLayer::Vehicles,
        3 => ContentLayer::Vegetation,
        4 => ContentLayer::Landmarks,
        5 => ContentLayer::NPCs,
        _ => ContentLayer::Roads, // Default fallback
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
            // Vegetation with extended range using billboard LOD
            match lod_level {
                0 => distance <= 50.0,   // Full detail
                1 => distance <= 150.0,  // Medium detail
                2 => distance <= 300.0,  // Billboard
                _ => false,
            }
        }
        ContentLayer::NPCs => {
            // NPCs only at very close range
            lod_level == 0 && distance <= 150.0
        }
        ContentLayer::Landmarks => {
            // Landmarks visible at all distances with LOD
            lod_level <= 3
        }
    }
}

/// Vehicle LOD processing plugin - replaces `vehicles/lod_manager.rs`
fn process_vehicle_lod(
    commands: &mut Commands,
    lod_coordinator: &mut MasterLODCoordinator,
    active_entity: Entity,
    active_pos: Vec3,
    vehicle_query: &mut Query<(Entity, &mut VehicleState, &Transform, Option<&VehicleRendering>), Without<ActiveEntity>>,
    distance_cache: &mut ResMut<DistanceCache>,
    _current_time: f32,
    start_time: std::time::Instant,
    max_frame_time: std::time::Duration,
) {
    let config = lod_coordinator.lod_plugin_configs.entry(EntityType::Vehicle)
        .or_insert_with(LODPluginConfig::vehicle);
    
    let mut processed = 0;
    const MAX_ENTITIES_PER_FRAME: usize = 10;
    
    for (entity, mut vehicle_state, transform, rendering) in vehicle_query.iter_mut() {
        // Early exit if time budget exceeded
        if start_time.elapsed() > max_frame_time {
            break;
        }
        
        // Limit entities processed per frame
        if processed >= MAX_ENTITIES_PER_FRAME {
            break;
        }
        
        let distance = get_cached_distance(
            distance_cache,
            entity,
            active_entity,
            transform.translation,
            active_pos,
        );
        
        let new_lod_level = config.get_lod_level(distance);
        let should_cull = config.should_cull(distance);
        
        // Convert unified LOD level to VehicleLOD
        let new_vehicle_lod = match new_lod_level {
            0 => VehicleLOD::Full,
            1 => VehicleLOD::Medium,
            2 => VehicleLOD::Low,
            _ => VehicleLOD::StateOnly,
        };
        
        if should_cull {
            // Remove rendering when culled
            if rendering.is_some() {
                commands.entity(entity).remove::<VehicleRendering>();
                commands.entity(entity).insert(Visibility::Hidden);
            }
            vehicle_state.current_lod = VehicleLOD::StateOnly;
        } else if new_vehicle_lod != vehicle_state.current_lod {
            // LOD changed - mark for rendering system to handle
            vehicle_state.current_lod = new_vehicle_lod;
            commands.entity(entity).insert(VehicleLODUpdate { new_lod: new_vehicle_lod });
            commands.entity(entity).insert(Visibility::Visible);
        }
        
        processed += 1;
    }
    
    lod_coordinator.performance_stats.entities_processed.insert(EntityType::Vehicle, processed);
    
    // Track processing time for performance monitoring
    let processing_time = start_time.elapsed().as_secs_f32() * 1000.0; // Convert to ms
    lod_coordinator.performance_stats.processing_times.insert(EntityType::Vehicle, processing_time);
}

/// NPC LOD processing plugin - replaces `world/npc_lod.rs`
fn process_npc_lod(
    commands: &mut Commands,
    lod_coordinator: &mut MasterLODCoordinator,
    active_entity: Entity,
    active_pos: Vec3,
    npc_query: &mut Query<(Entity, &mut NPCState, &Transform, Option<&NPCRendering>), (Without<ActiveEntity>, Without<VehicleState>)>,
    distance_cache: &mut ResMut<DistanceCache>,
    current_time: f32,
    start_time: std::time::Instant,
    max_frame_time: std::time::Duration,
) {
    let config = lod_coordinator.lod_plugin_configs.entry(EntityType::NPC)
        .or_insert_with(LODPluginConfig::npc);
    
    let mut processed = 0;
    const MAX_ENTITIES_PER_FRAME: usize = 10;
    
    for (entity, mut npc_state, transform, rendering) in npc_query.iter_mut() {
        // Early exit if time budget exceeded
        if start_time.elapsed() > max_frame_time {
            break;
        }
        
        // Limit entities processed per frame
        if processed >= MAX_ENTITIES_PER_FRAME {
            break;
        }
        
        let distance = get_cached_distance(
            distance_cache,
            entity,
            active_entity,
            transform.translation,
            active_pos,
        );
        
        let new_lod_level = config.get_lod_level(distance);
        let should_cull = config.should_cull(distance);
        
        // Convert unified LOD level to NPCLOD
        let new_npc_lod = match new_lod_level {
            0 => NPCLOD::Full,
            1 => NPCLOD::Medium,
            2 => NPCLOD::Low,
            _ => NPCLOD::StateOnly,
        };
        
        if should_cull {
            // Remove rendering when culled
            if rendering.is_some() {
                commands.entity(entity).remove::<NPCRendering>();
                commands.entity(entity).insert(Visibility::Hidden);
            }
            npc_state.current_lod = NPCLOD::StateOnly;
        } else if new_npc_lod != npc_state.current_lod {
            // LOD changed - update and mark for rendering
            npc_state.current_lod = new_npc_lod;
            npc_state.last_lod_check = current_time;
            commands.entity(entity).insert(NPCLODUpdate { new_lod: new_npc_lod });
            commands.entity(entity).insert(Visibility::Visible);
        }
        
        processed += 1;
    }
    
    lod_coordinator.performance_stats.entities_processed.insert(EntityType::NPC, processed);
    
    // Track processing time for performance monitoring
    let processing_time = start_time.elapsed().as_secs_f32() * 1000.0; // Convert to ms
    lod_coordinator.performance_stats.processing_times.insert(EntityType::NPC, processing_time);
}

/// Vegetation LOD processing plugin - replaces `world/vegetation_lod.rs`
fn process_vegetation_lod(
    commands: &mut Commands,
    lod_coordinator: &mut MasterLODCoordinator,
    active_entity: Entity,
    active_pos: Vec3,
    vegetation_query: &mut Query<(Entity, &mut VegetationLOD, &Transform, &mut Visibility, &mut Mesh3d), (With<VegetationMeshLOD>, Without<ActiveEntity>, Without<VehicleState>, Without<NPCState>)>,
    distance_cache: &mut ResMut<DistanceCache>,
    current_frame: u64,
    start_time: std::time::Instant,
    max_frame_time: std::time::Duration,
) {
    let config = lod_coordinator.lod_plugin_configs.entry(EntityType::Vegetation)
        .or_insert_with(LODPluginConfig::vegetation);
    
    let mut processed = 0;
    const MAX_ENTITIES_PER_FRAME: usize = 10;
    
    for (entity, mut veg_lod, transform, mut visibility, _mesh_handle) in vegetation_query.iter_mut() {
        // Early exit if time budget exceeded
        if start_time.elapsed() > max_frame_time {
            break;
        }
        
        // Limit entities processed per frame
        if processed >= MAX_ENTITIES_PER_FRAME {
            break;
        }
        
        let distance = get_cached_distance(
            distance_cache,
            entity,
            active_entity,
            transform.translation,
            active_pos,
        );
        
        let new_lod_level = config.get_lod_level(distance);
        let should_cull = config.should_cull(distance);
        
        // Convert unified LOD level to VegetationDetailLevel
        let new_detail_level = if should_cull {
            VegetationDetailLevel::Culled
        } else {
            match new_lod_level {
                0 => VegetationDetailLevel::Full,
                1 => VegetationDetailLevel::Medium,
                2 => VegetationDetailLevel::Billboard,
                _ => VegetationDetailLevel::Culled,
            }
        };
        
        let old_level = veg_lod.detail_level;
        veg_lod.detail_level = new_detail_level;
        veg_lod.distance_to_player = distance;
        veg_lod.update_from_distance(distance, current_frame);
        
        // Update visibility
        *visibility = if veg_lod.should_be_visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        
        // Update mesh if LOD level changed
        if old_level != new_detail_level {
            commands.entity(entity).insert(VegetationLODUpdate { 
                new_detail_level,
                distance,
            });
        }
        
        processed += 1;
    }
    
    lod_coordinator.performance_stats.entities_processed.insert(EntityType::Vegetation, processed);
    
    // Track processing time for performance monitoring
    let processing_time = start_time.elapsed().as_secs_f32() * 1000.0; // Convert to ms
    lod_coordinator.performance_stats.processing_times.insert(EntityType::Vegetation, processing_time);
}

/// Component to signal vehicle LOD updates (replaces `vehicles/lod_manager.rs` functionality)
#[derive(Component)]
pub struct VehicleLODUpdate {
    pub new_lod: VehicleLOD,
}

/// Component to signal NPC LOD updates (replaces `world/npc_lod.rs` functionality)
#[derive(Component)]
pub struct NPCLODUpdate {
    pub new_lod: NPCLOD,
}

/// Component to signal vegetation LOD updates (replaces `world/vegetation_lod.rs` functionality)
#[derive(Component)]
pub struct VegetationLODUpdate {
    pub new_detail_level: VegetationDetailLevel,
    pub distance: f32,
}

/// Master LOD system initialization
pub fn initialize_master_lod_system(mut commands: Commands) {
    let mut coordinator = MasterLODCoordinator::default();
    
    // Initialize entity-type configurations
    coordinator.lod_plugin_configs.insert(EntityType::Vehicle, LODPluginConfig::vehicle());
    coordinator.lod_plugin_configs.insert(EntityType::NPC, LODPluginConfig::npc());
    coordinator.lod_plugin_configs.insert(EntityType::Vegetation, LODPluginConfig::vegetation());
    coordinator.lod_plugin_configs.insert(EntityType::Building, LODPluginConfig::building());
    coordinator.lod_plugin_configs.insert(EntityType::Chunk, LODPluginConfig::chunk());
    
    commands.insert_resource(coordinator);
    
    info!("Master LOD Coordinator initialized with unified pipeline");
}

/// Enhanced performance monitoring system for the master LOD system
pub fn master_lod_performance_monitor(
    lod_coordinator: Res<MasterLODCoordinator>,
    world_manager: Res<UnifiedWorldManager>,
    _chunk_query: Query<&UnifiedChunkEntity>,
    vehicle_query: Query<&VehicleState>,
    npc_query: Query<&NPCState>,
    vegetation_query: Query<&VegetationLOD>,
    mut performance_stats: ResMut<PerformanceStats>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    *last_report += time.delta_secs();
    
    // Report every 5 seconds to reduce log spam
    if *last_report < 5.0 {
        return;
    }
    *last_report = 0.0;
    
    // Count entities by type and LOD level
    let mut vehicle_lod_counts = [0; 4]; // Full, Medium, Low, StateOnly
    let mut npc_lod_counts = [0; 4];
    let mut vegetation_lod_counts = [0; 4]; // Full, Medium, Billboard, Culled
    
    for vehicle in vehicle_query.iter() {
        match vehicle.current_lod {
            VehicleLOD::Full => vehicle_lod_counts[0] += 1,
            VehicleLOD::Medium => vehicle_lod_counts[1] += 1,
            VehicleLOD::Low => vehicle_lod_counts[2] += 1,
            VehicleLOD::StateOnly => vehicle_lod_counts[3] += 1,
        }
    }
    
    for npc in npc_query.iter() {
        match npc.current_lod {
            NPCLOD::Full => npc_lod_counts[0] += 1,
            NPCLOD::Medium => npc_lod_counts[1] += 1,
            NPCLOD::Low => npc_lod_counts[2] += 1,
            NPCLOD::StateOnly => npc_lod_counts[3] += 1,
        }
    }
    
    for veg in vegetation_query.iter() {
        match veg.detail_level {
            VegetationDetailLevel::Full => vegetation_lod_counts[0] += 1,
            VegetationDetailLevel::Medium => vegetation_lod_counts[1] += 1,
            VegetationDetailLevel::Billboard => vegetation_lod_counts[2] += 1,
            VegetationDetailLevel::Culled => vegetation_lod_counts[3] += 1,
        }
    }
    
    let total_chunks = world_manager.chunks.len();
    let loaded_chunks = world_manager.chunks.values()
        .filter(|chunk| matches!(chunk.state, ChunkState::Loaded { .. }))
        .count();
    
    let total_entities = lod_coordinator.performance_stats.total_entities;
    let culled_entities = lod_coordinator.performance_stats.culled_entities;
    
    // Update performance stats
    performance_stats.entity_count = total_entities;
    performance_stats.culled_entities = culled_entities;
    
    info!(
        "Master LOD Performance | Chunks: {}/{} | Vehicles: F:{} M:{} L:{} S:{} | NPCs: F:{} M:{} L:{} S:{} | Vegetation: F:{} M:{} B:{} C:{}",
        loaded_chunks, total_chunks,
        vehicle_lod_counts[0], vehicle_lod_counts[1], vehicle_lod_counts[2], vehicle_lod_counts[3],
        npc_lod_counts[0], npc_lod_counts[1], npc_lod_counts[2], npc_lod_counts[3],
        vegetation_lod_counts[0], vegetation_lod_counts[1], vegetation_lod_counts[2], vegetation_lod_counts[3]
    );
    
    // Report processing efficiency
    for (entity_type, processed) in &lod_coordinator.performance_stats.entities_processed {
        let processing_time = lod_coordinator.performance_stats.processing_times.get(entity_type).copied().unwrap_or(0.0);
        if *processed > 0 {
            info!("  {:?}: {} entities processed in {:.2}ms", entity_type, processed, processing_time);
        }
    }
}

/// System to handle dynamic LOD adjustments based on performance
pub fn adaptive_lod_system(
    mut world_manager: ResMut<UnifiedWorldManager>,
    _performance_stats: Res<PerformanceStats>,
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

// NOTE: This function is disabled to avoid conflicts with the main unified_distance_culling_system
// The main culling is handled by src/systems/world/unified_distance_culling.rs
/*
/// Unified culling system that replaces the old distance_culling_system
pub fn unified_distance_culling_system(
    mut cullable_query: Query<(&mut Cullable, &Transform, &mut Visibility), Without<DirtyVisibility>>,
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
*/

/// System to clean up entities that have been culled for too long
pub fn unified_cleanup_system(
    mut commands: Commands,
    world_manager: ResMut<UnifiedWorldManager>,
    cullable_query: Query<(Entity, &Cullable, &Transform)>,
    time: Res<Time>,
) {
    let _current_time = time.elapsed_secs();
    let _cleanup_delay = 30.0; // Clean up entities culled for 30+ seconds
    
    for (entity, cullable, transform) in cullable_query.iter() {
        if cullable.is_culled {
            // In a full implementation, you'd track when entities were first culled
            // For now, we'll just clean up very distant entities immediately
            let distance_to_any_chunk = world_manager
                .chunks
                .values()
                .map(|chunk| transform.translation.distance(chunk.coord.to_world_pos_local()))
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
