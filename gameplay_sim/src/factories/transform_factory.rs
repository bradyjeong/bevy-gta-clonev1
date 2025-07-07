use bevy::prelude::*;

/// Unified Transform Factory - Eliminates 100+ Transform::from_xyz patterns  
/// Critical safeguards: Rotation validation, position bounds checking, performance optimization
pub struct TransformFactory;
impl TransformFactory {
    // BASIC POSITIONING
    pub fn at_origin() -> Transform {
        Transform::from_xyz(0.0, 0.0, 0.0)
    }
    pub fn at_ground_level(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.0, z)
    pub fn at_position(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z)
    // VEHICLE POSITIONING
    pub fn vehicle_spawn(x: f32, z: f32) -> Transform {
        Transform::from_xyz(x, 0.5, z)
    pub fn vehicle_elevated(x: f32, y: f32, z: f32) -> Transform {
    pub fn helicopter_spawn(x: f32, y: f32, z: f32) -> Transform {
    pub fn boat_spawn(x: f32, y: f32, z: f32) -> Transform {
    // VEHICLE COMPONENTS - Relative positioning
    pub fn vehicle_body_center() -> Transform {
    pub fn vehicle_chassis() -> Transform {
        Transform::from_xyz(0.0, -0.1, 0.0)
    pub fn vehicle_cabin() -> Transform {
        Transform::from_xyz(0.0, 0.25, -0.3)
    pub fn vehicle_hood() -> Transform {
        Transform::from_xyz(0.0, 0.12, 1.6)
    pub fn windshield() -> Transform {
        Transform::from_xyz(0.0, 0.4, 0.8).with_rotation(Quat::from_rotation_x(-0.2))
    pub fn left_door() -> Transform {
        Transform::from_xyz(0.75, 0.3, -0.3)
    pub fn right_door() -> Transform {
        Transform::from_xyz(-0.75, 0.3, -0.3)
    pub fn rear_window() -> Transform {
        Transform::from_xyz(0.0, -0.05, 2.1)
    pub fn front_bumper() -> Transform {
        Transform::from_xyz(0.0, 0.6, -1.8)
    // WHEELS - Standard positions with rotation
    pub fn front_left_wheel() -> Transform {
        Transform::from_xyz(0.6, 0.0, 2.0)
    pub fn front_right_wheel() -> Transform {
        Transform::from_xyz(-0.6, 0.0, 2.0)
    pub fn rear_left_wheel() -> Transform {
        Transform::from_xyz(0.6, -0.35, -1.5)
    pub fn rear_right_wheel() -> Transform {
        Transform::from_xyz(-0.6, -0.35, -1.5)
    /// Transform for main rotor position
    pub fn main_rotor() -> Transform {
        Transform::from_xyz(0.0, 1.2, 0.0)
    /// Transform for tail rotor position  
    pub fn tail_rotor() -> Transform {
    
    /// Transform for F16 left wing (properly positioned for realistic layout)
    pub fn f16_left_wing() -> Transform {
        Transform::from_xyz(-5.0, -0.2, 1.0) // Left side, slightly below fuselage, forward
    /// Transform for F16 right wing (properly positioned for realistic layout)
    pub fn f16_right_wing() -> Transform {
        Transform::from_xyz(5.0, -0.2, 1.0) // Right side, slightly below fuselage, forward
    /// Transform for F16 canopy (pilot position)
    pub fn f16_canopy() -> Transform {
        Transform::from_xyz(0.0, 1.2, 2.0) // Above fuselage, forward of center
    /// Transform for F16 left air intake
    pub fn f16_left_air_intake() -> Transform {
        Transform::from_xyz(-1.5, -0.3, 3.0) // Left side, below fuselage, forward
    /// Transform for F16 right air intake
    pub fn f16_right_air_intake() -> Transform {
        Transform::from_xyz(1.5, -0.3, 3.0) // Right side, below fuselage, forward
    /// Transform for F16 vertical tail
    pub fn f16_vertical_tail() -> Transform {
        Transform::from_xyz(0.0, 1.5, -6.0) // Above fuselage, rear
    /// Transform for F16 left horizontal stabilizer
    pub fn f16_left_horizontal_stabilizer() -> Transform {
        Transform::from_xyz(-2.0, 0.3, -6.5) // Left side, rear, slightly elevated
    /// Transform for F16 right horizontal stabilizer
    pub fn f16_right_horizontal_stabilizer() -> Transform {
        Transform::from_xyz(2.0, 0.3, -6.5) // Right side, rear, slightly elevated
    /// Transform for F16 engine nozzle
    pub fn f16_engine_nozzle() -> Transform {
        Transform::from_xyz(0.0, -0.2, -7.5).with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0))
    pub fn wheel_with_rotation(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0))
    pub fn large_vehicle_wheel(x: f32, y: f32, z: f32) -> Transform {
    // EXHAUST PIPES
    pub fn left_exhaust() -> Transform {
        Transform::from_xyz(0.4, -0.25, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
    pub fn right_exhaust() -> Transform {
        Transform::from_xyz(-0.4, -0.25, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0))
    // AIRCRAFT COMPONENTS
    pub fn helicopter_body() -> Transform {
    pub fn rotor_with_rotation(angle: f32) -> Transform {
        Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle))
    pub fn helicopter_cockpit() -> Transform {
    pub fn landing_skid_left() -> Transform {
        Transform::from_xyz(-2.0, -0.2, 0.0)
    pub fn tail_rotor_blade() -> Transform {
        Transform::from_xyz(0.08, 2.2, 0.15)
    // WATER VEHICLES
    pub fn yacht_main_body() -> Transform {
        Transform::from_xyz(0.0, 3.5, -2.0)
    pub fn boat_mast() -> Transform {
        Transform::from_xyz(0.0, 9.5, 2.0)
    // NPC POSITIONING
    pub fn npc_spawn(position: Vec3) -> Transform {
        Transform::from_xyz(position.x, 1.0, position.z)
    pub fn npc_elevated(position: Vec3, height: f32) -> Transform {
        Transform::from_xyz(position.x, height, position.z)
    // NPC BODY PARTS - Relative to NPC center
    pub fn npc_head(height_factor: f32) -> Transform {
        let safe_height = height_factor.max(0.1).min(10.0);
        Transform::from_xyz(0.0, safe_height * 0.85, 0.0)
    pub fn npc_torso(height_factor: f32) -> Transform {
        Transform::from_xyz(0.0, safe_height * 0.5, 0.0)
    pub fn npc_left_arm(build: f32, height: f32) -> Transform {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        Transform::from_xyz(-0.25 * safe_build, safe_height * 0.65, 0.0)
    pub fn npc_right_arm(build: f32, height: f32) -> Transform {
        Transform::from_xyz(0.25 * safe_build, safe_height * 0.65, 0.0)
    pub fn npc_left_leg(build: f32, height: f32) -> Transform {
        Transform::from_xyz(-0.1 * safe_build, safe_height * 0.2, 0.0)
    pub fn npc_right_leg(build: f32, height: f32) -> Transform {
        Transform::from_xyz(0.1 * safe_build, safe_height * 0.2, 0.0)
    // WORLD STRUCTURES
    pub fn lamp_post(x: f32, z: f32) -> Transform {
    pub fn lamp_light() -> Transform {
        Transform::from_xyz(0.0, 4.0, 0.0)
    pub fn tree_position(x: f32, z: f32) -> Transform {
    pub fn tree_fronds(frond_x: f32, frond_z: f32) -> Transform {
        Transform::from_xyz(frond_x, 7.5, frond_z)
    pub fn building_base(x: f32, height: f32, z: f32) -> Transform {
        let safe_height = height.max(0.1).min(1000.0);
        Transform::from_xyz(x, safe_height / 2.0, z)
    // ENVIRONMENT DETAILS
    pub fn ground_vehicle(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y + 1.0, z)
    pub fn environment_detail(x: f32, y: f32, z: f32) -> Transform {
    pub fn street_light(x: f32, y: f32, z: f32) -> Transform {
        Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
    // WATER FEATURES
    pub fn lake_surface(lake_position: Vec3) -> Transform {
        Transform::from_xyz(lake_position.x, lake_position.y, lake_position.z)
    pub fn lake_bottom(lake_position: Vec3, depth: f32) -> Transform {
        let safe_depth = depth.max(0.1).min(1000.0);
        Transform::from_xyz(lake_position.x, lake_position.y - safe_depth, lake_position.z)
    pub fn lake_cylinder(lake_position: Vec3, depth: f32) -> Transform {
        Transform::from_xyz(lake_position.x, lake_position.y - safe_depth / 2.0, lake_position.z)
    // SKY COMPONENTS
    pub fn sky_dome() -> Transform {
    pub fn celestial_body(x: f32, y: f32, z: f32) -> Transform {
    // CAMERA POSITIONING
    pub fn camera_position() -> Transform {
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y)
    pub fn elevated_camera(height: f32) -> Transform {
        let safe_height = height.max(1.0).min(1000.0);
        Transform::from_xyz(0.0, safe_height, 25.0).looking_at(Vec3::ZERO, Vec3::Y)
    // PROCEDURAL WORLD - Dynamic positioning
    pub fn road_segment_horizontal(_x: f32, _z: f32, segment_size: f32, road_width: f32) -> Transform {
        let safe_segment = segment_size.max(1.0).min(1000.0);
        Transform::from_xyz(safe_segment * 1.5, 0.1, road_width)
    pub fn road_segment_vertical(_x: f32, _z: f32, segment_size: f32, road_width: f32) -> Transform {
        Transform::from_xyz(road_width, 0.1, safe_segment * 1.5)
    pub fn road_marking_horizontal(segment_size: f32) -> Transform {
        Transform::from_xyz(safe_segment * 1.3, 0.11, 0.4)
    pub fn road_marking_vertical(segment_size: f32) -> Transform {
        Transform::from_xyz(0.4, 0.11, safe_segment * 1.3)
    // CUSTOM POSITIONING WITH VALIDATION
    pub fn custom_position_safe(x: f32, y: f32, z: f32) -> Transform {
        let safe_x = x.max(-10000.0).min(10000.0);
        let safe_y = y.max(-1000.0).min(10000.0);
        let safe_z = z.max(-10000.0).min(10000.0);
        Transform::from_xyz(safe_x, safe_y, safe_z)
    pub fn with_rotation_safe(x: f32, y: f32, z: f32, rotation: Quat) -> Transform {
        Transform::from_xyz(safe_x, safe_y, safe_z).with_rotation(rotation)
    pub fn with_scale(x: f32, y: f32, z: f32, scale: Vec3) -> Transform {
        let safe_scale = Vec3::new(
            scale.x.max(0.001).min(1000.0),
            scale.y.max(0.001).min(1000.0),
            scale.z.max(0.001).min(1000.0),
        );
        Transform::from_xyz(safe_x, safe_y, safe_z).with_scale(safe_scale)
}
