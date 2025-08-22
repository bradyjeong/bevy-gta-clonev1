use bevy::prelude::*;
use crate::systems::world::{
    debug_player_position,
    UnifiedWorldManager,
};
use crate::systems::effects::update_beacon_visibility;

/// Plugin responsible for world debugging and monitoring
pub struct WorldDebugPlugin;

impl Plugin for WorldDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, debug_player_position)
            .add_systems(Update, update_beacon_visibility)
            .add_systems(Update, debug_unified_world_activity.run_if(resource_exists::<UnifiedWorldManager>));
    }
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
        
        let loaded_chunks = world_manager.chunks.iter()
            .filter_map(|chunk_opt| chunk_opt.as_ref())
            .filter(|chunk| matches!(chunk.state, crate::systems::world::unified_world::ChunkState::Loaded { .. }))
            .count();
        
        let loading_chunks = world_manager.chunks.iter()
            .filter_map(|chunk_opt| chunk_opt.as_ref())
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
