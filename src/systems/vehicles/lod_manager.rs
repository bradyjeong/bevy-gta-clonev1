use bevy::prelude::*;
use crate::components::{
    VehicleState, VehicleRendering, VehicleLOD, VehicleType, ActiveEntity,
    LOD_FULL_DISTANCE, LOD_MEDIUM_DISTANCE, LOD_LOW_DISTANCE, LOD_CULL_DISTANCE
};
use crate::bundles::VisibleChildBundle;
// Simplified without timing service
use crate::systems::distance_cache::{DistanceCache, get_cached_distance};
use crate::factories::{MaterialFactory, MeshFactory, TransformFactory};

pub fn vehicle_lod_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    active_query: Query<(Entity, &Transform), With<ActiveEntity>>,
    mut vehicle_query: Query<(Entity, &mut VehicleState, Option<&VehicleRendering>, &Transform)>,
    mut distance_cache: ResMut<DistanceCache>,
) {
    // Simplified LOD system without timing service
    
    let Ok((active_entity, active_transform)) = active_query.single() else { return };
    let player_pos = active_transform.translation;
    
    for (entity, mut vehicle_state, rendering, transform) in vehicle_query.iter_mut() {
        
        let distance = get_cached_distance(
            active_entity,
            entity,
            player_pos,
            transform.translation,
            &mut distance_cache,
        );
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
    // Add hysteresis to prevent flickering at distance boundaries
    const HYSTERESIS: f32 = 5.0; // 5m buffer zone
    
    if distance > LOD_CULL_DISTANCE + HYSTERESIS {
        VehicleLOD::StateOnly
    } else if distance > LOD_LOW_DISTANCE + HYSTERESIS {
        VehicleLOD::Low
    } else if distance > LOD_MEDIUM_DISTANCE + HYSTERESIS {
        VehicleLOD::Medium
    } else if distance > LOD_FULL_DISTANCE + HYSTERESIS {
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
            // Main body - USING MATERIAL FACTORY
            let body = commands.spawn((
                Mesh3d(MeshFactory::create_sports_car_body(meshes)),
                MeshMaterial3d(MaterialFactory::create_vehicle_metallic(materials, vehicle_state.color)),
                TransformFactory::vehicle_chassis(),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
            
            // Wheels
            for (x, z) in [(-1.0, 1.2), (1.0, 1.2), (-1.0, -1.2), (1.0, -1.2)] {
                let wheel = commands.spawn((
                    Mesh3d(MeshFactory::create_standard_wheel(meshes)),
                    MeshMaterial3d(MaterialFactory::create_wheel_material(materials)),
                    TransformFactory::wheel_with_rotation(x, -0.35, z),
                    ChildOf(parent_entity),
                    VisibleChildBundle::default(),
                )).id();
                mesh_entities.push(wheel);
            }
        },
        VehicleType::BasicCar => {
            let body = commands.spawn((
                Mesh3d(MeshFactory::create_car_body(meshes)),
                MeshMaterial3d(MaterialFactory::create_simple_material(materials, vehicle_state.color)),
                TransformFactory::vehicle_body_center(),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
        },
        VehicleType::Helicopter => {
            // Body
            let body = commands.spawn((
                Mesh3d(MeshFactory::create_suv_body(meshes)),
                MeshMaterial3d(MaterialFactory::create_simple_material(materials, Color::srgb(0.9, 0.9, 0.9))),
                TransformFactory::helicopter_body(),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
            
            // Main rotor blades
            for i in 0..4 {
                let angle = i as f32 * std::f32::consts::PI / 2.0;
                let blade = commands.spawn((
                    Mesh3d(MeshFactory::create_rotor_blade(meshes)),
                    MeshMaterial3d(MaterialFactory::create_simple_material(materials, Color::srgb(0.05, 0.05, 0.05))),
                    TransformFactory::rotor_with_rotation(angle),
                    ChildOf(parent_entity),
                    VisibleChildBundle::default(),
                )).id();
                mesh_entities.push(blade);
            }
        },
        VehicleType::F16 => {
            // Fuselage
            let body = commands.spawn((
                Mesh3d(MeshFactory::create_truck_body(meshes)),
                MeshMaterial3d(MaterialFactory::create_aircraft_material(materials, Color::srgb(0.4, 0.4, 0.5))),
                TransformFactory::helicopter_body(),
                ChildOf(parent_entity),
                VisibleChildBundle::default(),
            )).id();
            mesh_entities.push(body);
            
            // Wings
            let wings = commands.spawn((
                Mesh3d(MeshFactory::create_helicopter_body(meshes)),
                MeshMaterial3d(MaterialFactory::create_aircraft_material(materials, Color::srgb(0.4, 0.4, 0.5))),
                TransformFactory::landing_skid_left(),
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
        VehicleType::F16 => (Vec3::new(15.0, 1.6, 1.5), Color::srgb(0.35, 0.37, 0.40)),
    };
    
    let body = commands.spawn((
        Mesh3d(MeshFactory::create_custom_cuboid(meshes, size.x, size.y, size.z)),
        MeshMaterial3d(MaterialFactory::create_simple_material(materials, color)),
        TransformFactory::vehicle_body_center(),
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
        VehicleType::F16 => Vec3::new(15.0, 1.6, 1.5),
    };
    
    let body = commands.spawn((
        Mesh3d(MeshFactory::create_custom_cuboid(meshes, size.x, size.y, size.z)),
        MeshMaterial3d(MaterialFactory::create_low_detail_material(materials, vehicle_state.color)),
        TransformFactory::vehicle_body_center(),
        ChildOf(parent_entity),
        VisibleChildBundle::default(),
    )).id();
    
    vec![body]
}
