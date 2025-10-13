use crate::bundles::VisibleChildBundle;
use crate::components::MovementTracker;
use crate::components::{
    ActiveEntity, BodyPart, ControlState, ControlsDisplay, ControlsText, DynamicTerrain,
    HumanAnimation, HumanMovement, MainCamera, Player, PlayerBody, PlayerControlled, PlayerHead,
    PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg, PlayerTorso, VehicleControlType,
};
use crate::constants::{
    CHARACTER_GROUP, LAND_ELEVATION, SEA_LEVEL, SPAWN_DROP_HEIGHT, STATIC_GROUP, VEHICLE_GROUP,
};

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

    // DUAL ISLAND CONFIGURATION
    // World bounds remain unchanged, terrain is shrunk and positioned as islands
    let terrain_size = 1200.0; // Shrunk from 4096m to 1200m

    // Left terrain position (vertically centered, with margin from left edge)
    let left_terrain_x = -1500.0; // Positioned left of center with margin

    // Right terrain position (mirror of left)
    let right_terrain_x = 1500.0; // Positioned right of center with margin

    // LEFT TERRAIN ISLAND
    spawn_terrain_island(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(left_terrain_x, LAND_ELEVATION, 0.0),
        terrain_size,
        "Left",
    );

    // RIGHT TERRAIN ISLAND
    spawn_terrain_island(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(right_terrain_x, LAND_ELEVATION, 0.0),
        terrain_size,
        "Right",
    );

    // OCEAN FLOOR - Fills entire world
    let ocean_size = 10000.0; // Large ocean covering entire world
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(ocean_size, ocean_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -10.0, 0.0),
        Name::new("Ocean Floor Visual"),
    ));

    commands.spawn((
        Transform::from_xyz(0.0, -10.05, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(ocean_size / 2.0, 0.05, ocean_size / 2.0),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new("Ocean Floor Collision"),
    ));

    // WORLD BOUNDARY MARKERS (visible on minimap at Y=5 for visibility)
    let boundary_size = 3200.0; // World bounds at ±3200m
    let boundary_color = Color::srgb(1.0, 0.0, 0.0); // Red boundary lines
    let boundary_thickness = 20.0; // Thick enough to see on minimap

    // North boundary (Z = +3200)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(boundary_size * 2.0, 2.0, boundary_thickness))),
        MeshMaterial3d(materials.add(boundary_color)),
        Transform::from_xyz(0.0, 5.0, boundary_size),
        Name::new("World Boundary North"),
    ));

    // South boundary (Z = -3200)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(boundary_size * 2.0, 2.0, boundary_thickness))),
        MeshMaterial3d(materials.add(boundary_color)),
        Transform::from_xyz(0.0, 5.0, -boundary_size),
        Name::new("World Boundary South"),
    ));

    // East boundary (X = +3200)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(boundary_thickness, 2.0, boundary_size * 2.0))),
        MeshMaterial3d(materials.add(boundary_color)),
        Transform::from_xyz(boundary_size, 5.0, 0.0),
        Name::new("World Boundary East"),
    ));

    // West boundary (X = -3200)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(boundary_thickness, 2.0, boundary_size * 2.0))),
        MeshMaterial3d(materials.add(boundary_color)),
        Transform::from_xyz(-boundary_size, 5.0, 0.0),
        Name::new("World Boundary West"),
    ));

    // LEFT TERRAIN BEACHES (all 4 edges)
    spawn_terrain_beaches(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(left_terrain_x, LAND_ELEVATION, 0.0),
        terrain_size,
        "Left",
    );

    // RIGHT TERRAIN BEACHES (all 4 edges)
    spawn_terrain_beaches(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(right_terrain_x, LAND_ELEVATION, 0.0),
        terrain_size,
        "Right",
    );

    // Spawn player above terrain, let gravity drop them
    let player_y = LAND_ELEVATION + SPAWN_DROP_HEIGHT;

    // Player character with human-like components in world coordinates
    const FOOT_LEVEL: f32 = -0.45;
    const CAPSULE_RADIUS: f32 = 0.25;
    const LOWER_SPHERE_Y: f32 = FOOT_LEVEL + CAPSULE_RADIUS;
    const UPPER_SPHERE_Y: f32 = 1.45;

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
            Transform::from_xyz(left_terrain_x, player_y, 0.0),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP),
            Damping {
                linear_damping: 0.1,
                angular_damping: 3.5,
            },
        ))
        .id();

    commands.entity(player_entity).insert((
        HumanMovement::default(),
        HumanAnimation::default(),
        PlayerBody::default(),
        FootstepTimer::default(),
        MovementTracker::new(Vec3::new(left_terrain_x, LAND_ELEVATION, 0.0), 5.0),
        ControlState::default(),
        PlayerControlled,
        VehicleControlType::Walking,
    ));

    spawn_registry.register_entity(
        player_entity,
        Vec3::new(left_terrain_x, player_y, 0.0),
        SpawnableType::Player,
    );

    // Human-like body parts
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 0.8, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.4, 0.8))),
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

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))),
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

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.08, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))),
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

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.08, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.5))),
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

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.12, 0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.6))),
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

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.12, 0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.6))),
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

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(-0.15, -0.4, 0.1),
        ChildOf(player_entity),
        crate::components::player::PlayerLeftFoot,
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(0.15, -0.4, 0.1),
        ChildOf(player_entity),
        crate::components::player::PlayerRightFoot,
        VisibleChildBundle::default(),
    ));
}

fn spawn_terrain_island(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    size: f32,
    name: &str,
) {
    let half_size = size / 2.0;

    commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))),
        Transform::from_translation(position),
        RigidBody::Fixed,
        Collider::cuboid(half_size, 0.05, half_size),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} Terrain Island")),
    ));
}

fn spawn_terrain_beaches(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain_center: Vec3,
    terrain_size: f32,
    name: &str,
) {
    use crate::factories::create_beach_slope;

    let land_elevation = terrain_center.y; // Extract land elevation from terrain center
    let beach_width = 100.0;
    let half_size = terrain_size / 2.0;

    // NORTH BEACH (positive Z) - slope extends outward in +Z direction
    let north_beach = create_beach_slope(beach_width, terrain_size, land_elevation, SEA_LEVEL, 32);
    let north_center = Vec3::new(
        terrain_center.x,
        0.0,
        terrain_center.z + half_size + beach_width / 2.0,
    );

    commands.spawn((
        Mesh3d(meshes.add(north_beach.clone())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.75, 0.6),
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_translation(north_center)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::PI / 2.0)),
        Name::new(format!("{name} North Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(north_center)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::PI / 2.0)),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&north_beach, &default()).unwrap(),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} North Beach Collision")),
    ));

    // SOUTH BEACH (negative Z) - slope extends outward in -Z direction
    let south_beach = create_beach_slope(beach_width, terrain_size, land_elevation, SEA_LEVEL, 32);
    let south_center = Vec3::new(
        terrain_center.x,
        0.0,
        terrain_center.z - (half_size + beach_width / 2.0),
    );

    commands.spawn((
        Mesh3d(meshes.add(south_beach.clone())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.75, 0.6),
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_translation(south_center)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0)),
        Name::new(format!("{name} South Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(south_center)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 2.0)),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&south_beach, &default()).unwrap(),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} South Beach Collision")),
    ));

    // EAST BEACH (positive X)
    let east_beach = create_beach_slope(beach_width, terrain_size, land_elevation, SEA_LEVEL, 32);
    let east_center = Vec3::new(
        terrain_center.x + half_size + beach_width / 2.0,
        0.0,
        terrain_center.z,
    );

    commands.spawn((
        Mesh3d(meshes.add(east_beach.clone())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.75, 0.6),
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_translation(east_center),
        Name::new(format!("{name} East Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(east_center),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&east_beach, &default()).unwrap(),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} East Beach Collision")),
    ));

    // WEST BEACH (negative X)
    let west_beach = create_beach_slope(beach_width, terrain_size, land_elevation, SEA_LEVEL, 32);
    let west_center = Vec3::new(
        terrain_center.x - (half_size + beach_width / 2.0),
        0.0,
        terrain_center.z,
    );

    commands.spawn((
        Mesh3d(meshes.add(west_beach.clone())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.75, 0.6),
            perceptual_roughness: 0.7,
            ..default()
        })),
        Transform::from_translation(west_center)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        Name::new(format!("{name} West Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(west_center)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&west_beach, &default()).unwrap(),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} West Beach Collision")),
    ));
}

/// Setup Dubai golden hour lighting
pub fn setup_dubai_noon_lighting(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 25000.0,
            color: Color::srgb(1.0, 0.8, 0.5),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
