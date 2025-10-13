//! # Entity Creation Factories
//!
//! This module provides factory patterns for creating game entities consistently.
//! Factories encapsulate entity creation logic and ensure proper component setup.
//!
//! ## Factory Design Principles
//!
//! - **Stateless**: Factories are pure functions, no internal state
//! - **Consistent**: Same inputs always produce same entity configurations
//! - **Composable**: Small factories can be combined for complex entities
//! - **Validation**: Factories validate inputs and provide sensible defaults
//!
//! ## Factory Types
//!
//! ### Unified Factories (Recommended)
//! - `entity_factory_unified`: Main entity creation with consistent patterns
//! - `entity_limits`: Focused entity count management and FIFO cleanup
//! - `position_validator`: Spawn position validation and ground height detection
//! - `collision_detector`: Entity collision detection for spawning
//!
//!
//! ### Specialized Factories
//! - `material_factory`: Material and texture creation
//! - `mesh_factory`: Mesh generation and caching  
//! - `transform_factory`: Transform and positioning utilities
//! - `rendering_factory`: Rendering component setup
//! - `generic_bundle`: Reusable component bundles
//!
//! ## Usage Patterns
//!
//! ```rust
//! // Simple entity creation
//! let vehicle_entity = UnifiedEntityFactory::create_vehicle(
//!     &mut commands,
//!     VehicleType::Car,
//!     Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//! );
//!
//! // Builder pattern for complex entities
//! let npc_entity = UnifiedEntityBuilder::new()
//!     .with_position(Vec3::new(10.0, 0.0, 5.0))
//!     .with_movement_speed(2.5)
//!     .with_ai_behavior(AIBehavior::Wandering)
//!     .spawn(&mut commands);
//! ```
//!
//! ## Factory Benefits (AGENT.md Compliant)
//!
//! - **Single Responsibility**: Each factory module has one clear purpose
//! - **Clear Boundaries**: No tangled interdependencies between modules
//! - **Minimal Coupling**: Modules only depend on what they need
//! - **Straightforward Data Flow**: Easy to trace entity creation process
//! - **Maintainability**: Entity creation logic is properly separated
//! - **Testing**: Individual modules can be unit tested in isolation
//! - **Performance**: Focused modules enable better optimization
//!
//! ## Adding New Factories
//!
//! 1. Identify common entity creation patterns
//! 2. Create stateless factory functions
//! 3. Include validation and default values
//! 4. Export from this module

// Utility modules for position and collision detection
pub mod collision_detector;
pub mod position_validator;

// Domain-specific factories with single responsibilities (following AGENT.md principles)
pub mod building_factory;
pub mod effect_factory;
pub mod entity_limit;
pub mod npc_factory;

pub mod vehicle_factory;

// Specialized utility factories
pub mod beach_terrain;
pub mod generic_bundle;
pub mod material_factory;
pub mod mesh_factory;
pub mod rendering_factory;
pub mod subdivided_plane;
pub mod transform_factory;

// Utility module exports (explicit imports, no wildcards per AGENT.md)
pub use collision_detector::CollisionDetector;
pub use position_validator::PositionValidator;

// Domain-specific factory exports with explicit imports (no wildcards)
pub use building_factory::{BuildingFactory, BuildingType};
pub use effect_factory::{EffectFactory, ParticleEffect};
pub use entity_limit::{EntityLimit, EntityLimitManager, EntityType};
pub use npc_factory::{NPCFactory, NPCType};
pub use vehicle_factory::VehicleFactory;

// Specialized factory exports (selective imports)
pub use beach_terrain::{create_beach_slope, create_circular_beach_ring};
pub use generic_bundle::{BundleError, GenericBundleFactory};
pub use material_factory::{MaterialFactory, initialize_material_factory};
pub use mesh_factory::MeshFactory;
pub use rendering_factory::{
    RenderingBundleType, RenderingFactory, StandardRenderingPattern, VehicleBodyType,
};
pub use subdivided_plane::create_subdivided_plane;
pub use transform_factory::TransformFactory;
