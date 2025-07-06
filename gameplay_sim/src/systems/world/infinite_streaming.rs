use bevy::prelude::*;
use std::collections::{HashMap, VecDeque, BTreeMap};
use game_core::components::*;
use game_core::config::GameConfig;
use crate::systems::world::road_network::RoadNetwork;
use crate::systems::world::road_generation::is_on_road_spline;

/// Revolutionary infinite world streaming system
/// Scales from 10km² to 1,000km² seamlessly with hierarchical LOD management

/// World scale constants - 100x increase from current
pub const MACRO_REGION_SIZE: f32 = 10000.0;  // 10km macro regions
pub const REGION_SIZE: f32 = 2000.0;         // 2km regions  
pub const LOCAL_CHUNK_SIZE: f32 = 400.0;     // 400m local chunks
pub const DETAIL_CHUNK_SIZE: f32 = 100.0;    // 100m detail chunks
pub const MICRO_CHUNK_SIZE: f32 = 25.0;      // 25m micro chunks

/// Streaming distances for each LOD level
pub const MACRO_STREAMING_RADIUS: f32 = 50000.0;  // 50km macro visibility
pub const REGION_STREAMING_RADIUS: f32 = 20000.0;  // 20km region visibility  
pub const LOCAL_STREAMING_RADIUS: f32 = 5000.0;    // 5km local visibility
pub const DETAIL_STREAMING_RADIUS: f32 = 2000.0;   // 2km detail visibility
pub const MICRO_STREAMING_RADIUS: f32 = 500.0;     // 500m micro visibility

/// Maximum entities per frame for smooth streaming - REDUCED FOR PERFORMANCE
pub const MAX_CHUNKS_LOADED_PER_FRAME: usize = 2;  // REDUCED: From 6 to 2
pub const MAX_CHUNKS_UNLOADED_PER_FRAME: usize = 4;  // REDUCED: From 8 to 4
pub const MAX_CONTENT_GENERATED_PER_FRAME: usize = 5;  // REDUCED: From 10 to 5

/// Hierarchical coordinate system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldCoord {
    pub level: LODLevel,
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LODLevel {
    Macro = 0,    // 10km regions - procedural generation only
    Region = 1,   // 2km regions - basic terrain and roads
    Local = 2,    // 400m chunks - buildings and major features
    Detail = 3,   // 100m chunks - detailed objects and NPCs
    Micro = 4,    // 25m chunks - full detail with physics
}

impl WorldCoord {
    pub fn new(level: LODLevel, x: i32, z: i32) -> Self {
        Self { level, x, z }
    }
    
    pub fn from_world_pos(world_pos: Vec3, level: LODLevel) -> Self {
        let chunk_size = match level {
            LODLevel::Macro => MACRO_REGION_SIZE,
            LODLevel::Region => REGION_SIZE,
            LODLevel::Local => LOCAL_CHUNK_SIZE,
            LODLevel::Detail => DETAIL_CHUNK_SIZE,
            LODLevel::Micro => MICRO_CHUNK_SIZE,
        };
        
        Self {
            level,
            x: (world_pos.x / chunk_size).floor() as i32,
            z: (world_pos.z / chunk_size).floor() as i32,
        }
    }
    
    pub fn to_world_pos(&self) -> Vec3 {
        let chunk_size = match self.level {
            LODLevel::Macro => MACRO_REGION_SIZE,
            LODLevel::Region => REGION_SIZE,
            LODLevel::Local => LOCAL_CHUNK_SIZE,
            LODLevel::Detail => DETAIL_CHUNK_SIZE,
            LODLevel::Micro => MICRO_CHUNK_SIZE,
        };
        
        Vec3::new(
            self.x as f32 * chunk_size + chunk_size * 0.5,
            0.0,
            self.z as f32 * chunk_size + chunk_size * 0.5,
        )
    }
    
    pub fn get_streaming_radius(&self) -> f32 {
        match self.level {
            LODLevel::Macro => MACRO_STREAMING_RADIUS,
            LODLevel::Region => REGION_STREAMING_RADIUS,
            LODLevel::Local => LOCAL_STREAMING_RADIUS,
            LODLevel::Detail => DETAIL_STREAMING_RADIUS,
            LODLevel::Micro => MICRO_STREAMING_RADIUS,
        }
    }
    
    /// Get parent coordinate at next higher LOD level
    pub fn get_parent(&self) -> Option<WorldCoord> {
        match self.level {
            LODLevel::Macro => None,
            LODLevel::Region => Some(WorldCoord::new(LODLevel::Macro, self.x / 5, self.z / 5)),
            LODLevel::Local => Some(WorldCoord::new(LODLevel::Region, self.x / 5, self.z / 5)),
            LODLevel::Detail => Some(WorldCoord::new(LODLevel::Local, self.x / 4, self.z / 4)),
            LODLevel::Micro => Some(WorldCoord::new(LODLevel::Detail, self.x / 4, self.z / 4)),
        }
    }
    
    /// Get children coordinates at next lower LOD level
    pub fn get_children(&self) -> Vec<WorldCoord> {
        let mut children = Vec::new();
        let (subdivisions, child_level) = match self.level {
            LODLevel::Macro => (5, LODLevel::Region),
            LODLevel::Region => (5, LODLevel::Local), 
            LODLevel::Local => (4, LODLevel::Detail),
            LODLevel::Detail => (4, LODLevel::Micro),
            LODLevel::Micro => return children, // No children for micro level
        };
        
        for dx in 0..subdivisions {
            for dz in 0..subdivisions {
                children.push(WorldCoord::new(
                    child_level,
                    self.x * subdivisions + dx,
                    self.z * subdivisions + dz,
                ));
            }
        }
        
        children
    }
}

/// Chunk state with generation priorities
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkState {
    Unloaded,
    PendingLoad { priority: f32 },
    Loading { progress: f32 },
    Loaded { entities: Vec<Entity> },
    PendingUnload,
    Unloading,
}

/// Chunk data with hierarchical management
#[derive(Debug, Clone)]
pub struct WorldChunk {
    pub coord: WorldCoord,
    pub state: ChunkState,
    pub distance_to_active: f32,
    pub last_accessed: f32,
    pub parent: Option<WorldCoord>,
    pub children: Vec<WorldCoord>,
    pub content_layers: ContentLayers,
    pub generation_seed: u64,
}

/// Content layer flags for different types
#[derive(Debug, Clone, Default)]
pub struct ContentLayers {
    pub terrain: bool,
    pub roads: bool,
    pub buildings: bool,
    pub vegetation: bool,
    pub vehicles: bool,
    pub npcs: bool,
    pub water: bool,
    pub details: bool,
}

impl WorldChunk {
    pub fn new(coord: WorldCoord, current_time: f32) -> Self {
        // Generate deterministic seed from coordinates
        let generation_seed = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&(coord.level as u8, coord.x, coord.z), &mut hasher);
            std::hash::Hasher::finish(&hasher)
        };
        
        Self {
            coord,
            state: ChunkState::Unloaded,
            distance_to_active: f32::INFINITY,
            last_accessed: current_time,
            parent: coord.get_parent(),
            children: coord.get_children(),
            content_layers: ContentLayers::default(),
            generation_seed,
        }
    }
    
    pub fn get_generation_priority(&self) -> f32 {
        // Higher priority = loaded first
        let distance_factor = 1.0 / (self.distance_to_active + 1.0);
        let lod_factor = match self.coord.level {
            LODLevel::Micro => 5.0,
            LODLevel::Detail => 4.0,  
            LODLevel::Local => 3.0,
            LODLevel::Region => 2.0,
            LODLevel::Macro => 1.0,
        };
        
        distance_factor * lod_factor
    }
}

/// Advanced world streaming manager with hierarchical LOD
#[derive(Resource)]
pub struct WorldLODManager {
    pub chunks: HashMap<WorldCoord, WorldChunk>,
    pub active_position: Vec3,
    pub active_coords: BTreeMap<LODLevel, WorldCoord>,
    
    // Streaming queues with priority
    pub load_queue: VecDeque<WorldCoord>,
    pub unload_queue: VecDeque<WorldCoord>,
    pub generation_queue: VecDeque<(WorldCoord, f32)>, // (coord, priority)
    
    // Performance tracking
    pub chunks_loaded_this_frame: usize,
    pub chunks_unloaded_this_frame: usize,
    pub content_generated_this_frame: usize,
    pub last_update_time: f32,
    
    // Memory management
    pub max_loaded_chunks: HashMap<LODLevel, usize>,
    pub loaded_chunk_count: HashMap<LODLevel, usize>,
}

impl Default for WorldLODManager {
    fn default() -> Self {
        let mut max_loaded_chunks = HashMap::new();
        max_loaded_chunks.insert(LODLevel::Macro, 100);     // 100 macro regions (100km²)
        max_loaded_chunks.insert(LODLevel::Region, 500);    // 500 regions (2000km²) 
        max_loaded_chunks.insert(LODLevel::Local, 1000);    // 1000 local chunks (160km²)
        max_loaded_chunks.insert(LODLevel::Detail, 2000);   // 2000 detail chunks (20km²)
        max_loaded_chunks.insert(LODLevel::Micro, 3000);    // 3000 micro chunks (1.9km²)
        
        let mut loaded_chunk_count = HashMap::new();
        for level in [LODLevel::Macro, LODLevel::Region, LODLevel::Local, LODLevel::Detail, LODLevel::Micro] {
            loaded_chunk_count.insert(level, 0);
        }
        
        Self {
            chunks: HashMap::new(),
            active_position: Vec3::ZERO,
            active_coords: BTreeMap::new(),
            load_queue: VecDeque::new(),
            unload_queue: VecDeque::new(),
            generation_queue: VecDeque::new(),
            chunks_loaded_this_frame: 0,
            chunks_unloaded_this_frame: 0,
            content_generated_this_frame: 0,
            last_update_time: 0.0,
            max_loaded_chunks,
            loaded_chunk_count,
        }
    }
}

impl WorldLODManager {
    pub fn update_active_position(&mut self, position: Vec3, current_time: f32) {
        self.active_position = position;
        self.last_update_time = current_time;
        
        // Update active coordinates for each LOD level
        for level in [LODLevel::Macro, LODLevel::Region, LODLevel::Local, LODLevel::Detail, LODLevel::Micro] {
            let coord = WorldCoord::from_world_pos(position, level);
            self.active_coords.insert(level, coord);
        }
        
        // Update distances for all chunks
        for chunk in self.chunks.values_mut() {
            chunk.distance_to_active = position.distance(chunk.coord.to_world_pos());
            chunk.last_accessed = current_time;
        }
    }
    
    pub fn should_load_chunk(&self, coord: WorldCoord) -> bool {
        if self.chunks.contains_key(&coord) {
            return false; // Already exists
        }
        
        let distance = self.active_position.distance(coord.to_world_pos());
        distance <= coord.get_streaming_radius()
    }
    
    pub fn should_unload_chunk(&self, coord: WorldCoord, current_time: f32) -> bool {
        if let Some(chunk) = self.chunks.get(&coord) {
            let distance = self.active_position.distance(coord.to_world_pos());
            let streaming_radius = coord.get_streaming_radius();
            
            // Unload if outside streaming radius with hysteresis
            if distance > streaming_radius * 1.2 {
                return true;
            }
            
            // Unload if not accessed recently and memory pressure
            let loaded_count = self.loaded_chunk_count.get(&coord.level).unwrap_or(&0);
            let max_count = self.max_loaded_chunks.get(&coord.level).unwrap_or(&1000);
            
            if loaded_count > max_count && (current_time - chunk.last_accessed) > 30.0 {
                return true;
            }
        }
        
        false
    }
    
    pub fn get_chunks_to_load(&mut self) -> Vec<WorldCoord> {
        let mut candidates = Vec::new();
        
        // Generate candidates for each LOD level around active position
        for level in [LODLevel::Macro, LODLevel::Region, LODLevel::Local, LODLevel::Detail, LODLevel::Micro] {
            if let Some(active_coord) = self.active_coords.get(&level) {
                let streaming_radius = level as i32 + 2; // Radius in chunks
                
                for dx in -streaming_radius..=streaming_radius {
                    for dz in -streaming_radius..=streaming_radius {
                        let coord = WorldCoord::new(level, active_coord.x + dx, active_coord.z + dz);
                        
                        if self.should_load_chunk(coord) {
                            candidates.push(coord);
                        }
                    }
                }
            }
        }
        
        // Sort by priority (distance and LOD level)
        candidates.sort_by(|a, b| {
            let dist_a = self.active_position.distance(a.to_world_pos());
            let dist_b = self.active_position.distance(b.to_world_pos());
            let priority_a = 1.0 / (dist_a + 1.0) * (a.level as u8 + 1) as f32;
            let priority_b = 1.0 / (dist_b + 1.0) * (b.level as u8 + 1) as f32;
            priority_b.partial_cmp(&priority_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        candidates.into_iter().take(MAX_CHUNKS_LOADED_PER_FRAME).collect()
    }
    
    pub fn get_chunks_to_unload(&mut self, current_time: f32) -> Vec<WorldCoord> {
        let mut candidates: Vec<_> = self.chunks.keys()
            .filter(|coord| self.should_unload_chunk(**coord, current_time))
            .cloned()
            .collect();
        
        // Sort by priority (distance and last access time)
        candidates.sort_by(|a, b| {
            let chunk_a = &self.chunks[a];
            let chunk_b = &self.chunks[b];
            
            let priority_a = chunk_a.distance_to_active + (current_time - chunk_a.last_accessed);
            let priority_b = chunk_b.distance_to_active + (current_time - chunk_b.last_accessed);
            
            priority_b.partial_cmp(&priority_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        candidates.into_iter().take(MAX_CHUNKS_UNLOADED_PER_FRAME).collect()
    }
    
    pub fn mark_chunk_for_loading(&mut self, coord: WorldCoord, current_time: f32) {
        let chunk = WorldChunk::new(coord, current_time);
        let priority = chunk.get_generation_priority();
        
        self.chunks.insert(coord, chunk);
        self.generation_queue.push_back((coord, priority));
        
        // Sort generation queue by priority
        self.generation_queue.make_contiguous().sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    pub fn mark_chunk_for_unloading(&mut self, coord: WorldCoord) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.state = ChunkState::PendingUnload;
            self.unload_queue.push_back(coord);
        }
    }
    
    pub fn get_next_chunk_to_generate(&mut self) -> Option<WorldCoord> {
        while let Some((coord, _priority)) = self.generation_queue.pop_front() {
            if let Some(chunk) = self.chunks.get(&coord) {
                if matches!(chunk.state, ChunkState::Unloaded) {
                    return Some(coord);
                }
            }
        }
        None
    }
    
    pub fn finalize_chunk_loading(&mut self, coord: WorldCoord, entities: Vec<Entity>) {
        if let Some(chunk) = self.chunks.get_mut(&coord) {
            chunk.state = ChunkState::Loaded { entities };
            
            // Update loaded count
            let count = self.loaded_chunk_count.entry(coord.level).or_insert(0);
            *count += 1;
        }
    }
    
    pub fn unload_chunk(&mut self, coord: WorldCoord) -> Option<Vec<Entity>> {
        if let Some(chunk) = self.chunks.remove(&coord) {
            // Update loaded count
            let count = self.loaded_chunk_count.entry(coord.level).or_insert(0);
            *count = count.saturating_sub(1);
            
            match chunk.state {
                ChunkState::Loaded { entities } => Some(entities),
                _ => None,
            }
        } else {
            None
        }
    }
    
    pub fn reset_frame_counters(&mut self) {
        self.chunks_loaded_this_frame = 0;
        self.chunks_unloaded_this_frame = 0;
        self.content_generated_this_frame = 0;
    }
    
    pub fn get_memory_usage(&self) -> WorldMemoryUsage {
        let mut usage = WorldMemoryUsage::default();
        
        for (level, count) in &self.loaded_chunk_count {
            let max_count = self.max_loaded_chunks.get(level).unwrap_or(&1000);
            match level {
                LODLevel::Macro => {
                    usage.macro_chunks = *count;
                    usage.macro_max = *max_count;
                }
                LODLevel::Region => {
                    usage.region_chunks = *count;
                    usage.region_max = *max_count;
                }
                LODLevel::Local => {
                    usage.local_chunks = *count;
                    usage.local_max = *max_count;
                }
                LODLevel::Detail => {
                    usage.detail_chunks = *count;
                    usage.detail_max = *max_count;
                }
                LODLevel::Micro => {
                    usage.micro_chunks = *count;
                    usage.micro_max = *max_count;
                }
            }
        }
        
        usage.total_chunks = self.chunks.len();
        usage.load_queue_size = self.load_queue.len();
        usage.unload_queue_size = self.unload_queue.len();
        usage.generation_queue_size = self.generation_queue.len();
        
        usage
    }
}

/// Memory usage tracking for diagnostics
#[derive(Debug, Default)]
pub struct WorldMemoryUsage {
    pub macro_chunks: usize,
    pub macro_max: usize,
    pub region_chunks: usize,
    pub region_max: usize,
    pub local_chunks: usize,
    pub local_max: usize,
    pub detail_chunks: usize,
    pub detail_max: usize,
    pub micro_chunks: usize,
    pub micro_max: usize,
    pub total_chunks: usize,
    pub load_queue_size: usize,
    pub unload_queue_size: usize,
    pub generation_queue_size: usize,
}

/// Main infinite world streaming system
pub fn infinite_world_streaming_system(
    mut commands: Commands,
    mut world_manager: ResMut<WorldLODManager>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let Ok(active_transform) = active_query.single() else { return };
    let current_time = time.elapsed_secs();
    
    // Reset frame counters
    world_manager.reset_frame_counters();
    
    // Update active position and chunk coordinates
    world_manager.update_active_position(active_transform.translation, current_time);
    
    // Unload distant chunks first to free memory
    let chunks_to_unload = world_manager.get_chunks_to_unload(current_time);
    for coord in chunks_to_unload {
        if world_manager.chunks_unloaded_this_frame >= MAX_CHUNKS_UNLOADED_PER_FRAME {
            break;
        }
        
        if let Some(entities) = world_manager.unload_chunk(coord) {
            // Despawn all entities in the chunk
            for entity in entities {
                commands.entity(entity).despawn();
            }
            world_manager.chunks_unloaded_this_frame += 1;
        }
    }
    
    // Load new chunks within streaming radius
    let chunks_to_load = world_manager.get_chunks_to_load();
    for coord in chunks_to_load {
        if world_manager.chunks_loaded_this_frame >= MAX_CHUNKS_LOADED_PER_FRAME {
            break;
        }
        
        world_manager.mark_chunk_for_loading(coord, current_time);
        world_manager.chunks_loaded_this_frame += 1;
    }
    
    // Process generation queue
    while world_manager.content_generated_this_frame < MAX_CONTENT_GENERATED_PER_FRAME {
        if let Some(coord) = world_manager.get_next_chunk_to_generate() {
            initiate_chunk_generation(&mut commands, &mut world_manager, coord, &road_network, &config);
            world_manager.content_generated_this_frame += 1;
        } else {
            break;
        }
    }
}

/// Initiate chunk content generation based on LOD level
fn initiate_chunk_generation(
    commands: &mut Commands,
    world_manager: &mut WorldLODManager,
    coord: WorldCoord,
    road_network: &RoadNetwork,
    _config: &GameConfig,
) {
    let chunk_center = coord.to_world_pos();
    let mut entities = Vec::new();
    
    // Generate content based on LOD level
    match coord.level {
        LODLevel::Macro => {
            // Macro level: Procedural heightmap and biome data only
            // No actual entities, just mark as loaded
        }
        LODLevel::Region => {
            // Region level: Basic terrain features and major roads
            let road_entity = commands.spawn((
                Name::new(format!("Region_Road_{}_{}_{}", coord.level as u8, coord.x, coord.z)),
                Transform::from_translation(chunk_center),
                Visibility::Visible,
                InheritedVisibility::VISIBLE,
                ViewVisibility::default(),
            )).id();
            entities.push(road_entity);
        }
        LODLevel::Local => {
            // Local level: Buildings, major vegetation, secondary roads
            for i in 0..3 {
                let building_pos = chunk_center + Vec3::new(
                    (i as f32 - 1.0) * 100.0,
                    0.0,
                    0.0,
                );
                
                // Only spawn building if not on road
                if is_on_road_spline(building_pos, road_network, 25.0) {
                    println!("DEBUG: INFINITE_STREAMING - Skipping building at {:?} - on road", building_pos);
                } else {
                    println!("DEBUG: INFINITE_STREAMING - Spawning building at {:?}", building_pos);
                    let building_entity = commands.spawn((
                        Name::new(format!("Local_Building_{}_{}_{}_{}", coord.level as u8, coord.x, coord.z, i)),
                        Transform::from_translation(building_pos),
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ViewVisibility::default(),
                    )).id();
                    entities.push(building_entity);
                }
            }
        }
        LODLevel::Detail => {
            // Detail level: Detailed objects, vehicles, some NPCs
            for i in 0..2 {
                let vehicle_pos = chunk_center + Vec3::new(
                    (i as f32 - 0.5) * 50.0,
                    0.0,
                    25.0,
                );
                
                let vehicle_entity = commands.spawn((
                    Name::new(format!("Detail_Vehicle_{}_{}_{}_{}", coord.level as u8, coord.x, coord.z, i)),
                    Transform::from_translation(vehicle_pos),
                    Visibility::Visible,
                    InheritedVisibility::VISIBLE,
                    ViewVisibility::default(),
                )).id();
                entities.push(vehicle_entity);
            }
        }
        LODLevel::Micro => {
            // Micro level: Full detail with physics, NPCs, interactive objects
            for i in 0..4 {
                let detail_pos = chunk_center + Vec3::new(
                    (i % 2) as f32 * 15.0 - 7.5,
                    0.0,
                    (i / 2) as f32 * 15.0 - 7.5,
                );
                
                let detail_entity = commands.spawn((
                    Name::new(format!("Micro_Detail_{}_{}_{}_{}", coord.level as u8, coord.x, coord.z, i)),
                    Transform::from_translation(detail_pos),
                    Visibility::Visible,
                    InheritedVisibility::VISIBLE,
                    ViewVisibility::default(),
                )).id();
                entities.push(detail_entity);
            }
        }
    }
    
    // Mark chunk as loaded
    world_manager.finalize_chunk_loading(coord, entities);
}

/// Debug system to display world streaming status
pub fn infinite_world_debug_system(
    world_manager: Res<WorldLODManager>,
    mut commands: Commands,
    time: Res<Time>,
) {
    // Only update debug info every second to avoid spam
    if (time.elapsed_secs() % 1.0) < time.delta_secs() {
        let usage = world_manager.get_memory_usage();
        
        info!(
            "🌍 INFINITE WORLD STATUS:\n\
            📊 Memory Usage:\n\
            • Macro:  {}/{} regions\n\
            • Region: {}/{} chunks\n\
            • Local:  {}/{} chunks\n\
            • Detail: {}/{} chunks\n\
            • Micro:  {}/{} chunks\n\
            📈 Queue Status:\n\
            • Load Queue: {} chunks\n\
            • Unload Queue: {} chunks\n\
            • Generation Queue: {} chunks\n\
            🎯 Active Position: {:.1}, {:.1}, {:.1}",
            usage.macro_chunks, usage.macro_max,
            usage.region_chunks, usage.region_max,
            usage.local_chunks, usage.local_max,
            usage.detail_chunks, usage.detail_max,
            usage.micro_chunks, usage.micro_max,
            usage.load_queue_size,
            usage.unload_queue_size,
            usage.generation_queue_size,
            world_manager.active_position.x,
            world_manager.active_position.y,
            world_manager.active_position.z
        );
    }
}
