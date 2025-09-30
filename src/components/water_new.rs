use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Water region component - represents a body of water with physics and rendering properties
#[derive(Component, Debug, Clone)]
pub struct WaterRegion {
    pub name: String,
    pub bounds: (f32, f32, f32, f32), // (min_x, min_z, max_x, max_z)
    pub base_level: f32,
    pub current_level: f32, // Updated by tide system
    pub tide: TideConfig,
    pub wave_params: Option<WaveParams>,
    pub surface_color: (f32, f32, f32, f32),
}

/// Tide configuration for water level oscillation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TideConfig {
    pub amplitude: f32,
    pub period_sec: f32,
}

/// Wave parameters for visual surface displacement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveParams {
    pub amplitude: f32,
    pub frequency: f32,
    pub speed: f32,
}

/// Water region asset for loading from RON files
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct WaterRegionAsset {
    pub name: String,
    pub bounds: (f32, f32, f32, f32),
    pub base_level: f32,
    pub tide: TideConfig,
    pub wave_params: Option<WaveParams>,
    pub surface_color: (f32, f32, f32, f32),
}

/// Marker component for entities that should experience water physics
#[derive(Component, Default)]
pub struct WaterBodyId;

// GlobalOcean removed - using only specific water bodies now

impl Default for WaterRegion {
    fn default() -> Self {
        Self {
            name: "Water".to_string(),
            bounds: (-100.0, -100.0, 100.0, 100.0),
            base_level: 0.0,
            current_level: 0.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 300.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        }
    }
}

impl WaterRegion {
    /// Check if a 2D position is within this water region
    pub fn contains_point(&self, x: f32, z: f32) -> bool {
        x >= self.bounds.0 && x <= self.bounds.2 && z >= self.bounds.1 && z <= self.bounds.3
    }

    /// Get water level at current time (including tide)
    pub fn get_water_level(&self, time: f32) -> f32 {
        let tide_offset = if self.tide.amplitude > 0.0 {
            (time * 2.0 * std::f32::consts::PI / self.tide.period_sec).sin() * self.tide.amplitude
        } else {
            0.0
        };
        self.base_level + tide_offset
    }

    /// Calculate submerged ratio for a box collider
    pub fn calculate_submersion_ratio(
        &self,
        transform: &Transform,
        half_extents: Vec3,
        time: f32,
    ) -> f32 {
        let water_level = self.get_water_level(time);
        let entity_bottom = transform.translation.y - half_extents.y;
        let entity_top = transform.translation.y + half_extents.y;

        if entity_bottom > water_level {
            0.0 // Completely above water
        } else if entity_top < water_level {
            1.0 // Completely submerged
        } else {
            // Partially submerged
            (water_level - entity_bottom) / (entity_top - entity_bottom)
        }
    }
}
