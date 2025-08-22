use crate::components::ActiveEntity;
use bevy::prelude::*;

/// Minimal asset streaming system for memory management
/// This handles loading/unloading heavy assets, not visibility (Bevy handles that)

#[derive(Resource, Default)]
pub struct AssetStreamingSettings {
    pub max_loaded_distance: f32,
    pub unload_distance: f32,
    pub max_operations_per_frame: usize,
}

impl AssetStreamingSettings {
    pub fn new() -> Self {
        Self {
            max_loaded_distance: 1000.0, // Load assets within 1km
            unload_distance: 1200.0,     // Unload assets beyond 1.2km
            max_operations_per_frame: 5, // Limit I/O operations per frame
        }
    }
}

#[derive(Component)]
pub struct StreamableAsset {
    pub asset_size: usize, // Memory footprint in bytes
    pub last_check: f32,
}

/// System to unload heavy assets beyond streaming distance
/// This is separate from visibility - Bevy handles rendering culling automatically
pub fn asset_streaming_system(
    time: Res<Time>,
    settings: Res<AssetStreamingSettings>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut streamable_query: Query<(Entity, &Transform, &mut StreamableAsset)>,
    mut commands: Commands,
    mut operations: Local<usize>,
) {
    let current_time = time.elapsed_secs();
    *operations = 0;

    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let player_pos = active_transform.translation;

    for (entity, transform, mut streamable) in streamable_query.iter_mut() {
        // Throttle operations per frame
        if *operations >= settings.max_operations_per_frame {
            break;
        }

        // Check only every 2 seconds to reduce overhead
        if current_time - streamable.last_check < 2.0 {
            continue;
        }
        streamable.last_check = current_time;

        let distance = player_pos.distance(transform.translation);

        // Unload assets beyond streaming distance
        if distance > settings.unload_distance {
            // Remove heavy components but keep entity for state tracking
            commands
                .entity(entity)
                .remove::<Mesh3d>()
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .remove::<bevy_rapier3d::prelude::Collider>();

            *operations += 1;
            info!(
                "Unloaded heavy assets for entity {:?} at {}m distance",
                entity, distance
            );
        }
    }
}

/// System to reload assets when player gets closer
pub fn asset_loading_system(
    time: Res<Time>,
    settings: Res<AssetStreamingSettings>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    // Query entities that don't have Mesh3d but should be loaded
    reload_query: Query<(Entity, &Transform, &StreamableAsset), Without<Mesh3d>>,
    _commands: Commands,
    mut operations: Local<usize>,
) {
    let _current_time = time.elapsed_secs();

    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let player_pos = active_transform.translation;

    for (entity, transform, _streamable) in reload_query.iter() {
        if *operations >= settings.max_operations_per_frame {
            break;
        }

        let distance = player_pos.distance(transform.translation);

        // Reload assets when player gets closer
        if distance < settings.max_loaded_distance {
            // This would need to be integrated with your entity factories
            // to restore the original mesh/material/collider components
            info!(
                "Should reload assets for entity {:?} at {}m distance",
                entity, distance
            );
            *operations += 1;
        }
    }
}
