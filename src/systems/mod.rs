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
//! - `persistence`: Save/load functionality
//! - `transform_sync`: Transform synchronization
//! - `rendering`: Render pipeline customization
//!
//! ## System Execution Order
//!
//! Systems use the sets defined in `crate::system_sets`:
//! - ServiceInit → WorldSetup → SecondarySetup → ServiceUpdates
//!
//! Use `.in_set()` to control when your system runs relative to others.

pub mod movement;
pub mod world;
pub mod interaction;
pub mod camera;
pub mod effects;
pub mod audio;
pub mod human_behavior;
pub mod physics_utils;

pub mod vehicles;
pub mod ui;
pub mod water;
pub mod debug;
// pub mod timing_service; // Moved to services/
pub mod spawn_validation;
pub mod safety;
pub mod input;
pub mod persistence;
// pub mod realistic_physics_safeguards; // DISABLED - conflicts with Rapier
// pub mod distance_cache; // Moved to services/
pub mod unified_distance_calculator;
pub mod transform_sync;
pub mod distance_cache_debug;
// pub mod batching; // Missing file
// pub mod batching_test; // Missing file

pub mod simple_service_example;
pub mod rendering;
pub mod vegetation_instancing_integration;
pub mod player_collision_resolution;

pub mod parallel_physics;

pub mod performance_monitor;
pub mod config;
// pub mod performance_integration; // Temporarily disabled - depends on deleted batching system
pub mod batching;
// Explicit exports following simplicity guidelines - only export what's needed
// Core systems that are commonly used across plugins
pub use crate::services::{DistanceCache, get_cached_distance, get_cached_distance_squared, MovementTracker, DistanceCachePlugin};
pub use crate::services::{TimingService, SystemType, EntityTimerType, ManagedTiming, TimingStats};
pub use spawn_validation::{SpawnRegistry, SpawnValidator, SpawnableType, SpawnValidationPlugin};
pub use safety::{world_bounds_safety_system, position_monitor_system};
pub use batching::frame_counter_system;
pub use performance_monitor::{UnifiedPerformanceTracker, PerformanceCategory, UnifiedPerformancePlugin};
pub use distance_cache_debug::DistanceCacheDebugPlugin;
pub use transform_sync::TransformSyncPlugin;
pub use unified_distance_calculator::UnifiedDistanceCalculatorPlugin;

// World systems (frequently used together)
pub use world::{
    road_network::{RoadNetwork, RoadSpline, RoadType, IntersectionType},
    road_generation::is_on_road_spline,
    unified_distance_culling::UnifiedCullable,
};

// Input system types
pub use input::{InputManager, InputConfig, InputAction};

// Physics utilities
pub use physics_utils::{PhysicsUtilities, InputProcessor, MovementInputs, CollisionGroupHelper, PhysicsBodySetup, apply_universal_physics_safeguards};

// Simple service examples
pub use simple_service_example::{
    service_example_vehicle_creation, service_example_config_validation, 
    service_example_timing_check
};

