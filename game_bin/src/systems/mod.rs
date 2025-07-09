// Re-export all systems from gameplay_sim + compat stubs
pub use gameplay_sim::systems::*;

// Game binary specific systems that exist locally
pub mod camera;
pub mod effects;
pub mod audio;
pub mod config_loader;
pub mod debug;
pub mod timing_service;
pub mod performance_dashboard;
pub mod performance_monitor;
pub mod performance_integration;
pub mod batch_processing;
pub mod distance_cache_debug;
pub mod simple_service_example;
pub mod batching_test;
pub mod rendering;
pub mod ui;

// Core system plugins (main API surface)
// (Only export the ones actually used in main.rs)

// System functions used by main.rs  
// (Only export the ones actually used in main.rs)

// Physics utilities (used by examples)
// (Only export the ones actually used in examples)

// Essential components

// Essential batching systems (non-conflicting with batch_processing.rs)
pub use gameplay_sim::systems::batching::frame_counter_system;

