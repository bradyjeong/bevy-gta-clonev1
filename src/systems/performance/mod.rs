//! Simplified performance monitoring using Bevy's built-in diagnostics
//!
//! Replaces the complex 780-line UnifiedPerformanceTracker with a minimal system
//! that leverages Bevy's FrameTimeDiagnosticsPlugin and LogDiagnosticsPlugin.

pub mod compatibility;
pub mod simple;

// Export the simple implementation
pub use simple::{DebugUIPlugin, PerformancePlugin};

// Re-export compatibility stubs to maintain backward compatibility
pub use compatibility::{PerformanceCategory, UnifiedPerformancePlugin, UnifiedPerformanceTracker};
