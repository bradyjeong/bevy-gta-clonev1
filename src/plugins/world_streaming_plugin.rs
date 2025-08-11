use bevy::prelude::*;

// V1 imports (default)
#[cfg(not(feature = "world_v2"))]
use crate::systems::world::{
    UnifiedWorldManager,
    unified_world_streaming_system,
    layered_generation_coordinator,
    query_dynamic_content,
    // Event handler systems
    handle_spawn_validation_request,
    handle_road_validation_result,
    handle_road_validation_request,
    handle_dynamic_spawn_request,
    handle_chunk_load_request,
    handle_chunk_unload_request,
    handle_validation_to_spawn_bridge,
};

// V2 imports (with feature flag)
#[cfg(feature = "world_v2")]
use crate::systems::world::{
    layered_generation_coordinator,
    query_dynamic_content,
    handle_dynamic_spawn_request,
    handle_chunk_load_request,
    handle_chunk_unload_request,
    handle_validation_to_spawn_bridge,
};

#[cfg(feature = "world_v2")]
use crate::systems::world::streaming_v2::{
    unified_world_streaming_system_v2,
    update_chunk_states_v2,
    validate_placement_system_v2,
    find_valid_spawn_position_v2,
    pathfinding_system_v2,
    road_validation_system_v2,
};

#[cfg(feature = "world_v2")]
use crate::systems::world::event_handlers::spawn_validation_handler_v2::{
    handle_spawn_validation_request_v2,
    handle_road_validation_result_v2,
    handle_road_validation_request_v2,
};

#[cfg(feature = "world_v2")]
use crate::world::{ChunkTracker, PlacementGrid, RoadNetwork, WorldCoordinator};

use crate::system_sets::WorldEventFlow;


/// Plugin responsible for world streaming and chunk management
pub struct WorldStreamingPlugin;

impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources based on feature flag
        #[cfg(not(feature = "world_v2"))]
        {
            app.init_resource::<UnifiedWorldManager>();
        }
        
        #[cfg(feature = "world_v2")]
        {
            app.init_resource::<ChunkTracker>()
                .init_resource::<PlacementGrid>()
                .init_resource::<RoadNetwork>()
                .init_resource::<WorldCoordinator>();
        }
        
        app.add_systems(Startup, initialize_streaming_world)
            .configure_sets(Update, (
                WorldEventFlow::SpawnQuery,
                WorldEventFlow::SpawnValidationTx.after(WorldEventFlow::SpawnQuery),
                WorldEventFlow::RoadValidation.after(WorldEventFlow::SpawnValidationTx),
                WorldEventFlow::SpawnValidationRx.after(WorldEventFlow::RoadValidation),
                WorldEventFlow::SpawnEmit.after(WorldEventFlow::SpawnValidationRx),
                WorldEventFlow::SpawnExecute.after(WorldEventFlow::SpawnEmit),
            ));
            
        // Register systems based on feature flag
        #[cfg(not(feature = "world_v2"))]
        {
            app.add_systems(Update, (
                // Phase 1: Core world streaming
                unified_world_streaming_system,
                layered_generation_coordinator,
                
                // World Event Flow - Dynamic Content Pipeline
                query_dynamic_content.in_set(WorldEventFlow::SpawnQuery),
                handle_spawn_validation_request.in_set(WorldEventFlow::SpawnValidationTx),
                handle_road_validation_request.in_set(WorldEventFlow::RoadValidation),
                handle_road_validation_result.in_set(WorldEventFlow::SpawnValidationRx),
                handle_validation_to_spawn_bridge.in_set(WorldEventFlow::SpawnEmit),
                handle_dynamic_spawn_request.in_set(WorldEventFlow::SpawnExecute),
                
                // Phase 2: Chunk management
                handle_chunk_load_request,
                handle_chunk_unload_request,
            ));
        }
        
        #[cfg(feature = "world_v2")]
        {
            app.add_systems(Update, (
                // Phase 1: Core world streaming V2
                unified_world_streaming_system_v2,
                update_chunk_states_v2,
                layered_generation_coordinator,
                
                // World Event Flow - Dynamic Content Pipeline V2
                query_dynamic_content.in_set(WorldEventFlow::SpawnQuery),
                handle_spawn_validation_request_v2.in_set(WorldEventFlow::SpawnValidationTx),
                handle_road_validation_request_v2.in_set(WorldEventFlow::RoadValidation),
                handle_road_validation_result_v2.in_set(WorldEventFlow::SpawnValidationRx),
                handle_validation_to_spawn_bridge.in_set(WorldEventFlow::SpawnEmit),
                handle_dynamic_spawn_request.in_set(WorldEventFlow::SpawnExecute),
                
                // Additional V2 systems
                validate_placement_system_v2,
                find_valid_spawn_position_v2,
                pathfinding_system_v2,
                road_validation_system_v2,
                
                // Phase 2: Chunk management
                handle_chunk_load_request,
                handle_chunk_unload_request,
            ));
        }
    }
}

#[cfg(not(feature = "world_v2"))]
fn initialize_streaming_world(mut world_manager: ResMut<UnifiedWorldManager>) {
    world_manager.chunks.clear();
    world_manager.placement_grid.clear();
    world_manager.road_network.reset();
    println!("DEBUG: World streaming initialized!");
}

#[cfg(feature = "world_v2")]
fn initialize_streaming_world(
    mut tracker: ResMut<ChunkTracker>,
    mut placement_grid: ResMut<PlacementGrid>,
    mut road_network: ResMut<RoadNetwork>,
) {
    tracker.loaded.clear();
    tracker.loading.clear();
    tracker.unloading.clear();
    tracker.distances.clear();
    placement_grid.clear();
    road_network.reset();
    println!("DEBUG: World streaming V2 initialized!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_streaming_plugin_instantiation() {
        // Test that plugin can be instantiated
        let plugin = WorldStreamingPlugin;
        let mut app = App::new();
        
        // Test that adding plugin doesn't panic during registration
        app.add_plugins(MinimalPlugins);
        
        // Just test the build method doesn't panic
        plugin.build(&mut app);
        
        // Verify UnifiedWorldManager resource is initialized
        assert!(app.world().get_resource::<crate::systems::world::unified_world::UnifiedWorldManager>().is_some());
    }
}
