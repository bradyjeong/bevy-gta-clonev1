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
    ) -> f32 {
        // Ensure consistent ordering for cache key
        let key = if entity1.index() < entity2.index() {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        };

        // Check cache first
        if let Some((distance, _, last_frame)) = self.cache.get(&key) {
            // Cache hit - return cached value if recent
            if self.current_frame - last_frame < 5 { // Valid for 5 frames
                self.stats.hits += 1;
                return *distance;
            }
        }

        // Cache miss - calculate and store
        self.stats.misses += 1;
        let distance = pos1.distance(pos2);
        let distance_squared = pos1.distance_squared(pos2);
        
        self.cache.insert(key, (distance, distance_squared, self.current_frame));
        distance
    }

    /// Get cached distance squared (more efficient for comparisons)
    pub fn get_distance_squared(
        &mut self,
        entity1: Entity,
        entity2: Entity,
        pos1: Vec3,
        pos2: Vec3,
    ) -> f32 {
        // Ensure consistent ordering for cache key
        let key = if entity1.index() < entity2.index() {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        };

        // Check cache first
        if let Some((_, distance_squared, last_frame)) = self.cache.get(&key) {
            // Cache hit - return cached value if recent
            if self.current_frame - last_frame < 5 { // Valid for 5 frames
                self.stats.hits += 1;
                return *distance_squared;
            }
        }

        // Cache miss - calculate and store
        self.stats.misses += 1;
        let distance_squared = pos1.distance_squared(pos2);
        let distance = distance_squared.sqrt();
        
        self.cache.insert(key, (distance, distance_squared, self.current_frame));
        distance_squared
    }

    /// Invalidate all cached distances for a specific entity (when it moves significantly)
    pub fn invalidate_entity_cache(&mut self, entity: Entity) {
        let keys_to_remove: Vec<_> = self.cache
            .keys()
            .filter(|(e1, e2)| *e1 == entity || *e2 == entity)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            self.stats.invalidations += 1;
        }
    }

    /// Clean up entries older than 10 frames gradually
    fn cleanup_old_entries_gradually(&mut self) {
        let cutoff_frame = self.current_frame.saturating_sub(10);
        let _initial_size = self.cache.len();
        
        // Only clean up a limited number of entries per frame (max 50)
        let mut removed = 0;
        const MAX_CLEANUP_PER_FRAME: usize = 50;
        
        self.cache.retain(|_, (_, _, last_frame)| {
            if *last_frame < cutoff_frame && removed < MAX_CLEANUP_PER_FRAME {
                removed += 1;
                false
            } else {
                true
            }
        });
        
        self.stats.cleanups += removed as u64;
    }

    /// Limit cache size by removing oldest entries
    fn limit_cache_size(&mut self) {
        if self.cache.len() <= self.max_cache_size {
            return;
        }

        // Sort by last access time and remove oldest entries
        let mut entries: Vec<_> = self.cache.iter().map(|(k, v)| (*k, *v)).collect();
        entries.sort_by_key(|(_, (_, _, frame))| *frame);

        let to_remove = self.cache.len() - (self.max_cache_size * 3 / 4); // Remove 25% when at limit
        
        for (key, _) in entries.iter().take(to_remove) {
            self.cache.remove(key);
        }
        
        self.stats.cleanups += to_remove as u64;
    }

    /// Get cache size for debugging
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats.cleanups += 1;
    }
}

// Re-export the canonical MovementTracker from game_core
pub use game_core::components::spatial::MovementTracker;

impl DistanceCache {
    /// Get the current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// System to manage the distance cache and track entity movement
pub fn distance_cache_management_system(
    mut cache: ResMut<DistanceCache>,
    mut movement_query: Query<(Entity, &mut MovementTracker, &Transform)>,
) {
    // Advance frame counter
    cache.advance_frame();

    // Check for significantly moved entities and invalidate their cache
    for (entity, mut tracker, transform) in movement_query.iter_mut() {
        if tracker.has_moved_significantly(transform.translation) {
            cache.invalidate_entity_cache(entity);
            tracker.update_position(transform.translation);
        }
    }
}

/// Helper functions for easy distance caching
pub fn get_cached_distance(
    entity1: Entity,
    entity2: Entity,
    pos1: Vec3,
    pos2: Vec3,
    cache: &mut ResMut<DistanceCache>,
) -> f32 {
    cache.get_distance(entity1, entity2, pos1, pos2)
}

pub fn get_cached_distance_squared(
    entity1: Entity,
    entity2: Entity,
    pos1: Vec3,
    pos2: Vec3,
    cache: &mut ResMut<DistanceCache>,
) -> f32 {
    cache.get_distance_squared(entity1, entity2, pos1, pos2)
}

/// Plugin to add distance caching system
pub struct DistanceCachePlugin;

impl Plugin for DistanceCachePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DistanceCache::new())
            .add_systems(Update, distance_cache_management_system);
    }
}
