//! # Plugin Architecture
//!
//! This module contains the game's plugin-based architecture. Each plugin is a
//! self-contained module that manages specific game functionality.
//!
//! ## Plugin Design Principles
//!
//! - **Single Responsibility**: Each plugin handles one clear domain
//! - **Resource & Direct Communication**: Plugins communicate primarily via resources and direct system APIs; events only when decoupling is necessary
//! - **Resource Sharing**: Shared state uses Bevy resources, not globals
//! - **Clear Boundaries**: Minimal cross-plugin dependencies
//!
//! ## Plugin Categories
//!
//! ### Core Plugins
//! - `game_core`: Essential game systems and state management
//! - `game_setup`: Initial world setup and configuration
//!
//! ### Gameplay Plugins
//! - `player_plugin`: Player character control and state
//! - `vehicle_plugin`: Vehicle physics and spawning
//! - `unified_world_plugin`: World generation and terrain
//! - `vegetation_lod_plugin`: Vegetation rendering and LOD
//! - `water_plugin`: Water simulation and rendering
//!
//! ### Interface Plugins
//! - `ui_plugin`: User interface and HUD
//! - `input_plugin`: Input handling and mapping
//!
//! ### Utility Plugins
//!
//! ## Event Flow Architecture
//!
//! ```text
//! Input Events → Player Plugin → Game State Events
//!       ↓              ↓              ↓
//! UI Plugin ← Vehicle Plugin → World Plugin
//!       ↓              ↓              ↓
//! Audio Effects → Vegetation
//! ```
//!
//! ## Adding New Plugins
//!
//! 1. Create new plugin file in this directory
//! 2. Implement the `Plugin` trait
//! 3. Use system sets from `crate::system_sets` for ordering
//! 4. Communicate via events, not direct calls
//! 5. Add to this mod.rs file

pub mod game_core;
pub mod game_setup;
pub mod input_plugin;
pub mod inspector_plugin;
pub mod map_plugin;
pub mod particle_plugin;
pub mod player_plugin;
pub mod ui_plugin;
pub mod underwater_plugin;
pub mod unified_world_plugin;
pub mod vehicle_plugin;
pub mod water_plugin;

// New focused world plugins
pub mod physics_activation_plugin;
pub mod static_world_generation_plugin;
pub mod world_debug_plugin;
pub mod world_npc_plugin;

// Core game plugins
pub use game_core::GameCorePlugin;
pub use game_setup::GameSetupPlugin;
pub use input_plugin::InputPlugin;
pub use inspector_plugin::InspectorPlugin;
pub use map_plugin::MapPlugin;
pub use particle_plugin::ParticlePlugin;
pub use player_plugin::PlayerPlugin;
pub use ui_plugin::UIPlugin;
pub use underwater_plugin::UnderwaterPlugin;

// World and rendering plugins
pub use unified_world_plugin::UnifiedWorldPlugin;

pub use vehicle_plugin::VehiclePlugin;
pub use water_plugin::WaterPlugin;

// Specialized world plugins
pub use physics_activation_plugin::PhysicsActivationPlugin;
pub use static_world_generation_plugin::StaticWorldGenerationPlugin;
pub use world_debug_plugin::WorldDebugPlugin;
pub use world_npc_plugin::WorldNpcPlugin;
