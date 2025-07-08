//! ───────────────────────────────────────────────
//! System:   Map System
//! Purpose:  Generates and manages map terrain and landmarks
//! Schedule: Initialization and Update
//! Reads:    `ActiveEntity`, Transform, Chunk
//! Writes:   Commands, Mesh, Material
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
// Removed bevy16_compat - using direct Bevy methods
use bevy::render::mesh::Mesh;
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;
use game_core::bundles::VisibleBundle;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use game_core::prelude::*;

const CHUNK_SIZE: f32 = 200.0;
const TERRAIN_HEIGHT_SCALE: f32 = 10.0;

#[derive(Resource, Default)]
pub struct MapSystem {
    pub generated_chunks: HashMap<(i32, i32), Entity>,
    pub chunk_templates: Vec<ChunkTemplate>,
}

#[derive(Clone)]
pub struct ChunkTemplate {
    pub terrain_height: f32,
    pub buildings: Vec<BuildingTemplate>,
    pub vegetation: Vec<VegetationPatch>,
}

#[derive(Clone)]
pub struct BuildingTemplate {
    pub position: Vec3,
    pub building_type: BuildingType,
    pub rotation: f32,
}

#[derive(Clone)]
pub struct VegetationPatch {
    pub area: Vec2,
    pub vegetation_type: VegetationType,
    pub density: f32,
}

#[derive(Clone)]
pub struct LandmarkTemplate {
    pub position: Vec3,
    pub landmark_type: LandmarkType,
}

#[derive(Clone, PartialEq)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industrial,
    Skyscraper,
}

#[derive(Clone, PartialEq)]
pub enum VegetationType {
    Trees,
    Bushes,
    Grass,
}

#[derive(Clone, PartialEq)]
pub enum LandmarkType {
    Park,
    Monument,
    Bridge,
}

impl MapSystem {
    #[must_use] pub fn new() -> Self {
        Self {
            generated_chunks: HashMap::new(),
            chunk_templates: Vec::new(),
        }
    }
    
    #[must_use] pub fn generate_chunk_template(&self, chunk_x: i32, chunk_z: i32) -> ChunkTemplate {
        let mut rng = StdRng::seed_from_u64(
            ((chunk_x as u64) << 32) | ((chunk_z as u64) & 0xFFFFFFFF)
        );
        
        let terrain_height = rng.gen_range(-2.0..5.0);
        
        let mut buildings = Vec::new();
        let building_count = rng.gen_range(0..8);
        
        for _ in 0..building_count {
            buildings.push(BuildingTemplate {
                position: Vec3::new(
                    rng.gen_range(-CHUNK_SIZE * 0.4..CHUNK_SIZE * 0.4),
                    terrain_height,
                    rng.gen_range(-CHUNK_SIZE * 0.4..CHUNK_SIZE * 0.4),
                ),
                building_type: match rng.gen_range(0..4) {
                    0 => BuildingType::Residential,
                    1 => BuildingType::Commercial,
                    2 => BuildingType::Industrial,
                    _ => BuildingType::Skyscraper,
                },
                rotation: rng.gen_range(0.0..std::f32::consts::TAU),
            });
        }
        
        let mut vegetation = Vec::new();
        let veg_patches = rng.gen_range(1..4);
        
        for _ in 0..veg_patches {
            vegetation.push(VegetationPatch {
                area: Vec2::new(
                    rng.gen_range(10.0..50.0),
                    rng.gen_range(10.0..50.0),
                ),
                vegetation_type: match rng.gen_range(0..3) {
                    0 => VegetationType::Trees,
                    1 => VegetationType::Bushes,
                    _ => VegetationType::Grass,
                },
                density: rng.gen_range(0.1..0.8),
            });
        }
        
        ChunkTemplate {
            terrain_height,
            buildings,
            vegetation,
        }
    }
}

pub fn map_generation_system(
    mut commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut map_system: ResMut<MapSystem>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(active_transform) = active_query.single() {
        let active_pos = active_transform.translation;
        
        // Calculate current chunk
        let current_chunk = (
            (active_pos.x / CHUNK_SIZE).floor() as i32,
            (active_pos.z / CHUNK_SIZE).floor() as i32,
        );
        
        let generation_radius = 2;
        
        // Generate chunks around player
        for dx in -generation_radius..=generation_radius {
            for dz in -generation_radius..=generation_radius {
                let chunk_x = current_chunk.0 + dx;
                let chunk_z = current_chunk.1 + dz;
                
                let chunk_key = (chunk_x, chunk_z);
                
                if !map_system.generated_chunks.contains_key(&chunk_key) {
                    let chunk_entity = generate_chunk(
                        &mut commands,
                        chunk_x,
                        chunk_z,
                        &map_system,
                        &mut meshes,
                        &mut materials,
                    );
                    
                    map_system.generated_chunks.insert(chunk_key, chunk_entity);
                }
            }
        }
        
        // Clean up distant chunks
        let cleanup_radius = 5;
        let mut chunks_to_remove = Vec::new();
        
        for (chunk_key, chunk_entity) in &map_system.generated_chunks {
            let distance = (chunk_key.0 - current_chunk.0).abs() + (chunk_key.1 - current_chunk.1).abs();
            
            if distance > cleanup_radius {
                commands.entity(*chunk_entity).despawn_recursive();
                chunks_to_remove.push(*chunk_key);
            }
        }
        
        for chunk_key in chunks_to_remove {
            map_system.generated_chunks.remove(&chunk_key);
        }
    }
}

fn generate_chunk(
    commands: &mut Commands,
    chunk_x: i32,
    chunk_z: i32,
    map_system: &MapSystem,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let template = map_system.generate_chunk_template(chunk_x, chunk_z);
    
    let chunk_center = Vec3::new(
        chunk_x as f32 * CHUNK_SIZE,
        template.terrain_height,
        chunk_z as f32 * CHUNK_SIZE,
    );
    
    // Create chunk parent entity
    let chunk_entity = commands.spawn((
        Transform::from_translation(chunk_center),
        VisibleBundle::default(),
        Chunk {
            x: chunk_x,
            z: chunk_z,
        },
    )).id();
    
    // Generate terrain
    let terrain_entity = generate_terrain(commands, chunk_center, meshes, materials);
    commands.entity(chunk_entity).add_child(terrain_entity);
    
    // Generate buildings
    for building in &template.buildings {
        let building_entity = generate_building(
            commands,
            chunk_center + building.position,
            &building.building_type,
            building.rotation,
            meshes,
            materials,
        );
        commands.entity(chunk_entity).add_child(building_entity);
    }
    
    // Generate vegetation
    for vegetation in &template.vegetation {
        let veg_entities = generate_vegetation_patch(
            commands,
            chunk_center,
            vegetation,
            meshes,
            materials,
        );
        
        for veg_entity in veg_entities {
            commands.entity(chunk_entity).add_child(veg_entity);
        }
    }
    
    chunk_entity
}

fn generate_terrain(
    commands: &mut Commands,
    chunk_center: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let terrain_mesh = create_terrain_mesh(chunk_center);
    let terrain_material = StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.3),
        ..default()
    };
    
    commands.spawn((
        Mesh3d(meshes.add(terrain_mesh)),
        MeshMaterial3d(materials.add(terrain_material)),
        Transform::from_translation(Vec3::ZERO),
        TerrainChunk,
    )).id()
}

fn create_terrain_mesh(chunk_center: Vec3) -> Mesh {
    let size = CHUNK_SIZE;
    let subdivisions = 32;
    let step = size / subdivisions as f32;
    
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Generate vertices
    for i in 0..=subdivisions {
        for j in 0..=subdivisions {
            let x = -size * 0.5 + i as f32 * step;
            let z = -size * 0.5 + j as f32 * step;
            let y = generate_height_at_position(chunk_center.x + x, chunk_center.z + z);
            
            vertices.push([x, y, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([i as f32 / subdivisions as f32, j as f32 / subdivisions as f32]);
        }
    }
    
    // Generate indices
    for i in 0..subdivisions {
        for j in 0..subdivisions {
            let base = i * (subdivisions + 1) + j;
            
            // First triangle
            indices.push(base as u32);
            indices.push((base + subdivisions + 1) as u32);
            indices.push((base + 1) as u32);
            
            // Second triangle
            indices.push((base + 1) as u32);
            indices.push((base + subdivisions + 1) as u32);
            indices.push((base + subdivisions + 2) as u32);
        }
    }
    
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    mesh
}

fn generate_height_at_position(x: f32, z: f32) -> f32 {
    // Simple noise-based height generation
    let noise = (x * 0.01).sin() * (z * 0.01).cos() * TERRAIN_HEIGHT_SCALE;
    noise.max(-5.0).min(15.0)
}

fn generate_building(
    commands: &mut Commands,
    position: Vec3,
    building_type: &BuildingType,
    rotation: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let (mesh, color, height) = match building_type {
        BuildingType::Residential => (
            meshes.add(Cuboid::new(8.0, 6.0, 8.0)),
            Color::srgb(0.8, 0.7, 0.6),
            6.0,
        ),
        BuildingType::Commercial => (
            meshes.add(Cuboid::new(12.0, 8.0, 10.0)),
            Color::srgb(0.6, 0.6, 0.8),
            8.0,
        ),
        BuildingType::Industrial => (
            meshes.add(Cuboid::new(16.0, 10.0, 12.0)),
            Color::srgb(0.5, 0.5, 0.5),
            10.0,
        ),
        BuildingType::Skyscraper => (
            meshes.add(Cuboid::new(10.0, 30.0, 10.0)),
            Color::srgb(0.4, 0.4, 0.6),
            30.0,
        ),
    };
    
    let mut transform = Transform::from_translation(position + Vec3::new(0.0, height * 0.5, 0.0));
    transform.rotate_y(rotation);
    
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })),
        transform,
        Building,
        Cullable {
            max_distance: 500.0,
            is_culled: false,
        },
    )).id()
}

fn generate_vegetation_patch(
    commands: &mut Commands,
    chunk_center: Vec3,
    vegetation: &VegetationPatch,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut rng = StdRng::seed_from_u64(
        ((chunk_center.x as u64) << 32) | ((chunk_center.z as u64) & 0xFFFFFFFF)
    );
    
    let items_count = (vegetation.area.x * vegetation.area.y * vegetation.density * 0.1) as usize;
    
    for _ in 0..items_count {
        let offset = Vec3::new(
            rng.gen_range(-vegetation.area.x * 0.5..vegetation.area.x * 0.5),
            0.0,
            rng.gen_range(-vegetation.area.y * 0.5..vegetation.area.y * 0.5),
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
            VegetationType::Grass => (
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
        
        entities.push(veg_entity);
    }
    
    entities
}

// Component markers
#[derive(Component)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct TerrainChunk;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Landmark;
