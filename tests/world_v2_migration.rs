#[cfg(feature = "world_v2")]
use bevy::prelude::*;
#[cfg(feature = "world_v2")]
use gta_game::world::*;
#[cfg(feature = "world_v2")]
use gta_game::systems::world::unified_world::UnifiedWorldManager;

#[test]
#[cfg(feature = "world_v2")]
fn test_migration_from_unified_world() {
    // Test migration from UnifiedWorldManager to decomposed resources
    let unified = UnifiedWorldManager {
        world_origin: Vec3::new(100.0, 0.0, 200.0),
        streaming_radius: 500.0,
        ..Default::default()
    };
    
    // Test ChunkTracker migration
    let chunk_tracker = ChunkTracker::from(&unified);
    assert_eq!(chunk_tracker.focus_chunk.x, 0); // 100.0 / 128.0 = 0
    assert_eq!(chunk_tracker.focus_chunk.z, 1); // 200.0 / 128.0 = 1
    assert_eq!(chunk_tracker.lod_radius, 8);
    assert!(chunk_tracker.focus_valid);
    
    // Test PlacementGrid migration
    let placement_grid = PlacementGrid::from(&unified);
    assert_eq!(placement_grid.grid_size, 16);
    assert_eq!(placement_grid.validation_frame, 0);
    assert_eq!(placement_grid.placement_mode, PlacementMode::Normal);
    
    // Test RoadNetwork migration
    let road_network = RoadNetwork::from(&unified);
    assert_eq!(road_network.active_nodes, 0);
    assert_eq!(road_network.generation_seed, 12345);
    
    // Test WorldCoordinator migration
    let world_coordinator = WorldCoordinator::from(&unified);
    assert_eq!(world_coordinator.focus_position.x, 100);
    assert_eq!(world_coordinator.focus_position.z, 200);
    assert_eq!(world_coordinator.streaming_radius, 500.0);
}

#[test]
#[cfg(feature = "world_v2")]
fn test_world_v2_plugin_registration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WorldV2Plugin);
    
    // Verify resources are registered
    assert!(app.world().contains_resource::<ChunkTracker>());
    assert!(app.world().contains_resource::<PlacementGrid>());
    assert!(app.world().contains_resource::<RoadNetwork>());
    assert!(app.world().contains_resource::<WorldCoordinator>());
}

#[test]
#[cfg(not(feature = "world_v2"))]
fn test_backward_compatibility() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(gta_game::world::WorldV2Plugin);
    
    // Verify facade resources are registered when world_v2 is disabled
    assert!(app.world().contains_resource::<gta_game::world::ChunkTrackerFacade>());
    assert!(app.world().contains_resource::<gta_game::world::PlacementGridFacade>());
    assert!(app.world().contains_resource::<gta_game::world::RoadNetworkFacade>());
    assert!(app.world().contains_resource::<gta_game::world::WorldCoordinatorFacade>());
}

#[test]
fn test_resource_sizes_are_truthful() {
    use std::mem::size_of;
    
    // Verify actual sizes are within our assertions
    #[cfg(feature = "world_v2")]
    {
        let chunk_tracker_size = size_of::<gta_game::world::ChunkTracker>();
        let placement_grid_size = size_of::<gta_game::world::PlacementGrid>();
        let road_network_size = size_of::<gta_game::world::RoadNetwork>();
        let world_coordinator_size = size_of::<gta_game::world::WorldCoordinator>();
        
        println!("ChunkTracker actual size: {} bytes", chunk_tracker_size);
        println!("PlacementGrid actual size: {} bytes", placement_grid_size);
        println!("RoadNetwork actual size: {} bytes", road_network_size);
        println!("WorldCoordinator actual size: {} bytes", world_coordinator_size);
        
        // Oracle's targets with truthful assertions
        assert!(chunk_tracker_size <= 64, "ChunkTracker exceeds 64 bytes");
        assert!(placement_grid_size <= 24, "PlacementGrid exceeds 24 bytes");
        assert!(road_network_size <= 32, "RoadNetwork exceeds 32 bytes");
        assert!(world_coordinator_size <= 32, "WorldCoordinator exceeds 32 bytes");
    }
}
