use crate::components::vehicles::Helicopter;
use crate::components::{ActiveEntity, MainCamera};
use crate::util::safe_math::safe_lerp;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[allow(clippy::type_complexity)]
pub fn helicopter_camera_system(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    helicopter_query: Query<
        (&GlobalTransform, &Velocity),
        (With<Helicopter>, With<ActiveEntity>, Without<MainCamera>),
    >,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let Ok((heli_transform, velocity)) = helicopter_query.single() else {
        return;
    };

    let heli_translation = heli_transform.translation();
    // Extract rotation from GlobalTransform
    let (_, heli_rotation, _) = heli_transform.to_scale_rotation_translation();

    if !heli_translation.is_finite() || !heli_rotation.is_finite() {
        return;
    }

    let forward = heli_transform.forward().as_vec3();
    let world_up = Vec3::Y;

    let speed = velocity.linvel.length();

    let heli_distance = 20.0;
    let heli_height = 8.0;
    let look_ahead_distance = 10.0 + speed * 0.2;

    let target_pos = heli_translation - forward * heli_distance + world_up * heli_height;

    if !target_pos.is_finite() {
        return;
    }

    let follow_speed = 4.0;
    let lerp_factor = (follow_speed * time.delta_secs()).clamp(0.0, 1.0);
    camera_transform.translation = safe_lerp(camera_transform.translation, target_pos, lerp_factor);

    // STABILITY FIX: Rely primarily on the helicopter's forward vector for look direction
    // Raw velocity normalization causes jitter due to physics solver noise
    let vel_dir = if velocity.linvel.length() > 5.0 { // Increased threshold from 1.0 to 5.0
        velocity.linvel.normalize()
    } else {
        forward
    };

    // Reduced velocity influence from 0.2 to 0.05 to prioritize stable forward orientation
    let look_dir = forward.lerp(vel_dir, 0.05).normalize_or_zero();
    let look_target = heli_translation + look_dir * look_ahead_distance;

    if !look_target.is_finite() {
        return;
    }

    let right = forward.cross(world_up).normalize_or_zero();
    let lateral_velocity = velocity.linvel.dot(right);
    let banking_angle =
        (lateral_velocity * 0.03).clamp(-4.0_f32.to_radians(), 4.0_f32.to_radians());

    let banking_rotation = Quat::from_axis_angle(forward, -banking_angle);
    let banked_up = banking_rotation * world_up;

    let look_direction = (look_target - camera_transform.translation).normalize_or_zero();
    if look_direction.length_squared() > 0.01 {
        let desired_rotation = Quat::from_rotation_arc(Vec3::NEG_Z, look_direction)
            * Quat::from_rotation_arc(Vec3::Y, banked_up);

        let rotation_lerp_speed = 6.0;
        let rotation_lerp_factor = (rotation_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
        camera_transform.rotation = camera_transform
            .rotation
            .slerp(desired_rotation, rotation_lerp_factor);
    }

    if let Projection::Perspective(perspective) = projection.as_mut() {
        let min_fov = 65.0_f32.to_radians();
        let max_fov = 80.0_f32.to_radians();
        let max_speed = 30.0;

        let speed_factor = (speed / max_speed).clamp(0.0, 1.0);
        let target_fov = min_fov + (max_fov - min_fov) * speed_factor;

        let fov_lerp_speed = 2.0;
        let fov_lerp_factor = (fov_lerp_speed * time.delta_secs()).clamp(0.0, 1.0);
        perspective.fov = perspective.fov.lerp(target_fov, fov_lerp_factor);
    }
}
