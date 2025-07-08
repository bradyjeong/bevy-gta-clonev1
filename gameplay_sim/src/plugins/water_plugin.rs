use bevy::prelude::*;
use crate::systems::water::{setup_lake, setup_yacht, water_wave_system, yacht_buoyancy_system, yacht_movement_system, yacht_water_constraint_system};

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_lake, setup_yacht))
            .add_systems(Update, (
                yacht_movement_system,
                water_wave_system,
                yacht_buoyancy_system,
                yacht_water_constraint_system,
            ));
    }
}
