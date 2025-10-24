// ==================================================================================
// DEPRECATED: This file is retained for reference but is no longer used in production.
//
// REPLACED BY: src/systems/water/merged_physics.rs::water_physics_system()
//
// REASON FOR DEPRECATION:
// This system and water_drag_system both performed O(n) water region lookups
// for every entity every frame. The merged_physics system combines both:
//   1. Performs region lookup ONCE per entity (not twice)
//   2. Uses CurrentWaterRegion cache for O(1) lookups (not O(n) scans)
//   3. Applies both buoyancy AND drag in a single pass
//
// PERFORMANCE IMPROVEMENT: ~2x faster for water physics (eliminated redundant scans)
// ==================================================================================

use crate::components::unified_water::{UnifiedWaterBody, WaterBodyId};
use crate::components::water::Yacht;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// GTA-style buoyancy system - applies upward force based on submersion
/// DEPRECATED: Use water_physics_system in merged_physics.rs instead
#[allow(clippy::type_complexity)]
pub fn buoyancy_system(
    time: Res<Time>,
    mut query: Query<
        (Entity, &GlobalTransform, &mut ExternalForce, &Collider),
        (With<WaterBodyId>, Without<Yacht>),
    >,
    water_regions: Query<&UnifiedWaterBody>,
) {
    let current_time = time.elapsed_secs();

    for (entity, global_transform, mut external_force, collider) in query.iter_mut() {
        let position = global_transform.translation();

        // Find the water region this entity is in
        let water_region = water_regions
            .iter()
            .find(|region| region.contains_point(position.x, position.z));

        if let Some(region) = water_region {
            // Get collider dimensions (assuming cuboid for simplicity)
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
                    // Apply buoyancy force - proportional to submerged volume
                    let buoyancy_force = 9.81 * 1000.0 * submersion_ratio * half_extents.volume();
                    external_force.force.y += buoyancy_force;

                    // Debug logging for development
                    if submersion_ratio > 0.1 {
                        debug!(
                            "Entity {:?} submersion: {:.2}, buoyancy force: {:.2}",
                            entity, submersion_ratio, buoyancy_force
                        );
                    }
                }
            }
        }
    }
}

// Extension trait for Vec3 volume calculation
trait VolumeExt {
    fn volume(self) -> f32;
}

impl VolumeExt for Vec3 {
    fn volume(self) -> f32 {
        8.0 * self.x * self.y * self.z // 8x because half_extents
    }
}
