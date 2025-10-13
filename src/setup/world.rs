use crate::bundles::VisibleChildBundle;
use crate::components::MovementTracker;
use crate::components::{
    ActiveEntity, BodyPart, ControlState, ControlsDisplay, ControlsText, DynamicTerrain,
    HumanAnimation, HumanMovement, MainCamera, Player, PlayerBody, PlayerControlled, PlayerHead,
    PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg, PlayerTorso, VehicleControlType,
};
use crate::constants::{CHARACTER_GROUP, STATIC_GROUP, VEHICLE_GROUP};

use crate::systems::audio::FootstepTimer;

use crate::systems::spawn_validation::{SpawnRegistry, SpawnableType};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup_basic_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
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

    // OCEAN FLOOR - Starts exactly where beach slope ends (no overlap)
    // Beach ends at X: 2148m at Y=-10, ocean floor must align perfectly
    let ocean_floor_width = 500.0;
    let ocean_floor_start_x = 2148.0; // Exactly where beach ends
    let ocean_floor_center_x = ocean_floor_start_x + (ocean_floor_width / 2.0); // 2148 + 250 = 2398m
    
    // Visual mesh at Y=-10
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(ocean_floor_width, 6000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(ocean_floor_center_x, -10.0, 0.0),
        Name::new("Ocean Floor East Visual"),
    ));
    
    // Thin collision at Y=-10 (top surface) to match beach slope end exactly
    commands.spawn((
        Transform::from_xyz(ocean_floor_center_x, -10.05, 0.0), // Top at -10.0
        RigidBody::Fixed,
        Collider::cuboid(ocean_floor_width / 2.0, 0.05, 3000.0), // Thin like terrain
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new("Ocean Floor East Collision"),
    ));

    // BEACH SLOPE - Inside the ocean area (X: 2048m to 2148m)
    use crate::factories::create_beach_slope;
    let beach_slope_width = 100.0; // Width of beach transition in ocean
    let beach_slope_depth = 6000.0; // Length along coastline
    
    // Position so left edge aligns with terrain edge at X: 2048m
    let beach_center_x = 2048.0 + (beach_slope_width / 2.0); // 2048 + 50 = 2098m

    // Create mesh for both visual and collision
    let beach_mesh = create_beach_slope(
        beach_slope_width,
        beach_slope_depth,
        0.0,   // Shore height (at water surface)
        -10.0, // Ocean floor height
        32,    // Subdivisions for smooth slope
    );

    // Visual mesh
    commands.spawn((
        Mesh3d(meshes.add(beach_mesh.clone())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.75, 0.6), // Sandy underwater color
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_xyz(beach_center_x, 0.0, 0.0), // Aligned with terrain edge
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        Name::new("Eastern Beach Slope Visual"),
    ));

    // BEACH COLLISION - Use mesh collider for smooth transition (no seams)
    commands.spawn((
        Transform::from_xyz(beach_center_x, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&beach_mesh, &default()).unwrap(), // Exact mesh collision
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new("Eastern Beach Collision"),
    ));

    // LAKE BEACH - Circular beach around the central lake
    use crate::factories::create_circular_beach_ring;
    let lake_center = Vec3::new(300.0, 0.0, 300.0);
    let lake_radius = 100.0; // Inner radius (water edge)
    let beach_ring = create_circular_beach_ring(
        lake_radius,
        lake_radius + 30.0, // Outer radius (30m beach width)
        lake_center,
        0.0,  // Land height
        -0.8, // Water edge (slightly below lake surface at 1.2)
        32,   // Radial segments for smooth circle
        16,   // Height segments for smooth slope
    );

    commands.spawn((
        Mesh3d(meshes.add(beach_ring)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.96, 0.82), // Bright tropical sand
            perceptual_roughness: 0.6,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        Name::new("Lake Beach Ring Visual"),
    ));

    // LAKE BEACH COLLISION - 8 angled planes forming octagonal ring
    // Simple approximation of circular slope (1° gentle slope)
    let lake_beach_width = 30.0_f32;
    let lake_beach_height_change = 0.8_f32;
    let lake_slope_angle = (lake_beach_height_change / lake_beach_width).atan();
    let lake_mid_height = -0.4_f32;
    let lake_mid_radius = lake_radius + 15.0_f32;

    // Create 8 angled segments around the lake
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::TAU / 8.0;
        let segment_x = lake_center.x + lake_mid_radius * angle.cos();
        let segment_z = lake_center.z + lake_mid_radius * angle.sin();

        commands.spawn((
            Transform::from_xyz(segment_x, lake_mid_height, segment_z).with_rotation(
                Quat::from_rotation_y(angle) // Point outward from center
                    * Quat::from_rotation_z(lake_slope_angle), // Slope downward
            ),
            RigidBody::Fixed,
            Collider::cuboid(15.0, 0.05, 25.0), // Thin segment
            CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
            Name::new(format!("Lake Beach Collision Segment {}", i)),
        ));
    }

    // Spawn player above ground, let gravity drop them
    let player_y = 10.0;

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
