use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::collections::HashMap;

use crate::systems::world::{ChunkCoord, ChunkState, UnifiedWorldManager};
use crate::components::world::ContentType;

/// Async chunk generation system following Oracle recommendations
/// Moves heavy chunk generation work off main thread for smooth 60+ FPS
pub struct AsyncChunkGenerationPlugin;

impl Plugin for AsyncChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AsyncChunkQueue>()
            .add_systems(Update, (
                queue_async_chunk_generation,
                process_completed_chunks,
            ).chain());
    }
}

/// Resource to track async chunk generation tasks
#[derive(Resource, Default)]
pub struct AsyncChunkQueue {
    /// Tasks currently being processed
    pub active_tasks: HashMap<ChunkCoord, Task<ChunkGenerationResult>>,
    /// Maximum concurrent tasks to prevent resource exhaustion
    pub max_concurrent_tasks: usize,
}

impl AsyncChunkQueue {
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
            max_concurrent_tasks: 4, // Limit concurrent generation for stability
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
pub fn queue_async_chunk_generation(
    mut async_queue: ResMut<AsyncChunkQueue>,
    mut world_manager: ResMut<UnifiedWorldManager>,
    active_query: Query<&Transform, With<crate::components::player::ActiveEntity>>,
    time: Res<Time>,
) {
    // Only queue new tasks if we have capacity
    if !async_queue.has_capacity() {
        return;
    }
    
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Find chunks that need loading and aren't already being generated
    let chunks_to_load = world_manager.get_chunks_to_load(active_pos);
    
    let task_pool = AsyncComputeTaskPool::get();
    
    for coord in chunks_to_load {
        // Skip if already generating
        if async_queue.is_chunk_generating(coord) {
            continue;
        }
        
        // Skip if no capacity
        if !async_queue.has_capacity() {
            break;
        }
        
        // Mark chunk as generating
        if let Some(chunk) = world_manager.get_chunk_mut(coord) {
            chunk.state = ChunkState::Loading;
        }
        
        // Spawn async task for chunk generation
        let generation_task = task_pool.spawn(async move {
            generate_chunk_async(coord).await
        });
        
        async_queue.active_tasks.insert(coord, generation_task);
        
        info!("Queued async generation for chunk {:?}", coord);
    }
}

/// System to process completed async chunk generation tasks
pub fn process_completed_chunks(
    mut commands: Commands,
    mut async_queue: ResMut<AsyncChunkQueue>,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let mut completed_coords = Vec::new();
    
    // Check for completed tasks
    for (coord, task) in &mut async_queue.active_tasks {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            completed_coords.push((*coord, result));
        }
    }
    
    // Process completed chunks
    for (coord, result) in completed_coords {
        // Remove from active tasks
        async_queue.active_tasks.remove(&coord);
        
        if result.success {
            // Spawn entities on main thread using generated data
            spawn_entities_from_async_data(
                &mut commands, 
                &mut meshes, 
                &mut materials, 
                &result
            );
            
            // Mark chunk as loaded
            if let Some(chunk) = world_manager.get_chunk_mut(coord) {
                chunk.state = ChunkState::Loaded { lod_level: 0 };
                chunk.last_update = time.elapsed_secs();
            }
            
            info!("Async chunk generation completed for {:?} in {:.2}ms", 
                  coord, result.generation_time * 1000.0);
        } else {
            // Mark as failed, will retry on next streaming update
            if let Some(chunk) = world_manager.get_chunk_mut(coord) {
                chunk.state = ChunkState::Unloaded;
            }
            
            warn!("Async chunk generation failed for {:?}", coord);
        }
    }
}

/// Async chunk generation function - runs off main thread
async fn generate_chunk_async(coord: ChunkCoord) -> ChunkGenerationResult {
    let start_time = std::time::Instant::now();
    
    // Simulate chunk generation work (roads, buildings, vegetation, etc.)
    // This is where the heavy procedural generation would happen
    let mut entities_data = Vec::new();
    
    // Generate sample content (replace with actual generation logic)
    let chunk_center = coord.to_world_pos_with_size(128.0); // Use finite world chunk size
    
    // Generate some sample buildings
    for i in 0..5 {
        let offset = Vec3::new(
            (i as f32 * 20.0) - 40.0,
            0.0,
            (i as f32 * 15.0) - 30.0,
        );
        
        entities_data.push(EntityGenerationData {
            position: chunk_center + offset,
            content_type: ContentType::Building,
            scale: Vec3::new(8.0, 12.0, 8.0),
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
        entities_data,
        success: true,
        generation_time,
    }
}

/// Spawn entities on main thread from async generation data
fn spawn_entities_from_async_data(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    result: &ChunkGenerationResult,
) {
    for entity_data in &result.entities_data {
        match entity_data.content_type {
            ContentType::Building => {
                spawn_async_building(commands, meshes, materials, entity_data);
            },
            ContentType::Tree => {
                spawn_async_vegetation(commands, meshes, materials, entity_data);
            },
            ContentType::Vehicle => {
                spawn_async_vehicle(commands, meshes, materials, entity_data);
            },
            ContentType::NPC => {
                spawn_async_npc(commands, meshes, materials, entity_data);
            },
            ContentType::Road => {
                spawn_async_road(commands, meshes, materials, entity_data);
            },
        }
    }
}

/// Spawn building from async data
fn spawn_async_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    data: &EntityGenerationData,
) {
    use bevy::render::view::VisibilityRange;
    
    let mesh = meshes.add(Mesh::from(Cuboid::from_size(data.scale)));
    let material = materials.add(StandardMaterial {
        base_color: data.color,
        ..default()
    });
    
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(data.position)
            .with_rotation(data.rotation)
            .with_scale(data.scale),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 350.0..400.0, // Buildings visible from 350-400m
            use_aabb: false,
        },
        crate::components::world::Building {
            building_type: crate::components::world::BuildingType::Generic,
            height: data.scale.y,
            scale: data.scale,
        },
        crate::components::world::DynamicContent {
            content_type: ContentType::Building,
        },
    ));
}

/// Spawn vegetation from async data
fn spawn_async_vegetation(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    data: &EntityGenerationData,
) {
    use bevy::render::view::VisibilityRange;
    
    let mesh = meshes.add(Mesh::from(Cylinder::new(data.scale.x * 0.1, data.scale.y)));
    let material = materials.add(StandardMaterial {
        base_color: data.color,
        ..default()
    });
    
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(data.position)
            .with_rotation(data.rotation),
        VisibilityRange {
            start_margin: 0.0..0.0,
            end_margin: 250.0..300.0, // Trees visible from 250-300m
            use_aabb: false,
        },
        crate::components::world::DynamicContent {
            content_type: ContentType::Tree,
        },
    ));
}

/// Placeholder functions for other content types
fn spawn_async_vehicle(
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    _data: &EntityGenerationData,
) {
    // TODO: Implement async vehicle spawning
}

fn spawn_async_npc(
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    _data: &EntityGenerationData,
) {
    // TODO: Implement async NPC spawning
}

fn spawn_async_road(
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    _data: &EntityGenerationData,
) {
    // TODO: Implement async road spawning
}
