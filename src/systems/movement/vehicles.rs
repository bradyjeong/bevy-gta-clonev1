use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Car, ActiveEntity};
use crate::systems::input::{ControlManager, ControlAction};
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

pub fn car_movement(
    control_manager: Res<ControlManager>,
    config: Res<GameConfig>,
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>)>,
    _time: Res<Time>,
) {
    let start_time = std::time::Instant::now();
    let Ok((mut velocity, transform)) = car_query.single_mut() else {
        return;
    };

    let speed = 25.0;
    let rotation_speed = 2.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Use UNIFIED ControlManager controls
    if control_manager.is_control_active(ControlAction::Accelerate) {
        let accel_value = control_manager.get_control_value(ControlAction::Accelerate);
        let forward = transform.forward();
        target_linear_velocity += forward * speed * accel_value;
    }
    
    if control_manager.is_control_active(ControlAction::Brake) {
        let brake_value = control_manager.get_control_value(ControlAction::Brake);
        let forward = transform.forward();
        target_linear_velocity -= forward * speed * brake_value;
    }
    
    // Steering (only when moving)
    if control_manager.is_control_active(ControlAction::Accelerate) || control_manager.is_control_active(ControlAction::Brake) {
        let steering = control_manager.get_control_value(ControlAction::Steer);
        if steering.abs() > 0.1 {
            target_angular_velocity.y = steering * rotation_speed;
        }
    }
    
    // Emergency brake override
    if control_manager.is_emergency_active() {
        target_linear_velocity *= 0.1;
        target_angular_velocity *= 0.5;
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
    
    // Apply unified physics safety systems
    PhysicsUtilities::validate_velocity(&mut velocity, &config);
    PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 0.1, 1.0);
    
    // Performance monitoring
    let processing_time = start_time.elapsed().as_millis() as f32;
    if processing_time > 1.0 {
        warn!("Car movement took {:.2}ms (> 1ms budget)", processing_time);
    }
}
