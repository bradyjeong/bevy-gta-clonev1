use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct Mesh3d(pub Handle<Mesh>);

#[derive(Component, Deref, DerefMut)]
pub struct MeshMaterial3d<M: Material>(pub Handle<M>);
