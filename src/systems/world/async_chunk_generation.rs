use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::collections::{HashMap, VecDeque};

use crate::components::world::ContentType;
use crate::systems::world::road_network::{ROAD_CELL_SIZE, RoadType, generate_unique_road_id};
use crate::systems::world::unified_world::{ChunkCoord, ChunkState, UnifiedWorldManager};
use rand::{Rng, SeedableRng};

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
            .insert_resource(crate::systems::world::road_network::RoadOwnership::default())
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
    pub road_blueprints: Vec<RoadBlueprint>,
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

#[derive(Clone, Debug)]
pub struct RoadBlueprint {
    pub road_id: u64,
    pub road_type: RoadType,
    pub control_points: Vec<Vec3>,
    pub center_pos: Vec3,
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
                        road_blueprints: Vec::new(),
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
#[allow(clippy::too_many_arguments)]
pub fn process_completed_chunks(
    mut commands: Commands,
    mut async_queue: ResMut<AsyncChunkQueue>,
    mut world_manager: ResMut<UnifiedWorldManager>,
    async_assets: Res<AsyncChunkAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_registry: ResMut<crate::resources::MaterialRegistry>,
    mut road_ownership: ResMut<crate::systems::world::road_network::RoadOwnership>,
    player_query: Query<&Transform, With<crate::components::ActiveEntity>>,
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
        let chunk_valid = world_manager.get_chunk(result.coord).is_some_and(|chunk| {
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
            let mut spawned_entities =
                spawn_entities_from_async_data(&mut commands, &result, &async_assets);

            let player_pos = player_query
                .iter()
                .next()
                .map(|t| t.translation)
                .unwrap_or(Vec3::ZERO);

            let roads_spawned = apply_road_blueprints(
                &mut commands,
                &mut world_manager,
                &result.road_blueprints,
                result.coord,
                player_pos,
                &mut meshes,
                &mut materials,
                &mut material_registry,
                &mut road_ownership,
            );

            if !result.road_blueprints.is_empty() {
                debug!(
                    "Spawned {} roads from {} blueprints for chunk {:?}",
                    roads_spawned.len(),
                    result.road_blueprints.len(),
                    result.coord
                );
            }

            spawned_entities.extend(roads_spawned);

            if let Some(chunk) = world_manager.get_chunk_mut(result.coord) {
                chunk.state = ChunkState::Loaded { lod_level: 0 };
                chunk.last_update = time.elapsed_secs();
                chunk.entities.extend(spawned_entities);
                chunk.roads_generated = true;
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

    let mut road_blueprints = Vec::new();

    let chunk_center_x = coord.x as f32 * chunk_size + chunk_size / 2.0;
    let chunk_center_z = coord.z as f32 * chunk_size + chunk_size / 2.0;
    let half = chunk_size / 2.0;

    let min_cell_x = ((chunk_center_x - half) / ROAD_CELL_SIZE).floor() as i32;
    let max_cell_x = ((chunk_center_x + half) / ROAD_CELL_SIZE).floor() as i32;
    let min_cell_z = ((chunk_center_z - half) / ROAD_CELL_SIZE).floor() as i32;
    let max_cell_z = ((chunk_center_z + half) / ROAD_CELL_SIZE).floor() as i32;

    for cx in min_cell_x..=max_cell_x {
        for cz in min_cell_z..=max_cell_z {
            let cell_coord = IVec2::new(cx, cz);
            let seed = ((cx as u64) << 32) | ((cz as u64) & 0xFFFFFFFF);
            let mut cell_rng = rand::rngs::StdRng::seed_from_u64(seed ^ 0xDEADBEEF);

            let before = road_blueprints.len();
            generate_road_cell_blueprints(
                cell_coord,
                ROAD_CELL_SIZE,
                &mut cell_rng,
                &mut road_blueprints,
            );
            let added = road_blueprints.len() - before;
            if added > 0 {
                debug!(
                    "Cell {:?} generated {} road blueprints for chunk {:?}",
                    cell_coord, added, coord
                );
            }
        }
    }

    let generation_time = start_time.elapsed().as_secs_f32();

    ChunkGenerationResult {
        coord,
        generation_id,
        entities_data,
        road_blueprints,
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
    None
}

#[allow(clippy::too_many_arguments)]
fn apply_road_blueprints(
    commands: &mut Commands,
    world: &mut UnifiedWorldManager,
    blueprints: &[RoadBlueprint],
    _chunk_coord: ChunkCoord,
    player_pos: Vec3,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    material_registry: &mut crate::resources::MaterialRegistry,
    road_ownership: &mut crate::systems::world::road_network::RoadOwnership,
) -> Vec<Entity> {
    use crate::bundles::VisibleChildBundle;
    use crate::components::{ContentType, DynamicContent, RoadEntity};
    use crate::resources::MaterialKey;
    use crate::systems::world::road_mesh::{generate_road_markings_mesh, generate_road_mesh};
    use crate::systems::world::road_network::RoadSpline;
    use crate::systems::world::unified_world::{
        ContentLayer, UNIFIED_CHUNK_SIZE, UnifiedChunkEntity,
    };

    let mut spawned = Vec::new();

    for blueprint in blueprints {
        if world.road_network.roads.contains_key(&blueprint.road_id) {
            debug!("Skipping duplicate road_id: {}", blueprint.road_id);
            continue;
        }

        let distance = blueprint.center_pos.distance(player_pos);

        let owner_coord = ChunkCoord::from_world_pos(blueprint.center_pos, UNIFIED_CHUNK_SIZE);

        let road_spline = RoadSpline {
            id: blueprint.road_id,
            control_points: blueprint.control_points.clone(),
            road_type: blueprint.road_type,
            connections: Vec::new(),
        };

        world
            .road_network
            .roads
            .insert(blueprint.road_id, road_spline.clone());

        let (base_color, roughness) = match blueprint.road_type {
            RoadType::Highway => (Color::srgb(0.4, 0.4, 0.45), 0.8),
            RoadType::MainStreet => (Color::srgb(0.35, 0.35, 0.4), 0.8),
            RoadType::SideStreet => (Color::srgb(0.45, 0.45, 0.5), 0.7),
            RoadType::Alley => (Color::srgb(0.5, 0.5, 0.45), 0.6),
        };

        let road_material_key = MaterialKey::road(base_color).with_roughness(roughness);
        let road_material = material_registry.get_or_create(materials, road_material_key);

        let road_entity = commands
            .spawn((
                UnifiedChunkEntity {
                    coord: owner_coord,
                    layer: ContentLayer::Roads,
                },
                RoadEntity {
                    road_id: blueprint.road_id,
                },
                Transform::from_translation(blueprint.center_pos),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
                DynamicContent {
                    content_type: ContentType::Road,
                },
            ))
            .id();

        let road_mesh = generate_road_mesh(&road_spline);
        commands.spawn((
            Mesh3d(meshes.add(road_mesh)),
            MeshMaterial3d(road_material),
            Transform::from_translation(-blueprint.center_pos + Vec3::new(0.0, 0.0, 0.0)),
            ChildOf(road_entity),
            VisibleChildBundle::default(),
        ));

        if distance < 100.0 {
            let marking_material_key =
                MaterialKey::road_marking(Color::srgb(0.95, 0.95, 0.95)).with_roughness(0.6);
            let marking_material = material_registry.get_or_create(materials, marking_material_key);

            let marking_meshes = generate_road_markings_mesh(&road_spline);
            for marking_mesh in marking_meshes {
                commands.spawn((
                    Mesh3d(meshes.add(marking_mesh)),
                    MeshMaterial3d(marking_material.clone()),
                    Transform::from_translation(-blueprint.center_pos + Vec3::new(0.0, 0.01, 0.0)),
                    ChildOf(road_entity),
                    VisibleChildBundle::default(),
                ));
            }
        }

        let samples = 20;
        for i in 0..samples {
            let t = i as f32 / (samples - 1) as f32;
            let road_point = road_spline.evaluate(t);
            world.placement_grid.add_entity(
                road_point,
                ContentType::Road,
                blueprint.road_type.width() * 0.5,
            );
        }

        road_ownership.register_road(blueprint.road_id, owner_coord, road_entity);

        spawned.push(road_entity);
    }

    spawned
}

fn generate_road_cell_blueprints(
    cell_coord: IVec2,
    cell_size: f32,
    _rng: &mut impl rand::Rng,
    blueprints: &mut Vec<RoadBlueprint>,
) {
    let cell_seed = ((cell_coord.x as u64) << 32) | ((cell_coord.y as u64) & 0xFFFFFFFF);
    let mut rng = rand::rngs::StdRng::seed_from_u64(cell_seed ^ 0x524F414453);

    let base_x = cell_coord.x as f32 * cell_size;
    let base_z = cell_coord.y as f32 * cell_size;

    let mut local_index: u16 = 0;

    if cell_coord == IVec2::ZERO {
        generate_premium_spawn_cell_blueprints(
            base_x,
            base_z,
            cell_size,
            cell_coord,
            &mut local_index,
            blueprints,
        );
        return;
    }

    if cell_coord.x % 2 == 0 && cell_coord.y % 2 != 0 {
        let road_type = RoadType::MainStreet;
        let height = road_type.height();
        let start = Vec3::new(base_x, height, base_z - cell_size * 0.5);
        let control = Vec3::new(
            base_x + rng.gen_range(-10.0..10.0),
            height,
            base_z + cell_size * 0.2,
        );
        let end = Vec3::new(base_x, height, base_z + cell_size * 1.5);

        let center_pos = (start + end) * 0.5;
        blueprints.push(RoadBlueprint {
            road_id: generate_unique_road_id(cell_coord, local_index),
            road_type,
            control_points: vec![start, control, end],
            center_pos,
        });
        local_index += 1;
    }

    if cell_coord.y % 2 == 0 && cell_coord.x % 2 != 0 {
        let road_type = RoadType::MainStreet;
        let height = road_type.height();
        let start = Vec3::new(base_x - cell_size * 0.5, height, base_z);
        let control = Vec3::new(
            base_x + cell_size * 0.2,
            height,
            base_z + rng.gen_range(-10.0..10.0),
        );
        let end = Vec3::new(base_x + cell_size * 1.5, height, base_z);

        let center_pos = (start + end) * 0.5;
        blueprints.push(RoadBlueprint {
            road_id: generate_unique_road_id(cell_coord, local_index),
            road_type,
            control_points: vec![start, control, end],
            center_pos,
        });
        local_index += 1;
    }

    let roads_before = blueprints.len();
    if cell_coord.x % 2 != 0 || cell_coord.y % 2 != 0 {
        for i in 0..2 {
            for j in 0..2 {
                let sub_x = base_x + (i as f32 + 0.5) * cell_size / 3.0;
                let sub_z = base_z + (j as f32 + 0.5) * cell_size / 3.0;

                let offset_x = rng.gen_range(-15.0..15.0);
                let offset_z = rng.gen_range(-15.0..15.0);

                if rng.gen_bool(0.8) {
                    let road_type = RoadType::SideStreet;
                    let height = road_type.height();
                    let start = Vec3::new(sub_x + offset_x, height, sub_z - 40.0);
                    let end = Vec3::new(sub_x + offset_x, height, sub_z + 40.0);

                    let center_pos = (start + end) * 0.5;
                    blueprints.push(RoadBlueprint {
                        road_id: generate_unique_road_id(cell_coord, local_index),
                        road_type,
                        control_points: vec![start, end],
                        center_pos,
                    });
                    local_index += 1;
                }

                if rng.gen_bool(0.8) {
                    let road_type = RoadType::SideStreet;
                    let height = road_type.height();
                    let start = Vec3::new(sub_x - 40.0, height, sub_z + offset_z);
                    let end = Vec3::new(sub_x + 40.0, height, sub_z + offset_z);

                    let center_pos = (start + end) * 0.5;
                    blueprints.push(RoadBlueprint {
                        road_id: generate_unique_road_id(cell_coord, local_index),
                        road_type,
                        control_points: vec![start, end],
                        center_pos,
                    });
                    local_index += 1;
                }
            }
        }
    }

    if blueprints.len() > roads_before {
        debug!(
            "Generated {} side streets for cell {:?}",
            blueprints.len() - roads_before,
            cell_coord
        );
    }
}

fn generate_premium_spawn_cell_blueprints(
    base_x: f32,
    base_z: f32,
    cell_size: f32,
    cell_coord: IVec2,
    local_index: &mut u16,
    blueprints: &mut Vec<RoadBlueprint>,
) {
    let height = 0.0;

    let cell_min_x = base_x;
    let cell_max_x = base_x + cell_size;
    let cell_min_z = base_z;
    let cell_max_z = base_z + cell_size;

    let highway_configs = [
        (
            Vec3::new(cell_min_x, height, base_z + cell_size * 0.5),
            Vec3::new(base_x + cell_size * 0.3, height, base_z + cell_size * 0.6),
            Vec3::new(cell_max_x, height, base_z + cell_size * 0.5),
            RoadType::Highway,
        ),
        (
            Vec3::new(base_x + cell_size * 0.5, height, cell_min_z),
            Vec3::new(base_x + cell_size * 0.6, height, base_z + cell_size * 0.3),
            Vec3::new(base_x + cell_size * 0.5, height, cell_max_z),
            RoadType::Highway,
        ),
    ];

    let main_street_configs = [
        (
            Vec3::new(cell_min_x, height, base_z + cell_size * 0.25),
            Vec3::new(base_x + cell_size * 0.3, height, base_z + cell_size * 0.3),
            Vec3::new(cell_max_x, height, base_z + cell_size * 0.25),
            RoadType::MainStreet,
        ),
        (
            Vec3::new(base_x + cell_size * 0.25, height, cell_min_z),
            Vec3::new(base_x + cell_size * 0.3, height, base_z + cell_size * 0.3),
            Vec3::new(base_x + cell_size * 0.25, height, cell_max_z),
            RoadType::MainStreet,
        ),
    ];

    for (start, control, end, road_type) in highway_configs.iter().chain(main_street_configs.iter())
    {
        let center_pos = (*start + *end) * 0.5;
        blueprints.push(RoadBlueprint {
            road_id: generate_unique_road_id(cell_coord, *local_index),
            road_type: *road_type,
            control_points: vec![*start, *control, *end],
            center_pos,
        });
        *local_index += 1;
    }

    for i in 0..3 {
        for j in 0..3 {
            if i == 1 && j == 1 {
                continue;
            }

            let sub_x = base_x + (i as f32 + 0.5) * cell_size / 4.0;
            let sub_z = base_z + (j as f32 + 0.5) * cell_size / 4.0;

            let start = Vec3::new(sub_x - 30.0, height, sub_z);
            let end = Vec3::new(sub_x + 30.0, height, sub_z);
            let center_pos = (start + end) * 0.5;
            blueprints.push(RoadBlueprint {
                road_id: generate_unique_road_id(cell_coord, *local_index),
                road_type: RoadType::SideStreet,
                control_points: vec![start, end],
                center_pos,
            });
            *local_index += 1;

            let start = Vec3::new(sub_x, height, sub_z - 30.0);
            let end = Vec3::new(sub_x, height, sub_z + 30.0);
            let center_pos = (start + end) * 0.5;
            blueprints.push(RoadBlueprint {
                road_id: generate_unique_road_id(cell_coord, *local_index),
                road_type: RoadType::SideStreet,
                control_points: vec![start, end],
                center_pos,
            });
            *local_index += 1;
        }
    }

    debug!(
        "Generated {} premium roads for spawn cell (0,0)",
        blueprints.len()
    );
}
