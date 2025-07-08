//! Mathematical utilities and constants

/// Clamp a value between min and max
#[must_use]
pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Common mathematical constants
pub mod constants {
    /// Physics constants
    pub const GRAVITY: f32 = 9.81;
    /// Air density constant (kg/m³)
    pub const AIR_DENSITY: f32 = 1.225;
}
