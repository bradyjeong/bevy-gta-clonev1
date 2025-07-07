//! ───────────────────────────────────────────────
//! System:   Map System
//! Purpose:  Handles user interface display and interaction
//! Schedule: Update
//! Reads:    ActiveEntity, Transform, Player, mut, MapSystem
//! Writes:   Visibility, MapChunk, MapSystem
//! Invariants:
//!   * Distance calculations are cached for performance
//!   * Only active entities can be controlled
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::constants::*;

use crate::systems::world::road_network::{RoadSpline, RoadNetwork};
use crate::systems::world::road_generation::is_on_road_spline;
use std::collections::HashMap;

// Map chunk system for performance
const CHUNK_SIZE: f32 = 200.0; // 200m x 200m chunks

const LOD_LEVELS: usize = 3;

#[derive(Component)]
pub struct MapChunk {
    pub coord: IVec2,
    pub lod_level: usize,
    pub entities: Vec<Entity>,
    pub is_loaded: bool,
    pub distance_to_player: f32,
}

#[derive(Resource)]
pub struct MapSystem {
    pub chunks: HashMap<IVec2, Entity>,
    pub chunk_templates: Vec<ChunkTemplate>,
    pub streaming_radius: f32,
    pub lod_distances: [f32; LOD_LEVELS],
}

#[derive(Clone)]
pub struct ChunkTemplate {
    pub buildings: Vec<BuildingTemplate>,
    pub roads: Vec<RoadSpline>,
    pub vegetation: Vec<VegetationPatch>,
    pub landmarks: Vec<LandmarkTemplate>,
}

#[derive(Clone)]
pub struct BuildingTemplate {
    pub position: Vec3,
    pub scale: Vec3,
    pub building_type: BuildingType,
    pub height: f32,
}

#[derive(Clone)]
pub struct VegetationPatch {
    pub position: Vec3,
    pub radius: f32,
    pub density: f32,
    pub vegetation_type: VegetationType,
}

#[derive(Clone)]
pub struct LandmarkTemplate {
    pub position: Vec3,
    pub landmark_type: LandmarkType,
    pub scale: f32,
}

#[derive(Clone)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industrial,
    Skyscraper,
    Apartment,
}

#[derive(Clone)]
pub enum VegetationType {
    Trees,
    Grass,
    Bushes,
    Park,
}

#[derive(Clone)]
pub enum LandmarkType {
    Fountain,
    Statue,
    Monument,
    Bridge,
    Stadium,
}

impl Default for MapSystem {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_templates: Vec::new(),
            streaming_radius: 800.0, // Stream chunks within 800m
            lod_distances: [150.0, 300.0, 500.0], // LOD transition distances - optimized
        }
    }
}

// Map streaming system - loads/unloads chunks based on player position
pub fn map_streaming_system(
    mut commands: Commands,
    mut map_system: ResMut<MapSystem>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut chunk_query: Query<(Entity, &mut MapChunk)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    road_network: Res<RoadNetwork>,
) {
    let Ok(active_transform) = active_query.single() else { return; };
    let player_pos = active_transform.translation;
    
    // Calculate which chunks should be loaded
    let player_chunk = world_to_chunk_coord(player_pos);
    let streaming_radius_chunks = (map_system.streaming_radius / CHUNK_SIZE) as i32;
    
    // Update existing chunks
    for (entity, mut chunk) in chunk_query.iter_mut() {
        chunk.distance_to_player = player_pos.distance(chunk_coord_to_world(chunk.coord));
        
        // Unload distant chunks
        if chunk.distance_to_player > map_system.streaming_radius {
            if chunk.is_loaded {
                unload_chunk(&mut commands, entity, &mut chunk);
            }
        }
    }
    
    // Load new chunks in streaming radius
    for x in (player_chunk.x - streaming_radius_chunks)..=(player_chunk.x + streaming_radius_chunks) {
        for z in (player_chunk.y - streaming_radius_chunks)..=(player_chunk.y + streaming_radius_chunks) {
            let coord = IVec2::new(x, z);
            
            if !map_system.chunks.contains_key(&coord) {
                let distance = player_pos.distance(chunk_coord_to_world(coord));
                if distance <= map_system.streaming_radius {
                    let chunk_entity = spawn_chunk(
                        &mut commands,
                        coord,
                        &mut meshes,
                        &mut materials,
                        &map_system,
                        &road_network,
                        distance,
                    );
                    map_system.chunks.insert(coord, chunk_entity);
                }
            }
        }
    }
}

// LOD system - adjusts detail level based on distance
pub fn map_lod_system(
    mut chunk_query: Query<&mut MapChunk>,
    player_query: Query<&Transform, With<Player>>,
    map_system: Res<MapSystem>,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let player_pos = player_transform.translation;
    
    for mut chunk in chunk_query.iter_mut() {
        if !chunk.is_loaded { continue; }
        
        let distance = player_pos.distance(chunk_coord_to_world(chunk.coord));
        let new_lod = calculate_lod_level(distance, &map_system.lod_distances);
        
        if new_lod != chunk.lod_level {
            chunk.lod_level = new_lod;
            update_chunk_lod(&mut visibility_query, &chunk);
        }
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    coord: IVec2,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_system: &MapSystem,
    road_network: &RoadNetwork,
    distance: f32,
) -> Entity {
    let world_pos = chunk_coord_to_world(coord);
    let template = generate_chunk_template(coord, road_network);
    let lod_level = calculate_lod_level(distance, &map_system.lod_distances);
    
    let chunk_entity = commands.spawn((
        Transform::from_translation(world_pos),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
    )).id();
    
    let mut entities = Vec::new();
    
    // Spawn buildings with LOD
    for building in &template.buildings {
        let building_entity = spawn_building(commands, meshes, materials, building, lod_level);
        commands.entity(building_entity).insert(ChildOf(chunk_entity));
        entities.push(building_entity);
    }
    
    // Spawn vegetation with density based on LOD
    for vegetation in &template.vegetation {
        if should_spawn_at_lod(lod_level, &EntityType::Vegetation) {
            let veg_entity = spawn_vegetation(commands, meshes, materials, vegetation, lod_level);
            commands.entity(veg_entity).insert(ChildOf(chunk_entity));
            entities.push(veg_entity);
        }
    }
    
    // Spawn landmarks (always visible at close range)
    for landmark in &template.landmarks {
        if lod_level <= 1 {
            let landmark_entity = spawn_landmark(commands, meshes, materials, landmark);
            commands.entity(landmark_entity).insert(ChildOf(chunk_entity));
            entities.push(landmark_entity);
        }
    }
    
    commands.entity(chunk_entity).insert(MapChunk {
        coord,
        lod_level,
        entities,
        is_loaded: true,
        distance_to_player: distance,
    });
    
    chunk_entity
}

fn spawn_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building: &BuildingTemplate,
    _lod_level: usize,
) -> Entity {
    // REPLACED: Use UnifiedEntityFactory for building spawning
    // This eliminates duplicate building spawning code
    use crate::factories::entity_factory_unified::UnifiedEntityFactory;
    use crate::config::GameConfig;
    
    let mut factory = UnifiedEntityFactory::with_config(GameConfig::default());
    let current_time = 0.0; // Placeholder time
    
    match factory.spawn_building_consolidated(commands, meshes, materials, building.position, current_time) {
        Ok(entity) => {
            // Add map-specific components to maintain compatibility
            commands.entity(entity).insert(Building);
            entity
        }
        Err(_) => {
            // Fallback to empty entity if spawn fails
            commands.spawn((
                Transform::from_translation(building.position),
                Visibility::Hidden,
                Building,
            )).id()
        }
    }
}

fn spawn_vegetation(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    vegetation: &VegetationPatch,
    lod_level: usize,
) -> Entity {
    let instances = match lod_level {
        0 => (vegetation.density * 20.0) as usize,
        1 => (vegetation.density * 10.0) as usize,
        _ => (vegetation.density * 5.0) as usize,
    };
    
    let parent = commands.spawn((
        Transform::from_translation(vegetation.position),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
    )).id();
    
    for _i in 0..instances {
        let offset = Vec3::new(
            (rand::random::<f32>() - 0.5) * vegetation.radius,
            0.0,
            (rand::random::<f32>() - 0.5) * vegetation.radius,
        );
        
        let (mesh, color) = match vegetation.vegetation_type {
            VegetationType::Trees => (
                meshes.add(Cylinder::new(0.3, 3.0)),
                Color::srgb(0.2, 0.8, 0.2)
            ),
            VegetationType::Bushes => (
                meshes.add(Sphere::new(0.5)),
                Color::srgb(0.3, 0.6, 0.2)
            ),
            _ => (
                meshes.add(Cuboid::new(0.2, 0.5, 0.2)),
                Color::srgb(0.4, 0.8, 0.3)
            ),
        };
        
        let veg_entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(color)),
            Transform::from_translation(offset),
            Cullable {
                max_distance: 200.0,
                is_culled: false,
            },
        )).id();
        
        commands.entity(veg_entity).insert(ChildOf(parent));
    }
    
    parent
}

fn spawn_landmark(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    landmark: &LandmarkTemplate,
) -> Entity {
    let (mesh, color) = match landmark.landmark_type {
        LandmarkType::Fountain => (
            meshes.add(Cylinder::new(2.0, 1.0)),
            Color::srgb(0.9, 0.9, 1.0)
        ),
        LandmarkType::Statue => (
            meshes.add(Cylinder::new(0.5, 3.0)),
            Color::srgb(0.6, 0.6, 0.5)
        ),
        LandmarkType::Monument => (
            meshes.add(Cuboid::new(3.0, 5.0, 3.0)),
            Color::srgb(0.7, 0.6, 0.5)
        ),
        LandmarkType::Stadium => (
            meshes.add(Cylinder::new(50.0, 20.0)),
            Color::srgb(0.8, 0.8, 0.9)
        ),
        _ => (
            meshes.add(Sphere::new(1.0)),
            Color::srgb(0.5, 0.5, 0.5)
        ),
    };
    
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(color)),
        Transform::from_translation(landmark.position).with_scale(Vec3::splat(landmark.scale)),
        RigidBody::Fixed,
        Collider::cuboid(2.0, 2.0, 2.0),
        CollisionGroups::new(STATIC_GROUP, Group::ALL),
        Cullable {
            max_distance: 1000.0,
            is_culled: false,
        },
        Landmark,
    )).id()
}



fn generate_chunk_template(coord: IVec2, road_network: &RoadNetwork) -> ChunkTemplate {
    use rand::Rng;
    use std::cell::RefCell;
    
    thread_local! {
        static MAP_RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
    }
    
    let mut buildings = Vec::new();
    let mut vegetation = Vec::new();
    let mut landmarks = Vec::new();
    
    // Generate buildings based on distance from center (density decreases)
    let distance_from_center = coord.as_vec2().length();
    let building_density = (1.0 - (distance_from_center / 20.0).min(0.8)).max(0.1);
    
    for _ in 0..(building_density * 30.0) as usize {
        let pos = Vec3::new(
            MAP_RNG.with(|rng| rng.borrow_mut().gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0)),
            0.0,
            MAP_RNG.with(|rng| rng.borrow_mut().gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0)),
        );
        
        // Convert to world position for road checking
        let world_pos = Vec3::new(
            coord.x as f32 * CHUNK_SIZE + pos.x,
            pos.y,
            coord.y as f32 * CHUNK_SIZE + pos.z,
        );
        
        // Skip if too close to roads using proper road detection
        if is_on_road_spline(world_pos, road_network, 25.0) {
            println!("DEBUG: MAP_SYSTEM - Skipping building at {:?} - on road", world_pos);
            continue;
        }
        println!("DEBUG: MAP_SYSTEM - Spawning building at {:?}", world_pos);
        
        let building_type = if distance_from_center < 5.0 {
            if MAP_RNG.with(|rng| rng.borrow_mut().gen_bool(0.3)) { BuildingType::Skyscraper } else { BuildingType::Commercial }
        } else if distance_from_center < 10.0 {
            if MAP_RNG.with(|rng| rng.borrow_mut().gen_bool(0.6)) { BuildingType::Residential } else { BuildingType::Commercial }
        } else {
            BuildingType::Residential
        };
        
        let (scale, height) = match building_type {
            BuildingType::Skyscraper => (Vec3::new(8.0, 15.0, 8.0), MAP_RNG.with(|rng| rng.borrow_mut().gen_range(40.0..80.0))),
            BuildingType::Commercial => (Vec3::new(12.0, 8.0, 6.0), MAP_RNG.with(|rng| rng.borrow_mut().gen_range(8.0..20.0))),
            BuildingType::Residential => (Vec3::new(6.0, 6.0, 8.0), MAP_RNG.with(|rng| rng.borrow_mut().gen_range(6.0..15.0))),
            _ => (Vec3::new(4.0, 4.0, 4.0), MAP_RNG.with(|rng| rng.borrow_mut().gen_range(3.0..8.0))),
        };
        
        buildings.push(BuildingTemplate {
            position: pos,
            scale,
            building_type,
            height,
        });
    }
    
    // Generate vegetation patches
    for _ in 0..MAP_RNG.with(|rng| rng.borrow_mut().gen_range(3..8)) {
        vegetation.push(VegetationPatch {
            position: Vec3::new(
                MAP_RNG.with(|rng| rng.borrow_mut().gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0)),
                0.0,
                MAP_RNG.with(|rng| rng.borrow_mut().gen_range(-CHUNK_SIZE/2.0..CHUNK_SIZE/2.0)),
            ),
            radius: MAP_RNG.with(|rng| rng.borrow_mut().gen_range(10.0..30.0)),
            density: MAP_RNG.with(|rng| rng.borrow_mut().gen_range(0.3..0.8)),
            vegetation_type: if MAP_RNG.with(|rng| rng.borrow_mut().gen_bool(0.6)) { VegetationType::Trees } else { VegetationType::Bushes },
        });
    }
    
    // Occasional landmarks
    if MAP_RNG.with(|rng| rng.borrow_mut().gen_bool(0.1)) {
        landmarks.push(LandmarkTemplate {
            position: Vec3::new(
                MAP_RNG.with(|rng| rng.borrow_mut().gen_range(-CHUNK_SIZE/4.0..CHUNK_SIZE/4.0)),
                0.0,
                MAP_RNG.with(|rng| rng.borrow_mut().gen_range(-CHUNK_SIZE/4.0..CHUNK_SIZE/4.0)),
            ),
            landmark_type: match MAP_RNG.with(|rng| rng.borrow_mut().gen_range(0..4)) {
                0 => LandmarkType::Fountain,
                1 => LandmarkType::Statue,
                2 => LandmarkType::Monument,
                _ => LandmarkType::Stadium,
            },
            scale: MAP_RNG.with(|rng| rng.borrow_mut().gen_range(0.5..2.0)),
        });
    }
    
    ChunkTemplate {
        buildings,
        roads: Vec::new(), // Roads handled by existing road system
        vegetation,
        landmarks,
    }
}

fn world_to_chunk_coord(world_pos: Vec3) -> IVec2 {
    IVec2::new(
        (world_pos.x / CHUNK_SIZE).floor() as i32,
        (world_pos.z / CHUNK_SIZE).floor() as i32,
    )
}

fn chunk_coord_to_world(coord: IVec2) -> Vec3 {
    Vec3::new(
        coord.x as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
        0.0,
        coord.y as f32 * CHUNK_SIZE + CHUNK_SIZE / 2.0,
    )
}

fn calculate_lod_level(distance: f32, lod_distances: &[f32; LOD_LEVELS]) -> usize {
    for (i, &max_distance) in lod_distances.iter().enumerate() {
        if distance <= max_distance {
            return i;
        }
    }
    LOD_LEVELS - 1
}

fn should_spawn_at_lod(lod_level: usize, entity_type: &EntityType) -> bool {
    match entity_type {
        EntityType::Vegetation => lod_level <= 1,
        EntityType::Detail => lod_level == 0,
        EntityType::Building => true,
        EntityType::Landmark => lod_level <= 2,
    }
}

fn update_chunk_lod(visibility_query: &mut Query<&mut Visibility>, chunk: &MapChunk) {
    // Update visibility of entities based on LOD level
    for &entity in &chunk.entities {
        if let Ok(mut visibility) = visibility_query.get_mut(entity) {
            *visibility = if chunk.lod_level <= 2 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn unload_chunk(commands: &mut Commands, chunk_entity: Entity, chunk: &mut MapChunk) {
    chunk.is_loaded = false;
    // The chunk entity and its children will be automatically cleaned up by Bevy's hierarchy system
    commands.entity(chunk_entity).despawn();
}

#[allow(dead_code)]
enum EntityType {
    Building,
    Vegetation,
    Detail,
    Landmark,
}

// Components for map entities
#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Landmark;
