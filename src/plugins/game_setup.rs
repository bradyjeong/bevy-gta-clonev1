use bevy::prelude::*;

use crate::config::AssetLoadingPolicy;
use crate::setup::world::setup_dubai_noon_lighting;
use crate::setup::{
    setup_basic_world, setup_initial_aircraft_unified, setup_initial_npcs_unified,
    setup_initial_vehicles_unified,
};
use crate::states::AppState;
use crate::system_sets::GameSystemSets;
use crate::systems::loading::{
    advance_to_ingame, check_vehicle_specs_loaded, start_loading_vehicle_specs,
};

/// Plugin for organizing all startup and runtime systems with proper ordering
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetLoadingPolicy::for_build())
            .add_systems(OnEnter(AppState::AssetLoading), start_loading_vehicle_specs)
            .add_systems(
                Update,
                check_vehicle_specs_loaded.run_if(in_state(AppState::AssetLoading)),
            )
            .add_systems(
                Update,
                advance_to_ingame.run_if(in_state(AppState::WorldGeneration)),
            )
            .configure_sets(
                Startup,
                (
                    GameSystemSets::WorldSetup,
                    GameSystemSets::SecondarySetup.after(GameSystemSets::WorldSetup),
                ),
            )
            .add_systems(
                OnEnter(AppState::WorldGeneration),
                (setup_basic_world, setup_dubai_noon_lighting).in_set(GameSystemSets::WorldSetup),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    setup_initial_aircraft_unified,
                    setup_initial_npcs_unified,
                    setup_initial_vehicles_unified,
                )
                    .in_set(GameSystemSets::SecondarySetup),
            );
    }
}
