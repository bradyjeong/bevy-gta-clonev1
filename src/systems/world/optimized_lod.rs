use bevy::prelude::*;
use crate::components::*;
use crate::systems::world::unified_world::{UnifiedWorldManager, UnifiedChunkEntity, ContentLayer, ChunkState};
// use crate::services::{Services, simple_services::ConfigService};
use crate::GameConfig;

/// Optimized LOD system using dirty flags for better performance
/// Only processes entities marked as needing LOD updates

/// Enhanced unified LOD system with dirty flag optimization
pub fn optimized_unified_lod_system(
    mut commands: Commands,
    world_manager: ResMut<UnifiedWorldManager>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut chunk_query: Query<(Entity, &UnifiedChunkEntity, &mut Visibility), With<DirtyLOD>>,
    mut cullable_query: Query<(Entity, &mut Cullable, &mut Visibility, &Transform), (With<DirtyLOD>, Without<UnifiedChunkEntity>)>,
    frame_counter: Res<FrameCounter>,
    // services: Services,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    let current_frame = frame_counter.frame;
    
    // let config_service = services.require::<ConfigService>();
    
    // Update chunk LOD levels only for dirty chunks
    // update_dirty_chunk_lod_levels(&mut world_manager, active_pos, &config_service);
    
    // Update chunk entity visibility for dirty chunks only
    for (entity, chunk_entity, mut visibility) in chunk_query.iter_mut() {
        if let Some(chunk) = world_manager.get_chunk(chunk_entity.coord) {
            let should_be_visible = match chunk.state {
                ChunkState::Loaded { lod_level } => {
                    should_layer_be_visible(chunk_entity.layer, lod_level, chunk.distance_to_player)
                }
                _ => false,
            };
            
            let new_visibility = if should_be_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            
            if *visibility != new_visibility {
                *visibility = new_visibility;
                
                // Mark for visibility update if it changed
                commands.entity(entity).insert(DirtyVisibility::new(
                    DirtyPriority::Normal,
                    current_frame,
                ));
            }
        }
        
        // Remove dirty LOD flag after processing
        commands.entity(entity).remove::<DirtyLOD>();
    }
    
    // Update individual entity culling for dirty entities only
    for (entity, mut cullable, mut visibility, transform) in cullable_query.iter_mut() {
        let distance = active_pos.distance(transform.translation);
        let should_be_culled = distance > cullable.max_distance;
        
        if should_be_culled != cullable.is_culled {
            cullable.is_culled = should_be_culled;
            
            let new_visibility = if should_be_culled {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
            
            if *visibility != new_visibility {
                *visibility = new_visibility;
                
                // Mark for visibility update
                commands.entity(entity).insert(DirtyVisibility::new(
                    DirtyPriority::High,
                    current_frame,
                ));
            }
        }
        
        // Remove dirty LOD flag after processing
        commands.entity(entity).remove::<DirtyLOD>();
    }
}

/// Optimized distance culling system that only processes dirty entities
pub fn optimized_distance_culling_system(
    mut commands: Commands,
    mut cullable_query: Query<(Entity, &mut Cullable, &Transform, &mut Visibility), With<DirtyVisibility>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<Cullable>)>,
    frame_counter: Res<FrameCounter>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    let current_frame = frame_counter.frame;
    
    for (entity, mut cullable, transform, mut visibility) in cullable_query.iter_mut() {
        let distance = active_pos.distance(transform.translation);
        let should_be_culled = distance > cullable.max_distance;
        
        if should_be_culled != cullable.is_culled {
            cullable.is_culled = should_be_culled;
            
            let new_visibility = if should_be_culled {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
            
            *visibility = new_visibility;
            
            // If entity becomes visible, mark LOD as dirty for next frame
            if !should_be_culled {
                commands.entity(entity).insert(DirtyLOD::new(
                    DirtyPriority::Normal,
                    current_frame,
                    distance,
                ));
            }
        }
        
        // Remove dirty visibility flag after processing
        commands.entity(entity).remove::<DirtyVisibility>();
    }
}

/// System to periodically mark all entities for LOD checks (fallback system)
pub fn periodic_lod_marking_system(
    mut commands: Commands,
    entities_query: Query<Entity, (With<Transform>, Without<DirtyLOD>)>,
    frame_counter: Res<FrameCounter>,
    config: Res<GameConfig>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_secs();
    
    // Mark all entities for LOD check every few seconds as a fallback
    let check_interval = 2.0; // Check every 2 seconds
    if *timer < check_interval {
        return;
    }
    *timer = 0.0;
    
    let current_frame = frame_counter.frame;
    let batch_size = config.batching.lod_batch_size / 4; // Process fewer in periodic checks
    
    // Mark a subset of entities each frame to spread the work
    for (i, entity) in entities_query.iter().enumerate().take(batch_size) {
        commands.entity(entity).insert(DirtyLOD::new(
            DirtyPriority::Low, // Low priority for periodic checks
            current_frame,
            0.0, // Distance will be calculated during processing
        ));
        
        // Only process a limited number per frame
        if i >= batch_size {
            break;
        }
    }
}

/// System to automatically mark entities as LOD dirty when they move significantly
pub fn movement_based_lod_marking_system(
    mut commands: Commands,
    moved_entities: Query<(Entity, Ref<Transform>), (Changed<Transform>, Without<DirtyLOD>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<Transform>)>,
    frame_counter: Res<FrameCounter>,
    config: Res<GameConfig>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    let current_frame = frame_counter.frame;
    
    for (entity, transform) in moved_entities.iter() {
        // Only mark as dirty if the entity moved significantly or moved closer/farther
        let distance = active_pos.distance(transform.translation);
        
        // Check if this is a significant movement that could affect LOD
        let _movement_threshold = config.batching.transform_change_threshold;
        let is_significant_movement = transform.is_changed() && 
            transform.as_ref().translation != transform.translation;
        
        if is_significant_movement {
            let priority = if distance < 100.0 {
                DirtyPriority::High // Close entities get high priority
            } else if distance < 300.0 {
                DirtyPriority::Normal
            } else {
                DirtyPriority::Low
            };
            
            commands.entity(entity).insert(DirtyLOD::new(
                priority,
                current_frame,
                distance,
            ));
        }
    }
}

/// Enhanced NPC LOD system with batch processing
pub fn optimized_npc_lod_system(
    mut commands: Commands,
    mut npc_query: Query<(Entity, &mut NPCState, &Transform, Option<&NPCRendering>), With<DirtyLOD>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPCState>)>,
    frame_counter: Res<FrameCounter>,
    time: Res<Time>,
    _config: Res<GameConfig>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    let current_frame = frame_counter.frame;
    
    for (entity, mut npc_state, transform, npc_rendering) in npc_query.iter_mut() {
        let distance = active_pos.distance(transform.translation);
        let new_lod = calculate_npc_lod(distance);
        
        // Only update if LOD actually changed
        if new_lod != npc_state.current_lod {
            npc_state.current_lod = new_lod;
            npc_state.last_lod_check = time.elapsed_secs();
            
            // Handle rendering component changes based on LOD
            match new_lod {
                NPCLOD::Full | NPCLOD::Medium | NPCLOD::Low => {
                    // Should have rendering component
                    if npc_rendering.is_none() {
                        commands.entity(entity).insert(NPCRendering {
                            lod_level: new_lod,
                            body_entities: Vec::new(),
                        });
                    }
                }
                NPCLOD::StateOnly => {
                    // Should not have rendering component
                    if npc_rendering.is_some() {
                        commands.entity(entity).remove::<NPCRendering>();
                    }
                }
            }
            
            // Mark visibility as dirty if LOD changed
            let priority = match new_lod {
                NPCLOD::Full => DirtyPriority::High,
                NPCLOD::Medium => DirtyPriority::Normal,
                NPCLOD::Low => DirtyPriority::Normal,
                NPCLOD::StateOnly => DirtyPriority::Low,
            };
            
            commands.entity(entity).insert(DirtyVisibility::new(
                priority,
                current_frame,
            ));
        }
        
        // Remove dirty LOD flag after processing
        commands.entity(entity).remove::<DirtyLOD>();
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

/// Performance monitoring for optimized LOD systems
pub fn optimized_lod_performance_monitor(
    dirty_lod_count: Query<(), With<DirtyLOD>>,
    dirty_visibility_count: Query<(), With<DirtyVisibility>>,
    dirty_transform_count: Query<(), With<DirtyTransform>>,
    dirty_physics_count: Query<(), With<DirtyPhysics>>,
    npc_query: Query<&NPCState>,
    cullable_query: Query<&Cullable>,
    mut performance_stats: ResMut<PerformanceStats>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    *last_report += time.delta_secs();
    
    if *last_report > 5.0 {
        let dirty_lod = dirty_lod_count.iter().count();
        let dirty_visibility = dirty_visibility_count.iter().count();
        let dirty_transform = dirty_transform_count.iter().count();
        let dirty_physics = dirty_physics_count.iter().count();
        
        let total_npcs = npc_query.iter().count();
        let culled_entities = cullable_query.iter().filter(|c| c.is_culled).count();
        let total_cullable = cullable_query.iter().count();
        
        let npc_lod_counts = npc_query.iter().fold([0; 4], |mut counts, npc| {
            match npc.current_lod {
                NPCLOD::Full => counts[0] += 1,
                NPCLOD::Medium => counts[1] += 1,
                NPCLOD::Low => counts[2] += 1,
                NPCLOD::StateOnly => counts[3] += 1,
            }
            counts
        });
        
        info!(
            "Optimized LOD Performance - Dirty: LOD:{} VIS:{} TRF:{} PHY:{} | NPCs: Total:{} Full:{} Med:{} Low:{} State:{} | Culled: {}/{}",
            dirty_lod, dirty_visibility, dirty_transform, dirty_physics,
            total_npcs, npc_lod_counts[0], npc_lod_counts[1], npc_lod_counts[2], npc_lod_counts[3],
            culled_entities, total_cullable
        );
        
        performance_stats.entity_count = total_cullable + total_npcs;
        performance_stats.culled_entities = culled_entities;
        
        *last_report = 0.0;
    }
}
