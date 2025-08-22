use crate::components::PerformanceStats;
use bevy::render::view::visibility::VisibilityRange;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub fn performance_monitoring_system(
    time: Res<Time>,
    mut stats: ResMut<PerformanceStats>,
    entity_query: Query<Entity>,
    _visibility_query: Query<&VisibilityRange>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let current_time = time.elapsed_secs();

    // Update stats
    stats.entity_count = entity_query.iter().count();
    // Note: With VisibilityRange, culling is handled automatically by Bevy
    // We can track entities with visibility ranges, but not "culled" state
    stats.culled_entities = 0; // This metric is deprecated with VisibilityRange

    // Get frame time from diagnostics
    if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_avg) = fps_diag.smoothed() {
            stats.frame_time = (1000.0 / fps_avg) as f32; // Convert to milliseconds
        }
    }

    // Report every 5 seconds
    if current_time - stats.last_report > 5.0 {
        stats.last_report = current_time;
        info!(
            "PERFORMANCE: Entities: {} | Culled: {} | Frame: {:.1}ms | FPS: {:.0}",
            stats.entity_count,
            stats.culled_entities,
            stats.frame_time,
            if stats.frame_time > 0.0 {
                1000.0 / stats.frame_time
            } else {
                0.0
            }
        );
    }
}
