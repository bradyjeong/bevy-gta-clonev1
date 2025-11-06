use crate::bundles::VisibleChildBundle;
use crate::components::MovementTracker;
use crate::components::{
    ActiveEntity, BodyPart, ControlState, ControlsDisplay, ControlsText, DynamicTerrain,
    HumanAnimation, HumanMovement, MainCamera, Player, PlayerBody, PlayerControlled, PlayerHead,
    PlayerLeftArm, PlayerLeftLeg, PlayerRightArm, PlayerRightLeg, PlayerTorso, UnderwaterSettings,
    VehicleControlType,
};
use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;
use crate::factories::spawn_bridge;
use crate::systems::audio::FootstepTimer;

use crate::systems::spawn_validation::{SpawnRegistry, SpawnableType};
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup_basic_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    config: Res<GameConfig>,
    env: Res<WorldEnvConfig>,
) {
    // Config drift guards: Verify consistency between WorldEnvConfig and GameConfig
    // These assertions prevent bugs where road_network.rs and world.rs disagree on island boundaries
    debug_assert!(
        (config.world_env.islands.grid_x - env.islands.grid_x).abs() < 0.001,
        "Config drift detected: grid_x mismatch between GameConfig ({}) and WorldEnvConfig ({})",
        config.world_env.islands.grid_x,
        env.islands.grid_x
    );
    debug_assert!(
        (config.world_env.islands.grid_z - env.islands.grid_z).abs() < 0.001,
        "Config drift detected: grid_z mismatch between GameConfig ({}) and WorldEnvConfig ({})",
        config.world_env.islands.grid_z,
        env.islands.grid_z
    );
    debug_assert!(
        (config.world_env.terrain.half_size * 2.0 - env.terrain.size).abs() < 0.001,
        "Config drift detected: terrain size mismatch between GameConfig (half_size {} -> {}) and WorldEnvConfig (size {})",
        config.world_env.terrain.half_size,
        config.world_env.terrain.half_size * 2.0,
        env.terrain.size
    );
    debug_assert!(
        (config.world_env.islands.left_x - env.islands.left_x).abs() < 0.001,
        "Config drift detected: left_x mismatch between GameConfig ({}) and WorldEnvConfig ({})",
        config.world_env.islands.left_x,
        env.islands.left_x
    );
    debug_assert!(
        (config.world_env.islands.right_x - env.islands.right_x).abs() < 0.001,
        "Config drift detected: right_x mismatch between GameConfig ({}) and WorldEnvConfig ({})",
        config.world_env.islands.right_x,
        env.islands.right_x
    );
    debug_assert!(
        (config.world_env.terrain.half_size - env.terrain.half_size).abs() < 0.001,
        "Config drift detected: terrain half_size mismatch between GameConfig ({}) and WorldEnvConfig ({})",
        config.world_env.terrain.half_size,
        env.terrain.half_size
    );

    // No longer need WorldRoot - spawn entities directly in world space

    // Camera (stays outside WorldRoot - doesn't move with world shifts)
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Msaa::Off,
        DepthPrepass,
        Camera {
            hdr: true, // Enable HDR for particle bloom effects
            ..default()
        },
        Tonemapping::AcesFitted, // Tone mapping for HDR display
        Bloom {
            intensity: 0.05, // Very subtle bloom for flame cores only
            low_frequency_boost: 0.0,
            low_frequency_boost_curvature: 0.0,
            high_pass_frequency: 1.0,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            // Extended for proper horizon/skybox rendering (skybox at 9500 units)
            far: 10000.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        UnderwaterSettings {
            sea_level: env.sea_level,
            // Research-based realistic ocean parameters:
            // - Red light absorbed in top 10m
            // - Blue/green penetrate deepest
            // - At 10m depth: only 16% light remains
            fog_density: 0.25, // Moderate density for clear ocean (0.1-0.5 range)
            absorption: Vec3::new(0.8, 0.3, 0.15), // RED >> GREEN > BLUE (realistic attenuation)
            scatter_color: Vec3::new(0.02, 0.35, 0.48), // Deep blue-cyan (clear ocean at depth)
            enabled: 1,
        },
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
        Vec3::new(env.islands.left_x, env.land_elevation, 0.0),
        env.terrain.size,
        "Left",
        &config,
    );

    // RIGHT TERRAIN ISLAND
    spawn_terrain_island(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(env.islands.right_x, env.land_elevation, 0.0),
        env.terrain.size,
        "Right",
        &config,
    );

    // OCEAN FLOOR - Extended to horizon for proper water depth rendering
    // Matches water surface bounds (18000x18000) to prevent see-through at distance
    let ocean_size = 18000.0;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(ocean_size, ocean_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, env.ocean_floor_depth, 0.0),
        Name::new("Ocean Floor Visual"),
    ));

    // Ocean floor collider - derive from ocean_size to match visual bounds
    // ocean_size is derived from world_bounds, so collider scales with world
    // Collider center at ocean_floor_depth - 0.05 so top surface is exactly ocean_floor_depth (matches beach slope bottom)
    commands.spawn((
        Transform::from_xyz(0.0, env.ocean_floor_depth - 0.05, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(ocean_size * 0.5, 0.05, ocean_size * 0.5),
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Name::new("Ocean Floor Collision"),
    ));

    // WORLD BOUNDARY MARKERS (visible on minimap at Y=5 for visibility)
    // Derived from config: world_half_size (3000.0) + edge_buffer (200.0) = 3200.0
    let boundary_size = config.world_bounds.world_half_size + config.world_bounds.edge_buffer;
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
        Vec3::new(env.islands.left_x, env.land_elevation, 0.0),
        env.terrain.size,
        "Left",
        &config,
    );

    // RIGHT TERRAIN BEACHES (all 4 edges with corners filled)
    spawn_terrain_beaches(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(env.islands.right_x, env.land_elevation, 0.0),
        env.terrain.size,
        "Right",
        &config,
    );

    // GRID TERRAIN ISLAND (Manhattan-style grid roads)
    spawn_terrain_island(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(env.islands.grid_x, env.land_elevation, env.islands.grid_z),
        env.terrain.size,
        "Grid",
        &config,
    );

    // GRID TERRAIN BEACHES (all 4 edges with corners filled)
    spawn_terrain_beaches(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(env.islands.grid_x, env.land_elevation, env.islands.grid_z),
        env.terrain.size,
        "Grid",
        &config,
    );

    // BRIDGE CONNECTING ISLANDS
    spawn_bridge(&mut commands, &mut meshes, &mut materials, &config, &env);

    // Spawn player above terrain, let gravity drop them
    let player_y = env.land_elevation + env.spawn_drop_height;

    // Player character with human-like components in world coordinates
    // Use player dimensions from config
    let player_dims = &config.character_dimensions.player;

    let player_entity = commands
        .spawn((
            Player,
            ActiveEntity,
            RigidBody::Dynamic,
            Collider::capsule(
                Vec3::new(0.0, player_dims.lower_sphere_y(), 0.0),
                Vec3::new(0.0, player_dims.upper_sphere_y, 0.0),
                player_dims.capsule_radius,
            ),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Velocity::zero(),
            Transform::from_xyz(env.islands.left_x, player_y, 0.0),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            CollisionGroups::new(
                config.physics.character_group,
                config.physics.static_group | config.physics.vehicle_group,
            ),
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
        MovementTracker::new(Vec3::new(env.islands.left_x, env.land_elevation, 0.0), 5.0),
        ControlState::default(),
        PlayerControlled,
        VehicleControlType::Walking,
        crate::components::unified_water::CurrentWaterRegion {
            region_entity: None,
        },
    ));

    spawn_registry.register_entity(
        player_entity,
        Vec3::new(env.islands.left_x, player_y, 0.0),
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

    // Foot meshes aligned with capsule foot_level from config
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(-0.15, player_dims.foot_level, 0.1),
        ChildOf(player_entity),
        crate::components::player::PlayerLeftFoot,
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.35))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(0.15, player_dims.foot_level, 0.1),
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
    config: &GameConfig,
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
    config: &GameConfig,
) {
    use crate::factories::beach_terrain::{
        CornerType, create_beach_slope, create_beach_slope_collider, create_corner_beach_slope,
        create_corner_beach_slope_collider,
    };

    let collider_half_height = 0.05; // Match terrain collider
    let land_elevation_phys = terrain_center.y; // Physics top surface
    let ocean_floor_y = config.world_env.ocean_floor_depth;
    let beach_width = config.world_env.terrain.beach_width;
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
        CollisionGroups::new(
            config.physics.static_group,
            config.physics.vehicle_group | config.physics.character_group,
        ),
        Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Max,
        },
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
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
