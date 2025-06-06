use bevy::prelude::*;
use crate::components::{Cullable, ActiveEntity, CullingSettings};

#[derive(Default)]
pub struct CullingTimer {
    timer: f32,
}

pub fn distance_culling_system(
    mut cullable_query: Query<(&mut Cullable, &mut Visibility, &Transform), Without<ActiveEntity>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<Cullable>)>,
    _settings: Res<CullingSettings>,
    time: Res<Time>,
    mut timer: Local<CullingTimer>,
) {
    let Ok(active_transform) = active_query.single() else { return; };
    let player_pos = active_transform.translation;
    
    // Update timer
    timer.timer += time.delta_secs();
    
    // Only check culling every 0.5 seconds to reduce overhead
    if timer.timer < 0.5 {
        return;
    }
    timer.timer = 0.0;
    
    for (mut cullable, mut visibility, transform) in cullable_query.iter_mut() {
        let distance = player_pos.distance(transform.translation);
        
        if distance > cullable.max_distance {
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
