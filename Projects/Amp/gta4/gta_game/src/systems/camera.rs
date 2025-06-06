use bevy::prelude::*;
use crate::components::{MainCamera, ActiveEntity};

pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<MainCamera>)>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };
    let Ok(active_transform) = active_query.single() else { return; };
    
    // Safety checks for invalid transforms
    if !active_transform.translation.is_finite() || !active_transform.rotation.is_finite() {
        return;
    }
    
    // Camera follows behind the entity, facing the same direction
    let entity_forward = active_transform.forward();
    
    // Additional safety check for invalid forward vector
    if !entity_forward.is_finite() {
        return;
    }
    
    let entity_up = Vec3::Y;
    
    // Position camera behind and above the entity
    let camera_distance = 20.0;
    let camera_height = 12.0;
    let camera_offset = -entity_forward * camera_distance + entity_up * camera_height;
    let target_pos = active_transform.translation + camera_offset;
    
    // Safety check for target position
    if !target_pos.is_finite() {
        return;
    }
    
    // Smooth camera movement
    camera_transform.translation = camera_transform.translation.lerp(target_pos, 0.05);
    
    // Camera looks forward in the same direction as the entity
    let look_target = active_transform.translation + entity_forward * 10.0 + Vec3::Y * 2.0;
    
    // Safety check for look target
    if !look_target.is_finite() {
        return;
    }
    
    camera_transform.look_at(look_target, Vec3::Y);
}
