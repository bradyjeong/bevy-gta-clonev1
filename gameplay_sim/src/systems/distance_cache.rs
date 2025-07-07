use bevy::prelude::*;
use std::collections::HashMap;

/// Distance cache resource to avoid repeated distance calculations per frame
#[derive(Resource, Default)]
pub struct DistanceCache {
    /// Cache of distance calculations: (entity1, entity2) -> (distance, distance_squared, last_frame)
    cache: HashMap<(Entity, Entity), (f32, f32, u64)>,
    /// Frame counter for cache invalidation
    current_frame: u64,
    /// Maximum cache size to prevent memory bloat
    max_cache_size: usize,
    /// Cache hit/miss statistics for debugging
    pub stats: DistanceCacheStats,
}

#[derive(Default, Debug)]
pub struct DistanceCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub cleanups: u64,
}

impl DistanceCacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

impl DistanceCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(1024),
            current_frame: 0,
            max_cache_size: 2048, // Limit cache size
            stats: DistanceCacheStats::default(),
        }
    }

    /// Advance to the next frame for cache management
    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
        
        // Clean up old entries every 120 frames (~2 seconds at 60 FPS)
        // Spread cleanup over multiple frames to prevent spikes
        if self.current_frame % 120 == 0 {
            self.cleanup_old_entries_gradually();
        }
        
        // Prevent cache from growing too large
        if self.cache.len() > self.max_cache_size {
            self.limit_cache_size();
        }
    }

    /// Get cached distance between two entities, calculating if not cached
    pub fn get_distance(
        &mut self,
        entity1: Entity,
        entity2: Entity,
        pos1: Vec3,
        pos2: Vec3,
    ) -> (f32, f32) {
        // Normalize entity order to ensure consistent keys
        let key = if entity1.index() < entity2.index() {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        };

        // Check if we have a recent cache entry (within 5 frames)
        if let Some((distance, distance_squared, cached_frame)) = self.cache.get(&key) {
            if self.current_frame - cached_frame <= 5 {
                self.stats.hits += 1;
                return (*distance, *distance_squared);
            }
        }

        // Calculate fresh distance
        let distance = pos1.distance(pos2);
        let distance_squared = pos1.distance_squared(pos2);
        
        // Cache the result
        self.cache.insert(key, (distance, distance_squared, self.current_frame));
        self.stats.misses += 1;
        
        (distance, distance_squared)
    }

    /// Get cached distance squared (optimized for performance-critical operations)
    pub fn get_distance_squared(
        &mut self,
        entity1: Entity,
        entity2: Entity,
        pos1: Vec3,
        pos2: Vec3,
    ) -> f32 {
        let (_, distance_squared) = self.get_distance(entity1, entity2, pos1, pos2);
        distance_squared
    }

    /// Clean up old entries gradually to prevent frame spikes
    fn cleanup_old_entries_gradually(&mut self) {
        let cleanup_count = 50; // Clean up at most 50 entries per frame
        let mut removed = 0;
        
        self.cache.retain(|_key, (_, _, cached_frame)| {
            if removed >= cleanup_count {
                true // Keep remaining entries for next cleanup
            } else if self.current_frame - cached_frame > 300 { // 5 seconds at 60 FPS
                removed += 1;
                self.stats.cleanups += 1;
                false // Remove old entry
            } else {
                true // Keep recent entry
            }
        });
    }

    /// Limit cache size by removing oldest entries
    fn limit_cache_size(&mut self) {
        if self.cache.len() <= self.max_cache_size {
            return;
        }

        // Convert to vector, sort by frame, and keep only the most recent entries
        let mut entries: Vec<_> = self.cache.iter().collect();
        entries.sort_by(|a, b| b.1.2.cmp(&a.1.2)); // Sort by frame descending
        
        // Keep only the most recent entries
        let keep_count = self.max_cache_size * 3 / 4; // Keep 75% of max size
        let mut new_cache = HashMap::with_capacity(keep_count);
        
        for (key, value) in entries.into_iter().take(keep_count) {
            new_cache.insert(*key, *value);
        }
        
        self.cache = new_cache;
        self.stats.cleanups += 1;
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &DistanceCacheStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DistanceCacheStats::default();
    }
}

// Import MovementTracker from game_core
use game_core::components::spatial::MovementTracker;

/// System to advance the distance cache frame counter
pub fn distance_cache_management_system(
    mut distance_cache: ResMut<DistanceCache>,
    mut movement_trackers: Query<(&mut MovementTracker, &Transform)>,
) {
    distance_cache.advance_frame();
    
    // Update movement trackers
    for (mut tracker, transform) in movement_trackers.iter_mut() {
        if tracker.has_moved_significantly(transform.translation) {
            tracker.update_position(transform.translation);
        }
    }
}

/// Plugin to add distance caching system
pub struct DistanceCachePlugin;

impl Plugin for DistanceCachePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DistanceCache::new())
            .add_systems(Update, distance_cache_management_system);
    }
}
