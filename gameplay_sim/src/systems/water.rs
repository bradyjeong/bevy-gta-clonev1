use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn water_wave_system() {
    // TODO: Implement water wave system
}

pub fn yacht_buoyancy_system() {
    // TODO: Implement yacht buoyancy system
}

pub fn yacht_water_constraint_system() {
    // TODO: Implement yacht water constraint system
}

#[derive(Component)]
pub struct WaterBody {
    pub size: f32,
    pub depth: f32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub position: Vec3,
}

#[derive(Component)]
pub struct Yacht {
    pub speed: f32,
    pub max_speed: f32,
    pub turn_speed: f32,
}

pub fn setup_lake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lake_size = 200.0;
    let lake_depth = 5.0;
    let lake_position = Vec3::new(300.0, -2.0, 300.0); // Positioned away from spawn and below ground
    
    // Create water surface
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(lake_size, lake_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.4, 0.8, 0.7),
            alpha_mode: AlphaMode::Blend,
            metallic: 0.0,
            perceptual_roughness: 0.1,
            ..default()
        })),
        Transform::from_translation(lake_position),
        WaterBody {
            size: lake_size,
            depth: lake_depth,
            wave_height: 0.5,
            wave_speed: 1.0,
            position: lake_position,
        },
        Collider::cuboid(lake_size / 2.0, 0.1, lake_size / 2.0),
        Sensor,
        Name::new("Lake"),
    ));
    
    // Create lake bottom
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(lake_size * 0.9, lake_size * 0.9))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.15, 0.1),
            ..default()
        })),
        Transform::from_xyz(lake_position.x, lake_position.y - lake_depth, lake_position.z),
        Name::new("Lake Bottom"),
    ));
}

pub fn setup_yacht(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let yacht_position = Vec3::new(300.0, -1.0, 300.0); // On the lake surface
    
    // Create yacht hull
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 2.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 0.9),
            ..default()
        })),
        Transform::from_translation(yacht_position),
        RigidBody::Dynamic,
        Collider::cuboid(4.0, 1.0, 10.0),
        Yacht {
            speed: 0.0,
            max_speed: 25.0,
            turn_speed: 2.0,
        },
        Name::new("Yacht"),
    ));
}

pub fn yacht_movement_system(
    input: Res<ButtonInput<KeyCode>>,
    mut yacht_query: Query<(&mut Transform, &mut Velocity, &Yacht)>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, yacht) in &mut yacht_query {
        let mut thrust = 0.0;
        let mut turn = 0.0;
        
        if input.pressed(KeyCode::KeyI) {
            thrust = yacht.max_speed;
        }
        if input.pressed(KeyCode::KeyK) {
            thrust = -yacht.max_speed * 0.5;
        }
        if input.pressed(KeyCode::KeyJ) {
            turn = -yacht.turn_speed;
        }
        if input.pressed(KeyCode::KeyL) {
            turn = yacht.turn_speed;
        }
        
        // Apply turning
        if turn != 0.0 {
            transform.rotate_y(turn * time.delta_secs());
        }
        
        // Apply thrust in forward direction
        if thrust == 0.0 {
            velocity.linvel *= 0.9; // Damping
        } else {
            let forward = transform.forward();
            velocity.linvel = forward.as_vec3() * thrust;
        }
    }
}

pub fn water_effects_system(
    mut water_query: Query<&mut Transform, With<WaterBody>>,
    time: Res<Time>,
) {
    for mut transform in &mut water_query {
        // Simple wave animation
        let wave_offset = (time.elapsed_secs() * 2.0).sin() * 0.1;
        transform.translation.y += wave_offset * 0.1;
    }
}
