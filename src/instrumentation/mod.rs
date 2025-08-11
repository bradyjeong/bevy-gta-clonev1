// Event instrumentation and system profiling for production debugging
// Zero-cost when debug-events feature is disabled

#[cfg(feature = "debug-events")]
pub mod event_metrics;

#[cfg(feature = "debug-events")]
pub mod system_profiling;

#[cfg(feature = "debug-events")]
pub mod schedule_ordering;

#[cfg(feature = "debug-events")]
pub use event_metrics::{EventMetrics, EventStats, EventMetricsPlugin};

#[cfg(feature = "debug-events")]
pub use system_profiling::{SystemMetrics, SystemProfiler};

#[cfg(feature = "debug-events")]
pub use schedule_ordering::{ScheduleOrdering, ScheduleOrderingPlugin, OrderedSystem};

// No-op exports when feature is disabled
#[cfg(not(feature = "debug-events"))]
pub struct EventMetricsPlugin;

#[cfg(not(feature = "debug-events"))]
impl bevy::app::Plugin for EventMetricsPlugin {
    fn build(&self, _app: &mut bevy::app::App) {}
}

#[cfg(not(feature = "debug-events"))]
pub struct ScheduleOrderingPlugin;

#[cfg(not(feature = "debug-events"))]
impl bevy::app::Plugin for ScheduleOrderingPlugin {
    fn build(&self, _app: &mut bevy::app::App) {}
}
