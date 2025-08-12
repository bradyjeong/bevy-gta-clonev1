//! Debug utilities and instrumentation
//! 
//! Feature-gated debug tools for performance monitoring and system analysis

#[cfg(feature = "event-audit")]
pub mod event_audit;

#[cfg(feature = "event-audit")]
pub use event_audit::EventAuditPlugin;

// Size audit modules for component and resource optimization
pub mod size_audit;
pub mod size_measurements;
pub mod component_size_audit;
pub mod size_optimization;

pub use size_audit::{SizeAuditPlugin, SizeAuditReport};
