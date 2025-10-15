use crate::components::water::{Yacht, YachtSpecs};
use crate::systems::water::yacht_buoyancy::YachtSpecsHandle;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn yacht_drag_system(
    yacht_specs: Res<Assets<YachtSpecs>>,
    mut query: Query<
        (
            &GlobalTransform,
            &mut ExternalForce,
            &Velocity,
            &YachtSpecsHandle,
        ),
        With<Yacht>,
    >,
) {
    for (transform, mut external_force, velocity, specs_handle) in query.iter_mut() {
        let Some(specs) = yacht_specs.get(&specs_handle.0) else {
            continue;
        };

        let rotation = transform.to_scale_rotation_translation().1;
        let local_velocity = rotation.inverse() * velocity.linvel;

        let drag_force_local = Vec3::new(
            -specs.drag_lateral * local_velocity.x * local_velocity.x.abs(),
            -specs.drag_vertical * local_velocity.y * local_velocity.y.abs(),
            -specs.drag_longitudinal * local_velocity.z * local_velocity.z.abs(),
        );

        let drag_force_world = rotation * drag_force_local;
        let yaw_damping_torque =
            -specs.yaw_damping * velocity.angvel.y * velocity.angvel.y.abs() * Vec3::Y;

        let clamped_drag = drag_force_world.clamp(Vec3::splat(-200000.0), Vec3::splat(200000.0));
        let clamped_yaw = yaw_damping_torque.clamp(Vec3::splat(-50000.0), Vec3::splat(50000.0));

        if clamped_drag.is_finite() && clamped_yaw.is_finite() {
            external_force.force += clamped_drag;
            external_force.torque += clamped_yaw;
        }
    }
}
