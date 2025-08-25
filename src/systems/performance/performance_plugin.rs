use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin, DiagnosticsPlugin},
    prelude::*,
};

#[cfg(feature = "debug-physics")]
use bevy_rapier3d::render::RapierDebugRenderPlugin;

#[cfg(feature = "csv-metrics")]
use std::time::Duration;

use super::metrics::{setup_custom_diagnostics, record_entity_count, record_chunk_count, record_active_chunks};

#[cfg(feature = "csv-metrics")]
use super::csv_export::export_csv_metrics;

/// Simple performance monitoring plugin using Bevy's built-in diagnostics
///
/// Replaces the complex 780-line UnifiedPerformanceTracker with:
/// - Bevy's built-in FrameTimeDiagnosticsPlugin for FPS tracking
/// - Bevy's built-in LogDiagnosticsPlugin for console output
/// - Simple systems for custom metrics (entities, chunks)
/// - Optional Rapier debug rendering when debug-physics feature enabled
/// - Optional CSV export when csv-metrics feature enabled
pub struct PerformancePlugin;

impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy's built-in diagnostic plugins
        app.add_plugins((
            DiagnosticsPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ));

        // Setup custom diagnostics at startup
        app.add_systems(Startup, setup_custom_diagnostics);

        // Add custom metric recording systems
        app.add_systems(
            Update,
            (record_entity_count, record_chunk_count, record_active_chunks),
        );

        // Add physics debug rendering if feature enabled
        #[cfg(feature = "debug-physics")]
        app.add_plugins(RapierDebugRenderPlugin::default());

        // Add CSV export if feature enabled
        #[cfg(feature = "csv-metrics")]
        app.add_systems(
            Update,
            export_csv_metrics.run_if(on_timer(Duration::from_secs(1))),
        );
    }
}
