//! ───────────────────────────────────────────────
//! System:   Mod
//! Purpose:  Manages level-of-detail based on distance
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @render-team
//! ───────────────────────────────────────────────

pub mod modern_lod_system;

pub use modern_lod_system::*;
