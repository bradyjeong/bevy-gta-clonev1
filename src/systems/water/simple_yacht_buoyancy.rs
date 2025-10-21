use crate::components::unified_water::UnifiedWaterBody;
use crate::components::water::Yacht;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// General buoyancy system for ALL yachts (not just player-controlled)
/// Keeps yachts floating at the correct water level using velocity correction
pub fn simple_yacht_buoyancy(
    time: Res<Time>,
    water_regions: Query<&UnifiedWaterBody>,
    mut query: Query<(&mut Velocity, &Transform), With<Yacht>>,
) {
    for (mut velocity, transform) in query.iter_mut() {
        // Check if yacht is in a water region
        if let Some(water) = water_regions
            .iter()
            .find(|w| w.contains_point(transform.translation.x, transform.translation.z))
        {
            let water_level = water.get_base_water_level(time.elapsed_secs());
            let draft = 0.6; // How deep the yacht sits in water (meters)
            let target_y = water_level + draft;
            let y_error = target_y - transform.translation.y;

            // Convert vertical position error to velocity correction
            // Stronger for larger errors (proportional control)
            let vertical_gain = 3.0; // Higher = faster floating response
            let target_vy = (y_error * vertical_gain).clamp(-5.0, 5.0);

            // Apply buoyancy force via velocity
            velocity.linvel.y = velocity.linvel.y * 0.9 + target_vy * 0.1;

            // Keep yacht upright: dampen roll and pitch strongly
            let upright_damping = 0.85;
            velocity.angvel.x *= upright_damping;
            velocity.angvel.z *= upright_damping;
        }
    }
}
