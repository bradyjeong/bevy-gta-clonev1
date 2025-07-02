use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::cell::RefCell;
use crate::components::*;
use crate::components::world::EntityLimits;
use crate::constants::*;
use crate::bundles::{VehicleVisibilityBundle, VisibleChildBundle};
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::road_generation::is_on_road_spline;


thread_local! {
    static CONTENT_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

pub fn dynamic_terrain_system(
    mut terrain_query: Query<&mut Transform, (With<DynamicTerrain>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicTerrain>)>,
) {
    if let Ok(active_transform) = active_query.single() {
        if let Ok(mut terrain_transform) = terrain_query.single_mut() {
            // Only move terrain if player has moved significantly to prevent sliding
            let distance_moved = (active_transform.translation.xz() - terrain_transform.translation.xz()).length();
            
            if distance_moved > 50.0 {  // Only follow when player moves 50+ units
                terrain_transform.translation.x = active_transform.translation.x;
                terrain_transform.translation.z = active_transform.translation.z;
                terrain_transform.translation.y = -0.1; // 10cm below road surface to prevent z-fighting
            }
        }
    }
}

// Add timer to reduce frequency of dynamic content checks
#[derive(Default)]
pub struct DynamicContentTimer {
    timer: f32,
    last_player_pos: Option<Vec3>,
}

pub fn dynamic_content_system(
    mut commands: Commands,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicContent>)>,
    content_query: Query<(Entity, &Transform, &DynamicContent)>,
    existing_vehicles_query: Query<&Transform, (With<Car>, Without<DynamicContent>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut entity_limits: ResMut<EntityLimits>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    mut timer: Local<DynamicContentTimer>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        
        // Update timer
        timer.timer += time.delta_secs();
        
        // Only process dynamic content every 2.0 seconds OR when player moves significantly
        let movement_threshold = 100.0;
        let player_moved = timer.last_player_pos
            .map(|last_pos| active_pos.distance(last_pos) > movement_threshold)
            .unwrap_or(true);
        
        let should_update = timer.timer >= 2.0 || player_moved;
        
        if !should_update {
            return;
        }
        
        timer.timer = 0.0;
        timer.last_player_pos = Some(active_pos);
        
        // EMERGENCY PERFORMANCE MODE - Drastically reduce entity spawning
        let active_radius = 150.0;   // HALVED: Minimal spawn radius
        let cleanup_radius = 2500.0;  // Match road cleanup radius to prevent premature despawning
        let spawn_density = 120.0;   // INCREASED: Much higher spacing between entities
        
        // Phase 1: Remove content outside cleanup radius (truly circular)
        for (entity, content_transform, _) in content_query.iter() {
            let distance = active_pos.distance(content_transform.translation);
            if distance > cleanup_radius {
                commands.entity(entity).despawn();
            }
        }
        
        // Phase 2: Collect existing content for collision avoidance
        let mut existing_content: Vec<(Vec3, ContentType, f32)> = content_query.iter()
            .map(|(_, transform, dynamic_content)| {
                let radius = match dynamic_content.content_type {
                    ContentType::Building => 20.0,
                    ContentType::Road => 15.0,
                    ContentType::Tree => 8.0,
                    ContentType::Vehicle => 25.0,
                    ContentType::NPC => 3.0,
                };
                (transform.translation, dynamic_content.content_type.clone(), radius)
            })
            .collect();
            
        // Add existing vehicles (non-dynamic) to the collision avoidance list with larger radius
        for vehicle_transform in existing_vehicles_query.iter() {
            existing_content.push((vehicle_transform.translation, ContentType::Vehicle, 25.0));
        }
        
        // Phase 3: TRUE CIRCULAR SPAWNING using polar coordinates
        // Generate content in concentric circles around the active entity
        let mut spawn_attempts = 0;
        let max_spawn_attempts = 50; // Reduced spawn attempts for better performance
        
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
                    spawn_dynamic_content_safe(&mut commands, spawn_pos, &existing_content, &mut meshes, &mut materials, &mut entity_limits, &road_network, time.elapsed_secs());
                }
            }
            if spawn_attempts > max_spawn_attempts { break; }
        }
    }
}

fn has_content_at_position(position: Vec3, existing_content: &[(Vec3, ContentType, f32)], min_distance: f32) -> bool {
    existing_content.iter().any(|(existing_pos, _, radius)| {
        // Fixed: Use sum of distances plus buffer instead of max
        let required_distance = min_distance + radius + 2.0; // 2.0 buffer
        position.distance(*existing_pos) < required_distance
    })
}

fn spawn_dynamic_content_safe(
    commands: &mut Commands,
    position: Vec3,
    existing_content: &[(Vec3, ContentType, f32)],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_limits: &mut ResMut<EntityLimits>,
    road_network: &RoadNetwork,
    current_time: f32,
) {
    
    // Buildings - re-enabled with proper road collision detection
    let on_road = is_on_road_spline(position, road_network, 25.0);
    if CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < 0.004 { // Reduced building density for better performance
        if on_road {
            println!("DEBUG: Skipping building at {:?} - on road", position);
        } else if is_in_water_area(position) {
            println!("DEBUG: Skipping building at {:?} - in water", position);
        } else if has_content_at_position(position, existing_content, 35.0) {
            println!("DEBUG: Skipping building at {:?} - content collision", position);
        } else {
            println!("DEBUG: Spawning building at {:?}", position);
            spawn_building(commands, position, meshes, materials, entity_limits, current_time);
        }
    }
    // Vehicles on roads only
    if is_on_road_spline(position, road_network, 8.0) && CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < 0.002 { // EMERGENCY: 0.02 -> 0.002 (10x reduction)
        // Ensure no overlap with other vehicles or buildings (increased spacing)
        if !has_content_at_position(position, existing_content, 80.0) {
            spawn_vehicle(commands, position, meshes, materials, entity_limits, current_time);
        }
    }
    // Trees in empty areas (away from roads and buildings)
    else if !is_on_road_spline(position, road_network, 15.0) && !is_in_water_area(position) && CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < 0.15 { // Increased from 0.003 to 0.15 (15%)
        // Ensure no overlap with buildings or roads
        if !has_content_at_position(position, existing_content, 10.0) {
            spawn_dynamic_tree(commands, position, meshes, materials, entity_limits, current_time);
        }
    }
    // NPCs on roads and near buildings  
    else if CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.0..1.0)) < 0.0005 { // EMERGENCY: 0.005 -> 0.0005 (10x reduction)
        // Ensure no overlap with other content
        if !has_content_at_position(position, existing_content, 5.0) {
            spawn_dynamic_npc(commands, position, meshes, materials, entity_limits, current_time);
        }
    }
}



fn is_in_water_area(position: Vec3) -> bool {
    // Lake position and size (must match water.rs setup)
    let lake_center = Vec3::new(300.0, -2.0, 300.0);
    let lake_size = 200.0;
    let buffer = 20.0; // Extra buffer around lake
    
    let distance = Vec2::new(
        position.x - lake_center.x,
        position.z - lake_center.z,
    ).length();
    
    distance < (lake_size / 2.0 + buffer)
}



pub fn vehicle_separation_system(
    mut vehicle_query: Query<(&mut Transform, &mut Velocity), (With<Car>, With<DynamicContent>)>,
) {
    let vehicles: Vec<(Vec3, Entity)> = vehicle_query.iter()
        .enumerate()
        .map(|(i, (transform, _))| (transform.translation, Entity::from_raw(i as u32)))
        .collect();
    
    for (mut transform, mut velocity) in vehicle_query.iter_mut() {
        let current_pos = transform.translation;
        
        for (other_pos, _) in &vehicles {
            if *other_pos == current_pos { continue; }
            
            let distance = current_pos.distance(*other_pos);
            if distance < 15.0 && distance > 0.1 { // Too close
                let separation_force = (current_pos - *other_pos).normalize() * (15.0 - distance) * 2.0;
                velocity.linvel += separation_force;
                
                // Also adjust position slightly to prevent exact overlap
                transform.translation += separation_force * 0.1;
            }
        }
    }
}



fn spawn_building(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_limits: &mut ResMut<EntityLimits>,
    current_time: f32,
) {
    let height = CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(8.0..30.0));
    let width = CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(8.0..15.0));
    
    let building_material = materials.add(StandardMaterial {
        base_color: Color::srgb(
            CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.5..0.9)),
            CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.5..0.9)),
            CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.5..0.9)),
        ),
        ..default()
    });
    
    // Check entity limits and despawn oldest if needed
    if entity_limits.building_entities.len() >= entity_limits.max_buildings {
        // Find oldest building and despawn it
        if let Some((oldest_entity, _)) = entity_limits.building_entities.first().copied() {
            commands.entity(oldest_entity).despawn();
            entity_limits.building_entities.remove(0);
        }
    }
    
    // Building positioned with base on terrain surface (y=-0.05) and mesh at half-height above ground
    let ground_level = -0.05; // Match terrain level
    let building_base_y = ground_level;
    let building_mesh_y = building_base_y + height / 2.0;
    
    let building_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Building,
        },
        Mesh3d(meshes.add(Cuboid::new(width, height, width))),
        MeshMaterial3d(building_material),
        Transform::from_translation(Vec3::new(position.x, building_mesh_y, position.z)),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, height / 2.0, width / 2.0),
        Building {
            building_type: BuildingType::Generic,
            height,
            scale: Vec3::new(width, height, width),
        },
        Cullable { max_distance: 300.0, is_culled: false },
    )).id();
    
    // Track the new building
    entity_limits.building_entities.push((building_entity, current_time));
}

fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_limits: &mut ResMut<EntityLimits>,
    current_time: f32,
) {
    let car_colors = [
        Color::srgb(1.0, 0.0, 0.0), // Red
        Color::srgb(0.0, 0.0, 1.0), // Blue
        Color::srgb(0.0, 1.0, 0.0), // Green
        Color::srgb(1.0, 1.0, 0.0), // Yellow
        Color::srgb(1.0, 0.0, 1.0), // Magenta
        Color::srgb(0.0, 1.0, 1.0), // Cyan
        Color::srgb(0.5, 0.5, 0.5), // Gray
        Color::srgb(1.0, 1.0, 1.0), // White
        Color::srgb(0.0, 0.0, 0.0), // Black
    ];
    
    let color = car_colors[CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0..car_colors.len()))];
    
    // Check entity limits and despawn oldest if needed
    if entity_limits.vehicle_entities.len() >= entity_limits.max_vehicles {
        if let Some((oldest_entity, _)) = entity_limits.vehicle_entities.first().copied() {
            commands.entity(oldest_entity).despawn();
            entity_limits.vehicle_entities.remove(0);
        }
    }
    
    // Create car parent entity with physics
    let car_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Vehicle,
        },
        Car,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),  // Half-height = 0.5, total height = 1.0
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(position.x, 0.5, position.z),  // Fixed: spawn at ground+half-height
        VehicleVisibilityBundle::default(),
        CollisionGroups::new(VEHICLE_GROUP, STATIC_GROUP | VEHICLE_GROUP | CHARACTER_GROUP),
        Damping { linear_damping: 1.0, angular_damping: 5.0 },
        Cullable { max_distance: 150.0, is_culled: false },
    )).id();

    // Car body (main hull) - Fixed: height matches collider
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 1.0, 3.6))),  // Fixed: height 1.0 matches collider
        MeshMaterial3d(materials.add(color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(car_entity),
        VisibleChildBundle::default(),
    ));
    
    // Track the new vehicle
    entity_limits.vehicle_entities.push((car_entity, current_time));
}

fn spawn_dynamic_tree(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_limits: &mut ResMut<EntityLimits>,
    current_time: f32,
) {
    // Check entity limits and despawn oldest if needed
    if entity_limits.tree_entities.len() >= entity_limits.max_trees {
        if let Some((oldest_entity, _)) = entity_limits.tree_entities.first().copied() {
            commands.entity(oldest_entity).despawn();
            entity_limits.tree_entities.remove(0);
        }
    }
    
    let tree_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Tree,
        },
        Transform::from_xyz(position.x, position.y, position.z),
        VehicleVisibilityBundle::default(),
        Cullable { max_distance: 200.0, is_culled: false },
    )).id();

    // Palm tree trunk - single brown cylinder
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.25, 0.15))), // Brown trunk
        Transform::from_xyz(0.0, 4.0, 0.0),
        ChildOf(tree_entity),
        VisibleChildBundle::default(),
    ));

    // Palm fronds - 4 green rectangles arranged in a cross
    for i in 0..4 {
        let angle = (i as f32) * std::f32::consts::PI / 2.0;
        
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.5, 0.1, 0.8))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 0.25))), // Green fronds
            Transform::from_xyz(
                angle.cos() * 1.2, 
                7.5, 
                angle.sin() * 1.2
            ).with_rotation(
                Quat::from_rotation_y(angle) * 
                Quat::from_rotation_z(-0.2) // Slight droop
            ),
            ChildOf(tree_entity),
            VisibleChildBundle::default(),
        ));
    }

    // Physics collider for palm trunk
    commands.spawn((
        RigidBody::Fixed,
        Collider::cylinder(4.0, 0.3),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
        Transform::from_xyz(0.0, 4.0, 0.0),
        ChildOf(tree_entity),
    ));
    
    // Track the new tree
    entity_limits.tree_entities.push((tree_entity, current_time));
}

fn spawn_dynamic_npc(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    entity_limits: &mut ResMut<EntityLimits>,
    current_time: f32,
) {
    let npc_colors = [
        Color::srgb(0.8, 0.6, 0.4), // Skin tone 1
        Color::srgb(0.6, 0.4, 0.3), // Skin tone 2
        Color::srgb(0.9, 0.7, 0.5), // Skin tone 3
        Color::srgb(0.7, 0.5, 0.4), // Skin tone 4
    ];
    
    let color = npc_colors[CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0..npc_colors.len()))];
    
    // Check entity limits and despawn oldest if needed
    if entity_limits.npc_entities.len() >= entity_limits.max_npcs {
        if let Some((oldest_entity, _)) = entity_limits.npc_entities.first().copied() {
            commands.entity(oldest_entity).despawn();
            entity_limits.npc_entities.remove(0);
        }
    }
    
    // Random target position for movement
    let target_x = CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(-900.0..900.0));
    let target_z = CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(-900.0..900.0));
    
    let npc_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::NPC,
        },
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
        MeshMaterial3d(materials.add(color)),
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -0.9, 0.0), Vec3::new(0.0, 0.9, 0.0), 0.3),
        Velocity::zero(),
        Transform::from_xyz(position.x, 1.0, position.z), // TODO: Replace with ground detection
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        NPC {
            target_position: Vec3::new(target_x, 1.0, target_z), // TODO: Replace with ground detection
            speed: CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(2.0..5.0)),
            last_update: 0.0,
            update_interval: CONTENT_RNG.with(|rng| rng.borrow_mut().gen_range(0.05..0.2)),
        },
        Cullable { max_distance: 100.0, is_culled: false },
    )).id();
    
    // Track the new NPC
    entity_limits.npc_entities.push((npc_entity, current_time));
}
