use bevy::prelude::*;

pub fn test_spawn(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Test what Bevy 0.16 actually wants for mesh rendering
    commands.spawn((
        // Try Bevy's actual components
        meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        materials.add(StandardMaterial::default()),
        Transform::default(),
    ));
}
