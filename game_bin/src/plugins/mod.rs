// Re-export plugins via compat layer
pub use gameplay_sim::plugins::*;

// Game binary specific plugins that exist locally
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

