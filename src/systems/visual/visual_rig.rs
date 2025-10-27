#![allow(clippy::type_complexity)]
use crate::components::{Car, SimpleCarSpecs, SimpleCarSpecsHandle, VisualRig, VisualRigRoot};
use crate::systems::physics::PhysicsUtilities;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Phase 3: Visual-only body lean system
/// Applies cosmetic roll/pitch to visual mesh based on acceleration
/// Physics body remains flat, only visual rig root child rotates
pub fn visual_rig_system(
    car_specs_assets: Res<Assets<SimpleCarSpecs>>,
    time: Res<Time>,
    mut query: Query<
        (
            &Velocity,
            &Transform,
            &SimpleCarSpecsHandle,
            &mut VisualRig,
            &Children,
        ),
        (With<Car>, Without<VisualRigRoot>),
    >,
    mut rig_roots: Query<&mut Transform, With<VisualRigRoot>>,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    for (velocity, transform, specs_handle, mut visual_rig, children) in query.iter_mut() {
        let Some(specs) = car_specs_assets.get(&specs_handle.0) else {
            continue;
        };

        // Calculate acceleration from velocity change
        let current_velocity = velocity.linvel;

        // First tick guard: prevent huge acceleration spike on first frame
        if visual_rig.last_velocity == Vec3::ZERO {
            visual_rig.last_velocity = current_velocity;
            continue;
        }

        let acceleration = (current_velocity - visual_rig.last_velocity) / dt;
        visual_rig.last_velocity = current_velocity;

        // Convert acceleration to local car space
        let inv_rotation = transform.rotation.inverse();
        let local_accel = inv_rotation * acceleration;

        // Target visual angles based on lateral/longitudinal acceleration
        // Lean into turns (negative roll for right turn, positive for left)
        let visual_roll_gain = specs.visual_roll_gain.clamp(0.0, 0.1);
        let visual_pitch_gain = specs.visual_pitch_gain.clamp(0.0, 0.1);
        let mut target_roll = -local_accel.x * visual_roll_gain;
        let mut target_pitch = -local_accel.z * visual_pitch_gain; // Pitch nose UP under forward acceleration

        // Angle clamps to prevent extreme lean
        target_roll = target_roll.clamp(-0.35, 0.35); // ~±20°
        target_pitch = target_pitch.clamp(-0.25, 0.25); // ~±14°

        // Spring-damper physics for smooth transitions
        // F = -k*x - c*v (hooke's law + damping)
        let spring = specs.visual_spring.clamp(1.0, 50.0);
        let damper = specs.visual_damper.clamp(0.1, 10.0);

        // Roll spring-damper
        let roll_error = target_roll - visual_rig.current_roll;
        let roll_force = spring * roll_error - damper * visual_rig.roll_velocity;
        visual_rig.roll_velocity += roll_force * dt;
        visual_rig.current_roll += visual_rig.roll_velocity * dt;

        // Pitch spring-damper
        let pitch_error = target_pitch - visual_rig.current_pitch;
        let pitch_force = spring * pitch_error - damper * visual_rig.pitch_velocity;
        visual_rig.pitch_velocity += pitch_force * dt;
        visual_rig.current_pitch += visual_rig.pitch_velocity * dt;

        // Apply visual rotation ONLY to VisualRigRoot child
        for child in children.iter() {
            if let Ok(mut t) = rig_roots.get_mut(child) {
                t.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    visual_rig.current_pitch,
                    0.0,
                    visual_rig.current_roll,
                );
            }
        }
    }
}
