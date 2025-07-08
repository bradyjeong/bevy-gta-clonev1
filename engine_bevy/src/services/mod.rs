//! Service abstractions for game engine

#[allow(missing_docs)]
pub mod simple_services_v2;
#[allow(missing_docs)]
pub mod timing_service;
#[allow(missing_docs)]
pub mod performance_service;

pub use simple_services_v2::*;
pub use timing_service::BevyTimingService;
pub use performance_service::*;
