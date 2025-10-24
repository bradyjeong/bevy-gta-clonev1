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

        // === GTA-STYLE ANGULAR CONTROL ===

        // Speed-based control effectiveness (once per frame)
        let airspeed = velocity.linvel.length();
        let control_eff = (airspeed / specs.control_full_speed).clamp(0.0, 1.0);
        let control_eff = specs.min_control_factor + (1.0 - specs.min_control_factor) * control_eff;

        // Discrete angular rates per axis (GTA-style input shaping)
        let pitch_input_abs = control_state.pitch.abs();
        let pitch_cmd = if pitch_input_abs < specs.input_deadzone {
            0.0
        } else {
            let rate = if pitch_input_abs < specs.input_step_threshold {
                specs.pitch_rate_min
            } else {
                specs.pitch_rate_max
            };
            control_state.pitch.signum() * rate * control_eff
        };

        let roll_input_abs = control_state.roll.abs();
        let roll_cmd = if roll_input_abs < specs.input_deadzone {
            0.0
        } else {
            let rate = if roll_input_abs < specs.input_step_threshold {
                specs.roll_rate_min
            } else {
                specs.roll_rate_max
            };
            -control_state.roll.signum() * rate * control_eff
        };

        let yaw_input_abs = control_state.yaw.abs();
        let yaw_cmd = if yaw_input_abs < specs.input_deadzone {
            0.0
        } else {
            let rate = if yaw_input_abs < specs.input_step_threshold {
                specs.yaw_rate_min
            } else {
                specs.yaw_rate_max
            };
            -control_state.yaw.signum() * rate * control_eff
        };

        // Auto-stabilization (horizon leveling) when input is in deadzone
        // Only apply when airspeed > 5.0 to prevent ground jitter
        let (pitch_auto, roll_auto, yaw_auto, roll_bank) = if airspeed > 5.0 {
            let pitch = if pitch_input_abs < specs.input_deadzone {
                transform.forward().dot(Vec3::Y) * -specs.pitch_auto_level_gain * control_eff
            } else {
                0.0
            };

            let roll = if roll_input_abs < specs.input_deadzone {
                -(*transform.right()).dot(Vec3::Y) * specs.roll_auto_level_gain * control_eff
            } else {
                0.0
            };

            let lateral_speed = velocity.linvel.dot(*transform.right());
            let yaw = if yaw_input_abs < specs.input_deadzone {
                -(lateral_speed / airspeed.max(1.0)) * specs.yaw_auto_level_gain * control_eff
            } else {
                0.0
            };

            let bank = if roll_input_abs < specs.input_deadzone {
                (-lateral_speed * specs.auto_bank_gain * control_eff).clamp(
                    -specs.auto_bank_max_rate,
                    specs.auto_bank_max_rate,
                )
            } else {
                0.0
            };

            (pitch, roll, yaw, bank)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        // Combine all control inputs
        let local_target_ang = Vec3::new(
            pitch_cmd + pitch_auto,
            yaw_cmd + yaw_auto,
            roll_cmd + roll_auto + roll_bank,
        );
        let world_target_ang = transform.rotation.mul_vec3(local_target_ang);

        // Apply angular velocity with lerp
        velocity.angvel = safe_lerp(
            velocity.angvel,
            world_target_ang,
            dt * specs.angular_lerp_factor,
        );

        // GTA-style multiplicative damping (one pass)
        let inv_rot = transform.rotation.inverse();
        let mut local_ang = inv_rot.mul_vec3(velocity.angvel);
        let pf = if pitch_input_abs < specs.input_deadzone {
            specs.pitch_stab.powf(dt)
        } else {
            1.0
        };
        let rf = if roll_input_abs < specs.input_deadzone {
            specs.roll_stab.powf(dt)
        } else {
            1.0
        };
        let yf = if yaw_input_abs < specs.input_deadzone {
            specs.yaw_stab.powf(dt)
        } else {
            1.0
        };
        local_ang.x *= pf;
        local_ang.y *= yf;
        local_ang.z *= rf;
        velocity.angvel = transform.rotation.mul_vec3(local_ang);

        // === ARCADE-REALISTIC VELOCITY CONTROL ===

        if flight.throttle > specs.throttle_deadzone {
            // Engine on: Direct thrust control (arcade style)
            let target_forward_speed = specs.max_forward_speed * flight.throttle * boost_multiplier;
            let target_forward_velocity = transform.forward() * target_forward_speed;

            // Banked-lift feedback: reduce lift when wings are banked
            let up_align = transform.up().dot(Vec3::Y).abs();
            let lift_mult = 1.0 - specs.bank_lift_scale * (1.0 - up_align);
            let lift_force = transform.up() * flight.throttle * specs.lift_per_throttle * lift_mult;

            let target_linear_velocity = target_forward_velocity + lift_force;

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
