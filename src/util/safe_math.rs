//! Safe Math Utilities
//! 
//! Prevents coordinate explosions, NaN propagation, and physics panics by providing
//! mathematically safe alternatives to standard operations.
//! 
//! Following AGENT.md "Simplicity First" - isolated utility functions that prevent
//! corruption at the source rather than complex error handling.

use bevy::prelude::*;

/// Divide with zero protection
#[inline]
pub fn safe_div(a: f32, b: f32) -> f32 {
    if b.abs() < 1e-6 { 
        0.0 
    } else { 
        a / b 
    }
}

/// Linear interpolation with overshoot protection
#[inline]
pub fn safe_lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    let t = t.clamp(0.0, 1.0);
    if !t.is_finite() {
        return a; // Default to start value on invalid t
    }
    if !a.is_finite() || !b.is_finite() {
        return Vec3::ZERO; // Safe fallback for invalid inputs
    }
    a + (b - a) * t
}

/// Scalar lerp with overshoot protection
#[inline]
pub fn safe_lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

/// Safe spherical linear interpolation for quaternions that clamps t to [0, 1]
#[inline]
pub fn safe_slerp(a: Quat, b: Quat, t: f32) -> Quat {
    let t = t.clamp(0.0, 1.0);
    if !t.is_finite() {
        return a; // Default to start rotation on invalid t
    }
    
    // Validate input quaternions
    if !a.is_finite() || !b.is_finite() {
        warn!("Invalid quaternion in safe_slerp, using identity");
        return Quat::IDENTITY;
    }
    
    // Use Bevy's slerp with validated inputs
    a.slerp(b, t)
}

/// Check if Vec3 contains valid, reasonable coordinates
#[inline]
pub fn is_valid_position(v: Vec3) -> bool {
    v.is_finite() && v.length() < 1.0e6 // 1000km reasonable limit
}

/// Check if velocity is safe for physics
#[inline] 
pub fn is_valid_velocity(v: Vec3) -> bool {
    v.is_finite() && v.length() < 1000.0 // 1000 m/s reasonable limit
}

/// Extension trait for Vec3 with safe operations
pub trait Vec3SafeExt {
    fn safe_normalize(self) -> Vec3;
    fn clamp_length(self, max_length: f32) -> Vec3;
}

impl Vec3SafeExt for Vec3 {
    /// Normalize with zero-length protection
    fn safe_normalize(self) -> Vec3 {
        if self.length_squared() < 1e-6 { 
            Vec3::ZERO 
        } else { 
            self.normalize() 
        }
    }
    
    /// Clamp length with NaN protection - single implementation
    fn clamp_length(self, max_length: f32) -> Vec3 {
        if !self.is_finite() {
            return Vec3::ZERO;
        }
        
        let length = self.length();
        if length > max_length {
            self * (max_length / length)
        } else {
            self
        }
    }
}

/// Constants for coordinate safety
pub const MAX_SAFE_COORDINATE: f32 = 100_000.0;    // 100km
pub const MAX_SAFE_VELOCITY: f32 = 600.0;          // 600 m/s  
pub const MAX_SAFE_ANGULAR_VEL: f32 = 20.0;        // 20 rad/s

/// Validate and fix a transform to safe values - single authority for transform safety  
pub fn validate_transform(transform: &mut Transform) -> bool {
    let mut was_corrupt = false;
    
    // Fix translation
    if !is_valid_position(transform.translation) {
        warn!("Sanitizing invalid transform translation: {:?}", transform.translation);
        transform.translation = Vec3::ZERO;
        was_corrupt = true;
    }
    
    // Fix rotation
    if !transform.rotation.is_normalized() || !transform.rotation.is_finite() {
        warn!("Sanitizing invalid transform rotation: {:?}", transform.rotation);
        transform.rotation = Quat::IDENTITY;
        was_corrupt = true;
    }
    
    // Fix scale
    if !transform.scale.is_finite() || transform.scale.cmpeq(Vec3::ZERO).any() {
        warn!("Sanitizing invalid transform scale: {:?}", transform.scale);
        transform.scale = Vec3::ONE;
        was_corrupt = true;
    }
    
    was_corrupt
}

/// Validate and fix velocity to safe values - single authority for velocity safety
pub fn validate_velocity(velocity: &mut bevy_rapier3d::prelude::Velocity) -> bool {
    let mut was_corrupt = false;
    
    if !is_valid_velocity(velocity.linvel) {
        warn!("Sanitizing invalid linear velocity: {:?}", velocity.linvel);
        velocity.linvel = Vec3::ZERO;
        was_corrupt = true;
    }
    
    if !velocity.angvel.is_finite() || velocity.angvel.length() > MAX_SAFE_ANGULAR_VEL {
        warn!("Sanitizing invalid angular velocity: {:?}", velocity.angvel);
        velocity.angvel = Vec3::ZERO;
        was_corrupt = true;
    }
    
    was_corrupt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_lerp_clamping() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 10.0, 10.0);
        
        // Normal case
        assert_eq!(safe_lerp(a, b, 0.5), Vec3::new(5.0, 5.0, 5.0));
        
        // Overshoot prevention
        assert_eq!(safe_lerp(a, b, 2.0), b); // t=2.0 should clamp to 1.0
        assert_eq!(safe_lerp(a, b, -1.0), a); // t=-1.0 should clamp to 0.0
        
        // NaN/infinite protection
        assert_eq!(safe_lerp(a, b, f32::NAN), a);
        assert_eq!(safe_lerp(a, b, f32::INFINITY), b);
        
        // Invalid input protection
        assert_eq!(safe_lerp(Vec3::NAN, b, 0.5), Vec3::ZERO);
        assert_eq!(safe_lerp(a, Vec3::NAN, 0.5), Vec3::ZERO);
    }

    #[test]
    fn test_safe_slerp_clamping() {
        let a = Quat::IDENTITY;
        let b = Quat::from_rotation_y(std::f32::consts::PI);
        
        // Normal case
        let result = safe_slerp(a, b, 0.5);
        assert!(result.is_finite());
        assert!(result.is_normalized());
        
        // Overshoot prevention (approximately equal due to floating point)
        let result_high = safe_slerp(a, b, 2.0);
        let result_low = safe_slerp(a, b, -1.0);
        assert!((result_high.dot(b)).abs() > 0.99); // Close to b
        assert!((result_low.dot(a)).abs() > 0.99); // Close to a
        
        // Invalid input protection
        assert_eq!(safe_slerp(Quat::NAN, b, 0.5), Quat::IDENTITY);
        assert_eq!(safe_slerp(a, Quat::NAN, 0.5), Quat::IDENTITY);
    }

    #[test] 
    fn test_coordinate_safety_limits() {
        // Valid positions
        assert!(is_valid_position(Vec3::new(1000.0, 100.0, 1000.0)));
        assert!(is_valid_position(Vec3::ZERO));
        
        // Invalid positions
        assert!(!is_valid_position(Vec3::new(f32::NAN, 0.0, 0.0)));
        assert!(!is_valid_position(Vec3::new(2_000_000.0, 0.0, 0.0))); // Too far
        
        // Valid velocities
        assert!(is_valid_velocity(Vec3::new(100.0, 50.0, 100.0)));
        assert!(is_valid_velocity(Vec3::ZERO));
        
        // Invalid velocities  
        assert!(!is_valid_velocity(Vec3::new(f32::INFINITY, 0.0, 0.0)));
        assert!(!is_valid_velocity(Vec3::new(2000.0, 0.0, 0.0))); // Too fast
    }

    #[test]
    fn test_validate_transform() {
        let mut transform = Transform {
            translation: Vec3::new(f32::NAN, 0.0, 0.0),
            rotation: Quat::NAN,
            scale: Vec3::new(0.0, 1.0, 1.0), // Invalid zero scale
        };
        
        assert!(validate_transform(&mut transform));
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }
}
