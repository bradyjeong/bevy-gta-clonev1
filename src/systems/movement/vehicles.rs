use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Car, ActiveEntity, SimpleCarSpecs};
use crate::components::ControlState;
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

pub fn car_movement(
    config: Res<GameConfig>,
    mut car_query: Query<(&mut Velocity, &Transform, &ControlState, &SimpleCarSpecs), (With<Car>, With<ActiveEntity>)>,
    time: Res<Time>,
) {
    let start_time = std::time::Instant::now();
    
    for (mut velocity, transform, control_state, specs) in car_query.iter_mut() {
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Use clean ControlState for car controls
    if control_state.is_accelerating() {
        let forward = transform.forward();
        target_linear_velocity += forward * specs.base_speed * control_state.throttle;
    }
    
    if control_state.is_braking() {
        let forward = transform.forward();
        target_linear_velocity -= forward * specs.base_speed * control_state.brake;
    }
    
    // Steering (only when moving)
    if control_state.is_accelerating() || control_state.is_braking() {
        if control_state.steering.abs() > 0.1 {
            target_angular_velocity.y = control_state.steering * specs.rotation_speed;
        }
    }
    
    // Emergency brake override
    if control_state.emergency_brake {
        target_linear_velocity *= specs.emergency_brake_linear;
        target_angular_velocity *= specs.emergency_brake_angular;
    }
    
    // Apply forces with smooth interpolation (consistent with aircraft)
    let dt = time.delta_secs().clamp(0.001, 0.05);
    velocity.linvel = velocity.linvel.lerp(target_linear_velocity, dt * 4.0);
    velocity.angvel = velocity.angvel.lerp(target_angular_velocity, dt * 6.0);
    
        // Apply velocity validation (kinematic bodies handle their own collision)
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
    }
    
    // Performance monitoring
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 1.0 {
        warn!("Car movement took {:.2}ms (> 1ms budget)", processing_time);
    }
}
