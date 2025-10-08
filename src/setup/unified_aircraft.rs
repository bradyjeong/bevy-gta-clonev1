// HighSpeed component removed - no longer needed for finite world
use crate::components::{
    AircraftFlight, F16, Helicopter, MainRotor, SimpleF16Specs, SimpleHelicopterSpecs, TailRotor,
    VehicleState, VehicleType,
};
use crate::constants::{CHARACTER_GROUP, STATIC_GROUP, VEHICLE_GROUP};
use crate::factories::{MaterialFactory, MeshFactory};
use crate::services::ground_detection::GroundDetectionService;
use crate::systems::spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType};
use bevy::{prelude::*, render::view::VisibilityRange};
use bevy_rapier3d::prelude::*;

use crate::GameConfig;
use crate::components::MovementTracker;

/// Aircraft types supported by the unified system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AircraftType {
    Helicopter, // Complex multi-component aircraft
    F16,        // Simple fighter jet
}

/// Unified aircraft setup system that replaces individual aircraft setup functions
/// - Replaces setup_simple_helicopter (src/setup/vehicles.rs line 219)
/// - Replaces setup_simple_f16 (src/setup/vehicles.rs line 310)
///
/// KEY IMPROVEMENTS:
/// - Ground detection using GroundDetectionService for proper positioning
/// - Spawn validation using SpawnValidator/SpawnRegistry
/// - Consistent bundle patterns similar to DynamicVehicleBundle
/// - Far visibility culling distances for aircraft
/// - Proper collision groups for aircraft physics
pub fn setup_initial_aircraft_unified(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_registry: ResMut<SpawnRegistry>,
    ground_service: Res<GroundDetectionService>,
    _config: Res<GameConfig>,
) {
    // Aircraft spawn positions (well-spaced from other content)
    let aircraft_spawns = [
        (Vec3::new(15.0, 0.0, 15.0), AircraftType::Helicopter),
        (Vec3::new(80.0, 0.0, 120.0), AircraftType::F16),
    ];

    let mut spawned_aircraft = Vec::new();

    for (preferred_pos, aircraft_type) in aircraft_spawns {
        if let Some(aircraft_entity) = spawn_aircraft_unified(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut spawn_registry,
            &ground_service,
            preferred_pos,
            aircraft_type,
        ) {
            spawned_aircraft.push(aircraft_entity);
        }
    }

    info!(
        "Unified aircraft setup complete - Spawned {} aircraft",
        spawned_aircraft.len()
    );
}

/// Spawn a single aircraft with ground detection and validation
fn spawn_aircraft_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_registry: &mut ResMut<SpawnRegistry>,
    ground_service: &Res<GroundDetectionService>,
    preferred_position: Vec3,
    aircraft_type: AircraftType,
) -> Option<Entity> {
    // Get ground height for proper positioning
    let ground_height = ground_service
        .get_ground_height_simple(Vec2::new(preferred_position.x, preferred_position.z));

    // Calculate spawn position (aircraft spawn at appropriate height above ground)
    let spawn_height = match aircraft_type {
        AircraftType::Helicopter => 1.2, // Helicopter collider half-height (1.2) + small margin
        AircraftType::F16 => 1.0, // F16 cuboid half-height (1.0) to ensure bottom touches ground
    };

    let spawn_position = Vec3::new(
        preferred_position.x,
        ground_height + spawn_height,
        preferred_position.z,
    );

    // Validate spawn position
    let safe_position = SpawnValidator::spawn_entity_safely(
        spawn_registry,
        spawn_position,
        SpawnableType::Vehicle,
        Entity::PLACEHOLDER, // Will be updated after spawn
    );

    if let Some(validated_position) = safe_position {
        let aircraft_entity = match aircraft_type {
            AircraftType::Helicopter => {
                spawn_helicopter_unified(commands, meshes, materials, validated_position)
            }
            AircraftType::F16 => spawn_f16_unified(commands, meshes, materials, validated_position),
        };

        // Update spawn registry with actual entity
        spawn_registry.update_entity_position(aircraft_entity, validated_position);

        Some(aircraft_entity)
    } else {
        warn!(
            "Failed to find safe spawn position for {:?} aircraft",
            aircraft_type
        );
        None
    }
}

/// Spawn helicopter with complex multi-component structure
fn spawn_helicopter_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) -> Entity {
    // Create helicopter entity with unified bundle pattern
    let helicopter_entity = commands
        .spawn((
            // Core aircraft components
            Helicopter,
            SimpleHelicopterSpecs::default(),
            // Physics components
            RigidBody::Dynamic,
            Collider::cuboid(1.2, 1.2, 4.8), // 0.8x visual mesh (3x3x12) for GTA-style forgiving collision
            Velocity::zero(),
            Transform::from_translation(position),
            Damping {
                linear_damping: 3.0,
                angular_damping: 10.0,
            },
            // Visibility components (required for child inheritance)
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            // Collision and culling
            CollisionGroups::new(
                VEHICLE_GROUP,
                STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP,
            ),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
            // Movement tracking
            MovementTracker::new(position, 50.0),
        ))
        .id();

    // Helicopter body - Realistic shape using capsule
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.8, 4.0))), // Helicopter fuselage shape
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.28, 0.3), // Military gunmetal
            metallic: 0.8,
            perceptual_roughness: 0.4,
            reflectance: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(helicopter_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
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
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
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
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    // Main rotor blades - thin and aerodynamic
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI / 2.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(8.0, 0.02, 0.3))), // Long thin blade
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.08, 0.08, 0.08),
                metallic: 0.2,
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle)),
            ChildOf(helicopter_entity),
            MainRotor,
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
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
            Transform::from_xyz(x, -1.0, 0.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ChildOf(helicopter_entity),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
        ));
    }

    // Tail rotor at end of tail boom
    commands.spawn((
        Mesh3d(MeshFactory::create_tail_rotor(meshes)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.08, 0.08),
            metallic: 0.2,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 6.2),
        ChildOf(helicopter_entity),
        TailRotor,
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    helicopter_entity
}

/// Spawn F16 with simple aircraft structure
fn spawn_f16_unified(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
) -> Entity {
    // Create F16 entity with unified bundle pattern
    let f16_entity = commands
        .spawn((
            F16,
            AircraftFlight::default(),
            SimpleF16Specs::default(),
            VehicleState::new(VehicleType::F16),
            RigidBody::Dynamic,
            Collider::cuboid(1.6, 1.0, 6.4),
            LockedAxes::empty(),
            Velocity::zero(),
            ExternalForce::default(),
            Transform::from_translation(position),
            Visibility::default(),
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ))
        .insert((
            CollisionGroups::new(
                VEHICLE_GROUP,
                STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP,
            ),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 450.0..550.0,
                use_aabb: false,
            },
            MovementTracker::new(position, 50.0),
        ))
        .id();

    // F16 main fuselage - using dedicated F16 mesh factory
    let fuselage_mesh = MeshFactory::create_f16_body(meshes);
    let fuselage_material = MaterialFactory::create_f16_fuselage_material(materials);

    commands.spawn((
        Mesh3d(fuselage_mesh),
        MeshMaterial3d(fuselage_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(f16_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    // F16 wings (left and right)
    let wing_mesh = MeshFactory::create_f16_wing(meshes);
    let wing_material = MaterialFactory::create_f16_fuselage_material(materials);

    // Left wing (positioned relative to new Z-axis fuselage)
    commands.spawn((
        Mesh3d(wing_mesh.clone()),
        MeshMaterial3d(wing_material.clone()),
        Transform::from_xyz(-5.0, 0.0, -2.0).with_rotation(Quat::from_rotation_y(0.2)), // Swept wing
        ChildOf(f16_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    // Right wing (positioned relative to new Z-axis fuselage)
    commands.spawn((
        Mesh3d(wing_mesh),
        MeshMaterial3d(wing_material),
        Transform::from_xyz(5.0, 0.0, -2.0).with_rotation(Quat::from_rotation_y(-0.2)), // Swept wing
        ChildOf(f16_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    // F16 canopy (bubble cockpit)
    let canopy_mesh = MeshFactory::create_f16_canopy(meshes);
    let canopy_material = MaterialFactory::create_f16_canopy_material(materials);

    commands.spawn((
        Mesh3d(canopy_mesh),
        MeshMaterial3d(canopy_material),
        Transform::from_xyz(0.0, 0.8, 3.0), // Forward position along +Z, raised
        ChildOf(f16_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    // F16 vertical tail
    let tail_mesh = MeshFactory::create_f16_vertical_tail(meshes);
    let tail_material = MaterialFactory::create_f16_fuselage_material(materials);

    commands.spawn((
        Mesh3d(tail_mesh),
        MeshMaterial3d(tail_material),
        Transform::from_xyz(0.0, 1.0, -5.0), // Rear position along -Z, raised
        ChildOf(f16_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    // Engine nozzle for visual effect
    let engine_mesh = meshes.add(Cylinder::new(0.8, 2.0));
    let engine_material = MaterialFactory::create_f16_engine_material(materials);

    commands.spawn((
        Mesh3d(engine_mesh),
        MeshMaterial3d(engine_material),
        Transform::from_xyz(0.0, 0.0, -8.0), // Rear nozzle along -Z
        ChildOf(f16_entity),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 450.0..550.0,
            use_aabb: false,
        },
    ));

    f16_entity
}
