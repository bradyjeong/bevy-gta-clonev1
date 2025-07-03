pub mod components;
pub mod config;
pub mod systems;
pub mod plugins;
pub mod setup;
pub mod constants;
pub mod game_state;
pub mod bundles;
pub mod factories;
pub mod services;

// Re-export specific items to avoid ambiguity
pub use components::{Player, ActiveEntity, Car, SuperCar, Helicopter, F16, NPC, Cullable, MainCamera, MainRotor, TailRotor};
pub use components::{DynamicTerrain, DynamicContent, ContentType, CullingSettings, PerformanceStats, Building, RoadEntity, IntersectionEntity};
pub use components::{ExhaustFlame, VehicleBeacon, ControlsText, ControlsDisplay};
pub use components::{Lake, Yacht, WaterBody, WaterWave, Boat};

pub use systems::movement::*;
pub use systems::camera::*;
pub use systems::interaction::*;
pub use systems::world::*;
pub use systems::effects::*;
pub use systems::performance_monitor::*;
pub use systems::performance_integration::*;

pub use plugins::*;
pub use factories::*;
pub use setup::{setup_basic_world, setup_initial_aircraft_unified, setup_palm_trees, setup_initial_npcs_unified, setup_initial_vehicles_unified};

pub use constants::*;
pub use config::*;
pub use game_state::*;
