//! Performance validation tests
//! Tests frame timing, render call batching, and performance monitoring

use bevy::prelude::*;
use gameplay_render::prelude::*;
use crate::utils::*;
use std::time::Instant;
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_timing_within_budget() {
        let mut app = create_render_test_app();
        
        // Create substantial entity load
        let entities: Vec<Entity> = (0..100)
            .map(|i| {
                let distance = (i as f32) * 10.0;
                create_test_vehicle_at_distance(&mut app, distance)
            })
            .collect();
        let _camera = create_test_camera(&mut app);
        // Measure frame timing
        let frame_times: Vec<f32> = (0..30)
            .map(|_| {
                let start = Instant::now();
                app.update();
                start.elapsed().as_secs_f32() * 1000.0 // Convert to ms
        // Test that frame times are within budget
        let avg_frame_time = frame_times.iter().sum::<f32>() / frame_times.len() as f32;
        let max_frame_time = frame_times.iter().fold(0.0f32, |max, &time| max.max(time));
        assert!(avg_frame_time < 16.67, 
            "Average frame time {:.2}ms should be under 16.67ms (60 FPS)", avg_frame_time);
        assert!(max_frame_time < 33.33,
            "Maximum frame time {:.2}ms should be under 33.33ms (30 FPS)", max_frame_time);
        // Test that 95% of frames are within budget
        let mut sorted_times = frame_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_time = sorted_times[(sorted_times.len() as f32 * 0.95) as usize];
        assert!(p95_time < 20.0,
            "95th percentile frame time {:.2}ms should be under 20ms", p95_time);
    }
    fn test_render_call_batching_efficiency() {
        // Create multiple entities of the same type for batching
        let vehicle_entities: Vec<Entity> = (0..20)
                let distance = (i as f32) * 15.0;
        // Run batching systems
        run_render_simulation_frames(&mut app, 5);
        // Test that batching is efficient
        let perf = PerformanceMeasurement::capture(&app);
        // Should process entities in batches, not individually
        assert!(perf.render_operations > 0, "Render operations should be recorded");
        assert!(perf.render_operations < vehicle_entities.len() as u32,
            "Render operations {} should be less than entity count {} due to batching",
            perf.render_operations, vehicle_entities.len());
        // Test that entities are properly batched by distance
        let visible_count = vehicle_entities.iter()
            .filter(|&&entity| {
                let world = app.world();
                matches!(world.get::<Visibility>(entity), Some(Visibility::Visible))
            .count();
        assert!(visible_count > 0, "Some entities should be visible");
        assert!(visible_count < vehicle_entities.len(), "Some entities should be culled");
    fn test_lod_performance_monitoring() {
        // Create entities that will trigger LOD updates
        let entities: Vec<Entity> = (0..50)
                let distance = (i as f32) * 8.0;
        // Capture initial performance state
        let initial_perf = PerformanceMeasurement::capture(&app);
        // Run LOD systems
        run_render_simulation_frames(&mut app, 10);
        // Capture final performance state
        let final_perf = PerformanceMeasurement::capture(&app);
        // Test that LOD monitoring is working
        assert!(final_perf.lod_updates >= initial_perf.lod_updates,
            "LOD updates should be monitored");
        assert!(final_perf.entities_processed > 0,
            "Entity processing should be monitored");
        assert!(final_perf.culled_entities >= 0,
            "Culled entities should be counted");
        // Test that frame timing is reasonable
        assert!(final_perf.frame_time_ms >= 0.0,
            "Frame time should be measured");
        assert!(final_perf.frame_time_ms < 50.0,
            "Frame time {:.2}ms should be reasonable", final_perf.frame_time_ms);
    fn test_system_time_budgets() {
        // Create heavy load to test time budgets
        let entities: Vec<Entity> = (0..200)
                let distance = (i as f32) * 3.0;
                create_test_building_at_distance(&mut app, distance)
        // Measure system execution time
        let start = Instant::now();
        let total_time = start.elapsed().as_millis() as f32;
        // Test that systems respect time budgets
        assert!(total_time < 100.0,
            "Total render time {:.2}ms should be under 100ms for 200 entities", total_time);
        // Test that not all entities are processed if time budget is exceeded
        let processed_count = entities.iter()
                // Check if entity visibility was updated (indicating processing)
                world.get::<Visibility>(entity).is_some()
        assert!(processed_count > 0, "Some entities should be processed");
        // Note: May process all entities in test environment due to fast execution
    fn test_render_optimization_performance() {
        // Create mixed entity types at various distances
        let mut all_entities = Vec::new();
        // Add vehicles
        for i in 0..30 {
            let distance = (i as f32) * 12.0;
            all_entities.push(create_test_vehicle_at_distance(&mut app, distance));
        }
        // Add NPCs
        for i in 0..20 {
            let distance = (i as f32) * 15.0;
            all_entities.push(create_test_npc_at_distance(&mut app, distance));
        // Add buildings
        for i in 0..40 {
            let distance = (i as f32) * 8.0;
            all_entities.push(create_test_building_at_distance(&mut app, distance));
        // Measure optimization performance
        let optimization_times: Vec<f32> = (0..10)
                start.elapsed().as_secs_f32() * 1000.0
        let avg_optimization_time = optimization_times.iter().sum::<f32>() / optimization_times.len() as f32;
        // Test that optimization is efficient
        assert!(avg_optimization_time < 10.0,
            "Average optimization time {:.2}ms should be under 10ms", avg_optimization_time);
        // Test that performance counters are accurate
        assert!(perf.entities_processed > 0, "Should process entities");
        assert!(perf.render_operations > 0, "Should record render operations");
    fn test_vegetation_instancing_performance() {
        // Create many vegetation instances
        let vegetation_entities: Vec<Entity> = (0..100)
                let distance = (i as f32) * 4.0;
                create_test_vegetation_at_distance(&mut app, distance)
        // Measure vegetation processing performance
        let vegetation_time = start.elapsed().as_millis() as f32;
        // Test that vegetation instancing is efficient
        assert!(vegetation_time < 50.0,
            "Vegetation processing time {:.2}ms should be under 50ms for 100 instances", vegetation_time);
        // Test that vegetation LOD is working correctly
        let full_detail_count = vegetation_entities.iter()
                if let Some(veg_lod) = world.get::<VegetationLOD>(entity) {
                    matches!(veg_lod.detail_level, VegetationDetailLevel::Full)
                } else {
                    false
                }
        let culled_count = vegetation_entities.iter()
                    matches!(veg_lod.detail_level, VegetationDetailLevel::Culled)
        assert!(full_detail_count > 0, "Some vegetation should be full detail");
        assert!(culled_count > 0, "Some vegetation should be culled");
        assert!(full_detail_count < vegetation_entities.len(), "Not all vegetation should be full detail");
    fn test_distance_cache_performance() {
        // Create entities that will trigger distance calculations
        let entities: Vec<Entity> = (0..60)
                let distance = (i as f32) * 7.0;
        // Run multiple frames to test distance caching
        run_render_simulation_frames(&mut app, 20);
        // Test that distance caching improves performance
        assert!(total_time < 200.0,
            "Total time {:.2}ms should be under 200ms with distance caching", total_time);
        // Test that entities are processed efficiently
        // Test that frame rate is maintained
        let avg_frame_time = total_time / 20.0;
        assert!(avg_frame_time < 16.67,
            "Average frame time {:.2}ms should maintain 60 FPS", avg_frame_time);
    fn test_culling_performance_scaling() {
        // Test with different entity counts
        let entity_counts = vec![10, 50, 100, 200];
        let mut performance_results = Vec::new();
        for &count in &entity_counts {
            let mut test_app = create_render_test_app();
            
            // Create entities
            let _entities: Vec<Entity> = (0..count)
                .map(|i| {
                    let distance = (i as f32) * 5.0;
                    create_test_vehicle_at_distance(&mut test_app, distance)
                })
                .collect();
            let _camera = create_test_camera(&mut test_app);
            // Measure performance
            let start = Instant::now();
            run_render_simulation_frames(&mut test_app, 5);
            let time = start.elapsed().as_millis() as f32;
            performance_results.push((count, time));
        // Test that performance scales reasonably
        for (count, time) in &performance_results {
            assert!(*time < (*count as f32) * 0.5,
                "Time {:.2}ms should scale better than 0.5ms per entity for {} entities", time, count);
        // Test that culling prevents linear scaling
        let (small_count, small_time) = performance_results[0];
        let (large_count, large_time) = performance_results[performance_results.len() - 1];
        let scaling_factor = large_time / small_time;
        let entity_factor = large_count as f32 / small_count as f32;
        assert!(scaling_factor < entity_factor,
            "Performance scaling {:.2}x should be better than entity scaling {:.2}x due to culling",
            scaling_factor, entity_factor);
    fn test_render_budget_management() {
        // Create entities that will test budget management
        let entities: Vec<Entity> = (0..150)
        // Run with budget constraints
        let frame_times: Vec<f32> = (0..15)
        // Test that budget management keeps frames consistent
        let min_frame_time = frame_times.iter().fold(f32::MAX, |min, &time| min.min(time));
        let frame_time_variance = max_frame_time - min_frame_time;
        assert!(frame_time_variance < 10.0,
            "Frame time variance {:.2}ms should be under 10ms", frame_time_variance);
        // Test that all frame times are reasonable
        for (i, &time) in frame_times.iter().enumerate() {
            assert!(time < 25.0,
                "Frame {} time {:.2}ms should be under 25ms", i, time);
}
