use crate::services::distance_cache::DistanceCache;
use bevy::prelude::*;

/// Debug system to display distance cache performance statistics
pub fn distance_cache_debug_system(
    cache: Res<DistanceCache>,
    mut timer: Local<f32>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Update timer
    *timer += time.delta_secs();

    // Only output stats every 5 seconds OR when F3 is pressed
    let should_output = *timer >= 5.0 || keyboard_input.just_pressed(KeyCode::F3);

    if should_output {
        *timer = 0.0;

        let stats = &cache.stats;
        let hit_rate = stats.hit_rate() * 100.0;
        let total_requests = stats.hits + stats.misses;

        if total_requests > 0 {
            info!(
                "DISTANCE CACHE STATS:\n\
Hit Rate: {:.1}% ({}/{} requests)\n\
Cache Size: {} entries\n\
Invalidations: {}\n\
Cleanups: {}",
                hit_rate,
                stats.hits,
                total_requests,
                cache.cache_size(),
                stats.invalidations,
                stats.cleanups,
            );
        }
    }
}

/// Plugin to add distance cache debugging
pub struct DistanceCacheDebugPlugin;

impl Plugin for DistanceCacheDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, distance_cache_debug_system);
    }
}
