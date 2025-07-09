//! ───────────────────────────────────────────────
//! System:   Unified Distance Culling
//! Purpose:  Manages distance-based culling for all entities
//! Schedule: Update
//! Reads:    `ActiveEntity`, Transform, `UnifiedCullable`
//! Writes:   Visibility, `UnifiedCullable`
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::*;

const MAX_ENTITIES_PER_FRAME: usize = 100;
const DISTANCE_CACHE_SIZE: usize = 2048;

#[derive(Component, Debug, Clone)]
pub struct UnifiedCullable {
    pub max_distance: f32,
    pub current_lod: usize,
    pub is_culled: bool,
    pub last_distance: f32,
    pub dirty: bool,
}

impl Default for UnifiedCullable {
    fn default() -> Self {
        Self {
            max_distance: 500.0,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            dirty: true,
        }
    }
}

impl UnifiedCullable {
    #[must_use] pub fn vehicle() -> Self {
        Self {
            max_distance: 200.0,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            dirty: true,
        }
    }
    
    #[must_use] pub fn building() -> Self {
        Self {
            max_distance: 300.0,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            dirty: true,
        }
    }
    
    #[must_use] pub fn npc() -> Self {
        Self {
            max_distance: 100.0,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            dirty: true,
        }
    }
    
    #[must_use] pub fn tree() -> Self {
        Self {
            max_distance: 150.0,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            dirty: true,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct DirtyVisibility {
    pub priority: DirtyPriority,
}

impl DirtyVisibility {
    #[must_use] pub fn new(priority: DirtyPriority) -> Self {
        Self { priority }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DirtyPriority {
    High,
    Normal,
    Low,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DirtyLOD;

#[derive(Resource, Default, Debug, Clone)]
pub struct DistanceCullingStats {
    pub entities_processed: usize,
    pub entities_culled: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

pub fn unified_distance_culling_system(
    mut commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut cullable_query: Query<(Entity, &mut UnifiedCullable, &Transform, &mut Visibility)>,
    mut stats: ResMut<DistanceCullingStats>,
    _time: Res<Time>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        let mut processed = 0;
        
        stats.entities_processed = 0;
        stats.entities_culled = 0;
        
        for (entity, mut cullable, transform, mut visibility) in &mut cullable_query {
            if processed >= MAX_ENTITIES_PER_FRAME {
                break;
            }
            
            let distance = active_pos.distance(transform.translation);
            let should_cull = distance > cullable.max_distance;
            
            let state_changed = cullable.is_culled != should_cull || 
                               (distance - cullable.last_distance).abs() > 10.0;
            
            if state_changed {
                cullable.is_culled = should_cull;
                cullable.last_distance = distance;
                cullable.dirty = true;
                
                // Update visibility
                *visibility = if should_cull {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
                
                // Mark for further processing if needed
                commands.entity(entity).insert(DirtyVisibility::new(
                    DirtyPriority::Normal,
                ));
                
                if should_cull {
                    stats.entities_culled += 1;
                }
            }
            
            processed += 1;
            stats.entities_processed += 1;
        }
    }
}

pub fn unified_vehicle_lod_system(
    vehicle_query: Query<(Entity, &UnifiedCullable, &VehicleState), (With<DirtyLOD>, Changed<UnifiedCullable>)>,
) {
    for (_entity, cullable, _vehicle_state) in vehicle_query.iter() {
        // Determine appropriate LOD level based on distance and vehicle state
        let _target_lod = if cullable.last_distance < 100.0 {
            0 // High detail
        } else if cullable.last_distance < 300.0 {
            1 // Medium detail
        } else {
            2 // Low detail
        };
        
        // In a real implementation, you'd update the vehicle's mesh/materials here
        // based on the target LOD level
    }
}

pub fn unified_building_lod_system(
    building_query: Query<(Entity, &UnifiedCullable), (With<Building>, Changed<UnifiedCullable>)>,
) {
    for (_entity, cullable) in building_query.iter() {
        // Determine building LOD based on distance
        let _target_lod = calculate_building_lod(cullable.last_distance);
        
        // In a real implementation, you'd update building detail here
    }
}

pub fn unified_npc_lod_system(
    npc_query: Query<(Entity, &UnifiedCullable), (With<NPC>, Changed<UnifiedCullable>)>,
) {
    for (_entity, cullable) in npc_query.iter() {
        // Determine NPC LOD based on distance
        let _target_lod = if cullable.last_distance < 50.0 {
            0 // Full detail
        } else if cullable.last_distance < 150.0 {
            1 // Reduced detail
        } else {
            2 // Billboard or hidden
        };
        
        // In a real implementation, you'd update NPC representation here
    }
}

pub fn dirty_visibility_processor_system(
    mut commands: Commands,
    dirty_query: Query<(Entity, &DirtyVisibility)>,
) {
    // Process entities that need visibility updates
    for (entity, dirty_vis) in dirty_query.iter() {
        // Priority-based processing could be implemented here
        match dirty_vis.priority {
            DirtyPriority::High => {
                // Process immediately
            }
            DirtyPriority::Normal => {
                // Process in normal queue
            }
            DirtyPriority::Low => {
                // Process when resources allow
            }
        }
        
        // Remove the dirty marker after processing
        commands.entity(entity).remove::<DirtyVisibility>();
    }
}

pub fn culling_stats_system(
    stats: Res<DistanceCullingStats>,
    time: Res<Time>,
) {
    // Log stats every 5 seconds
    if time.elapsed_secs() % 5.0 < 0.1 {
        info!(
            "Culling Stats - Processed: {} | Culled: {} | Cache Hits: {} | Cache Misses: {}",
            stats.entities_processed,
            stats.entities_culled,
            stats.cache_hits,
            stats.cache_misses
        );
    }
}

// Utility functions

fn calculate_building_lod(distance: f32) -> usize {
    if distance < 200.0 {
        0 // High detail
    } else if distance < 500.0 {
        1 // Medium detail
    } else {
        2 // Low detail
    }
}

pub fn mark_entity_for_lod_update(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(DirtyLOD);
}

pub fn cleanup_culled_entities_system(
    _commands: Commands,
    cullable_query: Query<Entity, (With<UnifiedCullable>, With<DynamicContent>)>,
    active_query: Query<&Transform, With<ActiveEntity>>,
) {
    if let Ok(active_transform) = active_query.single() {
        let _active_pos = active_transform.translation;
        let _cleanup_distance = 2000.0; // Very conservative cleanup distance
        
        for _entity in cullable_query.iter() {
            // In a real implementation, you'd check the entity's position
            // and despawn it if it's too far away
            // This is just a placeholder for the structure
        }
    }
}

// Component bundles for easy spawning

#[derive(Bundle)]
pub struct CullableEntityBundle {
    pub cullable: UnifiedCullable,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl CullableEntityBundle {
    #[must_use] pub fn new(position: Vec3, max_distance: f32) -> Self {
        Self {
            cullable: UnifiedCullable {
                max_distance,
                ..default()
            },
            transform: Transform::from_translation(position),
            visibility: Visibility::Visible,
        }
    }
}

// Resource for managing culling configuration

#[derive(Resource)]
pub struct CullingConfig {
    pub max_entities_per_frame: usize,
    pub cache_size: usize,
    pub update_frequency: f32,
}

impl Default for CullingConfig {
    fn default() -> Self {
        Self {
            max_entities_per_frame: MAX_ENTITIES_PER_FRAME,
            cache_size: DISTANCE_CACHE_SIZE,
            update_frequency: 0.1, // Update every 0.1 seconds
        }
    }
}
