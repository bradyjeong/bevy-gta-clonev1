pub use bevy::prelude::*;
use bevy::render::mesh::{Mesh, Indices};

/// Bevy 0.16 compatibility layer to ease API migration
/// Following Oracle's Phase 2 strategy for API Surface Migration

pub trait MeshExt {
    fn with_indices(self, indices: impl Into<Indices>) -> Self;
}

impl MeshExt for Mesh {
    fn with_indices(mut self, indices: impl Into<Indices>) -> Self {
        self.insert_indices(indices.into());
        self
    }
}

/// Helper for mesh creation with PrimitiveTopology
pub fn empty_mesh() -> Mesh {
    Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList)
}

/// Helper for deprecated time methods
pub trait TimeExt {
    fn delta_seconds(&self) -> f32;
    fn elapsed_seconds(&self) -> f32;
}

impl TimeExt for bevy::prelude::Time {
    fn delta_seconds(&self) -> f32 {
        self.delta_secs()
    }
    
    fn elapsed_seconds(&self) -> f32 {
        self.elapsed_secs()
    }
}

/// Helper for deprecated query methods
pub trait QueryExt<'w, 's, D: bevy::ecs::query::QueryData, F: bevy::ecs::query::QueryFilter> {
    fn get_single(&self) -> Result<D::Item<'w>, bevy::ecs::query::QuerySingleError>;
    fn get_single_mut(&mut self) -> Result<D::Item<'w>, bevy::ecs::query::QuerySingleError>;
}

impl<'w, 's, D: bevy::ecs::query::QueryData, F: bevy::ecs::query::QueryFilter> QueryExt<'w, 's, D, F> for bevy::ecs::query::Query<'w, 's, D, F> {
    fn get_single(&self) -> Result<D::Item<'w>, bevy::ecs::query::QuerySingleError> {
        self.single()
    }
    
    fn get_single_mut(&mut self) -> Result<D::Item<'w>, bevy::ecs::query::QuerySingleError> {
        self.single_mut()
    }
}

/// Helper for deprecated despawn_recursive
pub trait EntityCommandsExt<'a> {
    fn despawn_recursive(&mut self);
}

impl<'a> EntityCommandsExt<'a> for bevy::ecs::system::EntityCommands<'a> {
    fn despawn_recursive(&mut self) {
        self.despawn();
    }
}
