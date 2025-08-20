use bevy::prelude::*;
use crate::components::*;
use crate::services::distance_cache::{DistanceCache, get_cached_distance_squared};


/// Configuration for distance-based culling and LOD for different entity types
#[derive(Clone, Debug)]
pub struct DistanceCullingConfig {
    /// Distance thresholds for different LOD levels
    pub lod_distances: Vec<f32>,
    /// Maximum distance before entity is completely culled
    pub cull_distance: f32,
    /// Hysteresis buffer to prevent flickering (applied to all distances)
    pub hysteresis: f32,
    /// How often to check distance (in seconds)
    pub update_interval: f32,
    /// Entity type identifier for debugging
    pub entity_type: &'static str,
}

impl DistanceCullingConfig {
    /// Create config optimized for vehicles
    pub fn vehicle() -> Self {
        Self {
            lod_distances: vec![50.0, 150.0, 300.0], // Full, Medium, Low LOD
            cull_distance: 500.0,
            hysteresis: 5.0,
            update_interval: 0.5,
            entity_type: "Vehicle",
        }
    }

    /// Create config optimized for NPCs
    pub fn npc() -> Self {
        Self {
            lod_distances: vec![25.0, 75.0, 100.0], // Full, Medium, Low LOD
            cull_distance: 150.0,
            hysteresis: 3.0,
            update_interval: 0.3,
            entity_type: "NPC",
        }
    }

    /// Create config optimized for vegetation
    pub fn vegetation() -> Self {
        Self {
            lod_distances: vec![50.0, 150.0, 300.0], // Full, Medium, Billboard
            cull_distance: 400.0,
            hysteresis: 10.0,
            update_interval: 1.0, // Less frequent updates for vegetation
            entity_type: "Vegetation",
        }
    }

    /// Create config optimized for buildings
    pub fn buildings() -> Self {
        Self {
            lod_distances: vec![100.0, 300.0, 500.0], // Buildings visible at longer distances
            cull_distance: 800.0,
            hysteresis: 15.0,
            update_interval: 0.8,
            entity_type: "Building",
        }
    }

    /// Create config optimized for map chunks
    pub fn chunks() -> Self {
        Self {
            lod_distances: vec![150.0, 300.0, 500.0],
            cull_distance: 800.0,
            hysteresis: 20.0,
            update_interval: 0.5,
            entity_type: "Chunk",
        }
    }

    /// Get LOD level for given distance
    pub fn get_lod_level(&self, distance: f32) -> usize {
        for (level, &threshold) in self.lod_distances.iter().enumerate() {
            if distance <= threshold + self.hysteresis {
                return level;
            }
        }
        self.lod_distances.len() // Beyond all LOD levels
    }

    /// Check if entity should be culled
    pub fn should_cull(&self, distance: f32) -> bool {
        distance > self.cull_distance + self.hysteresis
    }
}

/// Component to mark entities that use the unified culling system
#[derive(Component)]
pub struct UnifiedCullable {
    pub config: DistanceCullingConfig,
    pub current_lod: usize,
    pub is_culled: bool,
    pub last_distance: f32,
    pub last_update: f32,
}

impl UnifiedCullable {
    pub fn new(config: DistanceCullingConfig) -> Self {
        Self {
            config,
            current_lod: 0,
            is_culled: false,
            last_distance: 0.0,
            last_update: 0.0,
        }
    }

    pub fn vehicle() -> Self {
        Self::new(DistanceCullingConfig::vehicle())
    }

    pub fn npc() -> Self {
        Self::new(DistanceCullingConfig::npc())
    }

    pub fn vegetation() -> Self {
        Self::new(DistanceCullingConfig::vegetation())
    }

    pub fn building() -> Self {
        Self::new(DistanceCullingConfig::buildings())
    }

    pub fn chunk() -> Self {
        Self::new(DistanceCullingConfig::chunks())
    }

    /// Check if this entity needs an update based on time and distance change
    pub fn needs_update(&self, current_time: f32, current_distance: f32) -> bool {
        let time_elapsed = current_time - self.last_update;
        let distance_changed = (current_distance - self.last_distance).abs() > self.config.hysteresis;
        
        time_elapsed >= self.config.update_interval || distance_changed
    }

    /// Update LOD and culling state
    pub fn update(&mut self, distance: f32, current_time: f32) -> bool {
        let old_lod = self.current_lod;
        let old_culled = self.is_culled;

        self.current_lod = self.config.get_lod_level(distance);
        self.is_culled = self.config.should_cull(distance);
        self.last_distance = distance;
        self.last_update = current_time;

        // Return true if state changed
        old_lod != self.current_lod || old_culled != self.is_culled
    }
}

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
    _commands: Commands,
    frame_counter: Res<FrameCounter>,
) {
    let Ok((active_entity, active_transform)) = active_query.single() else { return };
    let player_pos = active_transform.translation;
    
    timer.elapsed += time.delta_secs();
    let current_time = timer.elapsed;
    let _current_frame = frame_counter.frame;
    
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
            break;
        }
        
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
            let state_changed = cullable.update(distance, current_time);
            
            if state_changed {
                // Update visibility
                *visibility = if cullable.is_culled {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
                
                // LOD and visibility changes are handled directly by UnifiedCullable
            }
            
            processed += 1;
        }
    }
}

// Note: Vehicle LOD is now handled directly by UnifiedCullable component
// No separate adapter system needed

// LEGACY LOD UPDATE COMPONENTS REMOVED - functionality moved to UnifiedCullable

/// Component to mark chunks for unloading
#[derive(Component)]
pub struct ChunkUnloadRequest;

/// Performance monitoring system for unified culling
pub fn unified_culling_performance_monitor(
    cullable_query: Query<&UnifiedCullable>,
    mut performance_stats: ResMut<PerformanceStats>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
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
            }
        }
        
        info!(
            "Unified Culling Performance - Total: {} | Culled: {} | Types: {:?} | LOD Distribution: {:?}",
            total_entities, culled_count, type_counts, lod_counts
        );
        
        performance_stats.entity_count = total_entities;
        performance_stats.culled_entities = culled_count;
        
        *last_report = 0.0;
    }
}

// Note: Movement tracking is now handled directly by UnifiedCullable.needs_update()
// No separate movement tracker needed



/// Plugin to integrate unified distance culling system
pub struct UnifiedDistanceCullingPlugin;

impl Plugin for UnifiedDistanceCullingPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(UnifiedCullingTimer::default())
            .add_systems(Update, (
                // Main culling system handles everything directly
                new_unified_distance_culling_system,
                unified_culling_performance_monitor,
            ).chain().in_set(crate::system_sets::GameSystemSets::ServiceUpdates));
    }
}
