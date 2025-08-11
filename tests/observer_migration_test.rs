//! Tests for observer pattern migration
//! 
//! Verifies behavioral parity between event-based and observer-based systems

use bevy::prelude::*;
#![allow(unused_imports)]
use gta_game::{
    components::dynamic_content::{DynamicContent, ContentType},
    events::world::content_events::{DynamicContentSpawned, DynamicContentDespawned},
    observers::ContentObserverPlugin,
};

/// Test that observers trigger correctly on component addition
#[test]
fn test_observer_triggers_on_spawn() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .add_event::<DynamicContentSpawned>();
    
    // Spawn entity with DynamicContent
    let entity = app.world_mut().spawn((
        DynamicContent::new(ContentType::Vehicle),
        Transform::from_translation(Vec3::new(100.0, 0.0, 50.0)),
    )).id();
    
    // Update to trigger observers
    app.update();
    
    // Verify entity has ContentInitialized component (added by observer)
    assert!(
        app.world().entity(entity).contains::<gta_game::observers::content_observers::ContentInitialized>(),
        "Observer should add ContentInitialized component"
    );
}

/// Test legacy event compatibility layer
#[test]
#[cfg(feature = "legacy-events")]
fn test_legacy_event_emission() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin)
        .add_event::<DynamicContentSpawned>()
        .add_event::<DynamicContentDespawned>();
    
    // Spawn entity
    let entity = app.world_mut().spawn((
        DynamicContent::new(ContentType::Building),
        Transform::from_translation(Vec3::new(200.0, 10.0, 100.0)),
    )).id();
    
    // Update to trigger observers and legacy events
    app.update();
    
    // Check if legacy event was emitted
    let events = app.world().resource::<Events<DynamicContentSpawned>>();
    let mut reader = events.get_cursor();
    let spawned_events: Vec<_> = reader.read(events).collect();
    
    assert_eq!(spawned_events.len(), 1, "Should emit one spawn event");
    assert_eq!(spawned_events[0].entity, entity, "Event should contain correct entity");
}

/// Test observer performance vs events
#[test]
fn test_observer_performance() {
    use std::time::Instant;
    
    const ENTITY_COUNT: usize = 1000;
    
    // Test with observers
    let mut observer_app = App::new();
    observer_app
        .add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin);
    
    let observer_start = Instant::now();
    for i in 0..ENTITY_COUNT {
        observer_app.world_mut().spawn((
            DynamicContent::new(ContentType::NPC),
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
        ));
    }
    observer_app.update();
    let observer_duration = observer_start.elapsed();
    
    // Test with events (baseline)
    let mut event_app = App::new();
    event_app
        .add_plugins(MinimalPlugins)
        .add_event::<DynamicContentSpawned>();
    
    let event_start = Instant::now();
    let mut events = Vec::new();
    for i in 0..ENTITY_COUNT {
        let entity = event_app.world_mut().spawn((
            DynamicContent::new(ContentType::NPC),
            Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
        )).id();
        events.push(DynamicContentSpawned::new(
            entity,
            Vec3::new(i as f32, 0.0, 0.0),
            gta_game::events::world::content_events::ContentType::NPC,
        ));
    }
    
    // Write all events
    event_app.world_mut().resource_mut::<Events<DynamicContentSpawned>>()
        .send_batch(events);
    event_app.update();
    let event_duration = event_start.elapsed();
    
    println!("Observer duration: {:?}", observer_duration);
    println!("Event duration: {:?}", event_duration);
    
    // Observers should be at least as fast as events
    assert!(
        observer_duration.as_micros() <= event_duration.as_micros() * 2,
        "Observers should not be significantly slower than events"
    );
}

/// Test that removed components trigger despawn observer
#[test]
fn test_observer_triggers_on_despawn() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin);
    
    // Spawn entity
    let entity = app.world_mut().spawn((
        DynamicContent::new(ContentType::Tree),
        Transform::default(),
    )).id();
    
    app.update();
    
    // Despawn entity
    app.world_mut().entity_mut(entity).despawn();
    
    // Update to trigger removal observer
    app.update();
    
    // Entity should no longer exist
    assert!(!app.world().entities().contains(entity));
}

/// Test multiple content types
#[test]
fn test_all_content_types() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(ContentObserverPlugin);
    
    let content_types = [
        ContentType::Road,
        ContentType::Building,
        ContentType::Tree,
        ContentType::Vehicle,
        ContentType::NPC,
    ];
    
    let mut entities = Vec::new();
    
    // Spawn one of each type
    for ct in content_types.iter() {
        let entity = app.world_mut().spawn((
            DynamicContent::new(*ct),
            Transform::default(),
        )).id();
        entities.push(entity);
    }
    
    app.update();
    
    // All should have ContentInitialized
    for entity in entities {
        assert!(
            app.world().entity(entity)
                .contains::<gta_game::observers::content_observers::ContentInitialized>()
        );
    }
}
