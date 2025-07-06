use bevy::prelude::*;
use crate::components::*;
use crate::systems::input::{ControlManager, is_accelerating, is_braking};

/// Focused system for realistic vehicle input processing
pub fn realistic_vehicle_input_system(
    control_manager: Res<ControlManager>,
    mut query: Query<(&mut EnginePhysics, &RealisticVehicle), With<ActiveEntity>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    
    for (mut engine, vehicle) in query.iter_mut() {
        // Process realistic vehicle input with proper control systems
        process_vehicle_input(&control_manager, &mut engine, &vehicle, dt);
    }
}

/// Process realistic vehicle input with proper control systems
fn process_vehicle_input(
    control_manager: &Res<ControlManager>,
    engine: &mut EnginePhysics,
    _vehicle: &RealisticVehicle,
    dt: f32,
) {
    // Use ControlManager for realistic vehicle input
    // Throttle input with realistic response
    if is_accelerating(control_manager) {
        engine.throttle_input = (engine.throttle_input + dt * 2.0).clamp(0.0, 1.0);
    } else {
        engine.throttle_input = (engine.throttle_input - dt * 3.0).clamp(0.0, 1.0);
    }
    
    // Brake input with ABS simulation
    if is_braking(control_manager) {
        engine.brake_input = (engine.brake_input + dt * 4.0).clamp(0.0, 1.0);
    } else {
        engine.brake_input = (engine.brake_input - dt * 5.0).clamp(0.0, 1.0);
    }
    
    // Automatic transmission simulation (simplified)
    if engine.current_gear == 0 && engine.throttle_input > 0.1 {
        engine.current_gear = 1; // Engage first gear
    }
    
    // Simple gear shifting based on RPM
    if engine.current_gear > 0 {
        if engine.current_rpm > engine.max_rpm * 0.85 && engine.current_gear < engine.gear_ratios.len() as i8 - 2 {
            engine.current_gear += 1; // Upshift
        } else if engine.current_rpm < engine.max_rpm * 0.3 && engine.current_gear > 1 {
            engine.current_gear -= 1; // Downshift
        }
    }
}
