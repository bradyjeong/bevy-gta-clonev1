//! ───────────────────────────────────────────────
//! System:   Culling
//! Purpose:  Manages entity visibility based on distance
//! Schedule: Update (throttled)
//! Reads:    `ActiveEntity`, Transform, Cullable, Time, `CullingSettings`
//! Writes:   Visibility, `DistanceCache`
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Only active entities can be controlled
//!   * Timing intervals are respected
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use game_core::prelude::{Cullable, ActiveEntity, CullingSettings};
use crate::systems::distance_cache::{DistanceCache, get_cached_distance_squared};

#[derive(Default)]
pub struct CullingTimer {
    timer: f32,
}

pub fn distance_culling_system(
    mut cullable_query: Query<(Entity, &mut Cullable, &mut Visibility, &Transform), Without<ActiveEntity>>,
    active_query: Query<(Entity, &Transform), (With<ActiveEntity>, Without<Cullable>)>,
    _settings: Res<CullingSettings>,
    time: Res<Time>,
    mut timer: Local<CullingTimer>,
    mut distance_cache: ResMut<DistanceCache>,
) {
    let Ok((active_entity, active_transform)) = active_query.single() else { return; };
    let player_pos = active_transform.translation;
    
    // Update timer
    timer.timer += time.delta_secs();
    
    // Only check culling every 0.5 seconds to reduce overhead
    if timer.timer < 0.5 {
        return;
    }
    timer.timer = 0.0;
    
    for (entity, mut cullable, mut visibility, transform) in &mut cullable_query {
        // Use distance_squared for more efficient comparison
        let distance_squared = get_cached_distance_squared(
            &mut distance_cache,
            entity,
            active_entity,
            transform.translation,
            player_pos,
        );
        let max_distance_squared = cullable.max_distance * cullable.max_distance;
        
        if distance_squared > max_distance_squared {
            if !cullable.is_culled {
                cullable.is_culled = true;
                *visibility = Visibility::Hidden;
            }
        } else if cullable.is_culled {
            cullable.is_culled = false;
            *visibility = Visibility::Visible;
        }
    }
}
