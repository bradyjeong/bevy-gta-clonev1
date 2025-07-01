pub mod movement;
pub mod world;
pub mod interaction;
pub mod camera;
pub mod effects;
pub mod audio;
pub mod human_behavior;


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

pub use movement::*;
pub use parallel_physics::*;
pub use performance_dashboard::*;
pub use world::*;
pub use interaction::*;
pub use camera::*;
pub use effects::*;
pub use audio::*;
pub use human_behavior::*;


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
pub use distance_cache_debug::*;
pub use transform_sync::*;
pub use batching::*;
pub use batching_test::*;
pub use entity_creation_system::*;
pub use simple_service_example::*;
pub use rendering::*;
pub use vegetation_instancing_integration::*;
pub use player_collision_resolution::*;

