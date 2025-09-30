use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::collections::{HashMap, VecDeque};

use crate::components::world::ContentType;
use crate::systems::world::unified_world::{ChunkCoord, ChunkState, UnifiedWorldManager};

/// System sets for deterministic streaming order
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StreamingSet {
    Scan,     // Streaming decisions (unified_world_streaming_system)
    GenQueue, // Queue async generation jobs
    GenApply, // Apply completed jobs to ECS
}

/// Shared asset cache for chunk generation
/// Reuses meshes and materials to avoid memory bloat and FPS drops
#[derive(Resource)]
pub struct AsyncChunkAssets {
    pub cube_mesh: Handle<Mesh>,
    pub cylinder_mesh: Handle<Mesh>,
    pub building_material: Handle<StandardMaterial>,
    pub tree_material: Handle<StandardMaterial>,
}

/// Async chunk generation system following Oracle recommendations
/// Moves heavy chunk generation work off main thread for smooth 60+ FPS
pub struct AsyncChunkGenerationPlugin;

impl Plugin for AsyncChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AsyncChunkQueue::new())
            .configure_sets(
                Update,
                (
                    StreamingSet::Scan,
                    StreamingSet::GenQueue,
                    StreamingSet::GenApply,
                )
                    .chain(),
            )
            .add_systems(Startup, setup_async_chunk_assets)
            .add_systems(
                Update,
                queue_async_chunk_generation.in_set(StreamingSet::GenQueue),
            )
            .add_systems(
                Update,
                process_completed_chunks.in_set(StreamingSet::GenApply),
            );
    }
}

/// Setup shared assets for chunk generation (prevents asset duplication)
fn setup_async_chunk_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = AsyncChunkAssets {
        cube_mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0))),
        cylinder_mesh: meshes.add(Mesh::from(Cylinder::new(0.1, 1.0))),
        building_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.8),
            ..default()
        }),
        tree_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 0.2),
            ..default()
        }),
    };

    commands.insert_resource(assets);
    info!("Async chunk assets initialized (shared meshes/materials)");
}

/// Resource to track async chunk generation tasks
#[derive(Resource)]
pub struct AsyncChunkQueue {
    /// Tasks currently being processed
    pub active_tasks: HashMap<ChunkCoord, Task<ChunkGenerationResult>>,
    /// Completed results waiting to be applied (persisted across frames)
    pub completed_results: VecDeque<ChunkGenerationResult>,
    /// Maximum concurrent tasks to prevent resource exhaustion
    pub max_concurrent_tasks: usize,
    /// Maximum completed chunks to apply per frame (frame budget)
    pub max_completed_per_frame: usize,
}

impl Default for AsyncChunkQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncChunkQueue {
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
            completed_results: VecDeque::new(),
            max_concurrent_tasks: 3,    // Conservative limit for stability
            max_completed_per_frame: 2, // Apply at most 2 chunks per frame
        }
    }

    pub fn has_capacity(&self) -> bool {
        self.active_tasks.len() < self.max_concurrent_tasks
    }

    pub fn is_chunk_generating(&self, coord: ChunkCoord) -> bool {
        self.active_tasks.contains_key(&coord)
    }
}

/// Result of async chunk generation
#[derive(Debug)]
pub struct ChunkGenerationResult {
    pub coord: ChunkCoord,
    pub generation_id: u32,
    pub entities_data: Vec<EntityGenerationData>,
    pub success: bool,
    pub generation_time: f32,
}

/// Data needed to spawn an entity after async generation
#[derive(Debug, Clone)]
pub struct EntityGenerationData {
    pub position: Vec3,
    pub content_type: ContentType,
    pub scale: Vec3,
    pub rotation: Quat,
    pub color: Color,
}

/// System to queue chunks for async generation
/// Only consumes chunks already marked as Loading by unified_world_streaming_system
pub fn queue_async_chunk_generation(
    mut async_queue: ResMut<AsyncChunkQueue>,
    world_manager: Res<UnifiedWorldManager>,
) {
    // Only queue new tasks if we have capacity
    if !async_queue.has_capacity() {
        return;
    }

    // No ActiveEntity gate - rely on upstream streaming system to mark chunks Loading
    // If no ActiveEntity, streamer won't mark chunks, so we naturally do nothing

    // Find chunks already marked as Loading by the streamer
    // Sort by distance to prioritize closest chunks
    let mut loading_chunks: Vec<(ChunkCoord, f32)> = world_manager
        .chunks
        .iter()
        .flatten()
        .filter(|chunk| {
            matches!(chunk.state, ChunkState::Loading)
                && !async_queue.is_chunk_generating(chunk.coord)
        })
        .map(|chunk| (chunk.coord, chunk.distance_to_player))
        .collect();

    // Sort by distance (closest first)
    loading_chunks.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let task_pool = AsyncComputeTaskPool::get();
    let chunk_size = world_manager.chunk_size;

    for (coord, _distance) in loading_chunks {
        // Skip if no capacity
        if !async_queue.has_capacity() {
            break;
        }

        // Get generation_id from chunk
        let generation_id = world_manager
            .get_chunk(coord)
            .map(|chunk| chunk.generation_id)
            .unwrap_or(0);

        // Spawn async task for chunk generation with panic safety
        let generation_task = task_pool.spawn(async move {
            // CRITICAL: Wrap generation in panic guard to prevent main thread crashes
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                futures_lite::future::block_on(async {
                    generate_chunk_async(coord, chunk_size, generation_id).await
                })
            })) {
                Ok(result) => result,
                Err(_panic_info) => {
                    error!(
                        "PANIC caught during chunk generation for {:?} (gen_id: {})",
                        coord, generation_id
                    );
                    ChunkGenerationResult {
                        coord,
                        generation_id,
                        entities_data: Vec::new(),
                        success: false,
                        generation_time: 0.0,
                    }
                }
            }
        });

        async_queue.active_tasks.insert(coord, generation_task);

        debug!(
            "Queued async generation for chunk {:?} (gen_id: {})",
            coord, generation_id
        );
    }
}

/// System to process completed async chunk generation tasks
/// Applies strict per-frame budget and stale result protection with generation versioning
pub fn process_completed_chunks(
    mut commands: Commands,
    mut async_queue: ResMut<AsyncChunkQueue>,
    mut world_manager: ResMut<UnifiedWorldManager>,
    async_assets: Res<AsyncChunkAssets>,
    time: Res<Time>,
) {
    // Poll active tasks and push completed results into queue
    // CRITICAL: Remove tasks before polling to avoid "Task polled after completion" panic
    let coords_to_check: Vec<ChunkCoord> = async_queue.active_tasks.keys().copied().collect();

    for coord in coords_to_check {
        if let Some(mut task) = async_queue.active_tasks.remove(&coord) {
            if let Some(result) = future::block_on(future::poll_once(&mut task)) {
                // Task completed - push to queue for processing (may be deferred to future frames)
                async_queue.completed_results.push_back(result);
            } else {
                // Task not ready yet - put it back
                async_queue.active_tasks.insert(coord, task);
            }
        }
    }

    // Apply strict per-frame budget from completed results queue
    let max_to_process = async_queue
        .max_completed_per_frame
        .min(async_queue.completed_results.len());
    let mut processed_count = 0;
    let total_pending = async_queue.completed_results.len();

    // Process up to budget, leave remainder in queue for next frame
    for _ in 0..max_to_process {
        let Some(result) = async_queue.completed_results.pop_front() else {
            break;
        };

        // STALE RESULT GUARD: Check chunk exists, is Loading, AND generation_id matches
        let chunk_valid = world_manager
            .get_chunk(result.coord)
            .is_some_and(|chunk| {
                matches!(chunk.state, ChunkState::Loading)
                    && chunk.generation_id == result.generation_id
            });

        if !chunk_valid {
            debug!(
                "Discarding stale result for {:?} (gen_id: {}) - chunk unloaded, state changed, or regenerated",
                result.coord, result.generation_id
            );
            continue;
        }

        if result.success {
            // Spawn entities on main thread using generated data and shared assets
            let spawned_entities =
                spawn_entities_from_async_data(&mut commands, &result, &async_assets);

            // Mark chunk as loaded and track spawned entities
            if let Some(chunk) = world_manager.get_chunk_mut(result.coord) {
                chunk.state = ChunkState::Loaded { lod_level: 0 };
                chunk.last_update = time.elapsed_secs();
                chunk.entities.extend(spawned_entities);
            }

            info!(
                "Async generation completed for {:?} (gen_id: {}) in {:.2}ms ({} entities)",
                result.coord,
                result.generation_id,
                result.generation_time * 1000.0,
                result.entities_data.len()
            );

            processed_count += 1;
        } else {
            // Mark as failed, will retry on next streaming update
            if let Some(chunk) = world_manager.get_chunk_mut(result.coord) {
                chunk.state = ChunkState::Unloaded;
            }

            warn!(
                "Async generation failed for {:?} (gen_id: {})",
                result.coord, result.generation_id
            );
        }
    }

    if processed_count > 0 {
        debug!(
            "Applied {}/{} pending results this frame ({} active tasks, {} still queued)",
            processed_count,
            total_pending,
            async_queue.active_tasks.len(),
            async_queue.completed_results.len()
        );
    }
}

/// Async chunk generation function - runs off main thread
/// Only computes blueprint data, no ECS/Assets access
async fn generate_chunk_async(
    coord: ChunkCoord,
    chunk_size: f32,
    generation_id: u32,
) -> ChunkGenerationResult {
    let start_time = std::time::Instant::now();

    // Simulate chunk generation work (roads, buildings, vegetation, etc.)
    // This is where the heavy procedural generation would happen
    let mut entities_data = Vec::new();

    // Generate sample content (replace with actual generation logic)
    let chunk_center = coord.to_world_pos_with_size(chunk_size);

    // Generate some sample buildings
    for i in 0..5 {
        let offset = Vec3::new((i as f32 * 20.0) - 40.0, 0.0, (i as f32 * 15.0) - 30.0);

        entities_data.push(EntityGenerationData {
            position: chunk_center + offset,
            content_type: ContentType::Building,
            scale: Vec3::new(1.0, 1.0, 1.0), // Unit scale, will scale in Transform
            rotation: Quat::IDENTITY,
            color: Color::srgb(0.7, 0.7, 0.8),
        });
    }

    // Generate some vegetation
    for i in 0..10 {
        let offset = Vec3::new(
            (i as f32 * 12.0) - 60.0,
            0.0,
            ((i % 3) as f32 * 20.0) - 20.0,
        );

        entities_data.push(EntityGenerationData {
            position: chunk_center + offset,
            content_type: ContentType::Tree,
            scale: Vec3::new(2.0, 8.0, 2.0),
            rotation: Quat::from_rotation_y(i as f32 * 0.5),
            color: Color::srgb(0.2, 0.6, 0.2),
        });
    }

    let generation_time = start_time.elapsed().as_secs_f32();

    ChunkGenerationResult {
        coord,
        generation_id,
        entities_data,
        success: true,
        generation_time,
    }
}

/// Spawn entities on main thread from async generation data
/// Returns list of spawned entities for chunk tracking
/// Uses shared assets to prevent memory bloat
fn spawn_entities_from_async_data(
    commands: &mut Commands,
    result: &ChunkGenerationResult,
    assets: &AsyncChunkAssets,
) -> Vec<Entity> {
    let mut spawned_entities = Vec::with_capacity(result.entities_data.len());

    for entity_data in &result.entities_data {
        let entity = match entity_data.content_type {
            ContentType::Building => spawn_async_building(commands, assets, entity_data),
            ContentType::Tree => spawn_async_vegetation(commands, assets, entity_data),
            ContentType::Vehicle => spawn_async_vehicle(commands, assets, entity_data),
            ContentType::NPC => spawn_async_npc(commands, assets, entity_data),
            ContentType::Road => spawn_async_road(commands, assets, entity_data),
        };

        if let Some(entity) = entity {
            spawned_entities.push(entity);
        }
    }

    spawned_entities
}

/// Spawn building from async data
/// Uses shared unit cube mesh and material (no duplication)
fn spawn_async_building(
    commands: &mut Commands,
    assets: &AsyncChunkAssets,
    data: &EntityGenerationData,
) -> Option<Entity> {
    use bevy::render::view::VisibilityRange;

    // Use shared assets - no per-entity allocation!
    let mesh = assets.cube_mesh.clone();
    let material = assets.building_material.clone();

    // Building scale for visual appearance
    let building_scale = Vec3::new(8.0, 12.0, 8.0);

    let entity = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(data.position)
                .with_rotation(data.rotation)
                .with_scale(building_scale),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 350.0..400.0,
                use_aabb: false,
            },
            crate::components::world::Building {
                building_type: crate::components::world::BuildingType::Generic,
                height: building_scale.y,
                scale: building_scale,
            },
            crate::components::world::DynamicContent {
                content_type: ContentType::Building,
            },
        ))
        .id();

    Some(entity)
}

/// Spawn vegetation from async data
/// Uses shared cylinder mesh and material (no duplication)
fn spawn_async_vegetation(
    commands: &mut Commands,
    assets: &AsyncChunkAssets,
    data: &EntityGenerationData,
) -> Option<Entity> {
    use bevy::render::view::VisibilityRange;

    // Use shared assets - no per-entity allocation!
    let mesh = assets.cylinder_mesh.clone();
    let material = assets.tree_material.clone();

    // Tree scale for visual appearance
    let tree_scale = Vec3::new(2.0, 8.0, 2.0);

    let entity = commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(data.position)
                .with_rotation(data.rotation)
                .with_scale(tree_scale),
            VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: 250.0..300.0,
                use_aabb: false,
            },
            crate::components::world::DynamicContent {
                content_type: ContentType::Tree,
            },
        ))
        .id();

    Some(entity)
}

/// Placeholder functions for other content types
fn spawn_async_vehicle(
    _commands: &mut Commands,
    _assets: &AsyncChunkAssets,
    _data: &EntityGenerationData,
) -> Option<Entity> {
    // TODO: Implement async vehicle spawning
    None
}

fn spawn_async_npc(
    _commands: &mut Commands,
    _assets: &AsyncChunkAssets,
    _data: &EntityGenerationData,
) -> Option<Entity> {
    // TODO: Implement async NPC spawning
    None
}

fn spawn_async_road(
    _commands: &mut Commands,
    _assets: &AsyncChunkAssets,
    _data: &EntityGenerationData,
) -> Option<Entity> {
    // TODO: Implement async road spawning
    None
}
