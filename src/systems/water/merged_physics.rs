use crate::components::unified_water::{CurrentWaterRegion, UnifiedWaterBody, WaterBodyId};
use crate::components::water::Yacht;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Update cached water region references for entities
/// Only updates when entity moves out of current cached region (rare operation)
/// PERFORMANCE: Filtered to only check entities that moved or are newly added
#[allow(clippy::type_complexity)]
pub fn update_water_region_cache(
    mut query: Query<
        (Entity, &GlobalTransform, Option<&mut CurrentWaterRegion>),
        (
            With<WaterBodyId>,
            Or<(
                Changed<GlobalTransform>,
                Added<CurrentWaterRegion>,
                Added<WaterBodyId>,
            )>,
        ),
    >,
    water_regions: Query<(Entity, &UnifiedWaterBody)>,
    mut commands: Commands,
) {
    for (entity, global_transform, cached_region) in query.iter_mut() {
        let position = global_transform.translation();

        match cached_region {
            Some(mut cache) => {
                // Check if cached region is still valid
                let still_valid = if let Some(cached_entity) = cache.region_entity {
                    if let Ok((_, region)) = water_regions.get(cached_entity) {
                        region.contains_point(position.x, position.z)
                    } else {
                        false // Cached entity no longer exists
                    }
                } else {
                    false
                };

                // Update cache if current region is no longer valid
                if !still_valid {
                    cache.region_entity = water_regions
                        .iter()
                        .find(|(_, region)| region.contains_point(position.x, position.z))
                        .map(|(e, _)| e);
                }
            }
            None => {
                // Initialize cache for new entity
                let region_entity = water_regions
                    .iter()
                    .find(|(_, region)| region.contains_point(position.x, position.z))
                    .map(|(e, _)| e);

                commands
                    .entity(entity)
                    .insert(CurrentWaterRegion { region_entity });
            }
        }
    }
}

/// Merged water physics system - applies both buoyancy and drag in single pass
/// Uses CurrentWaterRegion cache for O(1) region lookup instead of O(n) scanning
/// FALLBACK: If cache is invalid, performs one-time region scan and writes cache
#[allow(clippy::type_complexity)]
pub fn water_physics_system(
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &GlobalTransform,
            &mut ExternalForce,
            &mut Velocity,
            &Collider,
            &mut CurrentWaterRegion,
        ),
        (With<WaterBodyId>, Without<Yacht>),
    >,
    water_regions: Query<(Entity, &UnifiedWaterBody)>,
) {
    let current_time = time.elapsed_secs();

    for (
        entity,
        global_transform,
        mut external_force,
        mut velocity,
        collider,
        mut cached_region,
    ) in query.iter_mut()
    {
        let position = global_transform.translation();

        // O(1) lookup using cached region with fallback to O(n) scan
        let region = match cached_region.region_entity {
            Some(region_entity) => {
                // Try cached region first
                water_regions.get(region_entity).ok().map(|(_, r)| r)
            }
            None => {
                // FALLBACK: Cache invalid, perform one-time region scan and update cache
                let found = water_regions
                    .iter()
                    .find(|(_, region)| region.contains_point(position.x, position.z));

                if let Some((found_entity, region)) = found {
                    // Update cache to avoid repeated scans
                    cached_region.region_entity = Some(found_entity);
                    Some(region)
                } else {
                    None
                }
            }
        };

        if let Some(region) = region {
            // Get collider dimensions (assuming cuboid for simplicity)
            if let Some(cuboid) = collider.as_cuboid() {
                let half_extents = Vec3::new(
                    cuboid.half_extents().x,
                    cuboid.half_extents().y,
                    cuboid.half_extents().z,
                );

                // OPTIMIZATION: Calculate submersion ratio once for both buoyancy and drag
                let submersion_ratio = region.calculate_submersion_ratio(
                    &Transform::from_translation(position),
                    half_extents,
                    current_time,
                );

                if submersion_ratio > 0.0 {
                    // BUOYANCY: Apply upward force based on submerged volume
                    let buoyancy_force = 9.81 * 1000.0 * submersion_ratio * half_extents.volume();
                    external_force.force.y += buoyancy_force;

                    // DRAG: Apply resistance forces
                    let drag_coefficient: f32 = 0.9 + (submersion_ratio * 0.08);
                    velocity.linvel *= drag_coefficient.powf(time.delta_secs());

                    let angular_drag_coefficient: f32 = 0.85 + (submersion_ratio * 0.12);
                    velocity.angvel *= angular_drag_coefficient.powf(time.delta_secs());

                    // Debug logging for development
                    if submersion_ratio > 0.1 {
                        debug!(
                            "Entity {:?} submersion: {:.2}, buoyancy: {:.2}, drag: {:.3}/{:.3}",
                            entity,
                            submersion_ratio,
                            buoyancy_force,
                            drag_coefficient,
                            angular_drag_coefficient
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
