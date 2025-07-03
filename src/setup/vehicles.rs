use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;

/// Luxury color schemes for the Bugatti Chiron
#[derive(Clone, Copy)]
pub enum BugattiColorScheme {
    MidnightBlue,     // Deep blue with subtle glow
    PlatinumSilver,   // Bright metallic silver
    CarbonBlack,      // Matte black with carbon fiber
    ChampagneGold,    // Luxurious gold finish
    RacingRed,        // Vibrant red with gloss
}

impl BugattiColorScheme {
    pub fn get_material(&self) -> StandardMaterial {
        match self {
            BugattiColorScheme::MidnightBlue => StandardMaterial {
                base_color: Color::srgb(0.08, 0.12, 0.2),
                metallic: 0.98,
                perceptual_roughness: 0.02,
                reflectance: 0.98,
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.01,
                emissive: LinearRgba::rgb(0.02, 0.05, 0.1),
                ..default()
            },
            BugattiColorScheme::PlatinumSilver => StandardMaterial {
                base_color: Color::srgb(0.9, 0.92, 0.95),
                metallic: 0.99,
                perceptual_roughness: 0.01,
                reflectance: 0.99,
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.005,
                emissive: LinearRgba::rgb(0.1, 0.1, 0.12),
                ..default()
            },
            BugattiColorScheme::CarbonBlack => StandardMaterial {
                base_color: Color::srgb(0.05, 0.05, 0.05),
                metallic: 0.95,
                perceptual_roughness: 0.08,
                reflectance: 0.9,
                clearcoat: 0.8,
                clearcoat_perceptual_roughness: 0.15,
                emissive: LinearRgba::rgb(0.01, 0.01, 0.01),
                ..default()
            },
            BugattiColorScheme::ChampagneGold => StandardMaterial {
                base_color: Color::srgb(0.8, 0.65, 0.3),
                metallic: 0.97,
                perceptual_roughness: 0.03,
                reflectance: 0.95,
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.02,
                emissive: LinearRgba::rgb(0.05, 0.04, 0.02),
                ..default()
            },
            BugattiColorScheme::RacingRed => StandardMaterial {
                base_color: Color::srgb(0.8, 0.05, 0.05),
                metallic: 0.96,
                perceptual_roughness: 0.02,
                reflectance: 0.95,
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.01,
                emissive: LinearRgba::rgb(0.08, 0.01, 0.01),
                ..default()
            },
        }
    }
}

/// Simplified vehicle setup - no deprecated factories
pub fn setup_simple_vehicles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Basic car
    let car_entity = commands.spawn((
        Car,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),  // Half-height = 0.5, total height = 1.0
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(15.0, 0.5, 0.0),  // Fixed: spawn at proper ground height
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        Cullable { max_distance: 300.0, is_culled: false },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Car body - Fixed: height matches collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),  // Fixed: height 1.0 matches collider
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(car_entity),
    ));

    // Supercar
    let supercar_entity = commands.spawn((
        Car,
        SuperCar::default(),
        RigidBody::Dynamic,
        Collider::cuboid(1.1, 0.5, 2.4),  // Half-height = 0.5, total height = 1.0
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(8.0, 0.5, 0.0),  // Safe distance from player spawn
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        Cullable { max_distance: 800.0, is_culled: false },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Supercar body - Premium luxury paint with color scheme
    let color_scheme = BugattiColorScheme::MidnightBlue; // Default to midnight blue
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 4.5))),
        MeshMaterial3d(materials.add(color_scheme.get_material())),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(supercar_entity),
    ));

    // Carbon fiber accents - side panels
    for side in [-1.0, 1.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.05, 0.6, 3.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.15, 0.15, 0.15), // Carbon fiber black
                metallic: 0.9,
                perceptual_roughness: 0.1,
                reflectance: 0.8,
                ..default()
            })),
            Transform::from_xyz(side * 1.0, 0.0, 0.0),
            ChildOf(supercar_entity),
        ));
    }

    // Chrome exhaust pipes
    for side in [-0.6, 0.6] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.08, 0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.95, 0.98), // Chrome
                metallic: 1.0,
                perceptual_roughness: 0.01,
                reflectance: 0.95,
                ..default()
            })),
            Transform::from_xyz(side, -0.2, -2.4)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(supercar_entity),
        ));
    }

    // LED headlights
    for side in [-0.7, 0.7] {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.95, 1.0), // Cool white
                emissive: LinearRgba::rgb(0.8, 0.9, 1.0), // Bright LED glow
                metallic: 0.0,
                perceptual_roughness: 0.1,
                ..default()
            })),
            Transform::from_xyz(side, 0.3, 2.0),
            ChildOf(supercar_entity),
        ));
    }

    // Red LED taillights
    for side in [-0.6, 0.6] {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.08))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.05, 0.05), // Deep red
                emissive: LinearRgba::rgb(1.0, 0.1, 0.1), // Bright red glow
                metallic: 0.0,
                perceptual_roughness: 0.2,
                ..default()
            })),
            Transform::from_xyz(side, 0.2, -2.0),
            ChildOf(supercar_entity),
        ));
    }

    // Aerodynamic wing spoiler
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 0.08, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.15), // Carbon fiber
            metallic: 0.9,
            perceptual_roughness: 0.1,
            reflectance: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.6, -1.8),
        ChildOf(supercar_entity),
    ));

    // Windshield with tinting
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.02, 1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.2, 0.3, 0.7), // Tinted glass
            metallic: 0.0,
            perceptual_roughness: 0.05,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.4, 0.8)
            .with_rotation(Quat::from_rotation_x(-0.3)),
        ChildOf(supercar_entity),
    ));
}

/// Simplified helicopter setup
pub fn setup_simple_helicopter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let helicopter_entity = commands.spawn((
        Helicopter,
        RigidBody::Dynamic,
        Collider::cuboid(1.5, 1.0, 3.0),  // Half-height = 1.0, total height = 2.0
        Velocity::zero(),
        Transform::from_xyz(15.0, 1.0, 15.0),  // Safe distance from player spawn
        Damping { linear_damping: 2.0, angular_damping: 8.0 },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Helicopter body - Realistic shape using capsule
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.8, 4.0))),  // Helicopter fuselage shape
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.28, 0.3), // Military gunmetal
            metallic: 0.8,
            perceptual_roughness: 0.4,
            reflectance: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(helicopter_entity),
    ));
    
    // Cockpit bubble - rounded cockpit
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.05, 0.05, 0.08, 0.15),
            metallic: 0.1,
            perceptual_roughness: 0.1,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.2, 1.5).with_scale(Vec3::new(1.2, 0.8, 1.0)),
        ChildOf(helicopter_entity),
    ));
    
    // Tail boom - tapered cylinder
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.25, 3.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.28, 0.3),
            metallic: 0.8,
            perceptual_roughness: 0.4,
            reflectance: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 4.5),
        ChildOf(helicopter_entity),
    ));
    
    // Main rotor blades - thin and aerodynamic
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI / 2.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(8.0, 0.02, 0.3))),  // Long thin blade
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.08, 0.08, 0.08),
                metallic: 0.2,
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle)),
            ChildOf(helicopter_entity),
            MainRotor,
        ));
    }
    
    // Landing skids - long narrow cylinders
    for x in [-0.8, 0.8] {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.04, 3.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.35, 0.35),
                metallic: 0.7,
                perceptual_roughness: 0.6,
                ..default()
            })),
            Transform::from_xyz(x, -1.0, 0.0).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ChildOf(helicopter_entity),
        ));
    }
}

/// Simplified F16 setup
pub fn setup_simple_f16(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let f16_entity = commands.spawn((
        F16,
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 1.5, 1.5),  // Half-height = 1.5, total height = 3.0
        LockedAxes::empty(),
        Velocity::zero(),
        Transform::from_xyz(80.0, 1.5, 120.0),  // Fixed: spawn at ground+half-height
        Cullable { max_distance: 2000.0, is_culled: false },
    )).id();

    // F16 body - Fixed: dimensions match collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0, 3.0, 3.0))),  // Fixed: height 3.0 matches collider
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.5),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(f16_entity),
    ));
}

/// Enhanced Bugatti Chiron with premium customization options
pub fn setup_luxury_bugatti_chiron(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    color_scheme: BugattiColorScheme,
    position: Vec3,
) {
    // Create the luxury Bugatti entity
    let bugatti_entity = commands.spawn((
        Car,
        SuperCar::default(),
        RigidBody::Dynamic,
        Collider::cuboid(1.1, 0.5, 2.4),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_translation(position),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        Cullable { max_distance: 1000.0, is_culled: false },
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
    )).id();

    // Main body with chosen luxury color scheme
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 4.5))),
        MeshMaterial3d(materials.add(color_scheme.get_material())),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(bugatti_entity),
    ));

    // Premium carbon fiber side panels
    for side in [-1.0, 1.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.05, 0.6, 3.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.12),
                metallic: 0.95,
                perceptual_roughness: 0.08,
                reflectance: 0.85,
                clearcoat: 0.9,
                clearcoat_perceptual_roughness: 0.05,
                ..default()
            })),
            Transform::from_xyz(side * 1.0, 0.0, 0.0),
            ChildOf(bugatti_entity),
        ));
    }

    // Quad chrome exhaust system (W16 engine)
    for x_pos in [-0.6, -0.2, 0.2, 0.6].iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.08, 0.35))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.96, 0.96, 0.98),
                metallic: 1.0,
                perceptual_roughness: 0.005,
                reflectance: 0.98,
                emissive: LinearRgba::rgb(0.02, 0.02, 0.02),
                ..default()
            })),
            Transform::from_xyz(*x_pos, -0.2, -2.4)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ChildOf(bugatti_entity),
        ));
    }

    // High-intensity LED headlights
    for side in [-0.7, 0.7] {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.12))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.98, 1.0),
                emissive: LinearRgba::rgb(1.2, 1.4, 1.6),
                metallic: 0.0,
                perceptual_roughness: 0.05,
                ..default()
            })),
            Transform::from_xyz(side, 0.3, 2.0),
            ChildOf(bugatti_entity),
        ));
    }

    // Dynamic LED taillights
    for side in [-0.6, 0.6] {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.09))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.02, 0.02),
                emissive: LinearRgba::rgb(1.5, 0.15, 0.15),
                metallic: 0.0,
                perceptual_roughness: 0.1,
                ..default()
            })),
            Transform::from_xyz(side, 0.2, -2.0),
            ChildOf(bugatti_entity),
        ));
    }

    // Active aerodynamic rear wing
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.1, 0.35))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.92,
            perceptual_roughness: 0.06,
            reflectance: 0.88,
            clearcoat: 0.95,
            clearcoat_perceptual_roughness: 0.03,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.65, -1.8),
        ChildOf(bugatti_entity),
    ));

    // Premium tinted windshield
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.7, 0.02, 1.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.08, 0.15, 0.25, 0.75),
            metallic: 0.0,
            perceptual_roughness: 0.02,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.42, 0.9)
            .with_rotation(Quat::from_rotation_x(-0.25)),
        ChildOf(bugatti_entity),
    ));

    // Side mirrors with LED indicators
    for side in [-1.0, 1.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.08, 0.05, 0.12))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.1, 0.1),
                metallic: 0.85,
                perceptual_roughness: 0.1,
                reflectance: 0.8,
                ..default()
            })),
            Transform::from_xyz(side * 1.1, 0.4, 1.2),
            ChildOf(bugatti_entity),
        ));
    }

    // Front splitter for aerodynamics
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 0.04, 0.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.08, 0.08),
            metallic: 0.9,
            perceptual_roughness: 0.05,
            reflectance: 0.85,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.4, 2.3),
        ChildOf(bugatti_entity),
    ));
}
