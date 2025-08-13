use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::bundles::VisibleChildBundle;
use crate::systems::audio::FootstepTimer;
use crate::systems::human_behavior::HumanEmotions;

pub fn setup_basic_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera (higher to see the massive Dubai world)
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Controls UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(400.0),
                height: Val::Auto,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BorderRadius::all(Val::Px(5.0)),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CONTROLS - Walking:\n\nArrow Keys: Move\nF: Enter Vehicle"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ControlsDisplay,
                ControlsText,
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ));
        });

    // Dynamic Sun Light
    commands.spawn((
        SunLight,
        DirectionalLight {
            illuminance: 10000.0,
            color: Color::srgb(1.0, 0.9, 0.7),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // DYNAMIC TERRAIN
    commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4000.0, 4000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(2000.0, 0.1, 2000.0),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
    ));

    // Player character with human-like components
    let player_entity = commands.spawn((
        Player,
        ActiveEntity,
        RigidBody::Dynamic,
        Collider::capsule_y(0.8, 0.4),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP),
        Damping { linear_damping: 2.0, angular_damping: 5.0 }, // Reduced damping for more natural movement
    )).id();
    
    // Add human behavior components separately
    commands.entity(player_entity).insert((
        HumanMovement::default(),
        HumanAnimation::default(),
        HumanBehavior::default(),
        PlayerBody::default(),
        FootstepTimer::default(),
        HumanEmotions::default(),
    ));

    // Human-like body parts
    
    // Torso
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 0.8, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.4, 0.8))), // Blue shirt
        Transform::from_xyz(0.0, 0.6, 0.0),
        ChildOf(player_entity),
        PlayerTorso,
        BodyPart {
            rest_position: Vec3::new(0.0, 0.6, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        VisibleChildBundle::default(),
    ));

    // Head
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))), // Skin tone
        Transform::from_xyz(0.0, 1.2, 0.0),
        ChildOf(player_entity),
        PlayerHead,
        BodyPart {
            rest_position: Vec3::new(0.0, 1.2, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        VisibleChildBundle::default(),
    ));

    // Left Arm
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.08, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))), // Skin tone
        Transform::from_xyz(-0.4, 0.7, 0.0),
        ChildOf(player_entity),
        PlayerLeftArm,
        BodyPart {
            rest_position: Vec3::new(-0.4, 0.7, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        VisibleChildBundle::default(),
    ));

    // Right Arm
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.08, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))), // Skin tone
        Transform::from_xyz(0.4, 0.7, 0.0),
        ChildOf(player_entity),
        PlayerRightArm,
        BodyPart {
            rest_position: Vec3::new(0.4, 0.7, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        VisibleChildBundle::default(),
    ));

    // Left Leg
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.12, 0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.6))), // Dark blue pants
        Transform::from_xyz(-0.15, 0.0, 0.0),
        ChildOf(player_entity),
        PlayerLeftLeg,
        BodyPart {
            rest_position: Vec3::new(-0.15, 0.0, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        VisibleChildBundle::default(),
    ));

    // Right Leg
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.12, 0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.6))), // Dark blue pants
        Transform::from_xyz(0.15, 0.0, 0.0),
        ChildOf(player_entity),
        PlayerRightLeg,
        BodyPart {
            rest_position: Vec3::new(0.15, 0.0, 0.0),
            rest_rotation: Quat::IDENTITY,
            animation_offset: Vec3::ZERO,
            animation_rotation: Quat::IDENTITY,
        },
        VisibleChildBundle::default(),
    ));

    // Feet (Left)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))), // Black shoes
        Transform::from_xyz(-0.15, -0.4, 0.1),
        ChildOf(player_entity),
        VisibleChildBundle::default(),
    ));

    // Feet (Right)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))), // Black shoes
        Transform::from_xyz(0.15, -0.4, 0.1),
        ChildOf(player_entity),
        VisibleChildBundle::default(),
    ));

    // Waypoint UI
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(300.0),
                height: Val::Auto,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            BorderRadius::all(Val::Px(5.0)),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Waypoints loading..."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                WaypointText,
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            ));
        });
}
