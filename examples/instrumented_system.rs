//! Example demonstrating how to instrument systems with the event metrics
//! 
//! Run with: cargo run --example instrumented_system --features debug-events

#![cfg_attr(not(feature = "debug-events"), allow(unused))]
use bevy::prelude::*;

#[cfg(not(feature = "debug-events"))]
fn main() {}

#[cfg(feature = "debug-events")]

#[cfg(feature = "debug-events")]
use gta_game::instrumentation::{EventMetricsPlugin, ScheduleOrderingPlugin};
use gta_game::events::{DynamicContentSpawned, DynamicContentDespawned, ContentType};

#[derive(Event, Clone, Copy)]
struct TestEvent {
    value: f32,
}

#[cfg(feature = "debug-events")]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EventMetricsPlugin)
        .add_plugins(ScheduleOrderingPlugin)
        .add_event::<TestEvent>()
        .add_event::<DynamicContentSpawned>()
        .add_event::<DynamicContentDespawned>()
        .add_systems(Update, (
            generate_test_events,
            handle_test_events,
            handle_spawn_events,
            handle_despawn_events,
        ).chain())
        .run();
}

fn generate_test_events(
    mut writer: EventWriter<TestEvent>,
    mut spawn_writer: EventWriter<DynamicContentSpawned>,
    mut despawn_writer: EventWriter<DynamicContentDespawned>,
    time: Res<Time>,
) {
    // Generate events at different rates
    let t = time.elapsed_secs();
    
    // High frequency test events
    for _ in 0..10 {
        writer.write(TestEvent { value: t.sin() });
    }
    
    // Medium frequency spawn events
    if t as u32 % 2 == 0 {
        for i in 0..5 {
            spawn_writer.write(DynamicContentSpawned {
                entity: Entity::from_raw(i),
                content_type: ContentType::Vehicle,
                position: Vec3::new(i as f32 * 10.0, 0.0, 0.0),
            });
        }
    }
    
    // Low frequency despawn events
    if t as u32 % 5 == 0 {
        despawn_writer.write(DynamicContentDespawned {
            entity: Entity::from_raw(1),
            content_type: ContentType::Vehicle,
        });
    }
}

#[cfg(feature = "debug-events")]
fn handle_test_events(
    mut reader: EventReader<TestEvent>,
    mut metrics: ResMut<gta_game::instrumentation::EventMetrics>,
) {
    use gta_game::instrument_events;
    
    // Use the instrumentation macro
    let events = instrument_events!(reader, "TestEvent", metrics);
    
    // Process events
    for event in events {
        // Simulate some work
        let _ = event.value * 2.0;
    }
}

#[cfg(not(feature = "debug-events"))]
fn handle_test_events(
    mut reader: EventReader<TestEvent>,
) {
    for event in reader.read() {
        // Same logic without instrumentation
        let _ = event.value * 2.0;
    }
}

#[cfg(feature = "debug-events")]
fn handle_spawn_events(
    mut reader: EventReader<DynamicContentSpawned>,
    mut metrics: ResMut<gta_game::instrumentation::EventMetrics>,
    mut profiler: ResMut<gta_game::instrumentation::SystemProfiler>,
) {
    use gta_game::instrument_events;
    use gta_game::profiled_system;
    
    profiled_system!("handle_spawn_events", profiler, {
        let events = instrument_events!(reader, "DynamicContentSpawned", metrics);
        
        for event in events {
            // Simulate spawn processing
            info!("Spawned {:?} at {:?}", event.content_type, event.position);
        }
    });
}

#[cfg(not(feature = "debug-events"))]
fn handle_spawn_events(
    mut reader: EventReader<DynamicContentSpawned>,
) {
    for event in reader.read() {
        info!("Spawned {:?} at {:?}", event.content_type, event.position);
    }
}

#[cfg(feature = "debug-events")]
fn handle_despawn_events(
    mut reader: EventReader<DynamicContentDespawned>,
    mut metrics: ResMut<gta_game::instrumentation::EventMetrics>,
    mut system_metrics: ResMut<gta_game::instrumentation::SystemMetrics>,
) {
    use std::time::Instant;
    
    // Manual system timing
    let start = Instant::now();
    
    let events: Vec<_> = reader.read().collect();
    if !events.is_empty() {
        metrics.record_event("DynamicContentDespawned", events.len());
    }
    
    for event in events {
        // Simulate despawn processing
        info!("Despawned {:?}", event.content_type);
    }
    
    system_metrics.record("handle_despawn_events", start.elapsed());
}

#[cfg(not(feature = "debug-events"))]
fn handle_despawn_events(
    mut reader: EventReader<DynamicContentDespawned>,
) {
    for event in reader.read() {
        info!("Despawned {:?}", event.content_type);
    }
}
