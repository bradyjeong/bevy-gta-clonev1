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

        let ray_length = specs.ground_ray_length.clamp(0.5, 5.0);
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
/// Runs in FixedUpdate alongside car_movement
pub fn car_stability_system(
    car_specs_assets: Res<Assets<SimpleCarSpecs>>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut car_query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &Grounded,
            &mut ExternalForce,
            &SimpleCarSpecsHandle,
            &ReadMassProperties,
        ),
        (With<Car>, With<ActiveEntity>),
    >,
    _time: Res<Time>,
) {
    for (entity, transform, velocity, grounded, mut ext_force, specs_handle, mass_props) in
        car_query.iter_mut()
    {
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

            // Calculate roll angle (deviation from horizontal plane)
            let roll_angle = right
                .y
                .asin()
                .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

            // Roll angular velocity (project onto local forward axis)
            let roll_rate = velocity.angvel.dot(forward);

            // PD controller gains
            let kp = specs.roll_kp.clamp(0.0, 20.0);
            let kd = specs.roll_kd.clamp(0.0, 10.0);
            let limit = specs.roll_torque_limit.clamp(0.0, 2000.0);

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
            let extra_accel = specs.air_gravity_scale.clamp(0.0, 10.0) * 9.81;

            // Soft speed modulation factor (0.5 to 1.0 range)
            let horiz_speed = Vec3::new(velocity.linvel.x, 0.0, velocity.linvel.z).length();
            let base_speed = specs.base_speed.clamp(1.0, 100.0);
            let speed_factor = (horiz_speed / base_speed).clamp(0.0, 1.0);
            let accel = extra_accel * (0.5 + 0.5 * speed_factor);

            // Scale by mass for proper force application
            let mass = mass_props.mass;
            ext_force.force = Vec3::NEG_Y * (accel * mass);
            ext_force.torque = Vec3::ZERO;
        }
    }
}
