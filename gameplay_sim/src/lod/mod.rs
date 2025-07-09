//! ───────────────────────────────────────────────
//! System:   Mod
//! Purpose:  Manages level-of-detail based on distance
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

pub mod modern_lod_system;

pub use modern_lod_system::{
    modern_lod_system, 
    lod_performance_monitoring_system,
    ModernLODPlugin,
    LodSystemSet,
};

