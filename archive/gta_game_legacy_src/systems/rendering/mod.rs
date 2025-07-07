//! ───────────────────────────────────────────────
//! System:   Mod
//! Purpose:  System functionality
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

pub mod vegetation_instancing;
pub mod render_optimizer_simple;

pub use vegetation_instancing::{
    collect_vegetation_instances_system,
    update_vegetation_instancing_system,
    mark_vegetation_instancing_dirty_system,
    animate_vegetation_instances_system,
    vegetation_instancing_metrics_system,
};
pub use render_optimizer_simple::*;
