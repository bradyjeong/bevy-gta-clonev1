use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone)]
pub struct UnifiedWaterBody {
    pub name: String,
    pub bounds: (f32, f32, f32, f32), // (min_x, min_z, max_x, max_z)
    pub surface_level: f32,           // Single source of truth for water height
    pub depth: f32,                   // How deep the water goes
    pub tide: TideConfig,
    pub wave_params: Option<WaveParams>,
    pub surface_color: (f32, f32, f32, f32),
}

impl UnifiedWaterBody {
    /// Get visual water surface level (includes tides and waves for rendering)
    pub fn get_water_surface_level(&self, time: f32) -> f32 {
        self.surface_level + self.tide.offset(time)
    }

    /// Get base water level for gameplay logic (no waves, only static level + tide)
    /// Professional games separate visual waves from physics/gameplay logic
    pub fn get_base_water_level(&self, time: f32) -> f32 {
        self.surface_level + self.tide.offset(time)
    }

    pub fn get_bed_level(&self) -> f32 {
        self.surface_level - self.depth
    }

    pub fn contains_point(&self, x: f32, z: f32) -> bool {
        let (min_x, min_z, max_x, max_z) = self.bounds;
        // Bounds validated at asset load time, no need for runtime error logging
        x >= min_x && x <= max_x && z >= min_z && z <= max_z
    }

    /// Calculate submersion ratio for gameplay (uses base water level, not visual waves)
    /// This prevents swimming animation from triggering in wave troughs
    pub fn calculate_submersion_ratio(
        &self,
        transform: &Transform,
        half_extents: Vec3,
        time: f32,
    ) -> f32 {
        // Use base water level for gameplay logic - professional approach
        let water_level = self.get_base_water_level(time);
        let entity_center = transform.translation.y;
        let entity_bottom = entity_center - half_extents.y;
        let entity_top = entity_center + half_extents.y;

        // CRITICAL FIX: If entity bottom is above water, not swimming (prevents beach swimming bug)
        if entity_bottom > water_level {
            0.0 // Completely above water - not swimming
        } else if entity_top < water_level {
            1.0 // Completely submerged
        } else {
            let height = entity_top - entity_bottom;
            // CRITICAL FIX: Prevent division by zero
            if height <= 0.0001 {
                return if entity_center < water_level {
                    1.0
                } else {
                    0.0
                };
            }
            let ratio = (water_level - entity_bottom) / height;
            // Validate result is finite
            if ratio.is_finite() {
                ratio.clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
    }
}

#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedWaterAsset {
    pub name: String,
    pub bounds: (f32, f32, f32, f32),
    pub surface_level: f32, // NEW: explicit surface height
    pub depth: f32,         // NEW: water depth
    pub tide: TideConfig,
    pub wave_params: Option<WaveParams>,
    pub surface_color: (f32, f32, f32, f32),
}

impl UnifiedWaterAsset {
    pub fn validate(&self) -> Result<(), String> {
        // Bounds validation (min < max)
        let (min_x, min_z, max_x, max_z) = self.bounds;
        if min_x >= max_x || min_z >= max_z {
            return Err(format!(
                "Invalid bounds: ({min_x}, {min_z}, {max_x}, {max_z}) - min must be < max"
            ));
        }

        // Depth must be positive
        if !self.depth.is_finite() || self.depth <= 0.0 {
            return Err(format!(
                "Invalid depth: {} (must be finite and > 0)",
                self.depth
            ));
        }

        // Surface level must be finite
        if !self.surface_level.is_finite() {
            return Err(format!(
                "Invalid surface_level: {} (must be finite)",
                self.surface_level
            ));
        }

        // Tide validation
        if !self.tide.amplitude.is_finite() || self.tide.amplitude < 0.0 {
            return Err(format!(
                "Invalid tide amplitude: {} (must be finite and >= 0)",
                self.tide.amplitude
            ));
        }
        if !self.tide.period_sec.is_finite() || self.tide.period_sec < 1e-3 {
            return Err(format!(
                "Invalid tide period: {} (must be finite and >= 0.001)",
                self.tide.period_sec
            ));
        }

        // Color validation - check if convertible to linear and finite
        let (r, g, b, a) = self.surface_color;
        if !r.is_finite() || !g.is_finite() || !b.is_finite() || !a.is_finite() {
            return Err("Invalid color values (must be finite)".to_string());
        }
        if !(0.0..=1.0).contains(&r)
            || !(0.0..=1.0).contains(&g)
            || !(0.0..=1.0).contains(&b)
            || !(0.0..=1.0).contains(&a)
        {
            return Err(format!(
                "Invalid color values ({r}, {g}, {b}, {a}) - must be in [0, 1]"
            ));
        }

        Ok(())
    }
}

/// Tide configuration for water level oscillation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TideConfig {
    pub amplitude: f32,
    pub period_sec: f32,
}

impl TideConfig {
    pub fn offset(&self, time: f32) -> f32 {
        if !self.amplitude.is_finite() || self.amplitude <= 0.0 {
            return 0.0;
        }
        let period = if self.period_sec.is_finite() && self.period_sec.abs() > 1e-3 {
            self.period_sec
        } else {
            // Tide disabled for invalid period
            return 0.0;
        };
        (time * 2.0 * std::f32::consts::PI / period).sin() * self.amplitude
    }
}

/// Wave parameters for visual surface displacement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveParams {
    pub amplitude: f32,
    pub frequency: f32,
    pub speed: f32,
}

/// Marker component for entities that should experience water physics
#[derive(Component, Default)]
pub struct WaterBodyId;

/// Links a water surface mesh to its parent water region
/// Enables O(1) updates instead of O(N) name-based scanning
#[derive(Component)]
pub struct WaterSurface {
    pub region_entity: Entity,
}

/// Cached water region reference for O(1) lookup performance
/// Only updated when entity moves out of current region (rare)
#[derive(Component, Clone, Debug, Default)]
pub struct CurrentWaterRegion {
    pub region_entity: Option<Entity>,
}

// GlobalOcean removed - using only lake water bodies now

impl Default for UnifiedWaterBody {
    fn default() -> Self {
        Self {
            name: "Water".to_string(),
            bounds: (-100.0, -100.0, 100.0, 100.0),
            surface_level: 1.0, // Default to above ground level
            depth: 2.0,
            tide: TideConfig {
                amplitude: 0.0,
                period_sec: 300.0,
            },
            wave_params: None,
            surface_color: (0.1, 0.4, 0.8, 0.7),
        }
    }
}
