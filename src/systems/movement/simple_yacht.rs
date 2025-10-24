use crate::components::control_state::ControlState;
use crate::components::unified_water::UnifiedWaterBody;
use crate::components::water::{Yacht, YachtSpecs};
use crate::components::{ActiveEntity, MissingSpecsWarned, PlayerControlled};
use crate::config::GameConfig;
use crate::systems::physics::PhysicsUtilities;
use crate::util::safe_math::{safe_lerp, safe_lerp_f32};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct YachtSpecsHandle(pub Handle<YachtSpecs>);

#[allow(clippy::type_complexity)]
pub fn simple_yacht_movement(
    time: Res<Time>,
    config: Res<GameConfig>,
    yacht_specs: Res<Assets<YachtSpecs>>,
    water_regions: Query<&UnifiedWaterBody>,
    mut commands: Commands,
    warned_query: Query<(), With<MissingSpecsWarned>>,
    mut query: Query<
        (
            Entity,
            &mut Velocity,
            &Transform,
            &ControlState,
            &YachtSpecsHandle,
        ),
        (With<Yacht>, With<ActiveEntity>, With<PlayerControlled>),
    >,
) {
    let dt = PhysicsUtilities::stable_dt(&time);

    for (entity, mut velocity, transform, controls, specs_handle) in query.iter_mut() {
        let Some(specs) = yacht_specs.get(&specs_handle.0) else {
            if !warned_query.contains(entity) {
                warn!(
                    "Yacht entity {:?} missing loaded specs - will skip until loaded",
                    entity
                );
                commands.entity(entity).insert(MissingSpecsWarned);
            }
            continue;
        };

        // Safety: prevent NaN/physics issues from modded configs
        let max_speed = specs.max_speed.clamp(1.0, 100.0);
        let throttle_ramp = specs.throttle_ramp.clamp(0.1, 20.0);
        let boat_grip = specs.boat_grip.clamp(0.1, 50.0);
        let drag_factor = specs.drag_factor.clamp(0.9, 1.0);

        // Convert world velocity to local yacht space
        let inv_rotation = transform.rotation.inverse();
        let mut v_local = inv_rotation * velocity.linvel;

        // Check if yacht is in water (prop-in-water check to prevent land driving)
        let in_water = water_regions
            .iter()
            .any(|w| w.contains_point(transform.translation.x, transform.translation.z));

        // Forward/backward speed control (arcade style)
        let mut input_throttle = controls.throttle - controls.brake;

        // Gate throttle if beached (GTA-style behavior)
        if !in_water {
            input_throttle = 0.0;
        }

        let target_speed = max_speed * input_throttle.clamp(-0.5, 1.0);

        // Smooth acceleration with frame-independent lerp
        let accel_rate = if input_throttle.abs() > 0.05 {
            throttle_ramp
        } else {
            throttle_ramp * 2.0 // Faster deceleration when no input
        };
        v_local.z = safe_lerp_f32(v_local.z, -target_speed, dt * accel_rate);

        // Lateral grip: boats slide more than cars (tunable from specs)
        v_local.x = safe_lerp_f32(v_local.x, 0.0, dt * boat_grip);

        // Coasting drag when no throttle input
        if input_throttle.abs() < 0.05 {
            let frame_drag = drag_factor.powf(dt);
            v_local.z *= frame_drag;
        }

        // Rudder control: yaw rate increases with speed (realistic boat behavior)
        let speed_factor = (v_local.z.abs() / max_speed).min(1.0);
        let rudder_effectiveness = 0.3 + 0.7 * speed_factor;
        let target_yaw = controls.steering * rudder_effectiveness * 0.6;

        let yaw_lerp_rate = 6.0;
        velocity.angvel = safe_lerp(
            velocity.angvel,
            Vec3::new(0.0, target_yaw, 0.0),
            dt * yaw_lerp_rate,
        );

        // Convert local velocity back to world space
        // Note: Y velocity is handled by separate simple_yacht_buoyancy system
        let world_v = transform.rotation * v_local;
        velocity.linvel.x = world_v.x;
        velocity.linvel.z = world_v.z;

        // Apply velocity clamping to prevent physics solver panics
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}
