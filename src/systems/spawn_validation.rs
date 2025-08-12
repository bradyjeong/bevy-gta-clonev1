use bevy::prelude::*;
use std::collections::HashSet;
use rustc_hash::FxHashMap as HashMap;
use smallvec::SmallVec;
use crate::components::ContentType;


// Universal spawn validation system to prevent entity overlap and collision issues

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpawnableType {
    Vehicle,
    Aircraft,
    NPC,
    Building,
    Tree,
    EnvironmentObject,
    Player,
}

impl SpawnableType {
    /// Convert from unified ContentType to SpawnableType
    pub fn from_content_type(content_type: ContentType) -> Self {
        match content_type {
            ContentType::Vehicle => SpawnableType::Vehicle,
            ContentType::NPC => SpawnableType::NPC,
            ContentType::Building => SpawnableType::Building,
            ContentType::Tree => SpawnableType::Tree,
            ContentType::Road => SpawnableType::EnvironmentObject,
        }
    }
    
    /// Get the minimum clearance radius for this entity type
    pub fn clearance_radius(&self) -> f32 {
        match self {
            SpawnableType::Vehicle => 8.0,      // Car size (increased from 4.0)
            SpawnableType::Aircraft => 16.0,    // Fighter jet/helicopter size (increased from 8.0)
            SpawnableType::NPC => 2.0,          // Human size (increased from 1.0)
            SpawnableType::Building => 20.0,    // Building footprint (increased from 12.0)
            SpawnableType::Tree => 6.0,         // Tree canopy (increased from 3.0)
            SpawnableType::EnvironmentObject => 4.0, // (increased from 2.0)
            SpawnableType::Player => 3.0,       // Player character (increased from 1.5)
        }
    }
    
    /// Get the minimum distance this entity should be from other entities
    pub fn minimum_spacing(&self, other: &SpawnableType) -> f32 {
        // Calculate safe distance as sum of clearance radii plus buffer
        let buffer = match (self, other) {
            // Special cases for important spacing
            (SpawnableType::Player, SpawnableType::Aircraft) => 10.0,
            (SpawnableType::Aircraft, SpawnableType::Player) => 10.0,
            (SpawnableType::Vehicle, SpawnableType::Player) => 5.0,
            (SpawnableType::Player, SpawnableType::Vehicle) => 5.0,
            (SpawnableType::Building, _) => 8.0,
            (_, SpawnableType::Building) => 8.0,
            _ => 3.0, // Default buffer
        };
        
        self.clearance_radius() + other.clearance_radius() + buffer
    }
}

#[derive(Debug, Clone)]
pub struct SpawnedEntity {
    pub position: Vec3,
    pub entity_type: SpawnableType,
    pub entity_id: Entity,
    pub radius: f32,
}

/// Global registry of all spawned entities for collision checking
#[derive(Resource, Default)]
pub struct SpawnRegistry {
    entities: HashMap<Entity, SpawnedEntity>,
    spatial_grid: SpatialGrid,
}

// Precomputed neighbor offsets for different cell ranges
const NEIGHBOR_OFFSETS_1: &[(i32, i32)] = &[
    (0, 0)
];

const NEIGHBOR_OFFSETS_2: &[(i32, i32)] = &[
    (-1, -1), (0, -1), (1, -1),
    (-1, 0), (0, 0), (1, 0),
    (-1, 1), (0, 1), (1, 1),
];

const NEIGHBOR_OFFSETS_3: &[(i32, i32)] = &[
    (-2, -2), (-1, -2), (0, -2), (1, -2), (2, -2),
    (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1),
    (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0),
    (-2, 1), (-1, 1), (0, 1), (1, 1), (2, 1),
    (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2),
];

const NEIGHBOR_OFFSETS_4: &[(i32, i32)] = &[
    (-3, -3), (-2, -3), (-1, -3), (0, -3), (1, -3), (2, -3), (3, -3),
    (-3, -2), (-2, -2), (-1, -2), (0, -2), (1, -2), (2, -2), (3, -2),
    (-3, -1), (-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1), (3, -1),
    (-3, 0), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, 0),
    (-3, 1), (-2, 1), (-1, 1), (0, 1), (1, 1), (2, 1), (3, 1),
    (-3, 2), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2), (3, 2),
    (-3, 3), (-2, 3), (-1, 3), (0, 3), (1, 3), (2, 3), (3, 3),
];

const NEIGHBOR_OFFSETS: [&[(i32, i32)]; 4] = [
    NEIGHBOR_OFFSETS_1,
    NEIGHBOR_OFFSETS_2,
    NEIGHBOR_OFFSETS_3,
    NEIGHBOR_OFFSETS_4,
];

/// Spatial grid for efficient proximity queries
#[derive(Debug, Default)]
struct SpatialGrid {
    #[allow(dead_code)] // Kept for debug output
    grid_size: f32,
    inv_grid_size: f32, // Reciprocal for faster division
    cells: HashMap<u64, SmallVec<[Entity; 8]>>,
}

impl SpatialGrid {
    fn new(grid_size: f32) -> Self {
        Self {
            grid_size,
            inv_grid_size: 1.0 / grid_size,
            cells: HashMap::default(),
        }
    }
    
    #[inline]
    fn pack_key(ix: i32, iz: i32) -> u64 {
        // Swap axes for better hash distribution (both axes affect low bits)
        ((iz as u64) << 32) | (ix as u32 as u64)
    }
    
    #[inline]
    fn get_cell_key(&self, position: Vec3) -> u64 {
        let ix = (position.x * self.inv_grid_size).floor() as i32;
        let iz = (position.z * self.inv_grid_size).floor() as i32;
        Self::pack_key(ix, iz)
    }
    
    #[inline]
    fn get_cell_coords(&self, position: Vec3) -> (i32, i32) {
        (
            (position.x * self.inv_grid_size).floor() as i32,
            (position.z * self.inv_grid_size).floor() as i32,
        )
    }
    
    fn add_entity(&mut self, entity: Entity, position: Vec3) {
        let key = self.get_cell_key(position);
        self.cells.entry(key).or_default().push(entity);
    }
    
    fn remove_entity(&mut self, entity: Entity, position: Vec3) {
        let key = self.get_cell_key(position);
        if let Some(entities) = self.cells.get_mut(&key) {
            entities.retain(|e| *e != entity);
        }
    }
    
    /// Visit nearby entities without allocation
    fn visit_nearby_entities<F>(&self, position: Vec3, radius: f32, mut visitor: F)
    where
        F: FnMut(Entity) -> bool,  // Return false to stop early
    {
        let (cx, cz) = self.get_cell_coords(position);
        let cell_range = ((radius * self.inv_grid_size).ceil() as i32).clamp(1, 4);
        
        // Use precomputed offsets
        let offsets = NEIGHBOR_OFFSETS[(cell_range - 1) as usize];
        
        for &(dx, dz) in offsets {
            let key = Self::pack_key(cx + dx, cz + dz);
            if let Some(entities) = self.cells.get(&key) {
                for &entity in entities {
                    if !visitor(entity) {
                        return; // Early exit if visitor returns false
                    }
                }
            }
        }
    }
    
    fn get_nearby_entities(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        // Pre-allocate with estimated capacity to reduce reallocations
        let mut result = Vec::with_capacity(32);
        self.visit_nearby_entities(position, radius, |entity| {
            result.push(entity);
            true // Continue collecting
        });
        result
    }
}

impl SpawnRegistry {
    pub fn new() -> Self {
        Self {
            entities: HashMap::default(),
            spatial_grid: SpatialGrid::new(40.0), // 40 unit grid cells (fewer neighbor checks)
        }
    }
    
    /// Register a newly spawned entity
    pub fn register_entity(&mut self, entity: Entity, position: Vec3, entity_type: SpawnableType) {
        let radius = entity_type.clearance_radius();
        let spawned_entity = SpawnedEntity {
            position,
            entity_type,
            entity_id: entity,
            radius,
        };
        
        info!("🎯 SPAWN REGISTRY: Registered {:?} at {:?} (radius: {:.1})", entity_type, position, radius);
        
        self.spatial_grid.add_entity(entity, position);
        self.entities.insert(entity, spawned_entity);
    }
    
    /// Remove an entity from the registry
    pub fn unregister_entity(&mut self, entity: Entity) {
        if let Some(spawned_entity) = self.entities.remove(&entity) {
            self.spatial_grid.remove_entity(entity, spawned_entity.position);
        }
    }
    
    /// Update an entity's position in the registry
    pub fn update_entity_position(&mut self, entity: Entity, new_position: Vec3) {
        if let Some(spawned_entity) = self.entities.get_mut(&entity) {
            self.spatial_grid.remove_entity(entity, spawned_entity.position);
            spawned_entity.position = new_position;
            self.spatial_grid.add_entity(entity, new_position);
        }
    }
    
    /// Check if a position is safe for spawning the given entity type
    pub fn is_position_safe(&self, position: Vec3, entity_type: SpawnableType) -> bool {
        let search_radius = entity_type.clearance_radius() + 15.0; // Extended search
        
        let mut is_safe = true;
        let mut entity_count = 0;
        
        // Use visitor pattern for early exit without allocation
        self.spatial_grid.visit_nearby_entities(position, search_radius, |nearby_entity| {
            entity_count += 1;
            
            if let Some(spawned_entity) = self.entities.get(&nearby_entity) {
                let required_distance = entity_type.minimum_spacing(&spawned_entity.entity_type);
                // Use distance_squared for more efficient comparison
                let actual_distance_sq = position.distance_squared(spawned_entity.position);
                let required_distance_sq = required_distance * required_distance;
                
                if actual_distance_sq < required_distance_sq {
                    #[cfg(debug_assertions)]
                    {
                        let actual_distance = actual_distance_sq.sqrt();
                        debug!("❌ COLLISION: {:?} at {:?} too close to {:?} at {:?} (distance: {:.1} < {:.1})",
                              entity_type, position, spawned_entity.entity_type, spawned_entity.position,
                              actual_distance, required_distance);
                    }
                    is_safe = false;
                    return false; // Stop checking, we found a collision
                }
            }
            true // Continue checking
        });
        
        debug!("🔍 SPAWN CHECK: Checked {:?} at {:?} against {} nearby entities", entity_type, position, entity_count);
        
        if is_safe {
            debug!("✅ SAFE: Position {:?} is safe for {:?}", position, entity_type);
        }
        
        is_safe
    }
    
    /// Find the nearest safe spawn position within a search area
    pub fn find_safe_spawn_position(
        &self,
        preferred_position: Vec3,
        entity_type: SpawnableType,
        max_search_radius: f32,
        max_attempts: u32,
    ) -> Option<Vec3> {
        // First try the preferred position
        if self.is_position_safe(preferred_position, entity_type) {
            return Some(preferred_position);
        }
        
        // Use spiral search pattern for better results
        for attempt in 0..max_attempts {
            let angle = (attempt as f32) * 2.39996; // Golden angle for even distribution
            let distance = (attempt as f32 / max_attempts as f32) * max_search_radius;
            
            let offset = Vec3::new(
                angle.cos() * distance,
                0.0,
                angle.sin() * distance,
            );
            
            let test_position = preferred_position + offset;
            
            // Validate Y position (ground level)
            let ground_level_position = Vec3::new(test_position.x, preferred_position.y, test_position.z);
            
            if self.is_position_safe(ground_level_position, entity_type) {
                return Some(ground_level_position);
            }
        }
        
        None
    }
    
    /// Get all entities within a radius of a position
    pub fn get_entities_in_radius(&self, position: Vec3, radius: f32) -> Vec<&SpawnedEntity> {
        let nearby_entities = self.spatial_grid.get_nearby_entities(position, radius);
        nearby_entities
            .iter()
            .filter_map(|&entity| self.entities.get(&entity))
            .filter(|entity| entity.position.distance(position) <= radius)
            .collect()
    }
}

/// Spawn request queue for throttling validation
#[derive(Resource, Default)]
pub struct SpawnRequestQueue {
    requests: Vec<SpawnRequest>,
}

#[derive(Debug, Clone)]
pub struct SpawnRequest {
    pub position: Vec3,
    pub entity_type: SpawnableType,
    pub entity: Entity,
}

/// Plugin to add spawn validation system
pub struct SpawnValidationPlugin;

impl Plugin for SpawnValidationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnRegistry::new())
            .insert_resource(SpawnRequestQueue::default())
            .insert_resource(Time::<Fixed>::from_hz(10.0)) // Run at 10 Hz
            .add_systems(FixedUpdate, process_spawn_requests)
            .add_systems(Update, cleanup_despawned_entities);
    }
}

/// Process spawn requests with frame cap to prevent stalls
fn process_spawn_requests(
    mut registry: ResMut<SpawnRegistry>,
    mut queue: ResMut<SpawnRequestQueue>,
    mut commands: Commands,
) {
    const MAX_VALIDATIONS_PER_FRAME: usize = 20;
    
    let mut processed = 0;
    let mut remaining_requests = Vec::new();
    
    for request in queue.requests.drain(..) {
        if processed >= MAX_VALIDATIONS_PER_FRAME {
            remaining_requests.push(request);
            continue;
        }
        
        // Try to find safe position
        if let Some(safe_position) = registry.find_safe_spawn_position(
            request.position,
            request.entity_type,
            30.0, // Max search radius
            10,   // Reduced attempts for performance
        ) {
            registry.register_entity(request.entity, safe_position, request.entity_type);
            // Entity spawning handled by caller
        } else {
            warn!(
                "Failed to find safe spawn position for {:?} near {:?}",
                request.entity_type, request.position
            );
            // Despawn the entity if we can't place it
            commands.entity(request.entity).despawn();
        }
        
        processed += 1;
    }
    
    // Put remaining requests back in queue
    queue.requests = remaining_requests;
}

/// System to clean up registry when entities are despawned
/// Optimized to run less frequently and avoid querying all entities
fn cleanup_despawned_entities(
    mut registry: ResMut<SpawnRegistry>,
    query: Query<Entity>,
    time: Res<Time>,
    mut cleanup_timer: Local<f32>,
) {
    // Early exit if registry is empty
    if registry.entities.is_empty() {
        return;
    }
    
    // Only run cleanup every 2 seconds to reduce overhead
    *cleanup_timer += time.delta_secs();
    if *cleanup_timer < 2.0 {
        return;
    }
    *cleanup_timer = 0.0;
    
    // Only check a batch of entities per frame to spread the work
    const BATCH_SIZE: usize = 50;
    let start_index = (registry.entities.len() / BATCH_SIZE) % registry.entities.len().max(1);
    
    let valid_entities: HashSet<Entity> = query.iter().collect();
    
    // Process only a batch of entities
    let entities_to_remove: Vec<(Entity, Vec3)> = registry
        .entities
        .iter()
        .skip(start_index)
        .take(BATCH_SIZE)
        .filter_map(|(&entity, spawned_entity)| {
            if !valid_entities.contains(&entity) {
                Some((entity, spawned_entity.position))
            } else {
                None
            }
        })
        .collect();
    
    // Remove them from both data structures
    for (entity, position) in entities_to_remove {
        registry.entities.remove(&entity);
        registry.spatial_grid.remove_entity(entity, position);
    }
}

/// Helper functions for easy spawn validation
pub struct SpawnValidator;

impl SpawnValidator {
    /// Validate and register a spawn location
    pub fn spawn_entity_safely(
        registry: &mut ResMut<SpawnRegistry>,
        preferred_position: Vec3,
        entity_type: SpawnableType,
        entity: Entity,
    ) -> Option<Vec3> {
        if let Some(safe_position) = registry.find_safe_spawn_position(
            preferred_position,
            entity_type,
            30.0, // Max search radius
            20,   // Max attempts - reduced for performance
        ) {
            registry.register_entity(entity, safe_position, entity_type);
            Some(safe_position)
        } else {
            warn!(
                "Failed to find safe spawn position for {:?} near {:?}",
                entity_type, preferred_position
            );
            None
        }
    }
    
    /// Quick check if a position is clear
    pub fn is_clear(
        registry: &SpawnRegistry,
        position: Vec3,
        entity_type: SpawnableType,
    ) -> bool {
        registry.is_position_safe(position, entity_type)
    }
}
