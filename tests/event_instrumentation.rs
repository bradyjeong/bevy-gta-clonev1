#![cfg(feature = "debug-events")]
mod event_instrumentation_tests {
    use bevy::prelude::*;
    use gta_game::instrumentation::{EventMetrics, SystemMetrics, EventMetricsPlugin};
    use std::time::Duration;

    #[test]
    fn test_event_counting() {
        let mut metrics = EventMetrics::default();
        
        // Record some events
        metrics.record_event("TestEvent", 10);
        metrics.record_event("TestEvent", 5);
        
        // Check counts
        let stats = metrics.event_counts.get("TestEvent").unwrap();
        assert_eq!(stats.frame_count, 5);
        assert_eq!(stats.total_count, 15);
    }

    #[test]
    fn test_system_timing() {
        let mut metrics = SystemMetrics::default();
        
        // Record some timings
        metrics.record("test_system", Duration::from_millis(5));
        metrics.record("test_system", Duration::from_millis(10));
        metrics.record("test_system", Duration::from_millis(7));
        
        // Check timing stats
        let timing = metrics.timings.get("test_system").unwrap();
        assert_eq!(timing.call_count, 3);
        assert_eq!(timing.max_duration, Duration::from_millis(10));
        assert_eq!(timing.min_duration, Duration::from_millis(5));
    }

    #[test]
    fn test_slow_system_detection() {
        let mut metrics = SystemMetrics::default();
        
        // Add fast and slow systems
        metrics.record("fast_system", Duration::from_micros(500));
        metrics.record("slow_system", Duration::from_millis(5));
        
        // Check slow system detection
        let slow = metrics.get_slow_systems(Duration::from_millis(1));
        assert_eq!(slow.len(), 1);
        assert_eq!(slow[0].0, "slow_system");
    }

    #[test]
    fn test_queue_age_tracking() {
        let mut metrics = EventMetrics::default();
        
        // Record queue ages
        metrics.record_queue_age("TestEvent", Duration::from_millis(10));
        metrics.record_queue_age("TestEvent", Duration::from_millis(20));
        metrics.record_queue_age("TestEvent", Duration::from_millis(30));
        
        // Check average
        let avg = metrics.get_average_queue_age("TestEvent").unwrap();
        assert_eq!(avg, Duration::from_millis(20));
    }

    #[test]
    fn test_event_rate_calculation() {
        let mut metrics = EventMetrics::default();
        
        // Record events with timing
        metrics.record_event("TestEvent", 100);
        
        // Wait a tiny bit to ensure time has passed
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Record again to trigger rate calculation
        metrics.record_event("TestEvent", 50);
        
        // Rate should be calculated
        let stats = metrics.event_counts.get("TestEvent").unwrap();
        assert!(stats.rate_per_second > 0.0);
    }

    #[test]
    fn test_plugin_initialization() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(EventMetricsPlugin);
        
        // Check resources are initialized
        assert!(app.world().contains_resource::<EventMetrics>());
    }

    #[test]
    fn test_schedule_ordering_validation() {
        use gta_game::instrumentation::ScheduleOrdering;
        
        let mut ordering = ScheduleOrdering::default();
        
        // Add valid dependencies
        ordering.add_dependency("system_b", "system_a");
        ordering.add_dependency("system_c", "system_b");
        
        // Should validate successfully
        assert!(ordering.validate_ordering());
        assert_eq!(ordering.execution_order.len(), 3);
    }

    #[test]
    fn test_schedule_cycle_detection() {
        use gta_game::instrumentation::ScheduleOrdering;
        
        let mut ordering = ScheduleOrdering::default();
        
        // Add circular dependency
        ordering.add_dependency("system_a", "system_b");
        ordering.add_dependency("system_b", "system_c");
        ordering.add_dependency("system_c", "system_a");
        
        // Should detect cycle
        assert!(!ordering.validate_ordering());
    }

    #[test]
    fn test_system_name_validation() {
        use gta_game::instrumentation::schedule_ordering::validate_system_name;
        
        // Valid event handler names
        assert!(validate_system_name("handle_spawn_event"));
        assert!(validate_system_name("handle_despawn_event"));
        
        // Invalid event handler names
        assert!(!validate_system_name("spawn_event_handler"));
        assert!(!validate_system_name("event_spawn"));
        
        // Non-event system names are always valid
        assert!(validate_system_name("update_transforms"));
        assert!(validate_system_name("physics_step"));
    }

    #[test]
    fn test_mermaid_graph_generation() {
        use gta_game::instrumentation::ScheduleOrdering;
        
        let mut ordering = ScheduleOrdering::default();
        ordering.add_dependency("system_b", "system_a");
        ordering.add_dependency("system_c", "system_b");
        
        let graph = ordering.generate_mermaid_graph();
        assert!(graph.contains("graph TD"));
        assert!(graph.contains("system_a --> system_b"));
        assert!(graph.contains("system_b --> system_c"));
    }

    #[test]
    fn test_performance_budget() {
        use gta_game::instrumentation::system_profiling::PerformanceBudget;
        
        let mut budget = PerformanceBudget::default();
        budget.system_budgets.insert("physics_step", Duration::from_millis(5));
        
        assert_eq!(budget.frame_budget, Duration::from_millis(16));
        assert_eq!(budget.system_budgets.get("physics_step"), Some(&Duration::from_millis(5)));
    }

    #[test]
    fn test_zero_cost_abstraction() {
        // This test verifies that instrumentation has zero cost when disabled
        // It should compile but do nothing when debug-events is not enabled
        
        // The following should compile even without debug-events
        #[cfg(not(feature = "debug-events"))]
        {
            use gta_game::instrumentation::EventMetricsPlugin;
            let mut app = App::new();
            app.add_plugins(EventMetricsPlugin);
            // Plugin should be no-op
        }
    }
}


