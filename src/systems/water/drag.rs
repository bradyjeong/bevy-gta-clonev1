use crate::components::unified_water::{UnifiedWaterBody, WaterBodyId};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Water drag system - applies resistance forces when in water
pub fn water_drag_system(
    time: Res<Time>,
    mut query: Query<(Entity, &GlobalTransform, &mut Velocity, &Collider), With<WaterBodyId>>,
    water_regions: Query<&UnifiedWaterBody>,
) {
    let current_time = time.elapsed_secs();

    for (entity, global_transform, mut velocity, collider) in query.iter_mut() {
        let position = global_transform.translation();

        // Find the water region this entity is in
        let water_region = water_regions
            .iter()
            .find(|region| region.contains_point(position.x, position.z));

        if let Some(region) = water_region {
            // Get collider dimensions
            if let Some(cuboid) = collider.as_cuboid() {
                let half_extents = Vec3::new(
                    cuboid.half_extents().x,
                    cuboid.half_extents().y,
                    cuboid.half_extents().z,
                );

                let submersion_ratio = region.calculate_submersion_ratio(
                    &Transform::from_translation(position),
                    half_extents,
                    current_time,
                );

                if submersion_ratio > 0.0 {
                    // Apply water drag - stronger for more submerged objects
                    let drag_coefficient = 0.9 + (submersion_ratio * 0.08); // Base drag + submersion scaling

                    // Linear drag (velocity-based resistance)
                    velocity.linvel *= drag_coefficient.powf(time.delta_secs());

                    // Angular drag (rotation resistance)
                    let angular_drag_coefficient = 0.85 + (submersion_ratio * 0.12);
                    velocity.angvel *= angular_drag_coefficient.powf(time.delta_secs());

                    // Debug logging
                    if submersion_ratio > 0.1 {
                        debug!(
                            "Entity {:?} water drag: linear={:.3}, angular={:.3}",
                            entity, drag_coefficient, angular_drag_coefficient
                        );
                    }
                }
            }
        }
    }
}
