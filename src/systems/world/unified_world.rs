#![allow(
    clippy::type_complexity,
    clippy::manual_flatten,
    clippy::collapsible_if
)]
use crate::components::{ActiveEntity, ContentType};
use crate::config::GameConfig;
use crate::systems::world::road_network::RoadNetwork;
use bevy::prelude::*;
use std::collections::HashMap;

// UNIFIED WORLD GENERATION SYSTEM
// This replaces map_system.rs, dynamic_content.rs coordination
// Provides single source of truth for world streaming and generation

/// Convert chunk coordinates to flat array index for finite world
/// Eliminates config duplication between WorldConfig and UnifiedWorldManager
pub fn chunk_coord_to_index(
    coord: ChunkCoord,
    total_chunks_x: usize,
    total_chunks_z: usize,
) -> Option<usize> {
    let half_chunks_x = (total_chunks_x / 2) as i32;
    let half_chunks_z = (total_chunks_z / 2) as i32;

    // Convert world chunk coords to array coords (0 to total_chunks - 1)
    let array_x = coord.x + half_chunks_x;
    let array_z = coord.z + half_chunks_z;

    // Bounds check for finite world
    if array_x >= 0
        && array_x < total_chunks_x as i32
        && array_z >= 0
        && array_z < total_chunks_z as i32
    {
        Some((array_z as usize) * total_chunks_x + (array_x as usize))
    } else {
        None
    }
}

/// Standard chunk size used across all world systems
pub const UNIFIED_CHUNK_SIZE: f32 = 200.0;

/// Maximum streaming radius around active entity
pub const UNIFIED_STREAMING_RADIUS: f32 = 800.0;

/// LOD transition distances - optimized for 60+ FPS target
pub const LOD_DISTANCES: [f32; 3] = [150.0, 300.0, 500.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_world_pos(world_pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (world_pos.x / chunk_size).floor() as i32,
            z: (world_pos.z / chunk_size).floor() as i32,
        }
    }

    // Backwards compatibility with old UNIFIED_CHUNK_SIZE
    pub fn from_world_pos_legacy(world_pos: Vec3) -> Self {
        Self::from_world_pos(world_pos, UNIFIED_CHUNK_SIZE)
    }

    pub fn to_world_pos_with_size(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(
            self.x as f32 * chunk_size + chunk_size * 0.5,
            0.0,
            self.z as f32 * chunk_size + chunk_size * 0.5,
        )
    }

    // Backwards compatibility
    pub fn to_world_pos(&self) -> Vec3 {
        self.to_world_pos_with_size(UNIFIED_CHUNK_SIZE)
    }

    pub fn distance_to(&self, other: ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }

    pub fn distance_squared_to(&self, other: ChunkCoord) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        dx * dx + dz * dz
    }
}

/// Zero-allocation iterator for ring pattern chunk coordinates
/// Replaces generate_ring_coords to eliminate Vec allocation per streaming tick
pub struct RingCoordinatesIter {
    center: ChunkCoord,
    ring: i32,
    current_index: i32,
    max_index: i32,
}

impl RingCoordinatesIter {
    pub fn new(center: ChunkCoord, ring: i32) -> Self {
        let max_index = if ring == 0 { 1 } else { ring * 8 };
        Self {
            center,
            ring,
            current_index: 0,
            max_index,
        }
    }
}

impl Iterator for RingCoordinatesIter {
    type Item = ChunkCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.max_index {
            return None;
        }

        let coord = if self.ring == 0 {
            self.center
        } else {
            let i = self.current_index;
            let (dx, dz) = match i / (self.ring * 2) {
                0 => (-self.ring + (i % (self.ring * 2)), -self.ring), // Top edge
                1 => (self.ring, -self.ring + (i % (self.ring * 2))),  // Right edge
                2 => (self.ring - (i % (self.ring * 2)), self.ring),   // Bottom edge
                3 => (-self.ring, self.ring - (i % (self.ring * 2))),  // Left edge
                _ => (0, 0),                                           // Should never happen
            };

            ChunkCoord::new(self.center.x + dx, self.center.z + dz)
        };

        self.current_index += 1;
        Some(coord)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkState {
    Unloaded,
    Loading,
    Loaded { lod_level: usize },
    Unloading,
}

#[derive(Debug, Clone)]
pub struct ChunkData {
    pub coord: ChunkCoord,
    pub state: ChunkState,
    pub distance_to_player: f32,
    pub entities: Vec<Entity>,
    pub last_update: f32,
    pub generation_id: u32,

    // Layer generation flags
    pub roads_generated: bool,
    pub buildings_generated: bool,
    pub vehicles_generated: bool,
    pub vegetation_generated: bool,
}

impl ChunkData {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            state: ChunkState::Unloaded,
            distance_to_player: f32::INFINITY,
            entities: Vec::new(),
            last_update: 0.0,
            generation_id: 0,
            roads_generated: false,
            buildings_generated: false,
            vehicles_generated: false,
            vegetation_generated: false,
        }
    }
}

/// Shared collision/placement grid for preventing entity overlap
#[derive(Debug, Default)]
pub struct PlacementGrid {
    /// Grid cells containing entity positions and types
    /// Key: (grid_x, grid_z), Value: Vec of (position, content_type, radius)
    grid: HashMap<(i32, i32), Vec<(Vec3, ContentType, f32)>>,
    /// Grid cell size (should be smaller than chunk size for efficiency)
    cell_size: f32,
}

impl PlacementGrid {
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
            cell_size: 50.0, // 4 cells per chunk
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn add_entity(&mut self, position: Vec3, content_type: ContentType, radius: f32) {
        let cell = self.world_to_grid(position);
        self.grid
            .entry(cell)
            .or_default()
            .push((position, content_type, radius));
    }

    pub fn remove_entity(&mut self, position: Vec3, content_type: ContentType) {
        let cell = self.world_to_grid(position);
        if let Some(entities) = self.grid.get_mut(&cell) {
            entities.retain(|(pos, content, _)| {
                !(pos.distance(position) < 1.0 && *content == content_type)
            });
        }
    }

    pub fn can_place(
        &self,
        position: Vec3,
        _content_type: ContentType,
        radius: f32,
        min_distance: f32,
    ) -> bool {
        let cell = self.world_to_grid(position);

        // Check current cell and adjacent cells
        for dx in -1..=1 {
            for dz in -1..=1 {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(entities) = self.grid.get(&check_cell) {
                    for (existing_pos, _existing_type, existing_radius) in entities {
                        let required_distance = min_distance.max(*existing_radius + radius);
                        let required_distance_squared = required_distance.powi(2);
                        let distance_squared = position.distance_squared(*existing_pos);

                        if distance_squared < required_distance_squared {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    pub fn get_nearby_entities(
        &self,
        position: Vec3,
        radius: f32,
    ) -> Vec<(Vec3, ContentType, f32)> {
        let mut result = Vec::new();
        let cell = self.world_to_grid(position);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let check_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(entities) = self.grid.get(&check_cell) {
                    for (entity_pos, content_type, entity_radius) in entities {
                        let radius_squared = radius.powi(2);
                        if position.distance_squared(*entity_pos) <= radius_squared {
                            result.push((*entity_pos, *content_type, *entity_radius));
                        }
                    }
                }
            }
        }

        result
    }

    fn world_to_grid(&self, position: Vec3) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }
}

/// Central resource managing all world generation - Now uses finite world with Vec storage
#[derive(Resource)]
pub struct UnifiedWorldManager {
    // FINITE WORLD: Vec<Option<ChunkData>> for O(1) access instead of HashMap
    pub chunks: Vec<Option<ChunkData>>,
    pub total_chunks_x: usize,
    pub total_chunks_z: usize,
    pub chunk_size: f32,
    pub world_bounds: (f32, f32, f32, f32), // min_x, max_x, min_z, max_z

    pub placement_grid: PlacementGrid,
    pub road_network: RoadNetwork,

    // Streaming state
    pub active_chunk: Option<ChunkCoord>,
    pub streaming_radius_chunks: i32,
    pub last_update: f32,

    // Performance tracking
    pub chunks_loaded_this_frame: usize,
    pub chunks_unloaded_this_frame: usize,
    pub max_chunks_per_frame: usize,
}

impl UnifiedWorldManager {
    /// Create UnifiedWorldManager from GameConfig for finite world
    pub fn from_config(config: &GameConfig) -> Self {
        let total_chunk_count = config.world.total_chunk_count();

        Self {
            chunks: vec![None; total_chunk_count],
            total_chunks_x: config.world.total_chunks_x,
            total_chunks_z: config.world.total_chunks_z,
            chunk_size: config.world.chunk_size,
            world_bounds: config.world.world_bounds(),
            placement_grid: PlacementGrid::new(),
            road_network: RoadNetwork::default(),
            active_chunk: None,
            streaming_radius_chunks: (config.world.streaming_radius / config.world.chunk_size)
                .ceil() as i32,
            last_update: 0.0,
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            max_chunks_per_frame: 4, // Prevent frame drops
        }
    }

    /// Convert chunk coordinates to Vec index (finite world bounds check)
    fn chunk_coord_to_index(&self, coord: ChunkCoord) -> Option<usize> {
        chunk_coord_to_index(coord, self.total_chunks_x, self.total_chunks_z)
    }

    /// Check if chunk coordinates are within finite world bounds
    pub fn is_chunk_in_bounds(&self, coord: ChunkCoord) -> bool {
        self.chunk_coord_to_index(coord).is_some()
    }
}

impl Default for UnifiedWorldManager {
    fn default() -> Self {
        // Default uses small world for backwards compatibility
        let total_chunks = 32; // 32x32 = 1024 chunks
        let total_chunk_count = total_chunks * total_chunks;

        Self {
            chunks: vec![None; total_chunk_count],
            total_chunks_x: total_chunks,
            total_chunks_z: total_chunks,
            chunk_size: UNIFIED_CHUNK_SIZE,
            world_bounds: (-3200.0, 3200.0, -3200.0, 3200.0), // 6.4km x 6.4km default
            placement_grid: PlacementGrid::new(),
            road_network: RoadNetwork::default(),
            active_chunk: None,
            streaming_radius_chunks: (UNIFIED_STREAMING_RADIUS / UNIFIED_CHUNK_SIZE).ceil() as i32,
            last_update: 0.0,
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            max_chunks_per_frame: 4, // Prevent frame drops
        }
    }
}

impl UnifiedWorldManager {
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&ChunkData> {
        self.chunk_coord_to_index(coord)
            .and_then(|index| self.chunks.get(index))
            .and_then(|chunk| chunk.as_ref())
    }

    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut ChunkData> {
        if let Some(index) = self.chunk_coord_to_index(coord) {
            if self.chunks[index].is_none() {
                self.chunks[index] = Some(ChunkData::new(coord));
            }
            self.chunks[index].as_mut()
        } else {
            None
        }
    }

    pub fn is_chunk_loaded(&self, coord: ChunkCoord) -> bool {
        matches!(
            self.get_chunk(coord).map(|c| c.state),
            Some(ChunkState::Loaded { .. })
        )
    }

    pub fn calculate_lod_level(&self, distance: f32) -> usize {
        for (i, &max_distance) in LOD_DISTANCES.iter().enumerate() {
            if distance <= max_distance {
                return i;
            }
        }
        LOD_DISTANCES.len() - 1
    }

    pub fn cleanup_distant_chunks(&mut self, active_pos: Vec3) -> Vec<ChunkCoord> {
        let mut to_unload = Vec::new();
        let max_distance_squared = (UNIFIED_STREAMING_RADIUS + self.chunk_size).powi(2);

        for chunk_opt in &mut self.chunks {
            if let Some(chunk) = chunk_opt {
                let chunk_pos = chunk.coord.to_world_pos_with_size(self.chunk_size);
                let distance_squared = active_pos.distance_squared(chunk_pos);
                chunk.distance_to_player = distance_squared.sqrt(); // Only calculate sqrt when needed

                if distance_squared > max_distance_squared {
                    if !matches!(chunk.state, ChunkState::Unloaded | ChunkState::Unloading) {
                        chunk.state = ChunkState::Unloading;
                        to_unload.push(chunk.coord);
                    }
                }
            }
        }

        to_unload
    }

    pub fn get_chunks_to_load(&mut self, active_pos: Vec3) -> Vec<ChunkCoord> {
        let active_chunk = ChunkCoord::from_world_pos(active_pos, self.chunk_size);
        let mut to_load = Vec::new();
        let max_distance_squared = UNIFIED_STREAMING_RADIUS.powi(2);

        // PERFORMANCE: Use ring pattern instead of nested loops for better cache locality
        for ring in 0..=self.streaming_radius_chunks {
            for coord in RingCoordinatesIter::new(active_chunk, ring) {
                // FINITE WORLD: Skip chunks outside bounds
                if !self.is_chunk_in_bounds(coord) {
                    continue;
                }

                let chunk_pos = coord.to_world_pos_with_size(self.chunk_size);
                let distance_squared = active_pos.distance_squared(chunk_pos);

                if distance_squared <= max_distance_squared {
                    if let Some(chunk) = self.get_chunk_mut(coord) {
                        if matches!(chunk.state, ChunkState::Unloaded) {
                            chunk.state = ChunkState::Loading;
                            chunk.distance_to_player = distance_squared.sqrt();
                            chunk.generation_id = chunk.generation_id.wrapping_add(1);
                            to_load.push(coord);
                        }
                    }
                }
            }
        }

        to_load
    }

    // generate_ring_coords replaced with RingCoordinatesIter for zero-allocation iteration

    pub fn clear_placement_grid_for_chunk(&mut self, coord: ChunkCoord) {
        // CRITICAL FIX: Actually clear placement grid entries for this chunk
        let chunk_center = coord.to_world_pos_with_size(self.chunk_size);
        let half_size = self.chunk_size * 0.5;

        // Calculate grid cell range for this chunk
        let min_x = ((chunk_center.x - half_size) / self.placement_grid.cell_size).floor() as i32;
        let max_x = ((chunk_center.x + half_size) / self.placement_grid.cell_size).ceil() as i32;
        let min_z = ((chunk_center.z - half_size) / self.placement_grid.cell_size).floor() as i32;
        let max_z = ((chunk_center.z + half_size) / self.placement_grid.cell_size).ceil() as i32;

        // Remove all grid cells within chunk bounds
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                self.placement_grid.grid.remove(&(x, z));
            }
        }
    }
}

#[derive(Component)]
pub struct UnifiedChunkEntity {
    pub coord: ChunkCoord,
    pub layer: ContentLayer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentLayer {
    Roads,
    Buildings,
    Vehicles,
    NPCs,
}

/// Main unified world streaming system with fixed update intervals
/// Runs at 0.2s intervals instead of every frame for optimal performance
pub fn unified_world_streaming_system(
    mut commands: Commands,
    mut world_manager: ResMut<UnifiedWorldManager>,
    mut road_ownership: ResMut<crate::systems::world::road_network::RoadOwnership>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    time: Res<Time>,
) {
    let Ok(active_transform) = active_query.single() else {
        return;
    };
    let current_time = time.elapsed_secs();

    // PERFORMANCE: Only update every 0.2 seconds, not every frame
    if current_time - world_manager.last_update < 0.2 {
        return;
    }

    // Use direct world coordinates (no coordinate conversion needed)
    let active_pos = active_transform.translation;

    // Update timing
    world_manager.last_update = current_time;
    world_manager.chunks_loaded_this_frame = 0;
    world_manager.chunks_unloaded_this_frame = 0;

    // Update active chunk
    let current_chunk = ChunkCoord::from_world_pos(active_pos, world_manager.chunk_size);
    let _chunk_changed = world_manager.active_chunk != Some(current_chunk);
    world_manager.active_chunk = Some(current_chunk);

    // Cleanup distant chunks
    let chunks_to_unload = world_manager.cleanup_distant_chunks(active_pos);
    for coord in chunks_to_unload {
        if world_manager.chunks_unloaded_this_frame >= world_manager.max_chunks_per_frame {
            break;
        }
        unload_chunk(
            &mut commands,
            &mut world_manager,
            &mut road_ownership,
            coord,
        );
        world_manager.chunks_unloaded_this_frame += 1;
    }

    // Load new chunks
    let chunks_to_load = world_manager.get_chunks_to_load(active_pos);
    for coord in chunks_to_load {
        if world_manager.chunks_loaded_this_frame >= world_manager.max_chunks_per_frame {
            break;
        }
        initiate_chunk_loading(&mut commands, &mut world_manager, coord);
        world_manager.chunks_loaded_this_frame += 1;
    }
}

fn initiate_chunk_loading(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    coord: ChunkCoord,
) {
    // This function starts the chunk loading process
    // The actual content generation will be handled by layer-specific systems
    if let Some(chunk) = world_manager.get_chunk_mut(coord) {
        chunk.state = ChunkState::Loading;
    } else {
        // Chunk is outside finite world bounds - skip loading
        return;
    }

    // CRITICAL FIX: Convert logical chunk position to render space
    let world_pos = coord.to_world_pos_with_size(world_manager.chunk_size);

    // Create a marker entity for this chunk and PARENT TO WORLDROOT
    let chunk_entity = commands
        .spawn((
            UnifiedChunkEntity {
                coord,
                layer: ContentLayer::Roads, // Start with roads
            },
            Transform::from_translation(world_pos),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Add entity to chunk tracking
    if let Some(chunk) = world_manager.get_chunk_mut(coord) {
        chunk.entities.push(chunk_entity);
    }
}

fn unload_chunk(
    commands: &mut Commands,
    world_manager: &mut UnifiedWorldManager,
    road_ownership: &mut crate::systems::world::road_network::RoadOwnership,
    coord: ChunkCoord,
) {
    if let Some(index) = world_manager.chunk_coord_to_index(coord) {
        if let Some(chunk) = &world_manager.chunks[index] {
            for &entity in &chunk.entities {
                commands.entity(entity).despawn();
            }

            world_manager.clear_placement_grid_for_chunk(coord);
        }

        let road_ids = road_ownership.get_roads_for_chunk(coord);
        for road_id in road_ids {
            if let Some((_, entity)) = road_ownership.remove_road(road_id) {
                commands.entity(entity).despawn();
            }
            world_manager.road_network.roads.remove(&road_id);
        }

        world_manager.chunks[index] = None;
    }
}
