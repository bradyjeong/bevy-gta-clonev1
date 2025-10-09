#![allow(clippy::too_many_arguments, clippy::type_complexity)]
use crate::components::ControlState;
use crate::components::{ActiveEntity, Car, SimpleCarSpecs};
use crate::config::GameConfig;
use crate::systems::physics::PhysicsUtilities;
use crate::util::safe_math::safe_lerp;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn car_movement(
    config: Res<GameConfig>,
    mut car_query: Query<
        (&mut Velocity, &Transform, &ControlState, &SimpleCarSpecs),
        (With<Car>, With<ActiveEntity>),
    >,
    time: Res<Time>,
) {
    #[cfg(feature = "debug-movement")]
    let start_time = std::time::Instant::now();

    for (mut velocity, transform, control_state, specs) in car_query.iter_mut() {
        // Early exit if no meaningful input, but still run safety checks
        let has_input = control_state.is_accelerating()
            || control_state.is_braking()
            || control_state.emergency_brake
            || control_state.steering.abs() > 0.1;

        let mut target_linear_velocity = Vec3::ZERO;
        let mut target_angular_velocity = Vec3::ZERO;

        // Only calculate target velocities when there's meaningful input
        if has_input {
            // Use clean ControlState for car controls
            if control_state.is_accelerating() {
                let forward = transform.forward();
                target_linear_velocity += forward * specs.base_speed * control_state.throttle;
            }

            if control_state.is_braking() {
                let forward = transform.forward();
                target_linear_velocity -= forward * specs.base_speed * control_state.brake;
            }

            // Steering (always when steering input present)
            if control_state.steering.abs() > 0.1 {
                target_angular_velocity.y = control_state.steering * specs.rotation_speed;
            }
        }

        // Apply movement with momentum decay when no input (GTA-style)
        let dt = PhysicsUtilities::stable_dt(&time);

        if has_input {
            let lerped_velocity = safe_lerp(
                velocity.linvel,
                target_linear_velocity,
                dt * specs.linear_lerp_factor,
            );

            // Apply X/Z movement, let Rapier handle gravity acceleration in Y
            velocity.linvel.x = lerped_velocity.x;
            velocity.linvel.z = lerped_velocity.z;
            // Don't modify Y velocity - let gravity accelerate naturally
        } else {
            // No input: Apply frame-rate independent momentum decay (car coasts like GTA V)
            let drag_per_second = specs.drag_factor;
            let frame_drag = drag_per_second.powf(dt);
            velocity.linvel.x *= frame_drag;
            velocity.linvel.z *= frame_drag;
            // Don't modify Y velocity - let gravity accelerate naturally
        }
        velocity.angvel = safe_lerp(
            velocity.angvel,
            target_angular_velocity,
            dt * specs.angular_lerp_factor,
        );

        // Emergency brake affects current velocity (more effective than target velocity)
        if control_state.emergency_brake {
            velocity.linvel *= specs.emergency_brake_linear;
            velocity.angvel *= specs.emergency_brake_angular;
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
