use bevy::prelude::*;
use std::collections::HashMap;
use crate::systems::distance_cache::DistanceCache;
use engine_bevy::prelude::{UnifiedPerformanceTracker, PerformanceCategory};

/// Unified distance calculator that batches calculations and leverages caching
#[derive(Resource)]
pub struct UnifiedDistanceCalculator {
    /// Batch accumulator for distance requests
    distance_requests: Vec<DistanceRequest>,
    /// Cached results for this frame
    frame_cache: HashMap<(Entity, Entity), f32>,
    /// Reference position (usually player position)
    reference_position: Option<Vec3>,
    /// Performance metrics
    batch_count: u32,
    individual_count: u32,
}

/// A distance calculation request
struct DistanceRequest {
    entity1: Entity,
    entity2: Entity,
    pos1: Vec3,
    pos2: Vec3,
    _callback_id: u32,
}

/// Result of a distance calculation
#[derive(Debug, Clone, Copy)]
pub struct DistanceResult {
    pub distance: f32,
    pub distance_squared: f32,
}

impl Default for UnifiedDistanceCalculator {
    fn default() -> Self {
        Self {
            distance_requests: Vec::with_capacity(512),
            frame_cache: HashMap::with_capacity(256),
            reference_position: None,
            batch_count: 0,
            individual_count: 0,
        }
    }
}

impl UnifiedDistanceCalculator {
    /// Add a distance calculation request to the batch
    pub fn request_distance(&mut self, entity1: Entity, entity2: Entity, pos1: Vec3, pos2: Vec3) -> u32 {
        let callback_id = self.distance_requests.len() as u32;
        self.distance_requests.push(DistanceRequest {
            entity1,
            entity2,
            pos1,
            pos2,
            _callback_id: callback_id,
        });
        callback_id
    }

    /// Get immediate distance (bypass batching for critical calculations)
    pub fn get_immediate_distance(&mut self, entity1: Entity, entity2: Entity, pos1: Vec3, pos2: Vec3) -> f32 {
        let key = Self::make_key(entity1, entity2);
        
        // Check frame cache first
        if let Some(&distance) = self.frame_cache.get(&key) {
            return distance;
        }
        
        // Calculate and cache
        let distance = pos1.distance(pos2);
        self.frame_cache.insert(key, distance);
        self.individual_count += 1;
        distance
    }

    /// Get distance to reference position (typically player position)
    pub fn get_distance_to_reference(&mut self, entity: Entity, pos: Vec3) -> Option<f32> {
        self.reference_position.map(|ref_pos| {
            self.get_immediate_distance(entity, Entity::PLACEHOLDER, pos, ref_pos)
        })
    }

    /// Set the reference position for efficient reference-based calculations
    pub fn set_reference_position(&mut self, position: Vec3) {
        self.reference_position = Some(position);
    }

    /// Process all batched distance requests
    pub fn process_batch(
        &mut self,
        distance_cache: &mut DistanceCache,
        performance_tracker: &mut UnifiedPerformanceTracker,
    ) -> Vec<DistanceResult> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(self.distance_requests.len());
        
        // Sort requests by entity pairs to maximize cache efficiency
        self.distance_requests.sort_by_key(|req| Self::make_key(req.entity1, req.entity2));
        
        // Process requests directly without intermediate batching to avoid borrowing issues
        for request in self.distance_requests.drain(..) {
            let (distance, distance_squared) = distance_cache.get_distance(
                request.entity1,
                request.entity2,
                request.pos1,
                request.pos2,
            );
            
            results.push(DistanceResult {
                distance,
                distance_squared,
            });
        }
        
        self.batch_count += 1;
        
        // Update performance metrics
        let processing_time = start_time.elapsed().as_micros() as f32 / 1000.0;
        performance_tracker.record_category_time(PerformanceCategory::System, processing_time);
        
        // Clear frame cache for next frame
        self.frame_cache.clear();
        
        results
    }



    /// Create a consistent cache key for entity pairs
    fn make_key(entity1: Entity, entity2: Entity) -> (Entity, Entity) {
        if entity1.index() < entity2.index() {
            (entity1, entity2)
        } else {
            (entity2, entity1)
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> DistanceCalculatorStats {
        DistanceCalculatorStats {
            batch_count: self.batch_count,
            individual_count: self.individual_count,
            frame_cache_size: self.frame_cache.len(),
            pending_requests: self.distance_requests.len(),
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.batch_count = 0;
        self.individual_count = 0;
    }
}

/// Statistics for the distance calculator
#[derive(Debug, Clone)]
pub struct DistanceCalculatorStats {
    pub batch_count: u32,
    pub individual_count: u32,
    pub frame_cache_size: usize,
    pub pending_requests: usize,
}

/// System to process distance calculation batches
pub fn unified_distance_processing_system(
    mut distance_calculator: ResMut<UnifiedDistanceCalculator>,
    mut distance_cache: ResMut<DistanceCache>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
) {
    distance_calculator.process_batch(&mut distance_cache, &mut performance_tracker);
}

/// Utility functions for easy distance calculations
pub mod distance_utils {
    use super::*;
    
    /// Calculate distance immediately (for critical path)
    pub fn calculate_distance_immediate(
        calculator: &mut ResMut<UnifiedDistanceCalculator>,
        entity1: Entity,
        entity2: Entity,
        pos1: Vec3,
        pos2: Vec3,
    ) -> f32 {
        calculator.get_immediate_distance(entity1, entity2, pos1, pos2)
    }
    
    /// Calculate distance to player/reference position
    pub fn calculate_distance_to_reference(
        calculator: &mut ResMut<UnifiedDistanceCalculator>,
        entity: Entity,
        pos: Vec3,
    ) -> Option<f32> {
        calculator.get_distance_to_reference(entity, pos)
    }
    
    /// Batch multiple distance calculations for efficiency
    pub fn batch_distance_calculations(
        calculator: &mut ResMut<UnifiedDistanceCalculator>,
        requests: &[(Entity, Entity, Vec3, Vec3)],
    ) -> Vec<u32> {
        requests.iter()
            .map(|(e1, e2, p1, p2)| calculator.request_distance(*e1, *e2, *p1, *p2))
            .collect()
    }
}

/// Plugin to add unified distance calculation
pub struct UnifiedDistanceCalculatorPlugin;

impl Plugin for UnifiedDistanceCalculatorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UnifiedDistanceCalculator::default())
            .add_systems(
                Update,
                unified_distance_processing_system.before(crate::systems::distance_cache::distance_cache_management_system)
            );
    }
}
