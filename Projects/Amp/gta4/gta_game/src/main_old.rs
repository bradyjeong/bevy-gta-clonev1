use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use rand::Rng;

// Collision groups for proper physics separation
const STATIC_GROUP: Group = Group::GROUP_1;    // Buildings, terrain, trees
const VEHICLE_GROUP: Group = Group::GROUP_2;   // Cars, helicopters, jets
const CHARACTER_GROUP: Group = Group::GROUP_3; // Player, NPCs

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_state::<GameState>()
        .init_resource::<CullingSettings>()
        .init_resource::<PerformanceStats>()
        .insert_resource(ClearColor(Color::srgb(0.85, 0.9, 1.0))) // Light desert sky
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_movement.run_if(in_state(GameState::Walking)),
            car_movement.run_if(in_state(GameState::Driving)),
            supercar_movement.run_if(in_state(GameState::Driving)),
            helicopter_movement.run_if(in_state(GameState::Flying)),

            rotate_helicopter_rotors,
            optimized_npc_movement,
            interaction_system,
            camera_follow_system,
            distance_culling_system,
            performance_monitoring_system,

            debug_player_position,
            exhaust_effects_system,
            update_waypoint_system,
            update_beacon_visibility,
        ))
        .add_systems(Update, (
            dynamic_terrain_system,
            dynamic_content_system,
            controls_ui_system,
        ))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Walking,
    Driving,
    Flying,
    Jetting, // New state for F16 flying
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Car;

#[derive(Component)]
struct SuperCar {
    max_speed: f32,
    acceleration: f32,
    turbo_boost: bool,
    exhaust_timer: f32,
}

#[derive(Component)]
struct Helicopter;

#[derive(Component)]
struct F16;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ActiveEntity;



#[derive(Component)]
struct InCar(#[allow(dead_code)] Entity);

#[derive(Component)]
struct MainRotor;

#[derive(Component)]
struct TailRotor;

#[derive(Component)]
struct NPC {
    target_position: Vec3,
    speed: f32,
    last_update: f32,
    update_interval: f32,
}

#[derive(Component)]
struct Cullable {
    max_distance: f32,
    is_culled: bool,
}

#[derive(Component)]
struct DynamicTerrain;

#[derive(Component)]
struct DynamicContent {
    content_type: ContentType,

}

#[derive(Clone, PartialEq)]
enum ContentType {
    Road,
    Building,
    Tree,
    Vehicle,
    NPC,
}

#[derive(Component)]
struct PerformanceCritical;

#[derive(Resource)]
struct CullingSettings {
    _npc_cull_distance: f32,
    _car_cull_distance: f32,
    _building_cull_distance: f32,
    _tree_cull_distance: f32,
}

impl Default for CullingSettings {
    fn default() -> Self {
        Self {
            _npc_cull_distance: 200.0,
            _car_cull_distance: 300.0,
            _building_cull_distance: 800.0,
            _tree_cull_distance: 400.0,
        }
    }
}

#[derive(Resource)]
struct PerformanceStats {
    entity_count: usize,
    culled_entities: usize,
    frame_time: f32,
    last_report: f32,
}



impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            entity_count: 0,
            culled_entities: 0,
            frame_time: 0.0,
            last_report: 0.0,
        }
    }
}

// Dynamic World System - Circular radius around player



#[derive(Component)]
struct Building;

#[derive(Component)]
struct Buildable;

#[derive(Component)]
struct SunLight;

#[derive(Component)]
struct ControlsDisplay;

// Sky System Components
#[derive(Component)]
struct SkyDome;

#[derive(Component)]
struct Clouds;

#[derive(Component)]
struct ExhaustFlame;

#[derive(Component)]
struct VehicleBeacon;

#[derive(Component)]
struct WaypointText;

#[derive(Component)]
struct ControlsText;

// Sky Setup System


fn setup(
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
            ));
        });

    // Dynamic Sun Light (will be controlled by day/night cycle)
    commands.spawn((
        SunLight,
        DirectionalLight {
            illuminance: 10000.0,
            color: Color::srgb(1.0, 0.9, 0.7), // Warm daylight
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // DYNAMIC TERRAIN - Single terrain plane that follows the player
    commands.spawn((
        DynamicTerrain,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(4000.0, 4000.0))), // 4km x 4km terrain
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.75, 0.6))), // Desert sand color
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(2000.0, 0.1, 2000.0),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
    ));

    // Roads are now completely dynamic and will be generated by the dynamic content system as you explore

    // Player character (capsule shape)
    let player_entity = commands.spawn((
        Player,
        ActiveEntity,
        RigidBody::Dynamic,
        Collider::capsule_y(0.8, 0.4),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Visibility::Visible,
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP | VEHICLE_GROUP),
        Damping { linear_damping: 5.0, angular_damping: 10.0 },
    )).id();

    // Player body
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.4, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.6, 0.4))), // Skin color
        Transform::from_xyz(0.0, 0.6, 0.0),
        ChildOf(player_entity),
    ));

    // Player head
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.25))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.6, 0.4))), // Skin color
        Transform::from_xyz(0.0, 1.2, 0.0),
        ChildOf(player_entity),
    ));

    // PALM TREES EVERYWHERE! (Dubai oasis style)
    let palm_positions = [
        // Close to spawn area
        (10.0, 15.0), (15.0, 8.0), (-12.0, 18.0), (-8.0, -14.0),
        (22.0, -16.0), (-18.0, 12.0), (25.0, 25.0), (-25.0, -25.0),
        
        // Medium distance palm groves
        (45.0, 35.0), (38.0, -42.0), (-35.0, 48.0), (-45.0, -38.0),
        (60.0, 15.0), (-55.0, 25.0), (40.0, -60.0), (-40.0, 65.0),
        
        // Far distance oasis areas
        (80.0, 90.0), (85.0, -95.0), (-90.0, 85.0), (-85.0, -90.0),
        (120.0, 45.0), (-110.0, 55.0), (95.0, -115.0), (-105.0, 125.0),
        
        // Scattered throughout the massive world
        (150.0, 80.0), (140.0, -160.0), (-145.0, 155.0), (-155.0, -145.0),
        (180.0, 25.0), (-175.0, 35.0), (165.0, -185.0), (-165.0, 175.0),
        (200.0, 120.0), (-195.0, 110.0), (190.0, -205.0), (-200.0, 195.0),
        
        // Road-side palms (near highways)
        (15.0, 200.0), (15.0, -200.0), (-15.0, 180.0), (-15.0, -180.0),
        (200.0, 15.0), (-200.0, 15.0), (180.0, -15.0), (-180.0, -15.0),
    ];

    for &(x, z) in palm_positions.iter() {
        // Palm tree trunk
        let palm_entity = commands.spawn((
            Transform::from_xyz(x, 0.0, z),
            Visibility::Visible,
        )).id();

        // Trunk (tall brown cylinder)
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.4, 0.2, 0.1))), // Brown trunk
            Transform::from_xyz(0.0, 4.0, 0.0),
            RigidBody::Fixed,
            Collider::cylinder(4.0, 0.4), // Slightly larger than visual for reliable collision
            CollisionGroups::new(STATIC_GROUP, Group::ALL),
            ChildOf(palm_entity),
            Cullable { max_distance: 400.0, is_culled: false },
        ));

        // Palm fronds (green spheres arranged like palm leaves)
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::TAU / 8.0;
            let frond_x = angle.cos() * 2.0;
            let frond_z = angle.sin() * 2.0;
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.8))),
                MeshMaterial3d(materials.add(Color::srgb(0.1, 0.6, 0.1))), // Palm green
                Transform::from_xyz(frond_x, 7.5, frond_z)
                    .with_scale(Vec3::new(2.0, 0.3, 0.8)), // Flatten to look like leaves
                ChildOf(palm_entity),
            ));
        }
    }

    // LUXURY CARS scattered across Dubai world
    let car_positions = [
        // Near spawn (avoid overlap with Bugatti at 5,0,0)
        (15.0, 0.5, 8.0), (-8.0, 0.5, 12.0), (18.0, 0.5, -15.0),
        
        // On main highways
        (0.0, 0.5, 40.0), (0.0, 0.5, -45.0), (35.0, 0.5, 0.0), (-40.0, 0.5, 0.0),
        
        // In different districts
        (65.0, 0.5, 55.0), (-70.0, 0.5, 60.0), (75.0, 0.5, -65.0), (-60.0, 0.5, -70.0),
        
        // Far luxury areas
        (120.0, 0.5, 110.0), (-125.0, 0.5, 115.0), (130.0, 0.5, -120.0), (-115.0, 0.5, -125.0),
        
        // Near palm oases
        (180.0, 0.5, 25.0), (-175.0, 0.5, 35.0), (165.0, 0.5, -185.0), (-165.0, 0.5, 175.0),
    ];
    
    let car_colors = [
        // Luxury Dubai car colors
        Color::srgb(1.0, 1.0, 1.0), // Pearl White (classic luxury)
        Color::srgb(0.1, 0.1, 0.1), // Jet Black
        Color::srgb(0.8, 0.7, 0.0), // Gold (very Dubai!)
        Color::srgb(0.7, 0.7, 0.8), // Silver Metallic
        Color::srgb(0.8, 0.1, 0.1), // Ferrari Red
        Color::srgb(0.1, 0.3, 0.8), // Royal Blue
        Color::srgb(0.2, 0.6, 0.2), // British Racing Green
        Color::srgb(0.6, 0.3, 0.8), // Purple (Lamborghini style)
        
        // More luxury colors for extra cars
        Color::srgb(0.9, 0.8, 0.7), // Champagne
        Color::srgb(0.3, 0.1, 0.0), // Deep Brown
        Color::srgb(0.8, 0.4, 0.1), // Orange (McLaren style)
        Color::srgb(0.1, 0.7, 0.9), // Turquoise
        Color::srgb(0.9, 0.9, 0.1), // Bright Yellow (Lamborghini)
        Color::srgb(0.5, 0.0, 0.5), // Deep Purple
        Color::srgb(0.0, 0.8, 0.4), // Emerald Green
        Color::srgb(0.8, 0.5, 0.8), // Rose Gold
        Color::srgb(0.2, 0.2, 0.6), // Midnight Blue
        Color::srgb(0.7, 0.1, 0.4), // Burgundy
        Color::srgb(0.9, 0.6, 0.0), // Amber
    ];

    for (i, &(x, y, z)) in car_positions.iter().enumerate() {
        // Create car parent entity with physics
        let car_entity = commands.spawn((
            Car,
            RigidBody::Dynamic,
            Collider::cuboid(1.0, 0.5, 2.0),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            Velocity::zero(),
            Transform::from_xyz(x, y + 1.0, z),
            Visibility::Visible,
        )).id();

        // Car body (main hull)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 3.6))),
            MeshMaterial3d(materials.add(car_colors[i])),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(car_entity),
            Cullable { max_distance: 300.0, is_culled: false },
        ));

        // Car roof
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.4, 0.4, 2.4))),
            MeshMaterial3d(materials.add(car_colors[i].darker(0.3))),
            Transform::from_xyz(0.0, 0.5, -0.2),
            ChildOf(car_entity),
        ));

        // Windshield
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.3, 0.35, 0.1))),
            MeshMaterial3d(materials.add(Color::srgba(0.7, 0.8, 1.0, 0.3))), // Blue tinted glass
            Transform::from_xyz(0.0, 0.5, 0.8),
            ChildOf(car_entity),
        ));

        // 4 Wheels
        let wheel_positions = [
            (-0.7, -0.3, 1.3),  // Front left
            (0.7, -0.3, 1.3),   // Front right
            (-0.7, -0.3, -1.3), // Rear left
            (0.7, -0.3, -1.3),  // Rear right
        ];

        for &(wx, wy, wz) in wheel_positions.iter() {
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(0.3, 0.2))),
                MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))), // Black wheels
                Transform::from_xyz(wx, wy, wz).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                ChildOf(car_entity),
            ));
        }

        // Headlights
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.9))), // White/yellow headlight
            Transform::from_xyz(-0.5, 0.1, 1.7),
            ChildOf(car_entity),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.9))),
            Transform::from_xyz(0.5, 0.1, 1.7),
            ChildOf(car_entity),
        ));

        // Tail lights
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.12))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.2, 0.2))), // Red tail light
            Transform::from_xyz(-0.5, 0.1, -1.7),
            ChildOf(car_entity),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.12))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.2, 0.2))),
            Transform::from_xyz(0.5, 0.1, -1.7),
            ChildOf(car_entity),
        ));
    }

    // BUGATTI CHIRON SUPERCAR - Ultra high-performance hypercar
    let chiron_entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 0.6, 4.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.15),
            metallic: 0.9,
            reflectance: 0.9,
            ..default()
        })),
        Transform::from_xyz(5.0, 1.3, 0.0), // Raised to match regular car height
        RigidBody::Dynamic,
        Collider::cuboid(1.1, 0.5, 2.4), // Made slightly larger than visual mesh for reliable collision
        Velocity::zero(),
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
    )).id();

    // Add beacon for Bugatti Chiron
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 1.0), // Cyan beacon
            emissive: Color::srgb(0.0, 2.0, 2.0).into(),
            ..default()
        })),
        Transform::from_xyz(5.0, 4.0, 0.0),
        VehicleBeacon,
    ));

    // Chiron wheels (larger, more aggressive)
    let chiron_wheel_positions = [
        (-0.9, -0.4, -1.6), // Front left
        (0.9, -0.4, -1.6),  // Front right  
        (-0.9, -0.4, 1.6),  // Rear left
        (0.9, -0.4, 1.6),   // Rear right
    ];

    for &(wx, wy, wz) in chiron_wheel_positions.iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.4, 0.25))), // Larger wheels
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.1, 0.1),
                metallic: 0.8,
                ..default()
            })),
            Transform::from_xyz(wx, wy, wz).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ChildOf(chiron_entity),
        ));
    }

    // Chiron aggressive front lights (LED style)
    for (_i, &x_pos) in [-0.7, 0.7].iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.3, 0.1, 0.1))), // Rectangular LED style
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.9, 0.95, 1.0),
                emissive: LinearRgba::rgb(0.8, 0.9, 1.0), // Bright LED glow
                ..default()
            })),
            Transform::from_xyz(x_pos, 0.2, 2.1),
            ChildOf(chiron_entity),
        ));
    }

    // Chiron rear lights (LED strip style) 
    for &x_pos in [-0.8, 0.8].iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.4, 0.08, 0.06))), // Wide LED strip
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.1, 0.1),
                emissive: LinearRgba::rgb(1.0, 0.2, 0.2), // Glowing red
                ..default()
            })),
            Transform::from_xyz(x_pos, 0.1, -2.1),
            ChildOf(chiron_entity),
        ));
    }

    // Chiron side mirrors (carbon fiber style)
    for &x_pos in [-1.1, 1.1].iter() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.08, 0.15))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                metallic: 0.1,
                ..default()
            })),
            Transform::from_xyz(x_pos, 0.4, 0.5),
            ChildOf(chiron_entity),
        ));
    }

    // Chiron rear spoiler
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.6, 0.05, 0.3))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.1),
            metallic: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, -1.8),
        ChildOf(chiron_entity),
    ));

    // HELICOPTER - Spawn a luxury Dubai police helicopter
    let helicopter_entity = commands.spawn((
        Helicopter,
        RigidBody::Dynamic,
        Collider::cuboid(1.5, 1.0, 3.0),
        Velocity::zero(),
        Transform::from_xyz(120.0, 15.0, 80.0).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        Visibility::Visible,
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
    ));

    // Cockpit (front glass section)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.2, 1.2, 1.8))),
        MeshMaterial3d(materials.add(Color::srgba(0.1, 0.1, 0.2, 0.3))), // Dark tinted glass
        Transform::from_xyz(0.0, 0.2, 1.5),
        ChildOf(helicopter_entity),
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
        ));
    }

    // Rotor hub (center)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))), // Dark hub
        Transform::from_xyz(0.0, 2.0, 0.0),
        ChildOf(helicopter_entity),
    ));

    // Tail boom (long back section) - sleeker design
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.6, 0.6, 4.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.85, 0.85, 0.85))), // Light gray
        Transform::from_xyz(0.0, 0.0, -4.5),
        ChildOf(helicopter_entity),
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
        ));
    }

    // Tail rotor hub
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.15, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))), // Dark hub
        Transform::from_xyz(-1.0, 0.5, -6.5).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ChildOf(helicopter_entity),
    ));

    // Landing skids (2 runners underneath) - more realistic
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.08, 3.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.6, 0.6, 0.6))), // Light gray skids
        Transform::from_xyz(-0.8, -1.0, 0.0),
        ChildOf(helicopter_entity),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.08, 3.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.6, 0.6, 0.6))), // Light gray skids
        Transform::from_xyz(0.8, -1.0, 0.0),
        ChildOf(helicopter_entity),
    ));

    // Add Dubai police styling stripes
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 0.3, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.5, 0.8))), // Blue stripe
        Transform::from_xyz(0.0, 0.0, 1.0),
        ChildOf(helicopter_entity),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.6, 0.3, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.0, 0.0))), // Red stripe
        Transform::from_xyz(0.0, -0.4, 1.0),
        ChildOf(helicopter_entity),
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

    // F16 FIGHTER JET - Spawn an advanced military aircraft
    let f16_entity = commands.spawn((
        F16,
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 1.5, 1.5),
        LockedAxes::empty(), // Full 6DOF movement for realistic flight
        Velocity::zero(),
        Transform::from_xyz(80.0, 2.0, 120.0), // Spawn at airfield location, separated from helicopter
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
    ));

    // F16 Vertical tail
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 4.0, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.4, 0.5))),
        Transform::from_xyz(-6.0, 1.5, 0.0),
        ChildOf(f16_entity),
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
    ));

    // F16 Landing gear (retractable)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.3, 1.5, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_xyz(2.0, -1.2, 1.5),
        ChildOf(f16_entity),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.3, 1.5, 0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_xyz(2.0, -1.2, -1.5),
        ChildOf(f16_entity),
    ));

    // Front landing gear
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 1.2, 0.25))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::from_xyz(6.0, -1.0, 0.0),
        ChildOf(f16_entity),
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

    // NPCs scattered throughout the massive world
    use rand::prelude::*;
    let mut rng = rand::thread_rng();
    
    for _ in 0..50 { // 50 NPCs in the huge world
        let x = rng.gen_range(-900.0..900.0);
        let z = rng.gen_range(-900.0..900.0);
        let y = 1.0;
        
        // Random NPC colors
        let npc_colors = [
            Color::srgb(0.8, 0.6, 0.4), // Skin tone 1
            Color::srgb(0.6, 0.4, 0.3), // Skin tone 2
            Color::srgb(0.9, 0.7, 0.5), // Skin tone 3
            Color::srgb(0.7, 0.5, 0.4), // Skin tone 4
        ];
        
        let color = npc_colors[rng.gen_range(0..npc_colors.len())];
        
        // Random target position for movement
        let target_x = rng.gen_range(-900.0..900.0);
        let target_z = rng.gen_range(-900.0..900.0);
        
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x, y, z),
            RigidBody::Dynamic,
            Collider::capsule(Vec3::new(0.0, -0.9, 0.0), Vec3::new(0.0, 0.9, 0.0), 0.3),
            Velocity::zero(),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            NPC {
                target_position: Vec3::new(target_x, y, target_z),
                speed: rng.gen_range(2.0..5.0),
                last_update: 0.0,
                update_interval: rng.gen_range(0.05..0.2), // Random update intervals for staggered updates
            },
            Cullable { max_distance: 200.0, is_culled: false },
        ));
    }

    // BUILDINGS - Add Dubai-style skyscrapers and structures
    let building_positions = [
        // Downtown district (tall buildings)
        (-300.0, 150.0), (-250.0, 120.0), (-200.0, 180.0), (-150.0, 100.0),
        (200.0, 140.0), (250.0, 160.0), (300.0, 120.0), (350.0, 200.0),
        
        // Business district
        (-600.0, 80.0), (-550.0, 90.0), (-500.0, 70.0),
        (500.0, 85.0), (550.0, 95.0), (600.0, 75.0),
        
        // Residential areas (shorter buildings)
        (-150.0, 30.0), (-100.0, 25.0), (-50.0, 35.0),
        (100.0, 28.0), (150.0, 32.0), (200.0, 26.0),
        
        // Scattered buildings
        (-800.0, 60.0), (-400.0, 45.0), (400.0, 55.0), (800.0, 65.0),
    ];

    for (i, &(x, z)) in building_positions.iter().enumerate() {
        let height = match i {
            0..=7 => rng.gen_range(100.0..200.0), // Downtown towers
            8..=13 => rng.gen_range(60.0..100.0), // Business buildings  
            14..=19 => rng.gen_range(20.0..40.0), // Residential
            _ => rng.gen_range(40.0..80.0), // Mixed
        };
        
        let width = rng.gen_range(15.0..30.0);
        let depth = rng.gen_range(15.0..30.0);
        
        // Building colors
        let building_colors = [
            Color::srgb(0.8, 0.8, 0.9), // Light gray/white
            Color::srgb(0.7, 0.7, 0.8), // Medium gray
            Color::srgb(0.9, 0.85, 0.7), // Beige/sand
            Color::srgb(0.6, 0.7, 0.8), // Blue tint
            Color::srgb(0.8, 0.75, 0.65), // Warm beige
        ];
        
        let color = building_colors[rng.gen_range(0..building_colors.len())];
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x, 2.5, z), // Center collider at fixed height like palm trees
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0 + 2.0, 5.0, depth / 2.0 + 2.0), // Made significantly larger with tall height like palm trees 
            Cullable { max_distance: 800.0, is_culled: false },
            CollisionGroups::new(STATIC_GROUP, Group::ALL),
        ));
    }
}

fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &Transform), (With<Player>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = player_query.single_mut() else {
        return;
    };

    let speed = 8.0;
    let rotation_speed = 3.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement (relative to player's facing direction)
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * speed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * speed;
    }
    
    // Rotation - DIRECT velocity control to eliminate oscillations
    if input.pressed(KeyCode::ArrowLeft) {
        target_angular_velocity.y = rotation_speed;
    } else if input.pressed(KeyCode::ArrowRight) {
        target_angular_velocity.y = -rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation when no input
    }
    
    // Set velocity directly - no impulse calculations
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}

fn car_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut car_query: Query<(&mut Velocity, &Transform), (With<Car>, With<ActiveEntity>, Without<SuperCar>)>,
) {
    let Ok((mut velocity, transform)) = car_query.single_mut() else {
        return;
    };

    let speed = 25.0;
    let rotation_speed = 2.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * speed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * speed;
    }
    
    // Rotation (only when moving) - DIRECT velocity control
    if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::ArrowDown) {
        if input.pressed(KeyCode::ArrowLeft) {
            target_angular_velocity.y = rotation_speed;
        } else if input.pressed(KeyCode::ArrowRight) {
            target_angular_velocity.y = -rotation_speed;
        } else {
            target_angular_velocity.y = 0.0; // Force zero rotation
        }
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation when not moving
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}

fn supercar_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut supercar_query: Query<(&mut Velocity, &Transform, &mut SuperCar), (With<Car>, With<ActiveEntity>, With<SuperCar>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((mut velocity, transform, mut supercar)) = supercar_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    supercar.exhaust_timer += dt;
    
    // Enhanced performance parameters
    let base_speed = supercar.max_speed;
    let rotation_speed = 3.5; // Tighter turning than regular cars
    
    // Turbo boost activation
    let turbo_active = input.pressed(KeyCode::Space);
    supercar.turbo_boost = turbo_active;
    
    let speed_multiplier = if turbo_active { 1.8 } else { 1.0 };
    let target_speed = base_speed * speed_multiplier;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * target_speed;
        
        // Spawn exhaust flames when accelerating
        if supercar.exhaust_timer > 0.1 {
            supercar.exhaust_timer = 0.0;
            
            // Spawn exhaust particles behind the car
            let exhaust_pos = transform.translation + transform.back() * 2.5 + Vec3::new(0.0, 0.2, 0.0);
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.15))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if turbo_active { Color::srgb(0.2, 0.4, 1.0) } else { Color::srgb(1.0, 0.3, 0.0) },
                    emissive: if turbo_active { 
                        LinearRgba::rgb(0.3, 0.6, 1.5)  // Blue turbo flames
                    } else { 
                        LinearRgba::rgb(1.0, 0.4, 0.0)  // Orange flames
                    },
                    ..default()
                })),
                Transform::from_translation(exhaust_pos),
                ExhaustFlame,
            ));
        }
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * (target_speed * 0.7);
    }
    
    // Enhanced rotation (can turn even without moving for supercars) - DIRECT velocity control
    if input.pressed(KeyCode::ArrowLeft) {
        target_angular_velocity.y = rotation_speed;
    } else if input.pressed(KeyCode::ArrowRight) {
        target_angular_velocity.y = -rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}

fn helicopter_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut helicopter_query: Query<(&mut Velocity, &Transform), (With<Helicopter>, With<ActiveEntity>)>,
) {
    let Ok((mut velocity, transform)) = helicopter_query.single_mut() else {
        return;
    };

    let speed = 15.0;
    let rotation_speed = 2.5;
    let vertical_speed = 8.0;
    
    let mut target_linear_velocity = Vec3::ZERO;
    let mut target_angular_velocity = Vec3::ZERO;
    
    // Forward/backward movement
    if input.pressed(KeyCode::ArrowUp) {
        let forward = transform.forward();
        target_linear_velocity += forward * speed;
    }
    if input.pressed(KeyCode::ArrowDown) {
        let forward = transform.forward();
        target_linear_velocity -= forward * speed;
    }
    
    // Rotation - DIRECT velocity control
    if input.pressed(KeyCode::ArrowLeft) {
        target_angular_velocity.y = rotation_speed;
    } else if input.pressed(KeyCode::ArrowRight) {
        target_angular_velocity.y = -rotation_speed;
    } else {
        target_angular_velocity.y = 0.0; // Force zero rotation
    }
    
    // HELICOPTER SPECIFIC: Vertical movement with Shift (up) and Ctrl (down)
    if input.pressed(KeyCode::ShiftLeft) {
        target_linear_velocity.y += vertical_speed;
    }
    if input.pressed(KeyCode::ControlLeft) {
        target_linear_velocity.y -= vertical_speed;
    }
    
    // Set velocity directly
    velocity.linvel = target_linear_velocity;
    velocity.angvel = target_angular_velocity;
}

fn rotate_helicopter_rotors(
    time: Res<Time>,
    mut main_rotor_query: Query<&mut Transform, (With<MainRotor>, Without<TailRotor>)>,
    mut tail_rotor_query: Query<&mut Transform, (With<TailRotor>, Without<MainRotor>)>,
) {
    let main_rotor_speed = 20.0; // Fast rotation for main rotor
    let tail_rotor_speed = 35.0; // Even faster for tail rotor

    // Rotate main rotors (around Y axis)
    for mut transform in main_rotor_query.iter_mut() {
        let rotation = Quat::from_rotation_y(time.elapsed_secs() * main_rotor_speed);
        transform.rotation = rotation;
    }

    // Rotate tail rotors (around Z axis)  
    for mut transform in tail_rotor_query.iter_mut() {
        let rotation = Quat::from_rotation_z(time.elapsed_secs() * tail_rotor_speed);
        transform.rotation = rotation;
    }
}

fn optimized_npc_movement(
    time: Res<Time>,
    mut npc_query: Query<(&mut Transform, &mut Velocity, &mut NPC, &Cullable)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<NPC>)>,
) {
    use rand::prelude::*;
    let mut rng = rand::thread_rng();
    let current_time = time.elapsed_secs();
    
    // Get player position for distance-based optimization
    let player_pos = if let Ok(active_transform) = active_query.single() {
        active_transform.translation
    } else {
        Vec3::ZERO
    };
    
    for (mut transform, mut velocity, mut npc, cullable) in npc_query.iter_mut() {
        // Skip if culled
        if cullable.is_culled {
            velocity.linvel = Vec3::ZERO;
            continue;
        }
        
        // Only update NPCs at their specific intervals (staggered updates)
        if current_time - npc.last_update < npc.update_interval {
            continue;
        }
        npc.last_update = current_time;
        
        let current_pos = transform.translation;
        let target_pos = npc.target_position;
        
        // Calculate distance to target
        let distance = current_pos.distance(target_pos);
        
        // Reduce update frequency for distant NPCs
        let distance_to_player = current_pos.distance(player_pos);
        if distance_to_player > 100.0 {
            npc.update_interval = 0.5; // Very slow updates for distant NPCs
        } else if distance_to_player > 50.0 {
            npc.update_interval = 0.2; // Slower updates for far NPCs
        } else {
            npc.update_interval = 0.05; // Normal updates for close NPCs
        }
        
        // If close to target, pick a new random target
        if distance < 5.0 {
            npc.target_position = Vec3::new(
                rng.gen_range(-900.0..900.0),
                1.0,
                rng.gen_range(-900.0..900.0),
            );
        } else {
            // Move towards target
            let direction = (target_pos - current_pos).normalize();
            velocity.linvel = Vec3::new(
                direction.x * npc.speed,
                velocity.linvel.y, // Preserve gravity
                direction.z * npc.speed,
            );
            
            // Face movement direction
            if direction.length() > 0.1 {
                let rotation = Quat::from_rotation_y((-direction.x).atan2(-direction.z));
                transform.rotation = rotation;
            }
        }
    }
}

fn distance_culling_system(
    mut cullable_query: Query<(&mut Cullable, &mut Visibility, &Transform), Without<ActiveEntity>>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<Cullable>)>,
    _settings: Res<CullingSettings>,
) {
    let Ok(active_transform) = active_query.single() else { return; };
    let player_pos = active_transform.translation;
    
    for (mut cullable, mut visibility, transform) in cullable_query.iter_mut() {
        let distance = player_pos.distance(transform.translation);
        
        if distance > cullable.max_distance {
            if !cullable.is_culled {
                cullable.is_culled = true;
                *visibility = Visibility::Hidden;
            }
        } else {
            if cullable.is_culled {
                cullable.is_culled = false;
                *visibility = Visibility::Visible;
            }
        }
    }
}

fn dynamic_terrain_system(
    mut terrain_query: Query<&mut Transform, (With<DynamicTerrain>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicTerrain>)>,
) {
    if let Ok(active_transform) = active_query.single() {
        if let Ok(mut terrain_transform) = terrain_query.single_mut() {
            // Keep terrain centered on active entity (player/car/helicopter) but slightly below
            terrain_transform.translation.x = active_transform.translation.x;
            terrain_transform.translation.z = active_transform.translation.z;
            terrain_transform.translation.y = -0.5; // Always below ground level
        }
    }
}

fn dynamic_content_system(
    mut commands: Commands,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicContent>)>,
    content_query: Query<(Entity, &Transform, &DynamicContent)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        
        // CIRCULAR RADIUS SYSTEM PARAMETERS
        let active_radius = 800.0;   // Content stays active within this radius
        let cleanup_radius = 1000.0; // Content gets cleaned up beyond this radius
        let spawn_density = 50.0;    // Distance between spawn points
        
        // Phase 1: Remove content outside cleanup radius (truly circular)
        for (entity, content_transform, _) in content_query.iter() {
            let distance = active_pos.distance(content_transform.translation);
            if distance > cleanup_radius {
                commands.entity(entity).despawn();
            }
        }
        
        // Phase 2: Collect existing content for collision avoidance
        let existing_content: Vec<(Vec3, ContentType, f32)> = content_query.iter()
            .map(|(_, transform, dynamic_content)| {
                let radius = match dynamic_content.content_type {
                    ContentType::Building => 20.0,
                    ContentType::Road => 15.0,
                    ContentType::Tree => 8.0,
                    ContentType::Vehicle => 10.0,
                    ContentType::NPC => 3.0,
                };
                (transform.translation, dynamic_content.content_type.clone(), radius)
            })
            .collect();
        
        // Phase 3: TRUE CIRCULAR SPAWNING using polar coordinates
        // Generate content in concentric circles around the active entity
        let mut spawn_attempts = 0;
        let max_spawn_attempts = 200; // Prevent infinite loops
        
        for radius_step in (spawn_density as i32..active_radius as i32).step_by(spawn_density as usize) {
            let radius = radius_step as f32;
            let circumference = 2.0 * std::f32::consts::PI * radius;
            let points_on_circle = (circumference / spawn_density).max(8.0) as i32;
            
            for i in 0..points_on_circle {
                spawn_attempts += 1;
                if spawn_attempts > max_spawn_attempts { break; }
                
                let angle = (i as f32 / points_on_circle as f32) * 2.0 * std::f32::consts::PI;
                let spawn_x = active_pos.x + radius * angle.cos();
                let spawn_z = active_pos.z + radius * angle.sin();
                let spawn_pos = Vec3::new(spawn_x, 0.0, spawn_z);
                
                // Only spawn if no content exists nearby
                if !has_content_at_position(spawn_pos, &existing_content, spawn_density * 0.8) {
                    spawn_dynamic_content_safe(&mut commands, spawn_pos, &existing_content, &mut meshes, &mut materials);
                }
            }
            if spawn_attempts > max_spawn_attempts { break; }
        }
    }
}

fn has_content_at_position(position: Vec3, existing_content: &[(Vec3, ContentType, f32)], min_distance: f32) -> bool {
    existing_content.iter().any(|(existing_pos, _, radius)| {
        position.distance(*existing_pos) < min_distance.max(*radius)
    })
}

fn spawn_dynamic_content_safe(
    commands: &mut Commands,
    position: Vec3,
    existing_content: &[(Vec3, ContentType, f32)],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Roads at regular intervals (priority spawning) - denser road network
    let on_highway = (position.x % 100.0).abs() < 8.0 || (position.z % 100.0).abs() < 8.0;
    let on_secondary = (position.x % 50.0).abs() < 4.0 || (position.z % 50.0).abs() < 4.0;
    
    if on_highway || on_secondary {
        // Check if road conflicts with existing buildings
        if !has_content_at_position(position, existing_content, 20.0) {
            spawn_road(commands, position, meshes, materials);
        }
    }
    // Buildings away from roads (check for road conflicts)
    else if !is_on_road(position) && rng.gen_range(0.0..1.0) < 0.4 {
        // Ensure no overlap with roads or other buildings
        if !has_content_at_position(position, existing_content, 25.0) {
            spawn_building(commands, position, meshes, materials);
        }
    }
    // Vehicles on roads only
    else if is_on_road(position) && rng.gen_range(0.0..1.0) < 0.2 {
        // Ensure no overlap with other vehicles or buildings
        if !has_content_at_position(position, existing_content, 40.0) {
            spawn_vehicle(commands, position, meshes, materials);
        }
    }
    // Trees in empty areas (away from roads and buildings)
    else if !is_on_road(position) && rng.gen_range(0.0..1.0) < 0.3 {
        // Ensure no overlap with buildings or roads
        if !has_content_at_position(position, existing_content, 10.0) {
            spawn_dynamic_tree(commands, position, meshes, materials);
        }
    }
    // NPCs on roads and near buildings
    else if rng.gen_range(0.0..1.0) < 0.1 {
        // Ensure no overlap with other content
        if !has_content_at_position(position, existing_content, 5.0) {
            spawn_dynamic_npc(commands, position, meshes, materials);
        }
    }
}

fn is_on_road(position: Vec3) -> bool {
    // Main highways every 100 units - matches spawn_dynamic_content_safe
    let on_highway = (position.x % 100.0).abs() < 8.0 || (position.z % 100.0).abs() < 8.0;
    
    // Secondary roads every 50 units - matches spawn_dynamic_content_safe
    let on_secondary = (position.x % 50.0).abs() < 4.0 || (position.z % 50.0).abs() < 4.0;
    
    on_highway || on_secondary
}

fn spawn_road(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let road_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.3),
        ..default()
    });
    
    let road_size = if (position.x % 100.0).abs() < 8.0 {
        (100.0, 0.1, 16.0) // Horizontal highway
    } else if (position.x % 50.0).abs() < 4.0 {
        (50.0, 0.1, 8.0) // Horizontal secondary road
    } else if (position.z % 100.0).abs() < 8.0 {
        (16.0, 0.1, 100.0) // Vertical highway
    } else {
        (8.0, 0.1, 50.0) // Vertical secondary road
    };
    
    commands.spawn((
        DynamicContent {
            content_type: ContentType::Road,
        },
        Mesh3d(meshes.add(Cuboid::new(road_size.0, road_size.1, road_size.2))),
        MeshMaterial3d(road_material),
        Transform::from_translation(Vec3::new(position.x, -0.49, position.z)),
    ));
}

fn spawn_building(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let height = rng.gen_range(8.0..30.0);
    let width = rng.gen_range(8.0..15.0); // Slightly smaller to prevent overlaps
    
    let building_material = materials.add(StandardMaterial {
        base_color: Color::srgb(
            rng.gen_range(0.5..0.9),
            rng.gen_range(0.5..0.9),
            rng.gen_range(0.5..0.9),
        ),
        ..default()
    });
    
    commands.spawn((
        DynamicContent {
            content_type: ContentType::Building,
        },
        Mesh3d(meshes.add(Cuboid::new(width, height, width))),
        MeshMaterial3d(building_material),
        Transform::from_translation(Vec3::new(position.x, 2.5, position.z)), // Center collider at fixed height like palm trees
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0 + 2.0, 5.0, width / 2.0 + 2.0), // Made larger to prevent tunneling with tall height like palm trees
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
    ));
}

fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Vehicle types with different colors and sizes
    let vehicle_types = [
        ("Bugatti Chiron", Color::srgb(0.1, 0.1, 0.8), (4.5, 1.2, 2.0)), // Blue supercar
        ("Ferrari 488", Color::srgb(0.8, 0.1, 0.1), (4.3, 1.1, 1.9)),    // Red supercar
        ("Lamborghini", Color::srgb(0.9, 0.7, 0.1), (4.4, 1.0, 2.1)),    // Yellow supercar
        ("McLaren 720S", Color::srgb(0.9, 0.4, 0.1), (4.2, 1.1, 1.8)),   // Orange supercar
        ("Porsche 911", Color::srgb(0.2, 0.2, 0.2), (4.0, 1.3, 1.7)),    // Black sports car
        ("BMW M3", Color::srgb(0.7, 0.7, 0.7), (4.6, 1.4, 1.8)),         // Silver sedan
    ];
    
    let (car_name, base_color, (length, height, width)) = vehicle_types[rng.gen_range(0..vehicle_types.len())];
    
    let vehicle_material = materials.add(StandardMaterial {
        base_color,
        metallic: 0.9,
        perceptual_roughness: 0.1,
        ..default()
    });
    
    let car_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Vehicle,
        },
        Car,
        RigidBody::Dynamic,
        Collider::cuboid(length / 2.0, height / 2.0, width / 2.0),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_translation(Vec3::new(position.x, height / 2.0, position.z)),
        Friction::coefficient(0.3),
        Restitution::coefficient(0.1),
        Ccd::enabled(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        ExternalImpulse::default(),
        Damping { linear_damping: 1.5, angular_damping: 6.0 }
    )).id();
    
    // Main car body
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(length, height, width))),
        MeshMaterial3d(vehicle_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(car_entity),
    ));
    
    // Add wheels
    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        metallic: 0.8,
        ..default()
    });
    
    let wheel_positions = [
        (length / 3.0, -height / 2.5, width / 2.2),   // Front right
        (length / 3.0, -height / 2.5, -width / 2.2),  // Front left
        (-length / 3.0, -height / 2.5, width / 2.2),  // Rear right
        (-length / 3.0, -height / 2.5, -width / 2.2), // Rear left
    ];
    
    for (x, y, z) in wheel_positions {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.3, 0.2))),
            MeshMaterial3d(wheel_material.clone()),
            Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
            ChildOf(car_entity),
        ));
    }
    
    // Add beacon for this vehicle
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.15, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 1.0), // Cyan beacon
            emissive: Color::srgb(0.0, 1.0, 1.0).into(),
            ..default()
        })),
        Transform::from_xyz(position.x, 6.0, position.z),
        VehicleBeacon,
    ));
}

fn spawn_dynamic_tree(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let trunk_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.2, 0.1),
        ..default()
    });
    
    let leaves_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.6, 0.1),
        ..default()
    });
    
    let tree_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Tree,
        },
        Transform::from_translation(position),
        RigidBody::Fixed,
        Collider::cylinder(4.0, 0.5),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
    )).id();
    
    // Trunk
    commands.entity(tree_entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.5, 8.0))),
            MeshMaterial3d(trunk_material),
            Transform::from_translation(Vec3::Y * 4.0),
        ));
        
        // Leaves
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(4.0))),
            MeshMaterial3d(leaves_material),
            Transform::from_translation(Vec3::Y * 10.0),
        ));
    });
}

fn spawn_dynamic_npc(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Random NPC colors
    let npc_colors = [
        Color::srgb(0.8, 0.6, 0.4), // Skin tone 1
        Color::srgb(0.6, 0.4, 0.3), // Skin tone 2
        Color::srgb(0.9, 0.7, 0.5), // Skin tone 3
        Color::srgb(0.7, 0.5, 0.4), // Skin tone 4
    ];
    
    let color = npc_colors[rng.gen_range(0..npc_colors.len())];
    
    // Random target position for movement
    let target_x = position.x + rng.gen_range(-50.0..50.0);
    let target_z = position.z + rng.gen_range(-50.0..50.0);
    
    commands.spawn((
        DynamicContent {
            content_type: ContentType::NPC,
        },
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
        MeshMaterial3d(materials.add(color)),
        Transform::from_translation(Vec3::new(position.x, 1.0, position.z)),
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(0.9, 0.3),
        NPC {
            target_position: Vec3::new(target_x, 1.0, target_z),
            speed: rng.gen_range(1.0..3.0),
            last_update: 0.0,
            update_interval: rng.gen_range(0.1..0.3),
        },
        Cullable { max_distance: 150.0, is_culled: false },
        CollisionGroups::new(CHARACTER_GROUP, STATIC_GROUP),
    ));
}

fn performance_monitoring_system(
    time: Res<Time>,
    mut stats: ResMut<PerformanceStats>,
    entity_query: Query<Entity>,
    cullable_query: Query<&Cullable>,
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
) {
    let current_time = time.elapsed_secs();
    
    // Update stats
    stats.entity_count = entity_query.iter().count();
    stats.culled_entities = cullable_query.iter().filter(|c| c.is_culled).count();
    
    // Get frame time from diagnostics
    if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_avg) = fps_diag.smoothed() {
            stats.frame_time = (1000.0 / fps_avg) as f32; // Convert to milliseconds
        }
    }
    
    // Report every 5 seconds
    if current_time - stats.last_report > 5.0 {
        stats.last_report = current_time;
        info!(
            "PERFORMANCE: Entities: {} | Culled: {} | Frame: {:.1}ms | FPS: {:.0}",
            stats.entity_count,
            stats.culled_entities,
            stats.frame_time,
            if stats.frame_time > 0.0 { 1000.0 / stats.frame_time } else { 0.0 }
        );
    }
}

// World bounds system removed to allow infinite world exploration
// Dynamic content system now handles world generation as you explore

fn interaction_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut player_query: Query<(Entity, &mut Transform, &mut Velocity), (With<Player>, Without<Car>, Without<Helicopter>, Without<F16>)>,
    car_query: Query<(Entity, &Transform), (With<Car>, Without<Player>)>,
    helicopter_query: Query<(Entity, &Transform), (With<Helicopter>, Without<Player>)>,
    f16_query: Query<(Entity, &Transform), (With<F16>, Without<Player>)>,
    active_query: Query<Entity, With<ActiveEntity>>,
) {
    if !input.just_pressed(KeyCode::KeyF) {
        return;
    }

    match **current_state {
        GameState::Walking => {
            // Try to enter vehicle (car or helicopter)
            let Ok((player_entity, player_transform, _)) = player_query.single_mut() else { return; };
            
            // Check for cars first
            for (car_entity, car_transform) in car_query.iter() {
                let distance = player_transform.translation.distance(car_transform.translation);
                if distance < 3.0 {
                    // Remove ActiveEntity from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the car
                    commands.entity(player_entity).insert(ChildOf(car_entity));
                    
                    // Add ActiveEntity to car
                    commands.entity(car_entity).insert(ActiveEntity);
                    
                    // Store which car the player is in
                    commands.entity(player_entity).insert(InCar(car_entity));
                    
                    // Switch to driving state
                    state.set(GameState::Driving);
                    info!("Entered car!");
                    return;
                }
            }
            
            // Check for helicopters
            for (helicopter_entity, helicopter_transform) in helicopter_query.iter() {
                let distance = player_transform.translation.distance(helicopter_transform.translation);
                if distance < 5.0 { // Larger range for helicopters
                    // Remove ActiveEntity from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the helicopter
                    commands.entity(player_entity).insert(ChildOf(helicopter_entity));
                    
                    // Add ActiveEntity to helicopter
                    commands.entity(helicopter_entity).insert(ActiveEntity);
                    
                    // Store which helicopter the player is in
                    commands.entity(player_entity).insert(InCar(helicopter_entity)); // Reuse InCar for vehicles
                    
                    // Switch to flying state
                    state.set(GameState::Flying);
                    info!("Entered helicopter!");
                    return;
                }
            }
            
            // Check for F16s
            for (f16_entity, f16_transform) in f16_query.iter() {
                let distance = player_transform.translation.distance(f16_transform.translation);
                if distance < 8.0 { // Larger range for F16s
                    // Remove ActiveEntity from player and hide them
                    commands.entity(player_entity)
                        .remove::<ActiveEntity>()
                        .insert(Visibility::Hidden);
                    
                    // Make player a child of the F16
                    commands.entity(player_entity).insert(ChildOf(f16_entity));
                    
                    // Add ActiveEntity to F16
                    commands.entity(f16_entity).insert(ActiveEntity);
                    
                    // Store which F16 the player is in
                    commands.entity(player_entity).insert(InCar(f16_entity)); // Reuse InCar for vehicles
                    
                    // Switch to jetting state
                    state.set(GameState::Jetting);
                    info!("Entered F16 Fighter Jet!");
                    return;
                }
            }
        }
        GameState::Driving => {
            // Exit car
            if let Ok(active_car) = active_query.single() {
                // Get the specific active car's transform
                if let Ok((_, car_transform)) = car_query.get(active_car) {
                    // Remove ActiveEntity from car
                    commands.entity(active_car).remove::<ActiveEntity>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the car
                        let exit_position = car_transform.translation + car_transform.right() * 3.0;
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(player_entity)
                            .remove::<ChildOf>()
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(car_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
                        info!("Exited car at position: {:?}", exit_position);
                    }
                    
                    // Switch to walking state
                    state.set(GameState::Walking);
                    info!("Exited car!");
                }
            }
        }
        GameState::Flying => {
            // Exit helicopter
            if let Ok(active_helicopter) = active_query.single() {
                // Get the specific active helicopter's transform
                if let Ok((_, helicopter_transform)) = helicopter_query.get(active_helicopter) {
                    // Remove ActiveEntity from helicopter
                    commands.entity(active_helicopter).remove::<ActiveEntity>();
                    
                    // Find player and properly detach and position them
                    if let Ok((player_entity, _, _)) = player_query.single_mut() {
                        // Calculate exit position next to the helicopter (a bit further away)
                        let exit_position = helicopter_transform.translation + helicopter_transform.right() * 4.0 + Vec3::new(0.0, -1.0, 0.0); // Drop to ground level
                        
                        // Remove the child relationship and position the player in world space
                        commands.entity(player_entity)
                            .remove::<ChildOf>()
                            .remove::<InCar>()
                            .insert(Transform::from_translation(exit_position).with_rotation(helicopter_transform.rotation))
                            .insert(Velocity::zero())
                            .insert(Visibility::Visible)
                            .insert(ActiveEntity);
                        
                        info!("Exited helicopter at position: {:?}", exit_position);
                    }
                    
                    // Switch to walking state
                    state.set(GameState::Walking);
                    info!("Exited helicopter!");
                }
            }
        }
        GameState::Jetting => {
            // Exit F16 (handled in f16_movement system when F key pressed)
            // This state will be changed by the F16 movement system
        }
    }
}

fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<MainCamera>)>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else { return; };
    let Ok(active_transform) = active_query.single() else { return; };
    
    // Safety checks for invalid transforms
    if !active_transform.translation.is_finite() || !active_transform.rotation.is_finite() {
        return;
    }
    
    // Camera follows behind the entity, facing the same direction
    let entity_forward = active_transform.forward();
    
    // Additional safety check for invalid forward vector
    if !entity_forward.is_finite() {
        return;
    }
    
    let entity_up = Vec3::Y;
    
    // Position camera behind and above the entity
    let camera_distance = 20.0;
    let camera_height = 12.0;
    let camera_offset = -entity_forward * camera_distance + entity_up * camera_height;
    let target_pos = active_transform.translation + camera_offset;
    
    // Safety check for target position
    if !target_pos.is_finite() {
        return;
    }
    
    // Smooth camera movement
    camera_transform.translation = camera_transform.translation.lerp(target_pos, 0.05);
    
    // Camera looks forward in the same direction as the entity
    let look_target = active_transform.translation + entity_forward * 10.0 + Vec3::Y * 2.0;
    
    // Safety check for look target
    if !look_target.is_finite() {
        return;
    }
    
    camera_transform.look_at(look_target, Vec3::Y);
}

fn debug_player_position(
    player_query: Query<&Transform, (With<Player>, With<ActiveEntity>)>,
    car_query: Query<&Transform, (With<Car>, With<ActiveEntity>)>,
    helicopter_query: Query<&Transform, (With<Helicopter>, With<ActiveEntity>)>,
    f16_query: Query<&Transform, (With<F16>, With<ActiveEntity>)>,
    state: Res<State<GameState>>,
) {
    match **state {
        GameState::Walking => {
            if let Ok(player_transform) = player_query.single() {
                // Only log occasionally to avoid spam
                if player_transform.translation.x.abs() > 100.0 || player_transform.translation.z.abs() > 100.0 {
                    info!("DEBUG: Player walking at position: {:?}", player_transform.translation);
                }
            }
        }
        GameState::Driving => {
            if let Ok(car_transform) = car_query.single() {
                // Only log occasionally 
                if car_transform.translation.x.abs() > 100.0 || car_transform.translation.z.abs() > 100.0 {
                    info!("DEBUG: Car driving at position: {:?}", car_transform.translation);
                }
            }
        }
        GameState::Flying => {
            if let Ok(helicopter_transform) = helicopter_query.single() {
                // Log helicopter altitude and position
                if helicopter_transform.translation.y > 5.0 || helicopter_transform.translation.x.abs() > 100.0 || helicopter_transform.translation.z.abs() > 100.0 {
                    info!("DEBUG: Helicopter flying at position: {:?} (altitude: {:.1}m)", helicopter_transform.translation, helicopter_transform.translation.y);
                }
            }
        }
        GameState::Jetting => {
            if let Ok(f16_transform) = f16_query.single() {
                // Log F16 altitude and position
                if f16_transform.translation.y > 10.0 || f16_transform.translation.x.abs() > 100.0 || f16_transform.translation.z.abs() > 100.0 {
                    info!("DEBUG: F16 flying at position: {:?} (altitude: {:.1}m)", f16_transform.translation, f16_transform.translation.y);
                }
            }
        }
    }
}

// Exhaust effects system - cleans up old exhaust flames
fn exhaust_effects_system(
    mut commands: Commands,
    time: Res<Time>,
    mut exhaust_query: Query<(Entity, &mut Transform), With<ExhaustFlame>>,
) {
    let dt = time.delta_secs();
    
    for (entity, mut transform) in exhaust_query.iter_mut() {
        // Move exhaust particles backward and up slightly
        transform.translation += Vec3::new(0.0, 1.0, 0.0) * dt * 2.0;
        transform.scale *= 0.98; // Shrink over time
        
        // Remove exhaust flames after they've moved up or become too small
        if transform.translation.y > 3.0 || transform.scale.x < 0.1 {
            commands.entity(entity).despawn();
        }
    }
}

// Update waypoint system - shows distance to vehicles
fn update_waypoint_system(
    player_query: Query<&Transform, (With<Player>, Without<VehicleBeacon>)>,
    beacon_query: Query<&Transform, (With<VehicleBeacon>, Without<Player>)>,
    mut waypoint_text_query: Query<&mut Text, With<WaypointText>>,
) {
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation;
        
        for mut text in waypoint_text_query.iter_mut() {
            let mut waypoint_info = String::new();
            
            for (i, beacon_transform) in beacon_query.iter().enumerate() {
                let distance = player_pos.distance(beacon_transform.translation);
                let direction = (beacon_transform.translation - player_pos).normalize();
                
                let vehicle_name = match i {
                    0 => "BUGATTI CHIRON",
                    1 => "HELICOPTER", 
                    2 => "F16 FIGHTER JET",
                    _ => "VEHICLE",
                };
                
                waypoint_info.push_str(&format!(
                    "{}: {:.0}m ({:.0}, {:.0})\n", 
                    vehicle_name, 
                    distance,
                    direction.x * 100.0,
                    direction.z * 100.0
                ));
            }
            
            text.0 = waypoint_info;
        }
    }
}

// Update beacon visibility system
fn update_beacon_visibility(
    mut beacon_query: Query<&mut Visibility, With<VehicleBeacon>>,
    time: Res<Time>,
) {
    let flash_cycle = (time.elapsed_secs() * 2.0).sin() > 0.0;
    
    for mut visibility in beacon_query.iter_mut() {
        *visibility = if flash_cycle { 
            Visibility::Visible 
        } else { 
            Visibility::Hidden 
        };
    }
}

// Controls UI system - updates the control instructions based on current state
fn controls_ui_system(
    current_state: Res<State<GameState>>,
    mut controls_query: Query<&mut Text, With<ControlsText>>,
) {
    for mut text in controls_query.iter_mut() {
        let controls_text = match **current_state {
            GameState::Walking => "CONTROLS - Walking:\n\nArrow Keys: Move\nF: Enter Vehicle",
            GameState::Driving => "CONTROLS - Car/SuperCar:\n\nArrow Keys: Drive\nSPACE: Turbo Boost (SuperCar only)\nHold keys for acceleration\nF: Exit Car\nFind the Bugatti Chiron for max speed!",
            GameState::Flying => "CONTROLS - Helicopter:\n\nArrow Keys: Move\nQ/E: Up/Down\nF: Exit Helicopter",
            GameState::Jetting => "CONTROLS - F16 Fighter:\n\nArrow Keys: Fly\nQ/E: Up/Down\nSPACE: Afterburner\nF: Exit F16",
        };
        text.0 = controls_text.to_string();
    }
}
