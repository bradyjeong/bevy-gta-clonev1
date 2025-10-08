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
            ..default()
        });

        let road_asphalt_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        });

        let water_surface_template = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.3, 0.8, 0.7),
            metallic: 0.0,
            perceptual_roughness: 0.1,
            reflectance: 0.9,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        let building_concrete_template = materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.6, 0.6),
            metallic: 0.0,
            perceptual_roughness: 0.8,
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

    /// Get road asphalt material
    pub fn get_road_asphalt(&self) -> Handle<StandardMaterial> {
        self.road_asphalt_template.clone()
    }

    /// Get water surface material
    pub fn get_water_surface(&self) -> Handle<StandardMaterial> {
        self.water_surface_template.clone()
    }

    /// Get building material with specified color
    pub fn get_building_material(&self) -> Handle<StandardMaterial> {
        self.building_concrete_template.clone()
    }
}

/// Material creation helper functions that match exact patterns from codebase
/// CRITICAL: These create materials with identical properties to existing code
impl MaterialFactory {
    /// Create vehicle metallic material with specified color (matches exact pattern from codebase)
    pub fn create_vehicle_metallic(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.95,
            perceptual_roughness: 0.1,
            reflectance: 0.9,
            ..default()
        })
    }

    /// Create sky gradient material with specified color
    pub fn create_sky_gradient(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.0,
            perceptual_roughness: 1.0,
            unlit: true,
            ..default()
        })
    }

    /// Create building material with specified color
    pub fn create_building_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        })
    }

    /// Create wheel material (matches exact pattern from vehicle LOD system)
    pub fn create_wheel_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })
    }

    /// Create simple colored material (matches materials.add(color) pattern)
    pub fn create_simple_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })
    }

    /// Create aircraft material (matches F16 metallic pattern)
    pub fn create_aircraft_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })
    }

    /// Create F16 fuselage material (military gray with appropriate finish)
    pub fn create_f16_fuselage_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.50, 0.55), // Lighter tactical gray
            metallic: 0.85,
            perceptual_roughness: 0.25, // Shinier finish
            reflectance: 0.6,
            ..default()
        })
    }

    /// Create F16 canopy material (tinted glass)
    pub fn create_f16_canopy_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.3, 0.5, 0.3), // Blue-tinted glass
            metallic: 0.0,
            perceptual_roughness: 0.1, // Very smooth glass
            reflectance: 0.9,          // High reflectance for glass
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    }

    /// Create F16 engine nozzle material (heat-resistant steel)
    pub fn create_f16_engine_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25), // Dark steel
            metallic: 0.9,
            perceptual_roughness: 0.4, // Heat-treated finish
            reflectance: 0.5,
            ..default()
        })
    }

    /// Create F16 air intake material (dark interior)
    pub fn create_f16_intake_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1), // Very dark interior
            metallic: 0.6,
            perceptual_roughness: 0.7, // Rough internal surface
            reflectance: 0.2,
            ..default()
        })
    }

    /// Create low-detail material (high roughness for distant objects)
    pub fn create_low_detail_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.9,
            ..default()
        })
    }

    /// Create sky dome material (unlit with inside culling)
    pub fn create_sky_dome_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            cull_mode: Some(Face::Front),
            ..default()
        })
    }

    /// Create celestial body material (moon/stars with emissive and alpha)
    pub fn create_celestial_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        base_color: Color,
        emissive: LinearRgba,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color,
            emissive,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    }

    /// Create cloud material (unlit with alpha blending)
    pub fn create_cloud_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })
    }

    /// Create water bottom material (mud/sand with high roughness)
    pub fn create_water_bottom_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        })
    }

    /// Create water surface material (reflective with alpha blending)
    pub fn create_water_surface_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.8,
            metallic: 0.1,
            perceptual_roughness: 0.1,
            ..default()
        })
    }

    /// Create metallic material with custom properties
    pub fn create_metallic_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
        metallic: f32,
        roughness: f32,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic,
            perceptual_roughness: roughness,
            ..default()
        })
    }

    /// Create vehicle glass material (tinted glass with alpha blending)
    pub fn create_vehicle_glass_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color: color,
            metallic: 0.1,
            perceptual_roughness: 0.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    }

    /// Create emissive material for lights and glowing elements
    pub fn create_vehicle_emissive(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        base_color: Color,
        emissive_color: Color,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color,
            emissive: emissive_color.into(),
            metallic: 0.9,
            perceptual_roughness: 0.1,
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
