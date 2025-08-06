pub mod traits;
pub mod simple_services;
pub mod simple_container;
pub mod ground_detection;

pub use traits::{Service, TimingService as TimingServiceTrait, ConfigService as ConfigServiceTrait, PhysicsService as PhysicsServiceTrait};
pub use simple_services::{ConfigService, PhysicsService, EnhancedTimingService, initialize_simple_services, update_timing_service_system};
pub use simple_container::{SimpleServiceContainer, SimpleServices};
pub use ground_detection::{GroundDetectionService, GroundDetectionPlugin};
