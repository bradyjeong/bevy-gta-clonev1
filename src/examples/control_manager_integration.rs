use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{Car, ActiveEntity};
use crate::game_state::GameState;
use crate::systems::input::{
    InputManager, ControlManager, ControlAction,
    is_accelerating, is_braking, get_steering_input, is_turbo_active,
    control_action_system, control_validation_system
};

/// Example system showing how to use the Control Manager for car movement
/// This replaces the direct keyboard input with centralized control management
pub fn controlled_car_movement(
    control_manager: Res<ControlManager>,
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = car_query.single_mut() else {
        return;
    };

    let speed = 25.0;
    let rotation_speed = 2.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Use Control Manager instead of direct input
    if is_accelerating(&control_manager) {
        let accel_value = control_manager.get_control_value(ControlAction::Accelerate);
        let forward = transform.forward();
        target_linear_velocity += forward * speed * accel_value;
    }
    
    if is_braking(&control_manager) {
        let brake_value = control_manager.get_control_value(ControlAction::Brake);
        let forward = transform.forward();
        target_linear_velocity -= forward * speed * brake_value;
    }
    
    // Apply turbo boost if active
    if is_turbo_active(&control_manager) {
        target_linear_velocity *= 1.5; // 50% speed boost
    }
    
    // Steering (only when moving)
    if is_accelerating(&control_manager) || is_braking(&control_manager) {
        let steering = get_steering_input(&control_manager);
        if steering.abs() > 0.1 {
            target_angular_velocity.y = steering * rotation_speed;
        }
    }
    
    // Apply safety limits if emergency brake is active
    if control_manager.is_emergency_active() {
        target_linear_velocity *= 0.1; // Emergency slow down
        target_angular_velocity *= 0.5; // Reduce steering
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}



/// Example of how to set up the Control Manager in your app
pub fn setup_control_manager_example(app: &mut App) {
    app
        // Add the Control Manager resource
        .init_resource::<ControlManager>()
        
        // Add the core control systems
        .add_systems(Update, (
            // Process input and update control state
            control_action_system,
            // Validate control inputs for safety
            control_validation_system,
            // Apply controls to vehicles
            controlled_car_movement.run_if(in_state(GameState::Driving)),

        ).chain());
}



/// Example of monitoring control system performance
pub fn monitor_control_performance(control_manager: Res<ControlManager>) {
    let (max_time_us, failures) = control_manager.get_performance_stats();
    
    if max_time_us > 1000 { // More than 1ms
        warn!("Control system performance: {}μs max update time", max_time_us);
    }
    
    if failures > 0 {
        warn!("Control validation failures: {}", failures);
    }
    
    if control_manager.is_emergency_active() {
        warn!("Emergency brake system active!");
    }
    
    if control_manager.is_stability_active() {
        info!("Stability control intervening");
    }
}

