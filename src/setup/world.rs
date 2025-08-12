use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::GlobalRng;
use crate::constants::*;
use crate::bundles::VisibleChildBundle;
use crate::systems::audio::FootstepTimer;
use crate::systems::human_behavior::HumanEmotions;
use crate::systems::spawn_validation::{SpawnRegistry, SpawnableType};
use crate::services::distance_cache::MovementTracker;
use crate::services::ground_detection::GroundDetectionService;

pub fn setup_basic_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground_service: Res<GroundDetectionService>,
    mut global_rng: ResMut<GlobalRng>,
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



    // DYNAMIC TERRAIN - Below roads with proper collision separation
    commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4000.0, 4000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))),
        Transform::from_xyz(0.0, -0.15, 0.0), // 15cm below road surface at y=0.0
        RigidBody::Fixed,
        Collider::cuboid(2000.0, 0.05, 2000.0), // Terrain collider: -0.2 to -0.1
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP), // All entities collide with terrain
    ));

    // Calculate proper ground position for player spawn
    let player_spawn_pos = Vec2::new(0.0, 0.0);
    let ground_height = ground_service.get_ground_height_simple(player_spawn_pos);
    let player_y = ground_height + 0.45; // Position so feet (at -0.4) touch ground
    
    println!("DEBUG: Player spawn - ground height: {:.3}, final Y: {:.3}", ground_height, player_y);
    
    // Player character with human-like components
    let player_entity = commands.spawn((
        Player,
        ActiveEntity,
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -0.4, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.4),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(0.0, player_y, 0.0),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP),
        Damping { linear_damping: 1.2, angular_damping: 3.5 }, // Balanced damping to prevent overspin
    )).id();
    
    // Add human behavior components separately
    commands.entity(player_entity).insert((
        HumanMovement::default(),
        HumanAnimation::new(global_rng.gen_range(3.0..8.0)),
        HumanBehavior::new(
            global_rng.gen_range(0.95..1.05),
            global_rng.gen_range(0.95..1.05), 
            global_rng.gen_range(0.8..1.0)
        ),
        PlayerBody::default(),
        FootstepTimer::new(global_rng.gen_range(0.45..0.55)),
        HumanEmotions::default(),
        MovementTracker::new(Vec3::new(0.0, 1.0, 0.0), 5.0), // Track movement with 5m threshold
        // Control components for new input system
        ControlState::default(),
        PlayerControlled,
        VehicleControlType::Walking,
    ));
    
    // Register player in spawn registry
    spawn_registry.register_entity(player_entity, Vec3::new(0.0, player_y, 0.0), SpawnableType::Player);

    // Human-like body parts - spawn as children
    commands.entity(player_entity).with_children(|parent| {
        // Torso
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.6, 0.8, 0.3))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.4, 0.8))), // Blue shirt
            Transform::from_xyz(0.0, 0.6, 0.0),
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
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.2))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))), // Skin tone
            Transform::from_xyz(0.0, 1.2, 0.0),
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
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.08, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))), // Skin tone
            Transform::from_xyz(-0.4, 0.7, 0.0),
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
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.08, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))), // Skin tone
            Transform::from_xyz(0.4, 0.7, 0.0),
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
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.12, 0.6))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.6))), // Dark blue pants
            Transform::from_xyz(-0.15, 0.0, 0.0),
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
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.12, 0.6))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.6))), // Dark blue pants
            Transform::from_xyz(0.15, 0.0, 0.0),
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
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
            MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))), // Black shoes
            Transform::from_xyz(-0.15, -0.4, 0.1),
            VisibleChildBundle::default(),
        ));

        // Feet (Right)
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
            MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))), // Black shoes
            Transform::from_xyz(0.15, -0.4, 0.1),
            VisibleChildBundle::default(),
        ));
    });

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
            illuminance: 25000.0, // Balanced golden hour sun
            color: Color::srgb(1.0, 0.8, 0.5), // Warm golden sunset light
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y), // Lower angle golden hour sun
    ));
}
