#![allow(clippy::type_complexity)]
use crate::bundles::PlayerPhysicsBundle;
use crate::components::{ActiveEntity, Player};
use bevy::prelude::*;

/// Basic safety system - only handles extreme cases, avoids velocity manipulation
pub fn player_collision_resolution_system(
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, With<ActiveEntity>)>,
    mut commands: Commands,
) {
    let Ok((player_entity, mut player_transform)) = player_query.single_mut() else {
        return;
    };

    let player_position = player_transform.translation;

    // Only handle extreme position cases - let Rapier handle normal physics
    if player_position.y < -10.0 {
        warn!(
            "Player far below ground at y={:.2}, teleporting to safe position",
            player_position.y
        );
        player_transform.translation = Vec3::new(player_position.x, 0.5, player_position.z);
        commands
            .entity(player_entity)
            .insert(PlayerPhysicsBundle::default()); // Restore clean physics after teleport
    }

    // Only handle extreme world boundaries
    let max_coord = 2000.0;
    if player_position.x.abs() > max_coord || player_position.z.abs() > max_coord {
        warn!(
            "Player at extreme position {:?}, teleporting to spawn",
            player_position
        );
        player_transform.translation = Vec3::new(0.0, 0.5, 0.0);
        commands
            .entity(player_entity)
            .insert(PlayerPhysicsBundle::default()); // Restore clean physics after teleport
    }
}

/// Minimal ground check - avoid interfering with Rapier physics
pub fn player_movement_validation_system(
    mut player_query: Query<&mut Transform, (With<Player>, With<ActiveEntity>)>,
) {
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    // Only prevent falling through ground - adjusted for proper ground level
    if player_transform.translation.y < -1.0 {
        player_transform.translation.y = 0.5;
    }
}
