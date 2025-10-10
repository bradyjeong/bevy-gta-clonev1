#![allow(clippy::type_complexity)]
use crate::components::world::WorldBounds;
use crate::components::{Car, F16, Helicopter, Yacht};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// System that enforces world boundaries for all vehicles
pub fn world_boundary_system(
    bounds: Res<WorldBounds>,
    mut vehicle_query: Query<
        (&mut Velocity, &Transform),
        Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>)>,
    >,
) {
    const PUSHBACK_STRENGTH: f32 = 100.0;

    for (mut velocity, transform) in vehicle_query.iter_mut() {
        // Apply gentle pushback force near boundaries
        let pushback = bounds.get_pushback_force(transform.translation, PUSHBACK_STRENGTH);
        if pushback != Vec3::ZERO {
            // Apply pushback as velocity modification (gentler than force)
            velocity.linvel.x += pushback.x * 0.1; // Gentle pushback
            velocity.linvel.z += pushback.z * 0.1;

            // Visual feedback could be added here (screen shake, sound, etc.)
        }

        // Hard boundary check (safety net) - actually clamp position
        if !bounds.is_in_bounds(transform.translation) {
            warn!(
                "Vehicle exceeded world bounds at ({:.1}, {:.1}) - clamping back",
                transform.translation.x, transform.translation.z
            );

            // TODO: Clamp Transform via Commands (need mutable access)
            // let _clamped_pos = bounds.clamp_to_bounds(transform.translation);

            // For now, zero velocity perpendicular to boundary
            // For now, zero velocity perpendicular to boundary to prevent re-escape
            let out_x =
                transform.translation.x < bounds.min_x || transform.translation.x > bounds.max_x;
            let out_z =
                transform.translation.z < bounds.min_z || transform.translation.z > bounds.max_z;

            if out_x {
                velocity.linvel.x = 0.0;
            }
            if out_z {
                velocity.linvel.z = 0.0;
            }
        }
    }
}

/// System specifically for aircraft boundary handling
pub fn aircraft_boundary_system(
    bounds: Res<WorldBounds>,
    mut aircraft_query: Query<(&mut Velocity, &Transform), Or<(With<Helicopter>, With<F16>)>>,
) {
    const AIRCRAFT_PUSHBACK_STRENGTH: f32 = 150.0;

    for (mut velocity, transform) in aircraft_query.iter_mut() {
        // Aircraft get stronger pushback and turning forces
        let pushback = bounds.get_pushback_force(transform.translation, AIRCRAFT_PUSHBACK_STRENGTH);
        if pushback != Vec3::ZERO {
            // Apply both linear pushback and angular turning
            velocity.linvel.x += pushback.x * 0.2; // Stronger than ground vehicles
            velocity.linvel.z += pushback.z * 0.2;

            // Add turning torque to help aircraft turn back
            let turn_strength = pushback.length() * 0.1;
            if pushback.x > 0.0 {
                velocity.angvel.y += turn_strength; // Turn right
            } else if pushback.x < 0.0 {
                velocity.angvel.y -= turn_strength; // Turn left
            }
        }

        // Altitude limits for aircraft
        if transform.translation.y > 3000.0 {
            velocity.linvel.y -= 50.0; // Push down from max altitude
        }
    }
}
