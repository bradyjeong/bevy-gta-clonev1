//! Phase 2: Car stability systems
//!
//! Implements GTA-style stability helpers:
//! 1. Ground detection via raycast
//! 2. Downward force when airborne
//! 3. Roll stabilizer (upright correction)

#![allow(clippy::type_complexity)]

use crate::components::{
    ActiveEntity, Car, Grounded, MissingSpecsWarned, SimpleCarSpecs, SimpleCarSpecsHandle,
};
use crate::config::GameConfig;
use crate::systems::physics::PhysicsUtilities;
use crate::util::safe_specs::safe_clamp_f32;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Ground detection system using ReadRapierContext for raycasting
/// Runs before car_movement to update Grounded state
pub fn ground_detection_system(
    rapier_context: ReadRapierContext,
    car_specs_assets: Res<Assets<SimpleCarSpecs>>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut car_query: Query<
        (Entity, &Transform, &mut Grounded, &SimpleCarSpecsHandle),
        (With<Car>, With<ActiveEntity>),
    >,
) {
    let Ok(context) = rapier_context.single() else {
        return;
    };

    for (entity, transform, mut grounded, specs_handle) in car_query.iter_mut() {
        let Some(specs) = car_specs_assets.get(&specs_handle.0) else {
            if !warned_query.contains(entity) {
                commands.entity(entity).insert(MissingSpecsWarned);
            }
            continue;
        };

        let ray_length = safe_clamp_f32(specs.ground_ray_length, 0.5, 5.0).unwrap_or_else(|| {
            error!(
                "Invalid ground_ray_length for entity {:?} (spec {:?}), using default 2.0",
                entity, specs_handle.0
            );
            2.0
        });
        let ray_origin = transform.translation;

        // FIX 1: Exclude car's own collider from ground detection
        let filter = QueryFilter::default().exclude_rigid_body(entity);
        if let Some((_, hit_distance)) =
            context.cast_ray(ray_origin, Vec3::NEG_Y, ray_length, true, filter)
        {
            grounded.is_grounded = true;
            grounded.ground_distance = hit_distance;
        } else {
            grounded.is_grounded = false;
            grounded.ground_distance = ray_length;
        }
    }
}

/// Car stability system: downward force when airborne, roll correction when grounded
/// CRITICAL: Must run AFTER car_movement in FixedUpdate schedule
/// Reason: car_movement sets velocity, this system applies correction forces
pub fn car_stability_system(
    config: Res<GameConfig>,
    car_specs_assets: Res<Assets<SimpleCarSpecs>>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut car_query: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            &Grounded,
            &mut ExternalForce,
            &SimpleCarSpecsHandle,
            &ReadMassProperties,
        ),
        (With<Car>, With<ActiveEntity>),
    >,
    _time: Res<Time>,
) {
    for (entity, transform, mut velocity, grounded, mut ext_force, specs_handle, mass_props) in
        car_query.iter_mut()
    {
        // Bug #38: Validate velocity is finite before physics operations
        if !velocity.linvel.is_finite() || !velocity.angvel.is_finite() {
            error!(
                "Invalid velocity detected for car entity {:?}, resetting to zero",
                entity
            );
            velocity.linvel = Vec3::ZERO;
            velocity.angvel = Vec3::ZERO;
        }
        let Some(specs) = car_specs_assets.get(&specs_handle.0) else {
            if !warned_query.contains(entity) {
                commands.entity(entity).insert(MissingSpecsWarned);
            }
            continue;
        };

        if grounded.is_grounded {
            // FIX 3: ROLL STABILIZER - Apply torque about forward axis (not right)
            let right = *transform.right();
            let forward = *transform.forward();

            // Bug #1: Clamp right.y to [-1, 1] before asin() to prevent NaN
            let clamped_right_y = right.y.clamp(-1.0, 1.0);
            let roll_angle = clamped_right_y
                .asin()
                .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

            // Roll angular velocity (project onto local forward axis)
            let roll_rate = velocity.angvel.dot(forward);

            // PD controller gains
            let kp = safe_clamp_f32(specs.roll_kp, 0.0, 20.0).unwrap_or_else(|| {
                error!(
                    "Invalid roll_kp for entity {:?} (spec {:?}), using default 5.0",
                    entity, specs_handle.0
                );
                5.0
            });
            let kd = safe_clamp_f32(specs.roll_kd, 0.0, 10.0).unwrap_or_else(|| {
                error!(
                    "Invalid roll_kd for entity {:?} (spec {:?}), using default 2.0",
                    entity, specs_handle.0
                );
                2.0
            });
            let limit = safe_clamp_f32(specs.roll_torque_limit, 0.0, 2000.0).unwrap_or_else(|| {
                error!(
                    "Invalid roll_torque_limit for entity {:?} (spec {:?}), using default 500.0",
                    entity, specs_handle.0
                );
                500.0
            });

            // Deadzone to avoid jitter
            let mut u = if roll_angle.abs() < 0.01 && roll_rate.abs() < 0.01 {
                0.0
            } else {
                -(kp * roll_angle + kd * roll_rate)
            };
            u = u.clamp(-limit, limit);

            // Apply torque around forward axis to correct roll
            ext_force.torque = forward * u;
            ext_force.force = Vec3::ZERO; // Clear any airborne force
        } else {
            // FIX 2: AIRBORNE - Mass-scaled gravity acceleration
            let extra_accel =
                safe_clamp_f32(specs.air_gravity_scale, 0.0, 10.0).unwrap_or_else(|| {
                    error!(
                        "Invalid air_gravity_scale for entity {:?} (spec {:?}), using default 2.0",
                        entity, specs_handle.0
                    );
                    2.0
                }) * 9.81;

            // Soft speed modulation factor (0.5 to 1.0 range)
            let horiz_speed = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z).length();
            let base_speed = safe_clamp_f32(specs.base_speed, 1.0, 100.0).unwrap_or_else(|| {
                error!(
                    "Invalid base_speed for entity {:?} (spec {:?}), using default 20.0",
                    entity, specs_handle.0
                );
                20.0
            });
            let speed_factor = (horiz_speed / base_speed).clamp(0.0, 1.0);
            let accel = extra_accel * (0.5 + 0.5 * speed_factor);

            // Scale by mass for proper force application
            let mass = mass_props.mass.max(1.0);
            ext_force.force = Vec3::NEG_Y * (accel * mass);
            ext_force.torque = Vec3::ZERO;
        }

        // Bug #3: Clamp velocity AFTER all force/velocity modifications to prevent physics solver conflicts
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}
