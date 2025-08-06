use bevy::prelude::*;
use crate::systems::movement::{car_movement, helicopter_movement, f16_movement, rotate_helicopter_rotors};
use crate::systems::effects::{exhaust_effects_system, update_jet_flames, update_flame_colors};
use crate::systems::vehicles::vehicle_lod_system;
// use crate::systems::configuration_validation_system; // DISABLED - conflicts with Rapier
use crate::game_state::GameState;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app
        // CRITICAL SAFEGUARDS: Run configuration validation at startup
        // .add_systems(Startup, configuration_validation_system) // DISABLED - conflicts with Rapier
        .add_systems(Update, (
            // CRITICAL PHYSICS SAFEGUARDS: Temporarily disabled due to static mut issues
            // enhanced_physics_safeguards_system,
            // crate::systems::realistic_physics_safeguards::emergency_physics_recovery_system,
            
            // LOD system runs after safeguards
            vehicle_lod_system,
            
            // Movement systems (force-based for vehicles)
            car_movement.run_if(in_state(GameState::Driving)),
            // Removed: supercar_movement system (used deleted SuperCar struct)
            helicopter_movement.run_if(in_state(GameState::Flying)),
            f16_movement.run_if(in_state(GameState::Jetting)),
            rotate_helicopter_rotors,
            exhaust_effects_system,
        ))
        .add_systems(Update, (
            // Visual effects  
            update_jet_flames,
            update_flame_colors,
            
            // PERFORMANCE MONITORING: Temporarily disabled due to static mut issues
            // physics_performance_monitoring_system,
            // adaptive_performance_system,
        ));
    }
}
