//! Comprehensive test suite for gameplay_ui crate
//! Tests for UI logic, input handling, debug interfaces, and menu systems

// Import all test modules
mod utils;
mod ui_logic;
mod input_handling;
mod debug_interface;
mod menu_systems;
mod performance_monitoring;
mod telemetry_systems;
mod property_tests;
// Re-export test utilities
pub use utils::*;
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Integration test to verify all test modules are working
    #[test]
    fn test_modules_integration() {
        let app = create_ui_test_app();
        assert!(app.world().entities().len() > 0, "UI test app should have entities");
    }
    /// Test that UI plugin initialization works correctly
    fn test_ui_plugin_initialization() {
        
        // Verify resources are initialized
        assert!(app.world().contains_resource::<gameplay_ui::systems::ui::bugatti_telemetry::BugattiTelemetryState>());
        assert!(app.world().contains_resource::<gameplay_ui::systems::performance_monitor::UnifiedPerformanceTracker>());
        assert!(app.world().contains_resource::<gameplay_ui::systems::performance_dashboard::PerformanceDashboard>());
    /// Test headless UI rendering capability
    fn test_headless_ui_rendering() {
        let mut app = create_ui_test_app();
        // Run several update cycles to ensure UI systems work headlessly
        for _ in 0..10 {
            app.update();
        }
        // Verify no panics occurred and systems are functioning
        assert!(app.world().entities().len() > 0);
}
