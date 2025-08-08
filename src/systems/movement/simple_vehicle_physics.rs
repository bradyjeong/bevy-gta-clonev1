use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{ControlState, ActiveEntity, RealisticVehicle};
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

/// Simple vehicle physics system following AGENT.md simplicity principles
/// 
/// Replaces complex realistic_vehicle_physics.rs with straightforward force application:
/// - Throttle → Forward force
/// - Brake → Backward force  
/// - Steering → Rotation
/// - Boost → Speed multiplier
/// 
/// Benefits:
/// - Single responsibility: Apply forces based on ControlState
/// - Easy to understand: Linear force/rotation mapping
/// - Maintainable: No complex tire physics or aerodynamics
/// - Performance: Minimal calculations per vehicle
pub fn simple_vehicle_physics_system(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut query: Query<
        (&mut Velocity, &Transform, &ControlState, &mut RealisticVehicle),
        With<ActiveEntity>
    >,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<RealisticVehicle>)>,
) {
    let start_time = std::time::Instant::now();
    let dt = time.delta_secs().clamp(0.001, 0.05);
    let max_processing_time = 2.0; // 2ms time budget
    
    // Get active entity position for distance-based optimization
    let active_pos = active_query.single().map(|t| t.translation).unwrap_or_default();
    
    for (mut velocity, transform, control_state, mut vehicle) in query.iter_mut() {
        // Check time budget
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        // Skip physics for distant vehicles
        let distance = active_pos.distance(transform.translation);
        if distance > 150.0 {
            vehicle.physics_enabled = false;
            continue;
        }
        
        // Vehicle specifications (simplified)
        let base_acceleration = 25.0;
        let base_rotation_speed = 2.0;
        let boost_multiplier = if control_state.is_boosting() { 2.0 } else { 1.0 };
        
        // STEP 1: Calculate target velocities from control inputs
        let mut target_linear_velocity = Vec3::ZERO;
        let mut target_angular_velocity = Vec3::ZERO;
        
        // Forward/backward movement
        if control_state.is_accelerating() {
            let forward = transform.forward();
            target_linear_velocity += forward * base_acceleration * control_state.throttle * boost_multiplier;
        }
        
        if control_state.is_braking() {
            let forward = transform.forward();
            target_linear_velocity -= forward * base_acceleration * control_state.brake;
        }
        
        // Steering (only when moving for realistic feel)
        let is_moving = velocity.linvel.length() > 1.0;
        if is_moving && control_state.steering.abs() > 0.1 {
            target_angular_velocity.y = control_state.steering * base_rotation_speed;
        }
        
        // Emergency brake override
        if control_state.emergency_brake {
            target_linear_velocity *= 0.1;
            target_angular_velocity *= 0.5;
        }
        
        // STEP 2: Apply velocities with smooth interpolation
        let velocity_lerp_factor = dt * 5.0; // Smooth acceleration/deceleration
        let angular_lerp_factor = dt * 8.0;  // Responsive steering
        
        velocity.linvel = velocity.linvel.lerp(target_linear_velocity, velocity_lerp_factor);
        velocity.angvel = velocity.angvel.lerp(target_angular_velocity, angular_lerp_factor);
        
        // STEP 3: Apply simple physics effects
        
        // Gravity
        velocity.linvel.y -= 9.81 * dt;
        
        // Air resistance (simple drag)
        let drag_factor = 0.98;
        velocity.linvel *= drag_factor;
        velocity.angvel *= 0.95; // Angular damping
        
        // STEP 4: Apply safety systems using unified physics utilities
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.1, 2.0);
        
        // Update vehicle state
        vehicle.physics_enabled = true;
        vehicle.last_update_time = time.elapsed_secs();
    }
    
    // Performance monitoring
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 1.5 {
        warn!("Simple vehicle physics took {:.2}ms (> 1.5ms budget)", processing_time);
    }
}

/// Simple wheel visual update system
/// Updates wheel rotation based on vehicle speed for visual feedback
pub fn simple_wheel_update_system(
    time: Res<Time>,
    mut wheel_query: Query<&mut Transform, With<crate::components::VehicleWheel>>,
    vehicle_query: Query<&Velocity, (With<RealisticVehicle>, With<ActiveEntity>)>,
) {
    let dt = time.delta_secs();
    
    // Get average vehicle speed (simplified approach)
    let total_speed: f32 = vehicle_query.iter().map(|v| v.linvel.length()).sum();
    let vehicle_count = vehicle_query.iter().count() as f32;
    let average_speed = if vehicle_count > 0.0 { total_speed / vehicle_count } else { 0.0 };
    
    // Update all wheels based on average speed (simple but effective)
    let wheel_radius = 0.35;
    let rotation_speed = average_speed / wheel_radius;
    
    for mut wheel_transform in wheel_query.iter_mut() {
        wheel_transform.rotate_local_x(rotation_speed * dt);
    }
}

#[cfg(test)]
mod tests {
    use crate::components::ControlState;
    
    #[test]
    fn test_simple_physics_responds_to_control_state() {
        let mut control_state = ControlState::default();
        control_state.throttle = 0.5;
        control_state.steering = 1.0;
        
        assert!(control_state.is_accelerating());
        assert_eq!(control_state.steering, 1.0);
    }
    
    #[test]
    fn test_boost_multiplier() {
        let mut control_state = ControlState::default();
        control_state.boost = 1.0;
        
        assert!(control_state.is_boosting());
    }
}
