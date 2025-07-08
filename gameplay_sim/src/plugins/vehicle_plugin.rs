use bevy::prelude::*;
use crate::systems::movement::{
    car_movement, helicopter_movement, f16_movement, rotate_helicopter_rotors,
    supercar_input_system, supercar_physics_system, supercar_effects_system,
    realistic_vehicle_input_system, realistic_vehicle_physics_core_system,
    initialize_exhaust_pool_system, exhaust_flame_cleanup_system,
    configure_vehicle_system_sets, VehicleSet,
};
use crate::systems::effects::{exhaust_effects_system, update_jet_flames, update_flame_colors};
use crate::vehicles::vehicle_lod_system;
// use crate::systems::configuration_validation_system; // DISABLED - conflicts with Rapier
use game_core::game_state::GameState;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        // Configure vehicle system sets for proper execution order
        configure_vehicle_system_sets(app);
        
        // Initialize the exhaust flame pool resource
        app.init_resource::<crate::systems::movement::ExhaustFlamePool>();
        
        app
        // CRITICAL SAFEGUARDS: Run configuration validation at startup
        // .add_systems(Startup, configuration_validation_system) // DISABLED - conflicts with Rapier
        .add_systems(Startup, initialize_exhaust_pool_system)
        .add_systems(Update, (
            // CRITICAL PHYSICS SAFEGUARDS: Systems to be implemented
            // enhanced_physics_safeguards_system,
            // crate::systems::realistic_physics_safeguards::emergency_physics_recovery_system,
            
            // LOD system runs after safeguards
            vehicle_lod_system,
            
            // Movement systems (force-based for vehicles)
            car_movement.run_if(in_state(GameState::Driving)),
            
            // NEW: Split supercar systems with proper execution order
            supercar_input_system
                .run_if(in_state(GameState::Driving))
                .in_set(VehicleSet::Input),
            supercar_physics_system
                .run_if(in_state(GameState::Driving))
                .in_set(VehicleSet::Physics),
            supercar_effects_system
                .run_if(in_state(GameState::Driving))
                .in_set(VehicleSet::Effects),
            exhaust_flame_cleanup_system
                .in_set(VehicleSet::Effects),
            
            // NEW: Split realistic vehicle systems with proper execution order
            realistic_vehicle_input_system
                .run_if(in_state(GameState::Driving))
                .in_set(VehicleSet::Input),
            realistic_vehicle_physics_core_system
                .run_if(in_state(GameState::Driving))
                .in_set(VehicleSet::Physics),
            
            helicopter_movement.run_if(in_state(GameState::Flying)),
            f16_movement.run_if(in_state(GameState::Jetting)),
            rotate_helicopter_rotors,
            exhaust_effects_system,
        ))
        .add_systems(Update, (
            // Visual effects  
            update_jet_flames,
            update_flame_colors,
            
            // PERFORMANCE MONITORING: Systems to be implemented
            // physics_performance_monitoring_system,
            // adaptive_performance_system,
        ));
    }
}
