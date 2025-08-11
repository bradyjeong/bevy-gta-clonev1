//! Event audit system for tracking event frequency and performance
//! 
//! This module provides instrumentation for monitoring event usage patterns,
//! helping identify candidates for Observer pattern conversion.

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

use crate::events::{
    distance_events::*,
    world::content_events::*,
    world::chunk_events::*,
    world::validation_events::*,
    ground_events::*,
};

/// Resource tracking event counts and timing
#[derive(Resource, Default)]
pub struct EventAuditStats {
    /// Event counts by type name
    pub event_counts: HashMap<String, EventCounter>,
    /// Last reset time
    pub last_reset: Option<Instant>,
    /// Whether audit is enabled
    pub enabled: bool,
}

#[derive(Default, Debug, Clone)]
pub struct EventCounter {
    pub total_count: u64,
    pub current_frame_count: u32,
    pub peak_per_frame: u32,
    pub average_per_second: f32,
    pub last_triggered: Option<Instant>,
}

impl EventCounter {
    #[allow(dead_code)]
    fn record(&mut self) {
        self.total_count += 1;
        self.current_frame_count += 1;
        self.peak_per_frame = self.peak_per_frame.max(self.current_frame_count);
        self.last_triggered = Some(Instant::now());
    }
    
    fn reset_frame(&mut self) {
        self.current_frame_count = 0;
    }
}

/// Plugin providing event audit functionality
pub struct EventAuditPlugin;

impl Plugin for EventAuditPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EventAuditStats>()
            .add_systems(
                PreUpdate,
                reset_frame_counters.run_if(audit_enabled)
            )
            .add_systems(
                Update,
                (
                    // Distance events
                    audit_distance_events,
                    // Content events  
                    audit_content_events,
                    // Chunk events
                    audit_chunk_events,
                    // Validation events
                    audit_validation_events,
                    // Ground events
                    audit_ground_events,
                )
                    .run_if(audit_enabled)
                    .in_set(EventAuditSet)
            )
            .add_systems(
                PostUpdate,
                calculate_averages.run_if(audit_enabled)
            )
            .configure_sets(Update, EventAuditSet);
    }
}

/// System set for event audit systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventAuditSet;

fn audit_enabled(stats: Res<EventAuditStats>) -> bool {
    stats.enabled
}

fn reset_frame_counters(mut stats: ResMut<EventAuditStats>) {
    for counter in stats.event_counts.values_mut() {
        counter.reset_frame();
    }
}

fn audit_distance_events(
    mut stats: ResMut<EventAuditStats>,
    req_distance: EventReader<RequestDistance>,
    req_distance_ref: EventReader<RequestDistanceToReference>,
    res_distance: EventReader<DistanceResult>,
    res_distance_ref: EventReader<DistanceToReferenceResult>,
) {
    let req_count = req_distance.len();
    if req_count > 0 {
        stats.event_counts
            .entry("RequestDistance".to_string())
            .or_default()
            .total_count += req_count as u64;
    }
    
    let req_ref_count = req_distance_ref.len();
    if req_ref_count > 0 {
        stats.event_counts
            .entry("RequestDistanceToReference".to_string())
            .or_default()
            .total_count += req_ref_count as u64;
    }
    
    let res_count = res_distance.len();
    if res_count > 0 {
        stats.event_counts
            .entry("DistanceResult".to_string())
            .or_default()
            .total_count += res_count as u64;
    }
    
    let res_ref_count = res_distance_ref.len();
    if res_ref_count > 0 {
        stats.event_counts
            .entry("DistanceToReferenceResult".to_string())
            .or_default()
            .total_count += res_ref_count as u64;
    }
}

fn audit_content_events(
    mut stats: ResMut<EventAuditStats>,
    req_spawn: EventReader<RequestDynamicSpawn>,
    spawned: EventReader<DynamicContentSpawned>,
    req_despawn: EventReader<RequestDynamicDespawn>,
    despawned: EventReader<DynamicContentDespawned>,
) {
    let req_spawn_count = req_spawn.len();
    if req_spawn_count > 0 {
        stats.event_counts
            .entry("RequestDynamicSpawn".to_string())
            .or_default()
            .total_count += req_spawn_count as u64;
    }
    
    let spawned_count = spawned.len();
    if spawned_count > 0 {
        let counter = stats.event_counts
            .entry("DynamicContentSpawned".to_string())
            .or_default();
        counter.total_count += spawned_count as u64;
        counter.current_frame_count += spawned_count as u32;
        counter.peak_per_frame = counter.peak_per_frame.max(spawned_count as u32);
        counter.last_triggered = Some(Instant::now());
    }
    
    let req_despawn_count = req_despawn.len();
    if req_despawn_count > 0 {
        stats.event_counts
            .entry("RequestDynamicDespawn".to_string())
            .or_default()
            .total_count += req_despawn_count as u64;
    }
    
    let despawned_count = despawned.len();
    if despawned_count > 0 {
        let counter = stats.event_counts
            .entry("DynamicContentDespawned".to_string())
            .or_default();
        counter.total_count += despawned_count as u64;
        counter.current_frame_count += despawned_count as u32;
        counter.peak_per_frame = counter.peak_per_frame.max(despawned_count as u32);
        counter.last_triggered = Some(Instant::now());
    }
}

fn audit_chunk_events(
    mut stats: ResMut<EventAuditStats>,
    req_load: EventReader<RequestChunkLoad>,
    loaded: EventReader<ChunkLoaded>,
    req_unload: EventReader<RequestChunkUnload>,
    unloaded: EventReader<ChunkUnloaded>,
) {
    record_event_count(&mut stats, "RequestChunkLoad", req_load.len());
    record_event_count(&mut stats, "ChunkLoaded", loaded.len());
    record_event_count(&mut stats, "RequestChunkUnload", req_unload.len());
    record_event_count(&mut stats, "ChunkUnloaded", unloaded.len());
}

fn audit_validation_events(
    mut stats: ResMut<EventAuditStats>,
    req_spawn_val: EventReader<RequestSpawnValidation>,
    spawn_val_res: EventReader<SpawnValidationResult>,
    req_road_val: EventReader<RequestRoadValidation>,
    road_val_res: EventReader<RoadValidationResult>,
    req_pos_val: EventReader<RequestSpawnPositionValidation>,
    pos_val_res: EventReader<SpawnPositionValidationResult>,
) {
    record_event_count(&mut stats, "RequestSpawnValidation", req_spawn_val.len());
    record_event_count(&mut stats, "SpawnValidationResult", spawn_val_res.len());
    record_event_count(&mut stats, "RequestRoadValidation", req_road_val.len());
    record_event_count(&mut stats, "RoadValidationResult", road_val_res.len());
    record_event_count(&mut stats, "RequestSpawnPositionValidation", req_pos_val.len());
    record_event_count(&mut stats, "SpawnPositionValidationResult", pos_val_res.len());
}

fn audit_ground_events(
    mut stats: ResMut<EventAuditStats>,
    req_height: EventReader<RequestGroundHeight>,
    height_res: EventReader<GroundHeightResult>,
) {
    record_event_count(&mut stats, "RequestGroundHeight", req_height.len());
    record_event_count(&mut stats, "GroundHeightResult", height_res.len());
}

fn record_event_count(stats: &mut EventAuditStats, event_name: &str, count: usize) {
    if count > 0 {
        let counter = stats.event_counts
            .entry(event_name.to_string())
            .or_default();
        counter.total_count += count as u64;
        counter.current_frame_count += count as u32;
        counter.peak_per_frame = counter.peak_per_frame.max(count as u32);
        counter.last_triggered = Some(Instant::now());
    }
}

fn calculate_averages(
    mut stats: ResMut<EventAuditStats>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs();
    if elapsed > 0.0 {
        for counter in stats.event_counts.values_mut() {
            counter.average_per_second = counter.total_count as f32 / elapsed;
        }
    }
}

/// Integration with debug overlay (F3)
pub fn render_event_audit_overlay(
    stats: Res<EventAuditStats>,
    mut text_query: Query<&mut Text, With<EventAuditOverlay>>,
) {
    let Some(mut text) = text_query.iter_mut().next() else { return };
    
    let mut lines = vec!["=== Event Audit ===".to_string()];
    
    // Sort events by total count
    let mut sorted_events: Vec<_> = stats.event_counts.iter().collect();
    sorted_events.sort_by(|a, b| b.1.total_count.cmp(&a.1.total_count));
    
    // Show top 10 events
    for (name, counter) in sorted_events.iter().take(10) {
        lines.push(format!(
            "{}: {} total, {:.1}/s, peak: {}/frame",
            name,
            counter.total_count,
            counter.average_per_second,
            counter.peak_per_frame
        ));
    }
    
    // Highlight entity-specific events
    lines.push("\n=== Entity-Specific Events ===".to_string());
    let entity_events = [
        "DynamicContentSpawned",
        "DynamicContentDespawned", 
        "RequestDynamicDespawn",
        "DistanceResult",
        "DistanceToReferenceResult",
    ];
    
    for event_name in &entity_events {
        if let Some(counter) = stats.event_counts.get(*event_name) {
            lines.push(format!(
                "{}: {} ({:.1}/s) [OBSERVER CANDIDATE]",
                event_name,
                counter.total_count,
                counter.average_per_second
            ));
        }
    }
    
    text.0 = lines.join("\n");
}

/// Marker component for event audit overlay text
#[derive(Component)]
pub struct EventAuditOverlay;

/// Command to toggle event audit
pub fn toggle_event_audit(mut stats: ResMut<EventAuditStats>) {
    stats.enabled = !stats.enabled;
    if stats.enabled {
        stats.last_reset = Some(Instant::now());
        info!("Event audit enabled");
    } else {
        info!("Event audit disabled");
    }
}
