use crate::components::PlayerControlled;
use crate::components::control_state::ControlState;
use crate::components::unified_water::UnifiedWaterBody;
use crate::components::water::{Yacht, YachtSpecs, YachtState};
use crate::systems::water::yacht_buoyancy::YachtSpecsHandle;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[allow(clippy::type_complexity)]
pub fn yacht_controls_system(
    time: Res<Time>,
    yacht_specs: Res<Assets<YachtSpecs>>,
    water_regions: Query<&UnifiedWaterBody>,
    mut query: Query<
        (
            &GlobalTransform,
            &mut ExternalForce,
            &Velocity,
            &mut YachtState,
            &ControlState,
            &YachtSpecsHandle,
        ),
        (With<Yacht>, With<PlayerControlled>),
    >,
) {
    let dt = time.delta_secs();

    for (transform, mut external_force, velocity, mut state, controls, specs_handle) in
        query.iter_mut()
    {
        let Some(specs) = yacht_specs.get(&specs_handle.0) else {
            continue;
        };

        let target_throttle = controls.throttle - controls.brake;
        state.throttle += (target_throttle - state.throttle) * specs.throttle_ramp * dt;
        state.throttle = state.throttle.clamp(-1.0, 1.0);

        let target_rudder = controls.steering;
        state.rudder += (target_rudder - state.rudder) * specs.rudder_ramp * dt;
        state.rudder = state.rudder.clamp(-1.0, 1.0);

        let rotation = transform.to_scale_rotation_translation().1;
        let forward = rotation * Vec3::NEG_Z;
        let right = rotation * Vec3::X;
        let forward_speed = velocity.linvel.dot(forward);

        // GTA-STYLE THRUST POINT CHECK: Only apply thrust if propeller is underwater
        // Yacht dimensions: 8.0 (width) × 2.0 (height) × 20.0 (length)
        // Thrust point is at rear of boat, below waterline
        let yacht_half_length = 10.0; // Half of 20.0m length
        let thrust_point_offset = Vec3::new(0.0, -0.5, yacht_half_length); // Rear, slightly below center
        let thrust_point_world = transform.translation() + rotation * thrust_point_offset;

        // Check if yacht is in a water region and if thrust point is underwater
        let (on_water, thrust_in_water) = water_regions
            .iter()
            .find(|w| w.contains_point(thrust_point_world.x, thrust_point_world.z))
            .map(|water| {
                let water_level = water.get_base_water_level(time.elapsed_secs());
                let in_water = thrust_point_world.y < water_level - 0.3; // Propeller must be 0.3m underwater
                (true, in_water)
            })
            .unwrap_or((false, false));

        // Update state flags for other systems to use
        state.on_water = on_water;
        state.thrust_in_water = thrust_in_water;

        // Only apply thrust forces if propeller is in water (GTA SA behavior)
        let thrust_force = if thrust_in_water {
            state.throttle * specs.max_thrust
        } else {
            // Propeller out of water - no thrust! Physics naturally stops the boat.
            0.0
        };
        let prop_force_world = forward * thrust_force;

        state.current_thrust = thrust_force;

        let speed_norm = (forward_speed.abs() / 12.0).min(1.0);
        let rudder_effectiveness = state.rudder * (0.4 + 0.6 * speed_norm);

        let rudder_torque = specs.rudder_power * rudder_effectiveness * forward_speed.signum();
        let rudder_torque_vec = Vec3::new(0.0, rudder_torque, 0.0);

        let lateral_carve_force = right * (-state.rudder) * 15000.0 * (0.3 + 0.7 * speed_norm);

        let clamped_thrust = prop_force_world.clamp(Vec3::splat(-300000.0), Vec3::splat(300000.0));
        let clamped_rudder = rudder_torque_vec.clamp(Vec3::splat(-250000.0), Vec3::splat(250000.0));
        let clamped_lateral =
            lateral_carve_force.clamp(Vec3::splat(-50000.0), Vec3::splat(50000.0));

        if clamped_thrust.is_finite() && clamped_rudder.is_finite() && clamped_lateral.is_finite() {
            external_force.force += clamped_thrust + clamped_lateral;
            external_force.torque += clamped_rudder;
        }

        state.current_rudder = rudder_effectiveness;
    }
}
