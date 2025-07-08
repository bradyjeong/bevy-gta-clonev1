//! ───────────────────────────────────────────────
//! System:   Unified LOD
//! Purpose:  Manages level-of-detail for all entities
//! Schedule: Update
//! Reads:    `ActiveEntity`, Transform, Cullable
//! Writes:   Visibility, LOD state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashMap;
use game_core::prelude::*;

const MAX_FRAME_TIME: f32 = 3.0; // 3ms frame budget

#[derive(Resource, Default)]
pub struct LODCoordinator {
    pub stats: LODPerformanceStats,
    pub config: LODPluginConfig,
    pub frame_counter: FrameCounter,
    pub distance_cache: DistanceCache,
}

#[derive(Default)]
pub struct LODPerformanceStats {
    pub entities_processed: usize,
    pub entities_culled: usize,
    pub frame_time_ms: f32,
}

#[derive(Default)]
pub struct LODPluginConfig {
    pub building_distance: f32,
    pub vehicle_distance: f32,
    pub npc_distance: f32,
    pub vegetation_distance: f32,
}

impl LODPluginConfig {
    #[must_use] pub fn new() -> Self {
        Self {
            building_distance: 500.0,
            vehicle_distance: 300.0,
            npc_distance: 150.0,
            vegetation_distance: 200.0,
        }
    }
}

#[derive(Default)]
pub struct FrameCounter {
    pub frame: u64,
}

#[derive(Default)]
pub struct DistanceCache {
    pub cache: HashMap<Entity, (f32, u64)>, // distance, frame_calculated
    pub max_entries: usize,
}

impl DistanceCache {
    #[must_use] pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_entries: 2048,
        }
    }
    
    pub fn get_distance(&mut self, entity: Entity, current_frame: u64, entity_pos: Vec3, active_pos: Vec3) -> f32 {
        if let Some(&(cached_distance, frame)) = self.cache.get(&entity) {
            if current_frame.saturating_sub(frame) < 5 { // Cache for 5 frames
                return cached_distance;
            }
        }
        
        let distance = entity_pos.distance(active_pos);
        
        // Limit cache size
        if self.cache.len() >= self.max_entries {
            self.cache.clear();
        }
        
        self.cache.insert(entity, (distance, current_frame));
        distance
    }
}

pub fn unified_lod_system(
    commands: Commands,
    active_query: Query<(Entity, &Transform), With<ActiveEntity>>,
    mut cullable_query: Query<(Entity, &Transform, &mut Cullable)>,
    mut lod_coordinator: ResMut<LODCoordinator>,
    time: Res<Time>,
) {
    if let Ok((active_entity, active_transform)) = active_query.single() {
        let active_pos = active_transform.translation;
        let start_time = std::time::Instant::now();
        
        lod_coordinator.frame_counter.frame += 1;
        lod_coordinator.stats.entities_processed = 0;
        lod_coordinator.stats.entities_culled = 0;
        
        // Process cullable entities
        for (entity, transform, mut cullable) in &mut cullable_query {
            if entity == active_entity {
                continue; // Don't cull the active entity
            }
            
            let frame = lod_coordinator.frame_counter.frame;
            let distance = lod_coordinator.distance_cache.get_distance(
                entity,
                frame,
                transform.translation,
                active_pos,
            );
            
            let should_cull = distance > cullable.max_distance;
            
            if cullable.is_culled != should_cull {
                cullable.is_culled = should_cull;
                
                if should_cull {
                    lod_coordinator.stats.entities_culled += 1;
                }
            }
            
            lod_coordinator.stats.entities_processed += 1;
            
            // Frame time budget check
            if start_time.elapsed().as_millis() as f32 > MAX_FRAME_TIME {
                break;
            }
        }
        
        lod_coordinator.stats.frame_time_ms = start_time.elapsed().as_millis() as f32;
    }
}

pub fn visibility_update_system(
    mut visibility_query: Query<(&mut Visibility, &Cullable)>,
) {
    for (mut visibility, cullable) in &mut visibility_query {
        if cullable.is_culled {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
        }
    }
}

pub fn lod_stats_system(
    lod_coordinator: Res<LODCoordinator>,
    time: Res<Time>,
) {
    // Log performance stats every 5 seconds
    if time.elapsed_secs() % 5.0 < 0.1 {
        info!(
            "LOD Stats - Processed: {} | Culled: {} | Frame Time: {:.1}ms",
            lod_coordinator.stats.entities_processed,
            lod_coordinator.stats.entities_culled,
            lod_coordinator.stats.frame_time_ms
        );
    }
}

// Component for LOD-aware entities
#[derive(Component)]
pub struct LODEntity {
    pub entity_type: EntityType,
    pub base_distance: f32,
    pub current_lod: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Building,
    Vehicle,
    NPC,
    Vegetation,
}

impl LODEntity {
    #[must_use] pub fn new(entity_type: EntityType) -> Self {
        let base_distance = match entity_type {
            EntityType::Building => 500.0,
            EntityType::Vehicle => 300.0,
            EntityType::NPC => 150.0,
            EntityType::Vegetation => 200.0,
        };
        
        Self {
            entity_type,
            base_distance,
            current_lod: 0,
        }
    }
}

pub fn lod_level_system(
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut lod_query: Query<(&Transform, &mut LODEntity)>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        
        for (transform, mut lod_entity) in &mut lod_query {
            let distance = active_pos.distance(transform.translation);
            let new_lod = calculate_lod_level(distance, lod_entity.base_distance);
            
            if new_lod != lod_entity.current_lod {
                lod_entity.current_lod = new_lod;
                // Could trigger mesh swapping, texture changes, etc.
            }
        }
    }
}

fn calculate_lod_level(distance: f32, base_distance: f32) -> usize {
    if distance < base_distance * 0.5 {
        0 // High detail
    } else if distance < base_distance {
        1 // Medium detail
    } else if distance < base_distance * 2.0 {
        2 // Low detail
    } else {
        3 // Very low detail or culled
    }
}
