use bevy::prelude::*;
use crate::systems::world::{optimized_npc_movement, distance_culling_system, performance_monitoring_system, debug_player_position, dynamic_terrain_system, dynamic_content_system, vehicle_separation_system, road_network_system, update_road_dependent_systems, map_streaming_system, map_lod_system};
use crate::systems::effects::update_beacon_visibility;
use crate::systems::world::{RoadNetwork, MapSystem};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<RoadNetwork>()
            .init_resource::<MapSystem>()
            .add_systems(Startup, reset_road_network_once)
            .add_systems(Update, (
                // NEW: Road network system (replaces old grid roads)
                road_network_system,
                update_road_dependent_systems,
                // Map streaming and LOD systems
                map_streaming_system,
                map_lod_system,
                // OLD: Dynamic content system (now without road generation)
                dynamic_content_system,
                vehicle_separation_system,
                // Other systems
                optimized_npc_movement,
                distance_culling_system,
                dynamic_terrain_system,
                performance_monitoring_system,
                debug_player_position,
                update_beacon_visibility,
            ));
    }
}

fn reset_road_network_once(mut road_network: ResMut<RoadNetwork>) {
    road_network.reset();
    println!("DEBUG: Road network reset on startup!");
}
