//! Tests for debug interface - debug overlays, performance metrics display

use bevy::prelude::*;
use game_core::prelude::*;
use gameplay_ui::prelude::*;
use gameplay_ui::systems::performance_monitor::*;
use gameplay_ui::systems::performance_dashboard::*;
use crate::utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test performance overlay creation
    #[test]
    fn test_performance_overlay_creation() {
        let mut app = create_ui_test_app();
        
        // Initial state - no overlay
        assert!(!ui_element_exists::<PerformanceOverlay>(&app));
        // Trigger F3 to create overlay
        simulate_key_press(&mut app, KeyCode::F3);
        // Overlay should be created
        assert!(ui_element_exists::<PerformanceOverlay>(&app));
    }
    /// Test performance tracker initialization
    fn test_performance_tracker_initialization() {
        let tracker = UnifiedPerformanceTracker::default();
        assert!(tracker.enabled);
        assert_eq!(tracker.categories.len(), 12); // All performance categories
        assert_eq!(tracker.frame_analyzer.target_fps, 60.0);
        assert_eq!(tracker.bottleneck_detector.bottleneck_threshold_ms, 5.0);
    /// Test performance category recording
    fn test_performance_category_recording() {
        let mut tracker = UnifiedPerformanceTracker::default();
        // Record some performance data
        tracker.record_category_time(PerformanceCategory::Physics, 2.5);
        tracker.record_category_time(PerformanceCategory::Rendering, 8.3);
        tracker.record_category_time(PerformanceCategory::UI, 1.2);
        // Check recorded values
        assert_eq!(tracker.categories[&PerformanceCategory::Physics].execution_time_ms, 2.5);
        assert_eq!(tracker.categories[&PerformanceCategory::Rendering].execution_time_ms, 8.3);
        assert_eq!(tracker.categories[&PerformanceCategory::UI].execution_time_ms, 1.2);
    /// Test system timing recording
    fn test_system_timing_recording() {
        // Record system timings
        tracker.record_system_time("physics_update", 2.5);
        tracker.record_system_time("rendering_update", 8.3);
        tracker.record_system_time("ui_update", 1.2);
        assert_eq!(tracker.system_timings["physics_update"].last_execution_time, 2.5);
        assert_eq!(tracker.system_timings["rendering_update"].last_execution_time, 8.3);
        assert_eq!(tracker.system_timings["ui_update"].last_execution_time, 1.2);
    /// Test frame timing analysis
    fn test_frame_timing_analysis() {
        // Add frame timing data
        tracker.update_frame_timing(16.67, 60.0);
        tracker.update_frame_timing(33.33, 30.0);
        tracker.update_frame_timing(8.33, 120.0);
        // Check that data was recorded
        assert_eq!(tracker.frame_analyzer.frame_times.len(), 3);
        assert_eq!(tracker.frame_analyzer.fps_history.len(), 3);
        assert!(tracker.frame_analyzer.avg_frame_time > 0.0);
    /// Test bottleneck detection
    fn test_bottleneck_detection() {
        // Record a slow system
        tracker.record_system_time("slow_system", 15.0);
        // Check bottleneck detection
        assert!(tracker.system_timings["slow_system"].is_bottleneck);
        assert!(!tracker.bottleneck_detector.bottleneck_history.is_empty());
    /// Test performance alert generation
    fn test_performance_alert_generation() {
        // Record slow category time (should generate alert)
        tracker.record_category_time(PerformanceCategory::Physics, 15.0);
        // Check alert was generated
        assert!(!tracker.alerts.is_empty());
        assert_eq!(tracker.alerts[0].category, PerformanceCategory::Physics);
        assert!(matches!(tracker.alerts[0].severity, AlertSeverity::Warning));
    /// Test memory tracking
    fn test_memory_tracking() {
        // Update memory usage
        tracker.update_memory_usage(1024 * 1024, 512 * 1024, 2048 * 1024);
        // Check memory tracking
        assert_eq!(tracker.memory_tracker.entity_memory, 1024 * 1024);
        assert_eq!(tracker.memory_tracker.system_memory, 512 * 1024);
        assert_eq!(tracker.memory_tracker.total_allocated, 2048 * 1024);
    /// Test cache statistics
    fn test_cache_statistics() {
        // Update cache stats
        tracker.update_cache_stats(80, 20, "distance");
        tracker.update_cache_stats(90, 10, "asset");
        tracker.update_cache_stats(70, 30, "lod");
        // Check cache hit ratios
        assert_eq!(tracker.get_cache_hit_ratio("distance"), 0.8);
        assert_eq!(tracker.get_cache_hit_ratio("asset"), 0.9);
        assert_eq!(tracker.get_cache_hit_ratio("lod"), 0.7);
    /// Test entity counting
    fn test_entity_counting() {
        // Update entity counts
        tracker.update_entity_counts(1000, 800, 200);
        // Check counts
        assert_eq!(tracker.entity_counters.total_entities, 1000);
        assert_eq!(tracker.entity_counters.active_entities, 800);
        assert_eq!(tracker.entity_counters.culled_entities, 200);
    /// Test performance validation
    fn test_performance_validation() {
        // Valid performance metrics
        assert!(PerformanceValidator::validate_frame_time(16.67).is_ok());
        assert!(PerformanceValidator::validate_fps(60.0).is_ok());
        assert!(PerformanceValidator::validate_memory_usage(2.5).is_ok());
        // Invalid performance metrics
        assert!(PerformanceValidator::validate_frame_time(-1.0).is_err());
        assert!(PerformanceValidator::validate_fps(f32::INFINITY).is_err());
        assert!(PerformanceValidator::validate_memory_usage(100.0).is_err());
    /// Test debug overlay data accuracy
    fn test_debug_overlay_accuracy() {
        let tracker = get_performance_tracker(&app);
        // Create overlay
        // Run some updates to generate data
        run_ui_updates(&mut app, 1.0);
        // Check that overlay contains accurate data
        let summary = tracker.get_performance_summary();
        assert!(summary.avg_fps >= 0.0);
        assert!(summary.avg_frame_time >= 0.0);
        assert!(summary.total_entities >= 0);
    /// Test performance report generation
    fn test_performance_report_generation() {
        // Add some test data
        tracker.record_system_time("test_system", 1.0);
        // Generate report
        let report = tracker.generate_report();
        // Check report content
        assert!(report.contains("UNIFIED PERFORMANCE MONITOR REPORT"));
        assert!(report.contains("Frame Analysis"));
        assert!(report.contains("System Performance"));
        assert!(report.contains("Category Performance"));
    /// Test performance summary
    fn test_performance_summary() {
        // Add test data
        assert_eq!(summary.avg_fps, 60.0);
        assert_eq!(summary.total_entities, 1000);
        assert_eq!(summary.culled_entities, 200);
        assert!(summary.memory_usage_gb > 0.0);
    /// Test debug overlay toggle
    fn test_debug_overlay_toggle() {
        // Initially no overlay
        // Press F3 to show
        // Press F3 again to hide
        // Check visibility state
        let overlay_query = app.world().query::<&Visibility, With<PerformanceOverlay>>();
        if let Some(visibility) = overlay_query.iter(app.world()).next() {
            assert_eq!(*visibility, Visibility::Hidden);
        }
    /// Test performance metric bounds
    fn test_performance_metric_bounds() {
        // Test extreme values
        tracker.update_frame_timing(1000.0, 1.0); // Very slow frame
        tracker.update_memory_usage(0, 0, 8 * 1024 * 1024 * 1024); // 8GB
        // Should handle without crashing
        assert!(summary.avg_fps > 0.0);
    /// Test concurrent performance monitoring
    fn test_concurrent_performance_monitoring() {
        // Simulate multiple systems reporting simultaneously
        for i in 0..100 {
            tracker.record_system_time(&format!("system_{}", i), i as f32 * 0.1);
            tracker.record_category_time(PerformanceCategory::Physics, i as f32 * 0.05);
        // Should handle all reports
        assert_eq!(tracker.system_timings.len(), 100);
        assert!(tracker.categories[&PerformanceCategory::Physics].frame_count > 0);
    /// Test performance alert cleanup
    fn test_performance_alert_cleanup() {
        // Generate many alerts
        for i in 0..1000 {
            tracker.add_alert(PerformanceAlert {
                category: PerformanceCategory::Physics,
                severity: AlertSeverity::Warning,
                message: format!("Test alert {}", i),
                timestamp: std::time::Instant::now(),
                value: i as f32,
                threshold: 10.0,
            });
        // Should maintain reasonable alert count
        assert!(tracker.alerts.len() <= 1000);
}
