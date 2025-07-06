//! Gameplay simulation - physics, AI, rules
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;
pub use game_core;

pub mod prelude;
pub mod systems;

pub use prelude::*;

/// Main plugin for simulation systems
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // Movement systems
        app.add_systems(Update, (
            systems::movement::player::human_player_movement,
            systems::movement::player::human_player_animation,
            systems::movement::supercar_input::supercar_input_system,
            systems::movement::supercar_physics::supercar_physics_system,
            systems::movement::supercar_effects::supercar_effects_system,
            systems::movement::realistic_vehicle_input::realistic_vehicle_input_system,
            systems::movement::realistic_vehicle_physics::realistic_vehicle_physics_system,
            systems::movement::realistic_vehicle_physics_core::realistic_vehicle_physics_core_system,
            systems::movement::aircraft::helicopter_movement,
            systems::movement::aircraft::f16_movement,
            systems::movement::aircraft::rotate_helicopter_rotors,
            systems::movement::vehicles::car_movement,
        ));
    }
}
