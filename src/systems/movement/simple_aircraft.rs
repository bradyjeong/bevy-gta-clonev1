use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{F16, ActiveEntity, AircraftFlight, F16Specs, ControlState, PlayerControlled, MainRotor, TailRotor};
use crate::systems::physics_utils::PhysicsUtilities;
use crate::config::GameConfig;

/// Simplified F16 flight system following AGENT.md simplicity principles
/// 
/// Replaces complex aerodynamic calculations with straightforward flight physics:
/// - Pitch/roll → Direct angular velocity
/// - Throttle → Forward thrust
/// - Yaw → Rotation around Y-axis
/// - Afterburner → Thrust multiplier
/// 
/// Benefits:
/// - Easy to understand: Direct control mapping
/// - Maintainable: No complex aerodynamic formulas
/// - Performant: Minimal calculations per frame
/// - Flight Feel: Maintains responsive aircraft control
pub fn simple_f16_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut f16_query: Query<(
        &mut Velocity,
        &mut Transform,
        &mut AircraftFlight,
        &F16Specs,
        &ControlState,
    ), (With<F16>, With<ActiveEntity>, With<PlayerControlled>)>,
) {
    let dt = time.delta_secs().clamp(0.001, 0.05);
    
    for (mut velocity, transform, mut flight, specs, control_state) in f16_query.iter_mut() {
        
        // === SIMPLE INPUT PROCESSING ===
        
        // Direct control mapping from ControlState
        flight.pitch = control_state.pitch.clamp(-1.0, 1.0);
        flight.roll = control_state.roll.clamp(-1.0, 1.0);
        flight.yaw = control_state.yaw.clamp(-1.0, 1.0);
        
        // Simple throttle response
        if control_state.throttle > 0.0 {
            flight.throttle = (flight.throttle + dt * 2.0).clamp(0.0, 1.0);
        } else if control_state.brake > 0.0 {
            flight.throttle = (flight.throttle - dt * 3.0).clamp(0.0, 1.0);
        } else {
            // Gentle idle throttle decay to reduce sticky throttle feel
            flight.throttle = (flight.throttle - dt * 0.5).clamp(0.0, 1.0);
        }
        
        // Afterburner activation + VFX sync delay (now configurable)
        flight.afterburner = control_state.is_boosting();
        if flight.afterburner {
            flight.afterburner_timer += dt;
            if flight.afterburner_timer > specs.afterburner_delay { 
                flight.afterburner_active = true; 
            }
        } else {
            flight.afterburner_timer = 0.0;
            flight.afterburner_active = false;
        }
        
        // === SIMPLIFIED FLIGHT PHYSICS ===
        
        // Basic flight parameters
        let base_thrust = specs.max_thrust * 0.7; // Simplified thrust calculation
        let boost_multiplier = if flight.afterburner { 1.5 } else { 1.0 };
        let thrust_force = base_thrust * flight.throttle * boost_multiplier;
        
        // Forward thrust along aircraft direction
        let forward_force = transform.forward() * thrust_force;
        
        // === ANGULAR CONTROL (Direct and Responsive) ===
        
        // Use configurable control parameters from specs
        let control_sensitivity = specs.control_sensitivity;
        let yaw_scale = specs.yaw_scale;
        
        // Map angular control in LOCAL axes, then rotate into world space
        let local_target_ang = Vec3::new(
            flight.pitch * control_sensitivity,              // +X pitch
            flight.yaw * control_sensitivity * yaw_scale,    // +Y yaw
            -flight.roll * control_sensitivity,              // -Z roll
        );
        let world_target_ang = transform.rotation.mul_vec3(local_target_ang);
        
        // Apply angular velocity with damping
        velocity.angvel = velocity.angvel.lerp(world_target_ang, dt * 8.0);
        velocity.angvel *= 0.95;
        
        // === LINEAR FORCES ===
        
        // Apply forward thrust
        velocity.linvel += forward_force * dt / specs.mass.max(1000.0);
        
        // Simple drag (air resistance)
        let drag_factor = 0.98 - (velocity.linvel.length() * 0.00001); // Speed-dependent drag
        velocity.linvel *= drag_factor.clamp(0.90, 0.99);
        
        // Gravity
        velocity.linvel += Vec3::new(0.0, -9.81, 0.0) * dt;
        
        // Basic lift when moving forward (simplified)
        let forward_speed = transform.forward().dot(velocity.linvel);
        if forward_speed > 20.0 {
            let lift_force = Vec3::Y * forward_speed * 0.1; // Simple lift
            velocity.linvel += lift_force * dt;
        }
        
        // === FLIGHT STATE TRACKING ===
        
        flight.airspeed = velocity.linvel.length();
        flight.current_thrust = thrust_force;
        
        // Simple stall detection
        if flight.airspeed < 15.0 {
            // Add some instability at low speeds
            let stall_turbulence = Vec3::new(
                (time.elapsed_secs() * 2.0).sin() * 2.0,
                (time.elapsed_secs() * 1.5).cos() * 1.0,
                (time.elapsed_secs() * 3.0).sin() * 1.5,
            );
            velocity.linvel += stall_turbulence * dt;
            velocity.angvel += stall_turbulence * 0.5 * dt;
        }
        
        // === SAFETY SYSTEMS ===
        
        // Use unified physics utilities for safety
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 2.0, 10.0);
        
        // Aircraft-specific safety bounds
        let max_aircraft_speed = 300.0; // Reasonable max speed
        if velocity.linvel.length() > max_aircraft_speed {
            velocity.linvel = velocity.linvel.normalize() * max_aircraft_speed;
        }
        
        let max_angular_velocity = 5.0; // Prevent excessive spinning
        if velocity.angvel.length() > max_angular_velocity {
            velocity.angvel = velocity.angvel.normalize() * max_angular_velocity;
        }
        
        // Keep aircraft above minimum altitude (simple terrain avoidance)
        if transform.translation.y < 5.0 {
            // Emergency pull-up
            velocity.linvel.y += 20.0 * dt;
            velocity.angvel.x = -1.0; // Pitch up
        }
    }
}

/// Simple helicopter controls that work alongside F16
/// Uses similar simplified approach for consistency
pub fn simple_helicopter_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut helicopter_query: Query<(&mut Velocity, &Transform, &ControlState), 
        (With<crate::components::Helicopter>, With<ActiveEntity>, With<PlayerControlled>)>,
) {
    let dt = time.delta_secs().clamp(0.001, 0.05);
    
    for (mut velocity, transform, control_state) in helicopter_query.iter_mut() {
        
        // Basic helicopter parameters
        let speed = 30.0;
        let rotation_speed = 4.0;
        let vertical_speed = 15.0;
        
        let mut target_linear_velocity = Vec3::ZERO;
        let mut target_angular_velocity = Vec3::ZERO;
        
        // Forward/backward movement using pitch
        if control_state.pitch > 0.1 {
            target_linear_velocity += transform.forward() * speed * control_state.pitch;
        } else if control_state.pitch < -0.1 {
            target_linear_velocity -= transform.forward() * speed * control_state.pitch.abs();
        }
        
        // Rotation using yaw (invert sign for correct direction)
        if control_state.yaw.abs() > 0.1 {
            target_angular_velocity.y = -control_state.yaw * rotation_speed;
        }
        
        // Vertical movement (collective)
        if control_state.vertical > 0.1 {
            target_linear_velocity.y += vertical_speed * control_state.vertical;
        } else if control_state.vertical < -0.1 {
            target_linear_velocity.y -= vertical_speed * control_state.vertical.abs();
        }
        
        // Apply forces with smooth interpolation
        velocity.linvel = velocity.linvel.lerp(target_linear_velocity, dt * 4.0);
        velocity.angvel = velocity.angvel.lerp(target_angular_velocity, dt * 6.0);
        
        // Simple physics
        velocity.linvel *= 0.95; // Air resistance
        velocity.angvel *= 0.90; // Angular damping
        
        // Gravity
        velocity.linvel += Vec3::new(0.0, -9.81, 0.0) * dt;
        
        // Safety systems
        PhysicsUtilities::validate_velocity(&mut velocity, &config);
        PhysicsUtilities::apply_ground_collision(&mut velocity, &transform, 1.0, 5.0);
    }
}

/// Rotate helicopter main and tail rotors every frame
pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut main_rotor_query: Query<&mut Transform, (With<MainRotor>, Without<TailRotor>)>,
    mut tail_rotor_query: Query<&mut Transform, (With<TailRotor>, Without<MainRotor>)>,
) {
    // Use delta-based rotation to preserve initial blade offsets
    let main_delta = time.delta_secs() * 20.0; // rad/s
    let tail_delta = time.delta_secs() * 35.0; // rad/s

    for mut transform in main_rotor_query.iter_mut() {
        transform.rotate_y(main_delta);
    }

    for mut transform in tail_rotor_query.iter_mut() {
        transform.rotate_z(tail_delta);
    }
}

