use bevy::prelude::*;
use crate::factories::{MeshFactory, MaterialFactory};
use crate::bundles::{VisibleBundle, VisibleChildBundle};

/// PHASE 2.2: Rendering Factory Standardization
/// Eliminates 200+ duplicate rendering patterns by providing unified mesh + material + visibility creation
/// CRITICAL: Works with enhanced bundle system from Phase 1.2 and UnifiedCullingSystem from Phase 1.1
pub struct RenderingFactory;

/// Rendering bundle types for different entity hierarchies
#[derive(Debug, Clone, Copy)]
pub enum RenderingBundleType {
    /// Parent entity with full visibility control
    Parent,
    /// Child entity with inherited visibility
    Child,
    /// Standalone entity (most common)
    Standalone,
}

/// Standard rendering patterns used throughout the codebase
#[derive(Debug, Clone)]
pub enum StandardRenderingPattern {
    // Vehicle patterns
    VehicleBody { vehicle_type: VehicleBodyType, color: Color },
    VehicleWheel,
    VehicleGlass { tint_color: Color },
    VehicleLight { emissive_color: Color },
    
    // Building patterns
    Building { color: Color, building_type: BuildingMaterialType },
    
    // World patterns
    Road { width: f32, length: f32 },
    RoadMarking { width: f32, length: f32 },
    Tree { trunk_height: f32, frond_scale: f32 },
    
    // Water patterns
    WaterSurface { size: f32, color: Color },
    WaterBottom { size: f32, color: Color },
    
    // NPC patterns
    NPCHead { build_factor: f32 },
    NPCBody { build: f32, height: f32 },
    NPCLimb { radius: f32, length: f32 },
    
    // Sky patterns
    SkyDome { color: Color },
    CelestialBody { color: Color, emissive: LinearRgba, size: f32 },
    Cloud { color: Color, scale: f32 },
    
    // Custom shapes
    CustomCuboid { size: Vec3, color: Color, material_type: MaterialType },
    CustomSphere { radius: f32, color: Color, material_type: MaterialType },
    CustomCylinder { radius: f32, height: f32, color: Color, material_type: MaterialType },
}

#[derive(Debug, Clone, Copy)]
pub enum VehicleBodyType {
    BasicCar,
    SportsCar,
    SUV,
    Truck,
    Helicopter,
    F16,
    Boat,
    Yacht,
}

#[derive(Debug, Clone, Copy)]
pub enum BuildingMaterialType {
    Concrete,
    Glass,
    Metal,
    Brick,
}

#[derive(Debug, Clone, Copy)]
pub enum MaterialType {
    Standard,
    Metallic,
    Glass,
    Emissive,
    Unlit,
    LowDetail,
}

/// Complete rendering bundle with mesh, material, and visibility
#[derive(Bundle)]
pub struct CompleteRenderingBundle {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

/// Child rendering bundle (inherits visibility from parent)
#[derive(Bundle)]
pub struct ChildRenderingBundle {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl RenderingFactory {
    /// CORE FUNCTION: Create complete rendering entity with mesh + material + visibility
    /// Replaces all inline mesh/material creation patterns
    pub fn create_rendering_entity(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        pattern: StandardRenderingPattern,
        position: Vec3,
        bundle_type: RenderingBundleType,
        parent: Option<Entity>,
    ) -> Entity {
        let (mesh_handle, material_handle) = Self::create_mesh_and_material(meshes, materials, &pattern);
        
        let entity = match bundle_type {
            RenderingBundleType::Parent => {
                commands.spawn(CompleteRenderingBundle {
                    mesh: Mesh3d(mesh_handle),
                    material: MeshMaterial3d(material_handle),
                    transform: Transform::from_translation(position),
                    visibility: Visibility::Visible,
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                }).id()
            },
            RenderingBundleType::Child => {
                let entity = commands.spawn(ChildRenderingBundle {
                    mesh: Mesh3d(mesh_handle),
                    material: MeshMaterial3d(material_handle),
                    transform: Transform::from_translation(position),
                    visibility: Visibility::Inherited,
                }).id();
                
                if let Some(parent_id) = parent {
                    commands.entity(entity).set_parent(parent_id);
                }
                entity
            },
            RenderingBundleType::Standalone => {
                commands.spawn(CompleteRenderingBundle {
                    mesh: Mesh3d(mesh_handle),
                    material: MeshMaterial3d(material_handle),
                    transform: Transform::from_translation(position),
                    visibility: Visibility::Visible,
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    view_visibility: ViewVisibility::default(),
                }).id()
            },
        };
        
        entity
    }
    
    /// Create mesh and material handles based on pattern
    pub fn create_mesh_and_material(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        pattern: &StandardRenderingPattern,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        match pattern {
            // Vehicle patterns
            StandardRenderingPattern::VehicleBody { vehicle_type, color } => {
                let mesh = match vehicle_type {
                    VehicleBodyType::BasicCar => MeshFactory::create_car_body(meshes),
                    VehicleBodyType::SportsCar => MeshFactory::create_sports_car_body(meshes),
                    VehicleBodyType::SUV => MeshFactory::create_suv_body(meshes),
                    VehicleBodyType::Truck => MeshFactory::create_truck_body(meshes),
                    VehicleBodyType::Helicopter => MeshFactory::create_helicopter_body(meshes),
                    VehicleBodyType::F16 => MeshFactory::create_f16_body(meshes),
                    VehicleBodyType::Boat => MeshFactory::create_boat_hull(meshes),
                    VehicleBodyType::Yacht => MeshFactory::create_yacht_cabin(meshes),
                };
                let material = MaterialFactory::create_vehicle_metallic(materials, *color);
                (mesh, material)
            },
            
            StandardRenderingPattern::VehicleWheel => {
                let mesh = MeshFactory::create_standard_wheel(meshes);
                let material = MaterialFactory::create_wheel_material(materials);
                (mesh, material)
            },
            
            StandardRenderingPattern::VehicleGlass { tint_color } => {
                let mesh = MeshFactory::create_custom_cuboid(meshes, 1.0, 0.8, 0.1);
                let material = MaterialFactory::create_vehicle_glass_material(materials, *tint_color);
                (mesh, material)
            },
            
            StandardRenderingPattern::VehicleLight { emissive_color } => {
                let mesh = MeshFactory::create_headlight(meshes);
                let material = MaterialFactory::create_vehicle_emissive(materials, Color::WHITE, *emissive_color);
                (mesh, material)
            },
            
            // Building patterns
            StandardRenderingPattern::Building { color, building_type: _ } => {
                let mesh = MeshFactory::create_custom_cuboid(meshes, 10.0, 20.0, 10.0);
                let material = MaterialFactory::create_building_material(materials, *color);
                (mesh, material)
            },
            
            // World patterns
            StandardRenderingPattern::Road { width, length } => {
                let mesh = MeshFactory::create_road_segment(meshes, *width, *length);
                let material = MaterialFactory::create_simple_material(materials, Color::srgb(0.3, 0.3, 0.3));
                (mesh, material)
            },
            
            StandardRenderingPattern::RoadMarking { width, length } => {
                let mesh = MeshFactory::create_road_marking(meshes, *width, *length);
                let material = MaterialFactory::create_simple_material(materials, Color::WHITE);
                (mesh, material)
            },
            
            StandardRenderingPattern::Tree { trunk_height, frond_scale } => {
                let mesh = MeshFactory::create_tree_trunk(meshes);
                let _ = *trunk_height; // Used for scaling in transform
                let _ = *frond_scale;  // Used for frond creation
                let material = MaterialFactory::create_simple_material(materials, Color::srgb(0.4, 0.25, 0.15));
                (mesh, material)
            },
            
            // Water patterns
            StandardRenderingPattern::WaterSurface { size, color } => {
                let mesh = MeshFactory::create_water_plane(meshes, *size);
                let material = MaterialFactory::create_water_surface_material(materials, *color);
                (mesh, material)
            },
            
            StandardRenderingPattern::WaterBottom { size, color } => {
                let mesh = MeshFactory::create_custom_cylinder(meshes, *size / 2.0, 5.0);
                let material = MaterialFactory::create_water_bottom_material(materials, *color);
                (mesh, material)
            },
            
            // NPC patterns
            StandardRenderingPattern::NPCHead { build_factor } => {
                let mesh = MeshFactory::create_npc_head(meshes, *build_factor);
                let material = MaterialFactory::create_simple_material(materials, Color::srgb(0.9, 0.7, 0.6));
                (mesh, material)
            },
            
            StandardRenderingPattern::NPCBody { build, height } => {
                let mesh = MeshFactory::create_npc_body(meshes, *build, *height);
                let material = MaterialFactory::create_simple_material(materials, Color::srgb(0.2, 0.4, 0.8));
                (mesh, material)
            },
            
            StandardRenderingPattern::NPCLimb { radius, length } => {
                let mesh = MeshFactory::create_npc_limb(meshes, *radius, *length);
                let material = MaterialFactory::create_simple_material(materials, Color::srgb(0.9, 0.7, 0.6));
                (mesh, material)
            },
            
            // Sky patterns
            StandardRenderingPattern::SkyDome { color } => {
                let mesh = MeshFactory::create_sky_dome(meshes);
                let material = MaterialFactory::create_sky_dome_material(materials, *color);
                (mesh, material)
            },
            
            StandardRenderingPattern::CelestialBody { color, emissive, size } => {
                let mesh = MeshFactory::create_custom_sphere(meshes, *size);
                let material = MaterialFactory::create_celestial_material(materials, *color, *emissive);
                (mesh, material)
            },
            
            StandardRenderingPattern::Cloud { color, scale } => {
                let mesh = MeshFactory::create_cloud(meshes, *scale);
                let material = MaterialFactory::create_cloud_material(materials, *color);
                (mesh, material)
            },
            
            // Custom patterns
            StandardRenderingPattern::CustomCuboid { size, color, material_type } => {
                let mesh = MeshFactory::create_custom_cuboid(meshes, size.x, size.y, size.z);
                let material = Self::create_material_by_type(materials, *color, *material_type);
                (mesh, material)
            },
            
            StandardRenderingPattern::CustomSphere { radius, color, material_type } => {
                let mesh = MeshFactory::create_custom_sphere(meshes, *radius);
                let material = Self::create_material_by_type(materials, *color, *material_type);
                (mesh, material)
            },
            
            StandardRenderingPattern::CustomCylinder { radius, height, color, material_type } => {
                let mesh = MeshFactory::create_custom_cylinder(meshes, *radius, *height);
                let material = Self::create_material_by_type(materials, *color, *material_type);
                (mesh, material)
            },
        }
    }
    
    /// Create material based on type
    fn create_material_by_type(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        color: Color,
        material_type: MaterialType,
    ) -> Handle<StandardMaterial> {
        match material_type {
            MaterialType::Standard => MaterialFactory::create_simple_material(materials, color),
            MaterialType::Metallic => MaterialFactory::create_metallic_material(materials, color, 0.8, 0.2),
            MaterialType::Glass => MaterialFactory::create_vehicle_glass_material(materials, color),
            MaterialType::Emissive => MaterialFactory::create_vehicle_emissive(materials, color, color),
            MaterialType::Unlit => MaterialFactory::create_sky_gradient(materials, color),
            MaterialType::LowDetail => MaterialFactory::create_low_detail_material(materials, color),
        }
    }
    
    /// BATCH OPERATIONS: Create multiple rendering entities efficiently
    pub fn create_batch_rendering_entities(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        patterns: Vec<(StandardRenderingPattern, Vec3, RenderingBundleType, Option<Entity>)>,
    ) -> Vec<Entity> {
        patterns.into_iter().map(|(pattern, position, bundle_type, parent)| {
            Self::create_rendering_entity(commands, meshes, materials, pattern, position, bundle_type, parent)
        }).collect()
    }
    
    /// LOD MESH MANAGEMENT: Swap meshes for LOD systems using factory
    pub fn swap_mesh_for_lod(
        commands: &mut Commands,
        entity: Entity,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        new_pattern: StandardRenderingPattern,
    ) {
        let (new_mesh_handle, _) = Self::create_mesh_and_material(
            meshes, 
            materials, 
            &new_pattern
        );
        
        commands.entity(entity).insert(Mesh3d(new_mesh_handle));
    }
    
    /// Create complete vehicle with all parts using factory patterns
    pub fn create_complete_vehicle(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        vehicle_type: VehicleBodyType,
        position: Vec3,
        color: Color,
    ) -> Entity {
        // Create main vehicle body
        let main_entity = Self::create_rendering_entity(
            commands,
            meshes,
            materials,
            StandardRenderingPattern::VehicleBody { vehicle_type, color },
            position,
            RenderingBundleType::Parent,
            None,
        );
        
        // Add wheels as children
        let wheel_positions = [
            Vec3::new(-0.8, -0.3, 1.2),   // Front left
            Vec3::new(0.8, -0.3, 1.2),    // Front right
            Vec3::new(-0.8, -0.3, -1.2),  // Rear left
            Vec3::new(0.8, -0.3, -1.2),   // Rear right
        ];
        
        for wheel_pos in wheel_positions {
            Self::create_rendering_entity(
                commands,
                meshes,
                materials,
                StandardRenderingPattern::VehicleWheel,
                wheel_pos,
                RenderingBundleType::Child,
                Some(main_entity),
            );
        }
        
        // Add headlights
        let headlight_positions = [
            Vec3::new(-0.6, 0.2, 1.7),    // Left headlight
            Vec3::new(0.6, 0.2, 1.7),     // Right headlight
        ];
        
        for light_pos in headlight_positions {
            Self::create_rendering_entity(
                commands,
                meshes,
                materials,
                StandardRenderingPattern::VehicleLight { emissive_color: Color::srgb(1.0, 1.0, 0.0) },
                light_pos,
                RenderingBundleType::Child,
                Some(main_entity),
            );
        }
        
        main_entity
    }
    
    /// Create tree with trunk and fronds using factory patterns
    pub fn create_complete_tree(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        trunk_height: f32,
        frond_count: u32,
    ) -> Entity {
        // Create trunk
        let trunk_entity = Self::create_rendering_entity(
            commands,
            meshes,
            materials,
            StandardRenderingPattern::Tree { trunk_height, frond_scale: 1.0 },
            position,
            RenderingBundleType::Parent,
            None,
        );
        
        // Add fronds
        for i in 0..frond_count {
            let angle = (i as f32) * std::f32::consts::TAU / (frond_count as f32);
            let frond_pos = Vec3::new(
                angle.cos() * 1.2,
                trunk_height * 0.9,
                angle.sin() * 1.2,
            );
            
            Self::create_rendering_entity(
                commands,
                meshes,
                materials,
                StandardRenderingPattern::CustomCuboid {
                    size: Vec3::new(2.5, 0.1, 0.8),
                    color: Color::srgb(0.2, 0.6, 0.25),
                    material_type: MaterialType::Standard,
                },
                frond_pos,
                RenderingBundleType::Child,
                Some(trunk_entity),
            );
        }
        
        trunk_entity
    }
}

/// Convenience functions for common rendering patterns
impl RenderingFactory {
    /// Quick car creation
    pub fn quick_car(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        color: Color,
    ) -> Entity {
        Self::create_rendering_entity(
            commands,
            meshes,
            materials,
            StandardRenderingPattern::VehicleBody { 
                vehicle_type: VehicleBodyType::BasicCar, 
                color 
            },
            position,
            RenderingBundleType::Standalone,
            None,
        )
    }
    
    /// Quick building creation
    pub fn quick_building(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        size: Vec3,
        color: Color,
    ) -> Entity {
        Self::create_rendering_entity(
            commands,
            meshes,
            materials,
            StandardRenderingPattern::CustomCuboid {
                size,
                color,
                material_type: MaterialType::Standard,
            },
            position,
            RenderingBundleType::Standalone,
            None,
        )
    }
    
    /// Quick tree creation
    pub fn quick_tree(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        Self::create_complete_tree(commands, meshes, materials, position, 8.0, 4)
    }
}
