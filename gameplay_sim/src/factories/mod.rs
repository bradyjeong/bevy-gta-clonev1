// Unified factory system - replaces all duplicate factories
pub mod entity_factory_unified;
pub mod entity_builder_unified;

// Specialized factories
pub mod material_factory;
pub mod mesh_factory;
pub mod transform_factory;
pub mod generic_bundle;
pub mod rendering_factory;
// Public API - unified factory system
pub use entity_factory_unified::*;
pub use entity_builder_unified::*;
// Specialized factory exports
pub use material_factory::*;
pub use mesh_factory::*;
pub use transform_factory::*;
pub use generic_bundle::*;
pub use rendering_factory::*;
