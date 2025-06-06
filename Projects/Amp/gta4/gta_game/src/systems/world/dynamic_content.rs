use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::constants::*;
use crate::bundles::{VehicleVisibilityBundle, VisibleChildBundle};

pub fn dynamic_terrain_system(
    mut terrain_query: Query<&mut Transform, (With<DynamicTerrain>, Without<ActiveEntity>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DynamicTerrain>)>,
) {
    if let Ok(active_transform) = active_query.single() {
        if let Ok(mut terrain_transform) = terrain_query.single_mut() {
            // Keep terrain centered on active entity (player/car/helicopter) but slightly below
            terrain_transform.translation.x = active_transform.translation.x;
            terrain_transform.translation.z = active_transform.translation.z;
            terrain_transform.translation.y = 0.0; // At ground level
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        
        // CIRCULAR RADIUS SYSTEM PARAMETERS (Ultra-aggressive optimization for 60 FPS)
        let active_radius = 300.0;   // Further reduced radius for minimal entities
        let cleanup_radius = 400.0;  // Very close cleanup for maximum performance
        let spawn_density = 80.0;    // Very high density spacing to minimize entity count
        
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
    let mut rng = rand::thread_rng();
    
    // NOTE: Road generation now handled by road_network_system
    // Dramatically reduced spawn rates for 60 FPS target
    // Buildings away from roads (check for road conflicts using new road system)
    if !is_on_road_network(position) && rng.gen_range(0.0..1.0) < 0.08 { // Ultra-reduced from 0.15 to 0.08
        // Ensure no overlap with roads or other buildings
        if !has_content_at_position(position, existing_content, 25.0) {
            spawn_building(commands, position, meshes, materials);
        }
    }
    // Vehicles on roads only
    else if is_on_road_network(position) && rng.gen_range(0.0..1.0) < 0.04 { // Ultra-reduced from 0.08 to 0.04
        // Ensure no overlap with other vehicles or buildings
        if !has_content_at_position(position, existing_content, 40.0) {
            spawn_vehicle(commands, position, meshes, materials);
        }
    }
    // Trees in empty areas (away from roads and buildings)
    else if !is_on_road_network(position) && rng.gen_range(0.0..1.0) < 0.05 { // Ultra-reduced from 0.1 to 0.05
        // Ensure no overlap with buildings or roads
        if !has_content_at_position(position, existing_content, 10.0) {
            spawn_dynamic_tree(commands, position, meshes, materials);
        }
    }
    // NPCs on roads and near buildings  
    else if rng.gen_range(0.0..1.0) < 0.01 { // Ultra-reduced from 0.03 to 0.01
        // Ensure no overlap with other content
        if !has_content_at_position(position, existing_content, 5.0) {
            spawn_dynamic_npc(commands, position, meshes, materials);
        }
    }
}

fn is_on_road_network(position: Vec3) -> bool {
    // TODO: This should use the new road network system
    // For now, keeping simplified version to avoid breaking other systems
    // Will be replaced with proper road spline detection
    
    let grid_size = 200.0; // Larger grid to match new road chunks
    let road_width = 12.0; // Approximate road width
    
    let on_horizontal = (position.z % grid_size).abs() < road_width;
    let on_vertical = (position.x % grid_size).abs() < road_width;
    
    on_horizontal || on_vertical
}

fn spawn_road_segment(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let grid_size = 80.0;
    let road_width = 16.0; // Visual road width (wider for better continuity)
    let segment_size = 25.0; // Size of each road segment - increased to overlap and ensure continuity
    
    // Create asphalt material with better appearance
    let road_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.25, 0.25), // Darker asphalt
        ..default()
    });
    
    // Create yellow lane markings material
    let marking_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0), // Bright yellow
        ..default()
    });
    
    // Determine road type based on position in grid
    let x_mod = position.x % grid_size;
    let z_mod = position.z % grid_size;
    let road_threshold = 8.0;
    
    let is_on_vertical_road = x_mod.abs() < road_threshold || x_mod.abs() > (grid_size - road_threshold);
    let is_on_horizontal_road = z_mod.abs() < road_threshold || z_mod.abs() > (grid_size - road_threshold);
    
    let road_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Road,
        },
        Transform::from_translation(Vec3::new(position.x, 0.05, position.z)),
        Visibility::Visible,
    )).id();
    
    if is_on_vertical_road && is_on_horizontal_road {
        // Intersection - create a square
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(road_width, 0.1, road_width))),
            MeshMaterial3d(road_material.clone()),
            Transform::from_translation(Vec3::ZERO),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
    } else if is_on_horizontal_road {
        // Horizontal road segment - make it continuous with overlap
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(segment_size * 1.5, 0.1, road_width))),
            MeshMaterial3d(road_material.clone()),
            Transform::from_translation(Vec3::ZERO),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
        
        // Center line marking for horizontal road
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(segment_size * 1.3, 0.11, 0.4))),
            MeshMaterial3d(marking_material),
            Transform::from_translation(Vec3::new(0.0, 0.01, 0.0)),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
    } else if is_on_vertical_road {
        // Vertical road segment - make it continuous with overlap
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(road_width, 0.1, segment_size * 1.5))),
            MeshMaterial3d(road_material.clone()),
            Transform::from_translation(Vec3::ZERO),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
        
        // Center line marking for vertical road
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.4, 0.11, segment_size * 1.3))),
            MeshMaterial3d(marking_material),
            Transform::from_translation(Vec3::new(0.0, 0.01, 0.0)),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
    }
}

fn spawn_building(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    
    let height = rng.gen_range(8.0..30.0);
    let width = rng.gen_range(8.0..15.0);
    
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
        Transform::from_translation(Vec3::new(position.x, height / 2.0, position.z)),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, height / 2.0 + 10.0, width / 2.0),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
        Building,
        Cullable { max_distance: 300.0, is_culled: false }, // Reduced from 800 to 300
    ));
}

fn spawn_vehicle(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    
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
    
    let color = car_colors[rng.gen_range(0..car_colors.len())];
    
    // Create car parent entity with physics
    let car_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Vehicle,
        },
        Car,
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.5, 2.0),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        Velocity::zero(),
        Transform::from_xyz(position.x, 1.5, position.z),
        VehicleVisibilityBundle::default(),
        Cullable { max_distance: 150.0, is_culled: false }, // Reduced for better performance
    )).id();

    // Car body (main hull)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.8, 0.6, 3.6))),
        MeshMaterial3d(materials.add(color)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ChildOf(car_entity),
        VisibleChildBundle::default(),
    ));
}

fn spawn_dynamic_tree(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let tree_entity = commands.spawn((
        DynamicContent {
            content_type: ContentType::Tree,
        },
        Transform::from_xyz(position.x, 0.0, position.z),
        VehicleVisibilityBundle::default(),
        Cullable { max_distance: 200.0, is_culled: false }, // Reduced from 400 to 200
    )).id();

    // Tree trunk
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 8.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.2, 0.1))),
        Transform::from_xyz(0.0, 4.0, 0.0),
        RigidBody::Fixed,
        Collider::cylinder(4.0, 0.4),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
        ChildOf(tree_entity),
        VisibleChildBundle::default(),
    ));

    // Tree foliage
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(3.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.6, 0.1))),
        Transform::from_xyz(0.0, 7.0, 0.0),
        ChildOf(tree_entity),
        VisibleChildBundle::default(),
    ));
}

fn spawn_dynamic_npc(
    commands: &mut Commands,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    
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
        DynamicContent {
            content_type: ContentType::NPC,
        },
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.8))),
        MeshMaterial3d(materials.add(color)),
        Transform::from_xyz(position.x, 1.0, position.z),
        RigidBody::Dynamic,
        Collider::capsule(Vec3::new(0.0, -0.9, 0.0), Vec3::new(0.0, 0.9, 0.0), 0.3),
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        NPC {
            target_position: Vec3::new(target_x, 1.0, target_z),
            speed: rng.gen_range(2.0..5.0),
            last_update: 0.0,
            update_interval: rng.gen_range(0.05..0.2),
        },
        Cullable { max_distance: 100.0, is_culled: false }, // Reduced from 200 to 100 for NPCs
    ));
}
