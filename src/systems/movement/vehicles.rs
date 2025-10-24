#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::ControlState;
use crate::components::{
    ActiveEntity, Car, MissingSpecsWarned, SimpleCarSpecs, SimpleCarSpecsHandle,
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
        ),
        (With<Car>, With<ActiveEntity>),
    >,
    time: Res<Time>,
) {
    #[cfg(feature = "debug-movement")]
    let start_time = std::time::Instant::now();

    for (entity, mut velocity, transform, control_state, specs_handle) in car_query.iter_mut() {
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
        let current_speed = (-v_local.z).abs();

        // Speed-based steering: steering effectiveness decreases with speed
        // Safety: prevent NaN/physics issues from modded configs
        let steer_speed_drop = specs.steer_speed_drop.clamp(0.0, 1.0);
        let steer_gain = specs.steer_gain / (1.0 + steer_speed_drop * current_speed);

        // Stability term: auto-straighten based on lateral velocity (prevents spin-outs)
        let stability_term = -v_local.x * specs.stability;

        // Base steering from input
        let mut target_yaw = control_state.steering * steer_gain + stability_term;

        // Emergency brake handling: reduce grip and add yaw boost for drifting
        let base_grip = if control_state.emergency_brake {
            target_yaw += control_state.steering.signum() * specs.ebrake_yaw_boost;
            specs.drift_grip
        } else {
            specs.grip
        };

        // Forward/backward movement with proper brake/reverse separation
        // Bevy forward is -Z, so negate for correct direction
        if control_state.is_accelerating() {
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
        } else if control_state.is_reversing() {
            // Reverse (Arrow Down): move backward
            let base_speed = specs.base_speed.clamp(1.0, 100.0);
            let accel_lerp = specs.accel_lerp.clamp(1.0, 20.0);
            let target_speed = base_speed * 0.5; // Half speed for reverse
            v_local.z = safe_lerp_f32(v_local.z, target_speed, dt * accel_lerp);
        } else {
            // No input: apply momentum decay (GTA-style coasting)
            // Clamp to prevent modded configs from breaking physics
            let drag_factor = specs.drag_factor.clamp(0.9, 1.0);
            let frame_drag = drag_factor.powf(dt);
            v_local.z *= frame_drag;
        }

        // Downforce effect: increase grip at high speeds for stability
        // Clamp base_speed to prevent division by zero from modded configs
        let base_speed = specs.base_speed.clamp(1.0, 100.0);
        let speed_factor = (current_speed / base_speed).min(1.0);
        let effective_grip = base_grip * (1.0 + specs.downforce_scale * speed_factor);
        v_local.x = safe_lerp_f32(v_local.x, 0.0, dt * effective_grip);

        // Convert back to world space, preserve Y velocity (gravity)
        let world_velocity = transform.rotation * v_local;
        velocity.linvel.x = world_velocity.x;
        velocity.linvel.z = world_velocity.z;
        // Y velocity handled by Rapier gravity

        // Apply angular velocity with smoothing
        // Safety: prevent NaN/physics issues from modded configs
        let angular_lerp_factor = specs.angular_lerp_factor.clamp(1.0, 20.0);
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
