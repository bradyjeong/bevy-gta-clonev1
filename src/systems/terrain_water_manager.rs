use bevy::prelude::*;
use crate::components::unified_water::UnifiedWaterBody;

pub enum SurfaceType {
    Ground(f32),
    Water { surface: f32, depth: f32 },
    Air(f32),
}

pub fn get_surface_type_at(
    pos: Vec3,
    water_bodies: &Query<&UnifiedWaterBody>,
    time: f32,
) -> SurfaceType {
    // Check if position is in any water body
    for water in water_bodies.iter() {
        if water.contains_point(pos.x, pos.z) {
            let surface = water.get_water_surface_level(time);
            return SurfaceType::Water {
                surface,
                depth: water.depth,
            };
        }
    }
    
    // Default to ground level (you may want to integrate with actual terrain)
    SurfaceType::Ground(0.0)
}

pub fn get_safe_spawn_height(pos: Vec2, water_bodies: &Query<&UnifiedWaterBody>, time: f32) -> f32 {
    match get_surface_type_at(pos.extend(0.0), water_bodies, time) {
        SurfaceType::Ground(height) => height + 0.35, // Player height above ground
        SurfaceType::Water { surface, .. } => surface + 0.1, // Slightly above water
        SurfaceType::Air(height) => height,
    }
}
