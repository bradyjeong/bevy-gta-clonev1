//! ───────────────────────────────────────────────
//! System:   Layered Generation
//! Purpose:  Generates world content in layers (roads, buildings, vegetation)
//! Schedule: Update
//! Reads:    `ActiveEntity`, Transform
//! Writes:   Commands, `WorldManager`
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use std::collections::HashSet;
use game_core::prelude::*;
use crate::systems::world::unified_world::{WorldManager, ChunkCoord, UNIFIED_CHUNK_SIZE};
use game_core::bundles::VisibleBundle;

// Re-export canonical types from game_core
pub use game_core::world::ContentLayer;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ChunkState {
    Empty,
    Loading,
    Loaded,
}

#[derive(Clone)]
pub struct LayeredChunkData {
    pub state: ChunkState,
    pub completed_layers: HashSet<ContentLayer>,
    pub roads_generated: bool,
    pub buildings_generated: bool,
    pub vegetation_generated: bool,
}

impl Default for LayeredChunkData {
    fn default() -> Self {
        Self {
            state: ChunkState::Empty,
            completed_layers: HashSet::new(),
            roads_generated: false,
            buildings_generated: false,
            vegetation_generated: false,
        }
    }
}

pub fn layered_generation_system(
    _commands: Commands,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut world_manager: ResMut<WorldManager>,
    time: Res<Time>,
) {
    if let Ok(active_transform) = active_query.single() {
        let player_pos = active_transform.translation;
        let current_time = time.elapsed_secs();
        
        // Update active chunks
        world_manager.update_active_chunks(player_pos, 800.0);
        
        // Process chunks that need generation
        for &coord in &world_manager.active_chunks.clone() {
            advance_chunk_generation(&mut world_manager, coord, current_time);
        }
    }
}

fn advance_chunk_generation(
    world_manager: &mut WorldManager,
    coord: ChunkCoord,
    current_time: f32,
) {
    let chunk = world_manager.get_or_create_chunk(coord);
    
    // Update chunk timestamp
    chunk.last_updated = current_time;
    
    // Mark chunk as loaded if not already
    if !world_manager.is_chunk_loaded(coord) {
        world_manager.mark_chunk_loaded(coord);
    }
}

pub fn road_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<WorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Process chunks that need road generation
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .active_chunks
        .clone()
        .into_iter()
        .filter(|&coord| {
            world_manager.is_chunk_loaded(coord) && 
            !has_roads_generated(coord)
        })
        .collect();
        
    for coord in chunks_to_process {
        generate_roads_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn has_roads_generated(__coord: ChunkCoord) -> bool {
    // For now, assume no roads are generated
    // In a real implementation, this would check the chunk state
    false
}

fn generate_roads_for_chunk(
    commands: &mut Commands,
    world_manager: &mut WorldManager,
    coord: ChunkCoord,
    __meshes: &mut ResMut<Assets<Mesh>>,
    __materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos(UNIFIED_CHUNK_SIZE);
    
    // Simple road generation - just create a basic road entity
    let road_entity = commands.spawn((
        Transform::from_translation(chunk_center),
        VisibleBundle::default(),
        RoadEntity { road_id: coord.x as u32 * 1000 + coord.z as u32 },
        DynamicContent {
            content_type: ContentType::Road,
        },
    )).id();
    
    // Add to chunk
    world_manager.add_entity_to_chunk(road_entity, chunk_center);
}

pub fn building_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<WorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Process chunks that need building generation
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .active_chunks
        .clone()
        .into_iter()
        .filter(|&coord| {
            world_manager.is_chunk_loaded(coord) && 
            !has_buildings_generated(coord)
        })
        .collect();
        
    for coord in chunks_to_process {
        generate_buildings_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn has_buildings_generated(__coord: ChunkCoord) -> bool {
    // For now, assume no buildings are generated
    false
}

fn generate_buildings_for_chunk(
    commands: &mut Commands,
    world_manager: &mut WorldManager,
    coord: ChunkCoord,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos(UNIFIED_CHUNK_SIZE);
    
    // Simple building generation
    let building_entity = commands.spawn((
        Transform::from_translation(chunk_center + Vec3::new(10.0, 0.0, 10.0)),
        VisibleBundle::default(),
        DynamicContent {
            content_type: ContentType::Building,
        },
        Cullable {
            max_distance: 500.0,
            is_culled: false,
        },
    )).id();
    
    // Add to chunk
    world_manager.add_entity_to_chunk(building_entity, chunk_center);
}

pub fn vegetation_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<WorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Process chunks that need vegetation generation
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .active_chunks
        .clone()
        .into_iter()
        .filter(|&coord| {
            world_manager.is_chunk_loaded(coord) && 
            !has_vegetation_generated(coord)
        })
        .collect();
        
    for coord in chunks_to_process {
        generate_vegetation_for_chunk(&mut commands, &mut world_manager, coord, &mut meshes, &mut materials);
    }
}

fn has_vegetation_generated(_coord: ChunkCoord) -> bool {
    // For now, assume no vegetation is generated
    false
}

fn generate_vegetation_for_chunk(
    commands: &mut Commands,
    world_manager: &mut WorldManager,
    coord: ChunkCoord,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let chunk_center = coord.to_world_pos(UNIFIED_CHUNK_SIZE);
    
    // Simple vegetation generation
    for i in 0..5 {
        let offset = Vec3::new(
            (i as f32 - 2.0) * 20.0,
            0.0,
            (i as f32 - 2.0) * 20.0,
        );
        
        let tree_entity = commands.spawn((
            Transform::from_translation(chunk_center + offset),
            VisibleBundle::default(),
            DynamicContent {
                content_type: ContentType::Tree,
            },
            Cullable {
                max_distance: 200.0,
                is_culled: false,
            },
        )).id();
        
        // Add to chunk
        world_manager.add_entity_to_chunk(tree_entity, chunk_center + offset);
    }
}

// Utility functions
#[must_use] pub fn get_next_layer_to_generate(completed_layers: &HashSet<ContentLayer>) -> Option<ContentLayer> {
    let all_layers = [
        ContentLayer::Roads,
        ContentLayer::Buildings,
        ContentLayer::Vegetation,
        ContentLayer::Landmarks,
    ];
    
    for layer in &all_layers {
        if !completed_layers.contains(layer) {
            return Some(*layer);
        }
    }
    
    None
}

#[must_use] pub fn is_chunk_fully_generated(completed_layers: &HashSet<ContentLayer>) -> bool {
    completed_layers.len() >= 4 // All layers completed
}
