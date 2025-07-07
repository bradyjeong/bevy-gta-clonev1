#![deny(clippy::all, clippy::pedantic)]

pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod systems;
pub(crate) mod plugins;
pub(crate) mod setup;
pub(crate) mod constants;
pub(crate) mod game_state;
pub(crate) mod bundles;
pub(crate) mod factories;
pub(crate) mod services;
pub(crate) mod serialization;

// Engine-level abstractions only - minimal public surface
pub use game_state::GameState;
pub use plugins::GamePlugin;
pub use setup::setup_basic_world;
pub use serialization::WorldSerializer;
