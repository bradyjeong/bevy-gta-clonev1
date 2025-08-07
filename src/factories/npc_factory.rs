use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::factories::MeshFactory;

/// Factory for creating NPC entities with simplified, clear interface
pub struct NPCFactory;

impl NPCFactory {
    /// Spawn a single NPC entity
    pub fn spawn(
        commands: &mut Commands,
        _mesh_factory: &MeshFactory,
        position: Vec3,
        npc_type: NPCType,
    ) -> Entity {
        let entity = commands.spawn((
            Name::new("NPC"),
            Transform::from_translation(position),
            Visibility::default(),
        )).id();

        // Add NPC-specific components
        commands.entity(entity).insert((
            NPC {
                target_position: position + Vec3::new(5.0, 0.0, 0.0),
                speed: 2.0,
                last_update: 0.0,
                update_interval: 0.5,
            },
            NPCState::new(world::NPCType::Civilian),
            npc_type,
            // Basic physics
            RigidBody::Dynamic,
            Collider::capsule_y(0.3, 0.9),
            CollisionGroups::new(Group::GROUP_3, Group::ALL),
            Velocity::default(),
        ));

        entity
    }
}

#[derive(Component, Clone, Copy)]
pub enum NPCType {
    Pedestrian,
    Worker,
    Police,
}

impl Default for NPCType {
    fn default() -> Self {
        Self::Pedestrian
    }
}
