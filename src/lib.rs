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

pub use plugins::*;
pub use factories::*;
pub use setup::{setup_basic_world, setup_simple_vehicles, setup_simple_helicopter, setup_simple_f16, setup_palm_trees, setup_luxury_cars, setup_npcs, setup_buildings, setup_starter_vehicles, setup_basic_roads};

pub use constants::*;
pub use config::*;
pub use game_state::*;
