use crate::bundles::VisibleChildBundle;
use crate::components::MovementTracker;
use crate::components::{
    ActiveEntity, BodyPart, ControlState, ControlsDisplay, ControlsText, DynamicTerrain,
    HumanAnimation, HumanMovement, MainCamera, Player, PlayerBody, PlayerControlled, PlayerHead,
    PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg, PlayerTorso, VehicleControlType,
};
use crate::constants::{CHARACTER_GROUP, STATIC_GROUP, VEHICLE_GROUP};
use crate::services::ground_detection::GroundDetectionService;
use crate::systems::audio::FootstepTimer;

use crate::systems::spawn_validation::{SpawnRegistry, SpawnableType};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup_basic_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground_service: Res<GroundDetectionService>,
) {
    // No longer need WorldRoot - spawn entities directly in world space

    // Camera (stays outside WorldRoot - doesn't move with world shifts)
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Camera in direct world coordinates
    ));

    // Controls UI (stays outside WorldRoot - UI doesn't move with world shifts)
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
            // UI in screen space
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

    // FINITE TERRAIN - Ground plane for finite world (4km x 4km)
    commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4096.0, 4096.0))), // 4km x 4km
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))),
        Transform::from_xyz(0.0, 0.0, 0.0), // Ground at origin
        RigidBody::Fixed,
        Collider::cuboid(2048.0, 0.05, 2048.0), // 2km radius terrain collider
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP), // All entities collide with terrain
    ));

    // OCEAN BOUNDARY - Removed for procedural terrain implementation

    // Calculate proper ground position for player spawn
    let player_spawn_pos = Vec2::new(0.0, 0.0);
    let ground_height = ground_service.get_ground_height_simple(player_spawn_pos);
    // Distance from transform origin to collider bottom: -(-0.45) = 0.45
    let player_y = ground_height + 0.45; // Position so collider bottom touches ground

    println!("DEBUG: Player spawn - ground height: {ground_height:.3}, final Y: {player_y:.3}",);

    // Player character with human-like components in world coordinates
    // Align collider bottom with visual feet at y=-0.45
    const FOOT_LEVEL: f32 = -0.45;
    const CAPSULE_RADIUS: f32 = 0.25; // Slimmer for better door navigation
    const LOWER_SPHERE_Y: f32 = FOOT_LEVEL + CAPSULE_RADIUS; // -0.20
    const UPPER_SPHERE_Y: f32 = 1.45; // ~1.70m total height

    let player_entity = commands
        .spawn((
            Player,
            ActiveEntity,
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, LOWER_SPHERE_Y, 0.0),
                Vec3::new(0.0, UPPER_SPHERE_Y, 0.0),
                CAPSULE_RADIUS,
            ),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Velocity::zero(),
            Transform::from_xyz(0.0, player_y, 0.0),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP),
            Damping {
                linear_damping: 0.1, // Realistic air resistance for free-fall
                angular_damping: 3.5,
            }, // Balanced damping to prevent overspin
        ))
        .id();

    // Player moves freely in world coordinates

    // Add human behavior components separately
    commands.entity(player_entity).insert((
        HumanMovement::default(),
        HumanAnimation::default(),
        PlayerBody::default(),
        FootstepTimer::default(),
        MovementTracker::new(Vec3::new(0.0, 1.0, 0.0), 5.0), // Track movement with 5m threshold
        // Control components for new input system
        ControlState::default(),
        PlayerControlled,
        VehicleControlType::Walking,
    ));

    // Register player in spawn registry
    spawn_registry.register_entity(
        player_entity,
        Vec3::new(0.0, player_y, 0.0),
        SpawnableType::Player,
    );

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
        crate::components::player::PlayerLeftFoot,
        VisibleChildBundle::default(),
    ));

    // Feet (Right)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))), // Black shoes
        Transform::from_xyz(0.15, -0.4, 0.1),
        ChildOf(player_entity),
        crate::components::player::PlayerRightFoot,
        VisibleChildBundle::default(),
    ));

    // Waypoint UI - DISABLED to clean up top right corner
    /*
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
    */
}

/// Setup Dubai golden hour lighting
pub fn setup_dubai_noon_lighting(mut commands: Commands) {
    // Golden hour sun for that cinematic Dubai look
    commands.spawn((
        DirectionalLight {
            illuminance: 25000.0,              // Balanced golden hour sun
            color: Color::srgb(1.0, 0.8, 0.5), // Warm golden sunset light
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y), // Lower angle golden hour sun
    ));
}
