use crate::components::water::Yacht;
use crate::components::{ActiveEntity, MainCamera};
use crate::util::safe_math::safe_lerp;
use crate::util::transform_utils::horizontal_forward;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn yacht_camera_system(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<MainCamera>>,
    yacht_query: Query<(&Transform, &Velocity), (With<Yacht>, With<ActiveEntity>, Without<MainCamera>)>,
    time: Res<Time>,
) {
    let Ok((mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    
    let Ok((yacht_transform, velocity)) = yacht_query.single() else {
        return;
    };

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

    if let Projection::Perspective(perspective) = projection.as_mut() {
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
