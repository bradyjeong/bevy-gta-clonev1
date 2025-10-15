use crate::components::water::{WaterSurface, Yacht, YachtSpecs};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct YachtSpecsHandle(pub Handle<YachtSpecs>);

pub fn yacht_buoyancy_system(
    water_surface: Res<WaterSurface>,
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

        let yacht_translation = transform.translation();
        let yacht_rotation = transform.to_scale_rotation_translation().1;

        let mut total_buoyancy_force = Vec3::ZERO;
        let mut total_torque = Vec3::ZERO;

        for (local_x, local_y, local_z) in &specs.buoyancy_points {
            let local_point = Vec3::new(*local_x, *local_y, *local_z);
            let world_point = yacht_translation + yacht_rotation * local_point;

            let (water_height, water_normal) = water_surface.sample(world_point);
            let depth = (water_height - world_point.y).max(0.0);

            if depth > 0.0 {
                let weight = specs.mass * 9.81;
                let per_point = weight / specs.buoyancy_points.len() as f32;
                let target = specs.target_submersion.max(0.01);
                let submersion = (depth / target).clamp(0.0, 1.0);
                let buoyancy_force = water_normal.normalize_or_zero() * (per_point * submersion);

                let point_velocity =
                    velocity.linvel + velocity.angvel.cross(world_point - yacht_translation);
                let damping_force = -specs.buoyancy_damping * point_velocity.y * Vec3::Y;

                let total_point_force = buoyancy_force + damping_force;
                total_buoyancy_force += total_point_force;

                let torque = (world_point - yacht_translation).cross(total_point_force);
                total_torque += torque;
            }
        }

        let clamped_force =
            total_buoyancy_force.clamp(Vec3::splat(-500000.0), Vec3::splat(500000.0));
        let clamped_torque = total_torque.clamp(Vec3::splat(-100000.0), Vec3::splat(100000.0));

        if clamped_force.is_finite() && clamped_torque.is_finite() {
            external_force.force += clamped_force;
            external_force.torque += clamped_torque;
        }
    }
}
