pub mod movement;
pub mod world;
pub mod interaction;
pub mod camera;
pub mod effects;
pub mod audio;
pub mod config_loader;
pub mod human_behavior;
pub mod physics_utils;
pub mod lod;

pub mod vehicles;
pub mod ui;
pub mod water;
pub mod debug;
pub mod timing_service;
pub mod spawn_validation;
pub mod input;
pub mod persistence;
// pub mod realistic_physics_safeguards; // DISABLED - conflicts with Rapier
pub mod distance_cache;
pub mod unified_distance_calculator;
pub mod transform_sync;
pub mod distance_cache_debug;
pub mod batching;
pub mod batching_test;
pub mod entity_creation_system;
pub mod simple_service_example;
pub mod rendering;
pub mod vegetation_instancing_integration;
pub mod player_collision_resolution;

pub mod parallel_physics;
pub mod performance_dashboard;
pub mod performance_monitor;
pub mod performance_integration;
pub mod batch_processing;

// Core system plugins (main API surface)
pub use spawn_validation::SpawnValidationPlugin;
pub use distance_cache::DistanceCachePlugin;
pub use distance_cache_debug::DistanceCacheDebugPlugin;
pub use transform_sync::TransformSyncPlugin;
pub use unified_distance_calculator::UnifiedDistanceCalculatorPlugin;
pub use performance_monitor::UnifiedPerformancePlugin;
pub use performance_integration::PerformanceIntegrationPlugin;

// System functions used by main.rs
pub use simple_service_example::{
    service_example_vehicle_creation,
    service_example_config_validation,
    service_example_timing_check,
};

// Physics utilities (used by examples)
pub use physics_utils::{PhysicsUtilities, CollisionGroupHelper, PhysicsBodySetup, InputProcessor};

// Essential components
pub use distance_cache::MovementTracker;
// Essential batching systems (non-conflicting with batch_processing.rs)
pub use batching::{
    frame_counter_system,
    mark_transform_dirty_system,
    mark_visibility_dirty_system,
    mark_physics_dirty_system,
    batch_transform_processing_system,
    batch_physics_processing_system,
    batch_lod_processing_system,
    batch_culling_system,
    dirty_flag_cleanup_system,
    dirty_flags_metrics_system,
    // NOTE: batch_mark_vegetation_instancing_dirty_system renamed to avoid conflict
};
// Keep only essential batching systems from these modules
pub use batching_test::{
    batching_test_system, 
    batching_stress_test_system,
    batching_performance_comparison_system,
    cleanup_test_entities_system,
};
pub use entity_creation_system::{
    bevy_resource_entity_creation_system,
    bevy_resource_config_update_system,
    bevy_asset_cleanup_system,
    bevy_resource_factory_system,
};
pub use rendering::vegetation_instancing::{
    collect_vegetation_instances_system,
    update_vegetation_instancing_system,
};
// VegetationInstancingConfig is available through components::VegetationInstancingConfig
pub use player_collision_resolution::{
    player_collision_resolution_system,
    player_movement_validation_system,
};

