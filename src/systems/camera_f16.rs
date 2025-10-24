use crate::components::vehicles::F16;
use crate::components::{ActiveEntity, MainCamera};
use crate::util::safe_math::safe_lerp;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[allow(clippy::type_complexity)]
pub fn f16_camera_system(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    f16_query: Query<(&Transform, &Velocity), (With<F16>, With<ActiveEntity>, Without<MainCamera>)>,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let Ok((f16_transform, velocity)) = f16_query.single() else {
        return;
    };

    if !f16_transform.translation.is_finite() || !f16_transform.rotation.is_finite() {
        return;
    }

    let forward = f16_transform.forward().as_vec3();
    let world_up = Vec3::Y;

    let speed = velocity.linvel.length();

    let f16_distance = 30.0;
    let f16_height = 12.0;
    let look_ahead_distance = 20.0 + speed * 0.15;

    let target_pos = f16_transform.translation - forward * f16_distance + world_up * f16_height;

    if !target_pos.is_finite() {
        return;
    }

    let follow_speed = 3.5;
    let lerp_factor = (follow_speed * time.delta_secs()).clamp(0.0, 1.0);
    camera_transform.translation = safe_lerp(camera_transform.translation, target_pos, lerp_factor);

    let vel_dir = if velocity.linvel.length() > 2.0 {
        velocity.linvel.normalize()
    } else {
        forward
    };

    let look_dir = forward.lerp(vel_dir, 0.15).normalize_or_zero();
    let look_target = f16_transform.translation + look_dir * look_ahead_distance;

    if !look_target.is_finite() {
        return;
    }

    let right = forward.cross(world_up).normalize_or_zero();
    let lateral_velocity = velocity.linvel.dot(right);
    let banking_angle =
        (lateral_velocity * 0.02).clamp(-6.0_f32.to_radians(), 6.0_f32.to_radians());

    let banking_rotation = Quat::from_axis_angle(forward, -banking_angle);
    let banked_up = banking_rotation * world_up;

    let look_direction = (look_target - camera_transform.translation).normalize_or_zero();
    if look_direction.length_squared() > 0.01 {
        let desired_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, look_direction)
            * Quat::from_rotation_arc(Vec3::Y, banked_up);

        let rotation_lerp_speed = 5.0;
        let rotation_lerp_factor = (rotation_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
        camera_transform.rotation = camera_transform
            .rotation
            .slerp(desired_rotation, rotation_lerp_factor);
    }

    if let Projection::Perspective(perspective) = projection.as_mut() {
        let min_fov = 60.0_f32.to_radians();
        let max_fov = 90.0_f32.to_radians();
        let max_speed = 100.0;

        let speed_factor = (speed / max_speed).clamp(0.0, 1.0);
        let target_fov = min_fov + (max_fov - min_fov) * speed_factor;

        let fov_lerp_speed = 3.0;
        let fov_lerp_factor = (fov_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
        perspective.fov = perspective.fov.lerp(target_fov, fov_lerp_factor);
    }
}
