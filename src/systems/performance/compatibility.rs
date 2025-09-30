/// Compatibility stubs for the old performance system
/// These allow code that depends on the old system to still compile
/// while we transition to the new simplified system
use bevy::prelude::*;

/// Stub enum for performance categories - now unused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceCategory {
    Physics,
    Rendering,
    Culling,
    Input,
    Audio,
    Spawning,
    LOD,
    Batching,
    Transform,
    UI,
    Network,
    System,
}

/// Stub resource to replace UnifiedPerformanceTracker
/// This is a minimal implementation that does nothing
#[derive(Resource, Default)]
pub struct UnifiedPerformanceTracker {
    pub enabled: bool,
}

impl UnifiedPerformanceTracker {
    /// Stub method - does nothing
    pub fn record_category_time(&mut self, _category: PerformanceCategory, _time_ms: f32) {}

    /// Stub method - does nothing
    pub fn record_system_time(&mut self, _system_name: &str, _time_ms: f32) {}

    /// Stub method - does nothing
    pub fn update_cache_stats(&mut self, _hits: usize, _misses: usize, _cache_type: &str) {}
}

/// Compatibility plugin that provides the old resource as a stub
pub struct UnifiedPerformancePlugin;

impl Plugin for UnifiedPerformancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnifiedPerformanceTracker>();
    }
}
