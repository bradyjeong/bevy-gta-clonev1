use crate::components::water::Yacht;
use crate::components::{ActiveEntity, MainCamera};
use crate::config::GameConfig;
use crate::systems::swimming::ProneRotation;
use crate::util::safe_math::safe_lerp;
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
        &'static GlobalTransform,
        Option<&'static Velocity>,
        Option<&'static ProneRotation>,
        Option<&'static Yacht>,
    ),
    (With<ActiveEntity>, Without<MainCamera>),
>;

#[allow(clippy::type_complexity)]
pub fn camera_follow_system(
    mut camera_query: Query<
        (&mut Transform, &mut Projection),
        (With<MainCamera>, Without<ActiveEntity>),
    >,
    active_query: ActiveEntityQuery,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    let Ok((active_gt, velocity, prone_rotation, yacht)) = active_query.single() else {
        return;
    };

    if yacht.is_some() {
        yacht_camera_logic(
            &mut camera_transform,
            &mut projection,
            active_gt,
            velocity.unwrap_or(&Velocity::default()),
            &time,
        );
        return;
    }

    let world_pos = active_gt.translation();
    let (_, rot, _) = active_gt.to_scale_rotation_translation();

    // Safety checks for invalid transforms
    if !world_pos.is_finite() || !rot.is_finite() {
        return;
    }

    // Use horizontal forward and world up for all camera positioning
    let forward = rot * Vec3::NEG_Z;
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let world_up = Vec3::Y;

    // Additional safety checks for direction vectors
    if !forward_xz.is_finite() {
        return;
    }

    // Calculate camera position based on prone rotation (swimming camera mode)
    let target_pos = if prone_rotation.is_some() {
        // Swimming: behind the swimmer horizontally, above vertically
        world_pos - forward_xz * config.camera.swim_distance + world_up * config.camera.swim_height
    } else {
        // Walking: traditional behind and above positioning
        world_pos - forward_xz * config.camera.distance + world_up * config.camera.height
    };

    // Safety check for target position
    if !target_pos.is_finite() {
        return;
    }

    // Speed-dependent camera smoothing
    let current_speed = velocity.map_or(0.0, |v| v.linvel.length());

    // Base lerp speed with speed multiplier (faster = more responsive camera)
    let speed_multiplier = 1.0 + (current_speed / 50.0).clamp(0.0, 3.0);
    let dynamic_lerp_speed = config.camera.lerp_speed * speed_multiplier;

    let lerp_factor = (dynamic_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
    camera_transform.translation = safe_lerp(camera_transform.translation, target_pos, lerp_factor);

    // Calculate look target based on prone rotation (swimming camera mode)
    let look_target = if prone_rotation.is_some() {
        // Swimming: look ahead of swimmer's shoulders
        world_pos + forward_xz * config.camera.swim_look_ahead
    } else {
        // Walking: look slightly above ground
        world_pos + world_up * 0.5
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
    yacht_gt: &GlobalTransform,
    velocity: &Velocity,
    time: &Time,
) {
    let world_pos = yacht_gt.translation();
    let (_, rot, _) = yacht_gt.to_scale_rotation_translation();

    if !world_pos.is_finite() || !rot.is_finite() {
        return;
    }

    let forward = rot * Vec3::NEG_Z;
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let world_up = Vec3::Y;

    if !forward_xz.is_finite() {
        return;
    }

    let vel_xz = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
    let speed = vel_xz.length();

    let yacht_distance = 80.0;
    let yacht_height = 25.0;
    let look_ahead_distance = 20.0;

    let target_pos = world_pos - forward_xz * yacht_distance + world_up * yacht_height;

    if !target_pos.is_finite() {
        return;
    }

    let follow_speed = 3.0;
    let lerp_factor = (follow_speed * time.delta_secs()).clamp(0.0, 1.0);
    camera_transform.translation = safe_lerp(camera_transform.translation, target_pos, lerp_factor);

    let velocity_direction = if vel_xz.length() > 0.1 {
        vel_xz.normalize()
    } else {
        forward_xz
    };

    let look_target = world_pos + velocity_direction * look_ahead_distance;

    if !look_target.is_finite() {
        return;
    }

    let right = forward_xz.cross(world_up).normalize_or_zero();
    let lateral_velocity = vel_xz.dot(right);
    let banking_angle =
        (lateral_velocity * 0.05).clamp(-5.0_f32.to_radians(), 5.0_f32.to_radians());

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
