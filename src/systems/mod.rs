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

pub use movement::*;
pub use parallel_physics::*;
pub use performance_dashboard::*;
pub use performance_monitor::*;
pub use performance_integration::*;
pub use batch_processing::*;
pub use world::*;
pub use interaction::*;
pub use camera::*;
pub use effects::*;
pub use audio::*;
pub use human_behavior::*;
pub use physics_utils::*;


pub use vehicles::*;
pub use ui::*;
pub use water::*;
pub use debug::*;
pub use timing_service::*;
pub use spawn_validation::*;
pub use input::*;
pub use persistence::*;
// pub use realistic_physics_safeguards::*; // DISABLED - conflicts with Rapier
pub use distance_cache::*;
pub use unified_distance_calculator::*;
pub use distance_cache_debug::*;
pub use transform_sync::*;
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
pub use batching_test::*;
pub use entity_creation_system::*;
pub use simple_service_example::*;
pub use rendering::*;
pub use vegetation_instancing_integration::*;
pub use player_collision_resolution::*;

