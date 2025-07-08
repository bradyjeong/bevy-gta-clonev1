use bevy::prelude::*;
use game_core::prelude::*;

/// Validates position is within safe bounds
#[must_use] pub fn validate_position(position: Vec3) -> Vec3 {
    let max_coord = 2000.0;
    let min_coord = -2000.0;
    
    Vec3::new(
        position.x.clamp(min_coord, max_coord),
        position.y.clamp(0.0, 500.0),
        position.z.clamp(min_coord, max_coord),
    )
}

/// Validates collider size is within safe bounds
#[must_use] pub fn validate_collider_size(size: Vec3) -> Vec3 {
    let max_size = 50.0;
    let min_size = 0.1;
    
    Vec3::new(
        size.x.clamp(min_size, max_size),
        size.y.clamp(min_size, max_size),
        size.z.clamp(min_size, max_size),
    )
}

/// Validates mass is within safe bounds
#[must_use] pub fn validate_mass(mass: f32) -> f32 {
    mass.clamp(0.1, 10000.0)
}

/// Checks if position is in water area
#[must_use] pub fn is_in_water_area(position: Vec3) -> bool {
    // Simple water detection - can be enhanced with actual water body data
    position.y < 1.0
}

/// Checks if position is on road spline
#[must_use] pub fn is_on_road_spline(position: Vec3, _road_network: &RoadNetwork, tolerance: f32) -> bool {
    // Simple road detection - can be enhanced with actual road spline data
    // For now, use a grid pattern approximation
    let grid_size = 50.0;
    let grid_x = (position.x / grid_size).round() * grid_size;
    let grid_z = (position.z / grid_size).round() * grid_size;
    let nearest_road_point = Vec3::new(grid_x, position.y, grid_z);
    nearest_road_point.distance(position) < tolerance
}
