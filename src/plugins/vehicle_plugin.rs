use bevy::prelude::*;
use crate::systems::movement::{car_movement, simple_f16_movement, simple_helicopter_movement, rotate_helicopter_rotors};
use crate::systems::setup::on_f16_spawned;
// Complex aircraft systems moved to examples/complex_aircraft_physics.rs
use crate::systems::effects::{exhaust_effects_system, update_jet_flames_unified};
use crate::systems::safety::{validate_physics_config};
use crate::components::safety::WorldBounds;
// LOD system replaced with Bevy's VisibilityRange + simulation_lod
// use crate::systems::configuration_validation_system; // DISABLED - conflicts with Rapier
use crate::game_state::GameState;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app
        // Initialize safety resources
        .init_resource::<WorldBounds>()
        // CRITICAL SAFEGUARDS: Run configuration validation at startup
        .add_systems(Startup, validate_physics_config)
        // Observer for F16 setup when specs are added
        .add_observer(on_f16_spawned)
        .add_systems(Update, (
            // REMOVED: bounds_safety_system and diagnostics - finite world eliminates need
            
            // LOD now handled by Bevy's VisibilityRange automatically
            
            // Movement systems (simplified for AGENT.md compliance)
            car_movement.run_if(in_state(GameState::Driving)),
            // Simplified vehicle movement systems only
            
            // Aircraft systems: simplified versions only (complex moved to examples/)
            simple_helicopter_movement.run_if(in_state(GameState::Flying)),
            simple_f16_movement.run_if(in_state(GameState::Jetting)),
            
            // Visual rotor animation for helicopters
               rotate_helicopter_rotors,
            
            exhaust_effects_system,
        ))
        .add_systems(Update, (
            // Visual effects - unified flame system
            update_jet_flames_unified,
            
            // PERFORMANCE MONITORING: Temporarily disabled due to static mut issues
            // physics_performance_monitoring_system,
            // adaptive_performance_system,
        ));
    }
}
