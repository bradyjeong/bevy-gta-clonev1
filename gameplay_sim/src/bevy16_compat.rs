//! Bevy 0.16 compatibility layer for mechanical API fixes

use bevy::prelude::*;
use bevy::asset::Assets;
use bevy::render::mesh::{Mesh, PrimitiveTopology};
use bevy::ecs::query::{QueryData, QueryFilter};

/// Compatibility utilities for Bevy 0.16 migration
pub mod bevy16 {
    use super::*;

    /// Create empty mesh with triangle list topology
    pub fn empty_mesh() -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, bevy::render::render_asset::RenderAssetUsages::default());
        mesh
    }
}

/// Extension trait for Time API compatibility
pub trait TimeExt {
    fn delta_seconds(&self) -> f32;
    fn elapsed_seconds(&self) -> f32;
}

impl TimeExt for Time {
    fn delta_seconds(&self) -> f32 {
        self.delta_secs()
    }
    
    fn elapsed_seconds(&self) -> f32 {
        self.elapsed_secs()
    }
}

/// Extension trait for Query API compatibility
pub trait QueryExt<'w, 's, D: QueryData, F: QueryFilter> {
    fn get_single(&self) -> Result<D::Item<'_>, bevy::ecs::query::QuerySingleError>;
    fn get_single_mut(&mut self) -> Result<D::Item<'_>, bevy::ecs::query::QuerySingleError>;
}

impl<'w, 's, D: QueryData<ReadOnly = D>, F: QueryFilter> QueryExt<'w, 's, D, F> for Query<'w, 's, D, F>
where
    's: 'w,
{
    fn get_single(&self) -> Result<D::Item<'_>, bevy::ecs::query::QuerySingleError> {
        // Direct call to single() - API is the same in Bevy 0.16
        self.single()
    }
    
    fn get_single_mut(&mut self) -> Result<D::Item<'_>, bevy::ecs::query::QuerySingleError> {
        self.single_mut()
    }
}

/// Extension trait for EntityCommands API compatibility
pub trait EntityCommandsExt {
    fn despawn_recursive(&mut self) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_> {
    fn despawn_recursive(&mut self) -> &mut Self {
        self.despawn();
        self
    }
}

/// Extension trait for Mesh API compatibility
pub trait MeshExt {
    fn set_indices(&mut self, indices: Option<bevy::render::mesh::Indices>);
}

impl MeshExt for Mesh {
    fn set_indices(&mut self, indices: Option<bevy::render::mesh::Indices>) {
        if let Some(indices) = indices {
            self.insert_indices(indices);
        }
    }
}
