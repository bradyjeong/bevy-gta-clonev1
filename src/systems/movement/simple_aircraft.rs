#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::{
    ActiveEntity, AircraftFlight, ControlState, F16, Helicopter, HelicopterRuntime, MainRotor,
    MissingSpecsWarned, PlayerControlled, SimpleF16Specs, SimpleF16SpecsHandle,
    SimpleHelicopterSpecs, SimpleHelicopterSpecsHandle, TailRotor, VehicleHealth,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::config::GameConfig;
use crate::systems::movement::simple_flight_common::SimpleFlightCommon;
use crate::systems::physics::PhysicsUtilities;
use crate::util::safe_math::safe_lerp;

/// Simplified F16 flight system following AGENT.MD simplicity principles
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
    f16_specs_assets: Res<Assets<SimpleF16Specs>>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut f16_query: Query<
        (
            Entity,
            &mut Velocity,
            &Transform,
            &mut AircraftFlight,
            &SimpleF16SpecsHandle,
            &ControlState,
        ),
        (With<F16>, With<ActiveEntity>, With<PlayerControlled>),
    >,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    for (entity, mut velocity, transform, mut flight, specs_handle, control_state) in
        f16_query.iter_mut()
    {
        let Some(specs) = f16_specs_assets.get(&specs_handle.0) else {
            if !warned_query.contains(entity) {
                warn!(
                    "F16 entity {:?} missing loaded specs - will skip until loaded",
                    entity
                );
                commands.entity(entity).insert(MissingSpecsWarned);
            }
            continue;
        };
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
        // Safety: prevent NaN/physics issues from modded configs
        let afterburner_multiplier = specs.afterburner_multiplier.clamp(1.0, 3.0);
        let boost_multiplier = if flight.afterburner_active {
            afterburner_multiplier
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

            // Safety: prevent NaN/physics issues from modded configs
            let auto_bank_max_rate = specs.auto_bank_max_rate.clamp(0.0, 10.0);
            let bank = if roll_input_abs < specs.input_deadzone {
                (lateral_speed * specs.auto_bank_gain * control_eff)
                    .clamp(-auto_bank_max_rate, auto_bank_max_rate)
            } else {
                0.0
            };

            (pitch, roll, yaw, bank)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        // Combine control inputs (GTA-style: yaw is world-space, pitch/roll are local)
        // Pitch and roll in local space (aircraft body axes)
        let pr_local = Vec3::new(
            pitch_cmd + pitch_auto,
            0.0,
            roll_cmd + roll_auto + roll_bank,
        );
        let pr_world = transform.rotation.mul_vec3(pr_local);
        
        // Yaw in world space (always around world-up axis for arcade feel)
        let world_yaw = Vec3::new(0.0, yaw_cmd + yaw_auto, 0.0);
        let world_target_ang = pr_world + world_yaw;

        // Apply angular velocity with lerp
        // Safety: prevent NaN/physics issues from modded configs
        let angular_lerp_factor = specs.angular_lerp_factor.clamp(1.0, 20.0);
        velocity.angvel = safe_lerp(velocity.angvel, world_target_ang, dt * angular_lerp_factor);

        // GTA-style multiplicative damping (pitch/roll in local space, yaw in world space)
        // Safety: prevent NaN/physics issues from modded configs (critical for powf bases!)
        let pitch_stab = specs.pitch_stab.clamp(0.5, 1.0);
        let roll_stab = specs.roll_stab.clamp(0.5, 1.0);
        let yaw_stab = specs.yaw_stab.clamp(0.5, 1.0);
        let inv_rot = transform.rotation.inverse();
        let mut local_ang = inv_rot.mul_vec3(velocity.angvel);
        let safe_dt = dt.clamp(0.0, 1.0); // Prevent NaN from negative/huge dt
        
        // Pitch and roll damping in local space
        let pf = if pitch_input_abs < specs.input_deadzone {
            pitch_stab.powf(safe_dt)
        } else {
            1.0
        };
        let rf = if roll_input_abs < specs.input_deadzone {
            roll_stab.powf(safe_dt)
        } else {
            1.0
        };
        local_ang.x *= pf;
        local_ang.z *= rf;
        velocity.angvel = transform.rotation.mul_vec3(local_ang);
        
        // Yaw damping in world space (pure world-Y rotation for arcade)
        if yaw_input_abs < specs.input_deadzone {
            let yf = yaw_stab.powf(safe_dt);
            let yaw_world = velocity.angvel.dot(Vec3::Y);
            let other = velocity.angvel - Vec3::Y * yaw_world;
            velocity.angvel = other + Vec3::Y * (yaw_world * yf);
        }

        // === ARCADE-REALISTIC VELOCITY CONTROL ===

        if flight.throttle > specs.throttle_deadzone {
            // Engine on: Direct thrust control (arcade style)
            // Clamp to prevent modded configs from breaking physics
            let max_forward_speed = specs.max_forward_speed.clamp(50.0, 500.0);
            let linear_lerp_factor = specs.linear_lerp_factor.clamp(1.0, 20.0);
            // Safety: clamp final target speed to max velocity config
            let target_forward_speed = (max_forward_speed * flight.throttle * boost_multiplier)
                .min(config.physics.max_velocity);
            let target_forward_velocity = transform.forward() * target_forward_speed;

            // Banked-lift feedback: reduce lift when wings are banked
            let up_align = transform.up().dot(Vec3::Y).abs();
            let lift_mult = 1.0 - specs.bank_lift_scale * (1.0 - up_align);
            let lift_force = transform.up() * flight.throttle * specs.lift_per_throttle * lift_mult;

            let target_linear_velocity = target_forward_velocity + lift_force;

            velocity.linvel = safe_lerp(
                velocity.linvel,
                target_linear_velocity,
                dt * linear_lerp_factor,
            );
        } else {
            // Engine off: Apply frame-rate independent momentum decay (gliding like GTA V)
            // Clamp to prevent modded configs from breaking physics
            let drag_factor = specs.drag_factor.clamp(0.9, 1.0);
            let safe_dt = dt.clamp(0.0, 1.0); // Prevent NaN from negative/huge dt
            let frame_drag = drag_factor.powf(safe_dt);
            let vertical_drag = 0.999_f32.powf(safe_dt); // Slight vertical drag for realistic sink rate
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

/// Force-based helicopter physics following oracle's realistic flight model
///
/// Replaces direct velocity manipulation with force/torque-based system:
/// - RPM-based lift with realistic spool-up/down
/// - Main rotor cyclic tilt for directional control
/// - Tail rotor for yaw authority
/// - Damage affects control authority
/// - Proper physics integration via ExternalForce
pub fn simple_helicopter_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    rapier_context: ReadRapierContext,
    heli_specs_assets: Res<Assets<SimpleHelicopterSpecs>>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut helicopter_query: Query<
        (
            Entity,
            &mut Velocity,
            &Transform,
            &ControlState,
            &SimpleHelicopterSpecsHandle,
            &mut HelicopterRuntime,
            &mut ExternalForce,
            &AdditionalMassProperties,
            Option<&VehicleHealth>,
        ),
        (With<Helicopter>, With<ActiveEntity>, With<PlayerControlled>),
    >,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    let Ok(context) = rapier_context.single() else {
        return;
    };

    for (
        entity,
        mut velocity,
        transform,
        control_state,
        specs_handle,
        mut runtime,
        mut external_force,
        mass_props,
        vehicle_health,
    ) in helicopter_query.iter_mut()
    {
        let Some(specs) = heli_specs_assets.get(&specs_handle.0) else {
            if !warned_query.contains(entity) {
                warn!(
                    "Helicopter entity {:?} missing loaded specs - will skip until loaded",
                    entity
                );
                commands.entity(entity).insert(MissingSpecsWarned);
            }
            continue;
        };

        // === 0. GROUND DETECTION ===
        let on_ground = if let Some((_, distance)) = context.cast_ray(
            transform.translation,
            Vec3::NEG_Y,
            specs.ground_ray_length,
            true,
            QueryFilter::default().exclude_rigid_body(entity),
        ) {
            // More lenient ground check (landing gear height) + must be stopped
            distance < 2.0 && velocity.linvel.length() < 0.5
        } else {
            false
        };

        // === 1. AUTHORITY SCALING (DAMAGE) ===
        let health_pct = vehicle_health.map_or(1.0, |h| (h.current / h.max).clamp(0.0, 1.0));
        let damage_authority_min = specs.damage_authority_min.clamp(0.0, 1.0);
        let dmg_scale = damage_authority_min + (1.0 - damage_authority_min) * health_pct;

        // === 2. RPM EFFECTIVENESS ===
        let min_rpm_for_lift = specs.min_rpm_for_lift.clamp(0.0, 0.9);
        let rpm_eff = if runtime.rpm < min_rpm_for_lift {
            0.0
        } else {
            let rpm_to_lift_exp = specs.rpm_to_lift_exp.clamp(1.0, 3.0);
            ((runtime.rpm - min_rpm_for_lift) / (1.0 - min_rpm_for_lift))
                .clamp(0.0, 1.0)
                .powf(rpm_to_lift_exp)
        };

        // === INPUT PROCESSING WITH DEADZONE ===
        let dz = specs.input_deadzone.clamp(0.0, 0.3);
        let pitch_input_abs = control_state.pitch.abs();
        let roll_input_abs = control_state.roll.abs();
        let yaw_input_abs = control_state.yaw.abs();

        let yaw_cmd = if yaw_input_abs < dz {
            0.0
        } else {
            -control_state.yaw.signum() * specs.yaw_rate
        };

        // Vertical input (keep for lift control)
        let vertical_input_abs = control_state.vertical.abs();
        let vertical = if vertical_input_abs < dz {
            0.0
        } else {
            control_state.vertical.signum() * ((vertical_input_abs - dz) / (1.0 - dz))
        };

        // === 3. RPM UPDATE (CONTINUOUS) ===
        // Three-tier rotor speed based on input:
        // - Throttle up (Shift): 2.0 → 40 rad/s (fastest with blur)
        // - Hovering (neutral): 1.0 → 20 rad/s (medium with blur)
        // - Descending (C key): 0.7 → 14 rad/s (slowest with blur)
        let target_rpm = if on_ground && vertical_input_abs < dz {
            0.0
        } else if control_state.vertical < -0.1 {
            // Descending with C key - 14 rad/s
            0.7
        } else if control_state.vertical > 0.1 {
            // Throttling up with Shift - 40 rad/s
            2.0
        } else {
            // Hovering (neutral) - 20 rad/s
            1.0
        };
        let rate = if target_rpm > runtime.rpm {
            specs.spool_up_rate.clamp(0.1, 2.0)
        } else {
            specs.spool_down_rate.clamp(0.1, 2.0)
        };
        runtime.rpm += rate * dt * (target_rpm - runtime.rpm);
        runtime.rpm = runtime.rpm.clamp(0.0, 2.0);

        // === YAW-ONLY PHYSICS ROTATION (GTA-STYLE) ===
        // Physics body only rotates for yaw, stays level for lift calculation
        let local_target_ang = Vec3::new(0.0, yaw_cmd, 0.0) * rpm_eff * dmg_scale;
        let world_target_ang = transform.rotation.mul_vec3(local_target_ang);
        velocity.angvel = safe_lerp(
            velocity.angvel,
            world_target_ang,
            dt * specs.angular_lerp_factor.clamp(1.0, 20.0),
        );

        // === YAW DAMPING ===
        let inv_rot = transform.rotation.inverse();
        let mut local_ang = inv_rot.mul_vec3(velocity.angvel);

        let yf = if yaw_input_abs < dz {
            specs.yaw_stab.clamp(0.5, 1.0).powf(dt)
        } else {
            1.0
        };
        local_ang.y *= yf;

        velocity.angvel = transform.rotation.mul_vec3(local_ang);

        // === 6. LIFT CALCULATION ===
        let mass = match mass_props {
            AdditionalMassProperties::Mass(m) => m.max(1.0),
            AdditionalMassProperties::MassProperties(mp) => mp.mass.max(1.0),
        };
        let hover_force = mass * 9.81;

        let max_lift_margin_g = specs.max_lift_margin_g.clamp(1.0, 3.0);

        // === ARCADE-STYLE VERTICAL CONTROL ===
        // GTA-style lift: Zero when on ground with no throttle, allows settling via physics
        let lift_g = if on_ground && vertical_input_abs < dz {
            0.0
        } else {
            // Direct vertical control (arcade):
            // Shift (+1.0) → 1.03 + 0.75 = 1.78G → Climb
            // Nothing (0.0) → 1.03 + 0.0 = 1.03G → Stable hover
            // Ctrl (-1.0) → 1.03 - 0.75 = 0.28G → Descend
            (1.0 + specs.hover_bias + specs.collective_gain * vertical)
                .clamp(0.0, max_lift_margin_g)
        };
        let lift_mag = hover_force * lift_g;

        // === GTA-STYLE DIRECT HORIZONTAL THRUST ===
        // Vertical lift (always upward)
        let lift_force = Vec3::Y * lift_mag * rpm_eff * dmg_scale;

        // Direct horizontal thrust forces (no tilt required)
        let forward_input = if pitch_input_abs < dz {
            0.0
        } else {
            control_state.pitch
        };
        let strafe_input = if roll_input_abs < dz {
            0.0
        } else {
            control_state.roll
        };

        let forward_thrust =
            *transform.forward() * forward_input * specs.forward_thrust * rpm_eff * dmg_scale;
        let strafe_thrust =
            *transform.right() * strafe_input * specs.strafe_thrust * rpm_eff * dmg_scale;

        let main_force = lift_force + forward_thrust + strafe_thrust;

        // === 7. HORIZONTAL DRAG ===
        let vel_horizontal = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z);
        let horiz_drag = specs.horiz_drag.clamp(0.0, 10.0);
        let drag_force = -vel_horizontal * (horiz_drag * mass);

        // === 8. GROUND DAMPING ===
        // GTA-style: Apply extra damping when on ground for stable landing
        if on_ground {
            velocity.linvel *= 0.9_f32.powf(dt);
            velocity.angvel *= 0.85_f32.powf(dt);
        }

        // === 9. APPLY FORCES ===
        external_force.force = main_force + drag_force;
        external_force.torque = Vec3::ZERO;

        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}

/// Spool down helicopter rotors when not controlled by player
pub fn spool_helicopter_rpm_idle(
    time: Res<Time>,
    heli_specs_assets: Res<Assets<SimpleHelicopterSpecs>>,
    mut helicopter_query: Query<
        (
            &SimpleHelicopterSpecsHandle,
            &mut HelicopterRuntime,
            Option<&VehicleHealth>,
        ),
        (With<Helicopter>, Without<PlayerControlled>),
    >,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    for (specs_handle, mut runtime, _vehicle_health) in helicopter_query.iter_mut() {
        let Some(specs) = heli_specs_assets.get(&specs_handle.0) else {
            continue;
        };

        let target_rpm = 0.0;

        let rate = specs.spool_down_rate.clamp(0.1, 2.0);
        runtime.rpm += rate * dt * (target_rpm - runtime.rpm);
        runtime.rpm = runtime.rpm.clamp(0.0, 1.0);
    }
}

/// Rotate helicopter main and tail rotors every frame
pub fn rotate_helicopter_rotors(
    time: Res<Time>,
    heli_specs_assets: Res<Assets<SimpleHelicopterSpecs>>,
    mut rotor_query: Query<(
        &mut Transform,
        Option<&MainRotor>,
        Option<&TailRotor>,
        &ChildOf,
    )>,
    visual_body_query: Query<&ChildOf, With<crate::components::HelicopterVisualBody>>,
    helicopter_query: Query<(&SimpleHelicopterSpecsHandle, &HelicopterRuntime), With<Helicopter>>,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    for (mut transform, main_rotor, tail_rotor, child_of) in rotor_query.iter_mut() {
        // Rotors are children of HelicopterVisualBody, which is child of Helicopter
        // Need to look up grandparent
        let visual_body_parent = child_of.parent();
        let Ok(visual_body_child_of) = visual_body_query.get(visual_body_parent) else {
            continue;
        };
        let helicopter_entity = visual_body_child_of.parent();

        let Ok((specs_handle, runtime)) = helicopter_query.get(helicopter_entity) else {
            continue;
        };
        let Some(specs) = heli_specs_assets.get(&specs_handle.0) else {
            continue;
        };

        let (main_rpm, tail_rpm) = (
            specs.main_rotor_rpm.clamp(1.0, 100.0),
            specs.tail_rotor_rpm.clamp(1.0, 100.0),
        );

        if main_rotor.is_some() {
            let main_rotation_speed = main_rpm * runtime.rpm;
            transform.rotate_y(dt * main_rotation_speed);
        } else if tail_rotor.is_some() {
            let tail_rotation_speed = tail_rpm * runtime.rpm;
            transform.rotate_z(dt * tail_rotation_speed);
        }
    }
}
