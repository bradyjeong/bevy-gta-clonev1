use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{F16, ActiveEntity, AircraftFlight, SimpleF16Specs, ControlState, PlayerControlled, MainRotor, TailRotor, Helicopter, SimpleHelicopterSpecs};

use crate::systems::movement::simple_flight_common::SimpleFlightCommon;
use crate::systems::physics::PhysicsUtilities;
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
        &Transform,
        &mut AircraftFlight,
        &SimpleF16Specs,
        &ControlState,
    ), (With<F16>, With<ActiveEntity>, With<PlayerControlled>, Changed<ControlState>)>,
) {
    let dt = PhysicsUtilities::stable_dt(&time);
    
    for (mut velocity, transform, mut flight, specs, control_state) in f16_query.iter_mut() {
        
        // === ULTRA-SIMPLIFIED INPUT PROCESSING ===
        
        // Direct throttle processing (no round-trip conversion)
        flight.throttle = SimpleFlightCommon::process_throttle(
            control_state,
            flight.throttle,
            specs.throttle_increase_rate,
            specs.throttle_decrease_rate,
            dt,
        );
        
        // Direct afterburner state
        flight.afterburner_active = control_state.is_boosting();
        
        // === MINIMAL FLIGHT PHYSICS ===
        
        // Calculate thrust (all values from specs, no magic numbers)
        let boost_multiplier = if flight.afterburner_active { specs.afterburner_multiplier } else { 1.0 };
        let thrust_force = specs.max_thrust * flight.throttle * boost_multiplier;
        let forward_force = transform.forward() * thrust_force;
        
        // === DIRECT ANGULAR CONTROL ===
        
        // Read controls directly (no state duplication)
        let local_target_ang = Vec3::new(
            control_state.pitch * specs.pitch_rate_max,   // +X pitch
            control_state.yaw * specs.yaw_rate_max,       // +Y yaw
            -control_state.roll * specs.roll_rate_max,    // -Z roll
        );
        let world_target_ang = transform.rotation.mul_vec3(local_target_ang);
        
        // Apply angular velocity (lerp factor from specs)
        velocity.angvel = velocity.angvel.lerp(world_target_ang, dt * specs.angular_lerp_factor);
        
        // === LINEAR FORCES ===
        
        // Apply forward thrust (no magic mass clamp)
        velocity.linvel += forward_force * dt / specs.mass;
        
        // Orientation-aware lift assistance (deadzone from specs)
        if flight.throttle > specs.throttle_deadzone {
            velocity.linvel += transform.up() * flight.throttle * specs.lift_per_throttle * dt;
        }
        
        // === MINIMAL STATE TRACKING ===
        
        flight.airspeed = velocity.linvel.length();
        
        // === SHARED PHYSICS SAFETY ===
        
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
        
        // Dynamic bodies handle ground collision through Rapier
    }
}

/// Simple helicopter controls that work alongside F16
/// Uses similar simplified approach for consistency
pub fn simple_helicopter_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut helicopter_query: Query<(&mut Velocity, &Transform, &ControlState, &SimpleHelicopterSpecs), 
        (With<Helicopter>, With<ActiveEntity>, With<PlayerControlled>, Changed<ControlState>)>,
) {
    let dt = PhysicsUtilities::stable_dt(&time);
    
    for (mut velocity, transform, control_state, specs) in helicopter_query.iter_mut() {
        
        let mut target_linear_velocity = Vec3::ZERO;
        let mut target_angular_velocity = Vec3::ZERO;
        
        // Forward/backward movement using pitch
        if control_state.pitch > 0.1 {
            target_linear_velocity += transform.forward() * specs.forward_speed * control_state.pitch;
        } else if control_state.pitch < -0.1 {
            target_linear_velocity -= transform.forward() * specs.forward_speed * control_state.pitch.abs();
        }
        
        // Rotation using yaw (invert sign for correct direction)
        if control_state.yaw.abs() > 0.1 {
            target_angular_velocity.y = -control_state.yaw * specs.yaw_rate;
        }
        
        // Vertical movement (collective)
        if control_state.vertical > 0.1 {
            target_linear_velocity.y += specs.vertical_speed * control_state.vertical;
        } else if control_state.vertical < -0.1 {
            target_linear_velocity.y -= specs.vertical_speed * control_state.vertical.abs();
        }
        
        // Roll controls (Q/E keys) - banking and lateral movement
        if control_state.roll.abs() > 0.1 {
            // Roll angular velocity (banking around Z-axis)
            target_angular_velocity.z = -control_state.roll * specs.roll_rate;
            
            // Lateral movement when rolling (helicopter banks into turn)  
            let lateral_force = transform.right() * -control_state.roll * specs.lateral_speed;
            target_linear_velocity += lateral_force;
        }
        
        // Apply forces with smooth interpolation (dynamic bodies handle gravity)
        velocity.linvel = velocity.linvel.lerp(target_linear_velocity, dt * specs.linear_lerp_factor);
        velocity.angvel = velocity.angvel.lerp(target_angular_velocity, dt * specs.angular_lerp_factor);
        
        // === SHARED PHYSICS SAFETY ===
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}

/// Rotate helicopter main and tail rotors every frame
pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut rotor_query: Query<(&mut Transform, Option<&MainRotor>, Option<&TailRotor>)>,
    helicopter_query: Query<&SimpleHelicopterSpecs, With<Helicopter>>,
) {
    // Get rotor speeds from helicopter specs (fallback to defaults if no helicopter present)
    let (main_rpm, tail_rpm) = if let Ok(specs) = helicopter_query.single() {
        (specs.main_rotor_rpm, specs.tail_rotor_rpm)
    } else {
        (20.0, 35.0) // fallback defaults
    };
    
    let dt = PhysicsUtilities::stable_dt(&time);
    let main_delta = dt * main_rpm;
    let tail_delta = dt * tail_rpm;

    for (mut transform, main_rotor, tail_rotor) in rotor_query.iter_mut() {
        if main_rotor.is_some() {
            transform.rotate_y(main_delta);
        } else if tail_rotor.is_some() {
            transform.rotate_z(tail_delta);
        }
    }
}

