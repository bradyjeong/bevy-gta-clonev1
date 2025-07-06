use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Instant;
use crate::components::*;

use crate::config::GameConfig;

/// Core batching systems module
/// Implements efficient batch processing with dirty flags

/// System to automatically mark entities as dirty when Transform changes
pub fn mark_transform_dirty_system(
    mut commands: Commands,
    changed_transforms: Query<Entity, (Changed<Transform>, Without<DirtyTransform>)>,
    mut existing_dirty: Query<&mut DirtyTransform>,
    frame_counter: Res<FrameCounter>,

) {
    let current_frame = frame_counter.frame;
    
    // Mark newly changed transforms as dirty
    for entity in changed_transforms.iter() {
        commands.entity(entity).insert(DirtyTransform::new(
            DirtyPriority::Normal,
            current_frame,
        ));
    }
    
    // Update existing dirty flags if transform threshold exceeded
    for mut dirty in existing_dirty.iter_mut() {
        if dirty.is_stale(current_frame, 60) { // Use hardcoded value instead of config service
            dirty.priority = DirtyPriority::High;
        }
    }
}

/// System to mark entities as dirty when visibility changes
pub fn mark_visibility_dirty_system(
    mut commands: Commands,
    changed_visibility: Query<Entity, (Changed<Visibility>, Without<DirtyVisibility>)>,
    changed_cullable: Query<Entity, (Changed<Cullable>, Without<DirtyVisibility>)>,
    mut existing_dirty: Query<&mut DirtyVisibility>,
    frame_counter: Res<FrameCounter>,
    config: Res<GameConfig>,
) {
    let current_frame = frame_counter.frame;
    
    // Throttle visibility updates to reduce flickering (only every 2nd frame)
    if current_frame % 2 != 0 {
        return;
    }
    
    // Mark entities with changed visibility
    for entity in changed_visibility.iter() {
        commands.entity(entity).insert(DirtyVisibility::new(
            DirtyPriority::Normal,
            current_frame,
        ));
    }
    
    // Mark entities with changed cullable state
    for entity in changed_cullable.iter() {
        commands.entity(entity).insert(DirtyVisibility::new(
            DirtyPriority::High, // Cullable changes are important
            current_frame,
        ));
    }
    
    // Boost priority for stale dirty flags
    for mut dirty in existing_dirty.iter_mut() {
        if dirty.is_stale(current_frame, config.batching.priority_boost_frames) {
            dirty.priority = DirtyPriority::High;
        }
    }
}

/// System to mark vegetation entities as dirty when they change (batch processing version)
pub fn batch_mark_vegetation_instancing_dirty_system(
    mut commands: Commands,
    changed_vegetation: Query<Entity, (Changed<Transform>, With<VegetationBatchable>, Without<DirtyVegetationInstancing>)>,
    frame_counter: Res<FrameCounter>,
) {
    let current_frame = frame_counter.frame;
    
    for entity in changed_vegetation.iter() {
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Low, // Vegetation changes are low priority
            current_frame,
        ));
    }
}

/// System to mark entities as dirty when physics components change
pub fn mark_physics_dirty_system(
    mut commands: Commands,
    changed_velocity: Query<Entity, (Changed<Velocity>, Without<DirtyPhysics>)>,
    changed_rigid_body: Query<Entity, (Changed<RigidBody>, Without<DirtyPhysics>)>,
    changed_collider: Query<Entity, (Changed<Collider>, Without<DirtyPhysics>)>,
    mut existing_dirty: Query<&mut DirtyPhysics>,
    frame_counter: Res<FrameCounter>,
    config: Res<GameConfig>,
) {
    let current_frame = frame_counter.frame;
    
    // Mark entities with physics changes
    for entity in changed_velocity.iter().chain(changed_rigid_body.iter()).chain(changed_collider.iter()) {
        commands.entity(entity).insert(DirtyPhysics::new(
            DirtyPriority::High, // Physics changes are critical
            current_frame,
        ));
    }
    
    // Boost priority for stale dirty flags
    for mut dirty in existing_dirty.iter_mut() {
        if dirty.is_stale(current_frame, config.batching.priority_boost_frames) {
            dirty.priority = DirtyPriority::Critical;
        }
    }
}

/// Batch processing system for LOD updates
pub fn batch_lod_processing_system(
    mut commands: Commands,
    mut dirty_lod_query: Query<(Entity, &mut DirtyLOD, &Transform, Option<&mut NPCState>, Option<&mut Cullable>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DirtyLOD>)>,
    mut batch_state: ResMut<BatchState>,
    mut metrics: ResMut<DirtyFlagsMetrics>,
    config: Res<GameConfig>,
    frame_counter: Res<FrameCounter>,
    time: Res<Time>,
) {
    let start_time = Instant::now();
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    let current_frame = frame_counter.frame;
    
    // Collect entities to process, sorted by priority
    let mut entities_to_process: Vec<_> = dirty_lod_query.iter_mut().collect();
    entities_to_process.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    let batch_size = config.batching.lod_batch_size;
    let start_offset = batch_state.lod_offset;
    let end_offset = (start_offset + batch_size).min(entities_to_process.len());
    
    let mut processed = 0;
    
    for (entity, mut dirty_lod, transform, npc_state, cullable) in 
        entities_to_process.into_iter().skip(start_offset).take(batch_size) {
        
        // Check processing time limit
        if start_time.elapsed().as_millis() as f32 > config.batching.max_processing_time_ms {
            break;
        }
        
        let distance = active_pos.distance(transform.translation);
        
        // Only process if distance changed significantly
        if dirty_lod.distance_changed_significantly(distance, config.batching.lod_distance_threshold) {
            // Update NPC LOD if present
            if let Some(mut npc) = npc_state {
                let new_lod = calculate_npc_lod(distance);
                if new_lod != npc.current_lod {
                    npc.current_lod = new_lod;
                    npc.last_lod_check = time.elapsed_secs();
                }
            }
            
            // Update cullable if present
            if let Some(mut cull) = cullable {
                let should_cull = distance > cull.max_distance;
                if should_cull != cull.is_culled {
                    cull.is_culled = should_cull;
                    commands.entity(entity).insert(DirtyVisibility::new(
                        DirtyPriority::High,
                        current_frame,
                    ));
                }
            }
            
            dirty_lod.last_distance = distance;
        }
        
        // Remove dirty flag after processing
        commands.entity(entity).remove::<DirtyLOD>();
        processed += 1;
    }
    
    // Update batch state for round-robin processing
    batch_state.lod_offset = if end_offset >= dirty_lod_query.iter().count() { 0 } else { end_offset };
    
    // Update metrics
    metrics.entities_processed_lod += processed;
    metrics.processing_time_lod = start_time.elapsed().as_millis() as f32;
}

/// Batch processing system for culling/visibility updates
pub fn batch_culling_system(
    mut commands: Commands,
    mut dirty_visibility_query: Query<(Entity, &DirtyVisibility, &Transform, &mut Visibility, Option<&Cullable>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DirtyVisibility>)>,
    mut batch_state: ResMut<BatchState>,
    mut metrics: ResMut<DirtyFlagsMetrics>,
    config: Res<GameConfig>,
    _time: Res<Time>,
) {
    let start_time = Instant::now();
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Collect entities to process, sorted by priority
    let mut entities_to_process: Vec<_> = dirty_visibility_query.iter_mut().collect();
    entities_to_process.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    let batch_size = config.batching.visibility_batch_size;
    let start_offset = batch_state.visibility_offset;
    let end_offset = (start_offset + batch_size).min(entities_to_process.len());
    
    let mut processed = 0;
    
    for (entity, _dirty_vis, transform, mut visibility, cullable) in 
        entities_to_process.into_iter().skip(start_offset).take(batch_size) {
        
        // Check processing time limit
        if start_time.elapsed().as_millis() as f32 > config.batching.max_processing_time_ms {
            break;
        }
        
        if let Some(cull) = cullable {
            let distance = active_pos.distance(transform.translation);
            let should_be_visible = !cull.is_culled && distance <= cull.max_distance;
            
            let new_visibility = if should_be_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            
            if *visibility != new_visibility {
                *visibility = new_visibility;
            }
        }
        
        // Remove dirty flag after processing
        commands.entity(entity).remove::<DirtyVisibility>();
        processed += 1;
    }
    
    // Update batch state for round-robin processing
    batch_state.visibility_offset = if end_offset >= dirty_visibility_query.iter().count() { 0 } else { end_offset };
    
    // Update metrics
    metrics.entities_processed_visibility += processed;
    metrics.processing_time_visibility = start_time.elapsed().as_millis() as f32;
}

/// Batch processing system for physics updates
pub fn batch_physics_processing_system(
    mut commands: Commands,
    mut dirty_physics_query: Query<(Entity, &DirtyPhysics, &mut Velocity, Option<&mut RigidBody>)>,
    mut batch_state: ResMut<BatchState>,
    mut metrics: ResMut<DirtyFlagsMetrics>,
    config: Res<GameConfig>,
) {
    let start_time = Instant::now();
    
    // Collect entities to process, sorted by priority
    let mut entities_to_process: Vec<_> = dirty_physics_query.iter_mut().collect();
    entities_to_process.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    let batch_size = config.batching.physics_batch_size;
    let start_offset = batch_state.physics_offset;
    let end_offset = (start_offset + batch_size).min(entities_to_process.len());
    
    let mut processed = 0;
    
    for (entity, _dirty_physics, _velocity, rigid_body) in 
        entities_to_process.into_iter().skip(start_offset).take(batch_size) {
        
        // Check processing time limit
        if start_time.elapsed().as_millis() as f32 > config.batching.max_processing_time_ms {
            break;
        }
        
        // Apply physics constraints and validation
        let _max_velocity = config.physics.max_velocity;
        let _max_angular_velocity = config.physics.max_angular_velocity;
        
        // DISABLE VELOCITY CLAMPING - CAUSES MICRO-JITTER
        // Let physics engine handle velocity limits naturally
        
        // Additional rigid body processing if needed
        if let Some(_rb) = rigid_body {
            // Could add additional rigid body constraints here
        }
        
        // Remove dirty flag after processing
        commands.entity(entity).remove::<DirtyPhysics>();
        processed += 1;
    }
    
    // Update batch state for round-robin processing
    batch_state.physics_offset = if end_offset >= dirty_physics_query.iter().count() { 0 } else { end_offset };
    
    // Update metrics
    metrics.entities_processed_physics += processed;
    metrics.processing_time_physics = start_time.elapsed().as_millis() as f32;
}

/// Batch processing system for transform updates
pub fn batch_transform_processing_system(
    mut commands: Commands,
    mut dirty_transform_query: Query<(Entity, &DirtyTransform, &mut Transform)>,
    mut batch_state: ResMut<BatchState>,
    mut metrics: ResMut<DirtyFlagsMetrics>,
    config: Res<GameConfig>,
) {
    let start_time = Instant::now();
    
    // Collect entities to process, sorted by priority
    let mut entities_to_process: Vec<_> = dirty_transform_query.iter_mut().collect();
    entities_to_process.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    let batch_size = config.batching.transform_batch_size;
    let start_offset = batch_state.transform_offset;
    let end_offset = (start_offset + batch_size).min(entities_to_process.len());
    
    let mut processed = 0;
    
    for (entity, _dirty_transform, mut transform) in 
        entities_to_process.into_iter().skip(start_offset).take(batch_size) {
        
        // Check processing time limit
        if start_time.elapsed().as_millis() as f32 > config.batching.max_processing_time_ms {
            break;
        }
        
        // Apply transform validation and constraints
        let _max_coord = config.physics.max_world_coord;
        let _min_coord = config.physics.min_world_coord;
        
        // Clamp position to world bounds
        // Disable transform clamping to prevent shake
        // transform.translation.x = transform.translation.x.clamp(min_coord, max_coord);
        // transform.translation.z = transform.translation.z.clamp(min_coord, max_coord);
        
        // Normalize rotation to prevent drift
        transform.rotation = transform.rotation.normalize();
        
        // Remove dirty flag after processing
        commands.entity(entity).remove::<DirtyTransform>();
        processed += 1;
    }
    
    // Update batch state for round-robin processing
    batch_state.transform_offset = if end_offset >= dirty_transform_query.iter().count() { 0 } else { end_offset };
    
    // Update metrics
    metrics.entities_processed_transform += processed;
    metrics.processing_time_transform = start_time.elapsed().as_millis() as f32;
}

/// System to clean up stale dirty flags
pub fn dirty_flag_cleanup_system(
    mut commands: Commands,
    stale_transform: Query<Entity, With<DirtyTransform>>,
    stale_visibility: Query<Entity, With<DirtyVisibility>>,
    stale_physics: Query<Entity, With<DirtyPhysics>>,
    stale_lod: Query<Entity, With<DirtyLOD>>,
    frame_counter: Res<FrameCounter>,
    config: Res<GameConfig>,
    mut cleanup_timer: Local<f32>,
    time: Res<Time>,
) {
    if !config.batching.cleanup_stale_flags {
        return;
    }
    
    *cleanup_timer += time.delta_secs();
    if *cleanup_timer < config.batching.cleanup_interval {
        return;
    }
    *cleanup_timer = 0.0;
    
    let current_frame = frame_counter.frame;
    let max_stale = config.batching.max_stale_frames;
    
    // Clean up stale dirty flags (entities that were marked dirty too long ago)
    // This prevents dirty flags from accumulating indefinitely
    
    // In a real implementation, you'd need to query the dirty flag components
    // and check their marked_frame against current_frame
    // For brevity, this is a simplified cleanup that removes all flags periodically
    
    let mut cleaned = 0;
    
    // This is a simplified cleanup - in reality you'd check the marked_frame
    if current_frame % (max_stale * 2) == 0 {
        for entity in stale_transform.iter().take(10) {
            commands.entity(entity).remove::<DirtyTransform>();
            cleaned += 1;
        }
        
        for entity in stale_visibility.iter().take(10) {
            commands.entity(entity).remove::<DirtyVisibility>();
            cleaned += 1;
        }
        
        for entity in stale_physics.iter().take(10) {
            commands.entity(entity).remove::<DirtyPhysics>();
            cleaned += 1;
        }
        
        for entity in stale_lod.iter().take(10) {
            commands.entity(entity).remove::<DirtyLOD>();
            cleaned += 1;
        }
    }
    
    if cleaned > 0 {
        info!("Cleaned up {} stale dirty flags", cleaned);
    }
}

/// System to update frame counter
pub fn frame_counter_system(mut frame_counter: ResMut<FrameCounter>) {
    frame_counter.frame = frame_counter.frame.wrapping_add(1);
}

/// System to update dirty flag metrics
pub fn dirty_flags_metrics_system(
    dirty_transform: Query<&DirtyTransform>,
    dirty_visibility: Query<&DirtyVisibility>,
    dirty_physics: Query<&DirtyPhysics>,
    dirty_lod: Query<&DirtyLOD>,
    mut metrics: ResMut<DirtyFlagsMetrics>,
    time: Res<Time>,
) {
    metrics.entities_marked_transform = dirty_transform.iter().count();
    metrics.entities_marked_visibility = dirty_visibility.iter().count();
    metrics.entities_marked_physics = dirty_physics.iter().count();
    metrics.entities_marked_lod = dirty_lod.iter().count();
    
    // Report every 5 seconds
    let current_time = time.elapsed_secs();
    if current_time - metrics.last_report_time > 5.0 {
        info!(
            "Dirty Flags - Marked: T:{} V:{} P:{} L:{} | Processed: T:{} V:{} P:{} L:{} | Time: T:{:.1}ms V:{:.1}ms P:{:.1}ms L:{:.1}ms",
            metrics.entities_marked_transform,
            metrics.entities_marked_visibility,
            metrics.entities_marked_physics,
            metrics.entities_marked_lod,
            metrics.entities_processed_transform,
            metrics.entities_processed_visibility,
            metrics.entities_processed_physics,
            metrics.entities_processed_lod,
            metrics.processing_time_transform,
            metrics.processing_time_visibility,
            metrics.processing_time_physics,
            metrics.processing_time_lod,
        );
        
        metrics.last_report_time = current_time;
        
        // Reset processed counters
        metrics.entities_processed_transform = 0;
        metrics.entities_processed_visibility = 0;
        metrics.entities_processed_physics = 0;
        metrics.entities_processed_lod = 0;
    }
}

/// Helper function to calculate NPC LOD based on distance
fn calculate_npc_lod(distance: f32) -> NPCLOD {
    if distance <= NPC_LOD_FULL_DISTANCE {
        NPCLOD::Full
    } else if distance <= NPC_LOD_MEDIUM_DISTANCE {
        NPCLOD::Medium
    } else if distance <= NPC_LOD_LOW_DISTANCE {
        NPCLOD::Low
    } else {
        NPCLOD::StateOnly
    }
}
