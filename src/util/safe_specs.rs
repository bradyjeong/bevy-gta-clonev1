//! Safe spec validation utilities to prevent NaN propagation
//!
//! Prevents physics solver panics from invalid config values

/// Safely clamps a value, returning None if value is NaN or infinite
///
/// # Returns
/// - `Some(clamped_value)` if value is finite
/// - `None` if value is NaN or infinite
pub fn safe_clamp_f32(value: f32, min: f32, max: f32) -> Option<f32> {
    if value.is_nan() || !value.is_finite() {
        None
    } else {
        Some(value.clamp(min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_clamp_normal() {
        assert_eq!(safe_clamp_f32(5.0, 0.0, 10.0), Some(5.0));
        assert_eq!(safe_clamp_f32(-1.0, 0.0, 10.0), Some(0.0));
        assert_eq!(safe_clamp_f32(15.0, 0.0, 10.0), Some(10.0));
    }

    #[test]
    fn test_safe_clamp_nan() {
        assert_eq!(safe_clamp_f32(f32::NAN, 0.0, 10.0), None);
    }

    #[test]
    fn test_safe_clamp_infinity() {
        assert_eq!(safe_clamp_f32(f32::INFINITY, 0.0, 10.0), None);
        assert_eq!(safe_clamp_f32(f32::NEG_INFINITY, 0.0, 10.0), None);
    }
}
