//! Generic bundle factory system for creating game entities with type safety.
//!
//! # Overview
//! This module provides a unified, trait-based approach to creating entity bundles for
//! the Bevy ECS system. It eliminates code duplication by centralizing bundle creation
//! logic and enforces configuration-based validation for all entity types.
//!
//! The system supports vehicles, NPCs, buildings, physics objects, and various dynamic
//! content types. All bundles are created through validated specifications that ensure
//! proper physics bounds, component composition, and configuration compliance.
//!
//! ## Typical usage
//! ```rust
//! use bevy::prelude::*;
//! use game_core::config::GameConfig;
//! use gameplay_render::factories::generic_bundle::*;
//!
//! fn spawn_vehicle(
//!     mut commands: Commands,
//!     config: Res<GameConfig>,
//! ) {
//!     let vehicle_bundle = GenericBundleFactory::vehicle(
//!         VehicleType::BasicCar,
//!         Vec3::new(0.0, 0.0, 0.0),
//!         Color::RED,
//!         &config,
//!     ).expect("Valid vehicle configuration");
//!     
//!     commands.spawn(vehicle_bundle);
//! }
//! ```
//!
//! # Implementation notes
//! The factory system uses the [`BundleSpec`] trait to define creation and validation
//! behavior for each bundle type. This ensures consistent parameter validation and
//! configuration integration across all entity types.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::config::*;
use game_core::components::*;
use gameplay_sim::components::MovementTracker;
use game_core::bundles::*;
use game_core::components::UnifiedChunkEntity;
use game_core::components::UnifiedCullable;

/// Type alias for backwards compatibility with legacy NPC behavior systems.
///
/// This alias maps the old [`NPCBehavior`] name to the new [`NPCBehaviorComponent`]
/// struct, allowing existing code to continue working while the codebase migrates
/// to the unified component naming conventions.
pub type NPCBehavior = NPCBehaviorComponent;

/// Particle effect types for the unified visual effects system.
///
/// This enum defines the different particle effects that can be spawned and managed
/// by the game's particle system. Each variant represents a distinct visual effect
/// with specific rendering properties and behavioral characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleEffectType {
    /// Vehicle exhaust emission particles
    Exhaust,
    /// General smoke particles for various effects
    Smoke,
    /// Fire particles for combustion effects
    Fire,
    /// Water particles for liquid effects
    Water,
    /// Dust particles for environmental effects
    Dust,
    /// Explosion particles for destructive effects
    Explosion,
    /// Electrical spark particles
    Spark,
}

/// Core trait for bundle specifications with type safety and validation.
///
/// This trait defines the interface for all bundle specifications in the generic
/// bundle system. It provides compile-time type safety and runtime validation
/// for entity creation, eliminating over 200 duplicate bundle patterns.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use game_core::config::GameConfig;
/// use gameplay_render::factories::generic_bundle::*;
///
/// # fn example() -> Result<(), BundleError> {
/// let config = GameConfig::default();
/// let spec = VehicleBundleSpec {
///     vehicle_type: VehicleType::BasicCar,
///     position: Vec3::ZERO,
///     color: Color::RED,
///     max_speed_override: None,
///     mass_override: None,
///     include_physics: true,
///     include_collision: true,
///     include_visibility: true,
/// };
///
/// spec.validate(&config)?;
/// let bundle = spec.create_bundle(&config);
/// # Ok(())
/// # }
/// ```
pub trait BundleSpec: Send + Sync + 'static {
    /// The Bevy [`Bundle`] type that this specification creates
    type Bundle: Bundle;
    
    /// Creates a bundle from this specification using the provided configuration.
    ///
    /// This method assumes the specification has been validated and will clamp
    /// values to safe ranges as defined by the configuration.
    ///
    /// # Arguments
    /// * `config` - Game configuration containing validation bounds and defaults
    ///
    /// # Returns
    /// A fully initialized bundle ready for spawning
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle;
    
    /// Validates the specification parameters against configuration bounds.
    ///
    /// This method should be called before [`create_bundle`] to ensure all
    /// parameters are within valid ranges and meet safety requirements.
    ///
    /// # Arguments
    /// * `config` - Game configuration containing validation bounds
    ///
    /// # Returns
    /// `Ok(())` if validation passes, or a descriptive [`BundleError`] if not
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let spec = VehicleBundleSpec {
    ///     vehicle_type: VehicleType::BasicCar,
    ///     position: Vec3::new(0.0, 0.0, 0.0),
    ///     color: Color::RED,
    ///     max_speed_override: None,
    ///     mass_override: None,
    ///     include_physics: true,
    ///     include_collision: true,
    ///     include_visibility: true,
    /// };
    ///
    /// assert!(spec.validate(&config).is_ok());
    /// ```
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError>;
}

/// Bundle creation errors with detailed context information.
///
/// This enum represents all possible validation errors that can occur during
/// bundle creation. Each variant contains specific details about what went
/// wrong, making debugging and error handling more effective.
#[derive(Debug)]
pub enum BundleError {
    /// Position coordinates exceed world bounds
    PositionOutOfBounds { 
        /// The invalid position that was provided
        position: Vec3, 
        /// Maximum allowed coordinate value
        max_coord: f32 
    },
    /// Entity size is outside valid range
    InvalidSize { 
        /// The invalid size that was provided
        size: Vec3, 
        /// Minimum allowed size value
        min_size: f32, 
        /// Maximum allowed size value
        max_size: f32 
    },
    /// Entity mass is outside valid range for physics simulation
    InvalidMass { 
        /// The invalid mass that was provided
        mass: f32, 
        /// Minimum allowed mass value
        min_mass: f32, 
        /// Maximum allowed mass value
        max_mass: f32 
    },
    /// Velocity exceeds maximum allowed value
    InvalidVelocity { 
        /// The invalid velocity that was provided
        velocity: f32, 
        /// Maximum allowed velocity value
        max_velocity: f32 
    },
    /// Unknown or unsupported entity type
    InvalidEntityType { 
        /// The invalid entity type string
        entity_type: String 
    },
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::PositionOutOfBounds { position, max_coord } => {
                write!(f, "Position {:?} is out of bounds (max: {})", position, max_coord)
            }
            BundleError::InvalidSize { size, min_size, max_size } => {
                write!(f, "Size {:?} is invalid (min: {}, max: {})", size, min_size, max_size)
            }
            BundleError::InvalidMass { mass, min_mass, max_mass } => {
                write!(f, "Mass {} is invalid (min: {}, max: {})", mass, min_mass, max_mass)
            }
            BundleError::InvalidVelocity { velocity, max_velocity } => {
                write!(f, "Velocity {} exceeds maximum (max: {})", velocity, max_velocity)
            }
            BundleError::InvalidEntityType { entity_type } => {
                write!(f, "Invalid entity type: {}", entity_type)
            }
        }
    }
}

impl std::error::Error for BundleError {}

/// A specification for creating vehicle bundles with configurable parameters.
///
/// This struct defines all the parameters needed to create a complete vehicle
/// entity with physics, rendering, and behavioral components. It supports
/// various vehicle types from cars to aircraft, with optional parameter
/// overrides for customization.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use game_core::config::GameConfig;
/// use gameplay_render::factories::generic_bundle::*;
///
/// # fn example() -> Result<(), BundleError> {
/// let config = GameConfig::default();
/// let spec = VehicleBundleSpec {
///     vehicle_type: VehicleType::BasicCar,
///     position: Vec3::new(100.0, 0.0, 200.0),
///     color: Color::BLUE,
///     max_speed_override: Some(120.0),
///     mass_override: None,
///     include_physics: true,
///     include_collision: true,
///     include_visibility: true,
/// };
///
/// let bundle = GenericBundleFactory::create(spec, &config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct VehicleBundleSpec {
    /// The type of vehicle to create (car, helicopter, etc.)
    pub vehicle_type: VehicleType,
    /// World position where the vehicle will be spawned
    pub position: Vec3,
    /// Primary color for the vehicle rendering
    pub color: Color,
    /// Optional override for maximum speed (uses config default if None)
    pub max_speed_override: Option<f32>,
    /// Optional override for vehicle mass (uses config default if None)
    pub mass_override: Option<f32>,
    /// Whether to include physics components for movement
    pub include_physics: bool,
    /// Whether to include collision detection
    pub include_collision: bool,
    /// Whether to include visibility and culling components
    pub include_visibility: bool,
}

impl BundleSpec for VehicleBundleSpec {
    type Bundle = VehicleBundle;
    
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle {
        // Get vehicle type configuration
        let vehicle_config = match self.vehicle_type {
            VehicleType::BasicCar => &config.vehicles.basic_car,
            VehicleType::SuperCar => &config.vehicles.super_car,
            VehicleType::Helicopter => &config.vehicles.helicopter,
            VehicleType::F16 => &config.vehicles.f16,
            VehicleType::Car => &config.vehicles.basic_car,
        };
        
        // Apply overrides with validation
        let max_speed = self.max_speed_override
            .unwrap_or(vehicle_config.max_speed)
            .clamp(10.0, config.physics.max_velocity);
            
        let mass = self.mass_override
            .unwrap_or(vehicle_config.mass)
            .clamp(config.physics.min_mass, config.physics.max_mass);
        
        // Create validated transform
        let transform = Transform::from_translation(self.position.clamp(
            Vec3::splat(config.physics.min_world_coord),
            Vec3::splat(config.physics.max_world_coord)
        ));
        
        // Build bundle with configuration
        VehicleBundle {
            vehicle_type: self.vehicle_type,
            vehicle_state: VehicleState {
                color: self.color,
                max_speed,
                acceleration: vehicle_config.acceleration,
                damage: 0.0,
                fuel: 100.0,
                current_lod: VehicleLOD::Full,
                last_lod_check: 0.0,
                vehicle_type: self.vehicle_type,
            },
            transform,
            visibility: Visibility::Visible,
            rigid_body: if self.include_physics { RigidBody::Dynamic } else { RigidBody::Fixed },
            collider: if self.include_collision {
                Collider::cuboid(
                    vehicle_config.collider_size.x / 2.0,
                    vehicle_config.collider_size.y / 2.0,
                    vehicle_config.collider_size.z / 2.0,
                )
            } else {
                Collider::ball(0.1) // Minimal collider
            },
            collision_groups: CollisionGroups::new(config.physics.vehicle_group(), Group::ALL),
            additional_mass: AdditionalMassProperties::Mass(mass),
            velocity: Velocity::zero(),
            damping: Damping { 
                linear_damping: vehicle_config.linear_damping, 
                angular_damping: vehicle_config.angular_damping 
            },
            cullable: if self.include_visibility {
                UnifiedCullable::vehicle()
            } else {
                UnifiedCullable::vehicle()
            },
        }
    }
    
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError> {
        // Validate position bounds
        if self.position.x.abs() > config.physics.max_world_coord ||
           self.position.z.abs() > config.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position: self.position,
                max_coord: config.physics.max_world_coord,
            });
        }
        
        // Validate overrides if present
        if let Some(max_speed) = self.max_speed_override {
            if max_speed <= 0.0 || max_speed > config.physics.max_velocity {
                return Err(BundleError::InvalidVelocity {
                    velocity: max_speed,
                    max_velocity: config.physics.max_velocity,
                });
            }
        }
        
        if let Some(mass) = self.mass_override {
            if mass < config.physics.min_mass || mass > config.physics.max_mass {
                return Err(BundleError::InvalidMass {
                    mass,
                    min_mass: config.physics.min_mass,
                    max_mass: config.physics.max_mass,
                });
            }
        }
        
        Ok(())
    }
}

/// A specification for creating NPC (Non-Player Character) bundles.
///
/// This struct defines the parameters needed to create a complete NPC entity
/// with physics, AI behavior, and appearance components. NPCs can be civilian
/// pedestrians, traffic participants, or other autonomous characters.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use game_core::config::GameConfig;
/// use gameplay_render::factories::generic_bundle::*;
///
/// # fn example() -> Result<(), BundleError> {
/// let config = GameConfig::default();
/// let spec = NPCBundleSpec {
///     position: Vec3::new(50.0, 0.0, 100.0),
///     height: 1.75,
///     build: 1.0,
///     appearance: NPCAppearance::default(),
///     behavior: None,
///     include_physics: true,
///     include_ai: true,
/// };
///
/// let bundle = GenericBundleFactory::create(spec, &config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NPCBundleSpec {
    /// World position where the NPC will be spawned
    pub position: Vec3,
    /// Height of the NPC in meters (affects collision capsule)
    pub height: f32,
    /// Build factor affecting mass and collision size
    pub build: f32,
    /// Visual appearance configuration for the NPC
    pub appearance: NPCAppearance,
    /// Optional custom behavior component (uses default if None)
    pub behavior: Option<NPCBehavior>,
    /// Whether to include physics components for movement
    pub include_physics: bool,
    /// Whether to include AI behavior components
    #[allow(dead_code)]
    pub include_ai: bool,
}

impl BundleSpec for NPCBundleSpec {
    type Bundle = NPCBundle;
    
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle {
        // Validate and clamp parameters
        let _height = self.height.clamp(0.5, 3.0);
        let build = self.build.clamp(0.3, 2.0);
        
        let transform = Transform::from_translation(self.position.clamp(
            Vec3::splat(config.physics.min_world_coord),
            Vec3::splat(config.physics.max_world_coord)
        ));
        
        NPCBundle {
            npc_marker: NPCState {
                npc_type: NPCType::Civilian,
                appearance: self.appearance,
                behavior: NPCBehaviorType::Wandering,
                target_position: self.position,
                speed: config.npc.walk_speed,
                current_lod: NPCLOD::Full,
                last_lod_check: 0.0,
            },
            npc_behavior: self.behavior.unwrap_or(NPCBehavior {
                speed: config.npc.walk_speed,
                last_update: 0.0,
                update_interval: config.npc.update_intervals.close_interval,
            }),
            npc_appearance: self.appearance,
            movement_controller: MovementController {
                current_speed: 0.0,
                max_speed: config.npc.walk_speed,
                stamina: 100.0,
            },
            transform,
            visibility: Visibility::Inherited,
            rigid_body: if self.include_physics { RigidBody::Dynamic } else { RigidBody::Fixed },
            collider: Collider::capsule_y(config.npc.capsule_height, config.npc.capsule_radius),
            collision_groups: CollisionGroups::new(config.physics.character_group(), Group::ALL),
            additional_mass: AdditionalMassProperties::Mass(70.0 * build),
            velocity: Velocity::zero(),
            cullable: UnifiedCullable::npc(),
            movement_tracker: MovementTracker::new(self.position, 8.0), // Track NPC movement with 8m threshold
        }
    }
    
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError> {
        // Validate position bounds
        if self.position.x.abs() > config.physics.max_world_coord ||
           self.position.z.abs() > config.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position: self.position,
                max_coord: config.physics.max_world_coord,
            });
        }
        
        // Validate physical parameters
        if self.height < 0.5 || self.height > 3.0 {
            return Err(BundleError::InvalidSize {
                size: Vec3::new(self.build, self.height, self.build),
                min_size: 0.5,
                max_size: 3.0,
            });
        }
        
        if self.build < 0.3 || self.build > 2.0 {
            return Err(BundleError::InvalidSize {
                size: Vec3::new(self.build, self.height, self.build),
                min_size: 0.3,
                max_size: 2.0,
            });
        }
        
        Ok(())
    }
}

/// A specification for creating building bundles in the game world.
///
/// This struct defines the parameters needed to create static building entities
/// with collision, rendering, and occupancy tracking components. Buildings are
/// immobile structures that form the urban environment.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use game_core::config::GameConfig;
/// use gameplay_render::factories::generic_bundle::*;
///
/// # fn example() -> Result<(), BundleError> {
/// let config = GameConfig::default();
/// let spec = BuildingBundleSpec {
///     position: Vec3::new(0.0, 0.0, 0.0),
///     size: Vec3::new(20.0, 50.0, 15.0),
///     building_type: BuildingType::Residential,
///     color: Color::GRAY,
///     include_collision: true,
///     lod_level: 0,
/// };
///
/// let bundle = GenericBundleFactory::create(spec, &config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BuildingBundleSpec {
    /// World position where the building will be placed
    pub position: Vec3,
    /// Dimensions of the building (width, height, depth)
    pub size: Vec3,
    /// Type of building (residential, commercial, etc.)
    pub building_type: BuildingType,
    /// Primary color for the building rendering
    #[allow(dead_code)]
    pub color: Color,
    /// Whether to include collision detection for the building
    pub include_collision: bool,
    /// Level of detail for rendering optimization
    #[allow(dead_code)]
    pub lod_level: u8,
}

impl BundleSpec for BuildingBundleSpec {
    type Bundle = BuildingBundle;
    
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle {
        // Validate and clamp size
        let size = Vec3::new(
            self.size.x.clamp(1.0, config.physics.max_collider_size),
            self.size.y.clamp(1.0, config.physics.max_collider_size),
            self.size.z.clamp(1.0, config.physics.max_collider_size),
        );
        
        let transform = Transform::from_translation(self.position.clamp(
            Vec3::splat(config.physics.min_world_coord),
            Vec3::splat(config.physics.max_world_coord)
        ));
        
        BuildingBundle {
            building_marker: Building {
                building_type: self.building_type,
                height: size.y,
                scale: size,
                current_occupants: Some(0),
                max_occupants: Some((size.y * size.x * 0.1) as u32),
                spawn_time: Some(0.0),
            },
            transform,
            visibility: Visibility::Inherited,
            rigid_body: RigidBody::Fixed,
            collider: if self.include_collision {
                Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)
            } else {
                Collider::ball(0.1) // Minimal collider for LOD
            },
            collision_groups: CollisionGroups::new(config.physics.static_group(), Group::ALL),
            cullable: UnifiedCullable::building(),
        }
    }
    
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError> {
        // Validate position bounds
        if self.position.x.abs() > config.physics.max_world_coord ||
           self.position.z.abs() > config.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position: self.position,
                max_coord: config.physics.max_world_coord,
            });
        }
        
        // Validate size bounds
        if self.size.x < config.physics.min_collider_size || self.size.x > config.physics.max_collider_size ||
           self.size.y < config.physics.min_collider_size || self.size.y > config.physics.max_collider_size ||
           self.size.z < config.physics.min_collider_size || self.size.z > config.physics.max_collider_size {
            return Err(BundleError::InvalidSize {
                size: self.size,
                min_size: config.physics.min_collider_size,
                max_size: config.physics.max_collider_size,
            });
        }
        
        Ok(())
    }
}

/// A specification for creating standalone physics objects.
///
/// This struct defines parameters for creating entities that primarily serve
/// as physics objects, such as debris, props, or interactive items. These
/// objects focus on physical simulation rather than gameplay logic.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use bevy_rapier3d::prelude::*;
/// use game_core::config::GameConfig;
/// use gameplay_render::factories::generic_bundle::*;
///
/// # fn example() -> Result<(), BundleError> {
/// let config = GameConfig::default();
/// let spec = PhysicsBundleSpec {
///     position: Vec3::new(10.0, 5.0, 0.0),
///     collider_shape: ColliderShape::Box(Vec3::new(2.0, 2.0, 2.0)),
///     mass: 50.0,
///     friction: 0.7,
///     restitution: 0.3,
///     collision_group: Group::GROUP_1,
///     is_dynamic: true,
/// };
///
/// let bundle = GenericBundleFactory::create(spec, &config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PhysicsBundleSpec {
    /// World position where the physics object will be spawned
    pub position: Vec3,
    /// Shape of the collision geometry
    pub collider_shape: ColliderShape,
    /// Mass of the object for physics simulation
    pub mass: f32,
    /// Friction coefficient for surface interactions
    pub friction: f32,
    /// Restitution (bounciness) coefficient for collisions
    pub restitution: f32,
    /// Collision group for filtering physics interactions
    pub collision_group: Group,
    /// Whether the object can move dynamically or is static
    pub is_dynamic: bool,
}

/// Collision shape definitions for physics objects.
///
/// This enum defines the available collision shapes that can be used
/// for physics simulation. Each shape has different performance
/// characteristics and use cases.
#[derive(Debug, Clone)]
pub enum ColliderShape {
    /// Box-shaped collider with dimensions (width, height, depth)
    Box(Vec3),
    /// Spherical collider with radius
    Sphere(f32),
    /// Capsule-shaped collider with radius and height
    Capsule { 
        /// Radius of the capsule
        radius: f32, 
        /// Height of the cylindrical portion
        height: f32 
    },
    /// Cylindrical collider with radius and height
    Cylinder { 
        /// Radius of the cylinder
        radius: f32, 
        /// Height of the cylinder
        height: f32 
    },
}

impl BundleSpec for PhysicsBundleSpec {
    type Bundle = PhysicsBundle;
    
    fn create_bundle(self, config: &GameConfig) -> Self::Bundle {
        // Validate and clamp mass
        let mass = self.mass.clamp(config.physics.min_mass, config.physics.max_mass);
        
        // Validate and clamp friction/restitution
        let friction = self.friction.clamp(0.0, 2.0);
        let restitution = self.restitution.clamp(0.0, 1.0);
        
        let transform = Transform::from_translation(self.position.clamp(
            Vec3::splat(config.physics.min_world_coord),
            Vec3::splat(config.physics.max_world_coord)
        ));
        
        // Create collider based on shape
        let collider = match self.collider_shape {
            ColliderShape::Box(size) => {
                let clamped_size = Vec3::new(
                    size.x.clamp(config.physics.min_collider_size, config.physics.max_collider_size),
                    size.y.clamp(config.physics.min_collider_size, config.physics.max_collider_size),
                    size.z.clamp(config.physics.min_collider_size, config.physics.max_collider_size),
                );
                Collider::cuboid(clamped_size.x / 2.0, clamped_size.y / 2.0, clamped_size.z / 2.0)
            }
            ColliderShape::Sphere(radius) => {
                let clamped_radius = radius.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                Collider::ball(clamped_radius)
            }
            ColliderShape::Capsule { radius, height } => {
                let clamped_radius = radius.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                let clamped_height = height.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                Collider::capsule_y(clamped_height, clamped_radius)
            }
            ColliderShape::Cylinder { radius, height } => {
                let clamped_radius = radius.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                let clamped_height = height.clamp(config.physics.min_collider_size, config.physics.max_collider_size);
                Collider::cylinder(clamped_height, clamped_radius)
            }
        };
        
        PhysicsBundle {
            transform,
            visibility: Visibility::Inherited,
            rigid_body: if self.is_dynamic { RigidBody::Dynamic } else { RigidBody::Fixed },
            collider,
            collision_groups: CollisionGroups::new(self.collision_group, Group::ALL),
            additional_mass: AdditionalMassProperties::Mass(mass),
            velocity: Velocity::zero(),
            damping: Damping { 
                linear_damping: config.physics.linear_damping, 
                angular_damping: config.physics.angular_damping 
            },
            friction: Friction::coefficient(friction),
            restitution: Restitution::coefficient(restitution),
        }
    }
    
    fn validate(&self, config: &GameConfig) -> Result<(), BundleError> {
        // Validate position bounds
        if self.position.x.abs() > config.physics.max_world_coord ||
           self.position.z.abs() > config.physics.max_world_coord {
            return Err(BundleError::PositionOutOfBounds {
                position: self.position,
                max_coord: config.physics.max_world_coord,
            });
        }
        
        // Validate mass
        if self.mass < config.physics.min_mass || self.mass > config.physics.max_mass {
            return Err(BundleError::InvalidMass {
                mass: self.mass,
                min_mass: config.physics.min_mass,
                max_mass: config.physics.max_mass,
            });
        }
        
        // Validate collider shape dimensions
        match &self.collider_shape {
            ColliderShape::Box(size) => {
                if size.x < config.physics.min_collider_size || size.x > config.physics.max_collider_size ||
                   size.y < config.physics.min_collider_size || size.y > config.physics.max_collider_size ||
                   size.z < config.physics.min_collider_size || size.z > config.physics.max_collider_size {
                    return Err(BundleError::InvalidSize {
                        size: *size,
                        min_size: config.physics.min_collider_size,
                        max_size: config.physics.max_collider_size,
                    });
                }
            }
            ColliderShape::Sphere(radius) => {
                if *radius < config.physics.min_collider_size || *radius > config.physics.max_collider_size {
                    return Err(BundleError::InvalidSize {
                        size: Vec3::splat(*radius),
                        min_size: config.physics.min_collider_size,
                        max_size: config.physics.max_collider_size,
                    });
                }
            }
            ColliderShape::Capsule { radius, height } | ColliderShape::Cylinder { radius, height } => {
                if *radius < config.physics.min_collider_size || *radius > config.physics.max_collider_size ||
                   *height < config.physics.min_collider_size || *height > config.physics.max_collider_size {
                    return Err(BundleError::InvalidSize {
                        size: Vec3::new(*radius, *height, *radius),
                        min_size: config.physics.min_collider_size,
                        max_size: config.physics.max_collider_size,
                    });
                }
            }
        }
        
        Ok(())
    }
}

/// A unified factory for creating validated entity bundles from specifications.
///
/// This factory provides a type-safe interface for creating various entity bundles
/// with automatic validation and configuration integration. It supports both single
/// bundle creation and batch processing for performance-critical scenarios.
///
/// # Examples
/// ```rust
/// use bevy::prelude::*;
/// use game_core::config::GameConfig;
/// use gameplay_render::factories::generic_bundle::*;
///
/// # fn example() -> Result<(), BundleError> {
/// let config = GameConfig::default();
/// 
/// // Single bundle creation
/// let vehicle_spec = VehicleBundleSpec {
///     vehicle_type: VehicleType::BasicCar,
///     position: Vec3::ZERO,
///     color: Color::RED,
///     max_speed_override: None,
///     mass_override: None,
///     include_physics: true,
///     include_collision: true,
///     include_visibility: true,
/// };
/// 
/// let bundle = GenericBundleFactory::create(vehicle_spec, &config)?;
/// 
/// // Batch creation
/// let specs = vec![
///     VehicleBundleSpec { /* ... */ },
///     VehicleBundleSpec { /* ... */ },
/// ];
/// let bundles = GenericBundleFactory::create_batch(specs, &config)?;
/// # Ok(())
/// # }
/// ```
pub struct GenericBundleFactory;

impl GenericBundleFactory {
    /// Creates a bundle from a specification with validation.
    ///
    /// This method validates the specification against the game configuration
    /// and creates the corresponding bundle if validation passes.
    ///
    /// # Arguments
    /// * `spec` - The bundle specification to create from
    /// * `config` - Game configuration for validation and defaults
    ///
    /// # Returns
    /// The created bundle if validation succeeds, or a [`BundleError`] if not
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let spec = VehicleBundleSpec {
    ///     vehicle_type: VehicleType::BasicCar,
    ///     position: Vec3::ZERO,
    ///     color: Color::RED,
    ///     max_speed_override: None,
    ///     mass_override: None,
    ///     include_physics: true,
    ///     include_collision: true,
    ///     include_visibility: true,
    /// };
    /// 
    /// let bundle = GenericBundleFactory::create(spec, &config)?;
    /// # Ok::<(), BundleError>(())
    /// ```
    pub fn create<T: BundleSpec>(
        spec: T,
        config: &GameConfig,
    ) -> Result<T::Bundle, BundleError> {
        // Validate specification
        spec.validate(config)?;
        
        // Create bundle with validated configuration
        Ok(spec.create_bundle(config))
    }
    
    /// Creates multiple bundles from specifications with batch validation.
    ///
    /// This method validates all specifications before creating any bundles,
    /// ensuring either all succeed or all fail. This is more efficient than
    /// individual creation for large numbers of entities.
    ///
    /// # Arguments
    /// * `specs` - Vector of bundle specifications to create from
    /// * `config` - Game configuration for validation and defaults
    ///
    /// # Returns
    /// Vector of created bundles if all validations succeed, or first error
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let specs = vec![
    ///     VehicleBundleSpec {
    ///         vehicle_type: VehicleType::BasicCar,
    ///         position: Vec3::new(0.0, 0.0, 0.0),
    ///         color: Color::RED,
    ///         max_speed_override: None,
    ///         mass_override: None,
    ///         include_physics: true,
    ///         include_collision: true,
    ///         include_visibility: true,
    ///     },
    ///     VehicleBundleSpec {
    ///         vehicle_type: VehicleType::SuperCar,
    ///         position: Vec3::new(10.0, 0.0, 0.0),
    ///         color: Color::BLUE,
    ///         max_speed_override: None,
    ///         mass_override: None,
    ///         include_physics: true,
    ///         include_collision: true,
    ///         include_visibility: true,
    ///     },
    /// ];
    /// 
    /// let bundles = GenericBundleFactory::create_batch(specs, &config)?;
    /// # Ok::<(), BundleError>(())
    /// ```
    pub fn create_batch<T: BundleSpec>(
        specs: Vec<T>,
        config: &GameConfig,
    ) -> Result<Vec<T::Bundle>, BundleError> {
        // Validate all specs first
        for spec in &specs {
            spec.validate(config)?;
        }
        
        // Create all bundles
        Ok(specs.into_iter().map(|spec| spec.create_bundle(config)).collect())
    }
}

/// Convenience builder functions for common bundle types.
///
/// These functions provide simplified interfaces for creating the most common
/// entity bundles without needing to manually construct specification structs.
impl GenericBundleFactory {
    /// Creates a vehicle bundle with default configuration settings.
    ///
    /// This is a convenience function for creating vehicles with standard
    /// physics, collision, and visibility settings enabled.
    ///
    /// # Arguments
    /// * `vehicle_type` - The type of vehicle to create
    /// * `position` - World position for the vehicle
    /// * `color` - Primary color for the vehicle
    /// * `config` - Game configuration for validation and defaults
    ///
    /// # Returns
    /// A complete vehicle bundle ready for spawning
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let vehicle = GenericBundleFactory::vehicle(
    ///     VehicleType::BasicCar,
    ///     Vec3::new(0.0, 0.0, 0.0),
    ///     Color::RED,
    ///     &config,
    /// )?;
    /// # Ok::<(), BundleError>(())
    /// ```
    pub fn vehicle(
        vehicle_type: VehicleType,
        position: Vec3,
        color: Color,
        config: &GameConfig,
    ) -> Result<VehicleBundle, BundleError> {
        let spec = VehicleBundleSpec {
            vehicle_type,
            position,
            color,
            max_speed_override: None,
            mass_override: None,
            include_physics: true,
            include_collision: true,
            include_visibility: true,
        };
        Self::create(spec, config)
    }
    
    /// Creates an NPC bundle with default AI and physics settings.
    ///
    /// This is a convenience function for creating NPCs with standard
    /// behavior, physics, and AI components enabled.
    ///
    /// # Arguments
    /// * `position` - World position for the NPC
    /// * `height` - Height of the NPC in meters
    /// * `build` - Build factor affecting size and mass
    /// * `appearance` - Visual appearance configuration
    /// * `config` - Game configuration for validation and defaults
    ///
    /// # Returns
    /// A complete NPC bundle ready for spawning
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let npc = GenericBundleFactory::npc(
    ///     Vec3::new(5.0, 0.0, 10.0),
    ///     1.75,
    ///     1.0,
    ///     NPCAppearance::default(),
    ///     &config,
    /// )?;
    /// # Ok::<(), BundleError>(())
    /// ```
    pub fn npc(
        position: Vec3,
        height: f32,
        build: f32,
        appearance: NPCAppearance,
        config: &GameConfig,
    ) -> Result<NPCBundle, BundleError> {
        let spec = NPCBundleSpec {
            position,
            height,
            build,
            appearance,
            behavior: None,
            include_physics: true,
            include_ai: true,
        };
        Self::create(spec, config)
    }
    
    /// Creates a building bundle with default collision and LOD settings.
    ///
    /// This is a convenience function for creating buildings with standard
    /// collision detection and base LOD level.
    ///
    /// # Arguments
    /// * `position` - World position for the building
    /// * `size` - Dimensions of the building (width, height, depth)
    /// * `building_type` - Type of building to create
    /// * `color` - Primary color for the building
    /// * `config` - Game configuration for validation and defaults
    ///
    /// # Returns
    /// A complete building bundle ready for spawning
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let building = GenericBundleFactory::building(
    ///     Vec3::new(100.0, 0.0, 200.0),
    ///     Vec3::new(20.0, 50.0, 15.0),
    ///     BuildingType::Residential,
    ///     Color::GRAY,
    ///     &config,
    /// )?;
    /// # Ok::<(), BundleError>(())
    /// ```
    pub fn building(
        position: Vec3,
        size: Vec3,
        building_type: BuildingType,
        color: Color,
        config: &GameConfig,
    ) -> Result<BuildingBundle, BundleError> {
        let spec = BuildingBundleSpec {
            position,
            size,
            building_type,
            color,
            include_collision: true,
            lod_level: 0,
        };
        Self::create(spec, config)
    }
    
    /// Creates a physics object bundle with default friction and restitution.
    ///
    /// This is a convenience function for creating physics objects with standard
    /// surface properties derived from the game configuration.
    ///
    /// # Arguments
    /// * `position` - World position for the physics object
    /// * `collider_shape` - Shape of the collision geometry
    /// * `mass` - Mass of the object for physics simulation
    /// * `collision_group` - Collision group for physics filtering
    /// * `is_dynamic` - Whether the object can move or is static
    /// * `config` - Game configuration for validation and defaults
    ///
    /// # Returns
    /// A complete physics bundle ready for spawning
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_rapier3d::prelude::*;
    /// # use game_core::config::GameConfig;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let config = GameConfig::default();
    /// let physics_obj = GenericBundleFactory::physics_object(
    ///     Vec3::new(10.0, 5.0, 0.0),
    ///     ColliderShape::Box(Vec3::new(2.0, 2.0, 2.0)),
    ///     50.0,
    ///     Group::GROUP_1,
    ///     true,
    ///     &config,
    /// )?;
    /// # Ok::<(), BundleError>(())
    /// ```
    pub fn physics_object(
        position: Vec3,
        collider_shape: ColliderShape,
        mass: f32,
        collision_group: Group,
        is_dynamic: bool,
        config: &GameConfig,
    ) -> Result<PhysicsBundle, BundleError> {
        let spec = PhysicsBundleSpec {
            position,
            collider_shape,
            mass,
            friction: config.physics.ground_friction,
            restitution: 0.3,
            collision_group,
            is_dynamic,
        };
        Self::create(spec, config)
    }
}

/// Bundle utility functions for common entity patterns.
///
/// These functions create specialized bundles for specific use cases in the
/// game world, such as dynamic content, vegetation, and chunk-based entities.
impl GenericBundleFactory {
    /// Creates a dynamic content bundle for world entities.
    ///
    /// Dynamic content entities are used for world objects that can be
    /// spawned and despawned based on distance or other criteria.
    ///
    /// # Arguments
    /// * `content_type` - Type of content to create
    /// * `position` - World position for the entity
    /// * `_max_distance` - Maximum distance for culling (currently unused)
    ///
    /// # Returns
    /// A dynamic content bundle with visibility and culling components
    ///
    /// # Examples
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use gameplay_render::factories::generic_bundle::*;
    /// let bundle = GenericBundleFactory::dynamic_content(
    ///     ContentType::Tree,
    ///     Vec3::new(50.0, 0.0, 100.0),
    ///     200.0,
    /// );
    /// ```
    pub fn dynamic_content(
        content_type: ContentType,
        position: Vec3,
        _max_distance: f32,
    ) -> DynamicContentBundle {
        DynamicContentBundle {
            dynamic_content: DynamicContent { content_type },
            transform: Transform::from_translation(position),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            cullable: UnifiedCullable::vegetation(),
        }
    }
    
    /// Creates a dynamic physics bundle for moving objects with collision.
    ///
    /// # Arguments
    /// * `content_type` - Type of content to create
    /// * `position` - World position for the entity
    /// * `collider` - Collision shape for physics
    /// * `collision_groups` - Collision group configuration
    /// * `_max_distance` - Maximum distance for culling (currently unused)
    ///
    /// # Returns
    /// A dynamic physics bundle with movement and collision components
    pub fn dynamic_physics(
        content_type: ContentType,
        position: Vec3,
        collider: Collider,
        collision_groups: CollisionGroups,
        _max_distance: f32,
    ) -> DynamicPhysicsBundle {
        DynamicPhysicsBundle {
            dynamic_content: DynamicContent { content_type },
            transform: Transform::from_translation(position),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            rigid_body: RigidBody::Dynamic,
            collider,
            collision_groups,
            velocity: Velocity::zero(),
            cullable: UnifiedCullable::vegetation(),
        }
    }
    
    /// Creates a dynamic vehicle bundle for cars with physics.
    ///
    /// # Arguments
    /// * `position` - World position for the vehicle
    /// * `collision_groups` - Collision group configuration
    /// * `damping` - Physics damping settings
    ///
    /// # Returns
    /// A dynamic vehicle bundle with car components
    pub fn dynamic_vehicle(
        position: Vec3,
        collision_groups: CollisionGroups,
        damping: Damping,
    ) -> DynamicVehicleBundle {
        DynamicVehicleBundle {
            dynamic_content: DynamicContent { content_type: ContentType::Vehicle },
            car: Car,
            transform: Transform::from_xyz(position.x, position.y, position.z),
            visibility: Visibility::Visible,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(1.0, 0.5, 2.0),
            collision_groups,
            velocity: Velocity::zero(),
            damping,
            locked_axes: LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            cullable: UnifiedCullable::vehicle(),
        }
    }
    
    /// Creates a vegetation bundle for trees and plants.
    ///
    /// # Arguments
    /// * `position` - World position for the vegetation
    /// * `_max_distance` - Maximum distance for culling (currently unused)
    ///
    /// # Returns
    /// A vegetation bundle with tree content type
    pub fn vegetation(
        position: Vec3,
        _max_distance: f32,
    ) -> VegetationBundle {
        VegetationBundle {
            dynamic_content: DynamicContent { content_type: ContentType::Tree },
            transform: Transform::from_translation(position),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            cullable: UnifiedCullable::vegetation(),
        }
    }
    
    /// Creates a static physics bundle for immobile objects.
    ///
    /// # Arguments
    /// * `position` - World position for the static object
    /// * `collider` - Collision shape for physics
    /// * `collision_groups` - Collision group configuration
    ///
    /// # Returns
    /// A static physics bundle with fixed rigid body
    pub fn static_physics(
        position: Vec3,
        collider: Collider,
        collision_groups: CollisionGroups,
    ) -> StaticPhysicsBundle {
        StaticPhysicsBundle {
            transform: Transform::from_translation(position),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::VISIBLE,
            rigid_body: RigidBody::Fixed,
            collider,
            collision_groups,
        }
    }
    
    /// Creates a unified chunk bundle for world streaming.
    ///
    /// # Arguments
    /// * `chunk_coord` - Chunk coordinates in the world grid
    /// * `layer` - Content layer for the chunk
    /// * `content_type` - Type of content in the chunk
    /// * `position` - World position for the chunk
    /// * `_max_distance` - Maximum distance for culling (currently unused)
    ///
    /// # Returns
    /// A unified chunk bundle with chunk entity tracking
    pub fn unified_chunk(
        chunk_coord: (i32, i32),
        layer: gameplay_sim::world::unified_world::ContentLayer,
        content_type: ContentType,
        position: Vec3,
        _max_distance: f32,
    ) -> UnifiedChunkBundle {
        UnifiedChunkBundle {
            chunk_entity: UnifiedChunkEntity { 
                coord: game_core::components::ChunkCoord::new(chunk_coord.0, chunk_coord.1), 
                layer: layer as u32 
            },
            dynamic_content: DynamicContent { content_type },
            transform: Transform::from_translation(position),
            visibility: Visibility::Inherited,
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
            cullable: UnifiedCullable::vegetation(),
        }
    }
}
