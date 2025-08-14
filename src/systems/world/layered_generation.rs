use bevy::prelude::*;
use crate::components::*;
use crate::resources::GlobalRng;

use crate::world::chunk_coord::ChunkCoord;
use crate::world::chunk_data::{UnifiedChunkEntity, ContentLayer};
use crate::world::constants::UNIFIED_CHUNK_SIZE;
use crate::world::chunk_tracker::{ChunkProgress, ChunkTables};
use crate::world::chunk_coord::ChunkCoord as V2ChunkCoord;
use crate::world::placement_grid::PlacementGrid;
use crate::world::road_network::{RoadNetwork, RoadType};
use crate::events::world::chunk_events::{ChunkFinishedLoading, RequestChunkLoad};

// UNIFIED Y-COORDINATE SYSTEM (prevents z-fighting):
// - Terrain:      y = -0.15  (15cm below ground)
// - All Roads:    y =  0.0   (unified ground level - highways, main streets, side streets, alleys)
// - Road Markings:y =  0.01  (1cm above road surface)
// Unified road height prevents overlapping geometry and z-fighting issues



// LAYERED CONTENT GENERATION SYSTEMS
// These systems generate content in layers: Roads -> Buildings -> Vehicles -> Vegetation
// Each layer waits for previous layers to complete before starting

/// System that manages the progression through content generation layers
pub fn layered_generation_coordinator(
    tables: Res<ChunkTables>,
    mut progress: ResMut<ChunkProgress>,
    mut finished_writer: EventWriter<ChunkFinishedLoading>,
    time: Res<Time>,
    mut load_reader: EventReader<RequestChunkLoad>,
) {
    let current_time = time.elapsed_secs();
    
    // Also process any new chunk load requests
    for request in load_reader.read() {
        debug!("Processing RequestChunkLoad for chunk {:?}", request.coord);
    }
    
    // Process chunks that are in loading state
    let mut chunks_to_update = Vec::new();
    
    debug!("Checking {} chunks in loading state", tables.loading.len());
    
    for (chunk_coord, _) in &tables.loading {
        let last_update = progress.get_last_update(*chunk_coord);
        
        // Check if enough time has passed since last update (prevent frame drops)
        // PERFORMANCE: Increased from 0.1 to 0.2 seconds to reduce generation frequency
        if current_time - last_update > 0.2 {
            chunks_to_update.push(*chunk_coord);
            debug!("Will update chunk {:?} (last update: {}, current: {})", chunk_coord, last_update, current_time);
        }
    }
    
    // Update chunk generation progress
    for coord in chunks_to_update {
        advance_chunk_generation(coord, current_time, &mut progress, &mut finished_writer);
    }
}

fn advance_chunk_generation(
    coord: V2ChunkCoord,
    current_time: f32,
    progress: &mut ChunkProgress,
    finished_writer: &mut EventWriter<ChunkFinishedLoading>,
) {
    // Update timing in v2 progress
    progress.last_update.insert(coord, current_time);

    // Decide next layer using v2 progress flags
    let (roads_done, buildings_done, vehicles_done, vegetation_done) = (
        *progress.roads_generated.get(&coord).unwrap_or(&false),
        *progress.buildings_generated.get(&coord).unwrap_or(&false),
        *progress.vehicles_generated.get(&coord).unwrap_or(&false),
        *progress.vegetation_generated.get(&coord).unwrap_or(&false),
    );

    let _next_layer = if !roads_done {
        Some(ContentLayer::Roads)
    } else if !buildings_done {
        Some(ContentLayer::Buildings)
    } else if !vehicles_done {
        Some(ContentLayer::Vehicles)
    } else if !vegetation_done {
        Some(ContentLayer::Vegetation)
    } else {
        // All layers complete - chunk generation finished
        // Emit event to transition chunk state from Loading to Loaded
        let chunk_coord = crate::events::world::chunk_events::ChunkCoord::new(coord.x, coord.z);
        
        // Calculate LOD based on chunk distance (assuming default distance for now)
        // In production, you'd track the actual distance from player
        let lod_level = 0; // Default LOD for newly loaded chunks
        
        finished_writer.write(ChunkFinishedLoading::new(chunk_coord, lod_level));
        debug!("Chunk generation complete for {:?}, emitting ChunkFinishedLoading", coord);
        
        None
    };

    // Layer systems use v2 progress for selection
}

/// Layer 1: Road Generation System
pub fn road_layer_system(
    mut commands: Commands,
    mut progress: ResMut<ChunkProgress>,
    tables: Res<ChunkTables>,
    mut v2_grid: ResMut<PlacementGrid>,
    mut v2_roads: ResMut<RoadNetwork>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunks_to_process: Vec<V2ChunkCoord> = tables
        .loading
        .keys()
        .cloned()
        .filter(|coord| {
            let roads_done = *progress.roads_generated.get(coord).unwrap_or(&false);
            !roads_done
        })
        .collect();
    
    if !chunks_to_process.is_empty() {
        debug!("Processing {} chunks for road generation", chunks_to_process.len());
    }
    
    for coord in chunks_to_process {
        debug!("Generating roads for chunk {:?}", coord);
        generate_roads_for_chunk(&mut commands, coord, &mut meshes, &mut materials, &mut progress, &mut v2_grid, &mut v2_roads);
    }
}

fn generate_roads_for_chunk(
    commands: &mut Commands,
    coord: V2ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    progress: &mut ChunkProgress,
    v2_grid: &mut PlacementGrid,
    v2_roads: &mut RoadNetwork,
) {
    let chunk_center = Vec3::new(
        coord.x as f32 * UNIFIED_CHUNK_SIZE,
        0.0,
        coord.z as f32 * UNIFIED_CHUNK_SIZE,
    );
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Generate road nodes for this chunk using v2 RoadNetwork
    let road_type = if coord.x.abs() + coord.z.abs() < 2 {
        RoadType::MainStreet // Main roads near spawn
    } else {
        RoadType::SideStreet // Secondary roads elsewhere
    };
    
    // Create a simple grid of road nodes
    let node_spacing = 50.0; // Proper spacing for road network
    let nodes_per_side = ((UNIFIED_CHUNK_SIZE / node_spacing) + 1.0) as u8;
    
    for i in 0..nodes_per_side {
        for j in 0..nodes_per_side {
            let x = chunk_center.x - half_size + i as f32 * node_spacing;
            let z = chunk_center.z - half_size + j as f32 * node_spacing;
            
            let node_id = v2_roads.add_node(x, z);
            
            // Connect to adjacent nodes (horizontal and vertical)
            if i > 0 {
                let prev_id = node_id.saturating_sub(1);
                v2_roads.connect_nodes(prev_id, node_id);
            }
            if j > 0 && node_id >= nodes_per_side {
                let above_id = node_id - nodes_per_side;
                v2_roads.connect_nodes(above_id, node_id);
            }
            
            let world_pos = Vec3::new(x, 0.0, z);
            
            // Create road entity for this node
            let legacy_coord = ChunkCoord { x: coord.x, z: coord.z };
            let _road_entity = spawn_unified_road_entity_v2(
                commands,
                legacy_coord,
                node_id,
                world_pos,
                road_type,
                meshes,
                materials,
            );
            
            // Add road to placement grid
            v2_grid.add_entity(
                world_pos,
                crate::world::placement_grid::ContentType::Road,
                road_type.width() * 0.5,
            );
        }
    }
    
    // Mark roads as generated (v2 progress)
    progress.roads_generated.insert(coord, true);
}

fn spawn_unified_road_entity_v2(
    commands: &mut Commands,
    chunk_coord: ChunkCoord,
    node_id: u8,
    position: Vec3,
    road_type: RoadType,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let road_material = create_road_material(&road_type, materials);
    
    let road_entity = commands.spawn((
        UnifiedChunkEntity {
            coord: chunk_coord,
            layer: ContentLayer::Roads,
        },
        RoadEntity { road_id: node_id as u32 },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        DynamicContent {
            content_type: ContentType::Road,
        },
    )).id();
    
    // Create simple road mesh - a flat rectangle at ground level
    let road_width = road_type.width();
    let road_length = 50.0; // Standard road segment length
    
    let road_mesh = Mesh::from(Plane3d::default().mesh().size(road_width, road_length));
    commands.entity(road_entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(road_mesh)),
            MeshMaterial3d(road_material),
            Transform::from_translation(Vec3::ZERO),
        ));
    });
    
    road_entity
}

// Intersection handling simplified for v2 road network - connections are managed at the node level

/// Layer 2: Building Generation System
pub fn building_layer_system(
    mut commands: Commands,
    mut progress: ResMut<ChunkProgress>,
    tables: Res<ChunkTables>,
    v2_roads: Res<RoadNetwork>,
    mut v2_grid: ResMut<PlacementGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let chunks_to_process: Vec<V2ChunkCoord> = tables
        .loading
        .keys()
        .cloned()
        .filter(|coord| {
            let roads_done = *progress.roads_generated.get(coord).unwrap_or(&false);
            let buildings_done = *progress.buildings_generated.get(coord).unwrap_or(&false);
            roads_done && !buildings_done
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_buildings_for_chunk(&mut commands, coord, &mut meshes, &mut materials, &mut global_rng, &mut progress, &mut v2_grid, &v2_roads);
    }
}

fn generate_buildings_for_chunk(
    commands: &mut Commands,
    coord: V2ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    global_rng: &mut GlobalRng,
    progress: &mut ChunkProgress,
    v2_grid: &mut PlacementGrid,
    v2_roads: &RoadNetwork,
) {
    let chunk_center = Vec3::new(
        coord.x as f32 * UNIFIED_CHUNK_SIZE,
        0.0,
        coord.z as f32 * UNIFIED_CHUNK_SIZE,
    );
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Determine building density based on distance from center
    let distance_from_center = Vec2::new(chunk_center.x, chunk_center.z).length();
    let building_density = (1.0 - (distance_from_center / 2000.0).min(0.8)).max(0.1);
    
    // Generate building positions - REDUCED: From 20 to 8 attempts
    let building_attempts = (building_density * 8.0) as usize;
    
    for _ in 0..building_attempts {
        let local_x = global_rng.gen_range(-half_size..half_size);
        let local_z = global_rng.gen_range(-half_size..half_size);
        let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);
        
        // Check if position is valid (not on road, not overlapping other buildings)
        if !v2_roads.is_near_road(position, 6.0) {
            let building_size = global_rng.gen_range(8.0..15.0);
            let v2_ok = v2_grid.can_place(
                position,
                crate::world::placement_grid::ContentType::Building,
                building_size * 0.5,
            );
            if v2_ok {
                let legacy_coord = ChunkCoord { x: coord.x, z: coord.z };
                let _building_entity = spawn_unified_building(
                    commands,
                    legacy_coord,
                    position,
                    distance_from_center,
                    meshes,
                    materials,
                );
                
                // Add to placement grid (v2)
                v2_grid.add_entity(
                    position,
                    crate::world::placement_grid::ContentType::Building,
                    building_size * 0.5,
                );
                
                // V2: entity bookkeeping handled elsewhere; legacy chunk entity list no longer updated
            }
        }
    }
    
    // Mark buildings as generated (v2 progress)
    progress.buildings_generated.insert(coord, true);
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
    mut progress: ResMut<ChunkProgress>,
    tables: Res<ChunkTables>,
    v2_roads: Res<RoadNetwork>,
    mut v2_grid: ResMut<PlacementGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let chunks_to_process: Vec<V2ChunkCoord> = tables
        .loading
        .keys()
        .cloned()
        .filter(|coord| {
            let buildings_done = *progress.buildings_generated.get(coord).unwrap_or(&false);
            let vehicles_done = *progress.vehicles_generated.get(coord).unwrap_or(&false);
            buildings_done && !vehicles_done
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_vehicles_for_chunk(&mut commands, coord, &mut meshes, &mut materials, &mut global_rng, &mut progress, &mut v2_grid, &v2_roads);
    }
}

fn generate_vehicles_for_chunk(
    commands: &mut Commands,
    coord: V2ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    global_rng: &mut GlobalRng,
    progress: &mut ChunkProgress,
    v2_grid: &mut PlacementGrid,
    v2_roads: &RoadNetwork,
) {
    let chunk_center = Vec3::new(
        coord.x as f32 * UNIFIED_CHUNK_SIZE,
        0.0,
        coord.z as f32 * UNIFIED_CHUNK_SIZE,
    );
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Generate vehicles only on roads - REDUCED: From 5 to 2 attempts
    let vehicle_attempts = 2; // Conservative number to prevent overcrowding
    
    for _ in 0..vehicle_attempts {
        let local_x = global_rng.gen_range(-half_size..half_size);
        let local_z = global_rng.gen_range(-half_size..half_size);
        let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);
        
        // Only spawn on roads with sufficient spacing (use v2 road network)
        if v2_roads.is_near_road(position, 6.0) {
            let v2_ok = v2_grid.can_place(
                position,
                crate::world::placement_grid::ContentType::Vehicle,
                4.0,
            );
            if v2_ok {
                let legacy_coord = ChunkCoord { x: coord.x, z: coord.z };
                let _vehicle_entity = spawn_unified_vehicle(
                    commands,
                    legacy_coord,
                    position,
                    meshes,
                    materials,
                );
                
                // Add to placement grid (v2)
                v2_grid.add_entity(
                    position,
                    crate::world::placement_grid::ContentType::Vehicle,
                    4.0,
                );
                
                // V2: entity bookkeeping handled elsewhere; legacy chunk entity list no longer updated
            }
        }
    }
    
    // Mark vehicles as generated (v2 progress)
    progress.vehicles_generated.insert(coord, true);
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
    mut progress: ResMut<ChunkProgress>,
    tables: Res<ChunkTables>,
    v2_roads: Res<RoadNetwork>,
    mut v2_grid: ResMut<PlacementGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let chunks_to_process: Vec<V2ChunkCoord> = tables
        .loading
        .keys()
        .cloned()
        .filter(|coord| {
            let vehicles_done = *progress.vehicles_generated.get(coord).unwrap_or(&false);
            let vegetation_done = *progress.vegetation_generated.get(coord).unwrap_or(&false);
            vehicles_done && !vegetation_done
        })
        .collect();
    
    for coord in chunks_to_process {
        generate_vegetation_for_chunk(&mut commands, coord, &mut meshes, &mut materials, &mut global_rng, &mut progress, &mut v2_grid, &v2_roads);
    }
}

fn generate_vegetation_for_chunk(
    commands: &mut Commands,
    coord: V2ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    global_rng: &mut GlobalRng,
    progress: &mut ChunkProgress,
    v2_grid: &mut PlacementGrid,
    v2_roads: &RoadNetwork,
) {
    let chunk_center = Vec3::new(
        coord.x as f32 * UNIFIED_CHUNK_SIZE,
        0.0,
        coord.z as f32 * UNIFIED_CHUNK_SIZE,
    );
    let half_size = UNIFIED_CHUNK_SIZE * 0.5;
    
    // Generate trees and vegetation in open areas
    let vegetation_attempts = 8;
    
    for _ in 0..vegetation_attempts {
        let local_x = global_rng.gen_range(-half_size..half_size);
        let local_z = global_rng.gen_range(-half_size..half_size);
        let position = Vec3::new(chunk_center.x + local_x, 0.0, chunk_center.z + local_z);
        
        // Only spawn vegetation away from roads and buildings (v2 road network)
        if !v2_roads.is_near_road(position, 6.0) {
            let v2_ok = v2_grid.can_place(
                position,
                crate::world::placement_grid::ContentType::Vegetation,
                2.0,
            );
            if v2_ok {
                let legacy_coord = ChunkCoord { x: coord.x, z: coord.z };
                let _tree_entity = spawn_unified_tree(
                    commands,
                    legacy_coord,
                    position,
                    meshes,
                    materials,
                );
                
                // Add to placement grid (v2)
                v2_grid.add_entity(
                    position,
                    crate::world::placement_grid::ContentType::Vegetation,
                    2.0,
                );
                
                // V2: entity bookkeeping handled elsewhere; legacy chunk entity list no longer updated
            }
        }
    }
    
    // Mark vegetation as generated (v2 progress)
    progress.vegetation_generated.insert(coord, true);
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

#[allow(dead_code)]
fn is_on_road_unified(position: Vec3, road_network: &RoadNetwork) -> bool {
    road_network.is_near_road(position, 25.0)
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

// Road markings removed for v2 simplification


