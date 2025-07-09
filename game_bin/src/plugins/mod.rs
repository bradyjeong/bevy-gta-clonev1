// Game binary specific plugins that exist locally
pub mod game_plugin;
pub mod batching_plugin;

// Plugin exports - these are the main API surface
pub use game_plugin::GamePlugin;

