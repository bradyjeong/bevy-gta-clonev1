//! # Plugin Architecture
//!
//! This module contains the game's plugin-based architecture. Each plugin is a
//! self-contained module that manages specific game functionality.
//!
//! ## Plugin Design Principles
//!
//! - **Single Responsibility**: Each plugin handles one clear domain
//! - **Event-Based Communication**: Plugins communicate via Bevy events only
//! - **Resource Sharing**: Shared state uses Bevy resources, not globals
//! - **Clear Boundaries**: No direct plugin-to-plugin dependencies
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
//! - `persistence_plugin`: Save/load game state
//!
//! ## Event Flow Architecture
//!
//! ```text
//! Input Events → Player Plugin → Game State Events
//!       ↓              ↓              ↓
//! UI Plugin ← Vehicle Plugin → World Plugin
//!       ↓              ↓              ↓
//! Persistence ← Audio Effects → Vegetation
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
pub mod player_plugin;
pub mod vehicle_plugin;
pub mod unified_world_plugin;
pub mod ui_plugin;
pub mod water_plugin;
pub mod persistence_plugin;
pub mod input_plugin;
pub mod vegetation_lod_plugin;

// New focused world plugins
pub mod world_streaming_plugin;
pub mod world_content_plugin;
pub mod world_npc_plugin;
pub mod world_debug_plugin;
pub mod timing_plugin;

pub use game_core::*;
pub use game_setup::*;
pub use player_plugin::*;
pub use vehicle_plugin::*;
pub use unified_world_plugin::*;
pub use ui_plugin::*;
pub use water_plugin::*;
pub use persistence_plugin::*;
pub use input_plugin::*;
pub use vegetation_lod_plugin::*;

// New focused world plugins
pub use world_streaming_plugin::*;
pub use world_content_plugin::*;
pub use world_npc_plugin::*;
pub use world_debug_plugin::*;
pub use timing_plugin::*;

