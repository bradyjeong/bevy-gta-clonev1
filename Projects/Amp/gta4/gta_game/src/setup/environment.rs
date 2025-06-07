use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use crate::components::*;
use crate::constants::*;
use crate::bundles::{VehicleVisibilityBundle, VisibleChildBundle};

pub fn setup_palm_trees(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
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
            VisibleChildBundle::default(),
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
                VisibleChildBundle::default(),
            ));
        }
    }
}

pub fn setup_luxury_cars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
            VehicleVisibilityBundle::default(),
        )).id();

        // Car body (main hull)
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 3.6))),
            MeshMaterial3d(materials.add(car_colors[i % car_colors.len()])),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ChildOf(car_entity),
            Cullable { max_distance: 300.0, is_culled: false },
            VisibleChildBundle::default(),
        ));

        // Car roof
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.4, 0.4, 2.4))),
            MeshMaterial3d(materials.add(car_colors[i % car_colors.len()].darker(0.3))),
            Transform::from_xyz(0.0, 0.5, -0.2),
            ChildOf(car_entity),
            VisibleChildBundle::default(),
        ));

        // Windshield
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.3, 0.35, 0.1))),
            MeshMaterial3d(materials.add(Color::srgba(0.7, 0.8, 1.0, 0.3))), // Blue tinted glass
            Transform::from_xyz(0.0, 0.5, 0.8),
            ChildOf(car_entity),
            VisibleChildBundle::default(),
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
                VisibleChildBundle::default(),
            ));
        }

        // Headlights
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.9))), // White/yellow headlight
            Transform::from_xyz(-0.5, 0.1, 1.7),
            ChildOf(car_entity),
            VisibleChildBundle::default(),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.9))),
            Transform::from_xyz(0.5, 0.1, 1.7),
            ChildOf(car_entity),
            VisibleChildBundle::default(),
        ));

        // Tail lights
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.12))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.2, 0.2))), // Red tail light
            Transform::from_xyz(-0.5, 0.1, -1.7),
            ChildOf(car_entity),
            VisibleChildBundle::default(),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.12))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.2, 0.2))),
            Transform::from_xyz(0.5, 0.1, -1.7),
            ChildOf(car_entity),
            VisibleChildBundle::default(),
        ));
    }
}

pub fn setup_npcs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // NPCs scattered throughout the massive world
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
}

// Disabled - Buildings are now fully dynamic
#[allow(dead_code)]
pub fn setup_buildings(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    
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
            Transform::from_xyz(x, height / 2.0, z), // Visual mesh centered at its height  
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0 + 2.0, height / 2.0 + 10.0, depth / 2.0 + 2.0), // Large collider extending way below ground
            Cullable { max_distance: 800.0, is_culled: false },
            CollisionGroups::new(STATIC_GROUP, Group::ALL),
            Building {
                building_type: BuildingType::Generic,
                height: 30.0,
                scale: Vec3::new(20.0, 30.0, 20.0),
            },
        ));
    }
}
