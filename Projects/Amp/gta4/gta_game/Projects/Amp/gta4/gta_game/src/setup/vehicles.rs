use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::bundles::{VehicleVisibilityBundle, VisibleChildBundle};

pub fn setup_basic_vehicles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Basic car for testing
    let car_entity = commands.spawn((
        Car,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        ExternalForce::default(),
        Transform::from_xyz(15.0, 0.5, 8.0),
        VehicleVisibilityBundle::default(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
    )).id();

    // Car body
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 3.6))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(car_entity),
        Cullable { max_distance: 300.0, is_culled: false },
        VisibleChildBundle::default(),
    ));

    // BUGATTI CHIRON SUPERCAR
    let chiron_entity = commands.spawn((
        Transform::from_xyz(3.0, 1.3, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        Velocity::zero(),
        ExternalForce::default(),
        Friction::coefficient(0.3),
        Restitution::coefficient(0.0),
        Ccd::enabled(),
        Car,
        SuperCar {
            max_speed: 120.0,
            acceleration: 40.0,
            turbo_boost: false,
            exhaust_timer: 0.0,
        },
        Cullable { max_distance: 800.0, is_culled: false },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        VehicleVisibilityBundle::default(),
    )).id();

    // Main body (lower chassis)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 0.4, 4.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.15),
            metallic: 0.95,
            perceptual_roughness: 0.1,
            reflectance: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Upper body/cabin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.5, 2.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.15),
            metallic: 0.95,
            perceptual_roughness: 0.1,
            reflectance: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.25, -0.3),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Front hood
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.7, 0.15, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.15),
            metallic: 0.95,
            perceptual_roughness: 0.1,
            reflectance: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.12, 1.6),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Windshield
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 0.8, 0.1))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.3, 0.4, 0.8),
            metallic: 0.1,
            perceptual_roughness: 0.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.4, 0.8).with_rotation(Quat::from_rotation_x(-0.2)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Side windows
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.6, 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.3, 0.4, 0.8),
            metallic: 0.1,
            perceptual_roughness: 0.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.75, 0.3, -0.3),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.6, 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.3, 0.4, 0.8),
            metallic: 0.1,
            perceptual_roughness: 0.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(-0.75, 0.3, -0.3),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Front grille
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.2, 0.6, 0.05))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.05, 2.1),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Headlights
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 1.0),
            emissive: Color::srgb(0.5, 0.5, 0.8).into(),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(0.6, 0.0, 2.0),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 1.0),
            emissive: Color::srgb(0.5, 0.5, 0.8).into(),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(-0.6, 0.0, 2.0),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Rear spoiler
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.1, 0.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.02, 0.1),
            metallic: 0.9,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.6, -1.8),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Exhaust pipes
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.08, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.4, -0.25, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.08, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(-0.4, -0.25, -2.0).with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Wheels - Front Left
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.35, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(1.0, -0.35, 1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Rim - Front Left
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.25, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.9),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(1.05, -0.35, 1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Wheels - Front Right
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.35, 0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(-1.0, -0.35, 1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Rim - Front Right
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.25, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.9),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(-1.05, -0.35, 1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Wheels - Rear Left
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.4, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(1.0, -0.35, -1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Rim - Rear Left
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.35))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.9),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(1.05, -0.35, -1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Wheels - Rear Right
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.4, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(-1.0, -0.35, -1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Rim - Rear Right
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.35))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.9),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(-1.05, -0.35, -1.2).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(chiron_entity),
        VisibleChildBundle::default(),
    ));

    // Add beacon for Bugatti Chiron - BRIGHT CYAN BEACON
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.8, 15.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 1.0),
            emissive: Color::srgb(0.0, 5.0, 5.0).into(),
            ..default()
        })),
        Transform::from_xyz(3.0, 8.0, 0.0),
        VehicleBeacon,
    ));
}

pub fn setup_helicopter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // HELICOPTER - Spawn a luxury Dubai police helicopter
    let helicopter_entity = commands.spawn((
        Helicopter,
        RigidBody::Dynamic,
        Collider::cuboid(1.5, 1.0, 3.0),
        Velocity::zero(),
        ExternalForce::default(),
        Transform::from_xyz(120.0, 15.0, 80.0).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        VehicleVisibilityBundle::default(),
        Ccd::enabled(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 2.0, angular_damping: 8.0 },
    )).id();

    // Main helicopter body (sleek design)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.5, 1.5, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.9))), // Bright white/silver body
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // Cockpit (front glass section)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.2, 1.2, 1.8))),
        MeshMaterial3d(materials.add(Color::srgba(0.1, 0.1, 0.2, 0.3))), // Dark tinted glass
        Transform::from_xyz(0.0, 0.2, 1.5),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // ROTATING Main rotor (top blade) - 4 blades
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI / 2.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(10.0, 0.05, 0.2))), // Long thin blade
            MeshMaterial3d(materials.add(Color::srgb(0.05, 0.05, 0.05))), // Dark blade
            Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle)),
            ChildOf(helicopter_entity),
            MainRotor,
            VisibleChildBundle::default(),
        ));
    }

    // Rotor hub (center)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))), // Dark hub
        Transform::from_xyz(0.0, 2.0, 0.0),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // Tail boom (long back section) - sleeker design
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 0.6, 4.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.85, 0.85))), // Light gray
        Transform::from_xyz(0.0, 0.0, -4.5),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // ROTATING Tail rotor (side blade) - 3 blades  
    for i in 0..3 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 3.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.08, 2.2, 0.15))), // Vertical blade
            MeshMaterial3d(materials.add(Color::srgb(0.05, 0.05, 0.05))), // Dark blade
            Transform::from_xyz(-1.0, 0.5, -6.5).with_rotation(Quat::from_rotation_z(angle)),
            ChildOf(helicopter_entity),
            TailRotor,
            VisibleChildBundle::default(),
        ));
    }

    // Tail rotor hub
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.15, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))), // Dark hub
        Transform::from_xyz(-1.0, 0.5, -6.5).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // Landing skids (2 runners underneath) - more realistic
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.08, 3.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.6, 0.6, 0.6))), // Light gray skids
        Transform::from_xyz(-0.8, -1.0, 0.0),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.08, 3.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.6, 0.6, 0.6))), // Light gray skids
        Transform::from_xyz(0.8, -1.0, 0.0),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // Add Dubai police styling stripes
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 0.3, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.5, 0.8))), // Blue stripe
        Transform::from_xyz(0.0, 0.0, 1.0),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 0.3, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.0, 0.0))), // Red stripe
        Transform::from_xyz(0.0, -0.4, 1.0),
        ChildOf(helicopter_entity),
        VisibleChildBundle::default(),
    ));

    // Add beacon for Helicopter
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0), // Green beacon
            emissive: Color::srgb(0.0, 2.0, 0.0).into(),
            ..default()
        })),
        Transform::from_xyz(120.0, 23.0, 80.0),
        VehicleBeacon,
    ));
}

pub fn setup_f16(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // F16 FIGHTER JET - Spawn an advanced military aircraft
    let f16_entity = commands.spawn((
        F16,
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 1.5, 1.5),
        LockedAxes::empty(), // Full 6DOF movement for realistic flight
        Velocity::zero(),
        ExternalForce::default(),
        Transform::from_xyz(80.0, 2.0, 120.0), // Spawn at airfield location, separated from helicopter
        VehicleVisibilityBundle::default(),
        Cullable { max_distance: 2000.0, is_culled: false },
    )).id();

    // F16 Main fuselage (sleek fighter design)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 2.0, 3.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.5), // Military gray
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // F16 Wings (delta wing configuration)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 0.3, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.5),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(-2.0, -0.2, 0.0),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // F16 Nose cone
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 1.0, 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.4),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(9.0, 0.2, 0.0),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // F16 Vertical tail
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 4.0, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.4, 0.5))),
        Transform::from_xyz(-6.0, 1.5, 0.0),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // F16 Engine exhaust (glowing when afterburner active)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.8, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.3),
            metallic: 0.9,
            perceptual_roughness: 0.1,
            emissive: LinearRgba::new(0.0, 0.0, 0.0, 1.0), // Will glow blue when afterburner active
            ..default()
        })),
        Transform::from_xyz(-8.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // F16 Landing gear (retractable)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.3, 1.5, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_xyz(2.0, -1.2, 1.5),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.3, 1.5, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_xyz(2.0, -1.2, -1.5),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // Front landing gear
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 1.2, 0.25))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_xyz(6.0, -1.0, 0.0),
        ChildOf(f16_entity),
        VisibleChildBundle::default(),
    ));

    // Add beacon for F16
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0), // Red beacon
            emissive: Color::srgb(2.0, 0.0, 0.0).into(),
            ..default()
        })),
        Transform::from_xyz(80.0, 10.0, 120.0),
        VehicleBeacon,
    ));
}
