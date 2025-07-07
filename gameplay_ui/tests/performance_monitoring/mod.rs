//! Tests for performance monitoring systems

use bevy::prelude::*;
use gameplay_ui::systems::performance_monitor::*;
use crate::utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test performance tracker resource initialization
    #[test]
    fn test_performance_tracker_resource() {
        let mut app = create_ui_test_app();
        
        // Verify resource exists
        assert!(app.world().contains_resource::<UnifiedPerformanceTracker>());
        // Check initial state
        let tracker = get_performance_tracker(&app);
        assert!(tracker.enabled);
        assert!(!tracker.categories.is_empty());
    }
    /// Test performance category recording
    fn test_category_recording() {
        // Get mutable reference and record data
        {
            let mut tracker = app.world_mut().resource_mut::<UnifiedPerformanceTracker>();
            tracker.record_category_time(PerformanceCategory::Physics, 2.5);
            tracker.record_category_time(PerformanceCategory::Rendering, 8.3);
        }
        // Verify recorded data
        assert_eq!(tracker.categories[&PerformanceCategory::Physics].execution_time_ms, 2.5);
        assert_eq!(tracker.categories[&PerformanceCategory::Rendering].execution_time_ms, 8.3);
    /// Test system performance monitoring
    fn test_system_performance_monitoring() {
        // Run several update cycles
        for _ in 0..60 {
            app.update();
        // Check that performance data was collected
        assert!(tracker.frame_analyzer.frame_times.len() > 0);
    /// Test frame spike detection
    fn test_frame_spike_detection() {
        // Simulate frame spike
            tracker.update_frame_timing(50.0, 20.0); // 50ms frame time
        // Check spike detection
        assert!(tracker.frame_analyzer.spike_count > 0);
    /// Test performance alert system
    fn test_performance_alerts() {
        // Generate performance alert
            tracker.record_category_time(PerformanceCategory::Physics, 15.0); // Slow physics
        // Check alert generation
        assert!(!tracker.alerts.is_empty());
    /// Test memory usage tracking
    fn test_memory_usage_tracking() {
        // Update memory usage
            tracker.update_memory_usage(1024 * 1024, 512 * 1024, 2048 * 1024);
        // Verify tracking
        assert_eq!(tracker.memory_tracker.entity_memory, 1024 * 1024);
        assert_eq!(tracker.memory_tracker.system_memory, 512 * 1024);
    /// Test cache performance tracking
    fn test_cache_performance_tracking() {
        // Update cache stats
            tracker.update_cache_stats(80, 20, "distance");
            tracker.update_cache_stats(90, 10, "asset");
        // Verify cache hit ratios
        assert_eq!(tracker.get_cache_hit_ratio("distance"), 0.8);
        assert_eq!(tracker.get_cache_hit_ratio("asset"), 0.9);
    /// Test bottleneck detection
    fn test_bottleneck_detection() {
        // Record slow system
            tracker.record_system_time("slow_system", 20.0);
        // Check bottleneck detection
        assert!(tracker.system_timings.contains_key("slow_system"));
        assert!(tracker.system_timings["slow_system"].is_bottleneck);
    /// Test performance summary generation
    fn test_performance_summary() {
        // Add some performance data
            tracker.update_frame_timing(16.67, 60.0);
            tracker.update_entity_counts(1000, 800, 200);
        // Generate summary
        let summary = tracker.get_performance_summary();
        assert_eq!(summary.avg_fps, 60.0);
        assert_eq!(summary.total_entities, 1000);
        assert_eq!(summary.culled_entities, 200);
    /// Test performance report generation
    fn test_performance_report() {
        // Add performance data
            tracker.record_system_time("test_system", 1.0);
        // Generate report
        let report = tracker.generate_report();
        assert!(report.contains("UNIFIED PERFORMANCE MONITOR"));
        assert!(report.contains("Frame Analysis"));
        assert!(report.contains("System Performance"));
    /// Test performance validation
    fn test_performance_validation() {
        // Test valid metrics
        assert!(PerformanceValidator::validate_frame_time(16.67).is_ok());
        assert!(PerformanceValidator::validate_fps(60.0).is_ok());
        assert!(PerformanceValidator::validate_memory_usage(2.0).is_ok());
        // Test invalid metrics
        assert!(PerformanceValidator::validate_frame_time(-1.0).is_err());
        assert!(PerformanceValidator::validate_fps(-10.0).is_err());
        assert!(PerformanceValidator::validate_memory_usage(-1.0).is_err());
    /// Test performance bounds checking
    fn test_performance_bounds() {
        // Test extreme values
            tracker.update_frame_timing(100.0, 10.0); // Very slow
            tracker.update_memory_usage(0, 0, 16 * 1024 * 1024 * 1024); // 16GB
        // Should handle without panic
        assert!(summary.avg_fps > 0.0);
    /// Test performance data persistence
    fn test_performance_data_persistence() {
        // Record data over time
        for i in 0..120 {
            tracker.update_frame_timing(16.67 + i as f32 * 0.1, 60.0);
        // Check data retention
        assert_eq!(tracker.frame_analyzer.frame_times.len(), 120); // Max capacity
    /// Test concurrent performance monitoring
    fn test_concurrent_monitoring() {
        // Simulate multiple systems reporting
            for i in 0..10 {
                tracker.record_category_time(PerformanceCategory::Physics, i as f32);
                tracker.record_system_time(&format!("system_{}", i), i as f32);
            }
        // All data should be recorded
        assert_eq!(tracker.system_timings.len(), 10);
        assert!(tracker.categories[&PerformanceCategory::Physics].frame_count > 0);
    /// Test performance monitoring overhead
    fn test_monitoring_overhead() {
        // Measure overhead
        let start = std::time::Instant::now();
        // Run many performance recordings
            for i in 0..1000 {
                tracker.record_category_time(PerformanceCategory::Physics, i as f32 * 0.001);
                tracker.record_system_time("test_system", i as f32 * 0.001);
        let duration = start.elapsed();
        // Should be fast (under 10ms for 1000 operations)
        assert!(duration.as_millis() < 10, "Performance monitoring overhead too high: {:?}", duration);
}
