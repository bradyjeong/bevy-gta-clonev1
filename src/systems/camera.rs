use crate::components::{ActiveEntity, MainCamera};
use crate::config::GameConfig;
use crate::systems::swimming::ProneRotation;
use crate::util::safe_math::safe_lerp;
use crate::util::transform_utils::horizontal_forward;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ActiveEntity>)>,
    active_query: Query<(&Transform, Option<&Velocity>, Option<&ProneRotation>), (With<ActiveEntity>, Without<MainCamera>)>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };
    let Ok((active_transform, velocity, prone_rotation)) = active_query.single() else {
        return;
    };

    // Safety checks for invalid transforms
    if !active_transform.translation.is_finite() || !active_transform.rotation.is_finite() {
        return;
    }

    // Use horizontal forward and world up for all camera positioning
    let forward_xz = horizontal_forward(active_transform);
    let world_up = Vec3::Y;

    // Additional safety checks for direction vectors
    if !forward_xz.is_finite() {
        return;
    }

    // Calculate camera position based on prone rotation (swimming camera mode)
    let target_pos = if prone_rotation.is_some() {
        // Swimming: behind the swimmer horizontally, above vertically
        active_transform.translation
            - forward_xz * config.camera.swim_distance  // behind swimmer
            + world_up * config.camera.swim_height      // above swimmer's back
    } else {
        // Walking: traditional behind and above positioning
        active_transform.translation
            - forward_xz * config.camera.distance
            + world_up * config.camera.height
    };

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

    // Calculate look target based on prone rotation (swimming camera mode)
    let look_target = if prone_rotation.is_some() {
        // Swimming: look ahead of swimmer's shoulders
        active_transform.translation + forward_xz * config.camera.swim_look_ahead
    } else {
        // Walking: look slightly above ground
        active_transform.translation + world_up * 0.5
    };

    // Safety check for look target
    if !look_target.is_finite() {
        return;
    }

    // Always use world up to prevent horizon rolling
    camera_transform.look_at(look_target, world_up);
}
