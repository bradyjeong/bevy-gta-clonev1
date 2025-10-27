#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::ControlState;
use crate::components::{
    ActiveEntity, Car, Grounded, MissingSpecsWarned, SimpleCarSpecs,
    SimpleCarSpecsHandle,
};
use crate::config::GameConfig;
use crate::systems::physics::PhysicsUtilities;
use crate::util::safe_math::{safe_lerp, safe_lerp_f32};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Car movement system using asset-driven specs
pub fn car_movement(
    config: Res<GameConfig>,
    car_specs_assets: Res<Assets<SimpleCarSpecs>>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut car_query: Query<
        (
            Entity,
            &mut Velocity,
            &Transform,
            &ControlState,
            &SimpleCarSpecsHandle,
            &Grounded,
        ),
        (With<Car>, With<ActiveEntity>),
    >,
    time: Res<Time>,
) {
    #[cfg(feature = "debug-movement")]
    let start_time = std::time::Instant::now();

    for (entity, mut velocity, transform, control_state, specs_handle, grounded) in
        car_query.iter_mut()
    {
        let Some(specs) = car_specs_assets.get(&specs_handle.0) else {
            if !warned_query.contains(entity) {
                warn!(
                    "Car entity {:?} missing loaded specs - will skip until loaded",
                    entity
                );
                commands.entity(entity).insert(MissingSpecsWarned);
            }
            continue;
        };

        let dt = PhysicsUtilities::stable_dt(&time);

        // Convert world velocity to local car space for physics calculations
        let inv_rotation = transform.rotation.inverse();
        let mut v_local = inv_rotation * velocity.linvel;

        // Calculate current forward speed (-Z is forward in Bevy)
        let forward_speed = -v_local.z;
        let current_speed = forward_speed.abs();

        // PHASE 1: Auto-brake when throttle opposes velocity (GTA SA feature)
        // Detects when moving backward but trying to go forward (or vice versa)
        let auto_brake_gain = specs.auto_brake_gain.clamp(1.0, 50.0);
        let oppose_threshold = 0.5; // m/s
        let throttle = control_state.throttle;
        // FIX: Only check opposition when moving above threshold AND throttle/velocity have opposite signs
        let throttle_opposes_velocity = throttle.abs() > 0.01
            && current_speed > oppose_threshold
            && (throttle * forward_speed) < 0.0; // Negative product = opposite signs

        if throttle_opposes_velocity {
            // Auto-brake to zero before allowing acceleration in new direction
            v_local.z = safe_lerp_f32(v_local.z, 0.0, dt * auto_brake_gain);
        }

        // Speed-based steering: steering effectiveness decreases with speed
        // Safety: prevent NaN/physics issues from modded configs
        let steer_speed_drop = specs.steer_speed_drop.clamp(0.0, 1.0);
        let mut steer_gain = specs.steer_gain / (1.0 + steer_speed_drop * current_speed);

        // Phase 2: Reduce steering when airborne
        if !grounded.is_grounded {
            let airborne_scale = specs.airborne_steer_scale.clamp(0.0, 1.0);
            steer_gain *= airborne_scale;
        }

        // Stability term: auto-straighten based on lateral velocity (prevents spin-outs)
        let stability_term = -v_local.x * specs.stability;

        // Base steering from input
        let mut target_yaw = control_state.steering * steer_gain + stability_term;

        // Phase 3: Reverse steering inversion (realistic behavior)
        let reversing = v_local.z > oppose_threshold || control_state.is_reversing();
        if specs.reverse_steer_invert && reversing && !throttle_opposes_velocity {
            target_yaw = -target_yaw;
        }

        // Emergency brake handling: reduce grip and add yaw boost for drifting
        let base_grip = if control_state.emergency_brake {
            target_yaw += control_state.steering.signum() * specs.ebrake_yaw_boost;
            specs.drift_grip
        } else {
            specs.grip
        };

        // Forward/backward movement with proper brake/reverse separation
        // Bevy forward is -Z, so negate for correct direction
        // Gate acceleration/reverse to prevent fighting with auto-brake
        if !throttle_opposes_velocity && control_state.is_accelerating() {
            // Accelerate forward
            // Clamp to prevent modded configs from breaking physics
            let base_speed = specs.base_speed.clamp(1.0, 100.0);
            let accel_lerp = specs.accel_lerp.clamp(1.0, 20.0);
            let target_speed = -base_speed * control_state.throttle;
            v_local.z = safe_lerp_f32(v_local.z, target_speed, dt * accel_lerp);
        } else if control_state.brake > 0.0 {
            // Regular brake (Shift): slow down current velocity toward zero
            let brake_lerp = specs.brake_lerp.clamp(1.0, 20.0);
            v_local.z = safe_lerp_f32(v_local.z, 0.0, dt * brake_lerp * control_state.brake);
        } else if !throttle_opposes_velocity && control_state.is_reversing() {
            // Reverse (Arrow Down): move backward
            let base_speed = specs.base_speed.clamp(1.0, 100.0);
            let accel_lerp = specs.accel_lerp.clamp(1.0, 20.0);
            let target_speed = base_speed * 0.5; // Half speed for reverse
            v_local.z = safe_lerp_f32(v_local.z, target_speed, dt * accel_lerp);
        } else if !throttle_opposes_velocity {
            // No input: apply momentum decay (GTA-style coasting)
            // Clamp to prevent modded configs from breaking physics
            let drag_factor = specs.drag_factor.clamp(0.9, 1.0);
            let frame_drag = drag_factor.powf(dt);
            v_local.z *= frame_drag;
        }

        // Phase 2: Only apply lateral grip when grounded
        if grounded.is_grounded {
            // PHASE 1: Slip/friction curve for lateral grip (replaces constant grip)
            // Calculate slip ratio from lateral velocity
            let slip = v_local.x.abs();
            let slip_extremum = specs.slip_extremum.clamp(0.1, 5.0);
            let slip_asymptote = specs.slip_asymptote.clamp(1.0, 50.0);
            let slip_stiffness = specs.slip_stiffness.clamp(0.1, 5.0);

            // Rising to peak at extremum, then exponential decay toward asymptote limit (0.2)
            let slip_factor = if slip <= slip_extremum {
                // Rising phase: 0 -> 1.0 as slip goes 0..extremum
                (slip / slip_extremum).min(1.0)
            } else {
                // Falling phase: exponential decay from 1.0 toward 0.2
                let denom = (slip_asymptote - slip_extremum).max(0.001);
                let t = ((slip - slip_extremum) / denom).clamp(0.0, 1.0);
                0.2_f32 + 0.8_f32 * (-3.0_f32 * t).exp() // "asymptote" (approaches 0.2)
            };

            // Downforce effect: increase grip at high speeds for stability
            // Clamp base_speed to prevent division by zero from modded configs
            let base_speed = specs.base_speed.clamp(1.0, 100.0);
            let speed_factor = (current_speed / base_speed).min(1.0);
            let downforce_grip = base_grip * (1.0 + specs.downforce_scale * speed_factor);

            // PHASE 1: Traction circle coupling - reduce lateral grip during heavy braking
            let brake_grip_loss = specs.brake_grip_loss.clamp(0.0, 1.0);
            // Use longitudinal demand from either braking or throttle (arcade simplification)
            let long_demand = control_state
                .brake
                .max(control_state.throttle.abs())
                .clamp(0.0, 1.0);
            // Continuous coupling: reduce lateral grip as longitudinal demand increases
            let traction_circle_factor = 1.0 - brake_grip_loss * long_demand * long_demand;

            // Combine all grip modifiers: base grip * slip curve * downforce * traction circle
            let effective_grip =
                downforce_grip * slip_factor * slip_stiffness * traction_circle_factor;
            v_local.x = safe_lerp_f32(v_local.x, 0.0, dt * effective_grip);
        }
        // No lateral grip when airborne - let physics handle it

        // Convert back to world space, preserve Y velocity (gravity)
        let world_velocity = transform.rotation * v_local;
        velocity.linvel.x = world_velocity.x;
        velocity.linvel.z = world_velocity.z;
        // Y velocity handled by Rapier gravity

        // Apply angular velocity with smoothing
        // Phase 3: Reduce angular lerp when airborne for less responsive rotation
        let mut angular_lerp_factor = specs.angular_lerp_factor.clamp(1.0, 20.0);
        if !grounded.is_grounded {
            let airborne_angular_scale = specs.airborne_angular_scale.clamp(0.0, 1.0);
            angular_lerp_factor *= airborne_angular_scale;
        }
        let target_angvel = Vec3::new(0.0, target_yaw, 0.0);
        velocity.angvel = safe_lerp(velocity.angvel, target_angvel, dt * angular_lerp_factor);

        // Emergency brake multipliers (applied after calculations)
        // Only affect horizontal velocity, preserve Y for gravity
        if control_state.emergency_brake {
            // Safety: prevent NaN/physics issues from modded configs
            let emergency_brake_linear = specs.emergency_brake_linear.clamp(0.01, 1.0);
            let emergency_brake_angular = specs.emergency_brake_angular.clamp(0.01, 1.0);
            velocity.linvel.x *= emergency_brake_linear;
            velocity.linvel.z *= emergency_brake_linear;
            velocity.angvel *= emergency_brake_angular;
        }

        // Apply velocity validation every frame (critical for preventing physics panics)
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }

    // Performance monitoring (debug feature only)
    #[cfg(feature = "debug-movement")]
    {
        let processing_time = start_time.elapsed().as_millis() as f32;
        if processing_time > 1.0 {
            warn!("Car movement took {:.2}ms (> 1ms budget)", processing_time);
        }
    }
}
