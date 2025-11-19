use crate::systems::world::traffic::{
    TrafficManager, despawn_traffic_system, move_traffic_system, spawn_traffic_system,
    traffic_light_system,
};
use bevy::prelude::*;

pub struct TrafficPlugin;

impl Plugin for TrafficPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrafficManager>().add_systems(
            Update,
            (
                spawn_traffic_system,
                move_traffic_system,
                despawn_traffic_system,
                traffic_light_system,
            ),
        );
    }
}
