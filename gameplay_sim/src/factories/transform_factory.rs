use bevy::prelude::*;

/// Unified transform factory for consistent entity positioning
#[derive(Resource)]
pub struct TransformFactory;

impl TransformFactory {
    pub fn new() -> Self {
        Self
    }

    // Basic positioning
    pub fn at_ground_level(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.0, z)
    }

    pub fn at_position(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // Vehicle transforms
    pub fn vehicle_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.5, z)
    }

    pub fn vehicle_elevated(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // Aircraft transforms
    pub fn helicopter_spawn(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    pub fn aircraft_spawn(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    }

    // Building transforms
    pub fn building_spawn(x: f32, z: f32, height: f32) -> Transform {
        Transform::from_xyz(x, height / 2.0, z)
    }

    // NPC transforms
    pub fn npc_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.9, z) // NPC height offset
    }

    // Environment transforms
    pub fn tree_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 1.5, z) // Tree height offset
    }

    // Utility transforms
    pub fn with_rotation(position: Vec3, rotation: Quat) -> Transform {
        Transform {
            translation: position,
            rotation,
            scale: Vec3::ONE,
        }
    }

    pub fn with_scale(position: Vec3, scale: Vec3) -> Transform {
        Transform {
            translation: position,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    pub fn random_rotation_y() -> Quat {
        let angle = fastrand::f32() * std::f32::consts::TAU;
        Quat::from_rotation_y(angle)
    }
}

impl Default for TransformFactory {
    fn default() -> Self {
        Self::new()
    }
}
