use bevy::prelude::*;
use rand::Rng;
use std::cell::RefCell;
use crate::components::*;
use crate::bundles::VisibleChildBundle;
use crate::systems::world::unified_world::{
    UnifiedWorldManager, UnifiedChunkEntity, ContentLayer, ChunkCoord, ChunkState,
    UNIFIED_CHUNK_SIZE,
};
use crate::systems::world::road_network::{RoadNetwork, RoadSpline, RoadType, IntersectionType};
use crate::systems::world::road_mesh::{generate_road_mesh, generate_road_markings_mesh, generate_intersection_mesh};

// SIMPLIFIED CHUNK LOADING - Replaces complex layered state machine

// UNIFIED Y-COORDINATE SYSTEM (prevents z-fighting):
// - Terrain:      y = -0.15  (15cm below ground)
// - All Roads:    y =  0.0   (unified ground level - highways, main streets, side streets, alleys)
// - Road Markings:y =  0.01  (1cm above road surface)
// Unified road height prevents overlapping geometry and z-fighting issues

thread_local! {
    static LAYERED_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

// LAYERED CONTENT GENERATION SYSTEMS
// These systems generate content in layers: Roads -> Buildings -> Vehicles -> Vegetation
// Each layer waits for previous layers to complete before starting

/// Simplified system that generates all chunk content at once
/// Replaces the complex layered state machine with direct generation
pub fn layered_generation_coordinator(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    // Find chunks that need loading
    let mut chunks_to_load = Vec::new();
    for (chunk_coord, chunk) in &world_manager.chunks {
        if matches!(chunk.state, ChunkState::Loading) && 
           current_time - chunk.last_update > 0.1 {
            chunks_to_load.push(*chunk_coord);
        }
    }
    
    // Load chunks completely (up to budget)
    let mut loaded_this_frame = 0;
    for coord in chunks_to_load {
        if loaded_this_frame >= world_manager.max_chunks_per_frame {
            break;
        }
        
        // Generate all content layers in one pass
        generate_complete_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials, current_time);
        loaded_this_frame += 1;
    }
}

/// Generate all content for a chunk in one pass
/// Replaces the Roads -> Buildings -> Vehicles -> Vegetation pipeline
fn generate_complete_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    current_time: f32,
) {
    // Generate all content layers in sequence
    // 1. Roads (foundation for everything else)
    generate_roads_for_chunk(commands, world_manager, coord, meshes, materials);
    
    // 2. Buildings (depend on road layout)  
    generate_buildings_for_chunk(commands, world_manager, coord, meshes, materials);
    
    // 3. Vehicles (spawn on roads)
    generate_vehicles_for_chunk(commands, world_manager, coord, meshes, materials);
    
    // 4. Vegetation (fill empty spaces)
    generate_vegetation_for_chunk(commands, world_manager, coord, meshes, materials);
    
    // Mark chunk as complete
    let chunk = world_manager.get_chunk_mut(coord);
    chunk.state = ChunkState::Loaded { lod_level: 0 };
    chunk.last_update = current_time;
    
    info!("Generated complete chunk at {:?} in single pass", coord);
}

// Old layered state machine removed - using direct generation instead

/// Layer 1: Road Generation System
pub fn road_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|(coord, chunk)| {
            if matches!(chunk.state, ChunkState::Loading) && !chunk.roads_generated {
                Some(*coord)
            } else {
                None
            }
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_roads_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn generate_roads_for_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Use existing road network generation logic but adapted for unified system
    let new_road_ids = world_manager.road_network.generate_chunk_roads(coord.x, coord.z);
    
    // Create road entities and add to placement grid
    for road_id in new_road_ids {
        if let Some(road) = world_manager.road_network.roads.get(&road_id).cloned() {
            let road_entity = spawn_unified_road_entity(
                commands,
                coord,
                road_id,
                &road,
                meshes,
                materials,
            );
            
            // Add road to placement grid
            let samples = 20;
            for i in 0..samples {
                let t = i as f32 / (samples - 1) as f32;
                let road_point = road.evaluate(t);
                world_manager.placement_grid.add_entity(
                    road_point,
                    ContentType::Road,
                    road.road_type.width() * 0.5,
                );
            }
            
            // Add entity to chunk
            let chunk = world_manager.get_chunk_mut(coord);
            chunk.entities.push(road_entity);
        }
    }
    
    // INTERSECTION FIX: Detect and generate intersections after roads are created
    detect_and_spawn_intersections(commands, world_manager, coord, meshes, materials);
    
    // Mark roads as generated
    let chunk = world_manager.get_chunk_mut(coord);
    chunk.roads_generated = true;
}

fn spawn_unified_road_entity(
    commands: &mut Commands,
    chunk_coord: ChunkCoord,
    road_id: u32,
    road: &RoadSpline,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let center_pos = road.evaluate(0.5);
    
    let road_material = create_road_material(&road.road_type, materials);
    let marking_material = create_marking_material(materials);
    
    let road_entity = commands.spawn((
        UnifiedChunkEntity {
            coord: chunk_coord,
            layer: ContentLayer::Roads,
        },
        RoadEntity { road_id },
        Transform::from_translation(center_pos),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        DynamicContent {
            content_type: ContentType::Road,
        },
    )).id();
    
    // Road surface mesh - positioned at ground level (y = 0.0)
    let road_mesh = generate_road_mesh(road);
    commands.spawn((
        Mesh3d(meshes.add(road_mesh)),
        MeshMaterial3d(road_material),
        Transform::from_translation(-center_pos + Vec3::new(0.0, 0.0, 0.0)), // Ground level
        ChildOf(road_entity),
        VisibleChildBundle::default(),
    ));
    
    // Road markings - positioned exactly 1cm above road surface (y = 0.01)
    let marking_meshes = generate_road_markings_mesh(road);
    for marking_mesh in marking_meshes {
        commands.spawn((
            Mesh3d(meshes.add(marking_mesh)),
            MeshMaterial3d(marking_material.clone()),
            Transform::from_translation(-center_pos + Vec3::new(0.0, 0.01, 0.0)), // 1cm above road surface
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));
    }
    
    road_entity
}

/// INTERSECTION FIX: Detect and create intersections to prevent overlapping road conflicts
fn detect_and_spawn_intersections(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos();
    let chunk_size = UNIFIED_CHUNK_SIZE;
    let half_size = chunk_size * 0.5;
    
    // Collect all roads in and around this chunk
    let mut chunk_roads = Vec::new();
    for (road_id, road) in &world_manager.road_network.roads {
        // Check if road passes through or near this chunk
        let road_center = road.evaluate(0.5);
        if (road_center.x - chunk_center.x).abs() < chunk_size && 
           (road_center.z - chunk_center.z).abs() < chunk_size {
            chunk_roads.push((*road_id, road.clone()));
        }
    }
    
    // Find intersections between roads
    let mut detected_intersections = Vec::new();
    for i in 0..chunk_roads.len() {
        for j in (i + 1)..chunk_roads.len() {
            let (road1_id, road1) = &chunk_roads[i];
            let (road2_id, road2) = &chunk_roads[j];
            
            // Check for intersection between road1 and road2
            if let Some(intersection_point) = find_road_intersection(road1, road2) {
                println!("🚧 DEBUG: Intersection detected between road {} ({:?}) and road {} ({:?}) at {:?}", 
                    road1_id, road1.road_type, road2_id, road2.road_type, intersection_point);
                // Only create intersection if it's within this chunk bounds
                if intersection_point.x >= chunk_center.x - half_size &&
                   intersection_point.x <= chunk_center.x + half_size &&
                   intersection_point.z >= chunk_center.z - half_size &&
                   intersection_point.z <= chunk_center.z + half_size {
                    
                    // Determine intersection type and priority road type
                    let intersection_type = IntersectionType::Cross;
                    let dominant_road_type = determine_dominant_road_type(&road1.road_type, &road2.road_type);
                    
                    detected_intersections.push((
                        intersection_point,
                        vec![*road1_id, *road2_id],
                        intersection_type,
                        dominant_road_type,
                    ));
                }
            }
        }
    }
    
    // Create intersection entities
    for (position, connected_roads, intersection_type, road_type) in detected_intersections {
        println!("🚧 DEBUG: Creating intersection entity at {:?} with type {:?} and road type {:?}", 
            position, intersection_type, road_type);
        let intersection_id = world_manager.road_network.add_intersection(
            position,
            connected_roads,
            intersection_type,
        );
        
        if let Some(intersection) = world_manager.road_network.intersections.get(&intersection_id) {
            println!("🚧 DEBUG: Successfully spawned intersection entity {}", intersection_id);
            let intersection_entity = spawn_unified_intersection_entity(
                commands,
                coord,
                intersection_id,
                intersection,
                road_type,
                meshes,
                materials,
            );
            
            // Add entity to chunk
            let chunk = world_manager.get_chunk_mut(coord);
            chunk.entities.push(intersection_entity);
        }
    }
}

/// Find intersection point between two road splines
fn find_road_intersection(road1: &RoadSpline, road2: &RoadSpline) -> Option<Vec3> {
    let samples = 20;
    let intersection_threshold = 3.0; // Roads closer than 3 units are considered intersecting
    
    for i in 0..samples {
        let t1 = i as f32 / (samples - 1) as f32;
        let point1 = road1.evaluate(t1);
        
        for j in 0..samples {
            let t2 = j as f32 / (samples - 1) as f32;
            let point2 = road2.evaluate(t2);
            
            let distance = Vec3::new(point1.x - point2.x, 0.0, point1.z - point2.z).length();
            if distance < intersection_threshold {
                // Return midpoint as intersection
                return Some(Vec3::new(
                    (point1.x + point2.x) * 0.5,
                    0.0,
                    (point1.z + point2.z) * 0.5,
                ));
            }
        }
    }
    
    None
}

/// Determine which road type should dominate at intersection (higher priority wins)
fn determine_dominant_road_type(road_type1: &RoadType, road_type2: &RoadType) -> RoadType {
    if road_type1.priority() >= road_type2.priority() {
        *road_type1
    } else {
        *road_type2
    }
}

/// Spawn intersection entity with dominant road material to prevent color conflicts
fn spawn_unified_intersection_entity(
    commands: &mut Commands,
    chunk_coord: ChunkCoord,
    intersection_id: u32,
    intersection: &crate::systems::world::road_network::RoadIntersection,
    dominant_road_type: RoadType,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let position = intersection.position;
    
    // Use dominant road type material to prevent color conflicts
    let intersection_material = create_road_material(&dominant_road_type, materials);
    
    let intersection_entity = commands.spawn((
        UnifiedChunkEntity {
            coord: chunk_coord,
            layer: ContentLayer::Roads,
        },
        IntersectionEntity { intersection_id },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        DynamicContent {
            content_type: ContentType::Road,
        },
    )).id();
    
    // Generate intersection mesh - positioned at ground level (y = 0.0)
    let intersection_mesh = generate_intersection_mesh(intersection, &[]);
    commands.spawn((
        Mesh3d(meshes.add(intersection_mesh)),
        MeshMaterial3d(intersection_material),
        Transform::from_translation(Vec3::new(0.0, 0.01, 0.0)), // Slightly above road surface to prevent z-fighting
        ChildOf(intersection_entity),
        VisibleChildBundle::default(),
    ));
    
    intersection_entity
}

/// Layer 2: Building Generation System
pub fn building_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|(coord, chunk)| {
            if matches!(chunk.state, ChunkState::Loading) 
                && chunk.roads_generated 
                && !chunk.buildings_generated {
                Some(*coord)
            } else {
                None
            }
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_buildings_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn generate_buildings_for_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos();
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Determine building density based on distance from center
    let distance_from_center = Vec2::new(chunk_center.x, chunk_center.z).length();
    let building_density = (1.0 - (distance_from_center / 2000.0).min(0.8)).max(0.1);
    
    // Generate building positions - REDUCED: From 20 to 8 attempts
    let building_attempts = (building_density * 8.0) as usize;
    
    for _ in 0..building_attempts {
        let local_x = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(-half_size..half_size));
        let local_z = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(-half_size..half_size));
        let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);
        
        // Check if position is valid (not on road, not overlapping other buildings)
        if !is_on_road_unified(position, &world_manager.road_network) {
            let building_size = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(8.0..15.0));
            if world_manager.placement_grid.can_place(
                position,
                ContentType::Building,
                building_size * 0.5,
                building_size,
            ) {
                let building_entity = spawn_unified_building(
                    commands,
                    coord,
                    position,
                    distance_from_center,
                    meshes,
                    materials,
                );
                
                // Add to placement grid
                world_manager.placement_grid.add_entity(
                    position,
                    ContentType::Building,
                    building_size * 0.5,
                );
                
                // Add entity to chunk
                let chunk = world_manager.get_chunk_mut(coord);
                chunk.entities.push(building_entity);
            }
        }
    }
    
    // Mark buildings as generated
    let chunk = world_manager.get_chunk_mut(coord);
    chunk.buildings_generated = true;
}

fn spawn_unified_building(
    commands: &mut Commands,
    chunk_coord: ChunkCoord,
    position: Vec3,
    _distance_from_center: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // REPLACED: Use UnifiedEntityFactory for building spawning
    // This eliminates duplicate building spawning code
    use crate::factories::entity_factory_unified::UnifiedEntityFactory;
    use crate::GameConfig;
    
    let mut factory = UnifiedEntityFactory::with_config(GameConfig::default());
    let current_time = 0.0; // Placeholder time
    
    match factory.spawn_building_consolidated(commands, meshes, materials, position, current_time) {
        Ok(entity) => {
            // Add chunk-specific components to maintain compatibility
            commands.entity(entity).insert((
                UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Buildings,
                },
            ));
            entity
        }
        Err(_) => {
            // Fallback to empty entity if spawn fails
            commands.spawn((
                Transform::from_translation(position),
                Visibility::Hidden,
            )).id()
        }
    }
}

/// Layer 3: Vehicle Generation System
pub fn vehicle_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|(coord, chunk)| {
            if matches!(chunk.state, ChunkState::Loading) 
                && chunk.buildings_generated 
                && !chunk.vehicles_generated {
                Some(*coord)
            } else {
                None
            }
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_vehicles_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn generate_vehicles_for_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos();
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Generate vehicles only on roads - REDUCED: From 5 to 2 attempts
    let vehicle_attempts = 2; // Conservative number to prevent overcrowding
    
    for _ in 0..vehicle_attempts {
        let local_x = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(-half_size..half_size));
        let local_z = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(-half_size..half_size));
        let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);
        
        // Only spawn on roads with sufficient spacing
        if is_on_road_unified(position, &world_manager.road_network) {
            if world_manager.placement_grid.can_place(
                position,
                ContentType::Vehicle,
                4.0, // Vehicle radius
                25.0, // Minimum distance between vehicles
            ) {
                let vehicle_entity = spawn_unified_vehicle(
                    commands,
                    coord,
                    position,
                    meshes,
                    materials,
                );
                
                // Add to placement grid
                world_manager.placement_grid.add_entity(
                    position,
                    ContentType::Vehicle,
                    4.0,
                );
                
                // Add entity to chunk
                let chunk = world_manager.get_chunk_mut(coord);
                chunk.entities.push(vehicle_entity);
            }
        }
    }
    
    // Mark vehicles as generated
    let chunk = world_manager.get_chunk_mut(coord);
    chunk.vehicles_generated = true;
}

fn spawn_unified_vehicle(
    commands: &mut Commands,
    chunk_coord: ChunkCoord,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // REPLACED: Use UnifiedEntityFactory for vehicle spawning
    // This eliminates duplicate vehicle spawning code
    use crate::factories::entity_factory_unified::UnifiedEntityFactory;
    use crate::GameConfig;
    
    let mut factory = UnifiedEntityFactory::with_config(GameConfig::default());
    let current_time = 0.0; // Placeholder time
    
    match factory.spawn_vehicle_consolidated(commands, meshes, materials, position, current_time) {
        Ok(entity) => {
            // Add chunk-specific components to maintain compatibility
            commands.entity(entity).insert((
                UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vehicles,
                },
            ));
            entity
        }
        Err(_) => {
            // Fallback to empty entity if spawn fails
            commands.spawn((
                Transform::from_translation(position),
                Visibility::Hidden,
            )).id()
        }
    }
}

/// Layer 4: Vegetation Generation System
pub fn vegetation_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|(coord, chunk)| {
            if matches!(chunk.state, ChunkState::Loading) 
                && chunk.vehicles_generated 
                && !chunk.vegetation_generated {
                Some(*coord)
            } else {
                None
            }
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_vegetation_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn generate_vegetation_for_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos();
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Generate trees and vegetation in open areas
    let vegetation_attempts = 8;
    
    for _ in 0..vegetation_attempts {
        let local_x = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(-half_size..half_size));
        let local_z = LAYERED_RNG.with(|rng| rng.borrow_mut().gen_range(-half_size..half_size));
        let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);
        
        // Only spawn vegetation away from roads and buildings
        if !is_on_road_unified(position, &world_manager.road_network) {
            if world_manager.placement_grid.can_place(
                position,
                ContentType::Tree,
                2.0, // Tree radius
                8.0, // Minimum distance between trees
            ) {
                let tree_entity = spawn_unified_tree(
                    commands,
                    coord,
                    position,
                    meshes,
                    materials,
                );
                
                // Add to placement grid
                world_manager.placement_grid.add_entity(
                    position,
                    ContentType::Tree,
                    2.0,
                );
                
                // Add entity to chunk
                let chunk = world_manager.get_chunk_mut(coord);
                chunk.entities.push(tree_entity);
            }
        }
    }
    
    // Mark vegetation as generated
    let chunk = world_manager.get_chunk_mut(coord);
    chunk.vegetation_generated = true;
}

fn spawn_unified_tree(
    commands: &mut Commands,
    chunk_coord: ChunkCoord,
    position: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    // REPLACED: Use UnifiedEntityFactory for tree spawning
    // This eliminates duplicate tree spawning code
    use crate::factories::entity_factory_unified::UnifiedEntityFactory;
    use crate::GameConfig;
    
    let mut factory = UnifiedEntityFactory::with_config(GameConfig::default());
    let current_time = 0.0; // Placeholder time
    
    match factory.spawn_tree_consolidated(commands, meshes, materials, position, current_time) {
        Ok(entity) => {
            // Add chunk-specific components to maintain compatibility
            commands.entity(entity).insert((
                UnifiedChunkEntity {
                    coord: chunk_coord,
                    layer: ContentLayer::Vegetation,
                },
            ));
            entity
        }
        Err(_) => {
            // Fallback to empty entity if spawn fails
            commands.spawn((
                Transform::from_translation(position),
                Visibility::Hidden,
            )).id()
        }
    }
}

// Utility functions

fn is_on_road_unified(position: Vec3, road_network: &RoadNetwork) -> bool {
    for road in road_network.roads.values() {
        if is_point_on_road_spline_unified(position, road, 25.0) {
            return true;
        }
    }
    false
}

fn is_point_on_road_spline_unified(position: Vec3, road: &RoadSpline, tolerance: f32) -> bool {
    let samples = 20; // Reduced samples for performance
    let width = road.road_type.width();
    
    for i in 0..samples {
        let t = i as f32 / (samples - 1) as f32;
        let road_point = road.evaluate(t);
        let distance = Vec3::new(position.x - road_point.x, 0.0, position.z - road_point.z).length();
        
        if distance <= width * 0.5 + tolerance {
            return true;
        }
    }
    
    false
}

fn create_road_material(road_type: &RoadType, materials: &mut ResMut<Assets<StandardMaterial>>) -> Handle<StandardMaterial> {
    let (base_color, roughness) = match road_type {
        RoadType::Highway => (Color::srgb(0.4, 0.4, 0.45), 0.8),
        RoadType::MainStreet => (Color::srgb(0.35, 0.35, 0.4), 0.8),
        RoadType::SideStreet => (Color::srgb(0.45, 0.45, 0.5), 0.7),
        RoadType::Alley => (Color::srgb(0.5, 0.5, 0.45), 0.6),
    };
    
    materials.add(StandardMaterial {
        base_color,
        perceptual_roughness: roughness,
        metallic: 0.0,
        reflectance: 0.2,
        emissive: Color::BLACK.into(),
        ..default()
    })
}

fn create_marking_material(materials: &mut ResMut<Assets<StandardMaterial>>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.95),
        emissive: LinearRgba::new(0.2, 0.2, 0.2, 1.0),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        reflectance: 0.5,
        ..default()
    })
}


