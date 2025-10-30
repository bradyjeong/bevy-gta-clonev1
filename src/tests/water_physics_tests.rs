#[cfg(test)]
mod tests {
    use crate::components::unified_water::{TideConfig, UnifiedWaterBody};
    use bevy::prelude::*;

    #[test]
    fn test_tide_offset_normal() {
        let tide = TideConfig {
            amplitude: 1.0,
            period_sec: 10.0,
        };
        let offset = tide.offset(0.0);
        assert!(offset.is_finite());
        assert!(offset.abs() <= 1.0);
    }

    #[test]
    fn test_tide_offset_zero_period() {
        let tide = TideConfig {
            amplitude: 1.0,
            period_sec: 0.0,
        };
        let offset = tide.offset(5.0);
        assert_eq!(offset, 0.0); // Should return 0 for invalid period
    }

    #[test]
    fn test_tide_offset_nan_amplitude() {
        let tide = TideConfig {
            amplitude: f32::NAN,
            period_sec: 10.0,
        };
        let offset = tide.offset(5.0);
        assert_eq!(offset, 0.0);
    }

    #[test]
    fn test_tide_offset_negative_amplitude() {
        let tide = TideConfig {
            amplitude: -1.0,
            period_sec: 10.0,
        };
        let offset = tide.offset(5.0);
        assert_eq!(offset, 0.0); // Invalid amplitude should return 0
    }

    #[test]
    fn test_submersion_ratio_zero_height() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 0.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };
        let transform = Transform::from_xyz(0.0, 0.0, 0.0);
        let half_extents = Vec3::ZERO; // Zero height entity

        let ratio = water.calculate_submersion_ratio(&transform, half_extents, 0.0);
        assert!(ratio.is_finite());
        assert!(ratio >= 0.0 && ratio <= 1.0);
    }

    #[test]
    fn test_submersion_ratio_fully_submerged() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 10.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };
        let transform = Transform::from_xyz(0.0, 5.0, 0.0);
        let half_extents = Vec3::new(1.0, 1.0, 1.0);

        let ratio = water.calculate_submersion_ratio(&transform, half_extents, 0.0);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_submersion_ratio_not_submerged() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 0.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };
        let transform = Transform::from_xyz(0.0, 10.0, 0.0);
        let half_extents = Vec3::new(1.0, 1.0, 1.0);

        let ratio = water.calculate_submersion_ratio(&transform, half_extents, 0.0);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_submersion_ratio_partially_submerged() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 5.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };
        let transform = Transform::from_xyz(0.0, 4.5, 0.0); // Center at 4.5, water at 5.0
        let half_extents = Vec3::new(1.0, 2.0, 1.0); // Bottom at 2.5, top at 6.5

        let ratio = water.calculate_submersion_ratio(&transform, half_extents, 0.0);
        assert!(ratio > 0.0 && ratio < 1.0);
        assert!(ratio.is_finite());
    }

    #[test]
    fn test_contains_point_inside() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 0.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };

        assert!(water.contains_point(0.0, 0.0));
        assert!(water.contains_point(-9.0, -9.0));
        assert!(water.contains_point(9.0, 9.0));
    }

    #[test]
    fn test_contains_point_outside() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 0.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };

        assert!(!water.contains_point(15.0, 0.0));
        assert!(!water.contains_point(0.0, -15.0));
        assert!(!water.contains_point(-15.0, 15.0));
    }

    #[test]
    fn test_contains_point_invalid_bounds() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (10.0, 10.0, -10.0, -10.0), // Invalid: min > max
            surface_level: 0.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };

        assert!(!water.contains_point(0.0, 0.0)); // Should return false for invalid bounds
    }

    #[test]
    fn test_get_water_surface_level_with_tide() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 5.0,
            depth: 2.0,
            tide: TideConfig {
                amplitude: 2.0,
                period_sec: 10.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };

        let level_at_0 = water.get_water_surface_level(0.0);
        assert!(level_at_0.is_finite());
        assert!(level_at_0 >= 3.0 && level_at_0 <= 7.0); // 5.0 ± 2.0
    }

    #[test]
    fn test_get_bed_level() {
        let water = UnifiedWaterBody {
            name: "Test".to_string(),
            bounds: (-10.0, -10.0, 10.0, 10.0),
            surface_level: 5.0,
            depth: 3.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 1.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        };

        assert_eq!(water.get_bed_level(), 2.0); // 5.0 - 3.0
    }
}
