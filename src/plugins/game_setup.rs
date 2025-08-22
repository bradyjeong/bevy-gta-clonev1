use bevy::prelude::*;

use crate::services::{initialize_simple_services, update_timing_service_system};
use crate::setup::world::setup_dubai_noon_lighting;
use crate::setup::{
    setup_basic_world, setup_initial_aircraft_unified, setup_initial_npcs_unified,
    setup_initial_vehicles_unified, setup_palm_trees,
};
use crate::system_sets::GameSystemSets;
use crate::systems::{
    service_example_config_validation,
    service_example_timing_check, // setup_unified_entity_factory // Function doesn't exist
    service_example_vehicle_creation,
    validate_vehicle_consistency,
};

/// Plugin for organizing all startup and runtime systems with proper ordering
pub struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app
            // Configure system sets
            .configure_sets(
                Startup,
                (
                    GameSystemSets::ServiceInit,
                    GameSystemSets::WorldSetup.after(GameSystemSets::ServiceInit),
                    GameSystemSets::SecondarySetup.after(GameSystemSets::WorldSetup),
                ),
            )
            .configure_sets(Update, GameSystemSets::ServiceUpdates)
            // Service initialization
            .add_systems(
                Startup,
                (initialize_simple_services, validate_vehicle_consistency)
                    .in_set(GameSystemSets::ServiceInit),
            )
            // Core world setup
            .add_systems(
                Startup,
                (
                    // setup_unified_entity_factory, // Function doesn't exist
                    setup_basic_world,
                    setup_dubai_noon_lighting,
                    setup_initial_aircraft_unified,
                )
                    .in_set(GameSystemSets::WorldSetup),
            )
            // Secondary setup
            .add_systems(
                Startup,
                (
                    setup_palm_trees,
                    setup_initial_npcs_unified,
                    setup_initial_vehicles_unified,
                )
                    .in_set(GameSystemSets::SecondarySetup),
            )
            // Runtime service systems
            .add_systems(
                Update,
                (
                    update_timing_service_system,
                    service_example_vehicle_creation,
                    service_example_config_validation,
                    service_example_timing_check,
                )
                    .in_set(GameSystemSets::ServiceUpdates),
            );
    }
}
