use bevy::prelude::*;
use crate::systems::world::{
    optimized_npc_movement, debug_player_position, dynamic_terrain_system,
    UnifiedWorldManager, 
    unified_world_streaming_system,
    layered_generation_coordinator,
    road_layer_system,
    building_layer_system,
    vehicle_layer_system,
    vegetation_layer_system,
    master_unified_lod_system,
    master_lod_performance_monitor,
    initialize_master_lod_system,
    adaptive_lod_system,
    unified_cleanup_system,
    npc_lod_system,
    migrate_legacy_npcs,
    spawn_new_npc_system,
};
use crate::systems::batching::frame_counter_system;
use crate::systems::effects::update_beacon_visibility;
use crate::systems::timing_service::{TimingService, update_timing_service, cleanup_timing_service};
use crate::factories::{initialize_material_factory};


/// Unified world plugin providing coordinated world generation and management.
/// This plugin handles streaming, LOD, content generation, and NPC management
/// through a layered architecture with unified resource management.
pub struct UnifiedWorldPlugin;

impl Plugin for UnifiedWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<UnifiedWorldManager>()
            .init_resource::<TimingService>()
            .init_resource::<crate::components::FrameCounter>()
            
            // PreUpdate frame counter
            .add_systems(PreUpdate, frame_counter_system)
            
            // Timing and streaming systems
            .add_systems(Update, (
                update_timing_service,
                unified_world_streaming_system,
                layered_generation_coordinator,
                road_layer_system,
            ).chain())
            
            // Content generation layers
            .add_systems(Update, (
                building_layer_system,
                vehicle_layer_system,
                vegetation_layer_system,
            ).chain())
            
            // NPC systems
            .add_systems(Update, migrate_legacy_npcs)
            .add_systems(Update, spawn_new_npc_system)
            
            // LOD and performance systems
            .add_systems(Update, (
                master_unified_lod_system,
                npc_lod_system,
                adaptive_lod_system,
            ).chain())
            
            // Management and cleanup systems
            .add_systems(Update, (
                master_lod_performance_monitor,
                unified_cleanup_system,
                cleanup_timing_service,
                optimized_npc_movement,
                dynamic_terrain_system,
                debug_player_position,
            ).chain())
            
            // Effects system
            .add_systems(Update, update_beacon_visibility)
            
            // Startup systems to initialize the unified world
            .add_systems(Startup, (
                initialize_material_factory,
                initialize_unified_world,
                initialize_master_lod_system,
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
