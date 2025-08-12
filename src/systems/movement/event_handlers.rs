use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::events::cross_plugin_events::*;
use crate::components::movement::*;
use crate::components::player::HumanPlayer;

/// Handle movement input requests from the player plugin
pub fn handle_movement_input_system(
    mut events: EventReader<RequestMovementInput>,
    mut state_events: EventWriter<MovementStateUpdate>,
    mut query: Query<(&mut Velocity, &mut ExternalForce, &Transform, &mut Stamina), With<HumanPlayer>>,
    time: Res<Time>,
) {
    for event in events.read() {
        if let Ok((mut velocity, mut force, transform, mut stamina)) = query.get_mut(event.entity) {
            // Calculate movement forces
            let speed = if event.run && stamina.current > 0.0 { 
                12.0 
            } else { 
                6.0 
            };
            
            let forward = transform.forward().as_vec3() * event.forward * speed;
            let right = transform.right().as_vec3() * event.right * speed;
            let movement = forward + right;
            
            // Apply forces
            force.force = movement * 50.0; // Mass-based multiplier
            
            // Handle jumping
            if event.jump && velocity.linvel.y.abs() < 0.1 {
                velocity.linvel.y = 8.0;
            }
            
            // Update stamina
            if event.run && movement.length() > 0.1 {
                stamina.current = (stamina.current - 20.0 * time.delta_secs()).max(0.0);
            } else {
                stamina.current = (stamina.current + 10.0 * time.delta_secs()).min(stamina.max);
            }
            
            // Send state update
            state_events.send(MovementStateUpdate {
                entity: event.entity,
                velocity: velocity.linvel,
                is_moving: movement.length() > 0.1,
                is_running: event.run && stamina.current > 0.0,
                stamina: stamina.current,
            });
        }
    }
}

/// Handle vehicle movement requests
pub fn handle_vehicle_movement_system(
    mut events: EventReader<RequestVehicleMovement>,
    mut query: Query<(&mut Velocity, &mut ExternalForce, &Transform)>,
    time: Res<Time>,
) {
    for event in events.read() {
        if let Ok((mut velocity, mut force, transform)) = query.get_mut(event.entity) {
            match event.vehicle_type {
                VehicleType::Car => {
                    // Simple car physics
                    let forward_force = transform.forward().as_vec3() * event.throttle * 1000.0;
                    let brake_force = -velocity.linvel.normalize() * event.brake * 500.0;
                    force.force = forward_force + brake_force;
                    
                    // Steering as angular velocity
                    velocity.angvel.y = -event.steering * 2.0;
                },
                VehicleType::Helicopter => {
                    // Simplified helicopter physics
                    let up_force = Vec3::Y * event.throttle * 500.0;
                    let forward_tilt = transform.forward().as_vec3() * event.steering * 200.0;
                    force.force = up_force + forward_tilt;
                    
                    // Yaw rotation
                    velocity.angvel.y = event.steering * 1.5;
                },
                VehicleType::F16 => {
                    // Simplified jet physics
                    let thrust = if event.special { 2000.0 } else { 1000.0 };
                    let forward_force = transform.forward().as_vec3() * event.throttle * thrust;
                    force.force = forward_force;
                    
                    // Pitch and roll based on steering
                    velocity.angvel.x = event.steering * 3.0;
                    velocity.angvel.z = event.brake * 2.0; // Roll with brake input
                },
                VehicleType::Yacht => {
                    // Simple boat physics
                    let forward_force = transform.forward().as_vec3() * event.throttle * 300.0;
                    let drag = -velocity.linvel * 0.5; // Water drag
                    force.force = forward_force + drag;
                    force.force.y = 0.0; // Keep on water surface
                    
                    // Slow turning
                    velocity.angvel.y = -event.steering * 0.5;
                },
            }
        }
    }
}

// Component for stamina (if not defined elsewhere)
#[derive(Component)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}
