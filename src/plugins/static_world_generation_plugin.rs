use bevy::prelude::*;

use crate::components::unified_water::UnifiedWaterBody;
use crate::config::GameConfig;
use crate::constants::WorldEnvConfig;
use crate::resources::{MaterialRegistry, WorldRng};
use crate::states::AppState;

use crate::systems::spawn_validation::SpawnRegistry;
// use crate::systems::ui::loading_screen::{
//     cleanup_loading_screen, setup_loading_screen, update_loading_progress,
// };
use crate::systems::world::generators::{
    BuildingGenerator, RoadGenerator, VegetationGenerator, VehicleGenerator,
};
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::unified_world::{ChunkCoord, ChunkState, UnifiedWorldManager};

/// Static world generation plugin - generates all chunks at startup
/// Uses Loading state to parallelize generation without blocking window
pub struct StaticWorldGenerationPlugin;

impl Plugin for StaticWorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Note: SpawnRegistry is already initialized by SpawnValidationPlugin
            // World generation screen UI (camera stays active for UI rendering)
            // .add_systems(OnEnter(AppState::WorldGeneration), setup_loading_screen)
            .add_systems(
                OnExit(AppState::WorldGeneration),
                cleanup_generation_resources,
            )
            .add_systems(OnExit(AppState::AssetLoading), cleanup_generation_resources)
            // World generation systems
            .add_systems(
                OnEnter(AppState::WorldGeneration),
                queue_all_chunks_for_generation,
            )
            .add_systems(
                Update,
                apply_generated_chunks.run_if(in_state(AppState::WorldGeneration)),
            );
    }
}

/// Temporary resource to track static generation progress
#[derive(Resource)]
pub struct StaticGenerationQueue {
    /// Total chunks to generate
    pub total_chunks: usize,
    /// Chunks completed
    pub completed_chunks: usize,
    /// Start time for progress reporting
    pub start_time: std::time::Instant,
}

/// Queue all chunks for generation at startup
/// Creates bounded parallel generation (max 16 concurrent tasks)
fn queue_all_chunks_for_generation(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut world_manager: ResMut<UnifiedWorldManager>,
) {
    info!(
        "Starting static world generation for {}x{} chunks",
        config.world.total_chunks_x, config.world.total_chunks_z
    );

    // Only initialize chunks on terrain islands (with 200m margin for beaches)
    let half_x = (config.world.total_chunks_x / 2) as i32;
    let half_z = (config.world.total_chunks_z / 2) as i32;
    let margin = 200.0; // 1 chunk margin for beaches and nearshore content

    let mut total_count = 0;
    for array_z in 0..config.world.total_chunks_z {
        for array_x in 0..config.world.total_chunks_x {
            let chunk_x = array_x as i32 - half_x;
            let chunk_z = array_z as i32 - half_z;
            let coord = ChunkCoord::new(chunk_x, chunk_z);

            // Check if chunk is on a terrain island (skip ocean chunks)
            let chunk_center = coord.to_world_pos_with_size(world_manager.chunk_size);
            if world_manager.is_on_terrain_island_with_margin(chunk_center, margin) {
                if let Some(chunk) = world_manager.get_chunk_mut(coord) {
                    chunk.state = ChunkState::Loading;
                    total_count += 1;
                }
            }
        }
    }

    info!(
        "Initialized {} chunks for generation (island chunks only, skipped ocean)",
        total_count
    );

    // Create generation queue
    commands.insert_resource(StaticGenerationQueue {
        total_chunks: total_count,
        completed_chunks: 0,
        start_time: std::time::Instant::now(),
    });
}

/// Apply generated chunks - generates chunks synchronously with frame budget
/// Processes up to 10 chunks per frame to maintain responsiveness
#[allow(clippy::too_many_arguments)]
fn apply_generated_chunks(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut road_network: ResMut<RoadNetwork>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_registry: ResMut<MaterialRegistry>,
    mut world_rng: ResMut<WorldRng>,
    _spawn_registry: ResMut<SpawnRegistry>,
    mut queue: ResMut<StaticGenerationQueue>,
    mut next_state: ResMut<NextState<AppState>>,
    water_bodies: Query<&UnifiedWaterBody>,
    asset_server: Res<AssetServer>,
    config: Res<GameConfig>,
    env: Res<WorldEnvConfig>,
) {
    // Increased from 10 to 200 - no need to maintain 60 FPS during loading
    const CHUNKS_PER_FRAME: usize = 200;

    // Find chunks that need generation
    let chunks_to_process: Vec<ChunkCoord> = world_manager
        .chunks
        .iter()
        .flatten()
        .filter(|chunk| matches!(chunk.state, ChunkState::Loading))
        .map(|chunk| chunk.coord)
        .take(CHUNKS_PER_FRAME)
        .collect();

    if chunks_to_process.is_empty() {
        // All chunks generated - transition to InGame
        let elapsed = queue.start_time.elapsed();
        info!(
            "Static world generation complete! {} chunks in {:.2}s ({:.0} chunks/s)",
            queue.total_chunks,
            elapsed.as_secs_f32(),
            queue.total_chunks as f32 / elapsed.as_secs_f32()
        );
        next_state.set(AppState::InGame);
        return;
    }

    // Generate chunks using existing generators
    let road_generator = RoadGenerator;
    let building_generator = BuildingGenerator;
    let vehicle_generator = VehicleGenerator;
    let vegetation_generator = VegetationGenerator;

    for coord in chunks_to_process {
        // Generate all content layers
        road_generator.generate_roads(
            &mut commands,
            &mut world_manager,
            &mut road_network,
            coord,
            &mut meshes,
            &mut materials,
            &mut material_registry,
            &mut world_rng,
            &water_bodies,
            &config,
            &env,
        );

        building_generator.generate_buildings(
            &mut commands,
            &mut world_manager,
            &road_network,
            coord,
            &mut meshes,
            &mut materials,
            &mut world_rng,
            &water_bodies,
            &config,
            &env,
        );

        vehicle_generator.generate_vehicles(
            &mut commands,
            &mut world_manager,
            &road_network,
            coord,
            &mut meshes,
            &mut materials,
            &asset_server,
            &mut world_rng,
            &config,
        );

        vegetation_generator.generate_vegetation(
            &mut commands,
            &mut world_manager,
            &road_network,
            coord,
            &mut meshes,
            &mut materials,
            &mut world_rng,
            &water_bodies,
            &config,
            &env,
        );

        // Mark chunk as complete
        if let Some(chunk) = world_manager.get_chunk_mut(coord) {
            chunk.state = ChunkState::Loaded { lod_level: 0 };
        }

        queue.completed_chunks += 1;
    }

    // Progress logging every 100 chunks
    if queue.completed_chunks % 100 == 0 {
        let progress = (queue.completed_chunks as f32 / queue.total_chunks as f32) * 100.0;
        let elapsed = queue.start_time.elapsed().as_secs_f32();
        let rate = queue.completed_chunks as f32 / elapsed;
        let eta = (queue.total_chunks - queue.completed_chunks) as f32 / rate;

        info!(
            "Generation progress: {}/{} ({:.1}%) - {:.0} chunks/s - ETA: {:.1}s",
            queue.completed_chunks, queue.total_chunks, progress, rate, eta
        );
    }
}

/// Cleanup generation resources after loading completes
fn cleanup_generation_resources(mut commands: Commands) {
    commands.remove_resource::<StaticGenerationQueue>();
    info!("Static generation resources cleaned up");
}
