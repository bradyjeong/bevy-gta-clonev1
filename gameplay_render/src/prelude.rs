//! Common imports for gameplay rendering

pub use bevy::prelude::*;
pub use engine_core::prelude::*;
pub use engine_bevy::prelude::*;
pub use game_core::prelude::*;
pub use gameplay_sim::prelude::*;

// Re-export rendering modules (avoid glob conflicts)
pub use crate::factories::{
    entity_builder_unified::*,
    entity_factory_unified::*,
    generic_bundle::{GenericBundleFactory, ColliderShape, ParticleEffectType},
    material_factory::*,
    mesh_factory::*,
    rendering_factory::*,
    transform_factory::*,
};
pub use crate::batching::*;
pub use crate::batch_processing::*;
pub use crate::world::*;
pub use crate::plugins::*;
