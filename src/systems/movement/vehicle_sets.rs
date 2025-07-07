//! ───────────────────────────────────────────────
//! System:   Vehicle Sets
//! Purpose:  Handles audio playback and effects
//! Schedule: Update
//! Reads:    System components
//! Writes:   System state
//! Invariants:
//!   * Physics values are validated and finite
//! Owner:    @simulation-team
//! ───────────────────────────────────────────────

use bevy::prelude::*;

/// System sets for vehicle processing order
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VehicleSet {
    /// Input handling systems (read controls, convert to component data)
    Input,
    /// Physics systems (apply forces, handle suspension, aerodynamics)
    Physics,
    /// Audio systems (engine sounds, effects)
    Audio,
    /// Visual effects systems (particles, lighting)
    Effects,
    /// Performance monitoring and metrics
    Performance,
}

/// Configure vehicle system execution order
pub fn configure_vehicle_system_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            VehicleSet::Input,
            VehicleSet::Physics,
            VehicleSet::Audio,
            VehicleSet::Effects,
            VehicleSet::Performance,
        ).chain()
    );
}
