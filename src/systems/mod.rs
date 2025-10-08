//! # System Organization
//!
//! This module contains all game systems organized by functional domain.
//! Systems are pure functions that operate on components and resources.
//!
//! ## System Design Principles
//!
//! - **Pure Functions**: Systems take components/resources as input, produce side effects
//! - **Single Responsibility**: Each system handles one specific concern
//! - **No Direct Calls**: Systems communicate via events and shared resources
//! - **Deterministic**: Same inputs always produce same outputs
//!
//! ## System Categories
//!
//! ### Core Gameplay
//! - `movement`: Player and entity movement mechanics
//! - `physics_utils`: Physics simulation and collision handling  
//! - `interaction`: Object interaction and pickup systems
//! - `vehicles`: Vehicle physics, spawning, and AI
//!
//! ### World Management
//! - `world`: Terrain generation and world structure
//! - `water`: Water simulation and rendering
//! - `vegetation_instancing_integration`: Plant and tree systems
//! - `spawn_validation`: Entity spawning rules and limits
//!
//! ### Services
//! - `distance_cache`: Optimized distance calculations
//! - `timing_service`: Frame timing and performance tracking
//! - `performance_monitor`: System performance analysis
//! - `unified_distance_calculator`: Centralized distance management
//!
//! ### Interface & Feedback
//! - `ui`: User interface systems
//! - `camera`: Camera control and positioning
//! - `input`: Input processing and mapping
//! - `audio`: Sound effects and music
//! - `effects`: Visual effects and particles
//!
//! ### Utility Systems
//! - `debug`: Development and debugging tools
//! - `transform_sync`: Transform synchronization
//! - `rendering`: Render pipeline customization
//!
//! ## System Execution Order
//!
//! Systems use the sets defined in `crate::system_sets`:
//! - ServiceInit → WorldSetup → SecondarySetup → ServiceUpdates
//!
//! Use `.in_set()` to control when your system runs relative to others.

pub mod audio;
pub mod camera;
pub mod effects;

pub mod interaction;
pub mod movement;
pub mod world;

pub mod physics;
pub mod setup;

pub mod debug;
pub mod ui;
pub mod vehicles;
pub mod water;
// pub mod timing_service; // Moved to services/
pub mod input;
pub mod safety;
pub mod spawn_validation;
pub mod swimming;
pub mod terrain_water_manager;
pub mod validation;
// pub mod realistic_physics_safeguards; // DISABLED - conflicts with Rapier
pub mod transform_sync;
// pub mod batching; // Missing file
// pub mod batching_test; // Missing file

pub mod player_collision_resolution;
pub mod player_physics_enable;
pub mod safe_active_entity;
// pub mod floating_origin; - REMOVED: Finite world doesn't need floating origin

pub mod performance; // Simplified performance system (replaces performance_monitor)

// MINIMAL CURATED EXPORTS - Use explicit module paths elsewhere to maintain clear dependencies
// Only export items that are genuinely shared across multiple plugins and form stable APIs

// Plugins that must be registered in main.rs or other top-level configs
pub use performance::UnifiedPerformancePlugin;
pub use spawn_validation::SpawnValidationPlugin;
pub use transform_sync::TransformSyncPlugin;
