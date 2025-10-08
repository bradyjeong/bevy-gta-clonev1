use bevy::prelude::*;

use crate::setup::world::setup_dubai_noon_lighting;
use crate::setup::{
    setup_basic_world, setup_initial_aircraft_unified, setup_initial_npcs_unified,
    setup_initial_vehicles_unified,
};
use crate::system_sets::GameSystemSets;

/// Plugin for organizing all startup and runtime systems with proper ordering
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app
            // Configure system sets
            .configure_sets(
                Startup,
                (
                    GameSystemSets::WorldSetup,
                    GameSystemSets::SecondarySetup.after(GameSystemSets::WorldSetup),
                ),
            )
            // Core world setup
            .add_systems(
                Startup,
                (
                    setup_basic_world,
                    setup_dubai_noon_lighting,
                    setup_initial_aircraft_unified,
                )
                    .in_set(GameSystemSets::WorldSetup),
            )
            // Secondary setup
            .add_systems(
                Startup,
                (setup_initial_npcs_unified, setup_initial_vehicles_unified)
                    .in_set(GameSystemSets::SecondarySetup),
            );
    }
}
