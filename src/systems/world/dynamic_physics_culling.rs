use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{ActiveEntity, Car, F16, Helicopter, NPC, Yacht};

/// Distance where dynamic objects get full physics
const FULL_PHYSICS_RADIUS: f32 = 100.0;

/// Distance where physics gets disabled completely
const PHYSICS_DISABLE_RADIUS: f32 = 300.0;

/// Hysteresis buffer to prevent rapid toggling
const HYSTERESIS_BUFFER: f32 = 50.0;

/// Disable physics for distant vehicles and NPCs (GTA-style optimization)
/// Excludes player-controlled entities (ActiveEntity marker)
#[allow(clippy::type_complexity)]
pub fn disable_distant_dynamic_physics(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<ActiveEntity>>,
    dynamic_entities: Query<
        (Entity, &GlobalTransform),
        (
            Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>, With<NPC>)>,
            With<RigidBody>,
            Without<RigidBodyDisabled>,
            Without<ActiveEntity>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation();
    let disable_radius_sq = PHYSICS_DISABLE_RADIUS.powi(2);

    let mut disabled_count = 0;
    const MAX_DISABLES_PER_FRAME: usize = 50;

    for (entity, transform) in &dynamic_entities {
        if disabled_count >= MAX_DISABLES_PER_FRAME {
            break;
        }

        let distance_sq = player_pos.distance_squared(transform.translation());

        if distance_sq > disable_radius_sq {
            commands.entity(entity).insert(RigidBodyDisabled);
            disabled_count += 1;
        }
    }

    if disabled_count > 0 {
        debug!("Disabled physics for {} dynamic entities", disabled_count);
    }
}

/// Re-enable physics for nearby vehicles and NPCs when player approaches
#[allow(clippy::type_complexity)]
pub fn enable_nearby_dynamic_physics(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<ActiveEntity>>,
    disabled_entities: Query<
        (Entity, &GlobalTransform),
        (
            Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>, With<NPC>)>,
            With<RigidBodyDisabled>,
            Without<ActiveEntity>,
        ),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation();
    let enable_radius_sq = (FULL_PHYSICS_RADIUS + HYSTERESIS_BUFFER).powi(2);

    let mut enabled_count = 0;
    const MAX_ENABLES_PER_FRAME: usize = 50;

    for (entity, transform) in &disabled_entities {
        if enabled_count >= MAX_ENABLES_PER_FRAME {
            break;
        }

        let distance_sq = player_pos.distance_squared(transform.translation());

        if distance_sq < enable_radius_sq {
            commands.entity(entity).remove::<RigidBodyDisabled>();
            enabled_count += 1;
        }
    }

    if enabled_count > 0 {
        debug!("Enabled physics for {} dynamic entities", enabled_count);
    }
}

/// Debug system to report dynamic physics culling stats
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn debug_dynamic_physics_stats(
    active_physics: Query<
        Entity,
        (
            Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>, With<NPC>)>,
            With<RigidBody>,
            Without<RigidBodyDisabled>,
        ),
    >,
    disabled_physics: Query<
        Entity,
        (
            Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>, With<NPC>)>,
            With<RigidBodyDisabled>,
        ),
    >,
) {
    let active_count = active_physics.iter().count();
    let disabled_count = disabled_physics.iter().count();
    let total = active_count + disabled_count;

    if total > 0 {
        debug!(
            "Dynamic Physics: {}/{} active ({:.1}%)",
            active_count,
            total,
            (active_count as f32 / total as f32) * 100.0
        );
    }
}
