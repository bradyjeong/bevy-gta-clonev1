use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::{Lake, WaterBody, Yacht, Boat, Cullable};
use crate::factories::{MaterialFactory, RenderingFactory, StandardRenderingPattern, RenderingBundleType};

pub fn setup_lake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let lake_size = 200.0;
    let lake_depth = 5.0;
    let lake_position = Vec3::new(300.0, -2.0, 300.0); // Positioned away from spawn and below ground
    
    // Create lake basin (carved out ground) - FACTORY PATTERN
    let basin_entity = RenderingFactory::create_rendering_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        StandardRenderingPattern::WaterBottom { 
            size: lake_size, 
            color: Color::srgb(0.3, 0.25, 0.2) 
        },
        Vec3::new(lake_position.x, lake_position.y - lake_depth / 2.0, lake_position.z),
        RenderingBundleType::Standalone,
        None,
    );
    
    commands.entity(basin_entity).insert((
        RigidBody::Fixed,
        Collider::cylinder(lake_depth / 2.0, lake_size / 2.0),
        Name::new("Lake Basin"),
    ));
    
    // Create lake water surface - FACTORY PATTERN
    let water_entity = RenderingFactory::create_rendering_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        StandardRenderingPattern::WaterSurface { 
            size: lake_size, 
            color: Color::srgba(0.1, 0.4, 0.8, 0.7) 
        },
        Vec3::new(lake_position.x, lake_position.y, lake_position.z),
        RenderingBundleType::Standalone,
        None,
    );
    
    commands.entity(water_entity).insert((
        Lake {
            size: lake_size,
            depth: lake_depth,
            wave_height: 0.5,
            wave_speed: 1.0,
            position: lake_position,
        },
        WaterBody,
        RigidBody::Fixed,
        Collider::cuboid(lake_size / 2.0, 0.1, lake_size / 2.0),
        Sensor,
        Name::new("Lake"),
    ));

    // Create lake bottom
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(lake_size * 0.9, lake_size * 0.9))),
        MeshMaterial3d(MaterialFactory::create_water_bottom_material(&mut materials, Color::srgb(0.2, 0.15, 0.1))),
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
    
    // Yacht hull - FACTORY PATTERN
    let yacht_id = RenderingFactory::create_rendering_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        StandardRenderingPattern::VehicleBody { 
            vehicle_type: crate::factories::VehicleBodyType::Boat, 
            color: Color::srgb(0.9, 0.9, 0.9) 
        },
        yacht_position,
        RenderingBundleType::Parent,
        None,
    );
    
    commands.entity(yacht_id).insert((
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
        Name::new("Yacht"),
    ));

    // Yacht cabin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(6.0, 3.0, 8.0))),
        MeshMaterial3d(MaterialFactory::create_metallic_material(&mut materials, Color::srgb(0.8, 0.8, 0.9), 0.3, 0.4)),
        Transform::from_xyz(0.0, 3.5, -2.0),
        Name::new("Yacht Cabin"),
    )).insert(ChildOf(yacht_id));

    // Yacht mast
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.2, 15.0))),
        MeshMaterial3d(MaterialFactory::create_metallic_material(&mut materials, Color::srgb(0.6, 0.4, 0.2), 0.1, 0.8)),
        Transform::from_xyz(0.0, 9.5, 2.0),
        Name::new("Yacht Mast"),
    )).insert(ChildOf(yacht_id));
}

pub fn yacht_movement_system(
    time: Res<Time>,
    mut yacht_query: Query<(&mut Transform, &mut Yacht, &mut Velocity, &crate::components::ControlState), (With<Boat>, With<crate::components::ActiveEntity>)>,
) {
    for (mut transform, yacht, mut velocity, control_state) in yacht_query.iter_mut() {
        let mut acceleration = Vec3::ZERO;
        let mut angular_velocity = 0.0;

        // Forward/backward movement using ControlState
        if control_state.is_accelerating() {
            acceleration += transform.forward() * yacht.max_speed * control_state.throttle;
        }
        if control_state.is_braking() {
            acceleration -= transform.forward() * yacht.max_speed * control_state.brake * 0.5;
        }

        // Turning using ControlState steering
        if control_state.steering.abs() > 0.1 {
            angular_velocity = yacht.turning_speed * control_state.steering;
        }

        // Boost functionality
        let boost_multiplier = if control_state.is_boosting() { 2.0 } else { 1.0 };
        acceleration *= boost_multiplier;

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
    lake_query: Query<(&Transform, &Lake), (With<WaterBody>, Without<Boat>)>,
) {
    if let Ok((water_transform, lake)) = lake_query.single() {
        for (yacht_transform, mut velocity, yacht) in yacht_query.iter_mut() {
            let water_level = water_transform.translation.y;
            let yacht_bottom = yacht_transform.translation.y - 1.0;
            
            // Check if yacht is within lake boundaries
            let distance_from_center = Vec2::new(
                yacht_transform.translation.x - lake.position.x,
                yacht_transform.translation.z - lake.position.z,
            ).length();
            
            let max_distance = lake.size / 2.0 - 10.0; // 10m buffer from edge
            
            if distance_from_center > max_distance {
                // Push yacht back toward lake center
                let direction_to_center = Vec2::new(
                    lake.position.x - yacht_transform.translation.x,
                    lake.position.z - yacht_transform.translation.z,
                ).normalize();
                
                velocity.linvel.x += direction_to_center.x * 5.0;
                velocity.linvel.z += direction_to_center.y * 5.0;
            }
            
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

pub fn yacht_water_constraint_system(
    mut yacht_query: Query<(&mut Transform, &mut Velocity), With<Yacht>>,
    lake_query: Query<&Lake, With<WaterBody>>,
) {
    if let Ok(lake) = lake_query.single() {
        for (mut transform, mut velocity) in yacht_query.iter_mut() {
            // Ensure yacht stays above ground level
            if transform.translation.y < -4.0 {
                transform.translation.y = -1.0;
                velocity.linvel.y = 0.0;
            }
            
            // Keep yacht within a reasonable distance of lake
            let distance_from_lake = Vec2::new(
                transform.translation.x - lake.position.x,
                transform.translation.z - lake.position.z,
            ).length();
            
            if distance_from_lake > lake.size {
                // Teleport yacht back to lake if it gets too far
                transform.translation.x = lake.position.x;
                transform.translation.z = lake.position.z;
                transform.translation.y = lake.position.y + 1.0;
                velocity.linvel = Vec3::ZERO;
            }
        }
    }
}
