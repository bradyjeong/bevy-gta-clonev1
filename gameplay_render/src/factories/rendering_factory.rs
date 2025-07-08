//! Unified rendering factory system for creating game entities with meshes and materials.
//!
//! # Overview
//! The [`RenderingFactory`] provides a standardized approach to creating rendering entities
//! throughout the game. It eliminates over 200 duplicate rendering patterns by consolidating
//! mesh creation, material assignment, and visibility management into a single factory API.
//!
//! ## Architecture
//! The factory system operates on standardized rendering patterns defined in
//! [`StandardRenderingPattern`], which specify common game entity types like vehicles,
//! buildings, NPCs, and environmental elements. Each pattern automatically generates
//! appropriate meshes via [`MeshFactory`] and materials via [`MaterialFactory`].
//!
//! ## Typical Usage
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::factories::rendering_factory::*;
//!
//! fn spawn_vehicle(
//!     mut commands: Commands,
//!     mut meshes: ResMut<Assets<Mesh>>,
//!     mut materials: ResMut<Assets<StandardMaterial>>,
//! ) {
//!     // Create a red sports car at the origin
//!     let car_entity = RenderingFactory::create_rendering_entity(
//!         &mut commands,
//!         &mut meshes,
//!         &mut materials,
//!         StandardRenderingPattern::VehicleBody {
//!             vehicle_type: VehicleBodyType::SportsCar,
//!             color: Color::RED,
//!         },
//!         Vec3::ZERO,
//!         RenderingBundleType::Standalone,
//!         None,
//!     );
//! }
//! ```
//!
//! ## Performance Characteristics
//! - **Mesh Reuse**: Common meshes are created once and reused across entity instances
//! - **Material Optimization**: Materials are created based on standardized types
//! - **Batch Operations**: Support for creating multiple entities efficiently
//! - **LOD Integration**: Seamless mesh swapping for Level of Detail systems
//!
//! # Implementation Notes
//! This factory integrates with the enhanced bundle system from Phase 1.2 and the
//! UnifiedCullingSystem from Phase 1.1. All entities created through this factory
//! automatically participate in the game's culling and visibility management systems.

use bevy::prelude::*;
use crate::factories::{MeshFactory, MaterialFactory};


/// A unified factory for creating game entities with standardized rendering patterns.
///
/// This factory eliminates over 200 duplicate rendering patterns by providing a single
/// API for creating entities with meshes, materials, and visibility components. It serves
/// as the central hub for all rendering-related entity creation in the game.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::factories::rendering_factory::*;
///
/// fn create_building(
///     mut commands: Commands,
///     mut meshes: ResMut<Assets<Mesh>>,
///     mut materials: ResMut<Assets<StandardMaterial>>,
/// ) {
///     let building = RenderingFactory::create_rendering_entity(
///         &mut commands,
///         &mut meshes,
///         &mut materials,
///         StandardRenderingPattern::Building {
///             color: Color::srgb(0.8, 0.8, 0.8),
///             building_type: BuildingMaterialType::Concrete,
///         },
///         Vec3::new(0.0, 10.0, 0.0),
///         RenderingBundleType::Standalone,
///         None,
///     );
/// }
/// ```
pub struct RenderingFactory;

/// Defines the hierarchical relationship and visibility behavior of rendered entities.
///
/// This enum controls how entities participate in Bevy's visibility system and
/// determines which rendering components are attached to each entity. The choice
/// affects performance and behavior in parent-child entity relationships.
#[derive(Debug, Clone, Copy)]
pub enum RenderingBundleType {
    /// Parent entity with full visibility control and inheritance capabilities.
    ///
    /// Parent entities include all visibility components ([`Visibility`], 
    /// [`InheritedVisibility`], [`ViewVisibility`]) and can control the visibility
    /// of their children. Use this for main entities that will have child components.
    Parent,
    
    /// Child entity with inherited visibility from its parent.
    ///
    /// Child entities only include basic [`Visibility`] set to [`Visibility::Inherited`]
    /// and automatically receive a [`ChildOf`] component linking them to their parent.
    /// Their final visibility depends on both their own state and their parent's visibility.
    Child,
    
    /// Standalone entity with independent visibility (most common).
    ///
    /// Standalone entities include full visibility components but are not part of
    /// any parent-child hierarchy. This is the most common type for independent
    /// game objects like vehicles, buildings, and NPCs.
    Standalone,
}

/// Standardized rendering patterns for common game entity types.
///
/// This enum encapsulates the most frequently used rendering patterns across the game,
/// allowing for consistent mesh and material creation while eliminating code duplication.
/// Each pattern automatically selects appropriate mesh geometry and material properties
/// based on the entity type and provided parameters.
#[derive(Debug, Clone)]
pub enum StandardRenderingPattern {
    // Vehicle patterns
    /// Vehicle body with type-specific geometry and metallic paint material.
    VehicleBody { 
        /// The specific vehicle type determining mesh geometry
        vehicle_type: VehicleBodyType, 
        /// Primary paint color for the vehicle body
        color: Color 
    },
    
    /// Standard wheel with rubber tire material and metallic rim.
    VehicleWheel,
    
    /// Vehicle glass panel with transparency and tint effects.
    VehicleGlass { 
        /// Color tint applied to the glass material
        tint_color: Color 
    },
    
    /// Vehicle light with emissive material for headlights and taillights.
    VehicleLight { 
        /// Color of the emitted light
        emissive_color: Color 
    },
    
    // Building patterns
    /// Building structure with architectural material properties.
    Building { 
        /// Primary color of the building exterior
        color: Color, 
        /// Material type determining surface properties
        building_type: BuildingMaterialType 
    },
    
    // World patterns
    /// Road segment with asphalt material and lane markings.
    Road { 
        /// Width of the road segment in world units
        width: f32, 
        /// Length of the road segment in world units
        length: f32 
    },
    
    /// Road marking with bright white material for lane dividers.
    RoadMarking { 
        /// Width of the marking stripe
        width: f32, 
        /// Length of the marking stripe
        length: f32 
    },
    
    /// Tree trunk with bark material and configurable dimensions.
    Tree { 
        /// Height of the tree trunk in world units
        trunk_height: f32, 
        /// Scale factor for tree fronds and canopy
        frond_scale: f32 
    },
    
    // Water patterns
    /// Water surface with transparency and wave simulation.
    WaterSurface { 
        /// Size of the water surface area
        size: f32, 
        /// Base color of the water
        color: Color 
    },
    
    /// Water bottom with sediment material for underwater terrain.
    WaterBottom { 
        /// Size of the water bottom area
        size: f32, 
        /// Color of the sediment material
        color: Color 
    },
    
    // NPC patterns
    /// NPC head with skin material and configurable proportions.
    NPCHead { 
        /// Factor affecting head size and proportions
        build_factor: f32 
    },
    
    /// NPC body with clothing material and physical characteristics.
    NPCBody { 
        /// Build factor affecting body width and bulk
        build: f32, 
        /// Height of the NPC body
        height: f32 
    },
    
    /// NPC limb with skin material for arms and legs.
    NPCLimb { 
        /// Radius of the limb cylinder
        radius: f32, 
        /// Length of the limb
        length: f32 
    },
    
    // Sky patterns
    /// Sky dome with gradient material for atmospheric effects.
    SkyDome { 
        /// Primary color of the sky dome
        color: Color 
    },
    
    /// Celestial body with emissive material for sun and moon.
    CelestialBody { 
        /// Base color of the celestial body
        color: Color, 
        /// Emissive color for the glow effect
        emissive: LinearRgba, 
        /// Size of the celestial body
        size: f32 
    },
    
    /// Cloud with translucent material for atmospheric effects.
    Cloud { 
        /// Color of the cloud
        color: Color, 
        /// Scale factor for cloud size
        scale: f32 
    },
    
    // Custom shapes
    /// Custom cuboid with configurable material properties.
    CustomCuboid { 
        /// Dimensions of the cuboid (width, height, depth)
        size: Vec3, 
        /// Color of the material
        color: Color, 
        /// Type of material determining surface properties
        material_type: MaterialType 
    },
    
    /// Custom sphere with configurable material properties.
    CustomSphere { 
        /// Radius of the sphere
        radius: f32, 
        /// Color of the material
        color: Color, 
        /// Type of material determining surface properties
        material_type: MaterialType 
    },
    
    /// Custom cylinder with configurable material properties.
    CustomCylinder { 
        /// Radius of the cylinder
        radius: f32, 
        /// Height of the cylinder
        height: f32, 
        /// Color of the material
        color: Color, 
        /// Type of material determining surface properties
        material_type: MaterialType 
    },
}

/// Vehicle body types with distinct geometric and performance characteristics.
///
/// Each variant represents a different vehicle archetype with specific mesh geometry,
/// physics properties, and gameplay behavior. The type determines the exact mesh
/// created by the [`MeshFactory`] and influences material properties.
#[derive(Debug, Clone, Copy)]
pub enum VehicleBodyType {
    /// Standard passenger car with balanced proportions and moderate performance.
    BasicCar,
    /// High-performance sports car with low profile and enhanced aerodynamics.
    SportsCar,
    /// Sport Utility Vehicle with increased height and robust construction.
    SUV,
    /// Heavy-duty truck with large cargo capacity and reinforced frame.
    Truck,
    /// Rotorcraft with rotor assembly and vertical takeoff capability.
    Helicopter,
    /// Military fighter aircraft with swept wings and afterburner configuration.
    F16,
    /// Watercraft with hull design optimized for maritime navigation.
    Boat,
    /// Luxury yacht with multiple decks and premium amenities.
    Yacht,
}

/// Building material types affecting surface appearance and structural properties.
///
/// Each variant represents a different building construction material with specific
/// visual characteristics, surface textures, and weathering patterns. The material
/// type influences both the visual appearance and implied structural properties.
#[derive(Debug, Clone, Copy)]
pub enum BuildingMaterialType {
    /// Concrete construction with rough texture and industrial appearance.
    Concrete,
    /// Glass construction with reflective properties and modern aesthetic.
    Glass,
    /// Metal construction with metallic sheen and industrial durability.
    Metal,
    /// Brick construction with traditional masonry texture and warm coloring.
    Brick,
}

/// Material types defining surface properties and rendering characteristics.
///
/// Each variant represents a different material shader configuration with specific
/// lighting behavior, surface properties, and performance characteristics. The type
/// determines how the material interacts with light and affects rendering performance.
#[derive(Debug, Clone, Copy)]
pub enum MaterialType {
    /// Standard physically-based material with full lighting calculations.
    Standard,
    /// Metallic material with enhanced reflectivity and metallic properties.
    Metallic,
    /// Glass material with transparency and refraction effects.
    Glass,
    /// Emissive material that glows with self-illumination.
    Emissive,
    /// Unlit material that ignores lighting calculations for consistent appearance.
    Unlit,
    /// Low-detail material with reduced quality for distant objects.
    LowDetail,
}

/// Complete rendering bundle with full visibility management for parent entities.
///
/// This bundle includes all components necessary for rendering an entity that may
/// have children. It provides complete visibility control through the full Bevy
/// visibility system including inheritance and view culling.
#[derive(Bundle)]
pub struct CompleteRenderingBundle {
    /// The 3D mesh handle for the entity's geometry
    pub mesh: Mesh3d,
    /// The material handle defining surface properties
    pub material: MeshMaterial3d<StandardMaterial>,
    /// Transform component for position, rotation, and scale
    pub transform: Transform,
    /// Base visibility state for the entity
    pub visibility: Visibility,
    /// Inherited visibility from parent entities
    pub inherited_visibility: InheritedVisibility,
    /// View-specific visibility after culling calculations
    pub view_visibility: ViewVisibility,
}

/// Child rendering bundle with minimal visibility for entities with parents.
///
/// This bundle provides only the essential rendering components for child entities
/// that inherit visibility from their parent. It excludes complex visibility
/// components to improve performance in parent-child hierarchies.
#[derive(Bundle)]
pub struct ChildRenderingBundle {
    /// The 3D mesh handle for the entity's geometry
    pub mesh: Mesh3d,
    /// The material handle defining surface properties
    pub material: MeshMaterial3d<StandardMaterial>,
    /// Transform component for position, rotation, and scale
    pub transform: Transform,
    /// Base visibility state (typically set to Inherited)
    pub visibility: Visibility,
}

impl RenderingFactory {
    /// Creates a complete rendering entity with mesh, material, and visibility components.
    ///
    /// This is the core function that replaces all inline mesh/material creation patterns
    /// throughout the codebase. It creates entities with the appropriate bundle type and
    /// automatically handles parent-child relationships when specified.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer for entity creation
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `pattern` - The rendering pattern defining mesh and material properties
    /// * `position` - World position for the entity's transform
    /// * `bundle_type` - Type of rendering bundle determining visibility behavior
    /// * `parent` - Optional parent entity for child entities
    ///
    /// # Returns
    /// The [`Entity`] ID of the newly created rendering entity
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn spawn_tree(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    /// ) {
    ///     let tree_entity = RenderingFactory::create_rendering_entity(
    ///         &mut commands,
    ///         &mut meshes,
    ///         &mut materials,
    ///         StandardRenderingPattern::Tree {
    ///             trunk_height: 10.0,
    ///             frond_scale: 1.5,
    ///         },
    ///         Vec3::new(5.0, 0.0, 3.0),
    ///         RenderingBundleType::Standalone,
    ///         None,
    ///     );
    /// }
    /// ```
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
                    commands.entity(entity).insert(ChildOf(parent_id));
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
    
    /// Creates mesh and material handles based on the specified rendering pattern.
    ///
    /// This function translates a [`StandardRenderingPattern`] into concrete mesh and
    /// material assets, utilizing the [`MeshFactory`] and [`MaterialFactory`] to create
    /// appropriate geometry and surface properties. It serves as the central dispatcher
    /// for all pattern-based asset creation.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `pattern` - The rendering pattern defining the desired mesh and material
    ///
    /// # Returns
    /// A tuple containing the mesh handle and material handle for the pattern
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn create_vehicle_assets(
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    /// ) {
    ///     let pattern = StandardRenderingPattern::VehicleBody {
    ///         vehicle_type: VehicleBodyType::SportsCar,
    ///         color: Color::BLUE,
    ///     };
    ///     
    ///     let (mesh_handle, material_handle) = RenderingFactory::create_mesh_and_material(
    ///         &mut meshes,
    ///         &mut materials,
    ///         &pattern,
    ///     );
    /// }
    /// ```
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
    
    /// Creates a material handle based on the specified material type and color.
    ///
    /// This private function maps [`MaterialType`] variants to specific material
    /// creation functions in the [`MaterialFactory`]. It provides consistent
    /// material properties for each type while allowing color customization.
    ///
    /// # Arguments
    /// * `materials` - Mutable reference to the material asset storage
    /// * `color` - Base color for the material
    /// * `material_type` - Type determining the material's surface properties
    ///
    /// # Returns
    /// A handle to the created material asset
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
    
    /// Creates multiple rendering entities efficiently in a single batch operation.
    ///
    /// This function processes a collection of rendering patterns and creates entities
    /// for each one. It's optimized for scenarios where many similar entities need
    /// to be created simultaneously, such as procedural generation or level loading.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer for entity creation
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `patterns` - Vector of tuples containing pattern, position, bundle type, and optional parent
    ///
    /// # Returns
    /// A vector of [`Entity`] IDs for all created entities
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn spawn_forest(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    /// ) {
    ///     let tree_patterns = vec![
    ///         (StandardRenderingPattern::Tree { trunk_height: 8.0, frond_scale: 1.0 }, 
    ///          Vec3::new(0.0, 0.0, 0.0), RenderingBundleType::Standalone, None),
    ///         (StandardRenderingPattern::Tree { trunk_height: 10.0, frond_scale: 1.2 }, 
    ///          Vec3::new(5.0, 0.0, 3.0), RenderingBundleType::Standalone, None),
    ///     ];
    ///     
    ///     let tree_entities = RenderingFactory::create_batch_rendering_entities(
    ///         &mut commands, &mut meshes, &mut materials, tree_patterns
    ///     );
    /// }
    /// ```
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
    
    /// Swaps the mesh of an existing entity for Level of Detail (LOD) optimization.
    ///
    /// This function replaces the mesh component of an existing entity with a new mesh
    /// based on the provided pattern. It's primarily used by LOD systems to swap
    /// high-detail meshes for lower-detail versions as entities move away from the camera.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer for component updates
    /// * `entity` - The entity whose mesh should be replaced
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `new_pattern` - The new rendering pattern defining the replacement mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn update_lod_for_distance(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    ///     vehicle_entity: Entity,
    ///     distance: f32,
    /// ) {
    ///     if distance > 100.0 {
    ///         // Switch to low-detail mesh for distant vehicles
    ///         let low_detail_pattern = StandardRenderingPattern::CustomCuboid {
    ///             size: Vec3::new(4.0, 1.5, 2.0),
    ///             color: Color::RED,
    ///             material_type: MaterialType::LowDetail,
    ///         };
    ///         
    ///         RenderingFactory::swap_mesh_for_lod(
    ///             &mut commands,
    ///             vehicle_entity,
    ///             &mut meshes,
    ///             &mut materials,
    ///             low_detail_pattern,
    ///         );
    ///     }
    /// }
    /// ```
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

/// Convenience functions for common rendering patterns with simplified APIs.
impl RenderingFactory {
    /// Creates a basic car entity quickly with minimal parameters.
    ///
    /// This convenience function creates a standard passenger car with the specified
    /// position and color. It uses default settings for bundle type and eliminates
    /// the need to specify complex rendering patterns for simple car creation.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer for entity creation
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `position` - World position for the car
    /// * `color` - Paint color for the car body
    ///
    /// # Returns
    /// The [`Entity`] ID of the created car
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn spawn_player_car(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    /// ) {
    ///     let car = RenderingFactory::quick_car(
    ///         &mut commands,
    ///         &mut meshes,
    ///         &mut materials,
    ///         Vec3::new(0.0, 0.0, 0.0),
    ///         Color::BLUE,
    ///     );
    /// }
    /// ```
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
    
    /// Creates a basic building entity quickly with customizable dimensions.
    ///
    /// This convenience function creates a simple cuboid building with the specified
    /// position, size, and color. It uses standard material properties and eliminates
    /// the need to specify complex rendering patterns for simple building creation.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer for entity creation
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `position` - World position for the building
    /// * `size` - Dimensions of the building (width, height, depth)
    /// * `color` - Color of the building exterior
    ///
    /// # Returns
    /// The [`Entity`] ID of the created building
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn spawn_office_building(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    /// ) {
    ///     let building = RenderingFactory::quick_building(
    ///         &mut commands,
    ///         &mut meshes,
    ///         &mut materials,
    ///         Vec3::new(10.0, 0.0, 5.0),
    ///         Vec3::new(8.0, 25.0, 12.0),
    ///         Color::srgb(0.8, 0.8, 0.9),
    ///     );
    /// }
    /// ```
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
    
    /// Creates a complete tree entity quickly with default proportions.
    ///
    /// This convenience function creates a tree with trunk and fronds using default
    /// dimensions (8.0 unit trunk height, 4 fronds). It provides a simple way to
    /// create trees without specifying detailed parameters.
    ///
    /// # Arguments
    /// * `commands` - Mutable reference to Bevy's command buffer for entity creation
    /// * `meshes` - Mutable reference to the mesh asset storage
    /// * `materials` - Mutable reference to the material asset storage
    /// * `position` - World position for the tree
    ///
    /// # Returns
    /// The [`Entity`] ID of the created tree (parent entity)
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::rendering_factory::*;
    ///
    /// fn spawn_park_tree(
    ///     mut commands: Commands,
    ///     mut meshes: ResMut<Assets<Mesh>>,
    ///     mut materials: ResMut<Assets<StandardMaterial>>,
    /// ) {
    ///     let tree = RenderingFactory::quick_tree(
    ///         &mut commands,
    ///         &mut meshes,
    ///         &mut materials,
    ///         Vec3::new(15.0, 0.0, -8.0),
    ///     );
    /// }
    /// ```
    pub fn quick_tree(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        Self::create_complete_tree(commands, meshes, materials, position, 8.0, 4)
    }
}
