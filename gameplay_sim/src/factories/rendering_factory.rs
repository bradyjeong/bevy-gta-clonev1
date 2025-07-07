use bevy::prelude::*;
use game_core::prelude::*;

/// Simplified rendering factory for consistent entity rendering
#[derive(Resource)]
pub struct RenderingFactory;

impl RenderingFactory {
    pub fn new() -> Self {
        Self
    }

    /// Create a basic rendered entity with mesh and material
    pub fn create_rendered_entity(
        commands: &mut Commands,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        position: Vec3,
    ) -> Entity {
        commands.spawn(MaterialMeshBundle {
            mesh,
            material,
            transform: Transform::from_translation(position),
            ..default()
        }).id()
    }

    /// Create a vehicle with basic rendering
    pub fn create_rendered_vehicle(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        let mesh = meshes.add(Cuboid::new(1.8, 1.0, 4.2));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        });

        Self::create_rendered_entity(commands, mesh, material, position)
    }

    /// Create a building with basic rendering
    pub fn create_rendered_building(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        width: f32,
        height: f32,
        depth: f32,
    ) -> Entity {
        let mesh = meshes.add(Cuboid::new(width, height, depth));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.4),
            ..default()
        });

        Self::create_rendered_entity(commands, mesh, material, position)
    }

    /// Create a tree with basic rendering
    pub fn create_rendered_tree(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        let trunk_mesh = meshes.add(Cylinder::new(0.3, 3.0));
        let trunk_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.2, 0.1),
            ..default()
        });

        Self::create_rendered_entity(commands, trunk_mesh, trunk_material, position)
    }

    /// Create an NPC with basic rendering
    pub fn create_rendered_npc(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        let mesh = meshes.add(Capsule3d::new(0.3, 1.8));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.4, 0.2),
            ..default()
        });

        Self::create_rendered_entity(commands, mesh, material, position)
    }
}

impl Default for RenderingFactory {
    fn default() -> Self {
        Self::new()
    }
}
