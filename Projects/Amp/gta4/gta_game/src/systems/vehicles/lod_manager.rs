use bevy::prelude::*;
use crate::components::{
    VehicleState, VehicleRendering, VehicleLOD, VehicleType, ActiveEntity,
    LOD_FULL_DISTANCE, LOD_MEDIUM_DISTANCE, LOD_LOW_DISTANCE, LOD_CULL_DISTANCE
};
use crate::bundles::VisibleChildBundle;

pub fn vehicle_lod_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut vehicle_query: Query<(Entity, &mut VehicleState, Option<&VehicleRendering>, &Transform)>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let player_pos = active_transform.translation;
    let current_time = time.elapsed_secs();
    
    // Check LOD every 0.1 seconds to avoid constant updates
    const LOD_CHECK_INTERVAL: f32 = 0.1;
    
    for (entity, mut vehicle_state, rendering, transform) in vehicle_query.iter_mut() {
        // Skip if checked recently
        if current_time - vehicle_state.last_lod_check < LOD_CHECK_INTERVAL {
            continue;
        }
        
        vehicle_state.last_lod_check = current_time;
        let distance = player_pos.distance(transform.translation);
        
        let new_lod = determine_lod(distance);
        
        if new_lod != vehicle_state.current_lod {
            // LOD changed - update rendering
            update_vehicle_lod(
                entity,
                &mut vehicle_state,
                rendering,
                new_lod,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
    }
}

fn determine_lod(distance: f32) -> VehicleLOD {
    if distance > LOD_CULL_DISTANCE {
        VehicleLOD::StateOnly
    } else if distance > LOD_LOW_DISTANCE {
        VehicleLOD::Low
    } else if distance > LOD_MEDIUM_DISTANCE {
        VehicleLOD::Medium
    } else if distance > LOD_FULL_DISTANCE {
        VehicleLOD::Medium
    } else {
        VehicleLOD::Full
    }
}

fn update_vehicle_lod(
    entity: Entity,
    vehicle_state: &mut VehicleState,
    current_rendering: Option<&VehicleRendering>,
    new_lod: VehicleLOD,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Remove existing rendering if present
    if let Some(rendering) = current_rendering {
        // Despawn all mesh children
        for mesh_entity in &rendering.mesh_entities {
            commands.entity(*mesh_entity).despawn();
        }
        commands.entity(entity).remove::<VehicleRendering>();
    }
    
    vehicle_state.current_lod = new_lod;
    
    // Add new rendering if needed
    if new_lod != VehicleLOD::StateOnly {
        let mesh_entities = spawn_vehicle_meshes(
            entity,
            vehicle_state,
            new_lod,
            commands,
            meshes,
            materials,
        );
        
        commands.entity(entity).insert(VehicleRendering {
            lod_level: new_lod,
            mesh_entities,
        });
    }
}

fn spawn_vehicle_meshes(
    parent_entity: Entity,
    vehicle_state: &VehicleState,
    lod: VehicleLOD,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    match lod {
        VehicleLOD::Full => spawn_full_vehicle_mesh(parent_entity, vehicle_state, commands, meshes, materials),
        VehicleLOD::Medium => spawn_medium_vehicle_mesh(parent_entity, vehicle_state, commands, meshes, materials),
        VehicleLOD::Low => spawn_low_vehicle_mesh(parent_entity, vehicle_state, commands, meshes, materials),
        VehicleLOD::StateOnly => Vec::new(),
    }
}

fn spawn_full_vehicle_mesh(
    parent_entity: Entity,
    vehicle_state: &VehicleState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    let mut mesh_entities = Vec::new();
    
    match vehicle_state.vehicle_type {
        VehicleType::SuperCar => {
            // Main body
            let body = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.8, 0.4, 4.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: vehicle_state.color,
                    metallic: 0.95,
                    perceptual_roughness: 0.1,
                    reflectance: 0.9,
                    ..default()
                })),
                Transform::from_xyz(0.0, -0.1, 0.0),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
            
            // Wheels
            for (x, z) in [(-1.0, 1.2), (1.0, 1.2), (-1.0, -1.2), (1.0, -1.2)] {
                let wheel = commands.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.35, 0.25))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.1, 0.1, 0.1),
                        metallic: 0.1,
                        perceptual_roughness: 0.8,
                        ..default()
                    })),
                    Transform::from_xyz(x, -0.35, z).with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 2.0)),
                    ChildOf(parent_entity),
                    VisibleChildBundle::default(),
                )).id();
                mesh_entities.push(wheel);
            }
        },
        VehicleType::BasicCar => {
            let body = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 3.6))),
                MeshMaterial3d(materials.add(vehicle_state.color)),
                Transform::from_xyz(0.0, 0.0, 0.0),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
        },
        VehicleType::Helicopter => {
            // Body
            let body = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(2.5, 1.5, 5.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.9, 0.9, 0.9))),
                Transform::from_xyz(0.0, 0.0, 0.0),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
            
            // Main rotor blades
            for i in 0..4 {
                let angle = i as f32 * std::f32::consts::PI / 2.0;
                let blade = commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(10.0, 0.05, 0.2))),
                    MeshMaterial3d(materials.add(Color::srgb(0.05, 0.05, 0.05))),
                    Transform::from_xyz(0.0, 2.2, 0.0).with_rotation(Quat::from_rotation_y(angle)),
                    ChildOf(parent_entity),
                    VisibleChildBundle::default(),
                )).id();
                mesh_entities.push(blade);
            }
        },
        VehicleType::F16 => {
            // Fuselage
            let body = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(16.0, 2.0, 3.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.4, 0.4, 0.5),
                    metallic: 0.8,
                    perceptual_roughness: 0.2,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, 0.0),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
            
            // Wings
            let wings = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(4.0, 0.3, 8.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.4, 0.4, 0.5),
                    metallic: 0.8,
                    perceptual_roughness: 0.2,
                    ..default()
                })),
                Transform::from_xyz(-2.0, -0.2, 0.0),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(wings);
        }
    }
    
    mesh_entities
}

fn spawn_medium_vehicle_mesh(
    parent_entity: Entity,
    vehicle_state: &VehicleState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    // Single simplified mesh for all vehicle types
    let (size, color) = match vehicle_state.vehicle_type {
        VehicleType::SuperCar => (Vec3::new(1.8, 0.6, 4.0), vehicle_state.color),
        VehicleType::BasicCar => (Vec3::new(1.8, 0.6, 3.6), vehicle_state.color),
        VehicleType::Helicopter => (Vec3::new(2.5, 1.5, 5.0), Color::srgb(0.9, 0.9, 0.9)),
        VehicleType::F16 => (Vec3::new(16.0, 2.0, 3.0), Color::srgb(0.4, 0.4, 0.5)),
    };
    
    let body = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(parent_entity),
        VisibleChildBundle::default(),
    )).id();
    
    vec![body]
}

fn spawn_low_vehicle_mesh(
    parent_entity: Entity,
    vehicle_state: &VehicleState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    // Very basic box representation
    let size = match vehicle_state.vehicle_type {
        VehicleType::SuperCar | VehicleType::BasicCar => Vec3::new(2.0, 1.0, 4.0),
        VehicleType::Helicopter => Vec3::new(3.0, 2.0, 5.0),
        VehicleType::F16 => Vec3::new(16.0, 2.0, 3.0),
    };
    
    let body = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: vehicle_state.color,
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(parent_entity),
        VisibleChildBundle::default(),
    )).id();
    
    vec![body]
}
