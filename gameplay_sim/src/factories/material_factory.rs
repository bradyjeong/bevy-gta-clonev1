use bevy::prelude::*;
use bevy::render::render_resource::Face;

/// Unified material factory that eliminates duplicate StandardMaterial creation
/// CRITICAL: This replaces 53+ duplicate material patterns across the codebase
#[derive(Resource)]
pub struct MaterialFactory {
    // Pre-cached standard material templates
    vehicle_glass_template: Handle<StandardMaterial>,
    vehicle_wheel_template: Handle<StandardMaterial>,
    road_asphalt_template: Handle<StandardMaterial>,
    water_surface_template: Handle<StandardMaterial>,
    building_concrete_template: Handle<StandardMaterial>,
}
impl MaterialFactory {
    /// SAFETY: Initialize factory with pre-built material templates
    /// This must be called during app setup before any systems use the factory
    pub fn new(materials: &mut ResMut<Assets<StandardMaterial>>) -> Self {
        let vehicle_glass_template = materials.add(StandardMaterial {
            base_color: Color::srgba(0.8, 0.9, 1.0, 0.3),
            metallic: 0.0,
            perceptual_roughness: 0.0,
            reflectance: 0.1,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        
        let vehicle_wheel_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.8,
        let road_asphalt_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            perceptual_roughness: 0.9,
        let water_surface_template = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.3, 0.8, 0.7),
            perceptual_roughness: 0.1,
            reflectance: 0.9,
        let building_concrete_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.6, 0.6),
        Self {
            vehicle_glass_template,
            vehicle_wheel_template,
            road_asphalt_template,
            water_surface_template,
            building_concrete_template,
        }
    }
    
    /// Get standard vehicle glass material
    pub fn get_vehicle_glass(&self) -> Handle<StandardMaterial> {
        self.vehicle_glass_template.clone()
    /// Get standard vehicle wheel material
    pub fn get_vehicle_wheel(&self) -> Handle<StandardMaterial> {
        self.vehicle_wheel_template.clone()
    /// Get road asphalt material
    pub fn get_road_asphalt(&self) -> Handle<StandardMaterial> {
        self.road_asphalt_template.clone()
    /// Get water surface material
    pub fn get_water_surface(&self) -> Handle<StandardMaterial> {
        self.water_surface_template.clone()
    /// Get building material with specified color
    pub fn get_building_material(&self) -> Handle<StandardMaterial> {
        self.building_concrete_template.clone()
/// Material creation helper functions that match exact patterns from codebase
/// CRITICAL: These create materials with identical properties to existing code
    /// Create vehicle metallic material with specified color (matches exact pattern from codebase)
    pub fn create_vehicle_metallic(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.95,
        })
    /// Create sky gradient material with specified color
    pub fn create_sky_gradient(
            perceptual_roughness: 1.0,
            unlit: true,
    /// Create building material with specified color
    pub fn create_building_material(
    /// Create wheel material (matches exact pattern from vehicle LOD system)
    pub fn create_wheel_material(
    /// Create simple colored material (matches materials.add(color) pattern)
    pub fn create_simple_material(
    /// Create aircraft material (matches F16 metallic pattern)
    pub fn create_aircraft_material(
            metallic: 0.8,
            perceptual_roughness: 0.2,
    /// Create F16 fuselage material (military gray with appropriate finish)
    pub fn create_f16_fuselage_material(
            base_color: Color::srgb(0.35, 0.37, 0.40), // F16 Falcon Gray
            metallic: 0.7,
            perceptual_roughness: 0.3, // Semi-matte military finish
            reflectance: 0.4,
    /// Create F16 canopy material (tinted glass)
    pub fn create_f16_canopy_material(
            base_color: Color::srgba(0.1, 0.3, 0.5, 0.3), // Blue-tinted glass
            perceptual_roughness: 0.1, // Very smooth glass
            reflectance: 0.9, // High reflectance for glass
    /// Create F16 engine nozzle material (heat-resistant steel)
    pub fn create_f16_engine_material(
            base_color: Color::srgb(0.2, 0.2, 0.25), // Dark steel
            metallic: 0.9,
            perceptual_roughness: 0.4, // Heat-treated finish
            reflectance: 0.5,
    /// Create F16 air intake material (dark interior)
    pub fn create_f16_intake_material(
            base_color: Color::srgb(0.1, 0.1, 0.1), // Very dark interior
            metallic: 0.6,
            perceptual_roughness: 0.7, // Rough internal surface
            reflectance: 0.2,
    /// Create low-detail material (high roughness for distant objects)
    pub fn create_low_detail_material(
    /// Create sky dome material (unlit with inside culling)
    pub fn create_sky_dome_material(
            cull_mode: Some(Face::Front),
    /// Create celestial body material (moon/stars with emissive and alpha)
    pub fn create_celestial_material(
        base_color: Color,
        emissive: LinearRgba,
            base_color,
            emissive,
    /// Create cloud material (unlit with alpha blending)
    pub fn create_cloud_material(
    /// Create water bottom material (mud/sand with high roughness)
    pub fn create_water_bottom_material(
    /// Create water surface material (reflective with alpha blending)
    pub fn create_water_surface_material(
            reflectance: 0.8,
    /// Create metallic material with custom properties
    pub fn create_metallic_material(
        metallic: f32,
        roughness: f32,
            metallic,
            perceptual_roughness: roughness,
    /// Create vehicle glass material (tinted glass with alpha blending)
    pub fn create_vehicle_glass_material(
    /// Create emissive material for lights and glowing elements
    pub fn create_vehicle_emissive(
        emissive_color: Color,
            emissive: emissive_color.into(),
/// System to initialize the material factory during startup
/// CRITICAL: This must run before any systems that create materials
pub fn initialize_material_factory(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let factory = MaterialFactory::new(&mut materials);
    commands.insert_resource(factory);
    println!("🏭 MATERIAL FACTORY: Initialized with template materials");
