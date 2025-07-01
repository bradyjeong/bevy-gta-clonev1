use bevy::prelude::*;

/// Unified Mesh Factory - Eliminates 130+ duplicate mesh creation patterns
/// Critical safeguards: Input validation, performance optimization, consistent naming
pub struct MeshFactory;

impl MeshFactory {
    // VEHICLE MESHES - Standard vehicle components (Fixed: heights match colliders)
    pub fn create_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 3.6))  // Fixed: height 1.0 matches collider
    }

    pub fn create_sports_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 4.2))  // Fixed: height 1.0 matches collider
    }

    pub fn create_suv_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(2.5, 1.5, 5.0))
    }

    pub fn create_truck_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(16.0, 2.0, 3.0))
    }

    pub fn create_helicopter_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(3.0, 2.0, 6.0))  // Fixed: matches collider dimensions
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
    
    /// Create F16 fighter jet body (main fuselage) - Fixed: matches collider dimensions
    pub fn create_f16_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Fixed: dimensions match collider cuboid(8.0, 1.5, 1.5) -> total size (16.0, 3.0, 3.0)
        // Reoriented: length along X-axis (nose at -X, tail at +X)
        let width = 16.0_f32.clamp(0.1, 50.0);  // X-axis (length)
        let height = 3.0_f32.clamp(0.1, 10.0); // Y-axis (height)  
        let depth = 3.0_f32.clamp(0.1, 10.0); // Z-axis (width)
        meshes.add(Cuboid::new(width, height, depth))
    }
    
    /// Create F16 wing (swept delta wing)
    pub fn create_f16_wing(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // F16 wing dimensions: 32.8ft span, average chord ~6ft
        // Scaled: ~10 units span, 1.8 units chord, thin profile
        let span = 10.0_f32.clamp(0.1, 30.0);
        let chord = 1.8_f32.clamp(0.1, 10.0);
        let thickness = 0.15_f32.clamp(0.01, 1.0);
        meshes.add(Cuboid::new(span, thickness, chord))
    }
    
    /// Create F16 air intake (side-mounted)
    pub fn create_f16_air_intake(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // F16 has distinctive side air intakes
        let width = 2.0_f32.clamp(0.1, 5.0);
        let height = 1.2_f32.clamp(0.1, 3.0);
        let depth = 1.0_f32.clamp(0.1, 3.0);
        meshes.add(Cuboid::new(width, height, depth))
    }
    
    /// Create F16 canopy (bubble canopy)
    pub fn create_f16_canopy(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // F16's distinctive bubble canopy
        let radius = 1.0_f32.clamp(0.1, 3.0);
        let height = 1.2_f32.clamp(0.1, 3.0);
        meshes.add(Capsule3d::new(radius, height))
    }
    
    /// Create F16 vertical tail
    pub fn create_f16_vertical_tail(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Large vertical stabilizer characteristic of F16
        let width = 0.3_f32.clamp(0.1, 2.0);
        let height = 3.5_f32.clamp(0.1, 10.0);
        let chord = 2.5_f32.clamp(0.1, 8.0);
        meshes.add(Cuboid::new(width, height, chord))
    }
    
    /// Create F16 horizontal stabilizer
    pub fn create_f16_horizontal_stabilizer(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Horizontal tail surfaces
        let span = 4.0_f32.clamp(0.1, 15.0);
        let thickness = 0.1_f32.clamp(0.01, 1.0);
        let chord = 1.5_f32.clamp(0.1, 5.0);
        meshes.add(Cuboid::new(span, thickness, chord))
    }
    
    /// Create F16 engine nozzle
    pub fn create_f16_engine_nozzle(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Afterburning turbofan nozzle
        let radius = 0.8_f32.clamp(0.1, 3.0);
        let length = 1.5_f32.clamp(0.1, 5.0);
        meshes.add(Cylinder::new(radius, length))
    }
}
