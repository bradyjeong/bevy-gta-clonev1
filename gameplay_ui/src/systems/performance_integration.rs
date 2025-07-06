use bevy::prelude::*;
use crate::systems::performance_monitor::{UnifiedPerformanceTracker, PerformanceCategory};
use game_core::components::{PerformanceStats, DirtyFlagsMetrics};
use gameplay_sim::systems::input::ControlManager;

/// Integration system that collects performance data from existing systems
pub fn integrate_existing_performance_metrics(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    performance_stats: Res<PerformanceStats>,
    dirty_flags_metrics: Res<DirtyFlagsMetrics>,
    control_manager: Option<Res<ControlManager>>,
    _time: Res<Time>,
) {
    // Integrate existing PerformanceStats
    tracker.update_entity_counts(
        performance_stats.entity_count,
        performance_stats.entity_count - performance_stats.culled_entities,
        performance_stats.culled_entities,
    );
    
    // Update frame timing from existing stats
    if performance_stats.frame_time > 0.0 {
        let fps = 1000.0 / performance_stats.frame_time;
        tracker.update_frame_timing(performance_stats.frame_time, fps);
    }
    
    // Integrate batching metrics
    tracker.record_category_time(PerformanceCategory::Batching, 
        dirty_flags_metrics.processing_time_lod + 
        dirty_flags_metrics.processing_time_visibility + 
        dirty_flags_metrics.processing_time_physics + 
        dirty_flags_metrics.processing_time_transform);
    
    // Update entity processing counts
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::LOD) {
        metrics.entity_count = dirty_flags_metrics.entities_processed_lod;
        metrics.operations_per_frame = dirty_flags_metrics.entities_marked_lod;
    }
    
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::Physics) {
        metrics.entity_count = dirty_flags_metrics.entities_processed_physics;
        metrics.operations_per_frame = dirty_flags_metrics.entities_marked_physics;
    }
    
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::Transform) {
        metrics.entity_count = dirty_flags_metrics.entities_processed_transform;
        metrics.operations_per_frame = dirty_flags_metrics.entities_marked_transform;
    }
    
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::Rendering) {
        metrics.entity_count = dirty_flags_metrics.entities_processed_visibility;
        metrics.operations_per_frame = dirty_flags_metrics.entities_marked_visibility;
    }
    
    // Integrate input system performance
    if let Some(control_manager) = control_manager {
        let (max_time_us, _) = control_manager.get_performance_stats();
        let time_ms = max_time_us as f32 / 1000.0;
        tracker.record_category_time(PerformanceCategory::Input, time_ms);
    }
}

/// System to record distance cache performance
pub fn integrate_distance_cache_performance(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    distance_cache: Option<Res<gameplay_sim::systems::distance_cache::DistanceCache>>,
) {
    if let Some(cache) = distance_cache {
        tracker.update_cache_stats(
            cache.stats.hits as usize,
            cache.stats.misses as usize,
            "distance"
        );
        
        // Update cache size in tracker
        tracker.cache_stats.distance_cache_size = cache.len();
    }
}

/// System to monitor vehicle physics performance
pub fn monitor_vehicle_physics_performance(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    vehicle_query: Query<Entity, With<game_core::components::Car>>,
) {
    let start = std::time::Instant::now();
    
    let vehicle_count = vehicle_query.iter().count();
    
    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
    
    tracker.record_system_time("vehicle_physics_query", elapsed_ms);
    
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::Physics) {
        metrics.entity_count = vehicle_count;
    }
}

/// System to monitor culling performance
pub fn monitor_culling_performance(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    cullable_query: Query<Entity, With<game_core::components::Cullable>>,
) {
    let start = std::time::Instant::now();
    
    let cullable_count = cullable_query.iter().count();
    let visible_count = cullable_query.iter()
        .filter(|_| true) // This would check actual visibility
        .count();
    
    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
    
    tracker.record_category_time(PerformanceCategory::Culling, elapsed_ms);
    tracker.record_system_time("culling_query", elapsed_ms);
    
    tracker.update_entity_counts(cullable_count, visible_count, cullable_count - visible_count);
}

/// System to monitor audio performance
pub fn monitor_audio_performance(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    vehicle_query: Query<Entity, With<game_core::components::Car>>,
) {
    let start = std::time::Instant::now();
    
    let audio_source_count = vehicle_query.iter().count(); // Approximate audio sources as vehicles
    
    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
    
    tracker.record_category_time(PerformanceCategory::Audio, elapsed_ms);
    tracker.record_system_time("audio_system", elapsed_ms);
    
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::Audio) {
        metrics.entity_count = audio_source_count;
    }
}

/// System to monitor spawning performance
pub fn monitor_spawning_performance(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    mut last_entity_count: Local<usize>,
    entity_query: Query<Entity>,
) {
    let start = std::time::Instant::now();
    
    let current_count = entity_query.iter().count();
    let spawned_this_frame = current_count.saturating_sub(*last_entity_count);
    
    let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
    
    tracker.record_category_time(PerformanceCategory::Spawning, elapsed_ms);
    tracker.record_system_time("spawning_monitor", elapsed_ms);
    
    if let Some(metrics) = tracker.categories.get_mut(&PerformanceCategory::Spawning) {
        metrics.operations_per_frame = spawned_this_frame;
    }
    
    tracker.entity_counters.spawned_this_frame = spawned_this_frame;
    *last_entity_count = current_count;
}

/// System to estimate memory usage
pub fn monitor_memory_usage(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    entity_query: Query<Entity>,
    component_query: Query<Entity, With<Transform>>, // Example component count
) {
    let entity_count = entity_query.iter().count();
    let component_count = component_query.iter().count();
    
    // Rough estimation - each entity ~100 bytes, each component ~50 bytes
    let estimated_entity_memory = entity_count * 100;
    let estimated_component_memory = component_count * 50;
    let estimated_system_memory = 1024 * 1024; // 1MB for systems
    
    let total_estimated = estimated_entity_memory + estimated_component_memory + estimated_system_memory;
    
    tracker.update_memory_usage(
        estimated_entity_memory,
        estimated_system_memory,
        total_estimated
    );
}

/// Performance analysis system that detects patterns and suggests optimizations
pub fn performance_analysis_system(
    mut tracker: ResMut<UnifiedPerformanceTracker>,
    time: Res<Time>,
) {
    // Only run analysis every 5 seconds to avoid overhead
    if time.elapsed_secs() as u32 % 5 != 0 {
        return;
    }
    
    let summary = tracker.get_performance_summary();
    
    // Detect performance patterns
    if summary.avg_fps < 45.0 {
        tracker.add_alert(crate::systems::performance_monitor::PerformanceAlert {
            category: PerformanceCategory::System,
            severity: crate::systems::performance_monitor::AlertSeverity::Critical,
            message: "Low FPS detected - consider reducing entity spawn rates".to_string(),
            timestamp: std::time::Instant::now(),
            value: summary.avg_fps,
            threshold: 45.0,
        });
    }
    
    // Check for memory pressure
    if summary.memory_usage_gb > 1.5 {
        tracker.add_alert(crate::systems::performance_monitor::PerformanceAlert {
            category: PerformanceCategory::System,
            severity: crate::systems::performance_monitor::AlertSeverity::Warning,
            message: "High memory usage - consider more aggressive culling".to_string(),
            timestamp: std::time::Instant::now(),
            value: summary.memory_usage_gb,
            threshold: 1.5,
        });
    }
    
    // Check entity density
    let active_ratio = if summary.total_entities > 0 {
        (summary.total_entities - summary.culled_entities) as f32 / summary.total_entities as f32
    } else {
        0.0
    };
    
    if active_ratio > 0.8 {
        tracker.add_alert(crate::systems::performance_monitor::PerformanceAlert {
            category: PerformanceCategory::Culling,
            severity: crate::systems::performance_monitor::AlertSeverity::Warning,
            message: "Low culling efficiency - consider increasing culling distances".to_string(),
            timestamp: std::time::Instant::now(),
            value: active_ratio * 100.0,
            threshold: 80.0,
        });
    }
}

/// Plugin for performance integration systems
pub struct PerformanceIntegrationPlugin;

impl Plugin for PerformanceIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (
            integrate_existing_performance_metrics,
            integrate_distance_cache_performance,
            monitor_vehicle_physics_performance,
            monitor_culling_performance,
            monitor_audio_performance,
            monitor_spawning_performance,
            monitor_memory_usage,
            performance_analysis_system,
        ).chain()); // Run in sequence to ensure data consistency
    }
}
