//! ───────────────────────────────────────────────
//! System:   Performance
//! Purpose:  Monitors and optimizes performance
//! Schedule: Update
//! Reads:    DiagnosticsStore, Cullable, PerformanceStats, Time
//! Writes:   PerformanceStats
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use game_core::prelude::{PerformanceStats, Cullable};

pub fn performance_monitoring_system(
    time: Res<Time>,
    mut stats: ResMut<PerformanceStats>,
    entity_query: Query<Entity>,
    cullable_query: Query<&Cullable>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let current_time = time.elapsed_secs();
    
    // Update stats
    stats.entity_count = entity_query.iter().count();
    stats.culled_entities = cullable_query.iter().filter(|c| c.is_culled).count();
    
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
            if stats.frame_time > 0.0 { 1000.0 / stats.frame_time } else { 0.0 }
        );
    }
}
