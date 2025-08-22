use crate::components::*;
use crate::factories::MeshFactory;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Factory for creating tree/vegetation entities with simplified, clear interface
pub struct TreeFactory;

impl TreeFactory {
    /// Spawn a single tree entity
    pub fn spawn(
        commands: &mut Commands,
        _mesh_factory: &MeshFactory,
        position: Vec3,
        tree_type: VegetationType,
    ) -> Entity {
        let entity = commands
            .spawn((
                Name::new("Tree"),
                Transform::from_translation(position),
                Visibility::default(),
            ))
            .id();

        // Add tree-specific components
        commands.entity(entity).insert((
            VegetationLOD::default(),
            tree_type,
            // Basic physics
            RigidBody::Fixed,
            Collider::cylinder(0.2, 3.0),
            CollisionGroups::new(Group::GROUP_4, Group::ALL),
        ));

        entity
    }
}

#[derive(Component, Clone, Copy)]
pub enum VegetationType {
    Oak,
    Pine,
    Palm,
    Bush,
}

impl Default for VegetationType {
    fn default() -> Self {
        Self::Oak
    }
}
