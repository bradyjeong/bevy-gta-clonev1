pub mod environment;
pub mod unified_aircraft;
pub mod unified_npcs;
pub mod unified_vehicles;
pub mod vehicles;
pub mod world;

// Setup functions and utilities
pub use environment::setup_palm_trees;
pub use unified_aircraft::{AircraftType, setup_initial_aircraft_unified};
pub use unified_npcs::setup_initial_npcs_unified;
pub use unified_vehicles::setup_initial_vehicles_unified;
pub use vehicles::{BugattiColorScheme, setup_luxury_bugatti_chiron};
pub use world::{setup_basic_world, setup_dubai_noon_lighting};
