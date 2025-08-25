#[cfg(feature = "csv-metrics")]
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

#[cfg(feature = "csv-metrics")]
use std::fs::OpenOptions;

#[cfg(feature = "csv-metrics")]
use std::io::prelude::*;

#[cfg(feature = "csv-metrics")]
use super::metrics::{ENTITY_COUNT, CHUNK_COUNT, ACTIVE_CHUNKS};

/// Simple CSV export for performance metrics
///
/// Exports FPS, entity count, and chunk count to performance_metrics.csv
/// Only compiled when csv-metrics feature is enabled
#[cfg(feature = "csv-metrics")]
pub fn export_csv_metrics(diagnostics: Res<Diagnostics>) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let entities = diagnostics
        .get(ENTITY_COUNT)
        .and_then(|d| d.value())
        .unwrap_or(0.0) as u32;

    let chunks = diagnostics
        .get(CHUNK_COUNT)
        .and_then(|d| d.value())
        .unwrap_or(0.0) as u32;

    let active_chunks = diagnostics
        .get(ACTIVE_CHUNKS)
        .and_then(|d| d.value())
        .unwrap_or(0.0) as u32;

    let csv_line = format!("{},{:.1},{},{},{}\n", timestamp, fps, entities, chunks, active_chunks);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("performance_metrics.csv")
    {
        // Write header if file is new
        if file.metadata().map_or(true, |m| m.len() == 0) {
            let _ = file.write_all(b"timestamp,fps,entities,chunks,active_chunks\n");
        }
        let _ = file.write_all(csv_line.as_bytes());
    }
}

// Empty implementation when feature is disabled
#[cfg(not(feature = "csv-metrics"))]
pub fn export_csv_metrics() {}
