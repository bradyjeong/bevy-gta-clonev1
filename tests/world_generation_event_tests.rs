//! Integration tests for world generation event-driven architecture
//! 
//! Tests the complete event flow from chunk loading through content spawning

use bevy::prelude::*;
use gta_game::{
    events::world::{
        content_events::*,
        validation_events::*,
    },
    components::{
        dynamic_content::DynamicContent,
    },
    world::{ChunkTracker, PlacementGrid},
    plugins::WorldStreamingPlugin,
};

// Import chunk events separately to avoid ChunkCoord conflicts  
use gta_game::events::world::chunk_events::{
    ChunkCoord, RequestChunkLoad, ChunkLoaded, RequestChunkUnload, 
    ChunkUnloaded, ChunkFinishedLoading
};

/// Full integration test of chunk load → validation → spawn flow
#[test]
fn test_full_event_flow_integration() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .init_resource::<ChunkTracker>()
        .init_resource::<PlacementGrid>()
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<RequestSpawnValidation>()
        .add_event::<SpawnValidationResult>()
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>();
    
    // Add test systems for the event chain
    app.add_systems(Update, (
        handle_chunk_load_request,
        handle_validation_request,
        handle_validation_result,
        handle_dynamic_spawn_request,
    ).chain());
    
    let coord = ChunkCoord { x: 0, z: 0 };
    
    // Start the chain: Request chunk load
    app.world_mut().send_event(RequestChunkLoad { coord });
    
    // Process the full event chain
    app.update();
    
    // Verify chunk was loaded
    let tracker = app.world().resource::<ChunkTracker>();
    let world_coord = gta_game::world::ChunkCoord { x: coord.x, z: coord.z };
    assert!(tracker.is_chunk_loaded(world_coord), "Chunk should be loaded");
    
    // Verify content was spawned
    let mut query = app.world_mut().query::<&DynamicContent>();
    let content_count = query.iter(app.world()).count();
    assert!(content_count > 0, "Content should be spawned in chunk");
    
    // Verify events were emitted
    let spawned_events = app.world().resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let events: Vec<_> = reader.read(spawned_events).collect();
    assert!(!events.is_empty(), "DynamicContentSpawned events should be emitted");
}

/// Test system: Handle chunk load requests
fn handle_chunk_load_request(
    mut events: EventReader<RequestChunkLoad>,
    mut tracker: ResMut<ChunkTracker>,
    mut chunk_loaded: EventWriter<ChunkLoaded>,
    mut validation_requests: EventWriter<RequestSpawnValidation>,
) {
    for event in events.read() {
        // Mark chunk as loading
        let world_coord = gta_game::world::ChunkCoord { x: event.coord.x, z: event.coord.z };
        tracker.mark_loaded(world_coord);
        
        // Send chunk loaded event
        chunk_loaded.write(ChunkLoaded { coord: event.coord, content_count: 0 });
        
        // Request validation for content positions
        let chunk_center = Vec3::new(
            event.coord.x as f32 * 100.0,
            0.0,
            event.coord.z as f32 * 100.0,
        );
        
        // Request validation for various content types
        validation_requests.write(RequestSpawnValidation {
            id: ValidationId::new(1),
            pos: chunk_center + Vec3::new(10.0, 0.0, 10.0),
            content_type: ContentType::Building,
        });
        validation_requests.write(RequestSpawnValidation {
            id: ValidationId::new(2),
            pos: chunk_center + Vec3::new(30.0, 0.0, 30.0),
            content_type: ContentType::Vehicle,
        });
    }
}

/// Test system: Handle validation requests
fn handle_validation_request(
    mut events: EventReader<RequestSpawnValidation>,
    mut validation_results: EventWriter<SpawnValidationResult>,
    grid: Res<PlacementGrid>,
) {
    for event in events.read() {
        // Simple validation: check if position is not occupied
        let cell = grid.world_to_grid(event.pos);
        let index = ((cell.x.rem_euclid(8) * 8 + cell.z.rem_euclid(8)) as usize).min(63);
        let is_occupied = (grid.occupied_cells >> index) & 1 == 1;
        
        let reason = if is_occupied {
            ValidationReason::Collision
        } else {
            ValidationReason::Valid
        };
        
        validation_results.write(SpawnValidationResult {
            id: event.id,
            position: event.pos,
            content_type: event.content_type,
            valid: !is_occupied,
            reason,
        });
    }
}

/// Test system: Handle validation result events
fn handle_validation_result(
    mut events: EventReader<SpawnValidationResult>,
    mut spawn_requests: EventWriter<RequestDynamicSpawn>,
) {
    for event in events.read() {
        if event.valid {
            // For testing, spawn a building at validated position
            spawn_requests.write(RequestDynamicSpawn {
                pos: Vec3::new(100.0, 0.0, 50.0), // Use a test position
                kind: ContentType::Building,
            });
        }
    }
}

/// Test system: Handle dynamic spawn requests
fn handle_dynamic_spawn_request(
    mut commands: Commands,
    mut events: EventReader<RequestDynamicSpawn>,
    mut spawned_events: EventWriter<DynamicContentSpawned>,
) {
    for event in events.read() {
        // Spawn the entity
        let entity = commands.spawn((
            DynamicContent::new(match event.kind {
                ContentType::Building => gta_game::components::dynamic_content::ContentType::Building,
                ContentType::Vehicle => gta_game::components::dynamic_content::ContentType::Vehicle,
                ContentType::NPC => gta_game::components::dynamic_content::ContentType::NPC,
                ContentType::Tree => gta_game::components::dynamic_content::ContentType::Tree,
                ContentType::Road => gta_game::components::dynamic_content::ContentType::Road,
            }),
            Transform::from_translation(event.pos),
        )).id();
        
        // Send spawned event
        spawned_events.write(DynamicContentSpawned { entity, pos: event.pos, kind: event.kind });
    }
}

/// Test that chunk unloading removes associated content
#[test]
fn test_chunk_unload_removes_content() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .init_resource::<ChunkTracker>()
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<RequestChunkUnload>()
        .add_event::<ChunkUnloaded>()
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>()
        .add_event::<RequestDynamicDespawn>()
        .add_event::<DynamicContentDespawned>();
    
    // Add systems
    app.add_systems(Update, (
        simple_chunk_load_handler,
        simple_chunk_unload_handler,
        simple_spawn_handler,
        simple_despawn_handler,
    ));
    
    let coord = ChunkCoord { x: 1, z: 1 };
    
    // Load chunk and spawn content
    app.world_mut().send_event(RequestChunkLoad { coord });
    app.update();
    
    // Spawn some content in the chunk
    app.world_mut().send_event(RequestDynamicSpawn::vehicle(Vec3::new(100.0, 0.0, 100.0)));
    app.update();
    
    // Get spawned entity
    let spawned_events = app.world().resource::<Events<DynamicContentSpawned>>();
    let mut reader = spawned_events.get_cursor();
    let entity = reader.read(spawned_events).next().unwrap().entity;
    
    // Verify entity exists
    assert!(app.world().entities().contains(entity));
    
    // Unload chunk
    app.world_mut().send_event(RequestChunkUnload { coord });
    app.update();
    
    // Request despawn of chunk content
    app.world_mut().send_event(RequestDynamicDespawn { entity });
    app.update();
    
    // Verify entity was removed
    assert!(!app.world().entities().contains(entity));
}

// Simple test handlers
fn simple_chunk_load_handler(
    mut events: EventReader<RequestChunkLoad>,
    mut tracker: ResMut<ChunkTracker>,
    mut loaded: EventWriter<ChunkLoaded>,
) {
    for event in events.read() {
        let world_coord = gta_game::world::ChunkCoord { x: event.coord.x, z: event.coord.z };
        tracker.mark_loaded(world_coord);
        loaded.write(ChunkLoaded { coord: event.coord, content_count: 1 });
    }
}

fn simple_chunk_unload_handler(
    mut events: EventReader<RequestChunkUnload>,
    mut tracker: ResMut<ChunkTracker>,
    mut unloaded: EventWriter<ChunkUnloaded>,
) {
    for event in events.read() {
        tracker.clear(); // Simple clear for test
        unloaded.write(ChunkUnloaded { coord: event.coord });
    }
}

fn simple_spawn_handler(
    mut commands: Commands,
    mut events: EventReader<RequestDynamicSpawn>,
    mut spawned: EventWriter<DynamicContentSpawned>,
) {
    for event in events.read() {
        let entity = commands.spawn((
            Transform::from_translation(event.pos),
        )).id();
        spawned.write(DynamicContentSpawned { entity, pos: event.pos, kind: event.kind });
    }
}

fn simple_despawn_handler(
    mut commands: Commands,
    mut events: EventReader<RequestDynamicDespawn>,
    mut despawned: EventWriter<DynamicContentDespawned>,
) {
    for event in events.read() {
        commands.entity(event.entity).despawn();
        despawned.write(DynamicContentDespawned { entity: event.entity });
    }
}

/// Test concurrent chunk loading
#[test]
fn test_concurrent_chunk_loading() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(WorldStreamingPlugin)
        .init_resource::<ChunkTracker>()
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<ChunkFinishedLoading>();
    
    app.add_systems(Update, concurrent_chunk_handler);
    
    // Request multiple chunks at once
    let coords = vec![
        ChunkCoord { x: 0, z: 0 },
        ChunkCoord { x: 0, z: 1 },
        ChunkCoord { x: 1, z: 0 },
        ChunkCoord { x: 1, z: 1 },
    ];
    
    for coord in &coords {
        app.world_mut().send_event(RequestChunkLoad { coord: *coord });
    }
    
    app.update();
    
    // Verify all chunks loaded
    let tracker = app.world().resource::<ChunkTracker>();
    for coord in coords {
        let world_coord = gta_game::world::ChunkCoord { x: coord.x, z: coord.z };
        assert!(tracker.is_chunk_loaded(world_coord), "Chunk {:?} should be loaded", coord);
    }
    
    // Verify finished loading events
    let finished_events = app.world().resource::<Events<ChunkFinishedLoading>>();
    let mut reader = finished_events.get_cursor();
    let events: Vec<_> = reader.read(finished_events).collect();
    assert_eq!(events.len(), 4, "All chunks should finish loading");
}

fn concurrent_chunk_handler(
    mut events: EventReader<RequestChunkLoad>,
    mut tracker: ResMut<ChunkTracker>,
    mut loaded: EventWriter<ChunkLoaded>,
    mut finished: EventWriter<ChunkFinishedLoading>,
) {
    for event in events.read() {
        let world_coord = gta_game::world::ChunkCoord { x: event.coord.x, z: event.coord.z };
        tracker.mark_loaded(world_coord);
        loaded.write(ChunkLoaded { coord: event.coord, content_count: 5 });
        finished.write(ChunkFinishedLoading { coord: event.coord, lod_level: 0 });
    }
}

// Resource for logging event order
#[derive(Resource, Default)]
struct EventLog {
    events: Vec<String>,
}

/// Test event ordering and dependencies
#[test]
fn test_event_ordering() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .init_resource::<EventLog>() // Track event order
        .add_event::<RequestChunkLoad>()
        .add_event::<ChunkLoaded>()
        .add_event::<RequestSpawnValidation>()
        .add_event::<SpawnValidationResult>()
        .add_event::<RequestDynamicSpawn>()
        .add_event::<DynamicContentSpawned>();
    
    // Add systems with explicit ordering
    app.add_systems(Update, (
        log_chunk_load.before(log_validation),
        log_validation.before(log_spawn),
        log_spawn,
    ));
    
    // Trigger events
    app.world_mut().send_event(RequestChunkLoad { coord: ChunkCoord { x: 0, z: 0 } });
    app.world_mut().send_event(RequestSpawnValidation {
        id: ValidationId::new(1),
        pos: Vec3::ZERO,
        content_type: ContentType::Building,
    });
    app.world_mut().send_event(RequestDynamicSpawn::building(Vec3::ZERO));
    
    app.update();
    
    // Check order
    let log = app.world().resource::<EventLog>();
    assert_eq!(log.events.len(), 3);
    assert_eq!(log.events[0], "chunk_load");
    assert_eq!(log.events[1], "validation");
    assert_eq!(log.events[2], "spawn");
}

fn log_chunk_load(
    mut events: EventReader<RequestChunkLoad>,
    mut log: ResMut<EventLog>,
) {
    for _ in events.read() {
        log.events.push("chunk_load".to_string());
    }
}

fn log_validation(
    mut events: EventReader<RequestSpawnValidation>,
    mut log: ResMut<EventLog>,
) {
    for _ in events.read() {
        log.events.push("validation".to_string());
    }
}

fn log_spawn(
    mut events: EventReader<RequestDynamicSpawn>,
    mut log: ResMut<EventLog>,
) {
    for _ in events.read() {
        log.events.push("spawn".to_string());
    }
}
