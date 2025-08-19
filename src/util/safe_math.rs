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
    a + (b - a) * t
}

/// Scalar lerp with overshoot protection
#[inline]
pub fn safe_lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
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
    fn clamp_length_safe(self, max_length: f32) -> Vec3;
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
    
    /// Clamp length with NaN protection
    fn clamp_length_safe(self, max_length: f32) -> Vec3 {
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

/// Validate and fix a transform to safe values
pub fn sanitize_transform(transform: &mut Transform) -> bool {
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

/// Validate and fix velocity to safe values
pub fn sanitize_velocity(velocity: &mut bevy_rapier3d::prelude::Velocity) -> bool {
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
