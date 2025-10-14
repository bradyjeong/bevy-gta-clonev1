#![allow(clippy::type_complexity)]
use crate::bundles::PlayerPhysicsBundle;
use crate::components::world::WorldBounds;
use crate::components::{ActiveEntity, Player};
use crate::constants::{LAND_ELEVATION, MAX_WORLD_COORDINATE, OCEAN_FLOOR_DEPTH};
use bevy::prelude::*;

/// Basic safety system - only handles extreme cases, avoids velocity manipulation
/// Only triggers when player falls BELOW ocean floor (not AT ocean floor)
pub fn player_collision_resolution_system(
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, With<ActiveEntity>)>,
    mut commands: Commands,
    world_bounds: Option<Res<WorldBounds>>,
) {
    let Ok((player_entity, mut player_transform)) = player_query.single_mut() else {
        return;
    };

    let player_position = player_transform.translation;

    // Only trigger 2m BELOW ocean floor to allow swimming at ocean floor
    const SAFETY_MARGIN: f32 = 2.0;
    if player_position.y < OCEAN_FLOOR_DEPTH - SAFETY_MARGIN {
        warn!(
            "Player fell through ocean floor at y={:.2} (floor={:.2}), teleporting to safe position",
            player_position.y, OCEAN_FLOOR_DEPTH
        );
        let safe_position = world_bounds
            .as_ref()
            .map(|wb| wb.safe_respawn_position())
            .unwrap_or(Vec3::new(
                player_position.x,
                LAND_ELEVATION + 0.5,
                player_position.z,
            ));
        player_transform.translation = safe_position;
        commands
            .entity(player_entity)
            .insert(PlayerPhysicsBundle::default()); // Restore clean physics after teleport
    }

    // Only handle extreme world boundaries
    if player_position.x.abs() > MAX_WORLD_COORDINATE
        || player_position.z.abs() > MAX_WORLD_COORDINATE
    {
        warn!(
            "Player at extreme position {:?}, teleporting to spawn",
            player_position
        );
        let safe_position = world_bounds
            .as_ref()
            .map(|wb| wb.safe_respawn_position())
            .unwrap_or(Vec3::new(0.0, LAND_ELEVATION + 0.5, 0.0));
        player_transform.translation = safe_position;
        commands
            .entity(player_entity)
            .insert(PlayerPhysicsBundle::default()); // Restore clean physics after teleport
    }
}

/// Minimal ground check - avoid interfering with Rapier physics
/// NOTE: Beaches/slopes go from y=3.0 (land) to y=-10.0 (ocean floor)
/// Only trigger BELOW ocean floor, not at or above it
pub fn player_movement_validation_system(
    mut player_query: Query<&mut Transform, (With<Player>, With<ActiveEntity>)>,
) {
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    // Only trigger 2m below ocean floor to allow full range of swimming depth
    const SAFETY_MARGIN: f32 = 2.0;
    if player_transform.translation.y < OCEAN_FLOOR_DEPTH - SAFETY_MARGIN {
        warn!(
            "Player fell through ocean floor at y={:.2} (floor={:.2}), teleporting to safe position",
            player_transform.translation.y, OCEAN_FLOOR_DEPTH
        );
        player_transform.translation.y = LAND_ELEVATION + 0.5;
    }
}
