use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

use crate::states::AppState;
use crate::systems::world::physics_activation::{
    activate_nearby_building_physics, deactivate_distant_building_physics,
    disable_distant_dynamic_physics, enable_nearby_dynamic_physics,
};

/// Physics activation plugin - GTA-style dynamic physics
/// Only activates physics for buildings near the player
pub struct PhysicsActivationPlugin;

impl Plugin for PhysicsActivationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                activate_nearby_building_physics,
                deactivate_distant_building_physics,
                enable_nearby_dynamic_physics,
                disable_distant_dynamic_physics,
            )
                .chain()
                .run_if(in_state(AppState::InGame))
                .run_if(on_timer(Duration::from_millis(200))),
        );

        #[cfg(feature = "debug-ui")]
        info!(
            "Physics Activation Plugin initialized - GTA-style dynamic physics (throttled to 5Hz)"
        );
    }
}
