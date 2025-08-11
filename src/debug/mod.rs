//! Debug utilities and instrumentation
//! 
//! Feature-gated debug tools for performance monitoring and system analysis

#[cfg(feature = "event-audit")]
pub mod event_audit;

#[cfg(feature = "event-audit")]
pub use event_audit::EventAuditPlugin;
