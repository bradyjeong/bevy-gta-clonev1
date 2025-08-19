use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Car, ActiveEntity, SimpleCarSpecs};
use crate::components::ControlState;
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

pub fn car_movement(
    config: Res<GameConfig>,
    mut car_query: Query<(&mut Velocity, &Transform, &ControlState, &SimpleCarSpecs), (With<Car>, With<ActiveEntity>)>,
    _time: Res<Time>,
) {
    let start_time = std::time::Instant::now();
    let Ok((mut velocity, transform, control_state, specs)) = car_query.single_mut() else {
        return;
    };
    
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
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Apply unified physics safety systems
    PhysicsUtilities::validate_velocity(&mut velocity, &config);
    PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, specs.min_height, specs.ground_bounce);
    
    // Performance monitoring
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > specs.max_processing_time {
        warn!("Car movement took {:.2}ms (> {}ms budget)", processing_time, specs.max_processing_time);
    }
}
