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

pub mod save_system;
pub mod load_system;

pub use save_system::*;
pub use load_system::*;
