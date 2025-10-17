use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;

#[derive(Component, ExtractComponent, Clone, ShaderType)]
pub struct UnderwaterSettings {
    pub sea_level: f32,
    pub fog_density: f32,
    #[align(16)]
    pub absorption: Vec3,
    pub scatter_color: Vec3,
    pub enabled: u32,
}

impl Default for UnderwaterSettings {
    fn default() -> Self {
        Self {
            sea_level: 0.0,
            fog_density: 0.22,
            absorption: Vec3::new(0.20, 0.08, 0.03),
            scatter_color: Vec3::new(0.10, 0.35, 0.40),
            enabled: 1,
        }
    }
}
