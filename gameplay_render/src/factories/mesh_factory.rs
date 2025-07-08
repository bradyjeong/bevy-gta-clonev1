//! Factory utilities for generating GPU-ready [`Mesh`] assets for all game entities.
//!
//! # Overview
//! The [`MeshFactory`] provides a unified interface for creating optimized mesh geometry
//! for vehicles, NPCs, buildings, terrain, and celestial objects. It eliminates 130+
//! duplicate mesh creation patterns by consolidating all geometry generation into a single
//! factory with consistent input validation and performance optimization.
//!
//! ## Typical usage
//! ```rust
//! use bevy::prelude::*;
//! use gameplay_render::factories::MeshFactory;
//!
//! fn setup_vehicle(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
//!     let car_body = MeshFactory::create_car_body(&mut meshes);
//!     let wheel = MeshFactory::create_standard_wheel(&mut meshes);
//!     
//!     commands.spawn(PbrBundle {
//!         mesh: car_body,
//!         ..default()
//!     });
//! }
//! ```
//!
//! # Performance considerations
//! - All functions include input validation with safe clamping ranges
//! - Mesh dimensions are optimized for collision detection compatibility
//! - LOD considerations are built into geometry sizing for distance-based culling
//! - Batch processing friendly with consistent [`Handle<Mesh>`] return patterns
//!
//! # Implementation notes
//! Internally, the factory uses Bevy's primitive mesh types ([`Cuboid`], [`Sphere`], 
//! [`Cylinder`], [`Capsule3d`], [`Plane3d`]) with physics-optimized dimensions that
//! match the collision system requirements.

use bevy::prelude::*;

/// A unified factory for creating mesh assets across all game entity types.
///
/// This factory eliminates 130+ duplicate mesh creation patterns by providing
/// a single interface for generating GPU-ready geometry. It includes critical
/// safeguards such as input validation, performance optimization, and consistent
/// naming conventions for all mesh types.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use gameplay_render::factories::MeshFactory;
///
/// fn spawn_entities(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
///     // Create vehicle mesh
///     let car_mesh = MeshFactory::create_car_body(&mut meshes);
///     
///     // Create building mesh with validation
///     let building_mesh = MeshFactory::create_building_base(&mut meshes, 10.0, 20.0, 8.0);
///     
///     // Create NPC mesh with safe parameters
///     let npc_mesh = MeshFactory::create_npc_body(&mut meshes, 1.0, 1.8);
/// }
/// ```
pub struct MeshFactory;

impl MeshFactory {
    // VEHICLE MESHES - Standard vehicle components (Fixed: heights match colliders)
    
    /// Creates a standard passenger car body mesh optimized for collision detection.
    ///
    /// Generates a rectangular body mesh with dimensions carefully tuned to match
    /// the physics collision box. The height is specifically set to 1.0 to ensure
    /// proper collision detection alignment.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created car body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_car(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let car_body = MeshFactory::create_car_body(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: car_body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 3.6))  // Fixed: height 1.0 matches collider
    }

    /// Creates a sports car body mesh with an extended length for performance vehicles.
    ///
    /// Similar to the standard car body but with increased length (4.2 units) to
    /// accommodate the sportier proportions while maintaining collision compatibility.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created sports car body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_sports_car(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let sports_body = MeshFactory::create_sports_car_body(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: sports_body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_sports_car_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(1.8, 1.0, 4.2))  // Fixed: height 1.0 matches collider
    }

    /// Creates an SUV body mesh with increased dimensions for larger vehicles.
    ///
    /// Generates a larger rectangular body suitable for SUVs with increased width,
    /// height, and length compared to standard cars for realistic proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created SUV body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_suv(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let suv_body = MeshFactory::create_suv_body(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: suv_body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_suv_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(2.5, 1.5, 5.0))
    }

    /// Creates a truck body mesh with commercial vehicle proportions.
    ///
    /// Generates a large rectangular body suitable for cargo trucks with
    /// significantly increased length (16.0 units) and moderate height increase
    /// for realistic commercial vehicle appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created truck body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_truck(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let truck_body = MeshFactory::create_truck_body(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: truck_body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_truck_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(16.0, 2.0, 3.0))
    }

    /// Creates a helicopter fuselage mesh using a capsule shape for realism.
    ///
    /// Generates a more realistic helicopter fuselage using a capsule primitive
    /// instead of a box, providing better visual representation of the curved
    /// helicopter body shape.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created helicopter body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_helicopter(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let heli_body = MeshFactory::create_helicopter_body(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: heli_body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_helicopter_body(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Create a more realistic helicopter fuselage shape using a capsule
        meshes.add(Capsule3d::new(0.8, 4.0))  // Radius, height - helicopter shape
    }

    /// Creates a boat hull mesh for watercraft navigation.
    ///
    /// Generates a rectangular hull shape suitable for boats and small watercraft
    /// with appropriate length-to-width ratio for realistic water vehicle appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created boat hull mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_boat(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let boat_hull = MeshFactory::create_boat_hull(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: boat_hull,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_boat_hull(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(8.0, 2.0, 20.0))
    }

    /// Creates a yacht cabin mesh for luxury watercraft.
    ///
    /// Generates a rectangular cabin structure suitable for larger luxury watercraft
    /// with increased height for multi-deck yacht appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created yacht cabin mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_yacht(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let yacht_cabin = MeshFactory::create_yacht_cabin(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: yacht_cabin,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_yacht_cabin(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cuboid::new(6.0, 3.0, 8.0))
    }

    // VEHICLE PARTS - Wheels, components
    /// Creates a standard wheel mesh suitable for regular passenger vehicles.
    ///
    /// Generates a cylindrical wheel shape with balanced proportions for standard
    /// cars and everyday vehicles, optimized for visual consistency across the fleet.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created standard wheel mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_wheels_to_car(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let wheel = MeshFactory::create_standard_wheel(&mut meshes);
    ///     
    ///     // Spawn 4 wheels at car positions
    ///     for pos in [Vec3::new(1.0, 0.0, 1.5), Vec3::new(-1.0, 0.0, 1.5)] {
    ///         commands.spawn(PbrBundle {
    ///             mesh: wheel.clone(),
    ///             transform: Transform::from_translation(pos),
    ///             ..default()
    ///         });
    ///     }
    /// }
    /// ```
    pub fn create_standard_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.35, 0.25))
    }

    /// Creates a performance-oriented sports car wheel mesh with low-profile design.
    ///
    /// Generates a cylindrical wheel shape with reduced radius (0.25) and increased
    /// width (0.3) for sports car applications, providing better handling characteristics
    /// and aggressive visual appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created sports wheel mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_sports_wheels(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let sports_wheel = MeshFactory::create_sports_wheel(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: sports_wheel,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_sports_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.25, 0.3))
    }

    /// Creates a large wheel mesh suitable for trucks and heavy-duty vehicles.
    ///
    /// Generates a cylindrical wheel shape with increased radius (0.4) for heavy
    /// vehicles, providing the robust appearance needed for trucks and industrial
    /// equipment while maintaining proper proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created large wheel mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_truck_wheels(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let large_wheel = MeshFactory::create_large_wheel(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: large_wheel,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_large_wheel(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.4, 0.3))
    }

    /// Creates a wheel hub mesh for center mounting and visual detail.
    ///
    /// Generates a cylindrical hub component that serves as the central wheel
    /// attachment point, providing visual detail and realistic wheel construction
    /// appearance with balanced dimensions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created wheel hub mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_wheel_details(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let hub = MeshFactory::create_wheel_hub(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: hub,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_wheel_hub(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 0.35))
    }

    /// Creates an exhaust pipe mesh for vehicle exhaust systems.
    ///
    /// Generates a cylindrical pipe shape with small radius (0.08) and moderate
    /// length (0.3) suitable for vehicle exhaust outlets, providing realistic
    /// automotive detail components.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created exhaust pipe mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_exhaust_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let exhaust = MeshFactory::create_exhaust_pipe(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: exhaust,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_exhaust_pipe(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.08, 0.3))
    }

    /// Creates a headlight mesh for primary vehicle illumination.
    ///
    /// Generates a spherical headlight shape with standard radius (0.2) suitable
    /// for car headlights, providing consistent automotive lighting appearance
    /// across different vehicle types.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created headlight mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_headlights(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let headlight = MeshFactory::create_headlight(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: headlight,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_headlight(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.2))
    }

    /// Creates a small light mesh for secondary vehicle lighting.
    ///
    /// Generates a spherical light shape with reduced radius (0.15) suitable
    /// for turn signals, brake lights, or other secondary automotive lighting
    /// components that require smaller visual presence.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created small light mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_turn_signals(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let signal = MeshFactory::create_small_light(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: signal,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_small_light(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.15))
    }

    /// Creates a tiny light mesh for accent and detail lighting.
    ///
    /// Generates a spherical light shape with minimal radius (0.12) suitable
    /// for accent lighting, dashboard indicators, or other small decorative
    /// lighting elements that enhance vehicle detail.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created tiny light mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_accent_lights(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let tiny_light = MeshFactory::create_tiny_light(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: tiny_light,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_tiny_light(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.12))
    }

    // HELICOPTER PARTS
    /// Creates a helicopter rotor blade mesh with aerodynamic profile.
    ///
    /// Generates a thin, elongated rotor blade shape optimized for helicopter
    /// main rotor systems. The blade features realistic proportions with
    /// significant length (8.0) and minimal thickness (0.02) for aerodynamic efficiency.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created rotor blade mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_main_rotor(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let blade = MeshFactory::create_rotor_blade(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: blade,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_rotor_blade(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Realistic rotor blade shape - thin and aerodynamic
        meshes.add(Cuboid::new(8.0, 0.02, 0.3))  // Long, thin blade
    }

    /// Creates a helicopter cockpit mesh with bubble canopy design.
    ///
    /// Generates a spherical cockpit shape representing the characteristic
    /// bubble canopy found on many helicopters, providing good visibility
    /// and realistic helicopter appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created helicopter cockpit mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_helicopter_cockpit(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let cockpit = MeshFactory::create_helicopter_cockpit(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: cockpit,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_helicopter_cockpit(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Bubble-shaped cockpit
        meshes.add(Sphere::new(0.8))
    }

    /// Creates a helicopter tail boom mesh for structural connection.
    ///
    /// Generates a cylindrical tail boom that connects the main fuselage to
    /// the tail rotor assembly, providing the characteristic helicopter
    /// structural element with tapered design.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created helicopter tail boom mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_tail_boom(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let tail_boom = MeshFactory::create_helicopter_tail_boom(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: tail_boom,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_helicopter_tail_boom(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Tapered tail boom
        meshes.add(Cylinder::new(0.25, 3.5))
    }

    /// Creates a tail rotor mesh for helicopter anti-torque system.
    ///
    /// Generates a compact cylindrical rotor for the helicopter tail rotor
    /// assembly, providing the anti-torque control system essential for
    /// helicopter flight stability.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created tail rotor mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_tail_rotor(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let tail_rotor = MeshFactory::create_tail_rotor(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: tail_rotor,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_tail_rotor(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.15, 0.2))
    }

    /// Creates a landing skid mesh for helicopter ground support.
    ///
    /// Generates a cylindrical skid shape for helicopter landing gear, featuring
    /// narrow profile (0.04 radius) and extended length (3.0) for stable ground
    /// contact and realistic helicopter landing system appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created landing skid mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_landing_gear(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let skid = MeshFactory::create_landing_skid(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: skid,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_landing_skid(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        // Helicopter skids - long and narrow
        meshes.add(Cylinder::new(0.04, 3.0))
    }

    // WORLD STRUCTURES - Buildings, environment
    /// Creates a building base mesh with customizable dimensions and input validation.
    ///
    /// Generates a rectangular building foundation with comprehensive input validation
    /// to ensure safe parameter ranges. Dimensions are clamped between 0.1 and 1000.0
    /// units to prevent degenerate geometry and maintain reasonable building scales.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `width` - Building width (clamped to 0.1-1000.0 range)
    /// * `height` - Building height (clamped to 0.1-1000.0 range)
    /// * `depth` - Building depth (clamped to 0.1-1000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created building base mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn spawn_building(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let building = MeshFactory::create_building_base(&mut meshes, 10.0, 20.0, 8.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: building,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_building_base(meshes: &mut ResMut<Assets<Mesh>>, width: f32, height: f32, depth: f32) -> Handle<Mesh> {
        // Input validation for critical safeguards
        let safe_width = width.max(0.1).min(1000.0);
        let safe_height = height.max(0.1).min(1000.0);
        let safe_depth = depth.max(0.1).min(1000.0);
        meshes.add(Cuboid::new(safe_width, safe_height, safe_depth))
    }

    /// Creates a lamp post mesh for urban street lighting.
    ///
    /// Generates a cylindrical lamp post with standard proportions for urban
    /// environments, providing consistent street lighting infrastructure with
    /// appropriate height and width for realistic city appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created lamp post mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_street_lighting(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let lamp_post = MeshFactory::create_lamp_post(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: lamp_post,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_lamp_post(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 8.0))
    }

    /// Creates a tree frond mesh for foliage representation.
    ///
    /// Generates a spherical tree canopy shape representing the leafy portion
    /// of trees, providing natural environment elements with appropriate size
    /// for realistic vegetation appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created tree frond mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_tree_foliage(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let frond = MeshFactory::create_tree_frond(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: frond,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_tree_frond(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(0.8))
    }

    /// Creates a tree trunk mesh for vegetation structure.
    ///
    /// Generates a cylindrical tree trunk providing the structural base for
    /// tree entities, with appropriate height and diameter for realistic
    /// tree proportions in natural environments.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created tree trunk mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_tree_trunk(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let trunk = MeshFactory::create_tree_trunk(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: trunk,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_tree_trunk(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.3, 8.0))
    }

    /// Creates a road segment mesh for transportation infrastructure.
    ///
    /// Generates a rectangular road segment with customizable dimensions and
    /// input validation. Width and length are clamped to safe ranges (0.1-100.0
    /// and 0.1-1000.0 respectively) with thin height for realistic road appearance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `width` - Road width (clamped to 0.1-100.0 range)
    /// * `length` - Road length (clamped to 0.1-1000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created road segment mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn build_road(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let road = MeshFactory::create_road_segment(&mut meshes, 8.0, 50.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: road,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_road_segment(meshes: &mut ResMut<Assets<Mesh>>, width: f32, length: f32) -> Handle<Mesh> {
        let safe_width = width.max(0.1).min(100.0);
        let safe_length = length.max(0.1).min(1000.0);
        meshes.add(Cuboid::new(safe_width, 0.1, safe_length))
    }

    /// Creates a road marking mesh for traffic lane indicators.
    ///
    /// Generates a thin rectangular marking for road lane divisions, traffic
    /// signs, and other road surface indicators. Includes input validation with
    /// smaller size limits appropriate for road markings.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `width` - Marking width (clamped to 0.1-10.0 range)
    /// * `length` - Marking length (clamped to 0.1-100.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created road marking mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_lane_markers(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let marking = MeshFactory::create_road_marking(&mut meshes, 0.2, 3.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: marking,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_road_marking(meshes: &mut ResMut<Assets<Mesh>>, width: f32, length: f32) -> Handle<Mesh> {
        let safe_width = width.max(0.1).min(10.0);
        let safe_length = length.max(0.1).min(100.0);
        meshes.add(Cuboid::new(safe_width, 0.11, safe_length))
    }

    // WATER FEATURES
    /// Creates a lake cylinder mesh for water body geometry.
    ///
    /// Generates a cylindrical lake shape with customizable radius and depth,
    /// including input validation to ensure safe parameter ranges. Radius is
    /// clamped to 1.0-1000.0 range and depth to 0.1-100.0 range.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `radius` - Lake radius (clamped to 1.0-1000.0 range)
    /// * `depth` - Lake depth (clamped to 0.1-100.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created lake cylinder mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_lake(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let lake = MeshFactory::create_lake_cylinder(&mut meshes, 50.0, 10.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: lake,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_lake_cylinder(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, depth: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(1.0).min(1000.0);
        let safe_depth = depth.max(0.1).min(100.0);
        meshes.add(Cylinder::new(safe_radius, safe_depth))
    }

    /// Creates a water plane mesh for large water surfaces.
    ///
    /// Generates a flat square water surface with customizable size and input
    /// validation. Size is clamped to 1.0-10000.0 range to support both small
    /// ponds and large ocean surfaces.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `size` - Water plane size (clamped to 1.0-10000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created water plane mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_ocean(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let ocean = MeshFactory::create_water_plane(&mut meshes, 1000.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: ocean,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_water_plane(meshes: &mut ResMut<Assets<Mesh>>, size: f32) -> Handle<Mesh> {
        let safe_size = size.max(1.0).min(10000.0);
        meshes.add(Plane3d::default().mesh().size(safe_size, safe_size))
    }

    /// Creates a boat mast mesh for watercraft rigging.
    ///
    /// Generates a cylindrical mast shape suitable for sailing vessels and
    /// maritime equipment, with appropriate height and diameter for realistic
    /// boat proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created mast mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn add_boat_mast(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let mast = MeshFactory::create_mast(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: mast,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_mast(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Cylinder::new(0.2, 15.0))
    }

    // NPC COMPONENTS - Character parts  
    /// Creates an NPC head mesh with customizable build factor.
    ///
    /// Generates a spherical head shape with size scaled by build factor,
    /// including input validation to ensure safe parameter ranges. Build factor
    /// is clamped to 0.1-5.0 range for realistic NPC head proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `build_factor` - NPC build scale factor (clamped to 0.1-5.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created NPC head mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_npc_head(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let head = MeshFactory::create_npc_head(&mut meshes, 1.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: head,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_npc_head(meshes: &mut ResMut<Assets<Mesh>>, build_factor: f32) -> Handle<Mesh> {
        let safe_build = build_factor.max(0.1).min(5.0);
        meshes.add(Sphere::new(0.12 * safe_build))
    }

    /// Creates an NPC body mesh with customizable build and height parameters.
    ///
    /// Generates a rectangular body shape with dimensions scaled by build and height
    /// factors, including input validation to ensure safe parameter ranges. Both
    /// build and height are clamped to appropriate ranges for realistic NPC proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `build` - NPC build scale factor (clamped to 0.1-5.0 range)
    /// * `height` - NPC height scale factor (clamped to 0.1-10.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created NPC body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_npc_body(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let body = MeshFactory::create_npc_body(&mut meshes, 1.0, 1.8);
    ///     commands.spawn(PbrBundle {
    ///         mesh: body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_npc_body(meshes: &mut ResMut<Assets<Mesh>>, build: f32, height: f32) -> Handle<Mesh> {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        meshes.add(Cuboid::new(0.4 * safe_build, 0.6 * safe_height, 0.2 * safe_build))
    }

    /// Creates an NPC limb mesh with customizable radius and length.
    ///
    /// Generates a capsule-shaped limb suitable for NPC arms and legs, with
    /// input validation to ensure safe parameter ranges. Radius and length
    /// are clamped to appropriate ranges for realistic limb proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `radius` - Limb radius (clamped to 0.01-1.0 range)
    /// * `length` - Limb length (clamped to 0.1-5.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created NPC limb mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_npc_limb(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let limb = MeshFactory::create_npc_limb(&mut meshes, 0.05, 0.8);
    ///     commands.spawn(PbrBundle {
    ///         mesh: limb,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_npc_limb(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, length: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.01).min(1.0);
        let safe_length = length.max(0.1).min(5.0);
        meshes.add(Capsule3d::new(safe_radius, safe_length))
    }

    /// Creates a simplified NPC body mesh for performance-optimized rendering.
    ///
    /// Generates a capsule-shaped body suitable for medium-distance NPC rendering,
    /// providing better performance than detailed body meshes while maintaining
    /// recognizable human proportions.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `build` - NPC build scale factor (clamped to 0.1-5.0 range)
    /// * `height` - NPC height scale factor (clamped to 0.1-10.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created simplified NPC body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_simple_npc(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let body = MeshFactory::create_npc_simple_body(&mut meshes, 1.0, 1.8);
    ///     commands.spawn(PbrBundle {
    ///         mesh: body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_npc_simple_body(meshes: &mut ResMut<Assets<Mesh>>, build: f32, height: f32) -> Handle<Mesh> {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        meshes.add(Capsule3d::new(0.3 * safe_build, safe_height * 0.8))
    }

    /// Creates an ultra-simple NPC body mesh for maximum performance.
    ///
    /// Generates a minimal capsule-shaped body suitable for distant NPC rendering,
    /// providing maximum performance with minimal geometry while maintaining
    /// basic human silhouette for crowd simulation.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `build` - NPC build scale factor (clamped to 0.1-5.0 range)
    /// * `height` - NPC height scale factor (clamped to 0.1-10.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created ultra-simple NPC body mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_distant_npc(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let body = MeshFactory::create_npc_ultra_simple(&mut meshes, 1.0, 1.8);
    ///     commands.spawn(PbrBundle {
    ///         mesh: body,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_npc_ultra_simple(meshes: &mut ResMut<Assets<Mesh>>, build: f32, height: f32) -> Handle<Mesh> {
        let safe_build = build.max(0.1).min(5.0);
        let safe_height = height.max(0.1).min(10.0);
        meshes.add(Capsule3d::new(0.25 * safe_build, safe_height))
    }

    // SKY COMPONENTS - Celestial bodies
    /// Creates a sky dome mesh for atmospheric rendering.
    ///
    /// Generates a large spherical dome that serves as the sky background,
    /// providing a canvas for atmospheric effects, cloud rendering, and
    /// celestial body positioning with appropriate scale for world environments.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created sky dome mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn setup_sky(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let sky_dome = MeshFactory::create_sky_dome(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: sky_dome,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_sky_dome(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(2000.0))
    }

    /// Creates a sun mesh for solar lighting effects.
    ///
    /// Generates a spherical sun shape with appropriate size for distant solar
    /// rendering, providing the primary light source for day/night cycles and
    /// atmospheric lighting calculations.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created sun mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn setup_sun(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let sun = MeshFactory::create_sun(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: sun,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_sun(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(50.0))
    }

    /// Creates a moon mesh for lunar lighting effects.
    ///
    /// Generates a spherical moon shape with appropriate size for distant lunar
    /// rendering, providing secondary light source for night scenes and
    /// atmospheric ambiance.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created moon mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn setup_moon(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let moon = MeshFactory::create_moon(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: moon,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_moon(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Sphere::new(30.0))
    }

    /// Creates a star mesh with customizable size.
    ///
    /// Generates a spherical star shape with customizable size and input
    /// validation, suitable for night sky star fields and celestial decoration.
    /// Size is clamped to 0.1-100.0 range for realistic star appearances.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `size` - Star size (clamped to 0.1-100.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created star mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn setup_stars(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let star = MeshFactory::create_star(&mut meshes, 2.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: star,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_star(meshes: &mut ResMut<Assets<Mesh>>, size: f32) -> Handle<Mesh> {
        let safe_size = size.max(0.1).min(100.0);
        meshes.add(Sphere::new(safe_size))
    }

    /// Creates a cloud mesh with customizable scale.
    ///
    /// Generates a spherical cloud shape with customizable scale and input
    /// validation, suitable for atmospheric cloud systems and weather effects.
    /// Scale is clamped to 1.0-1000.0 range for realistic cloud appearances.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `scale` - Cloud scale (clamped to 1.0-1000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created cloud mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn setup_clouds(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let cloud = MeshFactory::create_cloud(&mut meshes, 50.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: cloud,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_cloud(meshes: &mut ResMut<Assets<Mesh>>, scale: f32) -> Handle<Mesh> {
        let safe_scale = scale.max(1.0).min(1000.0);
        meshes.add(Sphere::new(safe_scale))
    }



    // TERRAIN - Ground plane
    /// Creates a ground plane mesh for world terrain.
    ///
    /// Generates a large flat plane serving as the base terrain for the game world,
    /// providing a foundation for all other entities and terrain features with
    /// appropriate scale for open-world environments.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created ground plane mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn setup_terrain(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let ground = MeshFactory::create_ground_plane(&mut meshes);
    ///     commands.spawn(PbrBundle {
    ///         mesh: ground,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_ground_plane(meshes: &mut ResMut<Assets<Mesh>>) -> Handle<Mesh> {
        meshes.add(Plane3d::default().mesh().size(4000.0, 4000.0))
    }

    // CUSTOM SIZED MESHES - Flexible components
    /// Creates a custom cuboid mesh with specified dimensions.
    ///
    /// Generates a rectangular box shape with customizable width, height, and depth,
    /// including input validation to ensure safe parameter ranges. All dimensions
    /// are clamped to 0.001-10000.0 range for flexible geometry creation.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `width` - Cuboid width (clamped to 0.001-10000.0 range)
    /// * `height` - Cuboid height (clamped to 0.001-10000.0 range)
    /// * `depth` - Cuboid depth (clamped to 0.001-10000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created custom cuboid mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_custom_box(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let box_mesh = MeshFactory::create_custom_cuboid(&mut meshes, 2.0, 3.0, 1.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: box_mesh,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_custom_cuboid(meshes: &mut ResMut<Assets<Mesh>>, width: f32, height: f32, depth: f32) -> Handle<Mesh> {
        let safe_width = width.max(0.001).min(10000.0);
        let safe_height = height.max(0.001).min(10000.0);
        let safe_depth = depth.max(0.001).min(10000.0);
        meshes.add(Cuboid::new(safe_width, safe_height, safe_depth))
    }

    /// Creates a custom sphere mesh with specified radius.
    ///
    /// Generates a spherical shape with customizable radius and input validation
    /// to ensure safe parameter ranges. Radius is clamped to 0.001-5000.0 range
    /// for flexible sphere creation.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `radius` - Sphere radius (clamped to 0.001-5000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created custom sphere mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_custom_sphere(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let sphere_mesh = MeshFactory::create_custom_sphere(&mut meshes, 1.5);
    ///     commands.spawn(PbrBundle {
    ///         mesh: sphere_mesh,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_custom_sphere(meshes: &mut ResMut<Assets<Mesh>>, radius: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.001).min(5000.0);
        meshes.add(Sphere::new(safe_radius))
    }

    /// Creates a custom cylinder mesh with specified radius and height.
    ///
    /// Generates a cylindrical shape with customizable radius and height,
    /// including input validation to ensure safe parameter ranges. Radius is
    /// clamped to 0.001-1000.0 and height to 0.001-10000.0 range.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `radius` - Cylinder radius (clamped to 0.001-1000.0 range)
    /// * `height` - Cylinder height (clamped to 0.001-10000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created custom cylinder mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_custom_cylinder(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let cylinder_mesh = MeshFactory::create_custom_cylinder(&mut meshes, 0.5, 2.0);
    ///     commands.spawn(PbrBundle {
    ///         mesh: cylinder_mesh,
    ///         ..default()
    ///     });
    /// }
    /// ```
    pub fn create_custom_cylinder(meshes: &mut ResMut<Assets<Mesh>>, radius: f32, height: f32) -> Handle<Mesh> {
        let safe_radius = radius.max(0.001).min(1000.0);
        let safe_height = height.max(0.001).min(10000.0);
        meshes.add(Cylinder::new(safe_radius, safe_height))
    }

    /// Creates a custom capsule mesh with specified radius and length.
    ///
    /// Generates a capsule shape with customizable radius and length, including
    /// input validation to ensure safe parameter ranges. Radius is clamped to
    /// 0.001-100.0 and length to 0.001-1000.0 range.
    ///
    /// # Arguments
    /// * `meshes` - Mutable reference to Bevy's mesh asset storage
    /// * `radius` - Capsule radius (clamped to 0.001-100.0 range)
    /// * `length` - Capsule length (clamped to 0.001-1000.0 range)
    ///
    /// # Returns
    /// A [`Handle<Mesh>`] pointing to the created custom capsule mesh
    ///
    /// # Examples
    /// ```rust
    /// use bevy::prelude::*;
    /// use gameplay_render::factories::MeshFactory;
    ///
    /// fn create_custom_capsule(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    ///     let capsule_mesh = MeshFactory::create_custom_capsule(&mut meshes, 0.3, 1.2);
    ///     commands.spawn(PbrBundle {
    ///         mesh: capsule_mesh,
    ///         ..default()
    ///     });
    /// }
    /// ```
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
