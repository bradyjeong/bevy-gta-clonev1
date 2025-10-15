use crate::components::water::Yacht;
use crate::components::{ActiveEntity, MainCamera};
use crate::config::GameConfig;
use crate::systems::swimming::ProneRotation;
use crate::util::safe_math::safe_lerp;
use crate::util::transform_utils::horizontal_forward;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Disable 3D camera during Loading state to eliminate rendering overhead
/// Resilient to camera not existing yet
pub fn disable_camera_during_loading(mut camera_query: Query<&mut Camera, With<MainCamera>>) {
    for mut camera in &mut camera_query {
        camera.is_active = false;
    }
    if !camera_query.is_empty() {
        info!("3D camera disabled during loading for performance");
    }
}

/// Re-enable 3D camera when entering InGame state
pub fn enable_camera_for_gameplay(mut camera_query: Query<&mut Camera, With<MainCamera>>) {
    for mut camera in &mut camera_query {
        camera.is_active = true;
    }
    if !camera_query.is_empty() {
        info!("3D camera enabled for gameplay");
    }
}

type ActiveEntityQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        Option<&'static Velocity>,
        Option<&'static ProneRotation>,
        Option<&'static Yacht>,
    ),
    (With<ActiveEntity>, Without<MainCamera>),
>;

pub fn camera_follow_system(
    mut camera_query: Query<(&mut Transform, &mut Projection), (With<MainCamera>, Without<ActiveEntity>)>,
    active_query: ActiveEntityQuery,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let Ok((active_transform, velocity, prone_rotation, yacht)) = active_query.single() else {
        return;
    };

    if yacht.is_some() {
        yacht_camera_logic(&mut camera_transform, &mut projection, active_transform, velocity.unwrap_or(&Velocity::default()), &time);
        return;
    }

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
            + world_up * config.camera.swim_height // above swimmer's back
    } else {
        // Walking: traditional behind and above positioning
        active_transform.translation - forward_xz * config.camera.distance
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
    camera_transform.translation = safe_lerp(camera_transform.translation, target_pos, lerp_factor);

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

fn yacht_camera_logic(
    camera_transform: &mut Transform,
    projection: &mut Projection,
    yacht_transform: &Transform,
    velocity: &Velocity,
    time: &Time,
) {
    if !yacht_transform.translation.is_finite() || !yacht_transform.rotation.is_finite() {
        return;
    }

    let forward_xz = horizontal_forward(yacht_transform);
    let world_up = Vec3::Y;

    if !forward_xz.is_finite() {
        return;
    }

    let forward_speed = velocity.linvel.dot(forward_xz);
    let speed = forward_speed.abs();

    let yacht_distance = 80.0;
    let yacht_height = 25.0;
    let look_ahead_distance = 20.0;

    let target_pos = yacht_transform.translation
        - forward_xz * yacht_distance
        + world_up * yacht_height;

    if !target_pos.is_finite() {
        return;
    }

    let follow_speed = 3.0;
    let lerp_factor = (follow_speed * time.delta_secs()).clamp(0.0, 1.0);
    camera_transform.translation = safe_lerp(camera_transform.translation, target_pos, lerp_factor);

    let velocity_direction = if velocity.linvel.length() > 0.1 {
        velocity.linvel.normalize()
    } else {
        forward_xz
    };

    let look_target = yacht_transform.translation + velocity_direction * look_ahead_distance;

    if !look_target.is_finite() {
        return;
    }

    let right = forward_xz.cross(world_up).normalize_or_zero();
    let lateral_velocity = velocity.linvel.dot(right);
    let banking_angle = (lateral_velocity * 0.05).clamp(-5.0_f32.to_radians(), 5.0_f32.to_radians());
    
    let banking_rotation = Quat::from_axis_angle(forward_xz, -banking_angle);
    let banked_up = banking_rotation * world_up;

    camera_transform.look_at(look_target, banked_up);

    if let Projection::Perspective(perspective) = projection {
        let min_fov = 65.0_f32.to_radians();
        let max_fov = 80.0_f32.to_radians();
        let max_speed = 18.0;
        
        let speed_factor = (speed / max_speed).clamp(0.0, 1.0);
        let target_fov = min_fov + (max_fov - min_fov) * speed_factor;
        
        let fov_lerp_speed = 2.0;
        let fov_lerp_factor = (fov_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
        perspective.fov = perspective.fov.lerp(target_fov, fov_lerp_factor);
    }
}
