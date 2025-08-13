use bevy::prelude::*;

/// Unified Mesh Factory - Eliminates 130+ duplicate mesh creation patterns
/// Critical safeguards: Input validation, performance optimization, consistent naming
pub struct MeshFactory;

impl MeshFactory {
    // VEHICLE MESHES - Standard vehicle components
    pub fn create_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 0.6, 3.6))
    }

    pub fn create_sports_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 0.4, 4.2))
    }

    pub fn create_suv_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(2.5, 1.5, 5.0))
    }

    pub fn create_truck_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(16.0, 2.0, 3.0))
    }

    pub fn create_helicopter_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(4.0, 0.3, 8.0))
    }

    pub fn create_boat_hull(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(8.0, 2.0, 20.0))
    }

    pub fn create_yacht_cabin(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(6.0, 3.0, 8.0))
    }

    // VEHICLE PARTS - Wheels, components
    pub fn create_standard_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.35, 0.25))
    }

    pub fn create_sports_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.25, 0.3))
    }

    pub fn create_large_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.4, 0.3))
    }

    pub fn create_wheel_hub(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 0.35))
    }

    pub fn create_exhaust_pipe(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.08, 0.3))
    }

    pub fn create_headlight(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.2))
    }

    pub fn create_small_light(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.15))
    }

    pub fn create_tiny_light(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.12))
    }

    // HELICOPTER PARTS
    pub fn create_rotor_blade(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(10.0, 0.05, 0.2))
    }

    pub fn create_tail_rotor(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.15, 0.2))
    }

    pub fn create_landing_gear(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(0.6, 0.6, 4.0))
    }

    // WORLD STRUCTURES - Buildings, environment
    pub fn create_building_base(meshes: &mut ResMut<Assets<Mesh>>, width: f32, height: f32, depth: f32) -> Handle<Mesh> {
        // Input validation for critical safeguards
        let safe_width = width.max(0.1).min(1000.0);
        let safe_height = height.max(0.1).min(1000.0);
        let safe_depth = depth.max(0.1).min(1000.0);
        meshes.add(Cuboid::new(safe_width, safe_height, safe_depth))
    }

    pub fn create_lamp_post(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 8.0))
    }

    pub fn create_tree_frond(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.8))
    }

    pub fn create_tree_trunk(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 8.0))
    }

    pub fn create_road_segment(meshes: &mut ResMut<Assets<Mesh>>, width: f32, length: f32) -> Handle<Mesh> {
        let safe_width = width.max(0.1).min(100.0);
        let safe_length = length.max(0.1).min(1000.0);
        meshes.add(Cuboid::new(safe_width, 0.1, safe_length))
    }

    pub fn create_road_marking(meshes: &mut ResMut<Assets<Mesh>>, width: f32, length: f32) -> Handle<Mesh> {
        let safe_width = width.max(0.1).min(10.0);
        let safe_length = length.max(0.1).min(100.0);
        meshes.add(Cuboid::new(safe_width, 0.11, safe_length))
    }

    // WATER FEATURES
    pub fn create_lake_cylinder(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, depth: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(1.0).min(1000.0);
        let safe_depth = depth.max(0.1).min(100.0);
        meshes.add(Cylinder::new(safe_radius, safe_depth))
    }

    pub fn create_water_plane(meshes: &mut ResMut<Assets<Mesh>>, size: f32) -> Handle<Mesh> {
        let safe_size = size.max(1.0).min(10000.0);
        meshes.add(Plane3d::default().mesh().size(safe_size, safe_size))
    }

    pub fn create_mast(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.2, 15.0))
    }

    // NPC COMPONENTS - Character parts  
    pub fn create_npc_head(meshes: &mut ResMut<Assets<Mesh>>, build_factor: f32) -> Handle<Mesh> {
        let safe_build = build_factor.max(0.1).min(5.0);
        meshes.add(Sphere::new(0.12 * safe_build))
    }

    pub fn create_npc_body(meshes: &mut ResMut<Assets<Mesh>>, build: f32, height: f32) -> Handle<Mesh> {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        meshes.add(Cuboid::new(0.4 * safe_build, 0.6 * safe_height, 0.2 * safe_build))
    }

    pub fn create_npc_limb(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, length: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.01).min(1.0);
        let safe_length = length.max(0.1).min(5.0);
        meshes.add(Capsule3d::new(safe_radius, safe_length))
    }

    pub fn create_npc_simple_body(meshes: &mut ResMut<Assets<Mesh>>, build: f32, height: f32) -> Handle<Mesh> {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        meshes.add(Capsule3d::new(0.3 * safe_build, safe_height * 0.8))
    }

    pub fn create_npc_ultra_simple(meshes: &mut ResMut<Assets<Mesh>>, build: f32, height: f32) -> Handle<Mesh> {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        meshes.add(Capsule3d::new(0.25 * safe_build, safe_height))
    }

    // SKY COMPONENTS - Celestial bodies
    pub fn create_sky_dome(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(2000.0))
    }

    pub fn create_sun(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(50.0))
    }

    pub fn create_moon(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(30.0))
    }

    pub fn create_star(meshes: &mut ResMut<Assets<Mesh>>, size: f32) -> Handle<Mesh> {
        let safe_size = size.max(0.1).min(100.0);
        meshes.add(Sphere::new(safe_size))
    }

    pub fn create_cloud(meshes: &mut ResMut<Assets<Mesh>>, scale: f32) -> Handle<Mesh> {
        let safe_scale = scale.max(1.0).min(1000.0);
        meshes.add(Sphere::new(safe_scale))
    }

    // WEATHER EFFECTS
    pub fn create_rain_drop(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Capsule3d::new(0.1, 2.0))
    }

    pub fn create_fog_particle(meshes: &mut ResMut<Assets<Mesh>>, base_size: f32, variation: f32) -> Handle<Mesh> {
        let safe_base = base_size.max(0.1).min(100.0);
        let safe_variation = variation.max(0.0).min(50.0);
        let size = safe_base + rand::random::<f32>() * safe_variation;
        meshes.add(Sphere::new(size))
    }

    pub fn create_dust_particle(meshes: &mut ResMut<Assets<Mesh>>, base_size: f32, variation: f32) -> Handle<Mesh> {
        let safe_base = base_size.max(0.1).min(10.0);
        let safe_variation = variation.max(0.0).min(5.0);
        let size = safe_base + rand::random::<f32>() * safe_variation;
        meshes.add(Sphere::new(size))
    }

    // TERRAIN - Ground plane
    pub fn create_ground_plane(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Plane3d::default().mesh().size(4000.0, 4000.0))
    }

    // CUSTOM SIZED MESHES - Flexible components
    pub fn create_custom_cuboid(meshes: &mut ResMut<Assets<Mesh>>, width: f32, height: f32, depth: f32) -> Handle<Mesh> {
        let safe_width = width.max(0.001).min(10000.0);
        let safe_height = height.max(0.001).min(10000.0);
        let safe_depth = depth.max(0.001).min(10000.0);
        meshes.add(Cuboid::new(safe_width, safe_height, safe_depth))
    }

    pub fn create_custom_sphere(meshes: &mut ResMut<Assets<Mesh>>, radius: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.001).min(5000.0);
        meshes.add(Sphere::new(safe_radius))
    }

    pub fn create_custom_cylinder(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, height: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.001).min(1000.0);
        let safe_height = height.max(0.001).min(10000.0);
        meshes.add(Cylinder::new(safe_radius, safe_height))
    }

    pub fn create_custom_capsule(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, length: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.001).min(100.0);
        let safe_length = length.max(0.001).min(1000.0);
        meshes.add(Capsule3d::new(safe_radius, safe_length))
    }
    
    /// Create F16 fighter jet body
    pub fn create_f16_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(16.0, 2.0, 3.0))
    }
    
    /// Create F16 wing
    pub fn create_f16_wing(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(8.0, 0.2, 2.0))
    }
}
