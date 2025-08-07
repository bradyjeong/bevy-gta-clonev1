use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::factories::MeshFactory;

/// Factory for creating building entities with simplified, clear interface
pub struct BuildingFactory;

impl BuildingFactory {
    /// Spawn a single building entity
    pub fn spawn(
        commands: &mut Commands,
        _mesh_factory: &MeshFactory,
        position: Vec3,
        building_type: BuildingType,
    ) -> Entity {
        let entity = commands.spawn((
            Name::new("Building"),
            Transform::from_translation(position),
            Visibility::default(),
        )).id();

        // Add building-specific components
        commands.entity(entity).insert((
            building_type,
            Building { 
                building_type: world::BuildingType::Residential,
                height: 3.0,
                scale: Vec3::splat(1.0),
            },
            // Basic physics
            RigidBody::Fixed,
            Collider::cuboid(2.0, 3.0, 2.0),
            CollisionGroups::new(Group::GROUP_2, Group::ALL),
        ));

        entity
    }
}

#[derive(Component, Clone, Copy)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industrial,
}

impl Default for BuildingType {
    fn default() -> Self {
        Self::Residential
    }
}
