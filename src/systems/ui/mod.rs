//! ───────────────────────────────────────────────
//! System:   Mod
//! Purpose:  Handles user interface display and interaction
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

pub mod fps_display;
pub mod controls_ui;
pub mod bugatti_telemetry;

pub use fps_display::*;
pub use controls_ui::*;
pub use bugatti_telemetry::*;
