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

pub mod jet_flames;

pub use jet_flames::*;
