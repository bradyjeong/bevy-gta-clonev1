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

pub mod lod_manager;

pub use lod_manager::*;
