#![allow(clippy::type_complexity)]
use crate::components::{Car, F16, Helicopter, Yacht};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// World boundary configuration for finite world
#[derive(Resource)]
pub struct WorldBounds {
    pub min: Vec2,
    pub max: Vec2,
    pub pushback_strength: f32,
    pub pushback_zone: f32, // Distance from edge where pushback starts
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            min: Vec2::new(-2048.0, -2048.0), // 4km x 4km world
            max: Vec2::new(2048.0, 2048.0),
            pushback_strength: 100.0, // Gentle but firm
            pushback_zone: 100.0,     // Start pushing back 100m from edge
        }
    }
}

impl WorldBounds {
    /// Check if position is within world bounds
    pub fn contains(&self, position: Vec2) -> bool {
        position.x >= self.min.x
            && position.x <= self.max.x
            && position.y >= self.min.y
            && position.y <= self.max.y
    }

    /// Get pushback force for position near boundaries  
    pub fn get_pushback_force(&self, position: Vec2) -> Vec2 {
        let mut force = Vec2::ZERO;

        // X-axis pushback
        if position.x < self.min.x + self.pushback_zone {
            let distance = position.x - self.min.x;
            if distance < self.pushback_zone {
                let strength = (1.0 - distance / self.pushback_zone).max(0.0);
                force.x = strength * self.pushback_strength;
            }
        } else if position.x > self.max.x - self.pushback_zone {
            let distance = self.max.x - position.x;
            if distance < self.pushback_zone {
                let strength = (1.0 - distance / self.pushback_zone).max(0.0);
                force.x = -strength * self.pushback_strength;
            }
        }

        // Z-axis pushback (using .y for z-coordinate)
        if position.y < self.min.y + self.pushback_zone {
            let distance = position.y - self.min.y;
            if distance < self.pushback_zone {
                let strength = (1.0 - distance / self.pushback_zone).max(0.0);
                force.y = strength * self.pushback_strength;
            }
        } else if position.y > self.max.y - self.pushback_zone {
            let distance = self.max.y - position.y;
            if distance < self.pushback_zone {
                let strength = (1.0 - distance / self.pushback_zone).max(0.0);
                force.y = -strength * self.pushback_strength;
            }
        }

        force
    }
}

/// System that enforces world boundaries for all vehicles
pub fn world_boundary_system(
    bounds: Res<WorldBounds>,
    mut vehicle_query: Query<
        (&mut Velocity, &Transform),
        Or<(With<Car>, With<Helicopter>, With<F16>, With<Yacht>)>,
    >,
) {
    for (mut velocity, transform) in vehicle_query.iter_mut() {
        let position_2d = Vec2::new(transform.translation.x, transform.translation.z);

        // Apply gentle pushback force near boundaries
        let pushback = bounds.get_pushback_force(position_2d);
        if pushback != Vec2::ZERO {
            // Apply pushback as velocity modification (gentler than force)
            velocity.linvel.x += pushback.x * 0.1; // Gentle pushback
            velocity.linvel.z += pushback.y * 0.1;

            // Visual feedback could be added here (screen shake, sound, etc.)
        }

        // Hard boundary check (safety net)
        if !bounds.contains(position_2d) {
            // Teleport back to nearest valid position (emergency measure)
            let clamped_x = position_2d
                .x
                .clamp(bounds.min.x + 10.0, bounds.max.x - 10.0);
            let clamped_z = position_2d
                .y
                .clamp(bounds.min.y + 10.0, bounds.max.y - 10.0);

            warn!(
                "Vehicle exceeded world bounds - teleporting back from {:?} to ({:.1}, {:.1})",
                position_2d, clamped_x, clamped_z
            );

            // This would need additional position updating logic
            // For now, just log the violation
        }
    }
}

/// System specifically for aircraft boundary handling
pub fn aircraft_boundary_system(
    bounds: Res<WorldBounds>,
    mut aircraft_query: Query<(&mut Velocity, &Transform), Or<(With<Helicopter>, With<F16>)>>,
) {
    for (mut velocity, transform) in aircraft_query.iter_mut() {
        let position_2d = Vec2::new(transform.translation.x, transform.translation.z);

        // Aircraft get stronger pushback and turning forces
        let pushback = bounds.get_pushback_force(position_2d);
        if pushback != Vec2::ZERO {
            // Apply both linear pushback and angular turning
            velocity.linvel.x += pushback.x * 0.2; // Stronger than ground vehicles
            velocity.linvel.z += pushback.y * 0.2;

            // Add turning torque to help aircraft turn back
            let turn_strength = pushback.length() * 0.1;
            if pushback.x > 0.0 {
                velocity.angvel.y += turn_strength; // Turn right
            } else if pushback.x < 0.0 {
                velocity.angvel.y -= turn_strength; // Turn left
            }
        }

        // Altitude limits for aircraft
        if transform.translation.y > 2000.0 {
            velocity.linvel.y -= 50.0; // Push down from max altitude
        }
    }
}
