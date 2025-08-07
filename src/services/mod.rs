pub mod traits;
pub mod simple_services;
pub mod timing_service;
pub mod distance_cache;
pub mod entity_limits;

pub mod ground_detection;


pub use simple_services::{initialize_simple_services, update_timing_service_system};
pub use timing_service::{TimingService, SystemType, EntityTimerType, ManagedTiming, TimingStats};
pub use distance_cache::{DistanceCache, get_cached_distance, get_cached_distance_squared, MovementTracker, DistanceCachePlugin};
// pub use entity_limits::EntityLimitManager; // Available but not used yet

pub use ground_detection::GroundDetectionPlugin;
