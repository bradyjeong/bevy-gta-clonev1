#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::{
    ActiveEntity, AircraftFlight, ControlState, F16, Helicopter, MainRotor, PlayerControlled,
    SimpleF16Specs, SimpleHelicopterSpecs, TailRotor,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::config::GameConfig;
use crate::systems::movement::simple_flight_common::SimpleFlightCommon;
use crate::systems::physics::PhysicsUtilities;
use crate::util::safe_math::safe_lerp;

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
    mut f16_query: Query<
        (
            &mut Velocity,
            &Transform,
            &mut AircraftFlight,
            &SimpleF16Specs,
            &ControlState,
        ),
        (With<F16>, With<ActiveEntity>, With<PlayerControlled>),
    >,
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

        // Calculate boost multiplier (all values from specs, no magic numbers)
        let boost_multiplier = if flight.afterburner_active {
            specs.afterburner_multiplier
        } else {
            1.0
        };

        // === DIRECT ANGULAR CONTROL ===

        // Read controls directly (no state duplication)
        let local_target_ang = Vec3::new(
            control_state.pitch * specs.pitch_rate_max, // +X pitch (up arrow = pitch up)
            -control_state.yaw * specs.yaw_rate_max,    // -Y yaw (A = left, D = right)
            -control_state.roll * specs.roll_rate_max,  // -Z roll (left arrow = roll left)
        );
        let world_target_ang = transform.rotation.mul_vec3(local_target_ang);

        // Apply angular velocity (lerp factor from specs)
        velocity.angvel = safe_lerp(
            velocity.angvel,
            world_target_ang,
            dt * specs.angular_lerp_factor,
        );

        // === ARCADE-REALISTIC VELOCITY CONTROL ===

        if flight.throttle > specs.throttle_deadzone {
            // Engine on: Direct thrust control (arcade style)
            let target_forward_speed = specs.max_forward_speed * flight.throttle * boost_multiplier;
            let target_forward_velocity = transform.forward() * target_forward_speed;

            // Add lift assistance when throttling
            let target_linear_velocity = target_forward_velocity
                + transform.up() * flight.throttle * specs.lift_per_throttle;

            velocity.linvel = safe_lerp(
                velocity.linvel,
                target_linear_velocity,
                dt * specs.linear_lerp_factor,
            );
        } else {
            // Engine off: Apply frame-rate independent momentum decay (gliding like GTA V)
            let drag_per_second = specs.drag_factor;
            let frame_drag = drag_per_second.powf(dt);
            let vertical_drag = 0.999_f32.powf(dt); // Slight vertical drag for realistic sink rate
            velocity.linvel = Vec3::new(
                velocity.linvel.x * frame_drag,
                velocity.linvel.y * vertical_drag, // Gradual loss of vertical momentum
                velocity.linvel.z * frame_drag,
            );
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
    mut helicopter_query: Query<
        (
            &mut Velocity,
            &Transform,
            &ControlState,
            &SimpleHelicopterSpecs,
        ),
        (With<Helicopter>, With<ActiveEntity>, With<PlayerControlled>),
    >,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    for (mut velocity, transform, control_state, specs) in helicopter_query.iter_mut() {
        // Early exit check for input, but always run lerp and safety checks
        let has_input = control_state.pitch.abs() > 0.1
            || control_state.yaw.abs() > 0.1
            || control_state.vertical.abs() > 0.1
            || control_state.roll.abs() > 0.1;

        let mut target_linear_velocity = Vec3::ZERO;
        let mut target_angular_velocity = Vec3::ZERO;

        // Only calculate target velocities when there's meaningful input
        if has_input {
            // Forward/backward movement using pitch
            if control_state.pitch > 0.1 {
                target_linear_velocity +=
                    transform.forward() * specs.forward_speed * control_state.pitch;
            } else if control_state.pitch < -0.1 {
                target_linear_velocity -=
                    transform.forward() * specs.forward_speed * control_state.pitch.abs();
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
        }

        // Apply movement with momentum decay when no input (GTA-style)
        if has_input {
            let lerped_velocity = safe_lerp(
                velocity.linvel,
                target_linear_velocity,
                dt * specs.linear_lerp_factor,
            );

            // Preserve gravity in Y-axis unless actively controlling vertical movement
            velocity.linvel = if target_linear_velocity.y.abs() > 0.1 {
                lerped_velocity // Full control including Y when actively moving vertically
            } else {
                Vec3::new(lerped_velocity.x, velocity.linvel.y, lerped_velocity.z) // Preserve gravity
            };
        } else {
            // No input: Apply frame-rate independent momentum decay (helicopter keeps drifting like GTA V)
            let drag_per_second = specs.drag_factor;
            let frame_drag = drag_per_second.powf(dt);
            velocity.linvel = Vec3::new(
                velocity.linvel.x * frame_drag,
                velocity.linvel.y, // Preserve gravity
                velocity.linvel.z * frame_drag,
            );
        }
        velocity.angvel = safe_lerp(
            velocity.angvel,
            target_angular_velocity,
            dt * specs.angular_lerp_factor,
        );

        // Apply velocity validation every frame (critical for preventing physics panics)
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}

/// Rotate helicopter main and tail rotors every frame
pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut rotor_query: Query<(&mut Transform, Option<&MainRotor>, Option<&TailRotor>)>,
    helicopter_query: Query<&SimpleHelicopterSpecs, With<Helicopter>>,
) {
    // Get rotor speeds from helicopter specs (safe iteration, no panic)
    let (main_rpm, tail_rpm) = helicopter_query
        .iter()
        .next()
        .map(|specs| (specs.main_rotor_rpm, specs.tail_rotor_rpm))
        .unwrap_or((20.0, 35.0)); // fallback defaults

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
