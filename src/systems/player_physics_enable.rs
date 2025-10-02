#![allow(clippy::type_complexity)]
use crate::components::{PendingPhysicsEnable, Player};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Re-enable player physics on the frame after vehicle exit
/// This runs before Rapier's SyncBackend to ensure the pose is correctly read
pub fn enable_player_physics_next_frame(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<Player>,
            With<PendingPhysicsEnable>,
            With<RigidBodyDisabled>,
        ),
    >,
) {
    for entity in &query {
        commands
            .entity(entity)
            .remove::<RigidBodyDisabled>()
            .remove::<PendingPhysicsEnable>();

        info!("Re-enabled player physics after safe vehicle exit");
    }
}
