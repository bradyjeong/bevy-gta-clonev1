//! Gameplay UI - HUD, menus, debug overlays
#![deny(clippy::all, clippy::pedantic)]
#![warn(missing_docs)]

use bevy::prelude::*;
pub use engine_core;
pub use engine_bevy;
pub use game_core;
pub use gameplay_sim;

pub mod prelude;
pub mod systems;

pub use prelude::*;

/// Main plugin for UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<systems::ui::bugatti_telemetry::BugattiTelemetryState>()
            .init_resource::<systems::performance_monitor::UnifiedPerformanceTracker>()
            .init_resource::<systems::performance_dashboard::PerformanceDashboard>();

        // UI Display Systems
        app.add_systems(Update, (
            systems::ui::fps_display::update_fps_display,
            systems::ui::controls_ui::controls_ui_system,
            systems::ui::bugatti_telemetry::bugatti_telemetry_input_system,
            systems::ui::bugatti_telemetry::update_bugatti_telemetry_system,
        ));

        // Debug Systems
        app.add_systems(Update, (
            systems::debug::debug_game_state,
        ));

        // Performance Monitoring Systems
        app.add_systems(Update, (
            systems::performance_monitor::unified_performance_monitoring_system,
            systems::performance_dashboard::performance_dashboard_system,
            systems::performance_integration::integrate_existing_performance_metrics,
            systems::performance_integration::integrate_distance_cache_performance,
            systems::performance_integration::monitor_vehicle_physics_performance,
            systems::performance_integration::monitor_culling_performance,
            systems::performance_integration::monitor_audio_performance,
        ));

        // Setup Systems (run once)
        app.add_systems(Startup, (
            systems::ui::fps_display::setup_fps_display,
        ));
    }
}
