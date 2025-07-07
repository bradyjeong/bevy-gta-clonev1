//! ───────────────────────────────────────────────
//! System:   Mod
//! Purpose:  Handles audio playback and effects
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * System maintains consistent state
//! Owner:    @render-team
//! ───────────────────────────────────────────────

pub mod realistic_vehicle_audio;

pub use realistic_vehicle_audio::*;
