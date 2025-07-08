// Unified factory system - replaces all duplicate factories
/// Unified entity factory system for centralized entity creation and management.
pub mod entity_factory_unified;
/// Fluent API builder interface for the unified entity factory system.
pub mod entity_builder_unified;

// Specialized factories
/// Material creation and management utilities for rendering components.
pub mod material_factory;
pub mod mesh_factory;
pub mod transform_factory;
pub mod generic_bundle;
pub mod rendering_factory;

// Public API - specialized factory exports only
pub use material_factory::*;
pub use mesh_factory::*;
pub use transform_factory::*;
