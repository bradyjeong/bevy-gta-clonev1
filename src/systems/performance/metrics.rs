use bevy::{
    diagnostic::{Diagnostics, Diagnostic},
    prelude::*,
};

use crate::systems::world::unified_world::UnifiedWorldManager;

/// Setup custom diagnostic names
pub const ENTITY_COUNT: &str = "entity_count";
pub const CHUNK_COUNT: &str = "chunk_count";
pub const ACTIVE_CHUNKS: &str = "active_chunks";

/// Setup custom diagnostics
pub fn setup_custom_diagnostics(mut diagnostics: ResMut<Diagnostics>) {
    diagnostics.add_diagnostic(Diagnostic::new(ENTITY_COUNT, "Entity Count"));
    diagnostics.add_diagnostic(Diagnostic::new(CHUNK_COUNT, "Chunk Count"));
    diagnostics.add_diagnostic(Diagnostic::new(ACTIVE_CHUNKS, "Active Chunks"));
}

/// Record total entity count for diagnostics
pub fn record_entity_count(
    mut diagnostics: ResMut<Diagnostics>,
    query: Query<Entity>,
) {
    let count = query.iter().count() as f64;
    diagnostics.measure(ENTITY_COUNT, count);
}

/// Record total chunk count for diagnostics
pub fn record_chunk_count(
    mut diagnostics: ResMut<Diagnostics>,
    world: Option<Res<UnifiedWorldManager>>,
) {
    if let Some(world) = world {
        let count = world.chunks.len() as f64;
        diagnostics.measure(CHUNK_COUNT, count);
    }
}

/// Record active chunk count for diagnostics
pub fn record_active_chunks(
    mut diagnostics: ResMut<Diagnostics>,
    world: Option<Res<UnifiedWorldManager>>,
) {
    if let Some(world) = world {
        let active_count = world
            .chunks
            .iter()
            .flatten()
            .filter(|chunk| chunk.entity_count > 0)
            .count() as f64;
        diagnostics.measure(ACTIVE_CHUNKS, active_count);
    }
}
