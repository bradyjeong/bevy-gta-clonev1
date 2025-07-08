use bevy::prelude::*;

/// Unified transform factory for consistent entity positioning
#[derive(Resource)]
pub struct TransformFactory;

impl TransformFactory {
    #[must_use] pub fn new() -> Self {
        Self
    }

    // Basic positioning
    #[must_use] pub fn at_ground_level(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.0, z)
    }

    #[must_use] pub fn at_position(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // Vehicle transforms
    #[must_use] pub fn vehicle_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.5, z)
    }

    #[must_use] pub fn vehicle_elevated(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // Aircraft transforms
    #[must_use] pub fn helicopter_spawn(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    #[must_use] pub fn aircraft_spawn(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // Building transforms
    #[must_use] pub fn building_spawn(x: f32, z: f32, height: f32) -> Transform {
        Transform::from_xyz(x, height / 2.0, z)
    }

    // NPC transforms
    #[must_use] pub fn npc_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.9, z) // NPC height offset
    }

    // Environment transforms
    #[must_use] pub fn tree_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 1.5, z) // Tree height offset
    }

    // Utility transforms
    #[must_use] pub fn with_rotation(position: Vec3, rotation: Quat) -> Transform {
        Transform {
            translation: position,
            rotation,
            scale: Vec3::ONE,
        }
    }

    #[must_use] pub fn with_scale(position: Vec3, scale: Vec3) -> Transform {
        Transform {
            translation: position,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    #[must_use] pub fn random_rotation_y() -> Quat {
        let angle = fastrand::f32() * std::f32::consts::TAU;
        Quat::from_rotation_y(angle)
    }

    #[must_use] pub fn helicopter_body() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    #[must_use] pub fn landing_skid_left() -> Transform {
        Transform::from_xyz(-0.5, -0.5, 0.0)
    }

    #[must_use] pub fn landing_skid_right() -> Transform {
        Transform::from_xyz(0.5, -0.5, 0.0)
    }

    #[must_use] pub fn vehicle_body_center() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    #[must_use] pub fn vehicle_chassis() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }

    #[must_use] pub fn wheel_with_rotation(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }
}

impl Default for TransformFactory {
    fn default() -> Self {
        Self::new()
    }
}
