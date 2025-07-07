//! ───────────────────────────────────────────────
//! System:   Mod
//! Purpose:  Processes user input and control mapping
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

pub mod input_config;
pub mod input_manager;
pub mod vehicle_control_config;
pub mod control_manager;

pub use input_config::*;
pub use input_manager::*;
pub use vehicle_control_config::*;
pub use control_manager::*;
