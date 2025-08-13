use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;

pub fn setup_lake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lake_size = 200.0;
    let lake_depth = 5.0;
    
    // Create lake water surface
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(lake_size, lake_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.4, 0.8, 0.7),
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.8,
            metallic: 0.1,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Lake {
            size: lake_size,
            depth: lake_depth,
            wave_height: 0.5,
            wave_speed: 1.0,
        },
        WaterBody,
        RigidBody::Fixed,
        Collider::cuboid(lake_size / 2.0, 0.1, lake_size / 2.0),
        Sensor,
        Name::new("Lake"),
    ));

    // Create lake bottom (for depth visualization)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(lake_size, lake_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.15, 0.1),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, -lake_depth, 0.0),
        Name::new("Lake Bottom"),
    ));
}

pub fn setup_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Yacht hull
    let yacht_id = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 2.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 0.9),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(4.0, 1.0, 10.0),
        Yacht {
            speed: 0.0,
            max_speed: 25.0,
            turning_speed: 2.0,
            buoyancy: 15.0,
            wake_enabled: true,
        },
        Boat,
        Cullable::new(300.0),
        ActiveEntity,
        Name::new("Yacht"),
    )).id();

    // Yacht cabin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(6.0, 3.0, 8.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.9),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(0.0, 3.5, -2.0),
        Name::new("Yacht Cabin"),
    )).insert(ChildOf(yacht_id));

    // Yacht mast
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.2, 15.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.4, 0.2),
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(0.0, 9.5, 2.0),
        Name::new("Yacht Mast"),
    )).insert(ChildOf(yacht_id));
}

pub fn yacht_movement_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut yacht_query: Query<(&mut Transform, &mut Yacht, &mut Velocity), With<Boat>>,
) {
    for (mut transform, mut yacht, mut velocity) in yacht_query.iter_mut() {
        let mut acceleration = Vec3::ZERO;
        let mut angular_velocity = 0.0;

        // Forward/backward movement
        if keys.pressed(KeyCode::KeyI) {
            acceleration += transform.forward() * yacht.max_speed;
        }
        if keys.pressed(KeyCode::KeyK) {
            acceleration -= transform.forward() * yacht.max_speed * 0.5;
        }

        // Turning
        if keys.pressed(KeyCode::KeyJ) {
            angular_velocity = yacht.turning_speed;
        }
        if keys.pressed(KeyCode::KeyL) {
            angular_velocity = -yacht.turning_speed;
        }

        // Apply rotation
        transform.rotate_y(angular_velocity * time.delta_secs());

        // Apply movement with water resistance
        let drag = 0.95;
        velocity.linvel = velocity.linvel * drag + acceleration * time.delta_secs() * 0.1;

        // Keep yacht on water surface (simple buoyancy)
        if transform.translation.y < 0.5 {
            velocity.linvel.y += yacht.buoyancy * time.delta_secs();
        }
    }
}

pub fn water_wave_system(
    time: Res<Time>,
    mut lake_query: Query<(&mut Transform, &Lake), With<WaterBody>>,
) {
    for (mut transform, lake) in lake_query.iter_mut() {
        let wave_offset = (time.elapsed_secs() * lake.wave_speed).sin() * lake.wave_height * 0.1;
        transform.translation.y = wave_offset;
    }
}

pub fn yacht_buoyancy_system(
    mut yacht_query: Query<(&mut Transform, &mut Velocity, &Yacht), With<Boat>>,
    lake_query: Query<&Transform, (With<WaterBody>, Without<Boat>)>,
) {
    if let Ok(water_transform) = lake_query.single() {
        for (mut yacht_transform, mut velocity, yacht) in yacht_query.iter_mut() {
            let water_level = water_transform.translation.y;
            let yacht_bottom = yacht_transform.translation.y - 1.0;
            
            if yacht_bottom < water_level {
                let submersion = water_level - yacht_bottom;
                let buoyancy_force = submersion * yacht.buoyancy;
                velocity.linvel.y += buoyancy_force * 0.1;
                
                // Damping in water
                velocity.linvel *= 0.98;
            }
        }
    }
}
