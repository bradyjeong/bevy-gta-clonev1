use bevy::prelude::*;
use crate::systems::movement::{car_movement, supercar_movement, helicopter_movement, f16_movement, rotate_helicopter_rotors};
use crate::systems::effects::exhaust_effects_system;
use crate::systems::vehicles::vehicle_lod_system;
use crate::game_state::GameState;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            // LOD system runs first to update rendering
            vehicle_lod_system,
            // Movement systems
            car_movement.run_if(in_state(GameState::Driving)),
            supercar_movement.run_if(in_state(GameState::Driving)),
            helicopter_movement.run_if(in_state(GameState::Flying)),
            f16_movement.run_if(in_state(GameState::Jetting)),
            rotate_helicopter_rotors,
            exhaust_effects_system,
        ));
    }
}
