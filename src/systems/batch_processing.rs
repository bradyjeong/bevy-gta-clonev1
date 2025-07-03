use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Instant;
use std::collections::HashMap;
use crate::components::*;
use crate::systems::distance_cache::DistanceCache;
use crate::systems::unified_distance_calculator::{UnifiedDistanceCalculator, distance_utils};
use crate::systems::world::unified_distance_culling::UnifiedCullable;
use crate::config::GameConfig;
use crate::systems::performance_monitor::{UnifiedPerformanceTracker, PerformanceCategory};

/// Advanced batch processing system for similar entities
/// Provides enhanced performance through intelligent grouping and parallel processing

/// Core batch processor for managing batch operations on similar entities
#[derive(Resource, Default)]
pub struct BatchProcessor {
    /// Entity groups organized by batch type
    _entity_groups: HashMap<BatchType, Vec<Entity>>,
    /// Processing statistics for optimization
    pub processing_stats: BatchProcessingStats,
    /// Adaptive batch sizes based on performance
    pub adaptive_batch_sizes: HashMap<BatchType, usize>,
    /// Frame timing for batch optimization
    _frame_timings: Vec<f32>,
    /// Last optimization time
    pub last_optimization: f32,
}

/// Types of batch operations supported
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum BatchType {
    Transform,
    Visibility,
    Physics,
    LOD,
    VegetationInstancing,
    Culling,
}

/// Batch processing statistics for performance tracking
#[derive(Default, Debug)]
pub struct BatchProcessingStats {
    pub entities_processed_per_type: HashMap<BatchType, usize>,
    pub processing_time_per_type: HashMap<BatchType, f32>,
    pub batch_efficiency: HashMap<BatchType, f32>,
    pub total_batches_processed: usize,
    pub average_batch_size: f32,
    pub peak_processing_time: f32,
    pub frame_rate_impact: f32,
}

/// Enhanced batch culling system with intelligent grouping
pub fn batch_culling_system_enhanced(
    mut commands: Commands,
    mut dirty_visibility_query: Query<(Entity, &DirtyVisibility, &Transform, &mut Visibility, Option<&UnifiedCullable>)>,
    active_query: Query<&Transform, (With<ActiveEntity>, Without<DirtyVisibility>)>,
    mut batch_processor: ResMut<BatchProcessor>,
    _distance_cache: ResMut<DistanceCache>,
    mut distance_calculator: ResMut<UnifiedDistanceCalculator>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
    config: Res<GameConfig>,
    _time: Res<Time>,
) {
    let start_time = Instant::now();
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Set reference position for efficient distance calculations
    distance_calculator.set_reference_position(active_pos);
    
    // Group entities by distance ranges for efficient batch processing
    let mut distance_groups: HashMap<u32, Vec<_>> = HashMap::new();
    let mut entities_to_process: Vec<_> = dirty_visibility_query.iter_mut().collect();
    
    // Sort by priority first, then group by distance buckets
    entities_to_process.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    // Create distance-based groups (100m buckets) using unified distance calculator
    for entity_data in entities_to_process {
        let distance = distance_utils::calculate_distance_to_reference(
            &mut distance_calculator,
            entity_data.0,
            entity_data.2.translation,
        ).unwrap_or_else(|| active_pos.distance(entity_data.2.translation));
        let distance_bucket = (distance / 100.0) as u32;
        distance_groups.entry(distance_bucket).or_default().push(entity_data);
    }
    
    let batch_size = batch_processor.adaptive_batch_sizes.get(&BatchType::Culling)
        .copied().unwrap_or(config.batching.visibility_batch_size);
    
    let mut total_processed = 0;
    let max_processing_time = config.batching.max_processing_time_ms;
    
    // Process distance groups in order (closest first)
    let mut sorted_buckets: Vec<_> = distance_groups.keys().copied().collect();
    sorted_buckets.sort();
    
    for distance_bucket in sorted_buckets {
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        let mut entities_in_bucket = distance_groups.remove(&distance_bucket).unwrap_or_default();
        
        // Process this distance bucket in batches
        for batch in entities_in_bucket.chunks_mut(batch_size) {
            if start_time.elapsed().as_millis() as f32 > max_processing_time {
                break;
            }
            
            let batch_len = batch.len();
            
            // Batch process entities at similar distances
            for &mut (ref entity, ref _dirty_vis, ref transform, ref mut visibility, ref cullable) in batch {
                if let Some(cull) = cullable {
                    let distance = distance_utils::calculate_distance_to_reference(
                        &mut distance_calculator,
                        *entity,
                        transform.translation,
                    ).unwrap_or_else(|| active_pos.distance(transform.translation));
                    let should_be_visible = !cull.is_culled && distance <= cull.config.cull_distance;
                    
                    let new_visibility = if should_be_visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                    
                    if **visibility != new_visibility {
                        **visibility = new_visibility;
                    }
                }
                
                // Remove dirty flag after processing
                commands.entity(*entity).remove::<DirtyVisibility>();
            }
            total_processed += batch_len;
        }
    }
    
    // Update processing stats
    let processing_time = start_time.elapsed().as_millis() as f32;
    batch_processor.processing_stats.entities_processed_per_type
        .insert(BatchType::Culling, total_processed);
    batch_processor.processing_stats.processing_time_per_type
        .insert(BatchType::Culling, processing_time);
    
    // Update batch efficiency metric
    let efficiency = if processing_time > 0.0 {
        total_processed as f32 / processing_time
    } else {
        0.0
    };
    batch_processor.processing_stats.batch_efficiency
        .insert(BatchType::Culling, efficiency);

    // Integrate with unified performance monitoring
    performance_tracker.record_category_time(PerformanceCategory::Batching, processing_time);
    performance_tracker.record_system_time("batch_culling_enhanced", processing_time);
}



/// Enhanced batch physics updater with parallel processing hints
pub fn batch_physics_updater_system(
    mut commands: Commands,
    mut dirty_physics_query: Query<(Entity, &DirtyPhysics, &mut Velocity, Option<&mut RigidBody>, &Transform)>,
    mut batch_processor: ResMut<BatchProcessor>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
    config: Res<GameConfig>,
) {
    let start_time = Instant::now();
    
    // Group entities by physics complexity for better batch processing
    let mut physics_groups: HashMap<PhysicsComplexity, Vec<_>> = HashMap::new();
    let mut entities_to_process: Vec<_> = dirty_physics_query.iter_mut().collect();
    
    // Sort by priority first
    entities_to_process.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    
    // Group by physics complexity
    for entity_data in entities_to_process {
        let complexity = determine_physics_complexity(&entity_data.2, entity_data.3.as_deref());
        physics_groups.entry(complexity).or_default().push(entity_data);
    }
    
    let batch_size = batch_processor.adaptive_batch_sizes.get(&BatchType::Physics)
        .copied().unwrap_or(config.batching.physics_batch_size);
    
    let mut total_processed = 0;
    let max_processing_time = config.batching.max_processing_time_ms;
    
    // Process groups by complexity (high complexity first for responsive gameplay)
    let complexity_order = [PhysicsComplexity::High, PhysicsComplexity::Medium, PhysicsComplexity::Low];
    
    for complexity in complexity_order {
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        if let Some(mut entities_in_group) = physics_groups.remove(&complexity) {
            // Process this complexity group in batches
            for batch in entities_in_group.chunks_mut(batch_size) {
                if start_time.elapsed().as_millis() as f32 > max_processing_time {
                    break;
                }
                
                // Batch process entities with similar physics complexity
                process_physics_batch(&mut commands, batch, &config);
                total_processed += batch.len();
            }
        }
    }
    
    // Update processing stats
    let processing_time = start_time.elapsed().as_millis() as f32;
    batch_processor.processing_stats.entities_processed_per_type
        .insert(BatchType::Physics, total_processed);
    batch_processor.processing_stats.processing_time_per_type
        .insert(BatchType::Physics, processing_time);

    // Integrate with unified performance monitoring
    performance_tracker.record_category_time(PerformanceCategory::Physics, processing_time);
    performance_tracker.record_system_time("batch_physics_updater", processing_time);
}

/// Physics complexity classification for batch optimization
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum PhysicsComplexity {
    Low,    // Static or simple kinematic bodies
    Medium, // Dynamic bodies with simple shapes
    High,   // Complex dynamic bodies or multiple constraints
}

/// Determine physics complexity for batch grouping
fn determine_physics_complexity(velocity: &Velocity, rigid_body: Option<&RigidBody>) -> PhysicsComplexity {
    match rigid_body {
        Some(RigidBody::Fixed) => PhysicsComplexity::Low,
        Some(RigidBody::KinematicPositionBased) | Some(RigidBody::KinematicVelocityBased) => {
            if velocity.linvel.length() > 10.0 || velocity.angvel.length() > 2.0 {
                PhysicsComplexity::Medium
            } else {
                PhysicsComplexity::Low
            }
        }
        Some(RigidBody::Dynamic) => {
            if velocity.linvel.length() > 20.0 || velocity.angvel.length() > 5.0 {
                PhysicsComplexity::High
            } else {
                PhysicsComplexity::Medium
            }
        }
        None => PhysicsComplexity::Low,
    }
}

/// Process a batch of physics entities
fn process_physics_batch(
    commands: &mut Commands,
    batch: &mut [(Entity, &DirtyPhysics, Mut<Velocity>, Option<Mut<RigidBody>>, &Transform)],
    config: &GameConfig,
) {
    let max_velocity = config.physics.max_velocity;
    let max_angular_velocity = config.physics.max_angular_velocity;
    
    for (entity, _dirty_physics, velocity, rigid_body, _transform) in batch.iter_mut() {
        // Apply physics constraints and validation
        // Only apply constraints if velocity is extreme to avoid micro-jitter
        if velocity.linvel.length() > max_velocity * 1.5 {
            velocity.linvel = velocity.linvel.normalize() * max_velocity;
        }
        
        if velocity.angvel.length() > max_angular_velocity * 1.5 {
            velocity.angvel = velocity.angvel.normalize() * max_angular_velocity;
        }
        
        // Additional rigid body processing if needed
        if let Some(_rb) = rigid_body {
            // Could add additional rigid body constraints here
        }
        
        // Remove dirty flag after processing
        commands.entity(*entity).remove::<DirtyPhysics>();
    }
}

/// Batch visibility manager for group visibility changes
pub fn batch_visibility_manager_system(
    mut commands: Commands,
    visibility_query: Query<(Entity, &mut Visibility, &Transform, Option<&UnifiedCullable>)>,
    active_query: Query<&Transform, With<ActiveEntity>>,
    mut batch_processor: ResMut<BatchProcessor>,
    _distance_cache: ResMut<DistanceCache>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
    config: Res<GameConfig>,
    time: Res<Time>,
    mut last_run: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Run at a lower frequency to reduce overhead
    if current_time - *last_run < 0.2 {
        return;
    }
    *last_run = current_time;
    
    let start_time = Instant::now();
    let Ok(active_transform) = active_query.single() else { return };
    let active_pos = active_transform.translation;
    
    // Group entities by visibility state for batch operations
    let mut visibility_groups: HashMap<VisibilityState, Vec<_>> = HashMap::new();
    
    for (entity, visibility, transform, cullable) in visibility_query.iter() {
        let distance = active_pos.distance(transform.translation);
        let state = determine_visibility_state(&visibility, distance, cullable);
        visibility_groups.entry(state).or_default().push((entity, visibility, transform, cullable, distance));
    }
    
    let batch_size = config.batching.visibility_batch_size;
    let max_processing_time = config.batching.max_processing_time_ms * 0.3; // Use less time for this system
    
    let mut total_processed = 0;
    
    // Process visibility groups (prioritize transitions)
    let priority_order = [
        VisibilityState::NeedsToShow,
        VisibilityState::NeedsToHide,
        VisibilityState::Stable,
    ];
    
    for state in priority_order {
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        if let Some(entities_in_group) = visibility_groups.remove(&state) {
            for batch in entities_in_group.chunks(batch_size) {
                if start_time.elapsed().as_millis() as f32 > max_processing_time {
                    break;
                }
                
                process_visibility_state_batch(&mut commands, batch, state);
                total_processed += batch.len();
            }
        }
    }
    
    // Update processing stats
    let processing_time = start_time.elapsed().as_millis() as f32;
    batch_processor.processing_stats.entities_processed_per_type
        .insert(BatchType::Visibility, total_processed);
    batch_processor.processing_stats.processing_time_per_type
        .insert(BatchType::Visibility, processing_time);

    // Integrate with unified performance monitoring
    performance_tracker.record_category_time(PerformanceCategory::Rendering, processing_time);
    performance_tracker.record_system_time("batch_visibility_manager", processing_time);
}

/// Visibility state for batch grouping
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum VisibilityState {
    NeedsToShow,
    NeedsToHide,
    Stable,
}

/// Determine visibility state for batch grouping
fn determine_visibility_state(
    current_visibility: &Visibility,
    distance: f32,
    cullable: Option<&UnifiedCullable>,
) -> VisibilityState {
    if let Some(cull) = cullable {
        let should_be_visible = distance <= cull.config.cull_distance && !cull.is_culled;
        let is_currently_visible = matches!(current_visibility, Visibility::Visible);
        
        match (should_be_visible, is_currently_visible) {
            (true, false) => VisibilityState::NeedsToShow,
            (false, true) => VisibilityState::NeedsToHide,
            _ => VisibilityState::Stable,
        }
    } else {
        VisibilityState::Stable
    }
}

/// Process a batch of visibility state entities
fn process_visibility_state_batch(
    commands: &mut Commands,
    batch: &[(Entity, &Visibility, &Transform, Option<&UnifiedCullable>, f32)],
    state: VisibilityState,
) {
    for (entity, _visibility, _transform, cullable, distance) in batch {
        if let Some(cull) = cullable {
            let _should_be_visible = *distance <= cull.config.cull_distance && !cull.is_culled;
            
            let new_visibility = match state {
                VisibilityState::NeedsToShow => Visibility::Visible,
                VisibilityState::NeedsToHide => Visibility::Hidden,
                VisibilityState::Stable => continue, // No change needed
            };
            
            // Only update if the visibility actually needs to change
            commands.entity(*entity).insert(new_visibility);
        }
    }
}

/// System to mark vegetation entities as dirty when they change (advanced batch processing version)
pub fn advanced_batch_mark_vegetation_instancing_dirty_system(
    mut commands: Commands,
    changed_vegetation: Query<Entity, (Changed<Transform>, With<VegetationBatchable>, Without<DirtyVegetationInstancing>)>,
    frame_counter: Res<FrameCounter>,
) {
    let current_frame = frame_counter.frame;
    
    for entity in changed_vegetation.iter() {
        commands.entity(entity).insert(DirtyVegetationInstancing::new(
            DirtyPriority::Low, // Vegetation changes are low priority
            current_frame,
        ));
    }
}

/// Batch size optimization system based on performance metrics
pub fn batch_size_optimization_system(
    mut batch_processor: ResMut<BatchProcessor>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
    config: Res<GameConfig>,
    time: Res<Time>,
    mut frame_time_tracker: Local<f32>,
    mut frame_count: Local<u32>,
) {
    let current_time = time.elapsed_secs();
    *frame_time_tracker += time.delta_secs();
    *frame_count += 1;
    
    // Optimize batch sizes every 5 seconds
    if current_time - batch_processor.last_optimization < 5.0 {
        return;
    }
    
    batch_processor.last_optimization = current_time;
    
    // Calculate current frame rate
    let average_frame_time = *frame_time_tracker / *frame_count as f32;
    let current_fps = 1.0 / average_frame_time;
    
    // Reset frame tracking
    *frame_time_tracker = 0.0;
    *frame_count = 0;
    
    // Store frame rate for metrics
    batch_processor.processing_stats.frame_rate_impact = current_fps;
    
    // Target FPS from config (default 60)
    let target_fps = 60.0;
    let fps_ratio = current_fps / target_fps;
    
    // Optimize batch sizes based on performance
    optimize_batch_sizes(&mut batch_processor, fps_ratio, &config);
    
    // Update unified performance tracker with batch processing metrics
    performance_tracker.update_frame_timing(average_frame_time * 1000.0, current_fps);
    
    info!(
        "Batch Size Optimization - FPS: {:.1} | Target: {:.1} | Ratio: {:.2}",
        current_fps, target_fps, fps_ratio
    );
}

/// Optimize batch sizes based on performance metrics
fn optimize_batch_sizes(
    batch_processor: &mut BatchProcessor,
    fps_ratio: f32,
    config: &GameConfig,
) {
    let batch_types = [
        BatchType::Transform,
        BatchType::Visibility,
        BatchType::Physics,
        BatchType::LOD,
        BatchType::Culling,
    ];
    
    for batch_type in batch_types {
        let current_batch_size = batch_processor.adaptive_batch_sizes
            .get(&batch_type)
            .copied()
            .unwrap_or_else(|| get_default_batch_size(batch_type, config));
        
        // Get efficiency for this batch type
        let efficiency = batch_processor.processing_stats.batch_efficiency
            .get(&batch_type)
            .copied()
            .unwrap_or(0.0);
        
        // Adjust batch size based on FPS and efficiency
        let new_batch_size = calculate_optimal_batch_size(
            current_batch_size,
            fps_ratio,
            efficiency,
            batch_type,
        );
        
        batch_processor.adaptive_batch_sizes.insert(batch_type, new_batch_size);
    }
}

/// Get default batch size for a batch type
fn get_default_batch_size(batch_type: BatchType, config: &GameConfig) -> usize {
    match batch_type {
        BatchType::Transform => config.batching.transform_batch_size,
        BatchType::Visibility => config.batching.visibility_batch_size,
        BatchType::Physics => config.batching.physics_batch_size,
        BatchType::LOD => config.batching.lod_batch_size,
        BatchType::Culling => config.batching.visibility_batch_size,
        BatchType::VegetationInstancing => 100, // Default for vegetation
    }
}

/// Calculate optimal batch size based on performance metrics
fn calculate_optimal_batch_size(
    current_size: usize,
    fps_ratio: f32,
    efficiency: f32,
    batch_type: BatchType,
) -> usize {
    let mut adjustment_factor = 1.0;
    
    // Adjust based on FPS performance
    if fps_ratio < 0.9 {
        // FPS is low, reduce batch size
        adjustment_factor *= 0.85;
    } else if fps_ratio > 1.1 {
        // FPS is high, can increase batch size
        adjustment_factor *= 1.15;
    }
    
    // Adjust based on efficiency
    if efficiency < 10.0 {
        // Low efficiency, reduce batch size
        adjustment_factor *= 0.9;
    } else if efficiency > 50.0 {
        // High efficiency, can increase batch size
        adjustment_factor *= 1.1;
    }
    
    // Apply batch type specific constraints
    let (min_size, max_size) = get_batch_size_constraints(batch_type);
    
    let new_size = (current_size as f32 * adjustment_factor) as usize;
    new_size.clamp(min_size, max_size)
}

/// Get batch size constraints for a batch type
fn get_batch_size_constraints(batch_type: BatchType) -> (usize, usize) {
    match batch_type {
        BatchType::Transform => (20, 200),
        BatchType::Visibility => (30, 300),
        BatchType::Physics => (10, 100),
        BatchType::LOD => (25, 250),
        BatchType::Culling => (40, 400),
        BatchType::VegetationInstancing => (50, 500),
    }
}

/// Performance monitoring and reporting system
pub fn batch_performance_monitor_system(
    batch_processor: Res<BatchProcessor>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Report every 10 seconds
    if current_time - *last_report < 10.0 {
        return;
    }
    *last_report = current_time;
    
    let stats = &batch_processor.processing_stats;
    
    info!("=== Advanced Batch Processing Performance Report ===");
    info!("Frame Rate Impact: {:.1} FPS", stats.frame_rate_impact);
    info!("Total Batches Processed: {}", stats.total_batches_processed);
    info!("Average Batch Size: {:.1}", stats.average_batch_size);
    info!("Peak Processing Time: {:.2}ms", stats.peak_processing_time);
    
    for (batch_type, &processed) in &stats.entities_processed_per_type {
        let processing_time = stats.processing_time_per_type.get(batch_type).copied().unwrap_or(0.0);
        let efficiency = stats.batch_efficiency.get(batch_type).copied().unwrap_or(0.0);
        
        info!(
            "  {:?}: {} entities, {:.2}ms, {:.1} entities/ms",
            batch_type, processed, processing_time, efficiency
        );
        
        // Update unified performance tracker with detailed batch metrics
        if let Some(metrics) = performance_tracker.categories.get_mut(&PerformanceCategory::Batching) {
            metrics.entity_count = processed;
            metrics.operations_per_frame = processed;
        }
    }
    
    // Report current adaptive batch sizes
    info!("Current Adaptive Batch Sizes:");
    for (batch_type, &size) in &batch_processor.adaptive_batch_sizes {
        info!("  {:?}: {}", batch_type, size);
    }
}

/// Parallel job distribution system for large batch operations
pub fn parallel_batch_distribution_system(
    mut commands: Commands,
    large_batches_query: Query<Entity, (With<DirtyTransform>, With<LargeBatchMarker>)>,
    mut batch_processor: ResMut<BatchProcessor>,
    mut performance_tracker: ResMut<UnifiedPerformanceTracker>,
    config: Res<GameConfig>,
) {
    // This system would ideally use Bevy's task pool for parallel processing
    // For now, we'll simulate parallel processing with optimized sequential batching
    
    let entities: Vec<_> = large_batches_query.iter().collect();
    if entities.is_empty() {
        return;
    }
    
    let start_time = Instant::now();
    
    // Split large batches into smaller parallel-friendly chunks
    let chunk_size = 32; // Optimal for most CPU architectures
    let max_processing_time = config.batching.max_processing_time_ms;
    
    let mut total_processed = 0;
    
    for chunk in entities.chunks(chunk_size) {
        if start_time.elapsed().as_millis() as f32 > max_processing_time {
            break;
        }
        
        // Process chunk (in real implementation, this would be dispatched to task pool)
        for &entity in chunk {
            commands.entity(entity).remove::<DirtyTransform>();
            total_processed += 1;
        }
    }
    
    // Update stats
    let processing_time = start_time.elapsed().as_millis() as f32;
    batch_processor.processing_stats.total_batches_processed += 1;
    batch_processor.processing_stats.average_batch_size = 
        (batch_processor.processing_stats.average_batch_size + total_processed as f32) / 2.0;

    // Integrate with unified performance monitoring
    performance_tracker.record_category_time(PerformanceCategory::Transform, processing_time);
    performance_tracker.record_system_time("parallel_batch_distribution", processing_time);
}

/// Marker component for entities that benefit from large batch processing
#[derive(Component)]
pub struct LargeBatchMarker;

/// Initialize batch processor with default settings
pub fn initialize_batch_processor_system(
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    let mut batch_processor = BatchProcessor::default();
    
    // Initialize adaptive batch sizes with config defaults
    batch_processor.adaptive_batch_sizes.insert(BatchType::Transform, config.batching.transform_batch_size);
    batch_processor.adaptive_batch_sizes.insert(BatchType::Visibility, config.batching.visibility_batch_size);
    batch_processor.adaptive_batch_sizes.insert(BatchType::Physics, config.batching.physics_batch_size);
    batch_processor.adaptive_batch_sizes.insert(BatchType::LOD, config.batching.lod_batch_size);
    batch_processor.adaptive_batch_sizes.insert(BatchType::Culling, config.batching.visibility_batch_size);
    batch_processor.adaptive_batch_sizes.insert(BatchType::VegetationInstancing, 100);
    
    commands.insert_resource(batch_processor);
    
    info!("Advanced batch processor initialized with intelligent grouping and parallel processing support");
}
