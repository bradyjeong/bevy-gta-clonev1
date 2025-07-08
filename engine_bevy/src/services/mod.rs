//! Service abstractions for game engine

/// Simple service implementations for basic game functionality
pub mod simple_services_v2;
/// Timing service for system scheduling
pub mod timing_service;
/// Performance monitoring service
pub mod performance_service;

pub use simple_services_v2::*;
pub use timing_service::BevyTimingService;
pub use performance_service::*;
