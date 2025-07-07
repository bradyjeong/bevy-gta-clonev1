use bevy::prelude::*;

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
            ..default()
        });
        
        let road_asphalt_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        });
        
        let water_surface_template = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.3, 0.8, 0.7),
            perceptual_roughness: 0.1,
            reflectance: 0.9,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        
        let building_concrete_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.6, 0.6),
            perceptual_roughness: 0.7,
            ..default()
        });
        
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
    }
    
    /// Get standard vehicle wheel material
    pub fn get_vehicle_wheel(&self) -> Handle<StandardMaterial> {
        self.vehicle_wheel_template.clone()
    }
    
    /// Get standard road asphalt material
    pub fn get_road_asphalt(&self) -> Handle<StandardMaterial> {
        self.road_asphalt_template.clone()
    }
    
    /// Get standard water surface material
    pub fn get_water_surface(&self) -> Handle<StandardMaterial> {
        self.water_surface_template.clone()
    }
    
    /// Get standard building concrete material
    pub fn get_building_concrete(&self) -> Handle<StandardMaterial> {
        self.building_concrete_template.clone()
    }
    
    /// Create metallic material with custom properties
    pub fn create_metallic_material(
        &self,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        metallic: f32,
        roughness: f32,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.7),
            metallic,
            perceptual_roughness: roughness,
            ..default()
        })
    }
    
    /// Create emissive material for lights and glowing elements
    pub fn create_emissive_material(
        &self,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        emissive_color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: emissive_color.into(),
            ..default()
        })
    }
}

/// System to initialize the material factory during startup
/// CRITICAL: This must run before any systems that create materials
pub fn initialize_material_factory(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let factory = MaterialFactory::new(&mut materials);
    commands.insert_resource(factory);
    println!("🏭 MATERIAL FACTORY: Initialized with template materials");
}
