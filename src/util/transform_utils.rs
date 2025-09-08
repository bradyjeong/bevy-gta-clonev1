use bevy::prelude::*;

/// Get horizontal forward direction, ignoring pitch for prone swimming
/// Gimbal-proof version that rotates the forward vector then projects to XZ plane
pub fn horizontal_forward(transform: &Transform) -> Vec3 {
    // Rotate (0,0,-1) by the quaternion then zero-out the Y component
    let mut f = transform.rotation * Vec3::NEG_Z;
    f.y = 0.0;
    f.normalize_or_zero()
}
