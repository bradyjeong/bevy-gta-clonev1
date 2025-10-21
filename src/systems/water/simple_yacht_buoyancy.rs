use crate::components::unified_water::UnifiedWaterBody;
use crate::components::water::{Yacht, YachtSpecs};
use crate::config::GameConfig;
use crate::systems::movement::simple_yacht::YachtSpecsHandle;
use crate::systems::physics::PhysicsUtilities;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// General buoyancy system for ALL yachts (not just player-controlled)
/// Keeps yachts floating at the correct water level using velocity correction
pub fn simple_yacht_buoyancy(
    time: Res<Time>,
    config: Res<GameConfig>,
    yacht_specs: Res<Assets<YachtSpecs>>,
    water_regions: Query<&UnifiedWaterBody>,
    mut query: Query<(&mut Velocity, &Transform, &YachtSpecsHandle), With<Yacht>>,
) {
    for (mut velocity, transform, specs_handle) in query.iter_mut() {
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

            // Proportional control: direct velocity set (no blending to prevent oscillation)
            let target_vy = (y_error * specs.buoyancy_strength).clamp(-5.0, 5.0);
            velocity.linvel.y = target_vy;

            // Upright stabilization: let Rapier's angular_damping handle this naturally
            // No manual angular velocity manipulation - keeps it simple
        }

        // Apply velocity clamping to prevent physics solver panics
        PhysicsUtilities::clamp_velocity(&mut velocity, &config);
    }
}
