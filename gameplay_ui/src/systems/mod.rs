//! UI systems module

pub mod debug;
pub mod performance_dashboard;
pub mod performance_integration;
pub mod performance_monitor;
pub mod ui;

// ----------------------------------------------------------------
//  Forwarders to gameplay_sim
// ----------------------------------------------------------------
pub mod distance_cache {
    /// Re-export
    pub use gameplay_sim::systems::distance_cache::*;
}

pub mod input {
    // Temporary re-export for compatibility - temporarily disabled for Phase 5
    // pub use gameplay_sim::input::*;
    // pub use gameplay_sim::systems::input::*;
}

/// Temporary re-export for compatibility
pub mod unified_distance_calculator {
    pub use gameplay_sim::systems::unified_distance_calculator::*;
}
