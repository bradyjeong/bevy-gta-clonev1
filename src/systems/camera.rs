use bevy::prelude::*;
use crate::components::{MainCamera, ActiveEntity};
use crate::config::GameConfig;

pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<MainCamera>)>,
    config: Res<GameConfig>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };
    let Ok(active_transform) = active_query.single() else { return; };
    
    // Safety checks for invalid transforms
    if !active_transform.translation.is_finite() || !active_transform.rotation.is_finite() {
        return;
    }
    
    // Camera follows behind all entities using standard positioning
    let entity_behind_direction = -active_transform.forward();
    
    // Convert Dir3 to Vec3 for calculations
    let entity_behind_vec = entity_behind_direction.as_vec3();
    
    // Additional safety check for invalid direction vector
    if !entity_behind_vec.is_finite() {
        return;
    }
    
    let entity_up = Vec3::Y;
    
    // Position camera behind and above the entity
    let camera_distance = config.gameplay.camera.distance;
    let camera_height = config.gameplay.camera.height;
    let camera_offset = entity_behind_vec * camera_distance + entity_up * camera_height;
    let target_pos = active_transform.translation + camera_offset;
    
    // Safety check for target position
    if !target_pos.is_finite() {
        return;
    }
    
    // Smooth camera movement using interpolation - much more responsive
    camera_transform.translation = camera_transform.translation.lerp(target_pos, config.gameplay.camera.smoothing);
    
    // Camera looks toward the entity at player height (classic GTA style)
    let look_target = active_transform.translation + Vec3::Y * 1.0;
    
    // Safety check for look target
    if !look_target.is_finite() {
        return;
    }
    
    camera_transform.look_at(look_target, Vec3::Y);
}
