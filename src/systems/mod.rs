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
pub mod validation;
// pub mod realistic_physics_safeguards; // DISABLED - conflicts with Rapier
// pub mod distance_cache; // Moved to services/
pub mod distance_cache_debug;
pub mod transform_sync;
pub mod unified_distance_calculator;
// pub mod batching; // Missing file
// pub mod batching_test; // Missing file

pub mod player_collision_resolution;
pub mod rendering;
pub mod simple_service_example;
pub mod vegetation_instancing_integration;

pub mod safe_active_entity;
// pub mod floating_origin; - REMOVED: Finite world doesn't need floating origin

pub mod batching;
pub mod performance; // Simplified performance system (replaces performance_monitor)
// Explicit exports following simplicity guidelines - only export what's needed
// Core systems that are commonly used across plugins
pub use crate::services::{
    DistanceCache, DistanceCachePlugin, MovementTracker, get_cached_distance,
    get_cached_distance_squared,
};
pub use crate::services::{EntityTimerType, ManagedTiming, SystemType, TimingService, TimingStats};
pub use batching::frame_counter_system;
pub use distance_cache_debug::DistanceCacheDebugPlugin;
// Simplified performance system
pub use performance::{
    DebugUIPlugin, PerformanceCategory, PerformancePlugin, UnifiedPerformancePlugin,
    UnifiedPerformanceTracker,
};
pub use safety::validate_physics_config;
pub use spawn_validation::{SpawnRegistry, SpawnValidationPlugin, SpawnValidator, SpawnableType};
pub use transform_sync::TransformSyncPlugin;
pub use unified_distance_calculator::UnifiedDistanceCalculatorPlugin;

// World systems (frequently used together)
pub use world::{
    road_generation::is_on_road_spline,
    road_network::{IntersectionType, RoadNetwork, RoadSpline, RoadType},
    unified_world::{
        ChunkCoord, ChunkState, ContentLayer, UnifiedChunkEntity, UnifiedWorldManager,
    },
};

// Asset-based input system types only
pub use input::{LoadedVehicleControls, VehicleControlsConfig};

// Physics utilities
pub use physics::PhysicsUtilities;

// Safe ActiveEntity system
pub use safe_active_entity::{
    ActiveEntityTransferred, ActiveTransferRequest, active_entity_integrity_check,
    active_transfer_executor_system, queue_active_transfer,
};

// Validation systems
pub use validation::{ColliderType, MeshColliderConfig, validate_vehicle_consistency};

// Floating origin system - REMOVED: Finite world doesn't need coordinate translation

// World boundary systems
pub use world::boundaries::{WorldBounds, aircraft_boundary_system, world_boundary_system};

// Simple service examples
pub use simple_service_example::{
    service_example_config_validation, service_example_timing_check,
    service_example_vehicle_creation,
};
