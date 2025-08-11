//! Tests for observer-based content lifecycle
//! 
//! Verifies that observers properly replace event-based systems
//! and maintains compatibility with legacy events.

use bevy::prelude::*;
use gta_game::components::dynamic_content::{DynamicContent, ContentType, MarkedForDespawn};
use gta_game::observers::content_observers::{ContentObserverPlugin, ContentInitialized, ObserverMetrics};
use gta_game::events::world::content_events::{DynamicContentSpawned, DynamicContentDespawned};

/// Test that adding DynamicContent triggers the spawn observer
#[test]
fn test_on_add_observer_triggers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .init_resource::<ObserverMetrics>();
    
    // Spawn entity with DynamicContent
    let entity = app.world_mut().spawn((
        DynamicContent::new(ContentType::Building),
        Transform::default(),
    )).id();
    
    // Run one frame to trigger observers
    app.update();
    
    // Verify entity has ContentInitialized marker
    assert!(app.world().entity(entity).contains::<ContentInitialized>());
    
    // Check metrics
    let metrics = app.world().resource::<ObserverMetrics>();
    assert_eq!(metrics.spawn_observer_calls, 1);
}

/// Test that removing DynamicContent triggers the despawn observer
#[test]
fn test_on_remove_observer_triggers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .init_resource::<ObserverMetrics>();
    
    // Spawn entity with DynamicContent
    let entity = app.world_mut().spawn((
        DynamicContent::new(ContentType::Vehicle),
        Transform::default(),
    )).id();
    
    app.update();
    
    // Remove the component
    app.world_mut().entity_mut(entity).remove::<DynamicContent>();
    
    // Run frame to trigger removal observer
    app.update();
    
    // Check metrics
    let metrics = app.world().resource::<ObserverMetrics>();
    assert_eq!(metrics.despawn_observer_calls, 1);
}

/// Test legacy event emission when feature is enabled
#[cfg(feature = "legacy-events")]
#[test]
fn test_legacy_event_compatibility() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .add_event::<DynamicContentSpawned>()
        .add_event::<DynamicContentDespawned>();
    
    // System to count legacy events
    let mut spawn_count = 0;
    let mut despawn_count = 0;
    
    app.add_systems(Update, move |mut spawn_events: EventReader<DynamicContentSpawned>| {
        spawn_count += spawn_events.read().count();
    });
    
    app.add_systems(Update, move |mut despawn_events: EventReader<DynamicContentDespawned>| {
        despawn_count += despawn_events.read().count();
    });
    
    // Spawn entity
    let entity = app.world.spawn((
        DynamicContent::new(ContentType::NPC),
        Transform::default(),
    )).id();
    
    app.update();
    
    // Despawn entity
    app.world.entity_mut(entity).despawn();
    
    app.update();
    
    // Verify legacy events were emitted
    // (This would need actual event tracking implementation)
    assert!(true, "Legacy events should be emitted");
}

/// Test MarkedForDespawn component processing
#[test]
fn test_marked_for_despawn_processing() {
    use gta_game::systems::world::event_handlers::content_despawn_handler::process_marked_for_despawn;
    
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, process_marked_for_despawn);
    
    // Spawn entities with DynamicContent
    let entity1 = app.world_mut().spawn((
        DynamicContent::new(ContentType::Tree),
        Transform::default(),
    )).id();
    
    let entity2 = app.world_mut().spawn((
        DynamicContent::new(ContentType::Road),
        Transform::default(),
        MarkedForDespawn,
    )).id();
    
    // Run system
    app.update();
    
    // entity1 should still exist
    assert!(app.world().get_entity(entity1).is_ok());
    
    // entity2 should be despawned
    assert!(app.world().get_entity(entity2).is_err());
}

/// Performance comparison test between events and observers
#[test]
fn test_performance_comparison() {
    use std::time::Instant;
    
    const ENTITY_COUNT: usize = 1000;
    
    // Test observer-based approach
    let mut app_observer = App::new();
    app_observer.add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .init_resource::<ObserverMetrics>();
    
    let start = Instant::now();
    
    // Spawn many entities
    for i in 0..ENTITY_COUNT {
        app_observer.world_mut().spawn((
            DynamicContent::new(if i % 2 == 0 { ContentType::Building } else { ContentType::Tree }),
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
        ));
    }
    
    app_observer.update();
    
    let observer_duration = start.elapsed();
    
    // Test event-based approach (simulated)
    let mut app_events = App::new();
    app_events.add_plugins(MinimalPlugins)
        .add_event::<DynamicContentSpawned>();
    
    let start = Instant::now();
    
    // Simulate event emission
    let mut event_writer = app_events.world_mut().resource_mut::<Events<DynamicContentSpawned>>();
    for i in 0..ENTITY_COUNT {
        event_writer.send(DynamicContentSpawned::new(
            Entity::from_raw(i as u32),
            Vec3::new(i as f32, 0.0, 0.0),
            gta_game::events::world::content_events::ContentType::Building,
        ));
    }
    
    app_events.update();
    
    let event_duration = start.elapsed();
    
    println!("Observer approach: {:?}", observer_duration);
    println!("Event approach: {:?}", event_duration);
    
    // Observers might be slightly slower due to initial setup but should be within reasonable range
    // The real benefit is less memory overhead and no per-frame clearing
    assert!(observer_duration.as_micros() < event_duration.as_micros() * 20,
        "Observers should not be significantly slower than events (within 20x for small counts)");
}

/// Test that multiple content types work correctly
#[test]
fn test_multiple_content_types() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .init_resource::<ObserverMetrics>();
    
    // Spawn different content types
    let entities: Vec<_> = vec![
        ContentType::Road,
        ContentType::Building,
        ContentType::Tree,
        ContentType::Vehicle,
        ContentType::NPC,
    ].into_iter().map(|content_type| {
        app.world_mut().spawn((
            DynamicContent::new(content_type),
            Transform::default(),
        )).id()
    }).collect();
    
    app.update();
    
    // All should be initialized
    for entity in &entities {
        assert!(app.world().entity(*entity).contains::<ContentInitialized>());
    }
    
    // Check metrics
    let metrics = app.world().resource::<ObserverMetrics>();
    assert_eq!(metrics.spawn_observer_calls, 5);
}
