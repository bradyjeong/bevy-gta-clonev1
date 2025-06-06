use bevy::prelude::*;
use crate::systems::world::{
    // Legacy systems (to be phased out)
    optimized_npc_movement, debug_player_position, dynamic_terrain_system,
    
    // Unified world systems
    UnifiedWorldManager, 
    unified_world_streaming_system,
    layered_generation_coordinator,
    road_layer_system,
    building_layer_system,
    vehicle_layer_system,
    vegetation_layer_system,
    unified_lod_system,
    unified_lod_performance_monitor,
    adaptive_lod_system,
    unified_distance_culling_system,
    unified_cleanup_system,
    
    // NPC systems
    npc_lod_system,
    migrate_legacy_npcs,
    spawn_new_npc_system,
    setup_new_npcs,
};
use crate::systems::effects::update_beacon_visibility;
use crate::systems::timing_service::{TimingService, update_timing_service, cleanup_timing_service};

/// New unified world plugin that replaces the old WorldPlugin
/// This provides a single, coordinated world generation system
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<UnifiedWorldManager>()
            .init_resource::<TimingService>()
            
            // Core unified systems - run in order
            .add_systems(Update, (
                // 0. Update timing service first (must run before other systems)
                update_timing_service,
                
                // 1. Main streaming system (loads/unloads chunks)
                unified_world_streaming_system,
                
                // 2. Layered content generation (runs after streaming)
                layered_generation_coordinator,
                road_layer_system,
                building_layer_system,
                vehicle_layer_system,
                vegetation_layer_system,
                
                // 3. NPC systems (migration and spawning)
                migrate_legacy_npcs,
                spawn_new_npc_system,
                
                // 4. LOD and culling systems (runs after generation)
                unified_lod_system,
                npc_lod_system,
                unified_distance_culling_system,
                adaptive_lod_system,
                
                // 5. Performance monitoring and cleanup
                unified_lod_performance_monitor,
                unified_cleanup_system,
                cleanup_timing_service,
                
                // 6. Legacy systems (keeping for compatibility)
                optimized_npc_movement,
                dynamic_terrain_system,
                debug_player_position,
                update_beacon_visibility,
            ).chain()) // Chain ensures proper execution order
            
            // Startup systems to initialize the unified world
            .add_systems(Startup, (
                initialize_unified_world,
                setup_new_npcs,
            ))
            
            // Debug system to monitor unified world activity
            .add_systems(Update, debug_unified_world_activity.run_if(resource_exists::<UnifiedWorldManager>));
    }
}

fn initialize_unified_world(mut world_manager: ResMut<UnifiedWorldManager>) {
    // Clear any existing data
    world_manager.chunks.clear();
    world_manager.placement_grid.clear();
    world_manager.road_network.reset();
    
    println!("DEBUG: Unified world system initialized!");
}

/// Migration helper - use this plugin to test unified system alongside old system
pub struct MixedWorldPlugin;

impl Plugin for MixedWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Both old and new resources
            .init_resource::<UnifiedWorldManager>()
            .init_resource::<crate::systems::world::RoadNetwork>()
            .init_resource::<crate::systems::world::MapSystem>()
            
            .add_systems(Startup, (
                initialize_unified_world,
                reset_road_network_once,
            ))
            
            .add_systems(Update, (
                // NEW UNIFIED SYSTEMS (primary)
                unified_world_streaming_system,
                layered_generation_coordinator,
                road_layer_system,
                building_layer_system,
                vehicle_layer_system,
                vegetation_layer_system,
                unified_lod_system,
                npc_lod_system,
                unified_distance_culling_system,
                
                // OLD SYSTEMS (for comparison - comment out when ready)
                // road_network_system,
                // map_streaming_system,
                // map_lod_system,
                // dynamic_content_system,
                // vehicle_separation_system,
                // distance_culling_system,
                
                // SHARED SYSTEMS
                optimized_npc_movement,
                dynamic_terrain_system,
                debug_player_position,
                update_beacon_visibility,
                unified_lod_performance_monitor,
                adaptive_lod_system,
            ));
    }
}

fn reset_road_network_once(mut road_network: ResMut<crate::systems::world::RoadNetwork>) {
    road_network.reset();
    println!("DEBUG: Legacy road network reset on startup!");
}

fn debug_unified_world_activity(
    world_manager: Res<UnifiedWorldManager>,
    time: Res<Time>,
    mut last_report_time: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Report every 5 seconds
    if current_time - *last_report_time > 5.0 {
        *last_report_time = current_time;
        
        let loaded_chunks = world_manager.chunks.values()
            .filter(|chunk| matches!(chunk.state, crate::systems::world::unified_world::ChunkState::Loaded { .. }))
            .count();
        
        let loading_chunks = world_manager.chunks.values()
            .filter(|chunk| matches!(chunk.state, crate::systems::world::unified_world::ChunkState::Loading))
            .count();
        
        println!("🌍 UNIFIED WORLD STATUS:");
        println!("  📦 Total chunks: {}", world_manager.chunks.len());
        println!("  ✅ Loaded chunks: {}", loaded_chunks);
        println!("  ⏳ Loading chunks: {}", loading_chunks);
        println!("  🛣️ Roads generated: {}", world_manager.road_network.roads.len());
        println!("  🎯 Active chunk: {:?}", world_manager.active_chunk);
        println!("  ⚡ Max chunks/frame: {}", world_manager.max_chunks_per_frame);
    }
}
