//! ───────────────────────────────────────────────
//! System:   Unified Distance Culling
//! Purpose:  Handles entity movement and physics
//! Schedule: Update (throttled)
//! Reads:    VehicleState, ActiveEntity, DirtyLOD, MapChunk, Transform
//! Writes:   Visibility, DistanceCache, PerformanceStats, UnifiedCullingTimer
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Only active entities can be controlled
//!   * Timing intervals are respected
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;
use crate::systems::distance_cache::{DistanceCache, get_cached_distance_squared};
use crate::systems::world::map_system::MapChunk;

// Re-export the canonical spatial components from game_core
pub use game_core::prelude::*;
/// Timer resource for unified culling system
#[derive(Resource, Default)]
pub struct UnifiedCullingTimer {
    pub elapsed: f32,
}
/// Main unified distance culling system (renamed to avoid conflicts)
pub fn new_unified_distance_culling_system(
    mut cullable_query: Query<(Entity, &mut UnifiedCullable, &Transform, &mut Visibility)>,
    active_query: Query<(Entity, &Transform), (With<ActiveEntity>, Without<UnifiedCullable>)>,
    mut distance_cache: ResMut<DistanceCache>,
    mut timer: ResMut<UnifiedCullingTimer>,
    time: Res<Time>,
    mut commands: Commands,
    frame_counter: Res<FrameCounter>,
) {
    let Ok((active_entity, active_transform)) = active_query.single() else { return };
    let player_pos = active_transform.translation;
    
    timer.elapsed += time.delta_secs();
    let current_time = timer.elapsed;
    let current_frame = frame_counter.frame;
    // Time budgeting - max 4ms per frame
    let start_time = std::time::Instant::now();
    const MAX_FRAME_TIME: std::time::Duration = std::time::Duration::from_millis(4);
    // Reduced entity processing per frame
    let mut processed = 0;
    const MAX_ENTITIES_PER_FRAME: usize = 15;
    for (entity, mut cullable, transform, mut visibility) in cullable_query.iter_mut() {
        // Early exit if time budget exceeded
        if start_time.elapsed() > MAX_FRAME_TIME {
            break;
        }
        
        if processed >= MAX_ENTITIES_PER_FRAME {
        // Use cached distance calculation for efficiency
        let distance_squared = get_cached_distance_squared(
            active_entity,
            entity,
            player_pos,
            transform.translation,
            &mut distance_cache,
        );
        let distance = distance_squared.sqrt();
        // Only update if necessary
        if cullable.needs_update(current_time, distance) {
            let old_lod = cullable.current_lod;
            let old_culled = cullable.is_culled;
            cullable.update_distance(distance, current_time);
            let state_changed = old_lod != cullable.current_lod || old_culled != cullable.is_culled;
            
            if state_changed {
                // Update visibility
                *visibility = if cullable.is_culled {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
                
                // Mark entity as dirty for other systems to respond to LOD changes
                commands.entity(entity).insert(DirtyLOD::new(
                    if distance < 100.0 { DirtyPriority::High } else { DirtyPriority::Normal },
                    current_frame,
                    distance,
                ));
                // Mark visibility as dirty if changed
                commands.entity(entity).insert(DirtyVisibility::new(
                    DirtyPriority::Normal,
            }
            processed += 1;
    }
/// System specifically for vehicle LOD using unified culling
pub fn unified_vehicle_lod_system(
    vehicle_query: Query<(Entity, &UnifiedCullable, &VehicleState), (With<DirtyLOD>, Changed<UnifiedCullable>)>,
    for (entity, cullable, vehicle_state) in vehicle_query.iter() {
        if cullable.is_culled {
            // Remove rendering components when culled
            commands.entity(entity).remove::<VehicleRendering>();
        } else {
            // Update vehicle LOD based on unified culling LOD level
            let vehicle_lod = match cullable.current_lod {
                0 => VehicleLOD::Full,
                1 => VehicleLOD::Medium,
                2 => VehicleLOD::Low,
                _ => VehicleLOD::StateOnly,
            };
            // Only update if LOD actually changed
            if vehicle_state.current_lod != vehicle_lod {
                // Mark for rendering system to handle the mesh updates
                commands.entity(entity).insert(VehicleLODUpdate { new_lod: vehicle_lod });
        // Remove dirty flag after processing
        commands.entity(entity).remove::<DirtyLOD>();
/// Component to signal vehicle LOD updates
#[derive(Component)]
pub struct VehicleLODUpdate {
    pub new_lod: VehicleLOD,
/// System specifically for NPC LOD using unified culling
pub fn unified_npc_lod_system(
    npc_query: Query<(Entity, &UnifiedCullable, &NPCState), (With<DirtyLOD>, Changed<UnifiedCullable>)>,
    for (entity, cullable, npc_state) in npc_query.iter() {
            commands.entity(entity).remove::<NPCRendering>();
            // Update NPC LOD based on unified culling LOD level
            let npc_lod = match cullable.current_lod {
                0 => NPCLOD::Full,
                1 => NPCLOD::Medium,
                2 => NPCLOD::Low,
                _ => NPCLOD::StateOnly,
            if npc_state.current_lod != npc_lod {
                commands.entity(entity).insert(NPCLODUpdate { new_lod: npc_lod });
/// Component to signal NPC LOD updates
pub struct NPCLODUpdate {
    pub new_lod: NPCLOD,
/// System specifically for vegetation LOD using unified culling
pub fn unified_vegetation_lod_system(
    vegetation_query: Query<(Entity, &UnifiedCullable, &VegetationLOD), (With<DirtyLOD>, Changed<UnifiedCullable>)>,
    for (entity, cullable, vegetation_lod) in vegetation_query.iter() {
        let new_detail_level = if cullable.is_culled {
            VegetationDetailLevel::Culled
            match cullable.current_lod {
                0 => VegetationDetailLevel::Full,
                1 => VegetationDetailLevel::Medium,
                2 => VegetationDetailLevel::Billboard,
                _ => VegetationDetailLevel::Culled,
        };
        // Only update if LOD actually changed
        if vegetation_lod.detail_level != new_detail_level {
            // Mark for vegetation rendering system to handle the mesh updates
            commands.entity(entity).insert(VegetationLODUpdate { 
                new_detail_level,
                distance: cullable.last_distance,
            });
/// Component to signal vegetation LOD updates
pub struct VegetationLODUpdate {
    pub new_detail_level: VegetationDetailLevel,
    pub distance: f32,
/// System for chunk LOD using unified culling (replaces map_system chunk LOD)
pub fn unified_chunk_lod_system(
    chunk_query: Query<(Entity, &UnifiedCullable, &MapChunk), (With<DirtyLOD>, Changed<UnifiedCullable>)>,
    for (entity, cullable, chunk) in chunk_query.iter() {
            // Mark chunk for unloading
            commands.entity(entity).insert(ChunkUnloadRequest);
            // Update chunk LOD if changed
            if chunk.lod_level != cullable.current_lod {
                commands.entity(entity).insert(ChunkLODUpdate { 
                    new_lod: cullable.current_lod,
                    distance: cullable.last_distance,
                });
/// Component to signal chunk LOD updates
pub struct ChunkLODUpdate {
    pub new_lod: usize,
/// Component to mark chunks for unloading
pub struct ChunkUnloadRequest;
/// Performance monitoring system for unified culling
pub fn unified_culling_performance_monitor(
    cullable_query: Query<&UnifiedCullable>,
    mut performance_stats: ResMut<PerformanceStats>,
    mut last_report: Local<f32>,
    *last_report += time.delta_secs();
    if *last_report > 5.0 {
        let mut type_counts = std::collections::HashMap::new();
        let mut lod_counts = std::collections::HashMap::new();
        let mut culled_count = 0;
        let total_entities = cullable_query.iter().count();
        for cullable in cullable_query.iter() {
            *type_counts.entry(cullable.config.entity_type).or_insert(0) += 1;
            *lod_counts.entry(cullable.current_lod).or_insert(0) += 1;
            if cullable.is_culled {
                culled_count += 1;
        info!(
            "Unified Culling Performance - Total: {} | Culled: {} | Types: {:?} | LOD Distribution: {:?}",
            total_entities, culled_count, type_counts, lod_counts
        performance_stats.entity_count = total_entities;
        performance_stats.culled_entities = culled_count;
        *last_report = 0.0;
/// System to handle entity movement and automatically mark for distance updates
pub fn unified_culling_movement_tracker(
    moved_entities: Query<
        (Entity, &UnifiedCullable, &Transform), 
        (Changed<Transform>, Without<DirtyLOD>)
    >,
    for (entity, cullable, transform) in moved_entities.iter() {
        // Calculate how much the entity moved
        let movement_threshold = cullable.config.hysteresis;
        let distance_moved = (transform.translation - Vec3::ZERO).length(); // Simplified
        if distance_moved > movement_threshold {
            let priority = if cullable.last_distance < 100.0 {
                DirtyPriority::High // Close entities get higher priority
            } else {
                DirtyPriority::Normal
            commands.entity(entity).insert(DirtyLOD::new(
                priority,
                current_frame,
                cullable.last_distance,
            ));
/// Helper function to convert old Cullable component to UnifiedCullable
pub fn migrate_cullable_to_unified(
    query: Query<(Entity, &Cullable), Without<UnifiedCullable>>,
    for (entity, cullable) in query.iter() {
        // Create a generic config based on max_distance
        let config = if cullable.max_distance <= 150.0 {
            DistanceCullingConfig::npc()
        } else if cullable.max_distance <= 400.0 {
            DistanceCullingConfig::vegetation()
        } else if cullable.max_distance <= 500.0 {
            DistanceCullingConfig::vehicle()
            DistanceCullingConfig::buildings()
        let unified_cullable = UnifiedCullable::new(config);
        commands.entity(entity).insert(unified_cullable);
        commands.entity(entity).remove::<Cullable>();
/// Plugin to integrate unified distance culling system
pub struct UnifiedDistanceCullingPlugin;
impl Plugin for UnifiedDistanceCullingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UnifiedCullingTimer::default())
            .add_systems(Update, (
                // Main culling system runs first
                new_unified_distance_culling_system,
                // Entity-specific LOD systems run after
                unified_vehicle_lod_system,
                unified_npc_lod_system,
                unified_vegetation_lod_system,
                unified_chunk_lod_system,
                // Support systems
                unified_culling_movement_tracker,
                unified_culling_performance_monitor,
                // Migration helper (can be removed after migration)
                migrate_cullable_to_unified,
            ).chain());
