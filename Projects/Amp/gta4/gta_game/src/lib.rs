pub mod components;
pub mod config;
pub mod systems;
pub mod plugins;
pub mod setup;
pub mod constants;
pub mod game_state;
pub mod bundles;
pub mod factories;

// Re-export specific items to avoid ambiguity
pub use components::{Player, ActiveEntity, Car, SuperCar, Helicopter, F16, NPC, Cullable, MainCamera, MainRotor, TailRotor};
pub use components::{DynamicTerrain, DynamicContent, ContentType, CullingSettings, PerformanceStats, Building, RoadEntity, IntersectionEntity};
pub use components::{ExhaustFlame, VehicleBeacon, ControlsText, ControlsDisplay};
pub use components::{Lake, Yacht, WaterBody, WaterWave, Boat};
#[cfg(feature = "weather")]
pub use components::{WeatherManager, WeatherType, WeatherAffected, RainCollector, WindResponsive};
pub use systems::movement::*;
pub use systems::camera::*;
pub use systems::interaction::*;
pub use systems::world::*;
pub use systems::effects::*;
#[cfg(feature = "weather")]
pub use systems::weather::*;
pub use plugins::*;
pub use factories::*;
pub use setup::{setup_basic_world, setup_basic_vehicles, setup_helicopter, setup_f16, setup_lod_vehicles, setup_lod_helicopter, setup_lod_f16, setup_palm_trees, setup_luxury_cars, setup_npcs, setup_buildings, setup_starter_vehicles};
#[cfg(feature = "weather")]
pub use setup::{setup_weather_components, setup_weather_materials, setup_weather_ui, setup_weather_environment, update_weather_ui};
pub use constants::*;
pub use config::*;
pub use game_state::*;
