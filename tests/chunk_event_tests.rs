//! Integration tests for chunk loading event-driven architecture
//! 
//! Tests the complete event flow for chunk loading, unloading,
//! and dynamic content spawning.

use bevy::prelude::*;
use gta_game::{
    events::world::content_events::*,
    world::ChunkTracker,
    plugins::WorldStreamingPlugin,
};

// Import the chunk events and use ChunkCoord from chunk_events
use gta_game::events::world::chunk_events::{
    ChunkCoord, RequestChunkLoad, ChunkLoaded, RequestChunkUnload, ChunkUnloaded, ChunkFinishedLoading
};

/// Test that RequestChunkLoad events trigger ChunkLoaded responses
#[test]
fn test_chunk_load_event_flow() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<ChunkFinishedLoading>();
    
    // Send request to load a chunk
    let coord = ChunkCoord { x: 0, z: 0 };
    app.world_mut()
        .send_event(RequestChunkLoad { coord });
    
    // Update to process the event
    app.update();
    
    // Check that ChunkLoaded event was emitted
    let loaded_events = app.world()
        .resource::<Events<ChunkLoaded>>();
    let mut reader = loaded_events.get_cursor();
    let events: Vec<_> = reader.read(loaded_events).collect();
    
    assert_eq!(events.len(), 1, "Should emit one ChunkLoaded event");
    assert_eq!(events[0].coord, coord, "ChunkLoaded should have correct coordinate");
}

/// Test that multiple chunk load requests are handled correctly
#[test]
fn test_multiple_chunk_loads() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>();
    
    // Send multiple chunk load requests
    let coords = vec![
        ChunkCoord { x: 0, z: 0 },
        ChunkCoord { x: 1, z: 0 },
        ChunkCoord { x: 0, z: 1 },
        ChunkCoord { x: 1, z: 1 },
    ];
    
    for coord in &coords {
        app.world_mut()
            .send_event(RequestChunkLoad { coord: *coord });
    }
    
    app.update();
    
    // Verify all chunks generated events
    let loaded_events = app.world()
        .resource::<Events<ChunkLoaded>>();
    let mut reader = loaded_events.get_cursor();
    let events: Vec<_> = reader.read(loaded_events).collect();
    
    assert_eq!(events.len(), coords.len(), "Should load all requested chunks");
    
    // Verify each coordinate has a corresponding event
    let loaded_coords: Vec<_> = events.iter().map(|e| e.coord).collect();
    for coord in coords {
        assert!(
            loaded_coords.contains(&coord),
            "Chunk {:?} should be loaded", coord
        );
    }
}

/// Test chunk unload event flow
#[test]
fn test_chunk_unload_event_flow() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<RequestChunkUnload>()
        .add_event::<ChunkUnloaded>();
    
    let coord = ChunkCoord { x: 2, z: 3 };
    
    // First load the chunk
    app.world_mut()
        .send_event(RequestChunkLoad { coord });
    app.update();
    
    // Then request unload
    app.world_mut()
        .send_event(RequestChunkUnload { coord });
    app.update();
    
    // Verify unload event was emitted
    let unloaded_events = app.world()
        .resource::<Events<ChunkUnloaded>>();
    let mut reader = unloaded_events.get_cursor();
    let events: Vec<_> = reader.read(unloaded_events).collect();
    
    assert_eq!(events.len(), 1, "Should emit one ChunkUnloaded event");
    assert_eq!(events[0].coord, coord, "ChunkUnloaded should have correct coordinate");
}

/// Test that ChunkTracker state updates correctly with events
#[test]
fn test_chunk_tracker_event_integration() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .insert_resource(ChunkTracker::new());
    
    let coord = ChunkCoord { x: 5, z: -3 };
    
    // Send load request
    app.world_mut()
        .send_event(RequestChunkLoad { coord });
    app.update();
    
    // Check ChunkTracker was updated
    let tracker = app.world().resource::<ChunkTracker>();
    // Convert to world::ChunkCoord for ChunkTracker
    let world_coord = gta_game::world::ChunkCoord { x: coord.x, z: coord.z };
    assert!(
        tracker.is_chunk_loaded(world_coord),
        "ChunkTracker should mark chunk as loaded"
    );
}

/// Test RequestDynamicSpawn event triggers entity creation
#[test]
fn test_dynamic_spawn_event_flow() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>();
    
    let spawn_pos = Vec3::new(100.0, 0.0, 50.0);
    
    // Request vehicle spawn
    app.world_mut()
        .send_event(RequestDynamicSpawn::vehicle(spawn_pos));
    
    app.update();
    
    // Check spawned event was emitted
    let spawned_events = app.world()
        .resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let events: Vec<_> = reader.read(spawned_events).collect();
    
    assert_eq!(events.len(), 1, "Should emit one DynamicContentSpawned event");
    assert_eq!(events[0].pos, spawn_pos, "Spawned at correct position");
    assert_eq!(events[0].kind, ContentType::Vehicle, "Spawned correct type");
}

/// Test batch dynamic content spawning
#[test]
fn test_batch_dynamic_spawns() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>();
    
    // Request multiple spawns of different types
    let spawns = vec![
        RequestDynamicSpawn::building(Vec3::new(0.0, 0.0, 0.0)),
        RequestDynamicSpawn::vehicle(Vec3::new(50.0, 0.0, 0.0)),
        RequestDynamicSpawn::npc(Vec3::new(100.0, 0.0, 0.0)),
        RequestDynamicSpawn { pos: Vec3::new(150.0, 0.0, 0.0), kind: ContentType::Tree },
    ];
    
    for spawn in &spawns {
        app.world_mut().send_event(*spawn);
    }
    
    app.update();
    
    // Verify all content was spawned
    let spawned_events = app.world()
        .resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let events: Vec<_> = reader.read(spawned_events).collect();
    
    assert_eq!(events.len(), spawns.len(), "Should spawn all requested content");
    
    // Verify entities exist in world
    for event in &events {
        assert!(
            app.world().entities().contains(event.entity),
            "Spawned entity should exist in world"
        );
    }
}

/// Test dynamic despawn event flow
#[test]
fn test_dynamic_despawn_event_flow() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>()
        .add_event::<RequestDynamicDespawn>()
        .add_event::<DynamicContentDespawned>();
    
    // First spawn an entity
    app.world_mut()
        .send_event(RequestDynamicSpawn::npc(Vec3::ZERO));
    app.update();
    
    // Get the spawned entity
    let spawned_events = app.world()
        .resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let entity = reader.read(spawned_events).next().unwrap().entity;
    
    // Request despawn
    app.world_mut()
        .send_event(RequestDynamicDespawn { entity });
    app.update();
    
    // Verify despawn event
    let despawned_events = app.world()
        .resource::<Events<DynamicContentDespawned>>();
    let mut reader = despawned_events.get_cursor();
    let events: Vec<_> = reader.read(despawned_events).collect();
    
    assert_eq!(events.len(), 1, "Should emit despawn event");
    assert_eq!(events[0].entity, entity, "Correct entity despawned");
    
    // Verify entity no longer exists
    assert!(
        !app.world().entities().contains(entity),
        "Entity should be removed from world"
    );
}

/// Test chunk load triggers dynamic content spawning
#[test]
fn test_chunk_load_spawns_content() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>();
    
    // Load a chunk
    let coord = ChunkCoord { x: 0, z: 0 };
    app.world_mut()
        .send_event(RequestChunkLoad { coord });
    app.update();
    
    // Check that content was spawned in response
    let spawned_events = app.world()
        .resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let events: Vec<_> = reader.read(spawned_events).collect();
    
    assert!(
        events.len() > 0,
        "Chunk load should trigger content spawning"
    );
}

/// Test ChunkFinishedLoading event
#[test]
fn test_chunk_finished_loading_event() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<ChunkFinishedLoading>();
    
    let coord = ChunkCoord { x: 1, z: 1 };
    
    // Request chunk load
    app.world_mut()
        .send_event(RequestChunkLoad { coord });
    app.update();
    
    // Check for finished loading event
    let finished_events = app.world()
        .resource::<Events<ChunkFinishedLoading>>();
    let mut reader = finished_events.get_cursor();
    let events: Vec<_> = reader.read(finished_events).collect();
    
    assert_eq!(events.len(), 1, "Should emit ChunkFinishedLoading");
    assert_eq!(events[0].coord, coord, "Correct chunk finished loading");
}

/// Test that events are properly cleared each frame
#[test]
fn test_event_clearing() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>();
    
    // Send event and process
    app.world_mut()
        .send_event(RequestChunkLoad { coord: ChunkCoord { x: 0, z: 0 } });
    app.update();
    
    // Update again without sending new events
    app.update();
    
    // Events should be cleared
    let loaded_events = app.world()
        .resource::<Events<ChunkLoaded>>();
    let mut reader = loaded_events.get_cursor();
    let events: Vec<_> = reader.read(loaded_events).collect();
    
    assert_eq!(events.len(), 0, "Events should be cleared after frame");
}

/// Test coordinate conversion from world position
#[test]
fn test_chunk_coord_from_world_pos() {
    const CHUNK_SIZE: f32 = 100.0;
    
    // Test positive coordinates
    let pos = Vec3::new(150.0, 0.0, 250.0);
    let coord = ChunkCoord {
        x: (pos.x / CHUNK_SIZE).floor() as i32,
        z: (pos.z / CHUNK_SIZE).floor() as i32,
    };
    assert_eq!(coord.x, 1);
    assert_eq!(coord.z, 2);
    
    // Test negative coordinates
    let neg_pos = Vec3::new(-150.0, 0.0, -250.0);
    let neg_coord = ChunkCoord {
        x: (neg_pos.x / CHUNK_SIZE).floor() as i32,
        z: (neg_pos.z / CHUNK_SIZE).floor() as i32,
    };
    assert_eq!(neg_coord.x, -2);
    assert_eq!(neg_coord.z, -3);
    
    // Test edge cases
    let edge_pos = Vec3::new(100.0, 0.0, 100.0);
    let edge_coord = ChunkCoord {
        x: (edge_pos.x / CHUNK_SIZE).floor() as i32,
        z: (edge_pos.z / CHUNK_SIZE).floor() as i32,
    };
    assert_eq!(edge_coord.x, 1);
    assert_eq!(edge_coord.z, 1);
}

/// Performance test for event throughput
#[test]
fn test_event_performance() {
    use std::time::Instant;
    
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>();
    
    const SPAWN_COUNT: usize = 1000;
    
    let start = Instant::now();
    
    // Send many spawn requests
    for i in 0..SPAWN_COUNT {
        let pos = Vec3::new(i as f32, 0.0, 0.0);
        app.world_mut()
            .send_event(RequestDynamicSpawn::vehicle(pos));
    }
    
    app.update();
    
    let duration = start.elapsed();
    
    // Verify all spawns processed
    let spawned_events = app.world()
        .resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let event_count = reader.read(spawned_events).count();
    
    assert_eq!(event_count, SPAWN_COUNT, "All spawns should be processed");
    
    // Performance assertion (should process 1000 spawns in under 100ms)
    assert!(
        duration.as_millis() < 100,
        "Event processing took {}ms, should be under 100ms",
        duration.as_millis()
    );
}
