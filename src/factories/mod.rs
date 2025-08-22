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

// Unified factory system - replaces all duplicate factories
pub mod entity_factory_unified;

// Focused factory modules (following AGENT.md simplicity principles)
// pub mod entity_limits; // Moved to services/
pub mod collision_detector;
pub mod position_validator;

// Domain-specific factories (following simplicity guidelines)
pub mod building_factory;

pub mod entity_limit;
pub mod npc_factory;
pub mod tree_factory;

// Specialized factories
pub mod generic_bundle;
pub mod material_factory;
pub mod mesh_factory;
pub mod rendering_factory;
pub mod transform_factory;

// Public API - unified factory system
pub use entity_factory_unified::*;

// Focused module exports (AGENT.md compliant architecture)
pub use collision_detector::*;
pub use position_validator::*;

// Domain-specific factory exports
pub use building_factory::*;

pub use entity_limit::{EntityLimit, EntityLimitManager, EntityType};
pub use npc_factory::*;
pub use tree_factory::*; // Avoid conflict with unified factory

// Specialized factory exports
pub use generic_bundle::*;
pub use material_factory::*;
pub use mesh_factory::*;
pub use rendering_factory::*;
pub use transform_factory::*;
