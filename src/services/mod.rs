pub mod distance_cache;
pub mod entity_limits;
pub mod simple_services;
pub mod timing_service;

pub mod ground_detection;
pub mod terrain_service;

pub use distance_cache::{
    DistanceCache, DistanceCachePlugin, MovementTracker, get_cached_distance,
    get_cached_distance_squared,
};
pub use simple_services::{initialize_simple_services, update_timing_service_system};
pub use timing_service::{EntityTimerType, ManagedTiming, SystemType, TimingService, TimingStats};
// pub use entity_limits::EntityLimitManager; // Available but not used yet

pub use ground_detection::GroundDetectionPlugin;
