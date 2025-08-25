use crate::resources::{MaterialRegistry, WorldRng};
use crate::systems::world::generators::{
    BuildingGenerator, RoadGenerator, VegetationGenerator, VehicleGenerator,
};
use crate::systems::world::unified_world::{ChunkCoord, ChunkState, UnifiedWorldManager};
use bevy::prelude::*;

/// Simplified unified chunk generation system following AGENT.md simplicity principles
/// Replaces the complex 900-line layered state machine with focused generators
pub fn layered_generation_coordinator(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut world_rng: ResMut<WorldRng>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Find chunks that need loading
    let mut chunks_to_load = Vec::new();
    for chunk_opt in &world_manager.chunks {
        if let Some(chunk) = chunk_opt {
            if matches!(chunk.state, ChunkState::Loading) && current_time - chunk.last_update > 0.1
            {
                chunks_to_load.push(chunk.coord);
            }
        }
    }

    // Load chunks completely (up to budget)
    let mut loaded_this_frame = 0;
    for coord in chunks_to_load {
        if loaded_this_frame >= world_manager.max_chunks_per_frame {
            break;
        }

        // Generate all content layers using focused generators - simple sequential approach
        generate_complete_chunk(
            &mut commands,
            &mut world_manager,
            coord,
            &mut meshes,
            &mut materials,
            &mut material_registry,
            &mut world_rng,
            current_time,
        );
        loaded_this_frame += 1;
    }
}

/// Simple coordinator that uses focused generators instead of complex state machine
/// Each generator has single responsibility and can work independently
fn generate_complete_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    material_registry: &mut MaterialRegistry,
    world_rng: &mut WorldRng,
    current_time: f32,
) {
    // Use focused generators following single responsibility principle
    let road_generator = RoadGenerator;
    let building_generator = BuildingGenerator;
    let vehicle_generator = VehicleGenerator;
    let vegetation_generator = VegetationGenerator;

    // Generate all content layers in sequence
    road_generator.generate_roads(
        commands,
        world_manager,
        coord,
        meshes,
        materials,
        material_registry,
        world_rng,
    );
    building_generator.generate_buildings(
        commands,
        world_manager,
        coord,
        meshes,
        materials,
        world_rng,
    );
    vehicle_generator.generate_vehicles(
        commands,
        world_manager,
        coord,
        meshes,
        materials,
        world_rng,
    );
    vegetation_generator.generate_vegetation(
        commands,
        world_manager,
        coord,
        meshes,
        materials,
        world_rng,
    );

    // Mark chunk as complete
    if let Some(chunk) = world_manager.get_chunk_mut(coord) {
        chunk.state = ChunkState::Loaded { lod_level: 0 };
        chunk.last_update = current_time;
    }

    info!(
        "Generated complete chunk at {:?} using focused generators",
        coord
    );
}

/// Legacy systems for compatibility - can be removed once migration is complete
pub fn road_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut world_rng: ResMut<WorldRng>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|chunk_opt| {
            if let Some(chunk) = chunk_opt {
                if matches!(chunk.state, ChunkState::Loading) && !chunk.roads_generated {
                    Some(chunk.coord)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let road_generator = RoadGenerator;
    for coord in chunks_to_process {
        road_generator.generate_roads(
            &mut commands,
            &mut world_manager,
            coord,
            &mut meshes,
            &mut materials,
            &mut material_registry,
            &mut world_rng,
        );
    }
}

pub fn building_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_rng: ResMut<WorldRng>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|chunk_opt| {
            if let Some(chunk) = chunk_opt {
                if matches!(chunk.state, ChunkState::Loading)
                    && chunk.roads_generated
                    && !chunk.buildings_generated
                {
                    Some(chunk.coord)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let building_generator = BuildingGenerator;
    for coord in chunks_to_process {
        building_generator.generate_buildings(
            &mut commands,
            &mut world_manager,
            coord,
            &mut meshes,
            &mut materials,
            &mut world_rng,
        );
    }
}

pub fn vehicle_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_rng: ResMut<WorldRng>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|chunk_opt| {
            if let Some(chunk) = chunk_opt {
                if matches!(chunk.state, ChunkState::Loading)
                    && chunk.buildings_generated
                    && !chunk.vehicles_generated
                {
                    Some(chunk.coord)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let vehicle_generator = VehicleGenerator;
    for coord in chunks_to_process {
        vehicle_generator.generate_vehicles(
            &mut commands,
            &mut world_manager,
            coord,
            &mut meshes,
            &mut materials,
            &mut world_rng,
        );
    }
}

pub fn vegetation_layer_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_rng: ResMut<WorldRng>,
) {
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .filter_map(|chunk_opt| {
            if let Some(chunk) = chunk_opt {
                if matches!(chunk.state, ChunkState::Loading)
                    && chunk.vehicles_generated
                    && !chunk.vegetation_generated
                {
                    Some(chunk.coord)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let vegetation_generator = VegetationGenerator;
    for coord in chunks_to_process {
        vegetation_generator.generate_vegetation(
            &mut commands,
            &mut world_manager,
            coord,
            &mut meshes,
            &mut materials,
            &mut world_rng,
        );
    }
}
