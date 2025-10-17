use crate::bundles::VisibleChildBundle;
use crate::components::MovementTracker;
use crate::components::{
    ActiveEntity, BodyPart, ControlState, ControlsDisplay, ControlsText, DynamicTerrain,
    HumanAnimation, HumanMovement, MainCamera, Player, PlayerBody, PlayerControlled, PlayerHead,
    PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg, PlayerTorso, VehicleControlType,
};
use crate::constants::{
    CHARACTER_GROUP, LAND_ELEVATION, LEFT_ISLAND_X, RIGHT_ISLAND_X, SPAWN_DROP_HEIGHT,
    STATIC_GROUP, TERRAIN_SIZE, VEHICLE_GROUP,
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
        Projection::Perspective(PerspectiveProjection {
            far: 3500.0, // Covers world boundaries at ±3200m with buffer
            ..default()
        }),
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

    // DUAL RECTANGULAR ISLAND CONFIGURATION
    // Using centralized constants from constants.rs
    // LEFT TERRAIN ISLAND
    spawn_terrain_island(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(LEFT_ISLAND_X, LAND_ELEVATION, 0.0),
        TERRAIN_SIZE,
        "Left",
    );

    // RIGHT TERRAIN ISLAND
    spawn_terrain_island(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(RIGHT_ISLAND_X, LAND_ELEVATION, 0.0),
        TERRAIN_SIZE,
        "Right",
    );

    // OCEAN FLOOR - Fills entire world
    let ocean_size = 6400.0; // Match world bounds at ±3200m
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
        Transform::from_xyz(0.0, -10.2, 0.0), // Below heightfield ocean floor to prevent co-planar collision
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

    // LEFT TERRAIN BEACHES (all 4 edges with corners filled)
    spawn_terrain_beaches(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(LEFT_ISLAND_X, LAND_ELEVATION, 0.0),
        TERRAIN_SIZE,
        "Left",
    );

    // RIGHT TERRAIN BEACHES (all 4 edges with corners filled)
    spawn_terrain_beaches(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(RIGHT_ISLAND_X, LAND_ELEVATION, 0.0),
        TERRAIN_SIZE,
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
            Transform::from_xyz(LEFT_ISLAND_X, player_y, 0.0),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP),
            Ccd::enabled(),
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
        MovementTracker::new(Vec3::new(LEFT_ISLAND_X, LAND_ELEVATION, 0.0), 5.0),
        ControlState::default(),
        PlayerControlled,
        VehicleControlType::Walking,
    ));

    spawn_registry.register_entity(
        player_entity,
        Vec3::new(LEFT_ISLAND_X, player_y, 0.0),
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

    // Foot meshes aligned with capsule FOOT_LEVEL (-0.45)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(-0.15, FOOT_LEVEL, 0.1),
        ChildOf(player_entity),
        crate::components::player::PlayerLeftFoot,
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(0.15, FOOT_LEVEL, 0.1),
        ChildOf(player_entity),
        crate::components::player::PlayerRightFoot,
        VisibleChildBundle::default(),
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_terrain_island(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    size: f32,
    name: &str,
) {
    let half_size = size / 2.0;
    let collider_half_height = 0.05;

    commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))),
        // Lower visual mesh by half collider height to align top surface with physics
        Transform::from_translation(position - Vec3::Y * collider_half_height),
        RigidBody::Fixed,
        Collider::cuboid(half_size, collider_half_height, half_size),
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
    use crate::factories::beach_terrain::{
        create_beach_slope, create_beach_slope_collider, create_corner_beach_slope,
        create_corner_beach_slope_collider, CornerType,
    };

    let collider_half_height = 0.05; // Match terrain collider
    let land_elevation_phys = terrain_center.y; // Physics top surface
    let ocean_floor_y = -10.0;
    let beach_width = 100.0;
    let half_size = terrain_size / 2.0;

    // Beach center Y for physics (collider alignment)
    let beach_center_y_phys = (land_elevation_phys + ocean_floor_y) / 2.0;
    // Beach center Y for visuals (lowered to match terrain visual pattern)
    let beach_center_y_visual = beach_center_y_phys - collider_half_height;

    // Shared beach material
    let beach_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.75, 0.6),
        perceptual_roughness: 0.7,
        ..default()
    });

    // SMOOTH BEACH PHYSICS: Use create_beach_slope_collider (2 triangles) for smooth collision
    // Visual mesh: 32 subdivisions for smooth appearance
    // Collider mesh: Simple sloped quad (2 triangles) prevents jolting

    // EAST BEACH (+X direction) - mesh naturally slopes +X, NO rotation needed
    let east_visual = create_beach_slope(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
        32,
    );
    let east_collider_mesh = create_beach_slope_collider(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
    );

    let east_pos_visual = Vec3::new(
        terrain_center.x + half_size + beach_width / 2.0,
        beach_center_y_visual,
        terrain_center.z,
    );
    let east_pos_collider = Vec3::new(
        terrain_center.x + half_size + beach_width / 2.0,
        beach_center_y_phys,
        terrain_center.z,
    );

    commands.spawn((
        Mesh3d(meshes.add(east_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(east_pos_visual),
        Name::new(format!("{name} East Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(east_pos_collider),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&east_collider_mesh, &default())
            .expect("Beach collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} East Beach Collider")),
    ));

    // WEST BEACH (-X direction) - flip 180° to slope toward -X
    let west_visual = create_beach_slope(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
        32,
    );
    let west_collider_mesh = create_beach_slope_collider(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
    );

    let west_pos_visual = Vec3::new(
        terrain_center.x - (half_size + beach_width / 2.0),
        beach_center_y_visual,
        terrain_center.z,
    );
    let west_pos_collider = Vec3::new(
        terrain_center.x - (half_size + beach_width / 2.0),
        beach_center_y_phys,
        terrain_center.z,
    );
    let west_rotation = Quat::from_rotation_y(std::f32::consts::PI);

    commands.spawn((
        Mesh3d(meshes.add(west_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(west_pos_visual).with_rotation(west_rotation),
        Name::new(format!("{name} West Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(west_pos_collider).with_rotation(west_rotation),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&west_collider_mesh, &default())
            .expect("Beach collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} West Beach Collider")),
    ));

    // NORTH BEACH (+Z direction) - rotate -90° clockwise so X→-Z
    let north_visual = create_beach_slope(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
        32,
    );
    let north_collider_mesh = create_beach_slope_collider(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
    );

    let north_pos_visual = Vec3::new(
        terrain_center.x,
        beach_center_y_visual,
        terrain_center.z + half_size + beach_width / 2.0,
    );
    let north_pos_collider = Vec3::new(
        terrain_center.x,
        beach_center_y_phys,
        terrain_center.z + half_size + beach_width / 2.0,
    );
    let north_rotation = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);

    commands.spawn((
        Mesh3d(meshes.add(north_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(north_pos_visual).with_rotation(north_rotation),
        Name::new(format!("{name} North Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(north_pos_collider).with_rotation(north_rotation),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&north_collider_mesh, &default())
            .expect("Beach collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} North Beach Collider")),
    ));

    // SOUTH BEACH (-Z direction) - rotate +90° counterclockwise so X→Z
    let south_visual = create_beach_slope(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
        32,
    );
    let south_collider_mesh = create_beach_slope_collider(
        beach_width,
        terrain_size,
        land_elevation_phys,
        ocean_floor_y,
    );

    let south_pos_visual = Vec3::new(
        terrain_center.x,
        beach_center_y_visual,
        terrain_center.z - (half_size + beach_width / 2.0),
    );
    let south_pos_collider = Vec3::new(
        terrain_center.x,
        beach_center_y_phys,
        terrain_center.z - (half_size + beach_width / 2.0),
    );
    let south_rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);

    commands.spawn((
        Mesh3d(meshes.add(south_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(south_pos_visual).with_rotation(south_rotation),
        Name::new(format!("{name} South Beach Visual")),
    ));

    commands.spawn((
        Transform::from_translation(south_pos_collider).with_rotation(south_rotation),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&south_collider_mesh, &default())
            .expect("Beach collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Name::new(format!("{name} South Beach Collider")),
    ));

    // CORNER BEACHES: Fill 100m × 100m gaps at island corners
    // Slight overlap with side beaches to prevent visual gaps
    let corner_size = 100.0;
    let corner_offset = half_size + corner_size / 2.0 - 0.1; // Subtract 0.1m for overlap

    // NORTHEAST CORNER (+X, +Z)
    let ne_visual = create_corner_beach_slope(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        20,
        CornerType::NorthEast,
    );
    let ne_collider_mesh = create_corner_beach_slope_collider(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        CornerType::NorthEast,
    );

    let ne_pos_visual = Vec3::new(
        terrain_center.x + corner_offset,
        beach_center_y_visual,
        terrain_center.z + corner_offset,
    );
    let ne_pos_collider = Vec3::new(
        terrain_center.x + corner_offset,
        beach_center_y_phys,
        terrain_center.z + corner_offset,
    );

    commands.spawn((
        Mesh3d(meshes.add(ne_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(ne_pos_visual),
        Name::new(format!("{name} NE Corner Visual")),
    ));

    commands.spawn((
        Transform::from_translation(ne_pos_collider),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&ne_collider_mesh, &default())
            .expect("Corner collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Ccd::enabled(),
        Name::new(format!("{name} NE Corner Collider")),
    ));

    // NORTHWEST CORNER (-X, +Z)
    let nw_visual = create_corner_beach_slope(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        20,
        CornerType::NorthWest,
    );
    let nw_collider_mesh = create_corner_beach_slope_collider(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        CornerType::NorthWest,
    );

    let nw_pos_visual = Vec3::new(
        terrain_center.x - corner_offset,
        beach_center_y_visual,
        terrain_center.z + corner_offset,
    );
    let nw_pos_collider = Vec3::new(
        terrain_center.x - corner_offset,
        beach_center_y_phys,
        terrain_center.z + corner_offset,
    );

    commands.spawn((
        Mesh3d(meshes.add(nw_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(nw_pos_visual),
        Name::new(format!("{name} NW Corner Visual")),
    ));

    commands.spawn((
        Transform::from_translation(nw_pos_collider),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&nw_collider_mesh, &default())
            .expect("Corner collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Ccd::enabled(),
        Name::new(format!("{name} NW Corner Collider")),
    ));

    // SOUTHEAST CORNER (+X, -Z)
    let se_visual = create_corner_beach_slope(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        20,
        CornerType::SouthEast,
    );
    let se_collider_mesh = create_corner_beach_slope_collider(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        CornerType::SouthEast,
    );

    let se_pos_visual = Vec3::new(
        terrain_center.x + corner_offset,
        beach_center_y_visual,
        terrain_center.z - corner_offset,
    );
    let se_pos_collider = Vec3::new(
        terrain_center.x + corner_offset,
        beach_center_y_phys,
        terrain_center.z - corner_offset,
    );

    commands.spawn((
        Mesh3d(meshes.add(se_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(se_pos_visual),
        Name::new(format!("{name} SE Corner Visual")),
    ));

    commands.spawn((
        Transform::from_translation(se_pos_collider),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&se_collider_mesh, &default())
            .expect("Corner collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Ccd::enabled(),
        Name::new(format!("{name} SE Corner Collider")),
    ));

    // SOUTHWEST CORNER (-X, -Z)
    let sw_visual = create_corner_beach_slope(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        20,
        CornerType::SouthWest,
    );
    let sw_collider_mesh = create_corner_beach_slope_collider(
        corner_size,
        land_elevation_phys,
        ocean_floor_y,
        CornerType::SouthWest,
    );

    let sw_pos_visual = Vec3::new(
        terrain_center.x - corner_offset,
        beach_center_y_visual,
        terrain_center.z - corner_offset,
    );
    let sw_pos_collider = Vec3::new(
        terrain_center.x - corner_offset,
        beach_center_y_phys,
        terrain_center.z - corner_offset,
    );

    commands.spawn((
        Mesh3d(meshes.add(sw_visual)),
        MeshMaterial3d(beach_material.clone()),
        Transform::from_translation(sw_pos_visual),
        Name::new(format!("{name} SW Corner Visual")),
    ));

    commands.spawn((
        Transform::from_translation(sw_pos_collider),
        RigidBody::Fixed,
        Collider::from_bevy_mesh(&sw_collider_mesh, &default())
            .expect("Corner collider creation failed"),
        CollisionGroups::new(STATIC_GROUP, VEHICLE_GROUP | CHARACTER_GROUP),
        Ccd::enabled(),
        Name::new(format!("{name} SW Corner Collider")),
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

    // Note: Bevy 0.16 doesn't have built-in fog - would need external crate like bevy_atmosphere
    // For now, rely on natural atmospheric perspective from 3km far plane
}
