use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

const MAX_WAVES: usize = 4;

/// Custom material for water surfaces with Gerstner wave displacement
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    // Colors
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(0)]
    pub shallow_color: Vec3,
    #[uniform(0)]
    pub deep_color: Vec3,
    #[uniform(0)]
    pub foam_color: Vec3,

    // Material properties
    #[uniform(0)]
    pub roughness: f32,
    #[uniform(0)]
    pub fresnel_bias: f32,
    #[uniform(0)]
    pub fresnel_power: f32,

    // Time and wave count
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub wave_count: u32,
    #[uniform(0)]
    pub _pad: Vec2,

    // Wave parameters (dir.x, dir.y, amplitude, wavelength)
    #[uniform(0)]
    pub wave_data0: [Vec4; MAX_WAVES],

    // Wave parameters (speed, steepness, _pad, _pad)
    #[uniform(0)]
    pub wave_data1: [Vec4; MAX_WAVES],
}

impl Material for WaterMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/water_professional.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/water_professional.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            // Deep ocean blue with transparency
            base_color: LinearRgba::new(0.06, 0.20, 0.35, 0.85),
            shallow_color: Vec3::new(0.10, 0.60, 0.70),
            deep_color: Vec3::new(0.02, 0.08, 0.18),
            foam_color: Vec3::new(0.95, 0.97, 0.98),

            roughness: 0.05,
            fresnel_bias: 0.02,
            fresnel_power: 5.0,

            time: 0.0,
            wave_count: 4,
            _pad: Vec2::ZERO,

            // 4 Gerstner wave octaves with realistic calm ocean parameters
            // Reduced amplitudes to prevent surface gaps
            // Format: (dir.x, dir.y, amplitude, wavelength)
            // NOTE: Directions are normalized in shader
            wave_data0: [
                Vec4::new(1.0, 0.2, 0.25, 60.0),  // Large waves
                Vec4::new(-0.6, 1.0, 0.15, 30.0), // Medium waves
                Vec4::new(0.2, -1.0, 0.08, 15.0), // Small waves
                Vec4::new(-1.0, -0.3, 0.04, 8.0), // Ripples
            ],

            // Format: (speed_override, steepness, _pad, _pad)
            // Reduced steepness for smoother, more continuous waves
            // speed=0.0 means use deep-water dispersion: w = sqrt(g*k)
            wave_data1: [
                Vec4::new(0.0, 0.4, 0.0, 0.0),  // Gentle slopes (was 0.8)
                Vec4::new(0.0, 0.35, 0.0, 0.0), // (was 0.7)
                Vec4::new(0.0, 0.3, 0.0, 0.0),  // (was 0.6)
                Vec4::new(0.0, 0.25, 0.0, 0.0), // (was 0.5)
            ],
        }
    }
}
