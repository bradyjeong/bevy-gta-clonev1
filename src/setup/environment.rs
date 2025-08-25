use crate::bundles::VisibleChildBundle;
use crate::constants::STATIC_GROUP;
use bevy::prelude::*;
use bevy::render::view::visibility::VisibilityRange;
use bevy_rapier3d::prelude::*;

// NOTE: Roads are now fully dynamic - no static setup needed
// The dynamic road system guarantees immediate spawn area roads

pub fn setup_palm_trees(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // PALM TREES EVERYWHERE! (Dubai oasis style)
    let palm_positions = [
        // Close to spawn area
        (10.0, 15.0),
        (15.0, 8.0),
        (-12.0, 18.0),
        (-8.0, -14.0),
        (22.0, -16.0),
        (-18.0, 12.0),
        (25.0, 25.0),
        (-25.0, -25.0),
        // Medium distance palm groves
        (45.0, 35.0),
        (38.0, -42.0),
        (-35.0, 48.0),
        (-45.0, -38.0),
        (60.0, 15.0),
        (-55.0, 25.0),
        (40.0, -60.0),
        (-40.0, 65.0),
        // Far distance oasis areas
        (80.0, 90.0),
        (85.0, -95.0),
        (-90.0, 85.0),
        (-85.0, -90.0),
        (120.0, 45.0),
        (-110.0, 55.0),
        (95.0, -115.0),
        (-105.0, 125.0),
        // Scattered throughout the massive world
        (150.0, 80.0),
        (140.0, -160.0),
        (-145.0, 155.0),
        (-155.0, -145.0),
        (180.0, 25.0),
        (-175.0, 35.0),
        (165.0, -185.0),
        (-165.0, 175.0),
        (200.0, 120.0),
        (-195.0, 110.0),
        (190.0, -205.0),
        (-200.0, 195.0),
        // Road-side palms (near highways)
        (15.0, 200.0),
        (15.0, -200.0),
        (-15.0, 180.0),
        (-15.0, -180.0),
        (200.0, 15.0),
        (-200.0, 15.0),
        (180.0, -15.0),
        (-180.0, -15.0),
    ];

    for &(x, z) in palm_positions.iter() {
        // Simple palm tree - single trunk + simple crown
        let palm_entity = commands
            .spawn((
                Transform::from_xyz(x, 0.0, z),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ))
            .id();

        // Simple trunk - single brown cylinder
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.4, 0.25, 0.15))), // Brown trunk
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(palm_entity),
            VisibleChildBundle::default(),
        ));

        // Simple fronds - just 4 green rectangles arranged in a cross
        for i in 0..4 {
            let angle = (i as f32) * std::f32::consts::PI / 2.0;

            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.5, 0.1, 0.8))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.25))), // Green fronds
                Transform::from_xyz(angle.cos() * 1.2, 7.5, angle.sin() * 1.2).with_rotation(
                    Quat::from_rotation_y(angle) * Quat::from_rotation_z(-0.2), // Slight droop
                ),
                ChildOf(palm_entity),
                VisibleChildBundle::default(),
            ));
        }

        // Simple physics collider for trunk
        commands.spawn((
            RigidBody::Fixed,
            Collider::cylinder(4.0, 0.3),
            CollisionGroups::new(STATIC_GROUP, Group::ALL),
            Transform::from_xyz(0.0, 4.0, 0.0),
            ChildOf(palm_entity),
            VisibilityRange::abrupt(0.0, 200.0),
        ));
    }
}
