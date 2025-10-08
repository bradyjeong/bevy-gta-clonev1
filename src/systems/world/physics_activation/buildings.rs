use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{ActiveEntity, Building};
use crate::config::GameConfig;

/// Physics activation radius - buildings within this distance get physics
const PHYSICS_ACTIVATION_RADIUS: f32 = 200.0;

/// Physics deactivation radius - buildings beyond this distance lose physics
const PHYSICS_DEACTIVATION_RADIUS: f32 = 250.0;

/// Marker component for buildings that have physics active
#[derive(Component)]
pub struct PhysicsActive;

/// Activate physics for nearby buildings (GTA-style dynamic physics)
/// Only buildings within 200m of player get RigidBody + Collider
#[allow(clippy::type_complexity)]
pub fn activate_nearby_building_physics(
    mut commands: Commands,
    config: Res<GameConfig>,
    player_query: Query<&GlobalTransform, With<ActiveEntity>>,
    buildings_without_physics: Query<
        (Entity, &GlobalTransform, &Building),
        (Without<RigidBody>, Without<PhysicsActive>),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation();
    let activation_radius_sq = PHYSICS_ACTIVATION_RADIUS.powi(2);

    let mut activated_count = 0;
    const MAX_ACTIVATIONS_PER_FRAME: usize = 100;

    for (entity, building_transform, building) in &buildings_without_physics {
        if activated_count >= MAX_ACTIVATIONS_PER_FRAME {
            break;
        }

        let distance_sq = player_pos.distance_squared(building_transform.translation());

        if distance_sq < activation_radius_sq {
            // Add physics components to nearby building
            let half_scale = building.scale / 2.0;

            commands.entity(entity).insert((
                RigidBody::Fixed,
                Collider::cuboid(half_scale.x, half_scale.y, half_scale.z),
                CollisionGroups::new(config.physics.static_group, Group::ALL),
                PhysicsActive,
            ));

            activated_count += 1;
        }
    }

    if activated_count > 0 {
        debug!("Activated physics for {} buildings", activated_count);
    }
}

/// Deactivate physics for distant buildings to maintain performance
/// Removes RigidBody + Collider from buildings beyond 250m
#[allow(clippy::type_complexity)]
pub fn deactivate_distant_building_physics(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<ActiveEntity>>,
    buildings_with_physics: Query<
        (Entity, &GlobalTransform),
        (With<Building>, With<PhysicsActive>),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation();
    let deactivation_radius_sq = PHYSICS_DEACTIVATION_RADIUS.powi(2);

    let mut deactivated_count = 0;
    const MAX_DEACTIVATIONS_PER_FRAME: usize = 200;

    for (entity, building_transform) in &buildings_with_physics {
        if deactivated_count >= MAX_DEACTIVATIONS_PER_FRAME {
            break;
        }

        let distance_sq = player_pos.distance_squared(building_transform.translation());

        if distance_sq > deactivation_radius_sq {
            // Remove physics components from distant building
            commands
                .entity(entity)
                .remove::<RigidBody>()
                .remove::<Collider>()
                .remove::<CollisionGroups>()
                .remove::<PhysicsActive>();

            deactivated_count += 1;
        }
    }

    if deactivated_count > 0 {
        debug!("Deactivated physics for {} buildings", deactivated_count);
    }
}

/// Debug system to report physics activation stats
#[allow(dead_code)]
pub fn debug_physics_activation_stats(
    buildings_with_physics: Query<&Building, With<PhysicsActive>>,
    buildings_total: Query<&Building>,
) {
    let active_count = buildings_with_physics.iter().count();
    let total_count = buildings_total.iter().count();

    if active_count > 0 || total_count > 0 {
        debug!(
            "Physics Active Buildings: {}/{} ({:.1}%)",
            active_count,
            total_count,
            (active_count as f32 / total_count.max(1) as f32) * 100.0
        );
    }
}
