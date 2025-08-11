use bevy::prelude::*;
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
use crate::system_sets::WorldEventFlow;


/// Plugin responsible for world streaming and chunk management
pub struct WorldStreamingPlugin;

impl Plugin for WorldStreamingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnifiedWorldManager>()
            .add_systems(Startup, initialize_streaming_world)
            .configure_sets(Update, (
                WorldEventFlow::SpawnQuery,
                WorldEventFlow::SpawnValidationTx.after(WorldEventFlow::SpawnQuery),
                WorldEventFlow::RoadValidation.after(WorldEventFlow::SpawnValidationTx),
                WorldEventFlow::SpawnValidationRx.after(WorldEventFlow::RoadValidation),
                WorldEventFlow::SpawnEmit.after(WorldEventFlow::SpawnValidationRx),
                WorldEventFlow::SpawnExecute.after(WorldEventFlow::SpawnEmit),
            ))
            .add_systems(Update, (
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
}

fn initialize_streaming_world(mut world_manager: ResMut<UnifiedWorldManager>) {
    world_manager.chunks.clear();
    world_manager.placement_grid.clear();
    world_manager.road_network.reset();
    println!("DEBUG: World streaming initialized!");
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
        assert!(app.world().get_resource::<UnifiedWorldManager>().is_some());
    }
}
