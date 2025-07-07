use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{Car, SuperCar, ActiveEntity};
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
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCar>)>,
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

/// Example system showing advanced SuperCar integration with Control Manager
pub fn controlled_supercar_movement(
    time: Res<Time>,
    control_manager: Res<ControlManager>,
    mut supercar_query: Query<(&mut Velocity, &Transform, &mut SuperCar), (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
) {
    let Ok((mut velocity, transform, mut supercar)) = supercar_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    
    // Use Control Manager values with SuperCar's advanced physics
    let acceleration_input = control_manager.get_control_value(ControlAction::Accelerate);
    let brake_input = control_manager.get_control_value(ControlAction::Brake);
    let steering_input = get_steering_input(&control_manager);
    let turbo_active = is_turbo_active(&control_manager);
    
    // Enhanced control processing with validation
    let safe_acceleration = acceleration_input.clamp(0.0, 1.0);
    let safe_brake = brake_input.clamp(0.0, 1.0);
    let safe_steering = steering_input.clamp(-1.0, 1.0);
    
    // Apply SuperCar-specific physics modifications
    let current_speed_ms = velocity.linvel.length();
    let current_speed_mph = current_speed_ms * 2.237;
    
    // Use Control Manager physics config for consistency
    let vehicle_type = crate::systems::input::vehicle_control_config::VehicleType::SuperCar;
    if let Some(physics_config) = control_manager.get_physics_config(vehicle_type) {
        let effective_acceleration = safe_acceleration * physics_config.acceleration_sensitivity;
        let effective_steering = safe_steering * physics_config.steering_sensitivity;
        
        // Apply physics with safety limits
        if !control_manager.is_emergency_active() {
            let max_speed_ms = physics_config.max_speed / 2.237;
            
            if current_speed_ms < max_speed_ms && effective_acceleration > 0.0 {
                let forward = transform.forward();
                let acceleration_force = forward * physics_config.acceleration * effective_acceleration;
                velocity.linvel += acceleration_force * dt;
            }
            
            if safe_brake > 0.0 {
                let braking_force = -velocity.linvel.normalize_or_zero() * physics_config.brake_force * safe_brake;
                velocity.linvel += braking_force * dt;
            }
            
            // Turbo system
            if turbo_active {
                velocity.linvel *= 1.3; // Turbo boost
            }
            
            // Steering with stability assist
            if effective_steering.abs() > 0.1 {
                let angular_force = effective_steering * physics_config.turn_speed;
                let stability_factor = if physics_config.stability_assist && current_speed_mph > 30.0 {
                    0.8 // Reduce steering at high speed
                } else {
                    1.0
                };
                velocity.angvel.y = angular_force * stability_factor;
            }
        }
    }
    
    // Emergency brake override
    if control_manager.is_emergency_active() {
        velocity.linvel *= 0.9; // Gradual emergency stop
        velocity.angvel *= 0.8; // Reduce rotation
    }
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
            controlled_supercar_movement.run_if(in_state(GameState::Driving)),
        ).chain());
}

/// Example of customizing physics configuration
pub fn customize_supercar_physics_example(mut control_manager: ResMut<ControlManager>) {
    use crate::systems::input::{vehicle_control_config::VehicleType, VehiclePhysicsConfig};
    
    // Create custom physics config for SuperCar
    let custom_physics = VehiclePhysicsConfig {
        max_speed: 100.0,              // Higher top speed
        acceleration: 60.0,            // Faster acceleration
        turn_speed: 5.0,              // Better handling
        brake_force: 80.0,            // Better brakes
        acceleration_sensitivity: 0.7, // More precise control
        steering_sensitivity: 0.8,     // Responsive steering
        enable_safety_limits: true,    // Keep safety on
        max_safe_speed: 150.0,        // Higher safety limit
        stability_assist: true,        // Advanced stability
        ..Default::default()
    };
    
    // Update the configuration
    control_manager.update_physics_config(VehicleType::SuperCar, custom_physics);
    
    info!("SuperCar physics configuration updated for enhanced performance");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::input::{InputConfig, InputAction};
    
    /// Test Control Manager integration with mock input
    #[test]
    fn test_control_manager_integration() {
        // This would be a more comprehensive test in a real scenario
        let mut control_manager = ControlManager::default();
        
        // Test basic functionality
        assert!(!control_manager.is_emergency_active());
        assert!(!control_manager.is_stability_active());
        
        // Test performance stats
        let (max_time, failures) = control_manager.get_performance_stats();
        assert_eq!(max_time, 0);
        assert_eq!(failures, 0);
        
        // Test physics config retrieval
        use crate::systems::input::vehicle_control_config::VehicleType;
        let car_config = control_manager.get_physics_config(VehicleType::Car);
        assert!(car_config.is_some());
        
        println!("Control Manager integration test passed");
    }
}
