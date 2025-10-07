use bevy::prelude::*;

use crate::states::AppState;
use crate::systems::world::dynamic_physics_culling::{
    disable_distant_dynamic_physics, enable_nearby_dynamic_physics,
};
use crate::systems::world::physics_activation::{
    activate_nearby_building_physics, deactivate_distant_building_physics,
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
                .run_if(in_state(AppState::InGame)), // Only run during gameplay, not Loading
        );

        info!("Physics Activation Plugin initialized - GTA-style dynamic physics for buildings and vehicles");
    }
}
