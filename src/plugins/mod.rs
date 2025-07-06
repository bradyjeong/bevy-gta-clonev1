pub mod player_plugin;
pub mod vehicle_plugin;
pub mod unified_world_plugin;
pub mod ui_plugin;
pub mod water_plugin;
pub mod persistence_plugin;
pub mod input_plugin;
pub mod vegetation_lod_plugin;
pub mod game_plugin;

pub mod batching_plugin;

// Plugin exports - these are the main API surface
pub use player_plugin::PlayerPlugin;
pub use vehicle_plugin::VehiclePlugin;
pub use unified_world_plugin::UnifiedWorldPlugin;
pub use ui_plugin::UIPlugin;
pub use water_plugin::WaterPlugin;
pub use persistence_plugin::PersistencePlugin;
pub use input_plugin::InputPlugin;
pub use vegetation_lod_plugin::VegetationLODPlugin;
pub use game_plugin::GamePlugin;

pub use batching_plugin::BatchingPlugin;

