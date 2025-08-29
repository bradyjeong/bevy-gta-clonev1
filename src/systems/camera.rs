use crate::components::{ActiveEntity, MainCamera};
use crate::config::GameConfig;
use crate::util::safe_math::safe_lerp;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ActiveEntity>)>,
    active_query: Query<(&Transform, Option<&Velocity>), (With<ActiveEntity>, Without<MainCamera>)>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };
    let Ok((active_transform, velocity)) = active_query.single() else {
        return;
    };

    // Safety checks for invalid transforms
    if !active_transform.translation.is_finite() || !active_transform.rotation.is_finite() {
        return;
    }

    // Camera follows behind all entities using standard positioning
    let entity_behind_direction = -active_transform.forward();

    // Additional safety check for invalid direction vector
    if !entity_behind_direction.is_finite() {
        return;
    }

    let entity_up = Vec3::Y;

    // Position camera behind and above the entity
    let camera_distance = config.camera.distance;
    let camera_height = config.camera.height;
    let camera_offset = entity_behind_direction * camera_distance + entity_up * camera_height;
    let target_pos = active_transform.translation + camera_offset;

    // Safety check for target position
    if !target_pos.is_finite() {
        return;
    }

    // Speed-dependent camera smoothing
    let current_speed = velocity.map_or(0.0, |v| v.linvel.length());
    
    // Base lerp speed with speed multiplier (faster = more responsive camera)
    let speed_multiplier = 1.0 + (current_speed / 50.0).clamp(0.0, 3.0); // Scale 0-150 units/s to 1x-4x
    let dynamic_lerp_speed = config.camera.lerp_speed * speed_multiplier;
    
    let lerp_factor = (dynamic_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
    camera_transform.translation = safe_lerp(
        camera_transform.translation,
        target_pos,
        lerp_factor,
    );

    // Camera looks toward the entity slightly above ground (closer to parallel)
    let look_target = active_transform.translation + Vec3::Y * 0.5;

    // Safety check for look target
    if !look_target.is_finite() {
        return;
    }

    camera_transform.look_at(look_target, Vec3::Y);
}
