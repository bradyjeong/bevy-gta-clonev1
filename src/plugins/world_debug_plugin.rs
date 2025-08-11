use bevy::prelude::*;
use crate::systems::world::debug_player_position;
use crate::world::{ChunkTables, ChunkTracker, RoadNetwork};
use crate::systems::effects::update_beacon_visibility;

/// Plugin responsible for world debugging and monitoring
pub struct WorldDebugPlugin;

impl Plugin for WorldDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, debug_player_position)
            .add_systems(Update, update_beacon_visibility)
            .add_systems(Update, debug_unified_world_activity.run_if(resource_exists::<ChunkTracker>));
    }
}

fn debug_unified_world_activity(
    tracker: Res<ChunkTracker>,
    tables: Option<Res<ChunkTables>>,
    roads: Option<Res<RoadNetwork>>,
    time: Res<Time>,
    mut last_report_time: Local<f32>,
) {
    let current_time = time.elapsed_secs();
    
    // Report every 5 seconds
    if current_time - *last_report_time > 5.0 {
        *last_report_time = current_time;
        
        // Use tracker/tables/roads to print a compact status
        let total_loaded = tables
            .as_ref()
            .map(|t| t.loaded.len())
            .unwrap_or(0);
        let roads_count = roads.as_ref().map(|r| r.roads.len()).unwrap_or(0);
        
        println!("🌍 WORLD STATUS:");
        println!("  ✅ Loaded chunks (fast): {}", tracker.get_loaded_chunks().len());
        println!("  📦 Loaded chunks (full): {}", total_loaded);
        println!("  🛣️ Roads generated: {}", roads_count);
        println!("  🎯 Focus chunk: ({}, {})", tracker.focus_chunk.x, tracker.focus_chunk.z);
        println!("  🔄 LOD radius: {} | Active: {}", tracker.lod_radius, tracker.active_count);
    }
}
