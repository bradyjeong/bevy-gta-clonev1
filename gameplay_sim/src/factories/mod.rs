//! Factory systems for entity creation

pub mod entity_factory_unified;

pub use entity_factory_unified::*;

// Phase A: Compatibility shims for legacy factory types
pub use game_core::factories::*;
