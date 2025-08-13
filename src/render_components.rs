// Re-export Bevy's official Mesh3d and MeshMaterial3d components for Bevy 0.16
// These are NOT wrappers - they're the actual Bevy components
// Located in bevy_render and bevy_pbr modules

pub use bevy::render::mesh::Mesh3d;
pub use bevy::pbr::MeshMaterial3d;
