use crate::components::unified_water::UnifiedWaterBody;
use crate::components::water::{Yacht, YachtSpecs};
use crate::config::GameConfig;
use crate::systems::movement::simple_yacht::YachtSpecsHandle;
use crate::systems::physics::PhysicsUtilities;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// General buoyancy system for ALL yachts (not just player-controlled)
/// Keeps yachts floating at the correct water level using velocity correction
/// Also applies self-righting torque to keep yacht upright
pub fn simple_yacht_buoyancy(
    time: Res<Time>,
    config: Res<GameConfig>,
    yacht_specs: Res<Assets<YachtSpecs>>,
    water_regions: Query<&UnifiedWaterBody>,
    mut query: Query<
        (
            &mut Velocity,
            &Transform,
            &mut ExternalForce,
            &YachtSpecsHandle,
        ),
        With<Yacht>,
    >,
) {
    for (mut velocity, transform, mut external_force, specs_handle) in query.iter_mut() {
        let Some(specs) = yacht_specs.get(&specs_handle.0) else {
            continue;
        };

        // Check if yacht is in a water region
        if let Some(water) = water_regions
            .iter()
            .find(|w| w.contains_point(transform.translation.x, transform.translation.z))
        {
            let water_level = water.get_base_water_level(time.elapsed_secs());
            let target_y = water_level + specs.draft;
            let y_error = target_y - transform.translation.y;

            // Proportional control with smooth blending to prevent physics jitter
            let target_vy = (y_error * specs.buoyancy_strength).clamp(-5.0, 5.0);
            velocity.linvel.y = velocity.linvel.y.lerp(target_vy, 0.1);

            // Self-righting torque: yacht naturally returns to upright position
            let up = transform.up();
            let water_normal = Vec3::Y;
            let axis = up.cross(water_normal);
            let sin_angle = axis.length().clamp(0.0, 1.0);

            if sin_angle > 0.001 {
                let axis_n = axis / (sin_angle + 1e-6);
                let roll_rate = velocity.angvel.dot(axis_n);

                // PD controller gains tuned from yacht specs (increased for faster correction)
                let kp = specs.buoyancy_strength * 3500.0; // Increased from 1500.0
                let kd = specs.angular_damping * 350.0; // Increased from 150.0
                let mut torque_mag = kp * sin_angle - kd * roll_rate;

                // Clamp to prevent solver explosions
                let torque_limit = 70_000.0; // Increased from 30_000.0
                torque_mag = torque_mag.clamp(-torque_limit, torque_limit);

                external_force.torque += axis_n * torque_mag;
            }
        }

        // Apply velocity clamping to prevent physics solver panics
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}
