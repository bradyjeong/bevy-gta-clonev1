//! Gameplay UI - HUD, menus, debug overlays
#![deny(warnings)]
#![deny(clippy::all, clippy::pedantic)]
#![deny(missing_docs)]

use bevy::prelude::*;

// Re-export dependencies
pub use engine_core;
pub use engine_bevy;
pub use game_core;
pub use gameplay_sim;
pub use gameplay_render;

// ─── Temporary compatibility re-exports (Phase-3) ────────────────
pub mod components { 
    /// Temporary re-export for compatibility
    pub use game_core::components::*; 
}

pub mod config { 
    /// Temporary re-export for compatibility
    pub use game_core::config::*; 
}

pub mod game_state { 
    /// Temporary re-export for compatibility
    pub use game_core::game_state::*; 
}

pub mod systems;

pub mod services {
    /// Temporary re-export for compatibility
    pub use gameplay_sim::services::*;
}

// Public modules
pub mod prelude;
pub(crate) mod ui;
pub(crate) mod debug;
pub(crate) mod performance;
pub(crate) mod plugins;
pub(crate) mod examples;
pub(crate) mod timing_service;
pub(crate) mod config_loader;
pub(crate) mod simple_service_example;

/// Main plugin for UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<ui::bugatti_telemetry::BugattiTelemetryState>()
            .init_resource::<performance::monitor::UnifiedPerformanceTracker>()
            .init_resource::<performance::dashboard::PerformanceDashboard>();
        
        // UI Display Systems
        app.add_systems(Update, (
            ui::fps_display::update_fps_display,
            ui::controls_ui::controls_ui_system,
            ui::bugatti_telemetry::bugatti_telemetry_input_system,
            ui::bugatti_telemetry::update_bugatti_telemetry_system,
        ));
        
        // Debug Systems
        app.add_systems(Update, (
            debug::debug::debug_game_state,
        ));
        
        // Performance Monitoring Systems
        app.add_systems(Update, (
            performance::monitor::unified_performance_monitoring_system,
            performance::dashboard::performance_dashboard_system,
            performance::integration::integrate_existing_performance_metrics,
            performance::integration::integrate_distance_cache_performance,
            performance::integration::monitor_vehicle_physics_performance,
            performance::integration::monitor_culling_performance,
            performance::integration::monitor_audio_performance,
        ));
        
        // Setup Systems (run once)
        app.add_systems(Startup, (
            ui::fps_display::setup_fps_display,
        ));
    }
}
