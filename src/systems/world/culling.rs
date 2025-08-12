use bevy::prelude::*;
use crate::components::{Cullable, ActiveEntity, CullingSettings};
use crate::services::distance_cache::{DistanceCache, get_cached_distance_squared};
use crate::events::PlayerChunkChanged;

/// Observer-based culling system that reacts to player position changes
/// Replaces timer-based polling with reactive observer pattern (Bevy 0.16)
pub fn handle_culling_on_player_moved(
    trigger: Trigger<PlayerChunkChanged>,
    mut cullable_query: Query<(Entity, &mut Cullable, &mut Visibility, &Transform), Without<ActiveEntity>>,
    _settings: Res<CullingSettings>,
    mut distance_cache: ResMut<DistanceCache>,
) {
    let event = trigger.event();
    let player_pos = event.position;
    let active_entity = event.entity;
    
    for (entity, mut cullable, mut visibility, transform) in cullable_query.iter_mut() {
        // Use distance_squared for more efficient comparison
        let distance_squared = get_cached_distance_squared(
            active_entity,
            entity,
            player_pos,
            transform.translation,
            &mut distance_cache,
        );
        let max_distance_squared = cullable.max_distance * cullable.max_distance;
        
        if distance_squared > max_distance_squared {
            if !cullable.is_culled {
                cullable.is_culled = true;
                *visibility = Visibility::Hidden;
            }
        } else {
            if cullable.is_culled {
                cullable.is_culled = false;
                *visibility = Visibility::Visible;
            }
        }
    }
}
