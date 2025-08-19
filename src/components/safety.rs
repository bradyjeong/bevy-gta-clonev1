use bevy::prelude::*;

/// Tag component for entities that can move at high speeds and need bounds checking
#[derive(Component, Debug)]
pub struct HighSpeed {
    pub max_safe_speed: f32,
    pub bounds_check_interval: f32,
}

impl Default for HighSpeed {
    fn default() -> Self {
        Self {
            max_safe_speed: 100.0,
            bounds_check_interval: 1.0,
        }
    }
}

/// World bounds configuration resource
#[derive(Resource, Debug)]
pub struct WorldBounds {
    pub max_coordinate: f32,
    pub reset_position: Vec3,
    pub emergency_damping: f32,
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            max_coordinate: 1000.0, // Consistent with AGENT.md culling distances
            reset_position: Vec3::ZERO,
            emergency_damping: 0.1,
        }
    }
}
