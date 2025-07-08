use bevy::prelude::*;
use gameplay_ui::prelude::*;

mod utils;
use utils::*;

/// Test that UI plugin can be instantiated
#[test]
fn test_ui_plugin_creation() {
    let _plugin = UiPlugin;
    // Should not panic
    assert!(true);
}

/// Test headless app creation with UI plugin
#[test]
fn test_headless_app_basic() {
    let app = create_ui_test_app();
    // App should initialize
    assert!(app.world().entities().len() > 0);
}

/// Test basic UI resource initialization
#[test]
fn test_ui_resources_init() {
    let app = create_ui_test_app();
    // Check that core UI resources exist
    assert!(app.world().contains_resource::<gameplay_ui::systems::ui::bugatti_telemetry::BugattiTelemetryState>());
    assert!(app.world().contains_resource::<gameplay_ui::systems::performance_monitor::UnifiedPerformanceTracker>());
}

/// Test headless update cycles
#[test]
fn test_headless_updates() {
    let mut app = create_ui_test_app();
    // Run several update cycles
    for _ in 0..10 {
        app.update();
    }
    // Should complete without panicking
}
